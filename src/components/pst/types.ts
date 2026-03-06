// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { PstMetadataSection } from "../../types/viewerMetadata";

export interface PstViewerProps {
  /** Path to the PST/OST file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: PstMetadataSection) => void;
}
