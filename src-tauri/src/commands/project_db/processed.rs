// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for processed databases, integrity, metrics, AXIOM case info,
//! evidence sources, search results, and artifact categories.

use super::with_project_db;
use crate::project_db::{
    DbArtifactCategory, DbAxiomCaseInfo, DbAxiomEvidenceSource, DbAxiomSearchResult,
    DbProcessedDatabase, DbProcessedDbIntegrity, DbProcessedDbMetrics,
};

// =============================================================================
// Processed Database Commands
// =============================================================================

/// Insert or update a processed database record.
#[tauri::command]
pub fn project_db_upsert_processed_database(window: tauri::Window, db: DbProcessedDatabase) -> Result<(), String> {
    with_project_db(window.label(), |pdb| pdb.upsert_processed_database(&db))
}

/// Get all processed databases.
#[tauri::command]
pub fn project_db_get_processed_databases(window: tauri::Window) -> Result<Vec<DbProcessedDatabase>, String> {
    with_project_db(window.label(), |db| db.get_processed_databases())
}

/// Get a processed database by path.
#[tauri::command]
pub fn project_db_get_processed_database_by_path(window: tauri::Window, 
    path: String,
) -> Result<Option<DbProcessedDatabase>, String> {
    with_project_db(window.label(), |db| db.get_processed_database_by_path(&path))
}

/// Delete a processed database and all related records.
#[tauri::command]
pub fn project_db_delete_processed_database(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_processed_database(&id))
}

// =============================================================================
// Processed DB Integrity Commands
// =============================================================================

/// Insert or update a processed database integrity record.
#[tauri::command]
pub fn project_db_upsert_processed_db_integrity(window: tauri::Window, 
    integrity: DbProcessedDbIntegrity,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_processed_db_integrity(&integrity))
}

/// Get integrity records for a processed database.
#[tauri::command]
pub fn project_db_get_processed_db_integrity(window: tauri::Window, 
    processed_db_id: String,
) -> Result<Vec<DbProcessedDbIntegrity>, String> {
    with_project_db(window.label(), |db| db.get_processed_db_integrity(&processed_db_id))
}

// =============================================================================
// Processed DB Metrics Commands
// =============================================================================

/// Insert or update metrics for a processed database.
#[tauri::command]
pub fn project_db_upsert_processed_db_metrics(window: tauri::Window, metrics: DbProcessedDbMetrics) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_processed_db_metrics(&metrics))
}

/// Get metrics for a processed database.
#[tauri::command]
pub fn project_db_get_processed_db_metrics(window: tauri::Window, 
    processed_db_id: String,
) -> Result<Option<DbProcessedDbMetrics>, String> {
    with_project_db(window.label(), |db| db.get_processed_db_metrics(&processed_db_id))
}

// =============================================================================
// AXIOM Case Info Commands
// =============================================================================

/// Insert or update AXIOM case information.
#[tauri::command]
pub fn project_db_upsert_axiom_case_info(window: tauri::Window, info: DbAxiomCaseInfo) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_axiom_case_info(&info))
}

/// Get AXIOM case info for a processed database.
#[tauri::command]
pub fn project_db_get_axiom_case_info(window: tauri::Window, 
    processed_db_id: String,
) -> Result<Option<DbAxiomCaseInfo>, String> {
    with_project_db(window.label(), |db| db.get_axiom_case_info(&processed_db_id))
}

/// Get all AXIOM case info records.
#[tauri::command]
pub fn project_db_get_all_axiom_case_info(window: tauri::Window) -> Result<Vec<DbAxiomCaseInfo>, String> {
    with_project_db(window.label(), |db| db.get_all_axiom_case_info())
}

// =============================================================================
// AXIOM Evidence Source Commands
// =============================================================================

/// Insert an AXIOM evidence source.
#[tauri::command]
pub fn project_db_insert_axiom_evidence_source(window: tauri::Window, 
    source: DbAxiomEvidenceSource,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.insert_axiom_evidence_source(&source))
}

/// Get evidence sources for an AXIOM case.
#[tauri::command]
pub fn project_db_get_axiom_evidence_sources(window: tauri::Window, 
    axiom_case_id: String,
) -> Result<Vec<DbAxiomEvidenceSource>, String> {
    with_project_db(window.label(), |db| db.get_axiom_evidence_sources(&axiom_case_id))
}

// =============================================================================
// AXIOM Search Result Commands
// =============================================================================

/// Insert an AXIOM search result.
#[tauri::command]
pub fn project_db_insert_axiom_search_result(window: tauri::Window, result: DbAxiomSearchResult) -> Result<(), String> {
    with_project_db(window.label(), |db| db.insert_axiom_search_result(&result))
}

/// Get search results for an AXIOM case.
#[tauri::command]
pub fn project_db_get_axiom_search_results(window: tauri::Window, 
    axiom_case_id: String,
) -> Result<Vec<DbAxiomSearchResult>, String> {
    with_project_db(window.label(), |db| db.get_axiom_search_results(&axiom_case_id))
}

// =============================================================================
// Artifact Category Commands
// =============================================================================

/// Insert or replace artifact categories for a processed database.
#[tauri::command]
pub fn project_db_upsert_artifact_categories(window: tauri::Window, 
    categories: Vec<DbArtifactCategory>,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_artifact_categories(&categories))
}

/// Get artifact categories for a processed database.
#[tauri::command]
pub fn project_db_get_artifact_categories(window: tauri::Window, 
    processed_db_id: String,
) -> Result<Vec<DbArtifactCategory>, String> {
    with_project_db(window.label(), |db| db.get_artifact_categories(&processed_db_id))
}
