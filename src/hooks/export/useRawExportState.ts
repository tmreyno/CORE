// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useRawExportState — Raw disk image (.dd/.img) creation state and handler.
 */

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { createRawImage, buildRawExportOptions } from "../../api/rawExport";
import { formatBytes } from "../../api/archiveCreate";
import { getErrorMessage } from "../../utils/errorUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("RawExport");
import { joinPath } from "../../utils/pathUtils";
import {
  createActivity,
  updateProgress,
  completeActivity,
  failActivity,
} from "../../types/activity";
import type { ExportToast, ExportActivityCallbacks } from "./types";
import type { ExportCommonState } from "./useExportCommon";
import { handleAcquisitionComplete, startAcquisitionRecord } from "./companionHelper";
import { dbSync } from "../project/useProjectDbSync";
import type { DbExportRecord } from "../../types/projectDb";

export interface UseRawExportStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
  caseNumber?: string;
  examinerName?: string;
  systemStats?: { hostname?: string; systemModel?: string; systemSerialNumber?: string; systemManufacturer?: string; osName?: string; osVersion?: string } | null;
}

export function useRawExportState(options: UseRawExportStateOptions) {
  const { toast, common } = options;

  // === Raw Export State ===
  const [rawVerifyAfterWrite, setRawVerifyAfterWrite] = createSignal(true);
  const [rawComputeMd5, setRawComputeMd5] = createSignal(true);
  const [rawComputeSha1, setRawComputeSha1] = createSignal(false);
  const [rawComputeSha256, setRawComputeSha256] = createSignal(true);
  const [rawSegmentSize, setRawSegmentSize] = createSignal(2048);
  const [rawImageName, setRawImageName] = createSignal("evidence");
  const [rawCaseNumber, setRawCaseNumber] = createSignal("");
  const [rawEvidenceNumber, setRawEvidenceNumber] = createSignal("");
  const [rawExaminerName, setRawExaminerName] = createSignal("");
  const [rawDescription, setRawDescription] = createSignal("");
  const [rawNotes, setRawNotes] = createSignal("");

  // ─── Handler ────────────────────────────────────────────────────────────

  const handleCreateRawImage = async () => {
    log.info(`Starting Raw image: ${rawImageName()}.dd, MD5=${rawComputeMd5()}, SHA1=${rawComputeSha1()}, SHA256=${rawComputeSha256()}, sources=${common.sources().length}`);
    common.setIsProcessing(true);
    common.setIsAcquiring(true);

    const outputPath = joinPath(common.destination(), rawImageName());
    const shouldRestoreMounts = common.hasDriveSources() && common.mountDrivesReadOnly();

    const activity = createActivity("export", outputPath, common.sources().length, {
      operation: "Raw Disk Image Creation (.dd)",
    });

    options.onActivityCreate?.(activity);

    try {
      const rawOptions = buildRawExportOptions({
        sourcePaths: common.sources(),
        outputPath,
        computeMd5: rawComputeMd5(),
        computeSha1: rawComputeSha1(),
        computeSha256: rawComputeSha256(),
        caseNumber: rawCaseNumber() || undefined,
        evidenceNumber: rawEvidenceNumber() || undefined,
        examinerName: rawExaminerName() || undefined,
        description: rawDescription() || undefined,
        notes: rawNotes() || undefined,
      });

      if (rawSegmentSize() > 0) {
        rawOptions.segmentSize = rawSegmentSize() * 1024 * 1024;
      }

      const acquisitionStartedAt = new Date().toISOString();
      const capturedSources = [...common.sources()];

      // Track in DB
      const exportId = `raw-${Date.now()}`;
      const dbRecord: DbExportRecord = {
        id: exportId,
        exportType: "raw",
        sourcePathsJson: JSON.stringify(capturedSources),
        destination: common.destination(),
        status: "in_progress",
        startedAt: acquisitionStartedAt,
        initiatedBy: rawExaminerName() || "",
        totalFiles: 0,
        totalBytes: 0,
        encrypted: false,
        archiveFormat: "dd",
        optionsJson: JSON.stringify({
          computeMd5: rawComputeMd5(),
          computeSha1: rawComputeSha1(),
          computeSha256: rawComputeSha256(),
          segmentSize: rawSegmentSize(),
          verifyAfterWrite: rawVerifyAfterWrite(),
        }),
      };
      dbSync.insertExport(dbRecord);

      const acqRecord = startAcquisitionRecord({
        acquisitionType: "raw",
        outputPath,
        sources: capturedSources,
        caseNumber: options.caseNumber,
        examiner: options.examinerName,
        hostname: options.systemStats?.hostname,
        systemModel: options.systemStats?.systemModel,
        systemSerialNumber: options.systemStats?.systemSerialNumber,
        systemManufacturer: options.systemStats?.systemManufacturer,
      });

      const result = await createRawImage(rawOptions, (prog) => {
        options.onActivityUpdate?.(
          activity.id,
          updateProgress(activity, {
            bytesProcessed: prog.bytesWritten,
            bytesTotal: prog.totalBytes,
            percent: prog.percent,
            currentFile: prog.currentFile || undefined,
          }),
        );
      });

      // Post-write verification: re-read the image and compare hashes
      let verifyStatus = "";
      if (rawVerifyAfterWrite() && (result.sha256Hash || result.md5Hash)) {
        const algo = result.sha256Hash ? "SHA-256" : "MD5";
        const expected = result.sha256Hash || result.md5Hash;
        const t0 = performance.now();
        try {
          const computed = await invoke<string>("raw_verify", {
            inputPath: result.outputPath,
            algorithm: algo,
          });
          const verifyMs = (performance.now() - t0).toFixed(0);
          if (computed === expected) {
            verifyStatus = " | \u2713 Verified";
            log.info(`Raw verify passed in ${verifyMs}ms (${algo})`);
          } else {
            verifyStatus = " | \u2717 VERIFY FAILED";
            log.error(`Raw verify FAILED in ${verifyMs}ms — expected=${expected}, computed=${computed}`);
            toast.error("Verification Failed", "Written image hash does not match source data hash. Check disk integrity.");
          }
        } catch (verifyErr) {
          verifyStatus = " | \u26A0 Verify error";
          log.warn(`Raw verify error: ${getErrorMessage(verifyErr)}`);
        }
      }

      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      const hashInfo = result.sha256Hash
        ? ` | SHA-256: ${result.sha256Hash.substring(0, 16)}...`
        : result.md5Hash
          ? ` | MD5: ${result.md5Hash.substring(0, 16)}...`
          : "";
      const durationSec = (result.durationMs / 1000).toFixed(1);
      log.info(`Raw image created: ${formatBytes(result.bytesWritten)}, ${result.segmentsCreated} segment(s) in ${durationSec}s${verifyStatus}`);
      toast.success(
        "Raw Image Created",
        `Raw image created (${formatBytes(result.bytesWritten)}, ${result.segmentsCreated} segment${result.segmentsCreated !== 1 ? "s" : ""})${hashInfo}${verifyStatus}`,
      );
      options.onComplete?.(result.outputPath);

      dbSync.updateExport({
        ...dbRecord,
        status: "completed",
        completedAt: new Date().toISOString(),
        totalBytes: result.bytesWritten,
        totalFiles: result.filesIncluded,
        manifestHash: result.sha256Hash || result.md5Hash || result.sha1Hash || undefined,
      });

      const verifyResult = verifyStatus.includes("\u2713")
        ? "verified" as const
        : verifyStatus.includes("\u2717")
          ? "failed" as const
          : verifyStatus.includes("\u26A0")
            ? "error" as const
            : "skipped" as const;

      handleAcquisitionComplete({
        acquisitionType: "raw",
        outputPath: result.outputPath,
        sources: capturedSources,
        caseNumber: rawCaseNumber(),
        evidenceNumber: rawEvidenceNumber(),
        examiner: rawExaminerName(),
        description: rawDescription(),
        notes: rawNotes(),
        format: "dd",
        totalBytes: result.bytesWritten,
        totalFiles: result.filesIncluded,
        segmentSize: rawSegmentSize() > 0 ? rawSegmentSize() * 1024 * 1024 : 0,
        md5: result.md5Hash || undefined,
        sha1: result.sha1Hash || undefined,
        sha256: result.sha256Hash || undefined,
        startedAt: acquisitionStartedAt,
        completedAt: new Date().toISOString(),
        durationMs: result.durationMs,
        verifyResult,
        collectionId: acqRecord.collectionId,
        itemId: acqRecord.itemId,
        hostname: options.systemStats?.hostname,
        systemModel: options.systemStats?.systemModel,
        systemSerialNumber: options.systemStats?.systemSerialNumber,
        systemManufacturer: options.systemStats?.systemManufacturer,
        osName: options.systemStats?.osName,
        osVersion: options.systemStats?.osVersion,
      });
    } catch (error: unknown) {
      log.error(`Raw image creation failed: ${getErrorMessage(error)}`);
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Raw Image Creation Failed", getErrorMessage(error));
      dbSync.updateExport?.({
        ...dbRecord,
        status: "failed",
        completedAt: new Date().toISOString(),
        error: getErrorMessage(error),
      });
    } finally {
      common.setIsAcquiring(false);
      common.clearAllSources();
      setRawImageName("evidence");
      common.setIsProcessing(false);
      if (shouldRestoreMounts) {
        common.restoreAllDriveMounts();
      }
    }
  };

  // ─── Reset ──────────────────────────────────────────────────────────────

  const resetRawState = () => {
    setRawVerifyAfterWrite(true);
    setRawComputeMd5(true);
    setRawComputeSha1(false);
    setRawComputeSha256(true);
    setRawSegmentSize(2048);
    setRawImageName("evidence");
    setRawCaseNumber("");
    setRawEvidenceNumber("");
    setRawExaminerName("");
    setRawDescription("");
    setRawNotes("");
  };

  return {
    // Raw state
    rawVerifyAfterWrite,
    setRawVerifyAfterWrite,
    rawComputeMd5,
    setRawComputeMd5,
    rawComputeSha1,
    setRawComputeSha1,
    rawComputeSha256,
    setRawComputeSha256,
    rawSegmentSize,
    setRawSegmentSize,
    rawImageName,
    setRawImageName,
    rawCaseNumber,
    setRawCaseNumber,
    rawEvidenceNumber,
    setRawEvidenceNumber,
    rawExaminerName,
    setRawExaminerName,
    rawDescription,
    setRawDescription,
    rawNotes,
    setRawNotes,

    // Handler
    handleCreateRawImage,

    // Reset
    resetRawState,
  } as const;
}

export type RawExportState = ReturnType<typeof useRawExportState>;
