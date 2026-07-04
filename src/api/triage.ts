// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Forensic triage collection API.
 *
 * Wraps the `triage` Tauri commands for:
 * - Querying available triage profiles and artifact categories
 * - Performing triage collections (system artifacts, credentials, keys)
 * - Scanning collected files for secrets/credentials
 * - Cancelling in-progress triage operations
 * - Listening for triage progress events
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "../utils/platform";

// =============================================================================
// Types
// =============================================================================

/** A triage artifact category with its collection targets. */
export interface TriageCategory {
  id: string;
  name: string;
  description: string;
  artifactCount: number;
  /** Individual artifact names within this category (e.g., "SAM hive", "SSH keys"). */
  artifacts: string[];
}

/** A triage collection profile (preset). */
export interface TriageProfile {
  id: string;
  name: string;
  description: string;
  categories: string[];
}

/** Options for starting a triage collection. */
export interface TriageOptions {
  /** Output directory where collected artifacts are staged. */
  outputDir: string;
  /** Category IDs to collect (e.g., ["registry", "credentials", "browser"]). */
  categories: string[];
  /** Whether to scan collected text files for secrets/credentials. */
  scanForSecrets: boolean;
  /** Optional root path to collect from (default: system root). */
  targetRoot?: string;
  /** Maximum file size in bytes to collect (default: 100 MB). Files exceeding
   *  this are skipped to prevent hangs on very large system log files. */
  maxFileSize?: number;
  /** Optional container format for packaging collected artifacts (e.g., "7z"). */
  containerFormat?: string;
}

/** Progress event emitted during triage collection. */
export interface TriageProgress {
  /** Current phase: "collecting", "scanning", "packaging", "complete" */
  phase: string;
  /** Name of the file currently being processed */
  currentFile: string;
  /** Number of files collected so far */
  filesCollected: number;
  /** Total files to collect */
  filesTotal: number;
  /** Bytes collected so far */
  bytesCollected: number;
  /** Progress percentage (0–100) */
  percent: number;
  /** Current category being processed */
  currentCategory: string;
}

/** A detected secret/credential in a collected file. */
export interface SecretFinding {
  filePath: string;
  lineNumber: number;
  secretType: string;
  description: string;
  /** Redacted preview of the matched value. */
  preview: string;
  /** "high", "medium", or "low" */
  confidence: string;
}

/** Per-category collection breakdown. */
export interface CategoryResult {
  filesCollected: number;
  bytesCollected: number;
  filesSkipped: number;
  filesFailed: number;
  sampleFiles: string[];
}

/** Result of a completed triage collection. */
export interface TriageResult {
  outputDir: string;
  filesCollected: number;
  bytesCollected: number;
  /** Path to the packaged container file (e.g., .7z), if container_format was set. */
  containerPath?: string;
  filesSkipped: number;
  filesFailed: number;
  durationSecs: number;
  categoriesCollected: string[];
  categoryDetails: Record<string, CategoryResult>;
  secretFindings: SecretFinding[];
  cancelled: boolean;
}

// =============================================================================
// API Functions
// =============================================================================

/**
 * Get available triage profiles and categories for the current platform.
 *
 * @returns Tuple of [profiles, categories]
 */
export async function getTriageProfiles(): Promise<
  [TriageProfile[], TriageCategory[]]
> {
  if (!isTauri) {
    return [[], []];
  }

  return invoke<[TriageProfile[], TriageCategory[]]>("triage_get_profiles");
}

/**
 * Execute a triage collection.
 *
 * Collects system artifacts, security files, and credential-related files
 * based on the selected categories. Optionally scans collected files for
 * secrets, API keys, tokens, and private keys.
 *
 * @param options - Collection options
 * @returns Triage result with counts, findings, and timing
 */
export async function triageCollect(
  options: TriageOptions
): Promise<TriageResult> {
  if (!isTauri) {
    throw new Error("Forensic triage collection is available in the desktop app.");
  }

  return invoke<TriageResult>("triage_collect", { options });
}

/**
 * Cancel an in-progress triage collection.
 */
export async function triageCancel(): Promise<void> {
  if (!isTauri) {
    return;
  }

  return invoke<void>("triage_cancel");
}

/**
 * Listen for triage progress events.
 *
 * @param callback - Called with progress updates during collection
 * @returns Unlisten function to stop receiving events
 */
export async function listenTriageProgress(
  callback: (progress: TriageProgress) => void
): Promise<UnlistenFn> {
  if (!isTauri) {
    void callback;
    return () => {};
  }

  return listen<TriageProgress>("triage-progress", (event) => {
    callback(event.payload);
  });
}
