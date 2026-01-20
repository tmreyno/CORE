// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Transfer Panel Types
 * 
 * Centralized type definitions for the transfer/export functionality.
 */

import type { TransferProgress, TransferResult } from "../../transfer";
import type { DiscoveredFile, HashHistoryEntry } from "../../types";

// =============================================================================
// Container & File Types
// =============================================================================

/** Container type detection result */
export type ContainerType = "e01" | "ad1" | "l01" | "ufed" | "raw" | "unknown";

/** File type for icon selection */
export type FileType = "container" | "image" | "video" | "audio" | "document" | "code" | "database" | "archive" | "unknown";

/** Transfer file status */
export type TransferFileStatus = "pending" | "transferring" | "hashing" | "completed" | "failed";

/** Job status */
export type JobStatus = "pending" | "running" | "completed" | "failed" | "cancelled";

// =============================================================================
// Tree Node Types
// =============================================================================

/** Tree node for file preview */
export interface FileTreeNode {
  name: string;
  path: string;
  size: number;
  sizeFormatted: string;
  isDirectory: boolean;
  fileType: FileType;
  containerType: ContainerType;
  children: FileTreeNode[];
  expanded: boolean;
  /** Transfer status for this file */
  status?: TransferFileStatus;
  /** Transfer progress (0-100) */
  progress?: number;
}

/** Props for FileTreeNode component */
export interface FileTreeNodeProps {
  node: FileTreeNode;
  depth: number;
  onToggle: (path: string) => void;
  currentFile?: string | null;
  fileProgress?: Map<string, { status: string; progress: number }>;
}

// =============================================================================
// Transfer Job Types
// =============================================================================

/** Active transfer job */
export interface TransferJob {
  id: string;
  sources: string[];
  destination: string;
  status: JobStatus;
  progress: TransferProgress | null;
  result: TransferResult | null;
  startTime: Date;
  endTime?: Date;
  /** Hash algorithm used for this job */
  hashAlgorithm: string;
  /** Whether container-aware hashing was used */
  containerAware: boolean;
  /** Per-file progress tracking */
  fileProgress?: Map<string, { status: string; progress: number }>;
}

// =============================================================================
// Component Props
// =============================================================================

/** TransferPanel main component props */
export interface TransferPanelProps {
  /** Currently selected files in the file manager */
  selectedFiles?: DiscoveredFile[];
  /** Current scan/project directory (default source) */
  scanDir?: string;
  /** Callback when transfer completes */
  onTransferComplete?: (result: TransferResult) => void;
  /** Callback when hashes are computed during transfer - for hash history integration */
  onHashComputed?: (entries: HashHistoryEntry[]) => void;
  /** Callback for real-time progress updates */
  onProgressUpdate?: (jobs: TransferJob[]) => void;
  /** Externally controlled active jobs (for persistence across tab switches) */
  activeJobs?: TransferJob[];
  /** Callback to update active jobs in parent (for persistence) */
  onActiveJobsChange?: (jobs: TransferJob[]) => void;
}

/** TransferOptions component props */
export interface TransferOptionsProps {
  verify: boolean;
  setVerify: (v: boolean) => void;
  hashAlgorithm: string;
  setHashAlgorithm: (v: string) => void;
  preserveTimestamps: boolean;
  setPreserveTimestamps: (v: boolean) => void;
  overwrite: boolean;
  setOverwrite: (v: boolean) => void;
  recursive: boolean;
  setRecursive: (v: boolean) => void;
  parallelThreads: number;
  setParallelThreads: (v: number) => void;
}

/** TransferJobCard component props */
export interface TransferJobCardProps {
  job: TransferJob;
  onCancel: (jobId: string) => void;
}

/** SourceList component props */
export interface SourceListProps {
  sources: string[];
  onRemoveSource: (path: string) => void;
  onBrowseFile: () => void;
  onBrowseFolder: () => void;
}
