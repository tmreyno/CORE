// =============================================================================
// CORE-FFX - Forensic File Explorer
// Viewer Types - Hex viewer, metadata parser types
// =============================================================================

/**
 * Viewer Types
 * 
 * Types for the hex viewer, metadata parser, and file type detection.
 */

// ============================================================================
// Hex Viewer Types
// ============================================================================

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

// ============================================================================
// Metadata Parser Types
// ============================================================================

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

// ============================================================================
// File Type Detection Types
// ============================================================================

/** File type detection result */
export type FileTypeInfo = {
  mime_type: string | null;
  description: string;
  extension: string;
  is_text: boolean;
  is_forensic_format: boolean;
  magic_hex: string;
};
