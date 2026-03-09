// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for FTS search, database utilities, and form submissions.

use super::with_project_db;
use crate::project_db::{DbFormSubmission, FtsSearchResult};

// =============================================================================
// Full-Text Search Commands
// =============================================================================

/// Rebuild FTS5 indexes from source tables.
#[tauri::command]
pub fn project_db_rebuild_fts(window: tauri::Window) -> Result<(), String> {
    with_project_db(window.label(), |db| db.rebuild_fts_indexes())
}

/// Full-text search across notes, bookmarks, and activity log.
#[tauri::command]
pub fn project_db_fts_search(
    window: tauri::Window,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<FtsSearchResult>, String> {
    with_project_db(window.label(), |db| db.fts_search(&query, limit))
}

// =============================================================================
// Database Utility Commands
// =============================================================================

/// Run SQLite integrity check on the project database.
#[tauri::command]
pub fn project_db_integrity_check(window: tauri::Window) -> Result<Vec<String>, String> {
    with_project_db(window.label(), |db| db.integrity_check())
}

/// Force WAL checkpoint (flush write-ahead log to main DB file).
#[tauri::command]
pub fn project_db_wal_checkpoint(window: tauri::Window) -> Result<(i64, i64), String> {
    with_project_db(window.label(), |db| db.wal_checkpoint())
}

/// Create a backup copy of the project database.
#[tauri::command]
pub fn project_db_backup(window: tauri::Window, dest_path: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.backup_to(&dest_path))
}

/// Vacuum the database to reclaim space.
#[tauri::command]
pub fn project_db_vacuum(window: tauri::Window) -> Result<(), String> {
    with_project_db(window.label(), |db| db.vacuum())
}

// =============================================================================
// Form Submission Commands (Generic JSON-driven forms)
// =============================================================================

/// Upsert (insert or update) a form submission.
#[tauri::command]
pub fn project_db_upsert_form_submission(
    window: tauri::Window,
    submission: DbFormSubmission,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_form_submission(&submission))
}

/// Get a form submission by ID.
#[tauri::command]
pub fn project_db_get_form_submission(
    window: tauri::Window,
    id: String,
) -> Result<Option<DbFormSubmission>, String> {
    with_project_db(window.label(), |db| db.get_form_submission(&id))
}

/// List form submissions with optional filters.
#[tauri::command]
pub fn project_db_list_form_submissions(
    window: tauri::Window,
    template_id: Option<String>,
    case_number: Option<String>,
    status: Option<String>,
) -> Result<Vec<DbFormSubmission>, String> {
    with_project_db(window.label(), |db| {
        db.list_form_submissions(
            template_id.as_deref(),
            case_number.as_deref(),
            status.as_deref(),
        )
    })
}

/// Delete a form submission (only draft status).
#[tauri::command]
pub fn project_db_delete_form_submission(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_form_submission(&id))
}
