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
  /** Export metadata (only for forensic export) */
  metadata?: ExportMetadata[];
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
  /** SHA-256 hash of the file */
  sha256: string;
  /** Original modified time (Unix timestamp) */
  modifiedTime: number;
  /** Export timestamp (Unix timestamp) */
  exportTime: number;
  /** Whether verification passed */
  verified: boolean;
}

/**
 * Copy files to a destination with progress tracking
 * 
 * @param sources - Array of source file/directory paths
 * @param destination - Destination directory path
 * @param onProgress - Progress callback
 * @returns Copy result with statistics
 */
export async function copyFiles(
  sources: string[],
  destination: string,
  onProgress?: (progress: CopyProgress) => void
): Promise<CopyResult> {
  let unlistenFn: UnlistenFn | undefined;

  try {
    if (onProgress) {
      unlistenFn = await listen<CopyProgress>("copy-progress", (event) => {
        onProgress(event.payload);
      });
    }

    const result = await invoke<CopyResult>("copy_files", {
      sourcePaths: sources,
      destination,
    });

    return result;
  } finally {
    if (unlistenFn) {
      unlistenFn();
    }
  }
}

/**
 * Export files with forensic metadata (timestamps and SHA-256 hashes)
 * 
 * This is the forensic-grade export that:
 * - Preserves original file timestamps
 * - Generates SHA-256 hashes for each file
 * - Creates a JSON manifest file for verification
 * - Verifies each copied file against source hash
 * 
 * @param sources - Array of source file/directory paths
 * @param destination - Destination directory path
 * @param exportName - Name for the export (used for manifest filename)
 * @param onProgress - Progress callback
 * @returns Copy result with metadata
 */
export async function exportFiles(
  sources: string[],
  destination: string,
  exportName: string = "forensic_export",
  onProgress?: (progress: CopyProgress) => void
): Promise<CopyResult> {
  let unlistenFn: UnlistenFn | undefined;

  try {
    if (onProgress) {
      unlistenFn = await listen<CopyProgress>("copy-progress", (event) => {
        onProgress(event.payload);
      });
    }

    const result = await invoke<CopyResult>("export_files_forensic", {
      sourcePaths: sources,
      destination,
      exportName,
    });

    return result;
  } finally {
    if (unlistenFn) {
      unlistenFn();
    }
  }
}

/**
 * Format bytes as human-readable string
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
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
 * Calculate transfer speed
 */
export function calculateSpeed(bytes: number, durationMs: number): string {
  if (durationMs === 0) return "0 B/s";
  const bytesPerSecond = (bytes / durationMs) * 1000;
  return `${formatBytes(bytesPerSecond)}/s`;
}
