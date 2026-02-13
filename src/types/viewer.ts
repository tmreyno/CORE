// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Viewer types for hex viewer and file type detection
 */

/** A chunk of file data for hex viewer display */
export type FileChunk = {
  bytes: number[];
  offset: number;
  total_size: number;
  has_more: boolean;
  has_prev: boolean;
};

/** A highlighted region in the hex viewer */
export type HeaderRegion = {
  start: number;
  end: number;
  name: string;
  /** CSS class name for coloring */
  color_class: string;
  description: string;
};

/** A parsed metadata field from a file header */
export type MetadataField = {
  key: string;
  value: string;
  category: string;
  linked_region?: string;
  source_offset?: number;
};

/** Parsed metadata from a file header */
export type ParsedMetadata = {
  format: string;
  version: string | null;
  fields: MetadataField[];
  regions: HeaderRegion[];
};

/** File type detection result */
export type FileTypeInfo = {
  mime_type: string | null;
  description: string;
  extension: string;
  is_text: boolean;
  is_forensic_format: boolean;
  magic_hex: string;
};
