// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Processed Database Types (matching Rust types)

export type ProcessedDbType = 
  | "MagnetAxiom"
  | "CellebritePA"
  | "XWays"
  | "Autopsy"
  | "EnCase"
  | "FTK"
  | "GenericSqlite"
  | "Unknown";

export type ArtifactCategory =
  | "WebHistory"
  | "Email"
  | "Chat"
  | "Media"
  | "Documents"
  | "FileSystem"
  | "Registry"
  | "Network"
  | "Timeline"
  | "Artifacts"
  | "Mobile"
  | "Cloud"
  | "Encryption"
  | "Malware"
  | "Other";

export type DatabaseContents =
  | "CaseInfo"
  | "Artifacts"
  | "FileSystem"
  | "Keywords"
  | "Hashes"
  | "Media"
  | "Timeline"
  | "Bookmarks"
  | "Reports"
  | "Config"
  | "Unknown";

export interface ArtifactInfo {
  name: string;
  category: ArtifactCategory;
  table_name?: string;
  count?: number;
  description?: string;
}

export interface DatabaseFile {
  path: string;
  name: string;
  size: number;
  contents: DatabaseContents;
}

/** Processed database information (matches Rust ProcessedDatabase) */
export interface ProcessedDatabase {
  db_type: ProcessedDbType;
  path: string;
  name?: string;
  case_name?: string;
  case_number?: string;
  examiner?: string;
  created_date?: string;
  version?: string;
  total_size?: number;
  artifact_count?: number;
  artifacts?: ArtifactInfo[];
  database_files?: DatabaseFile[];
  notes?: string;
}

/** Legacy alias for backward compatibility */
export type ProcessedDbInfo = ProcessedDatabase;

export interface ProcessedDbSummary {
  total_count: number;
  by_type: Record<string, number>;
  total_size: number;
}

// Icon mappings for processed database types
export const PROCESSED_DB_ICONS: Record<ProcessedDbType, string> = {
  MagnetAxiom: "🧲",
  CellebritePA: "📱",
  XWays: "🔬",
  Autopsy: "🔍",
  EnCase: "📦",
  FTK: "🗃️",
  GenericSqlite: "🗄️",
  Unknown: "❓",
};

// Display names for processed database types
export const PROCESSED_DB_NAMES: Record<ProcessedDbType, string> = {
  MagnetAxiom: "Magnet AXIOM",
  CellebritePA: "Cellebrite PA",
  XWays: "X-Ways",
  Autopsy: "Autopsy",
  EnCase: "EnCase",
  FTK: "FTK",
  GenericSqlite: "SQLite Database",
  Unknown: "Unknown",
};

// Display names for database contents
export const DB_CONTENTS_NAMES: Record<DatabaseContents, string> = {
  CaseInfo: "Case Information",
  Artifacts: "Artifacts",
  FileSystem: "File System",
  Keywords: "Keywords",
  Hashes: "Hashes",
  Media: "Media",
  Timeline: "Timeline",
  Bookmarks: "Bookmarks",
  Reports: "Reports",
  Config: "Configuration",
  Unknown: "Unknown",
};

// ============================================================================
// AXIOM-specific types
// ============================================================================

/** AXIOM evidence source */
export interface AxiomEvidenceSource {
  name: string;
  evidence_number?: string;
  source_type: string;
  search_types: string[];
  path?: string;
  hash?: string;
  size?: number;
  acquired?: string;
}

/** AXIOM search result entry */
export interface AxiomSearchResult {
  artifact_type: string;
  hit_count: number;
}

/** AXIOM keyword entry */
export interface AxiomKeyword {
  value: string;
  is_regex: boolean;
  is_case_sensitive: boolean;
  encoding_types: string[];
  from_file: boolean;
  file_name?: string;
}

/** AXIOM keyword file */
export interface AxiomKeywordFile {
  file_name: string;
  file_path: string;
  date_added?: string;
  record_count: number;
  enabled: boolean;
  is_case_sensitive: boolean;
}

/** AXIOM keyword search info */
export interface AxiomKeywordInfo {
  keywords_entered: number;
  regex_count: number;
  keywords: AxiomKeyword[];
  keyword_files: AxiomKeywordFile[];
  privileged_content_keywords: AxiomKeyword[];
  privileged_content_mode?: string;
}

/** AXIOM case information */
export interface AxiomCaseInfo {
  case_name: string;
  case_number?: string;
  case_type?: string;
  description?: string;
  examiner?: string;
  agency?: string;
  user?: string;
  host_name?: string;
  operating_system?: string;
  created?: string;
  modified?: string;
  axiom_version?: string;
  search_start?: string;
  search_end?: string;
  search_duration?: string;
  search_outcome?: string;
  output_folder?: string;
  evidence_sources: AxiomEvidenceSource[];
  search_results: AxiomSearchResult[];
  total_artifacts: number;
  case_path?: string;
  keyword_info?: AxiomKeywordInfo;
}

/** AXIOM artifact category summary */
export interface ArtifactCategorySummary {
  category: string;
  artifact_type: string;
  count: number;
}

/** AXIOM artifact */
export interface AxiomArtifact {
  id: number;
  artifact_type: string;
  name: string;
  source: string;
  timestamp?: string;
  data: Record<string, string>;
}

/** Table info for database exploration */
export interface TableInfo {
  name: string;
  count: number;
}
