// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * File Export API
 * 
 * Provides file copy and forensic export operations with:
 * - Progress tracking
 * - Recursive directory support
 * - Forensic mode with SHA-256 hash manifest
 * - Preserved file timestamps
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { formatBytes } from "../utils";

/**
 * Progress event during copy/export operations
 */
export interface CopyProgress {
  /** Operation ID */
  operationId: string;
  /** Current file being processed */
  currentFile: string;
  /** Current file index (1-based) */
  currentIndex: number;
  /** Total number of files */
  totalFiles: number;
  /** Bytes copied for current file */
  currentFileBytes: number;
  /** Total bytes for current file */
  currentFileTotal: number;
  /** Total bytes copied across all files */
  totalBytesCopied: number;
  /** Total bytes to copy */
  totalBytes: number;
  /** Progress percentage (0-100) */
  percent: number;
  /** Current operation status */
  status: string;
  /** Copy speed in bytes per second */
  speedBps: number;
  /** Current phase: "calculating", "copying", "hashing", "verifying", "complete" */
  phase?: string;
  /** Bytes hashed so far (for hashing phase) */
  hashBytesProcessed?: number;
  /** Total bytes to hash (for hashing phase) */
  hashBytesTotal?: number;
}

/**
 * Result of a copy operation
 */
export interface CopyResult {
  /** Unique operation ID for this export (e.g., "export-1719842300000") */
  operationId: string;
  /** Number of files copied */
  filesCopied: number;
  /** Number of files failed */
  filesFailed: number;
  /** Total bytes copied */
  bytesCopied: number;
  /** Duration in milliseconds */
  durationMs: number;
  /** Average speed in bytes per second */
  avgSpeedBps: number;
  /** Failed file paths with error messages */
  failures: [string, string][];
  /** Export metadata (only when computeHashes is enabled) */
  metadata?: ExportMetadata[];
  /** Path to JSON manifest file (if generated) */
  jsonManifestPath?: string;
  /** Path to TXT report file (if generated) */
  txtReportPath?: string;
  /** Number of files that match known hashes */
  filesVerifiedKnown: number;
  /** Number of files that don't match known hashes */
  filesMismatchKnown: number;
}

/**
 * Export metadata for forensic exports
 */
export interface ExportMetadata {
  /** Source path */
  sourcePath: string;
  /** Destination path */
  destinationPath: string;
  /** File size in bytes */
  size: number;
  /** SHA-256 hash of the file (if computed) */
  sha256?: string;
  /** Original modified time (Unix timestamp) */
  modifiedTime: number;
  /** Export timestamp (Unix timestamp) */
  exportTime: number;
  /** Whether copy verification passed */
  copyVerified: boolean;
  /** Known hash from database/cache (if available) */
  knownHash?: string;
  /** Whether file matches known hash */
  matchesKnown?: boolean;
  /** Known hash source (e.g., "hash_cache", "database") */
  knownHashSource?: string;
}

/**
 * Export/Copy options
 */
export interface ExportOptions {
  /** Compute SHA-256 hashes for all files */
  computeHashes?: boolean;
  /** Verify copied files match source hashes */
  verifyAfterCopy?: boolean;
  /** Compare against known hashes (from hash cache/database) */
  verifyAgainstKnown?: boolean;
  /** Generate JSON manifest file */
  generateJsonManifest?: boolean;
  /** Generate TXT report file */
  generateTxtReport?: boolean;
  /** Preserve file timestamps (default: true) */
  preserveTimestamps?: boolean;
  /** Overwrite existing files (default: false) */
  overwrite?: boolean;
  /** Create parent directories (default: true) */
  createDirs?: boolean;
  /** Export name (for manifest/report filenames) */
  exportName?: string;
}

/**
 * Unified export/copy function with options
 * 
 * Modes:
 * - Simple copy: computeHashes = false (default)
 * - Forensic export: computeHashes = true, generateJsonManifest = true
 * 
 * @param sources - Array of source file/directory paths
 * @param destination - Destination directory path
 * @param options - Export options
 * @param onProgress - Progress callback
 * @returns Copy result with statistics and optional metadata
 */
export async function exportFiles(
  sources: string[],
  destination: string,
  options?: ExportOptions,
  onProgress?: (progress: CopyProgress) => void
): Promise<CopyResult> {
  let unlistenFn: UnlistenFn | undefined;

  try {
    if (onProgress) {
      // Track the operation_id from the first progress event so we only forward
      // events belonging to THIS export when multiple exports run concurrently.
      let myOperationId: string | null = null;

      unlistenFn = await listen<CopyProgress>("copy-progress", (event) => {
        const prog = event.payload;
        if (myOperationId === null) {
          // First event — latch onto this operation's ID
          myOperationId = prog.operationId;
        }
        if (prog.operationId === myOperationId) {
          onProgress(prog);
        }
      });
    }

    const result = await invoke<CopyResult>("export_files", {
      sourcePaths: sources,
      destination,
      options: options || null,
    });

    return result;
  } finally {
    if (unlistenFn) {
      unlistenFn();
    }
  }
}

/**
 * Format duration in milliseconds as human-readable string
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  const minutes = Math.floor(ms / 60000);
  const seconds = Math.floor((ms % 60000) / 1000);
  return `${minutes}m ${seconds}s`;
}

/**
 * Cancel an in-progress file export operation
 *
 * @param operationId - The operation ID returned by the export (from CopyResult
 *   or the progress event's operationId field)
 * @returns true if the cancel was accepted, false if no matching operation found
 */
export async function cancelExport(operationId: string): Promise<boolean> {
  return invoke<boolean>("cancel_export", { operationId });
}

/**
 * Calculate transfer speed
 */
export function calculateSpeed(bytes: number, durationMs: number): string {
  if (durationMs === 0) return "0 B/s";
  const bytesPerSecond = (bytes / durationMs) * 1000;
  return `${formatBytes(bytesPerSecond)}/s`;
}
