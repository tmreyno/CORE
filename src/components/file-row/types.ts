// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { DiscoveredFile, ContainerInfo, HashHistoryEntry } from "../../types";
import type { FileStatus, FileHashInfo } from "../../hooks";

export interface FileRowProps {
  file: DiscoveredFile;
  index: number;
  isSelected: boolean;
  isActive: boolean;
  isFocused: boolean;
  isHovered: boolean;
  fileStatus: FileStatus | undefined;
  fileInfo: ContainerInfo | undefined;
  fileHash: FileHashInfo | undefined;
  hashHistory: HashHistoryEntry[];
  busy: boolean;
  onSelect: () => void;
  onToggleSelection: () => void;
  onHash: () => void;
  onMouseEnter: () => void;
  onMouseLeave: () => void;
  onContextMenu?: (e: MouseEvent) => void;
}

export interface FileTooltipProps {
  file: DiscoveredFile;
  fileInfo: ContainerInfo | undefined;
  fileHash: FileHashInfo | undefined;
}

export type HashState = "incomplete" | "verified" | "failed" | "computed" | "stored" | "none";
