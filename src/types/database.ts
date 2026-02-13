// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Database persistence types for SQLite storage
 */

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
