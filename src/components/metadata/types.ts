// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ParsedMetadata, MetadataField } from "../HexViewer";
import type { ContainerInfo } from "../../types";
import type { SelectedEntry } from "../EvidenceTree";

// Re-export commonly used types for sub-components
export type { ParsedMetadata, MetadataField, ContainerInfo, SelectedEntry };

/** File info passed from parent */
export interface FileInfo {
  path: string;
  filename: string;
  size: number;
  created?: string;
  modified?: string;
  container_type?: string;
  segment_count?: number;
}

/** Props for the main MetadataPanel component */
export interface MetadataPanelProps {
  metadata: ParsedMetadata | null;
  fileInfo?: FileInfo | null;
  containerInfo?: ContainerInfo;
  selectedOffset?: number | null;
  onRegionClick?: (offset: number, size?: number) => void;
  /** Currently selected entry from evidence tree (for hex location display) */
  selectedEntry?: SelectedEntry | null;
}

/** Preferred category display order */
export const CATEGORY_ORDER = [
  "Format",
  "Case Info",
  "Acquisition",
  "Device",
  "Volume",
  "Hashes",
  "Errors",
  "Sections",
  "General",
];

/** Reusable style constants shared across metadata sub-components */
export const ROW_STYLES = {
  rowBase:
    "grid gap-2 py-1 px-2 text-[10px] leading-tight items-baseline transition-colors hover:bg-bg-panel/50",
  rowGrid: "grid-cols-[minmax(80px,1fr)_minmax(100px,2fr)_auto]",
  rowClickable: "cursor-pointer hover:bg-accent/30",
  keyStyle: "text-txt-muted truncate",
  valueStyle: "font-mono text-txt-tertiary truncate",
  offsetStyle: "font-mono text-[9px] text-txt-muted whitespace-nowrap",
  offsetClickable: "text-accent",
  categoryHeader:
    "flex items-center gap-1.5 py-1.5 px-2 bg-bg-panel/50 cursor-pointer select-none hover:bg-bg-panel transition-colors",
} as const;

export type RowStyles = typeof ROW_STYLES;
