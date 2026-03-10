// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for evidence collections and collected items.

use super::with_project_db;
use crate::project_db::{DbCollectedItem, DbEvidenceCollection, DbEvidenceDataAlternative};

// =============================================================================
// Evidence Collection Commands
// =============================================================================

/// Insert or update an evidence collection record.
#[tauri::command]
pub fn project_db_upsert_evidence_collection(
    window: tauri::Window,
    record: DbEvidenceCollection,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_evidence_collection(&record))
}

/// Get evidence collections, optionally filtered by case number.
#[tauri::command]
pub fn project_db_get_evidence_collections(
    window: tauri::Window,
    case_number: Option<String>,
) -> Result<Vec<DbEvidenceCollection>, String> {
    with_project_db(window.label(), |db| {
        db.get_evidence_collections(case_number.as_deref())
    })
}

/// Delete an evidence collection.
#[tauri::command]
pub fn project_db_delete_evidence_collection(
    window: tauri::Window,
    id: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_evidence_collection(&id))
}

/// Get a single evidence collection by ID (with item count).
#[tauri::command]
pub fn project_db_get_evidence_collection_by_id(
    window: tauri::Window,
    id: String,
) -> Result<DbEvidenceCollection, String> {
    with_project_db(window.label(), |db| db.get_evidence_collection_by_id(&id))
}

/// Update evidence collection status (draft → complete → locked).
#[tauri::command]
pub fn project_db_update_evidence_collection_status(
    window: tauri::Window,
    id: String,
    new_status: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.update_evidence_collection_status(&id, &new_status)
    })
}

// =============================================================================
// Collected Item Commands
// =============================================================================

/// Insert or update a collected item.
#[tauri::command]
pub fn project_db_upsert_collected_item(
    window: tauri::Window,
    record: DbCollectedItem,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_collected_item(&record))
}

/// Get collected items for a specific collection.
#[tauri::command]
pub fn project_db_get_collected_items(
    window: tauri::Window,
    collection_id: String,
) -> Result<Vec<DbCollectedItem>, String> {
    with_project_db(window.label(), |db| db.get_collected_items(&collection_id))
}

/// Get all collected items.
#[tauri::command]
pub fn project_db_get_all_collected_items(
    window: tauri::Window,
) -> Result<Vec<DbCollectedItem>, String> {
    with_project_db(window.label(), |db| db.get_all_collected_items())
}

/// Delete a collected item.
#[tauri::command]
pub fn project_db_delete_collected_item(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_collected_item(&id))
}

// =============================================================================
// Evidence Data Alternative Commands
// =============================================================================

/// Insert or update an evidence data alternative record.
#[tauri::command]
pub fn project_db_upsert_evidence_data_alternative(
    window: tauri::Window,
    record: DbEvidenceDataAlternative,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.upsert_evidence_data_alternative(&record)
    })
}

/// Get all evidence data alternatives for a collected item.
#[tauri::command]
pub fn project_db_get_evidence_data_alternatives(
    window: tauri::Window,
    collected_item_id: String,
) -> Result<Vec<DbEvidenceDataAlternative>, String> {
    with_project_db(window.label(), |db| {
        db.get_evidence_data_alternatives(&collected_item_id)
    })
}

/// Get all evidence data alternatives for a specific evidence file.
#[tauri::command]
pub fn project_db_get_evidence_data_alternatives_by_file(
    window: tauri::Window,
    evidence_file_id: String,
) -> Result<Vec<DbEvidenceDataAlternative>, String> {
    with_project_db(window.label(), |db| {
        db.get_evidence_data_alternatives_by_file(&evidence_file_id)
    })
}

/// Delete a single evidence data alternative record.
#[tauri::command]
pub fn project_db_delete_evidence_data_alternative(
    window: tauri::Window,
    id: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.delete_evidence_data_alternative(&id)
    })
}

/// Delete all evidence data alternatives for a collected item.
#[tauri::command]
pub fn project_db_delete_evidence_data_alternatives_for_item(
    window: tauri::Window,
    collected_item_id: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.delete_evidence_data_alternatives_for_item(&collected_item_id)
    })
}
