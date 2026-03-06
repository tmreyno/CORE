// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { BinaryMetadataSection } from "../../types/viewerMetadata";

// ============================================================================
// Types (matching Rust structs)
// ============================================================================

export interface ImportInfo {
  library: string;
  functions: string[];
  function_count: number;
}

export interface ExportInfo {
  name: string;
  ordinal: number;
  address: number;
}

export interface SectionInfo {
  name: string;
  virtual_address: number;
  virtual_size: number;
  raw_size: number;
  characteristics: string;
}

export interface BinaryInfo {
  path: string;
  format: string;
  architecture: string;
  is_64bit: boolean;
  entry_point: number | null;
  imports: ImportInfo[];
  exports: ExportInfo[];
  sections: SectionInfo[];
  file_size: number;
  // PE-specific
  pe_timestamp: number | null;
  pe_checksum: number | null;
  pe_subsystem: string | null;
  // Mach-O specific
  macho_cpu_type: string | null;
  macho_filetype: string | null;
  // Forensic indicators
  has_debug_info: boolean;
  is_stripped: boolean;
  has_code_signing: boolean;
}

export interface BinaryViewerProps {
  /** Path to the binary file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: BinaryMetadataSection) => void;
}
