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
} from "../../types/activity";
import type { ExportToast, ExportActivityCallbacks } from "./types";
import type { ExportCommonState } from "./useExportCommon";
import { dbSync } from "../project/useProjectDbSync";
import type { DbExportRecord } from "../../types/projectDb";
import { handleAcquisitionComplete } from "./companionHelper";

export interface UseMemoryDumpStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
  caseNumber?: string;
  examinerName?: string;
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

    const activity = createActivity("export", outputPath, 1, {
      operation: "Live Memory Capture",
    });
    options.onActivityCreate?.(activity);

    // Listen for progress events — set up BEFORE backend call to avoid race
    let unlisten: (() => void) | undefined;
    try {
      unlisten = await listenMemoryCaptureProgress((progress) => {
        setMemoryProgress(progress);
      });
    } catch (err) {
      console.warn("Failed to set up memory capture progress listener:", err);
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

    try {
      const result = await captureMemory(outputPath, memoryComputeHashes());
      setMemoryResult(result);

      const sizeMb = (result.bytesCaptured / (1024 * 1024)).toFixed(1);
      const durationStr = result.durationSecs < 60
        ? `${result.durationSecs.toFixed(1)}s`
        : `${Math.floor(result.durationSecs / 60)}m ${Math.floor(result.durationSecs % 60)}s`;

      toast.success(
        "Memory Captured",
        `${sizeMb} MB captured in ${durationStr}`,
      );

      completeActivity(activity);
      options.onActivityUpdate?.(activity.id, activity);

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
      });

      options.onComplete?.(common.destination());
    } catch (err) {
      const msg = getErrorMessage(err);
      log.error(`Memory capture failed: ${msg}`);
      toast.error("Memory Capture Failed", msg);

      failActivity(activity, msg);
      options.onActivityUpdate?.(activity.id, activity);

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
