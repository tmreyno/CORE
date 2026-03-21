// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useEwfExportState — E01/EWF physical image creation state and handler.
 */

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { createE01Image, buildEwfExportOptions } from "../../api/ewfExport";
import { formatBytes } from "../../api/archiveCreate";
import { getErrorMessage } from "../../utils/errorUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("E01Export");
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

export interface UseEwfExportStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
  caseNumber?: string;
  examinerName?: string;
  systemStats?: { hostname?: string; systemModel?: string; systemSerialNumber?: string; systemManufacturer?: string; osName?: string; osVersion?: string } | null;
}

export function useEwfExportState(options: UseEwfExportStateOptions) {
  const { toast, common } = options;

  // === EWF/E01 Export State ===
  const [ewfVerifyAfterWrite, setEwfVerifyAfterWrite] = createSignal(true);
  const [ewfFormat, setEwfFormat] = createSignal("e01");
  const [ewfCompression, setEwfCompression] = createSignal("none");
  const [ewfCompressionMethod, setEwfCompressionMethod] = createSignal("deflate");
  const [ewfComputeMd5, setEwfComputeMd5] = createSignal(true);
  const [ewfComputeSha1, setEwfComputeSha1] = createSignal(false);
  const [ewfSegmentSize, setEwfSegmentSize] = createSignal(2048);
  const [ewfImageName, setEwfImageName] = createSignal("evidence");
  const [ewfCaseNumber, setEwfCaseNumber] = createSignal("");
  const [ewfEvidenceNumber, setEwfEvidenceNumber] = createSignal("");
  const [ewfExaminerName, setEwfExaminerName] = createSignal("");
  const [ewfDescription, setEwfDescription] = createSignal("");
  const [ewfNotes, setEwfNotes] = createSignal("");

  // ─── Handler ────────────────────────────────────────────────────────────

  const handleCreateE01Image = async () => {
    log.info(`Starting E01 creation: ${ewfImageName()}.E01, format=${ewfFormat()}, compression=${ewfCompression()}, sources=${common.sources().length}`);
    common.setIsProcessing(true);
    common.setIsAcquiring(true);

    const outputPath = joinPath(common.destination(), ewfImageName());
    const shouldRestoreMounts = common.hasDriveSources() && common.mountDrivesReadOnly();

    const activity = createActivity("export", outputPath, common.sources().length, {
      operation: `E01 Image Creation (${ewfFormat()}, ${ewfCompression()})`,
    });

    options.onActivityCreate?.(activity);

    try {
      const ewfOptions = buildEwfExportOptions({
        sourcePaths: common.sources(),
        outputPath,
        format: ewfFormat(),
        compression: ewfCompression(),
        compressionMethod: ewfCompressionMethod(),
        caseNumber: ewfCaseNumber() || undefined,
        evidenceNumber: ewfEvidenceNumber() || undefined,
        examinerName: ewfExaminerName() || undefined,
        description: ewfDescription() || undefined,
        notes: ewfNotes() || undefined,
        computeMd5: ewfComputeMd5(),
        computeSha1: ewfComputeSha1(),
      });

      if (ewfSegmentSize() > 0) {
        ewfOptions.segmentSize = ewfSegmentSize() * 1024 * 1024;
      }

      const acquisitionStartedAt = new Date().toISOString();
      const capturedSources = [...common.sources()];

      // Track in DB
      const exportId = `e01-${Date.now()}`;
      const dbRecord: DbExportRecord = {
        id: exportId,
        exportType: "e01",
        sourcePathsJson: JSON.stringify(capturedSources),
        destination: common.destination(),
        status: "in_progress",
        startedAt: acquisitionStartedAt,
        initiatedBy: ewfExaminerName() || "",
        totalFiles: 0,
        totalBytes: 0,
        encrypted: false,
        archiveFormat: ewfFormat(),
        compressionLevel: ewfCompression(),
        optionsJson: JSON.stringify({
          format: ewfFormat(),
          compression: ewfCompression(),
          computeMd5: ewfComputeMd5(),
          computeSha1: ewfComputeSha1(),
          segmentSize: ewfSegmentSize(),
          verifyAfterWrite: ewfVerifyAfterWrite(),
        }),
      };
      dbSync.insertExport(dbRecord);

      const acqRecord = startAcquisitionRecord({
        acquisitionType: "e01",
        outputPath,
        sources: capturedSources,
        caseNumber: options.caseNumber,
        examiner: options.examinerName,
        hostname: options.systemStats?.hostname,
        systemModel: options.systemStats?.systemModel,
        systemSerialNumber: options.systemStats?.systemSerialNumber,
        systemManufacturer: options.systemStats?.systemManufacturer,
      });

      const result = await createE01Image(ewfOptions, (prog) => {
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
      if (ewfVerifyAfterWrite() && (result.md5Hash || result.sha1Hash)) {
        const algo = result.md5Hash ? "MD5" : "SHA1";
        const expected = result.md5Hash || result.sha1Hash;
        const t0 = performance.now();
        try {
          const computed = await invoke<string>("e01_v3_verify", {
            inputPath: result.outputPath,
            algorithm: algo,
          });
          const verifyMs = (performance.now() - t0).toFixed(0);
          if (computed === expected) {
            verifyStatus = " | \u2713 Verified";
            log.info(`E01 verify passed in ${verifyMs}ms (${algo})`);
          } else {
            verifyStatus = " | \u2717 VERIFY FAILED";
            log.error(`E01 verify FAILED in ${verifyMs}ms — expected=${expected}, computed=${computed}`);
            toast.error("Verification Failed", "Written image hash does not match source data hash. Check disk integrity.");
          }
        } catch (verifyErr) {
          verifyStatus = " | \u26A0 Verify error";
          log.warn(`E01 verify error: ${getErrorMessage(verifyErr)}`);
        }
      }

      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      const hashInfo = result.md5Hash ? ` | MD5: ${result.md5Hash.substring(0, 16)}...` : "";
      const durationSec = (result.durationMs / 1000).toFixed(1);
      log.info(`E01 image created: ${formatBytes(result.bytesWritten)} in ${durationSec}s${verifyStatus}`);
      toast.success(
        "E01 Image Created",
        `${result.format} image created (${formatBytes(result.bytesWritten)})${hashInfo}${verifyStatus}`,
      );
      options.onComplete?.(result.outputPath);

      dbSync.updateExport({
        ...dbRecord,
        status: "completed",
        completedAt: new Date().toISOString(),
        totalBytes: result.bytesWritten,
        totalFiles: result.filesIncluded,
        manifestHash: result.md5Hash || result.sha1Hash || undefined,
      });

      // Map verify status string to AcquisitionInfo verifyResult
      const verifyResult = verifyStatus.includes("\u2713")
        ? "verified" as const
        : verifyStatus.includes("\u2717")
          ? "failed" as const
          : verifyStatus.includes("\u26A0")
            ? "error" as const
            : "skipped" as const;

      handleAcquisitionComplete({
        acquisitionType: "e01",
        outputPath: result.outputPath,
        sources: capturedSources,
        caseNumber: ewfCaseNumber(),
        evidenceNumber: ewfEvidenceNumber(),
        examiner: ewfExaminerName(),
        description: ewfDescription(),
        notes: ewfNotes(),
        format: result.format,
        totalBytes: result.bytesWritten,
        totalFiles: result.filesIncluded,
        compressed: result.compressed,
        segmentSize: ewfSegmentSize() > 0 ? ewfSegmentSize() * 1024 * 1024 : 0,
        md5: result.md5Hash || undefined,
        sha1: result.sha1Hash || undefined,
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
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      log.error(`E01 creation failed: ${getErrorMessage(error)}`);
      toast.error("E01 Creation Failed", getErrorMessage(error));
      dbSync.updateExport?.({
        ...dbRecord,
        status: "failed",
        completedAt: new Date().toISOString(),
        error: getErrorMessage(error),
      });
    } finally {
      common.setIsAcquiring(false);
      common.clearAllSources();
      setEwfImageName("evidence");
      common.setIsProcessing(false);
      if (shouldRestoreMounts) {
        common.restoreAllDriveMounts();
      }
    }
  };

  // ─── Reset ──────────────────────────────────────────────────────────────

  const resetEwfState = () => {
    setEwfVerifyAfterWrite(true);
    setEwfFormat("e01");
    setEwfCompression("none");
    setEwfCompressionMethod("deflate");
    setEwfComputeMd5(true);
    setEwfComputeSha1(false);
    setEwfSegmentSize(2048);
    setEwfImageName("evidence");
    setEwfCaseNumber("");
    setEwfEvidenceNumber("");
    setEwfExaminerName("");
    setEwfDescription("");
    setEwfNotes("");
  };

  return {
    // EWF state
    ewfVerifyAfterWrite,
    setEwfVerifyAfterWrite,
    ewfFormat,
    setEwfFormat,
    ewfCompression,
    setEwfCompression,
    ewfCompressionMethod,
    setEwfCompressionMethod,
    ewfComputeMd5,
    setEwfComputeMd5,
    ewfComputeSha1,
    setEwfComputeSha1,
    ewfSegmentSize,
    setEwfSegmentSize,
    ewfImageName,
    setEwfImageName,
    ewfCaseNumber,
    setEwfCaseNumber,
    ewfEvidenceNumber,
    setEwfEvidenceNumber,
    ewfExaminerName,
    setEwfExaminerName,
    ewfDescription,
    setEwfDescription,
    ewfNotes,
    setEwfNotes,

    // Handler
    handleCreateE01Image,

    // Reset
    resetEwfState,
  } as const;
}

export type EwfExportState = ReturnType<typeof useEwfExportState>;
