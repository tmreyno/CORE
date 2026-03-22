// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useL01ExportState — L01 logical evidence creation state and handler.
 */

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { createL01Image, buildL01ExportOptions } from "../../api/l01Export";
import { formatBytes } from "../../api/archiveCreate";
import { getErrorMessage } from "../../utils/errorUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("L01Export");
import { joinPath } from "../../utils/pathUtils";
import {
  createActivity,
  updateProgress,
  completeActivity,
  failActivity,
} from "../../types/activity";
import type { ExportToast, ExportActivityCallbacks } from "./types";
import type { ExportCommonState } from "./useExportCommon";
import { dbSync } from "../project/useProjectDbSync";
import type { DbExportRecord } from "../../types/projectDb";
import { handleAcquisitionComplete, startAcquisitionRecord } from "./companionHelper";

/** Parse a comma/space-separated extension string into an array, stripping dots. */
function parseExtensionList(input: string): string[] | undefined {
  if (!input.trim()) return undefined;
  const exts = input
    .split(/[,;\s]+/)
    .map((s) => s.replace(/^\./, "").trim().toLowerCase())
    .filter(Boolean);
  return exts.length > 0 ? exts : undefined;
}

export interface UseL01ExportStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
  caseNumber?: string;
  examinerName?: string;
  systemStats?: { hostname?: string; systemModel?: string; systemSerialNumber?: string; systemManufacturer?: string; osName?: string; osVersion?: string } | null;
}

export function useL01ExportState(options: UseL01ExportStateOptions) {
  const { toast, common } = options;

  // === L01 Logical Evidence State ===
  const [l01ImageName, setL01ImageName] = createSignal("evidence");
  const [l01Compression, setL01Compression] = createSignal("none");
  const [l01HashAlgorithm, _setL01HashAlgorithm] = createSignal("md5");
  const [l01SegmentSize, setL01SegmentSize] = createSignal(2048);
  const [l01CaseNumber, setL01CaseNumber] = createSignal("");
  const [l01EvidenceNumber, setL01EvidenceNumber] = createSignal("");
  const [l01ExaminerName, setL01ExaminerName] = createSignal("");
  const [l01Description, setL01Description] = createSignal("");
  const [l01Notes, setL01Notes] = createSignal("");
  const [l01FilterExtensions, setL01FilterExtensions] = createSignal("");
  const [l01ExcludeExtensions, setL01ExcludeExtensions] = createSignal("");
  const [l01MinFileSize, setL01MinFileSize] = createSignal<number | undefined>(undefined);
  const [l01MaxFileSize, setL01MaxFileSize] = createSignal<number | undefined>(undefined);

  // ─── Handler ────────────────────────────────────────────────────────────

  const handleCreateL01Image = async () => {
    log.info(`Starting L01 creation: ${l01ImageName()}.L01, compression=${l01Compression()}, sources=${common.sources().length}`);
    common.setIsProcessing(true);
    common.setIsAcquiring(true);

    const outputPath = joinPath(common.destination(), l01ImageName());
    const shouldRestoreMounts = common.hasDriveSources() && common.mountDrivesReadOnly();

    const activity = createActivity("export", outputPath, common.sources().length, {
      operation: `L01 Logical Evidence Creation (${l01Compression()})`,
    });

    options.onActivityCreate?.(activity);

    const capturedSources = [...common.sources()];
    const exportId = `l01-${Date.now()}`;
    let dbRecord: DbExportRecord = {
      id: exportId,
      exportType: "l01",
      sourcePathsJson: JSON.stringify(capturedSources),
      destination: outputPath,
      startedAt: new Date().toISOString(),
      initiatedBy: l01ExaminerName() || "",
      status: "in_progress",
      totalFiles: capturedSources.length,
      totalBytes: 0,
      archiveName: l01ImageName() + ".L01",
      archiveFormat: "L01",
      compressionLevel: l01Compression(),
      encrypted: false,
    };

    try {
      const l01Options = buildL01ExportOptions({
        sourcePaths: common.sources(),
        outputPath,
        compression: l01Compression(),
        hashAlgorithm: l01HashAlgorithm(),
        segmentSize: l01SegmentSize() > 0 ? l01SegmentSize() * 1024 * 1024 : undefined,
        caseNumber: l01CaseNumber() || undefined,
        evidenceNumber: l01EvidenceNumber() || undefined,
        examinerName: l01ExaminerName() || undefined,
        description: l01Description() || undefined,
        notes: l01Notes() || undefined,
        filterExtensions: parseExtensionList(l01FilterExtensions()),
        excludeExtensions: parseExtensionList(l01ExcludeExtensions()),
        minFileSize: l01MinFileSize(),
        maxFileSize: l01MaxFileSize(),
      });

      // Track in DB
      dbRecord = { ...dbRecord, optionsJson: JSON.stringify(l01Options) };
      dbSync.insertExport(dbRecord);

      const acqRecord = startAcquisitionRecord({
        acquisitionType: "l01",
        outputPath,
        sources: capturedSources,
        caseNumber: options.caseNumber,
        examiner: options.examinerName,
        hostname: options.systemStats?.hostname,
        systemModel: options.systemStats?.systemModel,
        systemSerialNumber: options.systemStats?.systemSerialNumber,
        systemManufacturer: options.systemStats?.systemManufacturer,
      });

      const result = await createL01Image(l01Options, (prog) => {
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

      // Post-write verification: re-read the L01 and compare hash
      let verifyStatus = "";
      if (result.md5Hash || result.sha1Hash) {
        const algo = result.md5Hash ? "MD5" : "SHA1";
        const expected = result.md5Hash || result.sha1Hash;
        const verifyPath = result.outputPaths?.[0] || outputPath + ".L01";
        const t0 = performance.now();
        try {
          const computed = await invoke<string>("e01_v3_verify", {
            inputPath: verifyPath,
            algorithm: algo,
          });
          const verifyMs = (performance.now() - t0).toFixed(0);
          if (computed === expected) {
            verifyStatus = " | \u2713 Verified";
            log.info(`L01 verify passed in ${verifyMs}ms (${algo})`);
          } else {
            verifyStatus = " | \u2717 VERIFY FAILED";
            log.error(`L01 verify FAILED in ${verifyMs}ms — expected=${expected}, computed=${computed}`);
            toast.error("Verification Failed", "Written L01 hash does not match source data hash.");
          }
        } catch (verifyErr) {
          verifyStatus = " | \u26A0 Verify error";
          log.warn(`L01 verify error: ${getErrorMessage(verifyErr)}`);
        }
      }

      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      const hashInfo = result.md5Hash ? ` | MD5: ${result.md5Hash.substring(0, 16)}...` : "";
      const ratio = result.compressionRatio
        ? ` | Ratio: ${(result.compressionRatio * 100).toFixed(1)}%`
        : "";
      const durationSec = (result.durationMs / 1000).toFixed(1);
      log.info(`L01 image created: ${result.totalFiles} files, ${formatBytes(result.totalDataBytes)} in ${durationSec}s${verifyStatus}`);
      toast.success(
        "L01 Image Created",
        `${result.totalFiles} files packaged (${formatBytes(result.totalDataBytes)})${hashInfo}${ratio}${verifyStatus}`,
      );
      options.onComplete?.(outputPath);

      dbSync.updateExport({
        ...dbRecord,
        status: "completed",
        completedAt: new Date().toISOString(),
        totalFiles: result.totalFiles,
        totalBytes: result.totalDataBytes,
        manifestHash: result.md5Hash || undefined,
      });

      const verifyResult = verifyStatus.includes("\u2713")
        ? "verified" as const
        : verifyStatus.includes("\u2717")
          ? "failed" as const
          : verifyStatus.includes("\u26A0")
            ? "error" as const
            : "skipped" as const;

      handleAcquisitionComplete({
        acquisitionType: "l01",
        outputPath: result.outputPaths?.[0] || outputPath,
        sources: capturedSources,
        caseNumber: l01CaseNumber(),
        evidenceNumber: l01EvidenceNumber(),
        examiner: l01ExaminerName(),
        description: l01Description(),
        notes: l01Notes(),
        format: "L01",
        segments: result.segmentCount,
        totalBytes: result.totalDataBytes,
        totalFiles: result.totalFiles,
        compressed: result.compressionRatio != null && result.compressionRatio < 1,
        segmentSize: l01SegmentSize() > 0 ? l01SegmentSize() * 1024 * 1024 : 0,
        md5: result.md5Hash || undefined,
        sha1: result.sha1Hash || undefined,
        startedAt: dbRecord.startedAt,
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
      log.error(`L01 creation failed: ${getErrorMessage(error)}`);
      toast.error("L01 Creation Failed", getErrorMessage(error));
      dbSync.updateExport?.({
        ...dbRecord,
        status: "failed",
        completedAt: new Date().toISOString(),
        error: getErrorMessage(error),
      });
    } finally {
      common.setIsAcquiring(false);
      common.clearAllSources();
      setL01ImageName("evidence");
      common.setIsProcessing(false);
      if (shouldRestoreMounts) {
        common.restoreAllDriveMounts();
      }
    }
  };

  // ─── Reset ──────────────────────────────────────────────────────────────

  const resetL01State = () => {
    setL01ImageName("evidence");
    setL01Compression("none");
    setL01SegmentSize(2048);
    setL01CaseNumber("");
    setL01EvidenceNumber("");
    setL01ExaminerName("");
    setL01Description("");
    setL01Notes("");
    setL01FilterExtensions("");
    setL01ExcludeExtensions("");
    setL01MinFileSize(undefined);
    setL01MaxFileSize(undefined);
  };

  return {
    // L01 state
    l01ImageName,
    setL01ImageName,
    l01Compression,
    setL01Compression,
    l01HashAlgorithm,
    l01SegmentSize,
    setL01SegmentSize,
    l01CaseNumber,
    setL01CaseNumber,
    l01EvidenceNumber,
    setL01EvidenceNumber,
    l01ExaminerName,
    setL01ExaminerName,
    l01Description,
    setL01Description,
    l01Notes,
    setL01Notes,
    l01FilterExtensions,
    setL01FilterExtensions,
    l01ExcludeExtensions,
    setL01ExcludeExtensions,
    l01MinFileSize,
    setL01MinFileSize,
    l01MaxFileSize,
    setL01MaxFileSize,

    // Handler
    handleCreateL01Image,

    // Reset
    resetL01State,
  } as const;
}

export type L01ExportState = ReturnType<typeof useL01ExportState>;
