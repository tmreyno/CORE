// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared types for the detail panel system.
 */

import type { DiscoveredFile, ContainerInfo, TreeEntry, HashHistoryEntry, HashAlgorithm, StoredHash } from "../../types";
import type { FileStatus, FileHashInfo } from "../../hooks";

export type RowType = "normal" | "highlight" | "device" | "full-width" | "hash" | "warning";
export type RowFormat = "text" | "bytes" | "mono" | "notes" | "list" | "warning";

export interface InfoField {
  label: string;
  value: string | number | undefined | null;
  type?: RowType;
  format?: RowFormat;
  condition?: boolean;
}

export interface DetailPanelContentProps {
  activeFile: DiscoveredFile | null;
  fileInfo: ContainerInfo | undefined;
  fileHash: FileHashInfo | undefined;
  fileStatus: FileStatus | undefined;
  tree: TreeEntry[];
  filteredTree: TreeEntry[];
  treeFilter: string;
  onTreeFilterChange: (filter: string) => void;
  selectedHashAlgorithm: HashAlgorithm;
  hashHistory: HashHistoryEntry[];
  storedHashes: StoredHash[];
  busy: boolean;
  onLoadInfo: () => void;
  formatHashDate: (timestamp: string) => string;
}
