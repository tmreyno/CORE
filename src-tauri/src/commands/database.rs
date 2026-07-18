// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! SQLite persistence layer commands for sessions, files, hashes, and settings.

use crate::database;

fn with_db<T>(
    action: impl FnOnce(&database::Database) -> rusqlite::Result<T>,
) -> Result<T, String> {
    let db = database::get_db()?;
    action(db).map_err(|e| e.to_string())
}

/// Get or create a session for a directory path
#[tauri::command]
pub fn db_get_or_create_session(root_path: String) -> Result<database::Session, String> {
    with_db(|db| db.get_or_create_session(&root_path))
}

/// Get recent sessions
#[tauri::command]
pub fn db_get_recent_sessions(limit: i32) -> Result<Vec<database::Session>, String> {
    with_db(|db| db.get_recent_sessions(limit))
}

/// Get the last opened session
#[tauri::command]
pub fn db_get_last_session() -> Result<Option<database::Session>, String> {
    with_db(|db| db.get_last_session())
}

/// Save or update a file record
#[tauri::command]
pub fn db_upsert_file(file: database::FileRecord) -> Result<(), String> {
    with_db(|db| db.upsert_file(&file))
}

/// Get all files for a session
#[tauri::command]
pub fn db_get_files_for_session(session_id: String) -> Result<Vec<database::FileRecord>, String> {
    with_db(|db| db.get_files_for_session(&session_id))
}

/// Get a file by path
#[tauri::command]
pub fn db_get_file_by_path(
    session_id: String,
    path: String,
) -> Result<Option<database::FileRecord>, String> {
    with_db(|db| db.get_file_by_path(&session_id, &path))
}

/// Insert a hash record
#[tauri::command]
pub fn db_insert_hash(hash: database::HashRecord) -> Result<(), String> {
    with_db(|db| db.insert_hash(&hash))
}

/// Get all hashes for a file
#[tauri::command]
pub fn db_get_hashes_for_file(file_id: String) -> Result<Vec<database::HashRecord>, String> {
    with_db(|db| db.get_hashes_for_file(&file_id))
}

/// Get the latest hash for a file/algorithm/segment combo
#[tauri::command]
pub fn db_get_latest_hash(
    file_id: String,
    algorithm: String,
    segment_index: Option<i32>,
) -> Result<Option<database::HashRecord>, String> {
    with_db(|db| db.get_latest_hash(&file_id, &algorithm, segment_index))
}

/// Insert a verification record
#[tauri::command]
pub fn db_insert_verification(verification: database::VerificationRecord) -> Result<(), String> {
    with_db(|db| db.insert_verification(&verification))
}

/// Get verifications for a file
#[tauri::command]
pub fn db_get_verifications_for_file(
    file_id: String,
) -> Result<Vec<database::VerificationRecord>, String> {
    with_db(|db| db.get_verifications_for_file(&file_id))
}

/// Save open tabs for a session
#[tauri::command]
pub fn db_save_open_tabs(
    session_id: String,
    tabs: Vec<database::OpenTabRecord>,
) -> Result<(), String> {
    with_db(|db| db.save_open_tabs(&session_id, &tabs))
}

/// Get open tabs for a session
#[tauri::command]
pub fn db_get_open_tabs(session_id: String) -> Result<Vec<database::OpenTabRecord>, String> {
    with_db(|db| db.get_open_tabs(&session_id))
}

/// Set a setting value
#[tauri::command]
pub fn db_set_setting(key: String, value: String) -> Result<(), String> {
    with_db(|db| db.set_setting(&key, &value))
}

/// Get a setting value
#[tauri::command]
pub fn db_get_setting(key: String) -> Result<Option<String>, String> {
    with_db(|db| db.get_setting(&key))
}
