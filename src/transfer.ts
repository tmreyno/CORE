// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * File Transfer API - TypeScript bindings for file copy/transfer operations
 * 
 * Provides functions for:
 * - Copying single files or entire directories
 * - Progress tracking with real-time events
 * - Hash verification after copy
 * - Cancellation support for long operations
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { formatBytes } from "./utils";

// Re-export formatBytes from utils for backwards compatibility
export { formatBytes };

/**
 * Request to start a file transfer operation
 */
export interface TransferRequest {
  /** Source paths to copy */
  sources: string[];
  /** Destination directory */
  destination: string;
  /** Whether to verify files after copying (hash comparison) */
  verify?: boolean;
  /** Hash algorithm for verification ("md5", "sha256", "xxh3", etc.) */
  hashAlgorithm?: string;
  /** Whether to preserve file timestamps */
  preserveTimestamps?: boolean;
  /** Whether to preserve permissions */
  preservePermissions?: boolean;
  /** Whether to overwrite existing files */
  overwrite?: boolean;
  /** Whether to copy directories recursively */
  recursive?: boolean;
  /** Whether to flatten directory structure */
  flatten?: boolean;
  /** Whether sources contain forensic containers (E01, AD1, etc.) that should be treated as logical units */
  containerAware?: boolean;
  /** Number of parallel transfer threads (1-8, default: 4) */
  parallelThreads?: number;
}

/**
 * Preview of files to be transferred
 */
export interface TransferPreview {
  /** Total number of files (may be limited for large directories) */
  total_files: number;
  /** Total number of directories */
  total_directories: number;
  /** Total size in bytes */
  total_bytes: number;
  /** Human-readable size */
  total_size_formatted: string;
  /** List of files with their relative paths and sizes */
  files: TransferFileInfo[];
  /** Whether the preview was limited (more files exist than shown) */
  is_limited?: boolean;
}

/**
 * Information about a file in the transfer preview
 */
export interface TransferFileInfo {
  /** Source path */
  source: string;
  /** Relative path (for destination) */
  relative_path: string;
  /** Size in bytes */
  size: number;
  /** Human-readable size */
  size_formatted: string;
}

/**
 * Phase of a transfer operation
 */
export type TransferPhase = 
  | "Scanning"
  | "Copying"
  | "Verifying"
  | "Completed"
  | "Cancelled"
  | "Failed";

/**
 * Progress information for a transfer operation
 */
export interface TransferProgress {
  /** Unique operation ID */
  operation_id: string;
  /** Current phase of the operation */
  phase: TransferPhase;
  /** Total number of files to transfer */
  total_files: number;
  /** Number of files completed */
  files_completed: number;
  /** Total bytes to transfer */
  total_bytes: number;
  /** Bytes transferred so far */
  bytes_transferred: number;
  /** Current file being processed */
  current_file: string | null;
  /** Current file progress (0.0 - 1.0) */
  current_file_progress: number;
  /** Overall progress percentage (0.0 - 100.0) */
  overall_percent: number;
  /** Transfer rate in bytes per second */
  bytes_per_second: number;
  /** Estimated time remaining in seconds */
  eta_seconds: number | null;
  /** Whether the operation is cancelled */
  cancelled: boolean;
  /** Error message if any */
  error: string | null;
  /** Number of parallel threads being used */
  parallel_threads: number;
  /** Files currently being transferred (for parallel transfers) */
  active_files: string[];
}

/**
 * Result of a single file transfer
 */
export interface FileTransferResult {
  /** Source path */
  source: string;
  /** Destination path */
  destination: string;
  /** File size in bytes */
  size: number;
  /** Whether the transfer succeeded */
  success: boolean;
  /** Error message if failed */
  error: string | null;
  /** Source hash (if computed) */
  source_hash: string | null;
  /** Destination hash (if verified) */
  destination_hash: string | null;
  /** Whether hashes matched (if verified) */
  verified: boolean | null;
}

/**
 * Result of a complete transfer operation
 */
export interface TransferResult {
  /** Unique operation ID */
  operation_id: string;
  /** Whether the overall operation succeeded */
  success: boolean;
  /** Total files processed */
  total_files: number;
  /** Successfully transferred files */
  successful_files: number;
  /** Failed files */
  failed_files: number;
  /** Skipped files (already exist, etc.) */
  skipped_files: number;
  /** Total bytes transferred */
  bytes_transferred: number;
  /** Duration in milliseconds */
  duration_ms: number;
  /** Individual file results */
  files: FileTransferResult[];
  /** Error message if operation failed */
  error: string | null;
}

// =============================================================================
// API Functions
// =============================================================================

/**
 * Preview a transfer operation without executing it
 * 
 * Returns a summary of files that would be transferred.
 * 
 * @param sources - Array of source paths to preview
 * @param recursive - Whether to include subdirectories (default: true)
 * @returns Preview of transfer operation
 * 
 * @example
 * ```typescript
 * const preview = await transferPreview(["/path/to/folder"]);
 * console.log(`Would transfer ${preview.total_files} files (${preview.total_size_formatted})`);
 * ```
 */
export async function transferPreview(
  sources: string[],
  recursive: boolean = true
): Promise<TransferPreview> {
  return invoke<TransferPreview>("transfer_preview", { sources, recursive });
}

/**
 * Start a file transfer operation
 * 
 * Returns the operation ID which can be used to:
 * - Cancel the transfer with `transferCancel()`
 * - Track progress via the "transfer-progress" event
 * 
 * @param request - Transfer configuration
 * @returns Operation ID
 * 
 * @example
 * ```typescript
 * const opId = await transferStart({
 *   sources: ["/path/to/folder"],
 *   destination: "/path/to/destination",
 *   verify: true,
 * });
 * ```
 */
export async function transferStart(request: TransferRequest): Promise<string> {
  return invoke<string>("transfer_start", { request });
}

/**
 * Cancel an active transfer operation
 * 
 * @param operationId - The operation ID returned by `transferStart()`
 * @returns true if the operation was found and cancelled, false otherwise
 */
export async function transferCancel(operationId: string): Promise<boolean> {
  return invoke<boolean>("transfer_cancel", { operationId });
}

/**
 * Get list of active transfer operation IDs
 * 
 * @returns Array of operation IDs
 */
export async function transferListActive(): Promise<string[]> {
  return invoke<string[]>("transfer_list_active");
}

/**
 * Copy a single file with optional verification
 * 
 * Simpler API for single file copies without progress events.
 * 
 * @param source - Source file path
 * @param destination - Destination file path
 * @param verify - Whether to verify the copy with hash comparison
 * @param hashAlgorithm - Hash algorithm to use for verification
 * @returns Transfer result for the single file
 * 
 * @example
 * ```typescript
 * const result = await transferCopyFile(
 *   "/path/to/source.pdf",
 *   "/path/to/destination.pdf",
 *   true // verify
 * );
 * if (result.verified) {
 *   console.log("Copy verified successfully");
 * }
 * ```
 */
export async function transferCopyFile(
  source: string,
  destination: string,
  verify?: boolean,
  hashAlgorithm?: string
): Promise<FileTransferResult> {
  return invoke<FileTransferResult>("transfer_copy_file", {
    source,
    destination,
    verify,
    hash_algorithm: hashAlgorithm,
  });
}

/**
 * Copy an entire directory
 * 
 * @param source - Source directory path
 * @param destination - Destination directory path
 * @param verify - Whether to verify files after copying
 * @param recursive - Whether to copy subdirectories (default: true)
 * @returns Operation ID for tracking progress
 */
export async function transferCopyDirectory(
  source: string,
  destination: string,
  verify?: boolean,
  recursive?: boolean
): Promise<string> {
  return invoke<string>("transfer_copy_directory", {
    source,
    destination,
    verify,
    recursive,
  });
}

/**
 * Calculate the total size of files that would be transferred
 * 
 * @param sources - Array of source paths
 * @param recursive - Whether to include subdirectories
 * @returns Total size in bytes
 */
export async function transferCalculateSize(
  sources: string[],
  recursive: boolean = true
): Promise<number> {
  return invoke<number>("transfer_calculate_size", { sources, recursive });
}

// =============================================================================
// Event Listeners
// =============================================================================

/**
 * Listen for transfer progress events
 * 
 * @param callback - Function called with progress updates
 * @returns Unsubscribe function
 * 
 * @example
 * ```typescript
 * const unlisten = await onTransferProgress((progress) => {
 *   console.log(`${progress.overall_percent.toFixed(1)}% complete`);
 *   console.log(`Current file: ${progress.current_file}`);
 * });
 * 
 * // Later: stop listening
 * unlisten();
 * ```
 */
export async function onTransferProgress(
  callback: (progress: TransferProgress) => void
): Promise<UnlistenFn> {
  return listen<TransferProgress>("transfer-progress", (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for transfer completion events
 * 
 * @param callback - Function called when a transfer completes
 * @returns Unsubscribe function
 * 
 * @example
 * ```typescript
 * const unlisten = await onTransferComplete((result) => {
 *   if (result.success) {
 *     console.log(`Transferred ${result.successful_files} files`);
 *   } else {
 *     console.error(`Transfer failed: ${result.error}`);
 *   }
 * });
 * ```
 */
export async function onTransferComplete(
  callback: (result: TransferResult) => void
): Promise<UnlistenFn> {
  return listen<TransferResult>("transfer-complete", (event) => {
    callback(event.payload);
  });
}

// =============================================================================
// Utility Functions
// =============================================================================

/**
 * Format transfer speed to human-readable string
 * 
 * @param bytesPerSecond - Transfer rate in bytes per second
 * @returns Formatted string (e.g., "125 MB/s")
 */
export function formatSpeed(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`;
}

/**
 * Format ETA seconds to human-readable string
 * 
 * @param seconds - Estimated time remaining in seconds
 * @returns Formatted string (e.g., "2m 30s", "1h 15m")
 */
export function formatEta(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined || seconds < 0 || !isFinite(seconds)) {
    return "calculating...";
  }
  
  if (seconds < 60) {
    return `${Math.round(seconds)}s`;
  }
  
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  
  if (minutes < 60) {
    return `${minutes}m ${remainingSeconds}s`;
  }
  
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  
  return `${hours}h ${remainingMinutes}m`;
}
