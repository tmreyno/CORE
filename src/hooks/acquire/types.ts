// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Types for the unified acquisition runner.
 *
 * Drives the checkbox-driven capture workflow:
 *   Selection → ordered execution → inline evidence collection review.
 */

// -----------------------------------------------------------------------------
// Task types
// -----------------------------------------------------------------------------

export type AcquisitionTaskType =
  | "memory"
  | "triage"
  | "physical"
  | "aff4"
  | "logical"
  | "export";

export type AcquisitionTaskStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

/**
 * Forensic priority ordering — lower runs first.
 *
 *  0  Memory   (volatile — capture RAM before anything else)
 *  1  Triage   (semi-volatile system artifacts, credentials)
 *  2  Physical (full disk images — E01 or Raw)
 *  3  AFF4     (AFF4 forensic containers)
 *  4  Logical  (L01 logical evidence containers)
 *  5  Export   (7z archive or file copy)
 */
export const ACQUISITION_PRIORITY: Record<AcquisitionTaskType, number> = {
  memory: 0,
  triage: 1,
  physical: 2,
  aff4: 3,
  logical: 4,
  export: 5,
};

// -----------------------------------------------------------------------------
// Task configuration
// -----------------------------------------------------------------------------

export interface AcquisitionTaskConfig {
  // Physical / Logical format
  format?: "e01" | "raw" | "l01" | "aff4" | "7z" | "copy";
  compression?: "none" | "fast" | "best";
  segmentSize?: number; // bytes — 0 = no splitting
  hashMd5?: boolean;
  hashSha1?: boolean;
  hashSha256?: boolean;

  // Triage-specific
  triageCategories?: string[];
  scanSecrets?: boolean;

  // Case metadata (inherited from project settings)
  caseNumber?: string;
  evidenceNumber?: string;
  examiner?: string;
  description?: string;
  notes?: string;
}

// Default config per task type
export function defaultConfig(type: AcquisitionTaskType): AcquisitionTaskConfig {
  switch (type) {
    case "memory":
      return { hashMd5: true, hashSha256: true };
    case "triage":
      return { scanSecrets: true, triageCategories: [] };
    case "physical":
      return {
        format: "e01",
        compression: "none",
        segmentSize: 2048 * 1024 * 1024,
        hashMd5: true,
      };
    case "logical":
      return {
        format: "l01",
        compression: "none",
        segmentSize: 2048 * 1024 * 1024,
        hashMd5: true,
      };
    case "aff4":
      return {
        format: "aff4",
        compression: "none",
        segmentSize: 0,
        hashSha256: true,
      };
    case "export":
      return {
        format: "7z",
        compression: "none",
        segmentSize: 2048 * 1024 * 1024,
        hashSha256: true,
      };
  }
}

// -----------------------------------------------------------------------------
// Progress & result
// -----------------------------------------------------------------------------

export interface AcquisitionTaskProgress {
  percent: number;
  bytesProcessed: number;
  totalBytes: number;
  currentFile?: string;
  phase?: string;
}

export interface AcquisitionTaskResult {
  outputPath: string;
  outputSize: number;
  hashes: Record<string, string>;
  durationMs: number;
  totalFiles?: number;
  segments?: number;
}

// -----------------------------------------------------------------------------
// The task object
// -----------------------------------------------------------------------------

export interface AcquisitionTask {
  id: string;
  type: AcquisitionTaskType;
  label: string;
  source: string;      // Device/path
  sourceLabel: string;  // Human-readable label
  status: AcquisitionTaskStatus;
  config: AcquisitionTaskConfig;
  progress?: AcquisitionTaskProgress;
  result?: AcquisitionTaskResult;
  error?: string;
  startedAt?: string;
  completedAt?: string;
  /** ID of the evidence collection record created on start */
  collectionId?: string;
  /** Whether the inline evidence collection form is expanded */
  collectionExpanded?: boolean;
}

// -----------------------------------------------------------------------------
// Runner phase
// -----------------------------------------------------------------------------

export type AcquisitionPhase = "idle" | "running" | "complete";
