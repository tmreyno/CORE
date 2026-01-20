// =============================================================================
// CORE-FFX - Forensic File Explorer
// Container Types - Types specific to forensic container formats
// =============================================================================

/**
 * Container Structure Types
 * 
 * Types for working with container headers, trees, and metadata.
 * Mirrors Rust types from various container parsers.
 */

// ============================================================================
// AD1 Container Types
// ============================================================================

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

export type Ad1VolumeInfo = {
  volume_label?: string | null;
  filesystem?: string | null;
  os_info?: string | null;
  block_size?: number | null;
  volume_serial?: string | null;
};

export type Ad1CompanionLogInfo = {
  case_number?: string | null;
  evidence_number?: string | null;
  examiner?: string | null;
  notes?: string | null;
  md5_hash?: string | null;
  sha1_hash?: string | null;
  sha256_hash?: string | null;
  acquisition_date?: string | null;
  source_device?: string | null;
  source_path?: string | null;
  acquisition_tool?: string | null;
  total_items?: number | null;
  total_size?: number | null;
  acquisition_method?: string | null;
  organization?: string | null;
};

export type Ad1ContainerSummary = {
  total_items: number;
  total_size: number;
  file_count: number;
  dir_count: number;
  source_name?: string | null;
};

// ============================================================================
// Archive Types (ZIP, 7z, etc.)
// ============================================================================

export type ArchiveTreeEntry = {
  path: string;
  name: string;
  isDir: boolean;
  size: number;
  compressedSize: number;
  crc32: number;
  modified: string;
};

export type ArchiveInfo = {
  format: string;
  segment_count: number;
  total_size: number;
  segment_names: string[];
  segment_sizes: number[];
  first_segment: string;
  last_segment: string;
  is_multipart: boolean;
  entry_count?: number | null;
  encrypted_headers: boolean;
  aes_encrypted: boolean;
  central_dir_offset?: number | null;
  central_dir_size?: number | null;
  next_header_offset?: number | null;
  next_header_size?: number | null;
  version?: string | null;
  start_header_crc_valid?: boolean | null;
  next_header_crc?: number | null;
  cellebrite_detected?: boolean;
  cellebrite_files?: string[];
};

// ============================================================================
// Nested Container Types
// ============================================================================

export type NestedContainerEntry = {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  hash: string | null;
  modified: string | null;
  source_type: string;
  is_nested_container: boolean;
  nested_type: string | null;
};

export type NestedContainerInfo = {
  container_type: string;
  entry_count: number;
  total_size: number;
  encrypted: boolean;
  temp_path: string;
  original_path: string;
};

// ============================================================================
// EWF Types (E01, L01, Ex01, Lx01)
// ============================================================================

export type EwfInfo = {
  format_version: string;
  segment_count: number;
  sector_count: number;
  bytes_per_sector: number;
  chunk_count: number;
  sectors_per_chunk: number;
  total_size: number;
  compression: string;
  case_number?: string;
  description?: string;
  examiner_name?: string;
  evidence_number?: string;
  notes?: string;
  acquiry_date?: string;
  system_date?: string;
  model?: string;
  serial_number?: string;
  stored_hashes?: import("./hash").StoredHash[];
  header_section_offset?: number;
  volume_section_offset?: number;
  hash_section_offset?: number;
  digest_section_offset?: number;
};

/** @deprecated Use EwfInfo instead - L01 uses the same EWF format */
export type L01Info = EwfInfo;

/** @deprecated Use EwfInfo instead */
export type E01Info = EwfInfo;

// ============================================================================
// Raw Image Types
// ============================================================================

export type RawInfo = {
  segment_count: number;
  total_size: number;
  segment_sizes: number[];
  segment_names: string[];
  first_segment: string;
  last_segment: string;
};

// ============================================================================
// Segment Types
// ============================================================================

export type SegmentFileInfo = {
  number: number;
  path: string;
  filename: string;
  size: number;
  exists: boolean;
  data_size: number;
  offset_start: number;
  offset_end: number;
};

export type SegmentSummary = {
  expected_count: number;
  found_count: number;
  missing_count: number;
  total_size: number;
  total_data_size: number;
  segments: SegmentFileInfo[];
  is_complete: boolean;
};

// ============================================================================
// Combined Container Info
// ============================================================================

export type Ad1Info = {
  segment: SegmentHeader;
  logical: LogicalHeader;
  item_count: number;
  tree?: TreeEntry[];
  segment_files?: string[];
  segment_sizes?: number[];
  total_size?: number;
  missing_segments?: string[];
  segment_summary?: SegmentSummary | null;
  volume?: Ad1VolumeInfo | null;
  companion_log?: Ad1CompanionLogInfo | null;
};

export type ContainerInfo = {
  container: string;
  ad1?: Ad1Info | null;
  e01?: EwfInfo | null;
  l01?: EwfInfo | null;
  raw?: RawInfo | null;
  archive?: ArchiveInfo | null;
  ufed?: import("./ufed").UfedInfo | null;
  note?: string | null;
  companion_log?: import("./companion").CompanionLogInfo | null;
};
