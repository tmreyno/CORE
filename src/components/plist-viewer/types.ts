// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { PlistMetadataSection } from "../../types/viewerMetadata";
import type { HashSourceInput } from "../../api/commands";

export interface FlatPlistEntry {
  key_path: string;
  value_type: string;
  value_preview: string;
}

export interface PlistInfo {
  path: string;
  format: string;
  root_type: string;
  entry_count: number;
  entries: FlatPlistEntry[];
}

export interface PlistViewerProps {
  /** Path to the plist file */
  path: string;
  /** Optional evidence source for container or nested plist entries */
  source?: HashSourceInput | null;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: PlistMetadataSection) => void;
}
