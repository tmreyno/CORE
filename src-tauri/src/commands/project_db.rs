// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for per-project SQLite database (.ffxdb) operations.
//!
//! These commands operate on the currently-open project database.
//! The database is opened/created when a project is loaded, and closed
//! when the project is closed.

use crate::project_db::{
    ActivityQuery, DbActivityEntry, DbAnnotation, DbArtifactCategory,
    DbAxiomCaseInfo, DbAxiomEvidenceSource, DbAxiomSearchResult, DbBookmark,
    DbCaseDocument, DbCustodyRecord, DbEvidenceFile, DbEvidenceRelationship,
    DbExportRecord, DbExtractionRecord, DbFileClassification, DbNote,
    DbProcessedDatabase, DbProcessedDbIntegrity, DbProcessedDbMetrics,
    DbProjectHash, DbProjectSession, DbProjectUser, DbProjectVerification,
    DbReportRecord, DbSavedSearch, DbRecentSearch, DbTag, DbTagAssignment, DbViewerHistoryEntry,
    FtsSearchResult, ProjectDatabase, ProjectDbStats,
};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::{info, warn};

// =============================================================================
// Global Project Database State
// =============================================================================

/// The currently-open project database (one at a time)
static PROJECT_DB: OnceLock<Mutex<Option<ProjectDatabase>>> = OnceLock::new();

fn get_project_db_lock() -> &'static Mutex<Option<ProjectDatabase>> {
    PROJECT_DB.get_or_init(|| Mutex::new(None))
}

/// Helper: execute a closure with the active project database
fn with_project_db<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&ProjectDatabase) -> rusqlite::Result<T>,
{
    let guard = get_project_db_lock().lock();
    match guard.as_ref() {
        Some(db) => f(db).map_err(|e| format!("Project DB error: {}", e)),
        None => Err("No project database is open. Open or create a project first.".to_string()),
    }
}

// =============================================================================
// Lifecycle Commands
// =============================================================================

/// Open or create a project database for a .cffx project file.
/// If the .ffxdb doesn't exist, it will be created and data migrated from the .cffx.
#[tauri::command]
pub fn project_db_open(cffx_path: String) -> Result<String, String> {
    let cffx = PathBuf::from(&cffx_path);
    let db_path = ProjectDatabase::db_path_for_project(&cffx);

    let is_new = !db_path.exists();

    let db =
        ProjectDatabase::open(&db_path).map_err(|e| format!("Failed to open project DB: {}", e))?;

    // If this is a brand-new .ffxdb, migrate data from the .cffx file
    if is_new {
        if let Ok(content) = std::fs::read_to_string(&cffx) {
            if let Ok(project) = serde_json::from_str::<crate::project::FFXProject>(&content) {
                if let Err(e) = db.migrate_from_project(&project) {
                    warn!("Migration from .cffx had errors: {}", e);
                }
            }
        }
    }

    let db_path_str = db_path.to_string_lossy().to_string();
    info!("Project DB opened: {} (new: {})", db_path_str, is_new);

    // Store as the active project database
    let mut guard = get_project_db_lock().lock();
    *guard = Some(db);

    Ok(db_path_str)
}

/// Close the currently-open project database.
#[tauri::command]
pub fn project_db_close() -> Result<(), String> {
    let mut guard = get_project_db_lock().lock();
    if guard.is_some() {
        *guard = None;
        info!("Project DB closed");
    }
    Ok(())
}

/// Check if a project database is currently open.
#[tauri::command]
pub fn project_db_is_open() -> bool {
    let guard = get_project_db_lock().lock();
    guard.is_some()
}

/// Get the file path of the currently-open project database.
#[tauri::command]
pub fn project_db_path() -> Result<String, String> {
    with_project_db(|db| Ok(db.path().to_string_lossy().to_string()))
}

// =============================================================================
// Statistics
// =============================================================================

/// Get project database statistics.
#[tauri::command]
pub fn project_db_get_stats() -> Result<ProjectDbStats, String> {
    with_project_db(|db| db.get_stats())
}

// =============================================================================
// Activity Log Commands
// =============================================================================

/// Insert a new activity log entry.
#[tauri::command]
pub fn project_db_insert_activity(entry: DbActivityEntry) -> Result<(), String> {
    with_project_db(|db| db.insert_activity(&entry))
}

/// Query activity log with filters.
#[tauri::command]
pub fn project_db_query_activities(query: ActivityQuery) -> Result<Vec<DbActivityEntry>, String> {
    with_project_db(|db| db.query_activities(&query))
}

/// Get total activity count, optionally filtered by category.
#[tauri::command]
pub fn project_db_count_activities(category: Option<String>) -> Result<i64, String> {
    with_project_db(|db| db.count_activities(category.as_deref()))
}

// =============================================================================
// Session Commands
// =============================================================================

/// Insert or update a session.
#[tauri::command]
pub fn project_db_upsert_session(session: DbProjectSession) -> Result<(), String> {
    with_project_db(|db| db.upsert_session(&session))
}

/// Get all sessions.
#[tauri::command]
pub fn project_db_get_sessions() -> Result<Vec<DbProjectSession>, String> {
    with_project_db(|db| db.get_sessions())
}

/// End a session (set ended_at and duration).
#[tauri::command]
pub fn project_db_end_session(
    session_id: String,
    summary: Option<String>,
) -> Result<(), String> {
    with_project_db(|db| db.end_session(&session_id, summary.as_deref()))
}

// =============================================================================
// User Commands
// =============================================================================

/// Insert or update a user.
#[tauri::command]
pub fn project_db_upsert_user(user: DbProjectUser) -> Result<(), String> {
    with_project_db(|db| db.upsert_user(&user))
}

/// Get all users.
#[tauri::command]
pub fn project_db_get_users() -> Result<Vec<DbProjectUser>, String> {
    with_project_db(|db| db.get_users())
}

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

// =============================================================================
// Bookmark Commands
// =============================================================================

/// Insert or update a bookmark.
#[tauri::command]
pub fn project_db_upsert_bookmark(bookmark: DbBookmark) -> Result<(), String> {
    with_project_db(|db| db.upsert_bookmark(&bookmark))
}

/// Get all bookmarks.
#[tauri::command]
pub fn project_db_get_bookmarks() -> Result<Vec<DbBookmark>, String> {
    with_project_db(|db| db.get_bookmarks())
}

/// Delete a bookmark.
#[tauri::command]
pub fn project_db_delete_bookmark(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_bookmark(&id))
}

// =============================================================================
// Note Commands
// =============================================================================

/// Insert or update a note.
#[tauri::command]
pub fn project_db_upsert_note(note: DbNote) -> Result<(), String> {
    with_project_db(|db| db.upsert_note(&note))
}

/// Get all notes.
#[tauri::command]
pub fn project_db_get_notes() -> Result<Vec<DbNote>, String> {
    with_project_db(|db| db.get_notes())
}

/// Delete a note.
#[tauri::command]
pub fn project_db_delete_note(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_note(&id))
}

// =============================================================================
// Tag Commands
// =============================================================================

/// Insert or update a tag definition.
#[tauri::command]
pub fn project_db_upsert_tag(tag: DbTag) -> Result<(), String> {
    with_project_db(|db| db.upsert_tag(&tag))
}

/// Get all tags.
#[tauri::command]
pub fn project_db_get_tags() -> Result<Vec<DbTag>, String> {
    with_project_db(|db| db.get_tags())
}

/// Delete a tag and its assignments.
#[tauri::command]
pub fn project_db_delete_tag(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_tag(&id))
}

/// Assign a tag to a target.
#[tauri::command]
pub fn project_db_assign_tag(assignment: DbTagAssignment) -> Result<(), String> {
    with_project_db(|db| db.assign_tag(&assignment))
}

/// Remove a tag assignment.
#[tauri::command]
pub fn project_db_remove_tag(
    tag_id: String,
    target_type: String,
    target_id: String,
) -> Result<(), String> {
    with_project_db(|db| db.remove_tag(&tag_id, &target_type, &target_id))
}

/// Get tags for a specific target.
#[tauri::command]
pub fn project_db_get_tags_for_target(
    target_type: String,
    target_id: String,
) -> Result<Vec<DbTag>, String> {
    with_project_db(|db| db.get_tags_for_target(&target_type, &target_id))
}

// =============================================================================
// Report Commands
// =============================================================================

/// Insert a report record.
#[tauri::command]
pub fn project_db_insert_report(report: DbReportRecord) -> Result<(), String> {
    with_project_db(|db| db.insert_report(&report))
}

/// Get all reports.
#[tauri::command]
pub fn project_db_get_reports() -> Result<Vec<DbReportRecord>, String> {
    with_project_db(|db| db.get_reports())
}

// =============================================================================
// Search Commands
// =============================================================================

/// Insert or update a saved search.
#[tauri::command]
pub fn project_db_upsert_saved_search(search: DbSavedSearch) -> Result<(), String> {
    with_project_db(|db| db.upsert_saved_search(&search))
}

/// Get all saved searches.
#[tauri::command]
pub fn project_db_get_saved_searches() -> Result<Vec<DbSavedSearch>, String> {
    with_project_db(|db| db.get_saved_searches())
}

/// Insert or update a recent search.
#[tauri::command]
pub fn project_db_insert_recent_search(search: DbRecentSearch) -> Result<(), String> {
    with_project_db(|db| db.insert_recent_search(&search))
}

// =============================================================================
// Case Document Commands
// =============================================================================

/// Insert or update a case document.
#[tauri::command]
pub fn project_db_upsert_case_document(doc: DbCaseDocument) -> Result<(), String> {
    with_project_db(|db| db.upsert_case_document(&doc))
}

/// Get all case documents.
#[tauri::command]
pub fn project_db_get_case_documents() -> Result<Vec<DbCaseDocument>, String> {
    with_project_db(|db| db.get_case_documents())
}

// =============================================================================
// UI State Commands
// =============================================================================

/// Set a UI state value.
#[tauri::command]
pub fn project_db_set_ui_state(key: String, value: String) -> Result<(), String> {
    with_project_db(|db| db.set_ui_state(&key, &value))
}

/// Get a UI state value.
#[tauri::command]
pub fn project_db_get_ui_state(key: String) -> Result<Option<String>, String> {
    with_project_db(|db| db.get_ui_state(&key))
}

// =============================================================================
// Processed Database Commands
// =============================================================================

/// Insert or update a processed database record.
#[tauri::command]
pub fn project_db_upsert_processed_database(db: DbProcessedDatabase) -> Result<(), String> {
    with_project_db(|pdb| pdb.upsert_processed_database(&db))
}

/// Get all processed databases.
#[tauri::command]
pub fn project_db_get_processed_databases() -> Result<Vec<DbProcessedDatabase>, String> {
    with_project_db(|db| db.get_processed_databases())
}

/// Get a processed database by path.
#[tauri::command]
pub fn project_db_get_processed_database_by_path(
    path: String,
) -> Result<Option<DbProcessedDatabase>, String> {
    with_project_db(|db| db.get_processed_database_by_path(&path))
}

/// Delete a processed database and all related records.
#[tauri::command]
pub fn project_db_delete_processed_database(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_processed_database(&id))
}

// =============================================================================
// Processed DB Integrity Commands
// =============================================================================

/// Insert or update a processed database integrity record.
#[tauri::command]
pub fn project_db_upsert_processed_db_integrity(
    integrity: DbProcessedDbIntegrity,
) -> Result<(), String> {
    with_project_db(|db| db.upsert_processed_db_integrity(&integrity))
}

/// Get integrity records for a processed database.
#[tauri::command]
pub fn project_db_get_processed_db_integrity(
    processed_db_id: String,
) -> Result<Vec<DbProcessedDbIntegrity>, String> {
    with_project_db(|db| db.get_processed_db_integrity(&processed_db_id))
}

// =============================================================================
// Processed DB Metrics Commands
// =============================================================================

/// Insert or update metrics for a processed database.
#[tauri::command]
pub fn project_db_upsert_processed_db_metrics(
    metrics: DbProcessedDbMetrics,
) -> Result<(), String> {
    with_project_db(|db| db.upsert_processed_db_metrics(&metrics))
}

/// Get metrics for a processed database.
#[tauri::command]
pub fn project_db_get_processed_db_metrics(
    processed_db_id: String,
) -> Result<Option<DbProcessedDbMetrics>, String> {
    with_project_db(|db| db.get_processed_db_metrics(&processed_db_id))
}

// =============================================================================
// AXIOM Case Info Commands
// =============================================================================

/// Insert or update AXIOM case information.
#[tauri::command]
pub fn project_db_upsert_axiom_case_info(info: DbAxiomCaseInfo) -> Result<(), String> {
    with_project_db(|db| db.upsert_axiom_case_info(&info))
}

/// Get AXIOM case info for a processed database.
#[tauri::command]
pub fn project_db_get_axiom_case_info(
    processed_db_id: String,
) -> Result<Option<DbAxiomCaseInfo>, String> {
    with_project_db(|db| db.get_axiom_case_info(&processed_db_id))
}

/// Get all AXIOM case info records.
#[tauri::command]
pub fn project_db_get_all_axiom_case_info() -> Result<Vec<DbAxiomCaseInfo>, String> {
    with_project_db(|db| db.get_all_axiom_case_info())
}

// =============================================================================
// AXIOM Evidence Source Commands
// =============================================================================

/// Insert an AXIOM evidence source.
#[tauri::command]
pub fn project_db_insert_axiom_evidence_source(
    source: DbAxiomEvidenceSource,
) -> Result<(), String> {
    with_project_db(|db| db.insert_axiom_evidence_source(&source))
}

/// Get evidence sources for an AXIOM case.
#[tauri::command]
pub fn project_db_get_axiom_evidence_sources(
    axiom_case_id: String,
) -> Result<Vec<DbAxiomEvidenceSource>, String> {
    with_project_db(|db| db.get_axiom_evidence_sources(&axiom_case_id))
}

// =============================================================================
// AXIOM Search Result Commands
// =============================================================================

/// Insert an AXIOM search result.
#[tauri::command]
pub fn project_db_insert_axiom_search_result(
    result: DbAxiomSearchResult,
) -> Result<(), String> {
    with_project_db(|db| db.insert_axiom_search_result(&result))
}

/// Get search results for an AXIOM case.
#[tauri::command]
pub fn project_db_get_axiom_search_results(
    axiom_case_id: String,
) -> Result<Vec<DbAxiomSearchResult>, String> {
    with_project_db(|db| db.get_axiom_search_results(&axiom_case_id))
}

// =============================================================================
// Artifact Category Commands
// =============================================================================

/// Insert or replace artifact categories for a processed database.
#[tauri::command]
pub fn project_db_upsert_artifact_categories(
    categories: Vec<DbArtifactCategory>,
) -> Result<(), String> {
    with_project_db(|db| db.upsert_artifact_categories(&categories))
}

/// Get artifact categories for a processed database.
#[tauri::command]
pub fn project_db_get_artifact_categories(
    processed_db_id: String,
) -> Result<Vec<DbArtifactCategory>, String> {
    with_project_db(|db| db.get_artifact_categories(&processed_db_id))
}

// =============================================================================
// Export History Commands
// =============================================================================

/// Insert an export record.
#[tauri::command]
pub fn project_db_insert_export(record: DbExportRecord) -> Result<(), String> {
    with_project_db(|db| db.insert_export(&record))
}

/// Update an export record (status, completed_at, error, etc.).
#[tauri::command]
pub fn project_db_update_export(record: DbExportRecord) -> Result<(), String> {
    with_project_db(|db| db.update_export(&record))
}

/// Get export records, most recent first.
#[tauri::command]
pub fn project_db_get_exports(limit: Option<i64>) -> Result<Vec<DbExportRecord>, String> {
    with_project_db(|db| db.get_exports(limit))
}

/// Delete an export record.
#[tauri::command]
pub fn project_db_delete_export(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_export(&id))
}

// =============================================================================
// Chain of Custody Commands
// =============================================================================

/// Insert a chain-of-custody record.
#[tauri::command]
pub fn project_db_insert_custody_record(record: DbCustodyRecord) -> Result<(), String> {
    with_project_db(|db| db.insert_custody_record(&record))
}

/// Get all custody records in chronological order.
#[tauri::command]
pub fn project_db_get_custody_records() -> Result<Vec<DbCustodyRecord>, String> {
    with_project_db(|db| db.get_custody_records())
}

/// Delete a custody record.
#[tauri::command]
pub fn project_db_delete_custody_record(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_custody_record(&id))
}

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

// =============================================================================
// Full-Text Search Commands
// =============================================================================

/// Rebuild FTS5 indexes from source tables.
#[tauri::command]
pub fn project_db_rebuild_fts() -> Result<(), String> {
    with_project_db(|db| db.rebuild_fts_indexes())
}

/// Full-text search across notes, bookmarks, and activity log.
#[tauri::command]
pub fn project_db_fts_search(
    query: String,
    limit: Option<i64>,
) -> Result<Vec<FtsSearchResult>, String> {
    with_project_db(|db| db.fts_search(&query, limit))
}

// =============================================================================
// Database Utility Commands
// =============================================================================

/// Run SQLite integrity check on the project database.
#[tauri::command]
pub fn project_db_integrity_check() -> Result<Vec<String>, String> {
    with_project_db(|db| db.integrity_check())
}

/// Force WAL checkpoint (flush write-ahead log to main DB file).
#[tauri::command]
pub fn project_db_wal_checkpoint() -> Result<(i64, i64), String> {
    with_project_db(|db| db.wal_checkpoint())
}

/// Create a backup copy of the project database.
#[tauri::command]
pub fn project_db_backup(dest_path: String) -> Result<(), String> {
    with_project_db(|db| db.backup_to(&dest_path))
}

/// Vacuum the database to reclaim space.
#[tauri::command]
pub fn project_db_vacuum() -> Result<(), String> {
    with_project_db(|db| db.vacuum())
}
