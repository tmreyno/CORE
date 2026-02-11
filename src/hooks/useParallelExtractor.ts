// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, onMount, onCleanup, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { logger } from "../utils/logger";
const log = logger.scope("ParallelExtractor");

// Types matching Rust backend
export type ExtractionStatus =
  | "Queued"
  | "Extracting"
  | "Verifying"
  | "Completed"
  | "Failed"
  | "Cancelled";

export interface ExtractionJob {
  id: string;
  sourcePath: string;
  entryPath: string;
  destinationPath: string;
  sizeBytes: number;
  status: ExtractionStatus;
  bytesExtracted: number;
  percentComplete: number;
  hashAlgorithm: string | null;
  computedHash: string | null;
  expectedHash: string | null;
  errorMessage: string | null;
}

export interface BatchExtractionProgress {
  batchId: string;
  totalFiles: number;
  completedFiles: number;
  failedFiles: number;
  cancelledFiles: number;
  totalBytes: number;
  extractedBytes: number;
  percentComplete: number;
  throughputMbps: number;
  estimatedSecondsRemaining: number | null;
  activeJobs: ExtractionJob[];
}

export interface ExtractionEntry {
  entryPath: string;
  destinationPath: string;
  sizeBytes: number;
  expectedHash?: string;
}

/**
 * Hook for parallel file extraction with real-time progress
 */
export function useParallelExtractor() {
  const [initialized, setInitialized] = createSignal(false);
  const [activeBatches, setActiveBatches] = createSignal<Map<string, BatchExtractionProgress>>(
    new Map()
  );
  const [error, setError] = createSignal<string | null>(null);

  let unlistenProgress: UnlistenFn | undefined;

  onMount(async () => {
    try {
      // Initialize parallel extractor
      await invoke("parallel_extract_init");
      setInitialized(true);

      // Listen for progress updates
      unlistenProgress = await listen<BatchExtractionProgress>(
        "batch-extraction-progress",
        (event) => {
          setActiveBatches((batches) => {
            const newBatches = new Map(batches);
            newBatches.set(event.payload.batchId, event.payload);
            return newBatches;
          });
        }
      );
    } catch (err) {
      setError(`Failed to initialize: ${err}`);
      log.error("Failed to initialize parallel extractor:", err);
    }
  });

  onCleanup(() => {
    unlistenProgress?.();
  });

  /**
   * Start a batch extraction
   */
  const startBatch = async (
    batchId: string,
    containerPath: string,
    containerType: string,
    entries: ExtractionEntry[],
    destinationBase: string,
    options?: {
      hashAlgorithm?: string;
      maxConcurrent?: number;
    }
  ): Promise<void> => {
    try {
      setError(null);
      await invoke("parallel_extract_batch", {
        batchId,
        containerPath,
        containerType,
        entries,
        destinationBase,
        hashAlgorithm: options?.hashAlgorithm || null,
        maxConcurrent: options?.maxConcurrent || 4,
      });
    } catch (err) {
      setError(`Extraction failed: ${err}`);
      throw err;
    }
  };

  /**
   * Cancel a batch extraction
   */
  const cancelBatch = async (batchId: string): Promise<void> => {
    try {
      await invoke("parallel_extract_cancel", { batchId });
      
      // Update local state
      setActiveBatches((batches) => {
        const newBatches = new Map(batches);
        const batch = newBatches.get(batchId);
        if (batch) {
          newBatches.set(batchId, {
            ...batch,
            activeJobs: batch.activeJobs.map((job) => ({
              ...job,
              status: "Cancelled",
            })),
          });
        }
        return newBatches;
      });
    } catch (err) {
      setError(`Cancel failed: ${err}`);
      throw err;
    }
  };

  /**
   * Get active batch IDs
   */
  const getActiveBatchIds = async (): Promise<string[]> => {
    try {
      return await invoke<string[]>("parallel_extract_get_active");
    } catch (err) {
      log.error("Failed to get active batches:", err);
      return [];
    }
  };

  /**
   * Get progress for specific batch
   */
  const getBatchProgress = (batchId: string): BatchExtractionProgress | undefined => {
    return activeBatches().get(batchId);
  };

  /**
   * Remove completed batch from tracking
   */
  const removeBatch = (batchId: string) => {
    setActiveBatches((batches) => {
      const newBatches = new Map(batches);
      newBatches.delete(batchId);
      return newBatches;
    });
  };

  /**
   * Get all active batches as array
   */
  const activeBatchList = createMemo(() => {
    return Array.from(activeBatches().values());
  });

  /**
   * Get overall extraction statistics
   */
  const overallStats = createMemo(() => {
    const batches = activeBatchList();
    return {
      totalBatches: batches.length,
      totalFiles: batches.reduce((sum, b) => sum + b.totalFiles, 0),
      completedFiles: batches.reduce((sum, b) => sum + b.completedFiles, 0),
      failedFiles: batches.reduce((sum, b) => sum + b.failedFiles, 0),
      totalBytes: batches.reduce((sum, b) => sum + b.totalBytes, 0),
      extractedBytes: batches.reduce((sum, b) => sum + b.extractedBytes, 0),
      avgThroughputMbps:
        batches.length > 0
          ? batches.reduce((sum, b) => sum + b.throughputMbps, 0) / batches.length
          : 0,
    };
  });

  /**
   * Format bytes to human-readable string
   */
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  /**
   * Format time in seconds to human-readable string
   */
  const formatTime = (seconds: number | null): string => {
    if (seconds === null) return "Calculating...";
    if (seconds < 60) return `${Math.round(seconds)}s`;
    if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.round((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  return {
    // State
    initialized,
    activeBatches: activeBatchList,
    error,
    overallStats,

    // Actions
    startBatch,
    cancelBatch,
    getActiveBatchIds,
    getBatchProgress,
    removeBatch,

    // Utilities
    formatBytes,
    formatTime,
  };
}
