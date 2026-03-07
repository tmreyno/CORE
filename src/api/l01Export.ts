// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * L01 Logical Evidence Export API (pure-Rust backend)
 *
 * Creates L01 logical evidence containers (EWF v1 Logical format) with:
 * - zlib-compressed file data in 32 KB chunks
 * - Full LEF directory tree (ltree section)
 * - MD5 or SHA-1 image integrity hashing
 * - Case metadata (case number, evidence number, examiner, etc.)
 * - Progress tracking with cancellation support
 * - Recursive directory acquisition
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// =============================================================================
// Types
// =============================================================================

/**
 * Options for creating an L01 logical evidence container
 */
export interface L01ExportOptions {
  /** Source file or directory paths to include */
  sourcePaths: string[];
  /** Output path for the L01 file (without extension — .L01 is appended) */
  outputPath: string;
  /** Compression level: "none", "fast" (default), "best" */
  compression?: string;
  /**
   * Hash algorithm hint: "md5" (default), "sha1"
   * Note: The L01 writer always computes AND embeds both MD5 and SHA-1 hashes
   * (per-file and image-level). This field is kept for backward compatibility
   * but has no effect on output — both are always included.
   */
  hashAlgorithm?: string;
  /** Maximum segment file size in bytes (0 = no splitting) */
  segmentSize?: number;
  /** Case number for forensic metadata */
  caseNumber?: string;
  /** Evidence number for forensic metadata */
  evidenceNumber?: string;
  /** Examiner name for forensic metadata */
  examinerName?: string;
  /** Description for forensic metadata */
  description?: string;
  /** Notes for forensic metadata */
  notes?: string;
}

/**
 * Phases of the L01 write operation
 */
export type L01WritePhase =
  | "preparing"
  | "writingData"
  | "buildingTables"
  | "writingLtree"
  | "computingHash"
  | "finalizing";

/**
 * Progress event emitted during L01 creation
 */
export interface L01ExportProgress {
  /** Output file path */
  path: string;
  /** Current file being processed */
  currentFile: string;
  /** Files processed so far */
  filesProcessed: number;
  /** Total files to process */
  totalFiles: number;
  /** Bytes written so far */
  bytesWritten: number;
  /** Total bytes to write */
  totalBytes: number;
  /** Progress percentage (0–100) */
  percent: number;
  /** Current phase of the write operation */
  phase: L01WritePhase;
}

/**
 * Result of a successful L01 export
 */
export interface L01ExportResult {
  /** Output file path(s) created */
  outputPaths: string[];
  /** Total files written into the L01 */
  totalFiles: number;
  /** Total directories written */
  totalDirectories: number;
  /** Total bytes of file data written */
  totalDataBytes: number;
  /** Total compressed bytes */
  totalCompressedBytes: number;
  /** Compression ratio (compressed / original) */
  compressionRatio: number;
  /** Image MD5 hash (if computed) */
  md5Hash: string | null;
  /** Image SHA-1 hash (if computed) */
  sha1Hash: string | null;
  /** Number of segment files */
  segmentCount: number;
  /** Number of data chunks */
  chunkCount: number;
  /** Duration in milliseconds */
  durationMs: number;
}

// =============================================================================
// API Functions
// =============================================================================

/**
 * Create an L01 logical evidence container from source files/directories
 *
 * @param options Export configuration
 * @param onProgress Optional progress callback
 * @returns Export result with output paths, hashes, and stats
 */
export async function createL01Image(
  options: L01ExportOptions,
  onProgress?: (progress: L01ExportProgress) => void,
): Promise<L01ExportResult> {
  let unlisten: UnlistenFn | undefined;

  try {
    // Set up progress listener
    // Filter by output path so concurrent L01 exports don't cross-contaminate.
    if (onProgress) {
      let myPath: string | null = null;
      unlisten = await listen<L01ExportProgress>(
        "l01-export-progress",
        (event) => {
          // Latch onto first event's path (backend may append extension)
          if (myPath === null) {
            myPath = event.payload.path;
          }
          if (event.payload.path === myPath) {
            onProgress(event.payload);
          }
        },
      );
    }

    // Invoke the backend command
    const result = await invoke<L01ExportResult>("l01_create_image", {
      options,
    });

    return result;
  } finally {
    if (unlisten) {
      unlisten();
    }
  }
}

/**
 * Cancel an in-progress L01 export
 *
 * @param outputPath The output path of the export to cancel
 * @returns true if the export was found and cancelled, false if not found
 */
export async function cancelL01Export(outputPath: string): Promise<boolean> {
  return invoke<boolean>("l01_cancel_export", { outputPath });
}

/**
 * Estimate the output size for an L01 export
 *
 * Returns approximate total bytes based on source sizes and compression level.
 * This is a heuristic — actual size depends on data compressibility.
 *
 * @param sourcePaths Source file/directory paths
 * @param compression Compression level: "none", "fast", "best"
 * @returns Estimated output size in bytes
 */
export async function estimateL01Size(
  sourcePaths: string[],
  compression?: string,
): Promise<number> {
  return invoke<number>("l01_estimate_size", {
    sourcePaths,
    compression: compression ?? null,
  });
}

// =============================================================================
// Helpers
// =============================================================================

/**
 * Build L01ExportOptions from common UI inputs
 */
export function buildL01ExportOptions(params: {
  sourcePaths: string[];
  outputPath: string;
  compression?: string;
  hashAlgorithm?: string;
  segmentSize?: number;
  caseNumber?: string;
  evidenceNumber?: string;
  examinerName?: string;
  description?: string;
  notes?: string;
}): L01ExportOptions {
  return {
    sourcePaths: params.sourcePaths,
    outputPath: params.outputPath,
    compression: params.compression ?? "fast",
    hashAlgorithm: params.hashAlgorithm ?? "md5",
    segmentSize: params.segmentSize,
    caseNumber: params.caseNumber,
    evidenceNumber: params.evidenceNumber,
    examinerName: params.examinerName,
    description: params.description,
    notes: params.notes,
  };
}
