// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EWF/E01 Export API using libewf-ffi
 *
 * Provides forensic-grade E01 image creation with:
 * - EnCase 5/6/7 format support (.E01)
 * - V2 EnCase 7 format support (.Ex01) with BZIP2 compression
 * - Deflate/BZIP2 compression methods (none, fast, best)
 * - Full case metadata (case number, evidence number, examiner, etc.)
 * - MD5/SHA1 hash computation and embedding
 * - Progress tracking with cancellation support
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// =============================================================================
// Types
// =============================================================================

/**
 * Options for creating an E01/Ex01 forensic image
 *
 * Format values:
 * - "e01" / "encase5" → EnCase 5 (.E01) — most compatible
 * - "encase6" → EnCase 6 (.E01) — supports larger images & SHA1
 * - "encase7" → EnCase 7 (.E01) — EWF1 segment type
 * - "v2encase7" / "ex01" → V2 EnCase 7 (.Ex01) — EWF2, supports BZIP2
 * - "ftk" → FTK Imager (.E01)
 *
 * Compression method values:
 * - "deflate" (default) → Standard zlib compression
 * - "bzip2" → BZIP2 compression (requires V2 format: "v2encase7"/"ex01")
 * - "none" → No compression method
 */
export interface EwfExportOptions {
  /** Source file paths to include */
  sourcePaths: string[];
  /** Output path (base name without extension) */
  outputPath: string;
  /** Output format (see above for values). Default: "e01" */
  format?: string;
  /** Compression level: "none", "fast" (default), "best" */
  compression?: string;
  /**
   * Compression method: "deflate" (default), "bzip2", "none"
   * Note: "bzip2" requires a V2 format ("v2encase7" or "ex01")
   */
  compressionMethod?: string;
  /** Maximum segment file size in bytes */
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
  /** Whether to compute MD5 hash (default: true) */
  computeMd5?: boolean;
  /** Whether to compute SHA1 hash (default: false) */
  computeSha1?: boolean;
}

/**
 * Progress event emitted during E01 export
 */
export interface EwfExportProgress {
  /** Output path */
  outputPath: string;
  /** Current file being processed */
  currentFile: string;
  /** File index (1-based) */
  fileIndex: number;
  /** Total number of files */
  totalFiles: number;
  /** Bytes written so far */
  bytesWritten: number;
  /** Total bytes to write */
  totalBytes: number;
  /** Progress percentage (0-100) */
  percent: number;
  /** Current phase description */
  phase: string;
}

/**
 * Result of an E01 export operation
 */
export interface EwfExportResult {
  /** Path to the created E01 file */
  outputPath: string;
  /** Format used (e.g., "E01") */
  format: string;
  /** Total bytes written */
  bytesWritten: number;
  /** Number of files included */
  filesIncluded: number;
  /** Whether compression was used */
  compressed: boolean;
  /** MD5 hash of the data (if computed) */
  md5Hash: string | null;
  /** SHA1 hash of the data (if computed) */
  sha1Hash: string | null;
  /** Duration in milliseconds */
  durationMs: number;
}

// =============================================================================
// API Functions
// =============================================================================

/**
 * Get the libewf library version
 */
export async function getEwfVersion(): Promise<string> {
  return invoke<string>("ewf_get_version");
}

/**
 * Create an E01 forensic image from source files
 *
 * @param options Export configuration
 * @param onProgress Optional progress callback
 * @returns Export result with output path, hashes, and stats
 */
export async function createE01Image(
  options: EwfExportOptions,
  onProgress?: (progress: EwfExportProgress) => void,
): Promise<EwfExportResult> {
  let unlisten: UnlistenFn | undefined;

  try {
    // Set up progress listener
    if (onProgress) {
      unlisten = await listen<EwfExportProgress>(
        "ewf-export-progress",
        (event) => {
          onProgress(event.payload);
        },
      );
    }

    // Invoke the backend command
    const result = await invoke<EwfExportResult>("ewf_create_image", {
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
 * Cancel an in-progress E01 export
 *
 * @param outputPath The output path of the export to cancel
 * @returns true if the export was found and cancelled, false if not found
 */
export async function cancelE01Export(outputPath: string): Promise<boolean> {
  return invoke<boolean>("ewf_cancel_export", { outputPath });
}

// =============================================================================
// Helpers
// =============================================================================

/**
 * Build EwfExportOptions from common UI inputs
 */
export function buildEwfExportOptions(params: {
  sourcePaths: string[];
  outputPath: string;
  format?: string;
  compression?: string;
  compressionMethod?: string;
  caseNumber?: string;
  evidenceNumber?: string;
  examinerName?: string;
  description?: string;
  notes?: string;
  computeMd5?: boolean;
  computeSha1?: boolean;
}): EwfExportOptions {
  return {
    sourcePaths: params.sourcePaths,
    outputPath: params.outputPath,
    format: params.format ?? "e01",
    compression: params.compression ?? "fast",
    compressionMethod: params.compressionMethod ?? "deflate",
    caseNumber: params.caseNumber,
    evidenceNumber: params.evidenceNumber,
    examinerName: params.examinerName,
    description: params.description,
    notes: params.notes,
    computeMd5: params.computeMd5 ?? true,
    computeSha1: params.computeSha1 ?? false,
  };
}
