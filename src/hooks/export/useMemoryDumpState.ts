// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useMemoryDumpState — Live RAM capture state and handler.
 */

import { createSignal } from "solid-js";
import {
  getMemoryCaptureInfo,
  captureMemory,
  cancelMemoryCapture,
  listenMemoryCaptureProgress,
  type MemoryCaptureInfo,
  type MemoryCaptureProgress,
  type MemoryCaptureResult,
} from "../../api/memory";
import { getErrorMessage } from "../../utils/errorUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("MemoryCapture");
import {
  createActivity,
  completeActivity,
  failActivity,
  updateProgress,
} from "../../types/activity";
import type { ExportToast, ExportActivityCallbacks } from "./types";
import type { ExportCommonState } from "./useExportCommon";
import { dbSync } from "../project/useProjectDbSync";
import type { DbExportRecord } from "../../types/projectDb";
import { handleAcquisitionComplete, startAcquisitionRecord } from "./companionHelper";
import { canUseDesktopExportEngine } from "./desktopRuntimeGuard";

export interface UseMemoryDumpStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
  caseNumber?: string;
  examinerName?: string;
  /** Cached system stats from Identify phase (avoids re-fetching) */
  systemStats?: { hostname?: string; systemModel?: string; systemSerialNumber?: string; systemManufacturer?: string; osName?: string; osVersion?: string } | null;
}

export function useMemoryDumpState(options: UseMemoryDumpStateOptions) {
  const { toast, common } = options;

  // === Memory Capture State ===
  const [memoryInfo, setMemoryInfo] = createSignal<MemoryCaptureInfo | null>(null);
  const [memoryInfoLoading, setMemoryInfoLoading] = createSignal(false);
  const [memoryComputeHashes, setMemoryComputeHashes] = createSignal(true);
  const [memoryOutputName, setMemoryOutputName] = createSignal("memory_dump");
  const [memoryProgress, setMemoryProgress] = createSignal<MemoryCaptureProgress | null>(null);
  const [memoryResult, setMemoryResult] = createSignal<MemoryCaptureResult | null>(null);

  // ─── Load Memory Info ───────────────────────────────────────────────────

  const loadMemoryInfo = async () => {
    if (!canUseDesktopExportEngine(toast)) return;

    setMemoryInfoLoading(true);
    try {
      const info = await getMemoryCaptureInfo();
      setMemoryInfo(info);
    } catch (err) {
      toast.error("Memory Info", `Failed to query memory info: ${getErrorMessage(err)}`);
    } finally {
      setMemoryInfoLoading(false);
    }
  };

  // ─── Capture Handler ────────────────────────────────────────────────────

  const handleCaptureMemory = async () => {
    if (!canUseDesktopExportEngine(toast)) return;

    const info = memoryInfo();
    if (!info?.captureSupported) {
      toast.error("Not Supported", info?.unsupportedReason || "Memory capture is not supported on this platform");
      return;
    }

    if (!common.destination()) {
      toast.error("No Destination", "Please select a destination folder");
      return;
    }

    common.setIsProcessing(true);
    common.setIsAcquiring(true);
    setMemoryProgress(null);
    setMemoryResult(null);

    const outputPath = `${common.destination()}/${memoryOutputName()}.mem`;
    log.info(`Starting memory capture: ${outputPath}, hashes=${memoryComputeHashes()}, method=${info.captureMethod}`);

    const activity = createActivity("memory", outputPath, 1, {
      operation: "Live Memory Capture",
    });
    options.onActivityCreate?.(activity);

    // Listen for progress events — set up BEFORE backend call to avoid race
    let unlisten: (() => void) | undefined;
    try {
      unlisten = await listenMemoryCaptureProgress((progress) => {
        setMemoryProgress(progress);
        options.onActivityUpdate?.(
          activity.id,
          updateProgress(activity, {
            percent: progress.percent,
            currentFile: progress.phase === "capturing" ? "capturing RAM" : progress.phase === "hashing" ? "computing hashes" : progress.phase,
            bytesProcessed: progress.bytesCaptured,
            bytesTotal: progress.totalBytes,
          }),
        );
      });
    } catch (err) {
      log.warn("Failed to set up memory capture progress listener:", err);
      toast.warning("Progress Unavailable", "Memory capture will proceed but progress updates may not display");
    }

    // Track in DB
    const exportId = `memory-${Date.now()}`;
    const dbRecord: DbExportRecord = {
      id: exportId,
      exportType: "memory",
      sourcePathsJson: JSON.stringify([info.captureMethod]),
      destination: outputPath,
      status: "in_progress",
      startedAt: new Date().toISOString(),
      initiatedBy: "",
      totalFiles: 1,
      totalBytes: info.totalMemoryBytes,
      encrypted: false,
      optionsJson: JSON.stringify({
        computeHashes: memoryComputeHashes(),
        captureMethod: info.captureMethod,
      }),
    };
    dbSync.insertExport(dbRecord);

    const acqRecord = startAcquisitionRecord({
      acquisitionType: "memory",
      outputPath,
      sources: [info.captureMethod],
      caseNumber: options.caseNumber,
      examiner: options.examinerName,
      hostname: options.systemStats?.hostname,
      systemModel: options.systemStats?.systemModel,
      systemSerialNumber: options.systemStats?.systemSerialNumber,
      systemManufacturer: options.systemStats?.systemManufacturer,
    });

    try {
      const result = await captureMemory(outputPath, memoryComputeHashes());
      setMemoryResult(result);

      const sizeMb = (result.bytesCaptured / (1024 * 1024)).toFixed(1);
      const durationStr = result.durationSecs < 60
        ? `${result.durationSecs.toFixed(1)}s`
        : `${Math.floor(result.durationSecs / 60)}m ${Math.floor(result.durationSecs % 60)}s`;

      log.info(`Memory captured: ${sizeMb} MB in ${durationStr}`);
      toast.success(
        "Memory Captured",
        `${sizeMb} MB captured in ${durationStr}`,
      );

      options.onActivityUpdate?.(activity.id, completeActivity(activity));

      // Update DB record
      dbSync.updateExport({
        ...dbRecord,
        status: "completed",
        completedAt: new Date().toISOString(),
      });

      handleAcquisitionComplete({
        acquisitionType: "memory",
        outputPath,
        sources: [info.captureMethod],
        format: "raw_memory",
        totalBytes: result.bytesCaptured,
        startedAt: dbRecord.startedAt,
        completedAt: new Date().toISOString(),
        durationMs: result.durationSecs * 1000,
        caseNumber: options.caseNumber || "",
        examiner: options.examinerName || "",
        description: `Live memory capture — ${info.captureMethod}`,
        // System identification from Identify phase
        hostname: options.systemStats?.hostname,
        username: options.examinerName,
        systemModel: options.systemStats?.systemModel,
        systemSerialNumber: options.systemStats?.systemSerialNumber,
        systemManufacturer: options.systemStats?.systemManufacturer,
        osName: options.systemStats?.osName,
        osVersion: options.systemStats?.osVersion,
        collectionId: acqRecord.collectionId,
        itemId: acqRecord.itemId,
      });

      options.onComplete?.(common.destination());
    } catch (err) {
      const msg = getErrorMessage(err);
      log.error(`Memory capture failed: ${msg}`);
      toast.error("Memory Capture Failed", msg);

      options.onActivityUpdate?.(activity.id, failActivity(activity, msg));

      dbSync.updateExport({
        ...dbRecord,
        status: "failed",
        completedAt: new Date().toISOString(),
        error: msg,
      });
    } finally {
      unlisten?.();
      common.setIsProcessing(false);
      common.setIsAcquiring(false);
    }
  };

  // ─── Cancel ─────────────────────────────────────────────────────────────

  const handleCancelMemoryCapture = async () => {
    if (!canUseDesktopExportEngine(toast)) return;

    try {
      await cancelMemoryCapture();
      toast.info("Cancelling", "Memory capture will stop at the next chunk boundary");
    } catch (err) {
      toast.error("Cancel Failed", getErrorMessage(err));
    }
  };

  // ─── Reset ──────────────────────────────────────────────────────────────

  const resetMemoryState = () => {
    setMemoryProgress(null);
    setMemoryResult(null);
    setMemoryComputeHashes(true);
    setMemoryOutputName("memory_dump");
  };

  return {
    memoryInfo,
    memoryInfoLoading,
    memoryComputeHashes,
    setMemoryComputeHashes,
    memoryOutputName,
    setMemoryOutputName,
    memoryProgress,
    memoryResult,
    loadMemoryInfo,
    handleCaptureMemory,
    handleCancelMemoryCapture,
    resetMemoryState,
  };
}

export type MemoryDumpState = ReturnType<typeof useMemoryDumpState>;
