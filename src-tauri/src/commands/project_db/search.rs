// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for reports, saved searches, recent searches, case documents,
//! and UI state.

use super::with_project_db;
use crate::project_db::{DbCaseDocument, DbRecentSearch, DbReportRecord, DbSavedSearch};

// =============================================================================
// Report Commands
// =============================================================================

/// Insert a report record.
#[tauri::command]
pub fn project_db_insert_report(
    window: tauri::Window,
    report: DbReportRecord,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.insert_report(&report))
}

/// Get all reports.
#[tauri::command]
pub fn project_db_get_reports(window: tauri::Window) -> Result<Vec<DbReportRecord>, String> {
    with_project_db(window.label(), |db| db.get_reports())
}

// =============================================================================
// Search Commands
// =============================================================================

/// Insert or update a saved search.
#[tauri::command]
pub fn project_db_upsert_saved_search(
    window: tauri::Window,
    search: DbSavedSearch,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_saved_search(&search))
}

/// Get all saved searches.
#[tauri::command]
pub fn project_db_get_saved_searches(window: tauri::Window) -> Result<Vec<DbSavedSearch>, String> {
    with_project_db(window.label(), |db| db.get_saved_searches())
}

/// Insert or update a recent search.
#[tauri::command]
pub fn project_db_insert_recent_search(
    window: tauri::Window,
    search: DbRecentSearch,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.insert_recent_search(&search))
}

// =============================================================================
// Case Document Commands
// =============================================================================

/// Insert or update a case document.
#[tauri::command]
pub fn project_db_upsert_case_document(
    window: tauri::Window,
    doc: DbCaseDocument,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_case_document(&doc))
}

/// Get all case documents.
#[tauri::command]
pub fn project_db_get_case_documents(window: tauri::Window) -> Result<Vec<DbCaseDocument>, String> {
    with_project_db(window.label(), |db| db.get_case_documents())
}

// =============================================================================
// UI State Commands
// =============================================================================

/// Set a UI state value.
#[tauri::command]
pub fn project_db_set_ui_state(
    window: tauri::Window,
    key: String,
    value: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.set_ui_state(&key, &value))
}

/// Get a UI state value.
#[tauri::command]
pub fn project_db_get_ui_state(
    window: tauri::Window,
    key: String,
) -> Result<Option<String>, String> {
    with_project_db(window.label(), |db| db.get_ui_state(&key))
}
