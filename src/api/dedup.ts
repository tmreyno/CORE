// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Frontend API for file deduplication analysis.
 *
 * Wraps Tauri invoke calls with proper TypeScript types.
 */

import { invoke } from "@tauri-apps/api/core";

// =============================================================================
// Types (mirror Rust types with camelCase)
// =============================================================================

/** Options for controlling deduplication analysis */
export interface DedupOptions {
  /** Include 0-byte files (default: false) */
  includeEmptyFiles?: boolean;
  /** Include size-only matches (same size, different name — lower confidence) */
  includeSizeOnlyMatches?: boolean;
  /** Minimum file size in bytes */
  minFileSize?: number;
  /** Maximum file size in bytes */
  maxFileSize?: number;
  /** Filter to specific extensions (empty = all) */
  extensions?: string[];
  /** Filter to specific categories (empty = all) */
  categories?: string[];
  /** Filter to a specific container path */
  containerPath?: string;
}

/** Complete deduplication analysis results */
export interface DedupResults {
  /** All groups of duplicate files, sorted by wasted bytes descending */
  groups: DuplicateGroup[];
  /** Summary statistics */
  stats: DedupStats;
}

/** A group of files believed to be duplicates */
export interface DuplicateGroup {
  /** Unique group identifier */
  id: string;
  /** Representative display name */
  representativeName: string;
  /** File size in bytes (all files in the group share this size) */
  fileSize: number;
  /** Number of files in this group */
  fileCount: number;
  /** Bytes wasted by duplicates: (fileCount - 1) * fileSize */
  wastedBytes: number;
  /** How the duplicates were identified */
  matchType: DuplicateMatchType;
  /** Whether files come from different containers */
  crossContainer: boolean;
  /** File extension (if all files share one) */
  extension: string;
  /** File category */
  fileCategory: string;
  /** All files in this duplicate group */
  files: DuplicateFile[];
}

/** How duplicates were matched */
export type DuplicateMatchType = "exactHash" | "sizeAndName" | "sizeOnly";

/** A single file within a duplicate group */
export interface DuplicateFile {
  /** Container file path */
  containerPath: string;
  /** Container type (ad1, e01, zip, etc.) */
  containerType: string;
  /** Path within the container */
  entryPath: string;
  /** Filename */
  filename: string;
  /** File size in bytes */
  size: number;
  /** Last modified timestamp (unix) */
  modified: number;
  /** Hash value if available */
  hash: string | null;
  /** File category */
  fileCategory: string;
}

/** Summary statistics for a deduplication analysis */
export interface DedupStats {
  /** Total files scanned */
  totalFilesScanned: number;
  /** Number of duplicate groups */
  totalDuplicateGroups: number;
  /** Total duplicate files across all groups */
  totalDuplicateFiles: number;
  /** Total wasted bytes */
  totalWastedBytes: number;
  /** Unique files */
  uniqueFiles: number;
  /** Analysis time in ms */
  elapsedMs: number;
}

// =============================================================================
// API Functions
// =============================================================================

/** Run deduplication analysis on all indexed containers */
export async function analyzeDuplicates(
  options: DedupOptions = {}
): Promise<DedupResults> {
  return invoke<DedupResults>("dedup_analyze", { options });
}

/** Enrich results with stored hashes */
export async function enrichWithHashes(
  results: DedupResults,
  hashMap: Record<string, string>
): Promise<DedupResults> {
  return invoke<DedupResults>("dedup_enrich_hashes", { results, hashMap });
}

/** Export results as CSV string */
export async function exportDedupCsv(
  results: DedupResults
): Promise<string> {
  return invoke<string>("dedup_export_csv", { results });
}

// =============================================================================
// Helpers
// =============================================================================

/** Format byte count to human-readable string */
export function formatBytes(bytes: number): string {
  if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${bytes} B`;
}

/** Get display label for match type */
export function matchTypeLabel(type: DuplicateMatchType): string {
  switch (type) {
    case "exactHash":
      return "Exact Hash Match";
    case "sizeAndName":
      return "Size + Name Match";
    case "sizeOnly":
      return "Size Only";
    default:
      return "Unknown";
  }
}

/** Get CSS color class for match type confidence */
export function matchTypeColor(type: DuplicateMatchType): string {
  switch (type) {
    case "exactHash":
      return "text-success";
    case "sizeAndName":
      return "text-warning";
    case "sizeOnly":
      return "text-txt-muted";
    default:
      return "text-txt-muted";
  }
}
