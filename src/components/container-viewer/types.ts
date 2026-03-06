// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { SelectedEntry } from "../EvidenceTree";
import type { ViewerMetadata, ViewerMetadataSection } from "../../types/viewerMetadata";

/** View mode: hex and text are guaranteed to work, preview uses native viewers */
export type EntryViewMode = "auto" | "hex" | "text" | "document" | "preview";

export interface ContainerEntryViewerProps {
  /** The selected entry to display */
  entry: SelectedEntry;
  /** View mode: hex, text, auto, document, or preview */
  viewMode: EntryViewMode;
  /** Callback when user wants to go back/close this view */
  onBack?: () => void;
  /** Callback when user toggles view mode */
  onViewModeChange?: (mode: EntryViewMode) => void;
  /** Callback when viewer metadata is available (for right panel) */
  onMetadata?: (metadata: ViewerMetadata | null) => void;
}

export interface ContentDetectResult {
  format: string;
  viewerType: string;
  description: string;
  mimeType: string;
  method: string;
}

// Re-export for consumers
export type { SelectedEntry, ViewerMetadata, ViewerMetadataSection };
