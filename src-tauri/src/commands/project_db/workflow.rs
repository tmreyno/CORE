// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for file classifications, extraction log, viewer history,
//! annotations, and evidence relationships.

use super::with_project_db;
use crate::project_db::{
    DbAnnotation, DbEvidenceRelationship, DbExtractionRecord,
    DbFileClassification, DbViewerHistoryEntry,
};

// =============================================================================
// File Classification Commands
// =============================================================================

/// Insert or update a file classification.
#[tauri::command]
pub fn project_db_upsert_classification(record: DbFileClassification) -> Result<(), String> {
    with_project_db(|db| db.upsert_classification(&record))
}

/// Get classifications for a specific file path.
#[tauri::command]
pub fn project_db_get_classifications_for_path(
    file_path: String,
) -> Result<Vec<DbFileClassification>, String> {
    with_project_db(|db| db.get_classifications_for_path(&file_path))
}

/// Get all classifications.
#[tauri::command]
pub fn project_db_get_all_classifications() -> Result<Vec<DbFileClassification>, String> {
    with_project_db(|db| db.get_all_classifications())
}

/// Delete a classification.
#[tauri::command]
pub fn project_db_delete_classification(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_classification(&id))
}

// =============================================================================
// Extraction Log Commands
// =============================================================================

/// Insert an extraction log entry.
#[tauri::command]
pub fn project_db_insert_extraction(record: DbExtractionRecord) -> Result<(), String> {
    with_project_db(|db| db.insert_extraction(&record))
}

/// Get extraction records for a container.
#[tauri::command]
pub fn project_db_get_extractions_for_container(
    container_path: String,
) -> Result<Vec<DbExtractionRecord>, String> {
    with_project_db(|db| db.get_extractions_for_container(&container_path))
}

/// Get all extraction records.
#[tauri::command]
pub fn project_db_get_all_extractions(
    limit: Option<i64>,
) -> Result<Vec<DbExtractionRecord>, String> {
    with_project_db(|db| db.get_all_extractions(limit))
}

// =============================================================================
// Viewer History Commands
// =============================================================================

/// Insert a viewer history entry.
#[tauri::command]
pub fn project_db_insert_viewer_history(entry: DbViewerHistoryEntry) -> Result<(), String> {
    with_project_db(|db| db.insert_viewer_history(&entry))
}

/// Update viewer history when a file is closed.
#[tauri::command]
pub fn project_db_update_viewer_history_close(
    id: String,
    closed_at: String,
    duration_seconds: Option<i64>,
) -> Result<(), String> {
    with_project_db(|db| db.update_viewer_history_close(&id, &closed_at, duration_seconds))
}

/// Get recent viewer history.
#[tauri::command]
pub fn project_db_get_viewer_history(
    limit: Option<i64>,
) -> Result<Vec<DbViewerHistoryEntry>, String> {
    with_project_db(|db| db.get_viewer_history(limit))
}

// =============================================================================
// Annotation Commands
// =============================================================================

/// Insert a new annotation.
#[tauri::command]
pub fn project_db_insert_annotation(ann: DbAnnotation) -> Result<(), String> {
    with_project_db(|db| db.insert_annotation(&ann))
}

/// Update an annotation (label, content, color).
#[tauri::command]
pub fn project_db_update_annotation(ann: DbAnnotation) -> Result<(), String> {
    with_project_db(|db| db.update_annotation(&ann))
}

/// Get annotations for a file.
#[tauri::command]
pub fn project_db_get_annotations_for_path(
    file_path: String,
) -> Result<Vec<DbAnnotation>, String> {
    with_project_db(|db| db.get_annotations_for_path(&file_path))
}

/// Get all annotations.
#[tauri::command]
pub fn project_db_get_all_annotations() -> Result<Vec<DbAnnotation>, String> {
    with_project_db(|db| db.get_all_annotations())
}

/// Delete an annotation.
#[tauri::command]
pub fn project_db_delete_annotation(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_annotation(&id))
}

// =============================================================================
// Evidence Relationship Commands
// =============================================================================

/// Insert an evidence relationship.
#[tauri::command]
pub fn project_db_insert_relationship(rel: DbEvidenceRelationship) -> Result<(), String> {
    with_project_db(|db| db.insert_relationship(&rel))
}

/// Get relationships involving a path (source or target).
#[tauri::command]
pub fn project_db_get_relationships_for_path(
    path: String,
) -> Result<Vec<DbEvidenceRelationship>, String> {
    with_project_db(|db| db.get_relationships_for_path(&path))
}

/// Get all evidence relationships.
#[tauri::command]
pub fn project_db_get_all_relationships() -> Result<Vec<DbEvidenceRelationship>, String> {
    with_project_db(|db| db.get_all_relationships())
}

/// Delete a relationship.
#[tauri::command]
pub fn project_db_delete_relationship(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_relationship(&id))
}
