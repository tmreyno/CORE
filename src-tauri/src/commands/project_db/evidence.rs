// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for evidence files, hashes, and verifications.

use super::with_project_db;
use crate::project_db::{DbEvidenceFile, DbProjectHash, DbProjectVerification};

// =============================================================================
// Evidence File Commands
// =============================================================================

/// Insert or update an evidence file record.
#[tauri::command]
pub fn project_db_upsert_evidence_file(file: DbEvidenceFile) -> Result<(), String> {
    with_project_db(|db| db.upsert_evidence_file(&file))
}

/// Get all evidence files.
#[tauri::command]
pub fn project_db_get_evidence_files() -> Result<Vec<DbEvidenceFile>, String> {
    with_project_db(|db| db.get_evidence_files())
}

/// Get an evidence file by path.
#[tauri::command]
pub fn project_db_get_evidence_file_by_path(
    path: String,
) -> Result<Option<DbEvidenceFile>, String> {
    with_project_db(|db| db.get_evidence_file_by_path(&path))
}

// =============================================================================
// Hash Commands
// =============================================================================

/// Insert a hash record.
#[tauri::command]
pub fn project_db_insert_hash(hash: DbProjectHash) -> Result<(), String> {
    with_project_db(|db| db.insert_hash(&hash))
}

/// Get all hashes for an evidence file.
#[tauri::command]
pub fn project_db_get_hashes_for_file(file_id: String) -> Result<Vec<DbProjectHash>, String> {
    with_project_db(|db| db.get_hashes_for_file(&file_id))
}

/// Get the latest hash for a file/algorithm.
#[tauri::command]
pub fn project_db_get_latest_hash(
    file_id: String,
    algorithm: String,
) -> Result<Option<DbProjectHash>, String> {
    with_project_db(|db| db.get_latest_hash(&file_id, &algorithm))
}

/// Look up latest hash by file path and algorithm.
#[tauri::command]
pub fn project_db_lookup_hash_by_path(
    path: String,
    algorithm: String,
) -> Result<Option<(String, String)>, String> {
    with_project_db(|db| db.lookup_hash_by_path(&path, &algorithm))
}

// =============================================================================
// Verification Commands
// =============================================================================

/// Insert a verification record.
#[tauri::command]
pub fn project_db_insert_verification(v: DbProjectVerification) -> Result<(), String> {
    with_project_db(|db| db.insert_verification(&v))
}

/// Get verifications for a hash.
#[tauri::command]
pub fn project_db_get_verifications_for_hash(
    hash_id: String,
) -> Result<Vec<DbProjectVerification>, String> {
    with_project_db(|db| db.get_verifications_for_hash(&hash_id))
}
