// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ContainerInfo, HashHistoryEntry } from "../../types";
import type { FileStatus, FileHashInfo } from "../../hooks";

/** Hash verification state */
export type HashState = "verified" | "failed" | "computed" | "stored" | "incomplete" | "none";

/** Props for HashBadge component */
export interface HashBadgeProps {
  /** Current file hash info from computation */
  fileHash?: FileHashInfo | null;
  /** Hash computation status */
  fileStatus?: FileStatus | null;
  /** Container info with stored hashes */
  containerInfo?: ContainerInfo | null;
  /** Hash history entries */
  hashHistory?: HashHistoryEntry[];
  /** Whether the file is busy (disable click) */
  busy?: boolean;
  /** Callback when badge is clicked (for hash/re-hash) */
  onHash?: () => void;
  /** Size variant */
  size?: "sm" | "md";
  /** Show count badge */
  showCount?: boolean;
}
