// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Hex viewer types.
 */

import type { DiscoveredFile, ParsedMetadata } from "../../types";
import type { SelectedEntry } from "../EvidenceTree/types";

export interface HexViewerProps {
  /** Regular disk file */
  file?: DiscoveredFile | null;
  /** Container entry (file inside AD1/E01/etc.) */
  entry?: SelectedEntry;
  onMetadataLoaded?: (metadata: ParsedMetadata | null) => void;
  onNavigatorReady?: (navigateTo: (offset: number, size?: number) => void) => void;
}
