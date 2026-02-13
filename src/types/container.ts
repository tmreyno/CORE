// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Container structure types for forensic evidence containers
 *
 * NOTE ON TYPE NAMING:
 * - `TreeEntry` (this file) - AD1-specific tree entry type matching Rust's `ad1::TreeEntry`
 *   Used for direct Tauri command responses from AD1 operations.
 * - `TreeEntryInfo` (types/lifecycle.ts) - Generic tree entry type for the unified
 *   container trait API. Used in the trait-based abstraction layer.
 */

// --- Container Structure Types ---

export type SegmentHeader = {
  signature: string;
  segment_index: number;
  segment_number: number;
  fragments_size: number;
  header_size: number;
};

export type LogicalHeader = {
  signature: string;
  image_version: number;
  zlib_chunk_size: number;
  logical_metadata_addr: number;
  first_item_addr: number;
  data_source_name_length: number;
  ad_signature: string;
  data_source_name_addr: number;
  attrguid_footer_addr: number;
  locsguid_footer_addr: number;
  data_source_name: string;
};

/**
 * AD1-specific tree entry type
 *
 * This type mirrors Rust's `ad1::TreeEntry` and contains AD1-specific fields
 * like `item_type`, `data_addr`, `first_child_addr` etc. for lazy loading
 * and hex viewer navigation.
 *
 * For a generic tree entry type, see `TreeEntryInfo` in types/lifecycle.ts.
 */
export type TreeEntry = {
  path: string;
  /** Item name only (without path) */
  name: string;
  is_dir: boolean;
  size: number;
  item_type: number;
  /** Address of first child item (for lazy loading directories) */
  first_child_addr?: number | null;
  /** Address of compressed data (for reading file content) */
  data_addr?: number | null;
  // === HEX LOCATION FIELDS ===
  /** Address of the item header in the container (hex location of this entry) */
  item_addr?: number | null;
  /** Size of compressed data in bytes (if file) */
  compressed_size?: number | null;
  /** Address where compressed data ends (data_addr + 4 + compressed_size) */
  data_end_addr?: number | null;
  /** Address of first metadata entry for this item */
  metadata_addr?: number | null;
  // === END HEX LOCATION FIELDS ===
  /** MD5 hash if stored in metadata */
  md5_hash?: string | null;
  /** SHA1 hash if stored in metadata */
  sha1_hash?: string | null;
  /** Created timestamp (ISO 8601) */
  created?: string | null;
  /** Accessed timestamp (ISO 8601) */
  accessed?: string | null;
  /** Modified timestamp (ISO 8601) */
  modified?: string | null;
  /** File attributes flags */
  attributes?: string[] | null;
  /** Number of child entries (for lazy loading indicator) */
  child_count?: number | null;
};

/**
 * Archive tree entry for ZIP, 7z, etc.
 */
export type ArchiveTreeEntry = {
  /** Path within the archive */
  path: string;
  /** Filename only */
  name: string;
  /** Whether this is a directory */
  isDir: boolean;
  /** Uncompressed size */
  size: number;
  /** Compressed size */
  compressedSize: number;
  /** CRC32 checksum */
  crc32: number;
  /** Last modified timestamp */
  modified: string;
};

/**
 * Nested container entry - unified type for entries inside nested containers
 * Used when expanding containers inside other containers (e.g., AD1 inside ZIP)
 */
export type NestedContainerEntry = {
  /** Path within the nested container */
  path: string;
  /** Filename only */
  name: string;
  /** Whether this is a directory */
  isDir: boolean;
  /** Uncompressed size */
  size: number;
  /** Hash/checksum if available */
  hash: string | null;
  /** Last modified timestamp */
  modified: string | null;
  /** Type of nested container this entry is from (ad1, zip, etc.) */
  sourceType: string;
  /** Whether this entry is itself a nested container */
  isNestedContainer: boolean;
  /** Container type if this is a nested container */
  nestedType: string | null;
};

/**
 * Nested container info - quick metadata for nested containers
 */
export type NestedContainerInfo = {
  /** Type of container (zip, ad1, e01, etc.) */
  containerType: string;
  /** Total entry count */
  entryCount: number;
  /** Total size (uncompressed) */
  totalSize: number;
  /** Whether container is encrypted */
  encrypted: boolean;
  /** Path where container was extracted (for forensic logging) */
  tempPath: string;
  /** Original path within parent container */
  originalPath: string;
};

/**
 * UFED tree entry for UFED containers
 */
export type UfedTreeEntry = {
  /** Path within the UFED container */
  path: string;
  /** Filename */
  name: string;
  /** Whether this is a directory */
  isDir: boolean;
  /** File size (0 for directories) */
  size: number;
  /** UFED-specific type (file, folder, extraction, etc.) */
  entryType: string;
  /** Associated hash if available */
  hash?: string | null;
  /** Modified timestamp if available */
  modified?: string | null;
};

/** AD1 container summary info for tree display */
export type Ad1ContainerSummary = {
  /** Total number of items in container */
  total_items: number;
  /** Total size of all files (decompressed) */
  total_size: number;
  /** Number of files */
  file_count: number;
  /** Number of directories */
  dir_count: number;
  /** Source data name from logical header */
  source_name?: string | null;
};

export type VerifyEntry = {
  path: string;
  status: string;
  message?: string;
};

// --- Segment Information Types ---

/** Detailed information about a single AD1 segment file */
export type SegmentFileInfo = {
  /** Segment number (1-based: .ad1, .ad2, etc.) */
  number: number;
  /** Full path to the segment file */
  path: string;
  /** File name only */
  filename: string;
  /** Segment file size in bytes */
  size: number;
  /** Whether the segment file exists */
  exists: boolean;
  /** Expected data size (excluding header margin) */
  data_size: number;
  /** Offset range this segment covers (start) */
  offset_start: number;
  /** Offset range this segment covers (end) */
  offset_end: number;
};

/** Summary of all segment files for an AD1 container */
export type SegmentSummary = {
  /** Total number of expected segments */
  expected_count: number;
  /** Number of segments found */
  found_count: number;
  /** Number of segments missing */
  missing_count: number;
  /** Total size of all segment files */
  total_size: number;
  /** Total data size (excluding headers) */
  total_data_size: number;
  /** List of segment file information */
  segments: SegmentFileInfo[];
  /** Whether the container is complete (all segments present) */
  is_complete: boolean;
};

// --- File Discovery Types ---

export type DiscoveredFile = {
  path: string;
  filename: string;
  container_type: string;
  size: number;
  segment_count?: number;
  created?: string;
  modified?: string;
};
