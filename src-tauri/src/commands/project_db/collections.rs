// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for evidence collections and collected items.

use super::with_project_db;
use crate::project_db::{DbCollectedItem, DbEvidenceCollection};

// =============================================================================
// Evidence Collection Commands
// =============================================================================

/// Insert or update an evidence collection record.
#[tauri::command]
pub fn project_db_upsert_evidence_collection(window: tauri::Window, record: DbEvidenceCollection) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_evidence_collection(&record))
}

/// Get evidence collections, optionally filtered by case number.
#[tauri::command]
pub fn project_db_get_evidence_collections(window: tauri::Window, 
    case_number: Option<String>,
) -> Result<Vec<DbEvidenceCollection>, String> {
    with_project_db(window.label(), |db| db.get_evidence_collections(case_number.as_deref()))
}

/// Delete an evidence collection.
#[tauri::command]
pub fn project_db_delete_evidence_collection(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_evidence_collection(&id))
}

/// Get a single evidence collection by ID (with item count).
#[tauri::command]
pub fn project_db_get_evidence_collection_by_id(window: tauri::Window, 
    id: String,
) -> Result<DbEvidenceCollection, String> {
    with_project_db(window.label(), |db| db.get_evidence_collection_by_id(&id))
}

/// Update evidence collection status (draft → complete → locked).
#[tauri::command]
pub fn project_db_update_evidence_collection_status(window: tauri::Window, 
    id: String,
    new_status: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.update_evidence_collection_status(&id, &new_status))
}

// =============================================================================
// Collected Item Commands
// =============================================================================

/// Insert or update a collected item.
#[tauri::command]
pub fn project_db_upsert_collected_item(window: tauri::Window, record: DbCollectedItem) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_collected_item(&record))
}

/// Get collected items for a specific collection.
#[tauri::command]
pub fn project_db_get_collected_items(window: tauri::Window, 
    collection_id: String,
) -> Result<Vec<DbCollectedItem>, String> {
    with_project_db(window.label(), |db| db.get_collected_items(&collection_id))
}

/// Get all collected items.
#[tauri::command]
pub fn project_db_get_all_collected_items(window: tauri::Window) -> Result<Vec<DbCollectedItem>, String> {
    with_project_db(window.label(), |db| db.get_all_collected_items())
}

/// Delete a collected item.
#[tauri::command]
pub fn project_db_delete_collected_item(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_collected_item(&id))
}
