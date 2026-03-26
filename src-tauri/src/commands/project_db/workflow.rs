// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for annotations.

use super::with_project_db;
use crate::project_db::DbAnnotation;

// =============================================================================
// Annotation Commands
// =============================================================================

/// Insert a new annotation.
#[tauri::command]
pub fn project_db_insert_annotation(
    window: tauri::Window,
    ann: DbAnnotation,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.insert_annotation(&ann))
}

/// Update an annotation (label, content, color).
#[tauri::command]
pub fn project_db_update_annotation(
    window: tauri::Window,
    ann: DbAnnotation,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.update_annotation(&ann))
}

/// Get annotations for a file.
#[tauri::command]
pub fn project_db_get_annotations_for_path(
    window: tauri::Window,
    file_path: String,
) -> Result<Vec<DbAnnotation>, String> {
    with_project_db(window.label(), |db| db.get_annotations_for_path(&file_path))
}

/// Get all annotations.
#[tauri::command]
pub fn project_db_get_all_annotations(window: tauri::Window) -> Result<Vec<DbAnnotation>, String> {
    with_project_db(window.label(), |db| db.get_all_annotations())
}

/// Delete an annotation.
#[tauri::command]
pub fn project_db_delete_annotation(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_annotation(&id))
}
