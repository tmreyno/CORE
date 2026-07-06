// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import type { CompanionFile } from "./companion";
import { isTauri } from "../utils/platform";

// --- Types matching src-tauri/src/commands/companion.rs ---

/** A companion file discovered during a directory scan */
export interface DiscoveredAcquisition {
  /** Path to the .ffx-companion.json file */
  companionPath: string;
  /** Parsed companion file contents */
  companion: CompanionFile;
  /** Whether the primary output file/directory still exists on disk */
  outputExists: boolean;
  /** Size of the output file in bytes (if it exists and is a file) */
  outputSize?: number;
}

/** Result of importing acquisitions into a project */
export interface ImportResult {
  imported: number;
  skipped: number;
  errors: string[];
}

// --- Invoke wrappers ---

/**
 * Recursively scan a directory for `.ffx-companion.json` sidecar files.
 * Returns parsed companion files with output existence checks.
 */
export async function scanForAcquisitions(
  dirPath: string,
): Promise<DiscoveredAcquisition[]> {
  if (!isTauri) {
    throw new Error(
      "Acquisition directory scanning is available in the desktop app.",
    );
  }

  return invoke<DiscoveredAcquisition[]>("scan_for_acquisitions", { dirPath });
}
