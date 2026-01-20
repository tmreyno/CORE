// =============================================================================
// CORE-FFX - Forensic File Explorer
// UFED Types - Cellebrite UFED container types
// =============================================================================

/**
 * UFED (Cellebrite) Types
 * 
 * Types for UFED containers and mobile device extractions.
 */

// ============================================================================
// UFED Tree Entry
// ============================================================================

export type UfedTreeEntry = {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  entry_type: string;
  hash?: string | null;
  modified?: string | null;
};

// ============================================================================
// UFED Metadata Types
// ============================================================================

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

// ============================================================================
// UFED Container Info
// ============================================================================

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
