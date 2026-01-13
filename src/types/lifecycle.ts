// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Evidence Lifecycle Types
 *
 * This module defines the types for the evidence container lifecycle,
 * mirroring the Rust `containers/traits.rs` for frontend consistency.
 *
 * @module lifecycle
 */

// =============================================================================
// LIFECYCLE STAGES
// =============================================================================

/**
 * Evidence container lifecycle stages
 *
 * Represents the stages a container goes through during analysis:
 *
 * ```
 * Discovered → Detected → Opened → Verified → Extracted
 *     ↓           ↓          ↓         ↓          ↓
 * scan_dir   detect()   info()   verify()   extract()
 * ```
 */
export type LifecycleStage =
  | "discovered"  // File found during directory scan
  | "detected"    // Format identified by magic bytes
  | "opened"      // Metadata parsed
  | "verified"    // Hashes verified
  | "extracted"   // Contents exported
  | "error";      // Error state

/**
 * Lifecycle stage metadata
 */
export interface LifecycleStageInfo {
  stage: LifecycleStage;
  name: string;
  description: string;
  icon: string;
}

/**
 * All lifecycle stages with metadata
 */
export const LIFECYCLE_STAGES: Record<LifecycleStage, LifecycleStageInfo> = {
  discovered: {
    stage: "discovered",
    name: "Discovered",
    description: "Container file found during directory scan",
    icon: "🔍",
  },
  detected: {
    stage: "detected",
    name: "Detected",
    description: "Format identified by magic bytes/signature",
    icon: "📋",
  },
  opened: {
    stage: "opened",
    name: "Opened",
    description: "Container opened and metadata parsed",
    icon: "📂",
  },
  verified: {
    stage: "verified",
    name: "Verified",
    description: "Integrity verified (hashes checked)",
    icon: "✅",
  },
  extracted: {
    stage: "extracted",
    name: "Extracted",
    description: "Contents extracted to output directory",
    icon: "📤",
  },
  error: {
    stage: "error",
    name: "Error",
    description: "An error occurred during processing",
    icon: "❌",
  },
};

// =============================================================================
// VERIFICATION TYPES
// =============================================================================

/**
 * Verification status
 */
export type VerifyStatus =
  | "verified"       // All hashes match
  | "computed"       // Hash computed (no stored hash to compare)
  | "mismatch"       // Hash mismatch detected
  | "not-supported"  // Format doesn't support verification
  | "error";         // Verification failed

/**
 * Hash verification result
 */
export interface HashResult {
  /** Hash algorithm used */
  algorithm: string;
  /** Computed hash value */
  computed: string;
  /** Expected hash value (if stored) */
  expected?: string | null;
  /** Whether hashes match */
  verified?: boolean | null;
  /** Time taken to compute (seconds) */
  durationSecs: number;
}

/**
 * Per-chunk verification result
 */
export interface ChunkVerifyResult {
  /** Chunk index */
  index: number;
  /** Status ("ok", "mismatch", "error") */
  status: string;
  /** Error message if any */
  message?: string | null;
}

/**
 * Container verification result
 */
export interface VerifyResult {
  /** Overall verification status */
  status: VerifyStatus;
  /** Hash verification results */
  hashes: HashResult[];
  /** Per-chunk/block verification (if applicable) */
  chunks: ChunkVerifyResult[];
  /** Verification messages/warnings */
  messages: string[];
}

// =============================================================================
// SEGMENT TYPES
// =============================================================================

/**
 * Segment information for multi-segment containers
 */
export interface SegmentInfo {
  /** Total number of segments */
  count: number;
  /** Paths to all segment files */
  files: string[];
  /** Size of each segment in bytes */
  sizes: number[];
  /** Total size of all segments */
  totalSize: number;
  /** Missing segment files (gaps) */
  missing: string[];
}

/**
 * Metadata for a single segment
 */
export interface SegmentMetadata {
  index: number;
  path: string;
  size: number;
  hash?: string | null;
}

// =============================================================================
// CONTAINER METADATA TYPES
// =============================================================================

/**
 * Case metadata from container
 */
export interface CaseMetadata {
  caseNumber?: string | null;
  evidenceNumber?: string | null;
  examinerName?: string | null;
  description?: string | null;
  notes?: string | null;
  acquisitionDate?: string | null;
}

/**
 * Stored hash information
 */
export interface StoredHashInfo {
  /** Algorithm (e.g., "MD5", "SHA1") */
  algorithm: string;
  /** Hash value (hex string) */
  hash: string;
  /** Hash source ("container", "companion", "computed") */
  source: string;
  /** Verification status */
  verified?: boolean | null;
}

/**
 * Generic container metadata (returned by info())
 */
export interface ContainerMetadata {
  /** Format identifier */
  format: string;
  /** Format version (if applicable) */
  version?: string | null;
  /** Total logical size of data */
  totalSize: number;
  /** Segment information */
  segments?: SegmentInfo | null;
  /** Stored hashes */
  storedHashes: StoredHashInfo[];
  /** Case/evidence metadata */
  caseInfo?: CaseMetadata | null;
  /** Additional format-specific data */
  formatSpecific?: unknown;
}

// =============================================================================
// TREE ENTRY TYPES
// =============================================================================

/**
 * Generic tree entry information for the unified container trait API
 *
 * This is the format-agnostic tree entry type used in the trait-based
 * abstraction layer (containers::traits::TreeEntryInfo in Rust).
 *
 * For AD1-specific tree entries with format-specific fields like
 * `item_type`, `data_addr`, `first_child_addr`, see `TreeEntry` in types.ts.
 *
 * Key differences from TreeEntry:
 * - Uses `isDirectory` instead of `is_dir` (naming convention)
 * - No format-specific fields (item_type, data_addr, etc.)
 * - Single `hash` field instead of separate md5_hash/sha1_hash
 */
export interface TreeEntryInfo {
  path: string;
  name: string;
  isDirectory: boolean;
  size: number;
  created?: string | null;
  modified?: string | null;
  accessed?: string | null;
  hash?: string | null;
}

// =============================================================================
// ERROR TYPES
// =============================================================================

/**
 * Container error type
 */
export type ContainerErrorKind =
  | "file-not-found"
  | "invalid-format"
  | "unsupported-operation"
  | "io-error"
  | "parse-error"
  | "hash-mismatch"
  | "segment-error"
  | "extraction-error";

/**
 * Container error
 */
export interface ContainerError {
  kind: ContainerErrorKind;
  message: string;
  path?: string;
  details?: unknown;
}

// =============================================================================
// FORMAT INFO TYPES
// =============================================================================

/**
 * Basic information about a container format
 * (Mirrors FormatInfo from Rust traits.rs)
 */
export interface FormatInfo {
  /** Unique identifier (e.g., "e01", "ad1") */
  id: string;
  /** Display name (e.g., "Expert Witness Format") */
  name: string;
  /** File extensions (lowercase, no dot) */
  extensions: readonly string[];
  /** Format category */
  category: string;
  /** Whether format supports segmentation */
  supportsSegments: boolean;
  /** Whether format stores hashes internally */
  storesHashes: boolean;
  /** Whether format contains file/folder tree */
  hasFileTree: boolean;
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/**
 * Get lifecycle stage from status string
 */
export function getLifecycleStage(status: string): LifecycleStageInfo {
  const stage = status.toLowerCase() as LifecycleStage;
  return LIFECYCLE_STAGES[stage] ?? LIFECYCLE_STAGES.error;
}

/**
 * Check if a lifecycle stage is complete
 */
export function isStageComplete(current: LifecycleStage, target: LifecycleStage): boolean {
  const stages: LifecycleStage[] = ["discovered", "detected", "opened", "verified", "extracted"];
  const currentIdx = stages.indexOf(current);
  const targetIdx = stages.indexOf(target);
  return currentIdx >= targetIdx && current !== "error";
}

/**
 * Get verification status icon
 */
export function getVerifyStatusIcon(status: VerifyStatus): string {
  switch (status) {
    case "verified":
      return "✅";
    case "computed":
      return "🔢";
    case "mismatch":
      return "❌";
    case "not-supported":
      return "⚠️";
    case "error":
      return "❗";
  }
}

/**
 * Format hash result for display
 */
export function formatHashResult(result: HashResult): string {
  const status = result.verified === true
    ? "✅ Verified"
    : result.verified === false
      ? "❌ Mismatch"
      : "🔢 Computed";
  return `${result.algorithm.toUpperCase()}: ${result.computed.substring(0, 16)}... ${status}`;
}
