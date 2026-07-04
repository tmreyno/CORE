// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "../utils/platform";

// ─── Types ──────────────────────────────────────────────────────────────────

export interface Aff4ExportOptions {
  sourcePaths: string[];
  outputPath: string;
  compression?: string;
  hashAlgorithms?: string[];
  caseNumber?: string;
  evidenceNumber?: string;
  examinerName?: string;
  description?: string;
  notes?: string;
}

export interface Aff4ExportProgress {
  outputPath: string;
  phase: string;
  currentFile: string;
  filesProcessed: number;
  totalFiles: number;
  bytesProcessed: number;
  totalBytes: number;
  percent: number;
}

export interface Aff4ExportResult {
  outputPath: string;
  volumeUrn: string;
  imageUrn: string;
  totalBytes: number;
  containerBytes: number;
  compressionRatio: number;
  bevyCount: number;
  fileCount: number;
  linearHashes: Record<string, string>;
  durationMs: number;
}

// ─── API Functions ──────────────────────────────────────────────────────────

export async function createAff4Image(
  options: Aff4ExportOptions,
  onProgress?: (progress: Aff4ExportProgress) => void,
): Promise<Aff4ExportResult> {
  if (!isTauri) {
    void options;
    void onProgress;
    throw new Error("AFF4 image creation is available in the desktop app.");
  }

  let unlisten: UnlistenFn | undefined;

  try {
    if (onProgress) {
      unlisten = await listen<Aff4ExportProgress>("aff4-export-progress", (event) => {
        if (event.payload.outputPath === options.outputPath) {
          onProgress(event.payload);
        }
      });
    }

    const result = await invoke<Aff4ExportResult>("aff4_create_image", { options });
    return result;
  } finally {
    unlisten?.();
  }
}

export async function cancelAff4Export(outputPath: string): Promise<boolean> {
  if (!isTauri) {
    void outputPath;
    return false;
  }

  return invoke<boolean>("aff4_cancel_export", { outputPath });
}
