// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared TypeScript types for forensic container analysis
 *
 * NOTE ON TYPE NAMING:
 * - `TreeEntry` (this file) - AD1-specific tree entry type matching Rust's `ad1::TreeEntry`
 *   Used for direct Tauri command responses from AD1 operations.
 * - `TreeEntryInfo` (types/lifecycle.ts) - Generic tree entry type for the unified
 *   container trait API. Used in the trait-based abstraction layer.
 *
 * Both exist because AD1 has format-specific fields (item_type, data_addr, etc.)
 * that don't apply to all container formats.
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

// --- Container Info Types ---

export type Ad1VolumeInfo = {
  volume_label?: string | null;
  filesystem?: string | null;
  os_info?: string | null;
  block_size?: number | null;
  volume_serial?: string | null;
};

export type Ad1CompanionLogInfo = {
  /** Case number or identifier */
  case_number?: string | null;
  /** Evidence number or item number */
  evidence_number?: string | null;
  /** Examiner or analyst name */
  examiner?: string | null;
  /** Free-form notes or description */
  notes?: string | null;
  /** MD5 hash of the container or source */
  md5_hash?: string | null;
  /** SHA1 hash of the container or source */
  sha1_hash?: string | null;
  /** SHA256 hash (if available) */
  sha256_hash?: string | null;
  /** Date/time of acquisition */
  acquisition_date?: string | null;
  /** Source device or media description */
  source_device?: string | null;
  /** Source path or location */
  source_path?: string | null;
  /** Acquisition tool name and version */
  acquisition_tool?: string | null;
  /** Total items or files processed */
  total_items?: number | null;
  /** Total size of acquired data */
  total_size?: number | null;
  /** Acquisition method (logical, physical, etc.) */
  acquisition_method?: string | null;
  /** Organization or agency */
  organization?: string | null;
};

export type Ad1Info = {
  segment: SegmentHeader;
  logical: LogicalHeader;
  item_count: number;
  tree?: TreeEntry[];
  segment_files?: string[];
  /** Size of each segment file in bytes */
  segment_sizes?: number[];
  /** Total size of all segment files combined */
  total_size?: number;
  /** Missing segment files (incomplete container) */
  missing_segments?: string[];
  /** Detailed segment information with offset ranges */
  segment_summary?: SegmentSummary | null;
  volume?: Ad1VolumeInfo | null;
  companion_log?: Ad1CompanionLogInfo | null;
};

/** EWF container info (E01/L01/Ex01/Lx01 formats) */
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
  stored_hashes?: StoredHash[];
  // Section offsets for hex navigation
  header_section_offset?: number;
  volume_section_offset?: number;
  hash_section_offset?: number;
  digest_section_offset?: number;
};

/** @deprecated Use EwfInfo instead - L01 uses the same EWF format */
export type L01Info = EwfInfo;

/** @deprecated Use EwfInfo instead */
export type E01Info = EwfInfo;

export type RawInfo = {
  segment_count: number;
  total_size: number;
  segment_sizes: number[];
  segment_names: string[];
  first_segment: string;
  last_segment: string;
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
  // ZIP-specific
  central_dir_offset?: number | null;
  central_dir_size?: number | null;
  // 7z-specific
  next_header_offset?: number | null;
  next_header_size?: number | null;
  /** 7z archive version (major.minor) */
  version?: string | null;
  /** Whether Start Header CRC is valid */
  start_header_crc_valid?: boolean | null;
  /** Next Header CRC value */
  next_header_crc?: number | null;
  /** Whether Cellebrite UFED files were detected inside the archive */
  cellebrite_detected?: boolean;
  /** List of Cellebrite files found (UFD, UFDR, UFDX) */
  cellebrite_files?: string[];
};

// --- UFED (Cellebrite) Types ---

export type UfedAssociatedFile = {
  filename: string;
  file_type: string;
  size: number;
  stored_hash?: string | null;
};

export type UfedCaseInfo = {
  case_identifier?: string | null;
  crime_type?: string | null;
  department?: string | null;
  device_name?: string | null;
  examiner_name?: string | null;
  location?: string | null;
};

export type UfedDeviceInfo = {
  vendor?: string | null;
  model?: string | null;
  full_name?: string | null;
  imei?: string | null;
  imei2?: string | null;
  iccid?: string | null;
  os_version?: string | null;
  serial_number?: string | null;
};

export type UfedExtractionInfo = {
  acquisition_tool?: string | null;
  tool_version?: string | null;
  unit_id?: string | null;
  extraction_type?: string | null;
  connection_type?: string | null;
  start_time?: string | null;
  end_time?: string | null;
  guid?: string | null;
  machine_name?: string | null;
};

export type UfedStoredHash = {
  filename: string;
  algorithm: string;
  hash: string;
  /** When the hash was recorded (from extraction timestamp) */
  timestamp?: string | null;
};

export type UfedCollectionInfo = {
  evidence_id?: string | null;
  vendor?: string | null;
  model?: string | null;
  device_guid?: string | null;
  extractions: string[];
  ufdx_path: string;
};

export type UfedInfo = {
  format: string;
  size: number;
  parent_folder?: string | null;
  associated_files: UfedAssociatedFile[];
  is_extraction_set: boolean;
  device_hint?: string | null;
  case_info?: UfedCaseInfo | null;
  device_info?: UfedDeviceInfo | null;
  extraction_info?: UfedExtractionInfo | null;
  stored_hashes?: UfedStoredHash[] | null;
  evidence_number?: string | null;
  collection_info?: UfedCollectionInfo | null;
};

// --- Hash Types ---

export type StoredHash = {
  algorithm: string;
  hash: string;
  verified?: boolean | null;
  timestamp?: string | null;
  source?: string | null;
  /** Filename this hash belongs to (for UFED which has per-file hashes) */
  filename?: string | null;
  /** Byte offset in file where raw hash bytes are located */
  offset?: number | null;
  /** Size in bytes of the hash (MD5=16, SHA1=20, SHA256=32) */
  size?: number | null;
};

export type SegmentHash = {
  segment_name: string;
  segment_number: number;
  algorithm: string;
  hash: string;
  offset_from?: number | null;
  offset_to?: number | null;
  size?: number | null;
  verified?: boolean | null;
};

export type HashHistoryEntry = {
  algorithm: string;
  hash: string;
  timestamp: Date;
  source: "computed" | "stored" | "verified";
  verified?: boolean; // Changed from boolean | null to match canonical type
  verified_against?: string; // Changed from string | null to match canonical type
};

// --- Companion Log Types ---

export type CompanionLogInfo = {
  log_path: string;
  created_by?: string;
  case_number?: string;
  evidence_number?: string;
  unique_description?: string;
  examiner?: string;
  notes?: string;
  acquisition_started?: string;
  acquisition_finished?: string;
  verification_started?: string;
  verification_finished?: string;
  stored_hashes: StoredHash[];
  segment_list: string[];
  segment_hashes: SegmentHash[];
};

// --- Combined Container Info ---

export type ContainerInfo = {
  container: string;
  ad1?: Ad1Info | null;
  /** EWF physical image (E01/Ex01) */
  e01?: EwfInfo | null;
  /** EWF logical evidence (L01/Lx01) */
  l01?: EwfInfo | null;
  raw?: RawInfo | null;
  archive?: ArchiveInfo | null;
  ufed?: UfedInfo | null;
  note?: string | null;
  companion_log?: CompanionLogInfo | null;
};

// --- Hash Algorithm Types ---

// Re-export centralized hash types for convenience
import type { HashAlgorithmName } from "./types/hash";
export type HashAlgorithm = HashAlgorithmName; // Alias for backward compatibility

export type HashAlgorithmInfo = { 
  value: HashAlgorithmName; 
  label: string; 
  speed: "fast" | "medium" | "slow";
  forensic: boolean;  // Court-accepted for forensics
  cryptographic: boolean;
};

export const HASH_ALGORITHMS: HashAlgorithmInfo[] = [
  { value: "SHA-1", label: "SHA-1", speed: "medium", forensic: true, cryptographic: true },
  { value: "SHA-256", label: "SHA-256", speed: "medium", forensic: true, cryptographic: true },
  { value: "MD5", label: "MD5", speed: "medium", forensic: true, cryptographic: false },
  { value: "BLAKE3", label: "BLAKE3 ⚡", speed: "fast", forensic: false, cryptographic: true },
  { value: "SHA-512", label: "SHA-512", speed: "slow", forensic: true, cryptographic: true },
  { value: "BLAKE2b", label: "BLAKE2b", speed: "fast", forensic: false, cryptographic: true },
  { value: "XXH3", label: "XXH3 ⚡⚡", speed: "fast", forensic: false, cryptographic: false },
  { value: "XXH64", label: "XXH64 ⚡⚡", speed: "fast", forensic: false, cryptographic: false },
  { value: "CRC32", label: "CRC32", speed: "fast", forensic: false, cryptographic: false },
];

// --- Database Persistence Types ---

/** A session represents an open directory/workspace */
export type DbSession = {
  id: string;
  name: string;
  root_path: string;
  created_at: string;
  last_opened_at: string;
};

/** A file record in the database */
export type DbFileRecord = {
  id: string;
  session_id: string;
  path: string;
  filename: string;
  container_type: string;
  total_size: number;
  segment_count: number;
  discovered_at: string;
};

/** A hash record - immutable audit trail */
export type DbHashRecord = {
  id: string;
  file_id: string;
  algorithm: string;
  hash_value: string;
  computed_at: string;
  segment_index?: number | null;
  segment_name?: string | null;
  source: "computed" | "stored" | "imported";
};

/** A verification record */
export type DbVerificationRecord = {
  id: string;
  hash_id: string;
  verified_at: string;
  result: "match" | "mismatch";
  expected_hash: string;
  actual_hash: string;
};

/** An open tab record for UI state */
export type DbOpenTabRecord = {
  id: string;
  session_id: string;
  file_path: string;
  tab_order: number;
  is_active: boolean;
};

// --- Project File Types ---
// Project types are now defined in types/project.ts for comprehensive state management
// Re-export for backward compatibility
export * from './types/project';

// --- Viewer Types (from viewer.rs) ---

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

// =============================================================================
// VFS (Virtual Filesystem) Types - For mounting disk images
// =============================================================================

/** Entry in a mounted virtual filesystem */
export type VfsEntry = {
  /** Entry name */
  name: string;
  /** Full path within the VFS */
  path: string;
  /** Is this a directory? */
  isDir: boolean;
  /** File size (0 for directories) */
  size: number;
  /** File type hint */
  fileType?: string;
};

/** Information about a partition in a mounted disk image */
export type VfsPartitionInfo = {
  /** Partition number (1-based) */
  number: number;
  /** Mount name (e.g., "Partition1_NTFS") */
  mountName: string;
  /** Filesystem type (NTFS, FAT32, etc.) */
  fsType: string;
  /** Partition size in bytes */
  size: number;
  /** Start offset in the disk image */
  startOffset: number;
};

/** Information about a mounted disk image */
export type VfsMountInfo = {
  /** Container path */
  containerPath: string;
  /** Container type (e01, raw, etc.) */
  containerType: string;
  /** Total disk size */
  diskSize: number;
  /** Detected partitions */
  partitions: VfsPartitionInfo[];
  /** Mount mode (physical or filesystem) */
  mode: string;
};

// --- Case Document Types ---

/** Types of case documents */
export type CaseDocumentType =
  | "ChainOfCustody"
  | "EvidenceIntake"
  | "CaseNotes"
  | "EvidenceReceipt"
  | "LabRequest"
  | "ExternalReport"
  | "Other";

/** A discovered case document (COC form, intake form, etc.) */
export type CaseDocument = {
  /** Full path to the document */
  path: string;
  /** Filename */
  filename: string;
  /** Document type */
  document_type: CaseDocumentType;
  /** File size in bytes */
  size: number;
  /** File format (PDF, DOCX, TXT, etc.) */
  format: string;
  /** Case number extracted from filename (if found) */
  case_number?: string | null;
  /** Evidence ID extracted from filename (if found) */
  evidence_id?: string | null;
  /** Last modified timestamp (ISO 8601) */
  modified?: string | null;
};

// Re-export processed database types
export * from './types/processed';
