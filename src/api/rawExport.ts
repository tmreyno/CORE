// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "../utils/platform";

// =============================================================================
// Types
// =============================================================================

export interface RawExportOptions {
  sourcePaths: string[];
  outputPath: string;
  segmentSize?: number;
  computeMd5?: boolean;
  computeSha1?: boolean;
  computeSha256?: boolean;
  caseNumber?: string;
  evidenceNumber?: string;
  examinerName?: string;
  description?: string;
  notes?: string;
}

export interface RawExportProgress {
  outputPath: string;
  currentFile: string;
  fileIndex: number;
  totalFiles: number;
  bytesWritten: number;
  totalBytes: number;
  percent: number;
  phase: string;
  currentSegment: number;
}

export interface RawExportResult {
  outputPath: string;
  bytesWritten: number;
  filesIncluded: number;
  segmentsCreated: number;
  md5Hash: string | null;
  sha1Hash: string | null;
  sha256Hash: string | null;
  durationMs: number;
}

// =============================================================================
// API Functions
// =============================================================================

/**
 * Create a raw disk image (.dd/.img) from source files/devices.
 * Progress is reported via the callback; resolves with the result.
 */
export async function createRawImage(
  options: RawExportOptions,
  onProgress?: (progress: RawExportProgress) => void
): Promise<RawExportResult> {
  if (!isTauri) {
    void options;
    void onProgress;
    throw new Error("Raw image creation is available in the desktop app.");
  }

  let unlisten: UnlistenFn | undefined;

  if (onProgress) {
    unlisten = await listen<RawExportProgress>(
      "raw-export-progress",
      (event) => {
        if (event.payload.outputPath === options.outputPath) {
          onProgress(event.payload);
        }
      }
    );
  }

  try {
    return await invoke<RawExportResult>("raw_create_image", { options });
  } finally {
    unlisten?.();
  }
}

/**
 * Cancel an in-progress raw image export.
 */
export async function cancelRawExport(
  outputPath: string
): Promise<boolean> {
  if (!isTauri) {
    void outputPath;
    return false;
  }

  return invoke<boolean>("raw_cancel_export", { outputPath });
}

/**
 * Build RawExportOptions from individual parameters.
 */
export function buildRawExportOptions(params: {
  sourcePaths: string[];
  outputPath: string;
  segmentSize?: number;
  computeMd5?: boolean;
  computeSha1?: boolean;
  computeSha256?: boolean;
  caseNumber?: string;
  evidenceNumber?: string;
  examinerName?: string;
  description?: string;
  notes?: string;
}): RawExportOptions {
  return {
    sourcePaths: params.sourcePaths,
    outputPath: params.outputPath,
    segmentSize: params.segmentSize,
    computeMd5: params.computeMd5 ?? true,
    computeSha1: params.computeSha1 ?? false,
    computeSha256: params.computeSha256 ?? true,
    caseNumber: params.caseNumber,
    evidenceNumber: params.evidenceNumber,
    examinerName: params.examinerName,
    description: params.description,
    notes: params.notes,
  };
}
