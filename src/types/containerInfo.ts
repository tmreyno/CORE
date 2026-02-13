// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Container info types — detailed metadata for each container format
 */

import type {
  SegmentHeader,
  LogicalHeader,
  TreeEntry,
  SegmentSummary,
} from "./container";
import type { HashAlgorithmName } from "./hash";

// --- AD1 Info Types ---

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

// --- EWF Info Types ---

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

// --- Raw Info Types ---

export type RawInfo = {
  segment_count: number;
  total_size: number;
  segment_sizes: number[];
  segment_names: string[];
  first_segment: string;
  last_segment: string;
};

// --- Archive Info Types ---

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
  verified?: boolean;
  verified_against?: string;
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
