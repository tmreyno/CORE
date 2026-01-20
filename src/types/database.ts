// =============================================================================
// CORE-FFX - Forensic File Explorer
// Database Types - SQLite persistence types
// =============================================================================

/**
 * Database Types
 * 
 * Types for SQLite database persistence of sessions, files, and hashes.
 */

// ============================================================================
// Session Types
// ============================================================================

/** A session represents an open directory/workspace */
export type DbSession = {
  id: string;
  name: string;
  root_path: string;
  created_at: string;
  last_opened_at: string;
};

// ============================================================================
// File Record Types
// ============================================================================

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

// ============================================================================
// Hash Record Types
// ============================================================================

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

// ============================================================================
// Tab State Types
// ============================================================================

/** An open tab record for UI state */
export type DbOpenTabRecord = {
  id: string;
  session_id: string;
  file_path: string;
  tab_order: number;
  is_active: boolean;
};
