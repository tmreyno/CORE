// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project merge logic — combine multiple .cffx projects into one.

use super::migration::make_paths_absolute;
use super::types::*;
use super::{FFXProject, APP_VERSION, PROJECT_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// A wrapper that holds a rusqlite connection and an optional temp directory.
/// The temp directory is kept alive (not deleted) until this struct is dropped.
struct MergeDbConnection {
    conn: rusqlite::Connection,
    _temp_dir: Option<tempfile::TempDir>,
}

/// Open an .ffxdb database for read-only merge analysis.
///
/// If the database has an active WAL file (`.ffxdb-wal` exists and is non-empty),
/// the main database file may contain no data — it's all in the WAL. Opening with
/// `SQLITE_OPEN_READ_ONLY` prevents WAL replay, so queries see empty tables.
///
/// To handle this forensically (without modifying the original files):
/// 1. Copy `.ffxdb`, `.ffxdb-wal`, and `.ffxdb-shm` to a temp directory
/// 2. Open the temp copy read-write so SQLite replays the WAL automatically
/// 3. Query from the temp copy
/// 4. Temp directory is cleaned up when `MergeDbConnection` is dropped
///
/// If no WAL file exists, the database is opened directly with `READ_ONLY`.
fn open_ffxdb_for_analysis(ffxdb_path: &Path) -> Result<MergeDbConnection, String> {
    let wal_path = ffxdb_path.with_extension("ffxdb-wal");
    let shm_path = ffxdb_path.with_extension("ffxdb-shm");

    // Check if there's an active WAL file with data
    let has_active_wal =
        wal_path.exists() && wal_path.metadata().map(|m| m.len() > 0).unwrap_or(false);

    if has_active_wal {
        info!(
            "WAL file detected for {}, copying to temp dir for safe replay",
            ffxdb_path.display()
        );

        // Create temp directory and copy DB + WAL + SHM
        let temp_dir = tempfile::tempdir()
            .map_err(|e| format!("Failed to create temp dir for WAL replay: {}", e))?;

        let temp_db = temp_dir.path().join("merge_analysis.ffxdb");
        let temp_wal = temp_dir.path().join("merge_analysis.ffxdb-wal");
        let temp_shm = temp_dir.path().join("merge_analysis.ffxdb-shm");

        std::fs::copy(ffxdb_path, &temp_db)
            .map_err(|e| format!("Failed to copy ffxdb to temp: {}", e))?;
        std::fs::copy(&wal_path, &temp_wal)
            .map_err(|e| format!("Failed to copy WAL to temp: {}", e))?;
        if shm_path.exists() {
            // SHM may not exist; WAL replay can proceed without it
            let _ = std::fs::copy(&shm_path, &temp_shm);
        }

        // Open read-write so SQLite replays the WAL automatically
        let open_flags =
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = rusqlite::Connection::open_with_flags(&temp_db, open_flags)
            .map_err(|e| format!("Failed to open temp ffxdb copy: {}", e))?;

        // Force a checkpoint to ensure all WAL data is in the main DB
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");

        Ok(MergeDbConnection {
            conn,
            _temp_dir: Some(temp_dir),
        })
    } else {
        // No WAL — safe to open read-only directly
        let open_flags =
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = rusqlite::Connection::open_with_flags(ffxdb_path, open_flags)
            .map_err(|e| format!("Failed to open ffxdb: {}", e))?;

        Ok(MergeDbConnection {
            conn,
            _temp_dir: None,
        })
    }
}

// =============================================================================
// Merge Result Types
// =============================================================================

/// Summary of a project to be merged (returned by analyze_projects)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMergeSummary {
    /// .cffx file path
    pub cffx_path: String,
    /// Companion .ffxdb path (may not exist)
    pub ffxdb_path: String,
    /// Whether the .ffxdb file exists
    pub ffxdb_exists: bool,
    /// Project name
    pub name: String,
    /// Project ID
    pub project_id: String,
    /// Owner/Examiner name (from project owner_name field)
    pub owner_name: Option<String>,
    /// When created
    pub created_at: String,
    /// When last saved
    pub saved_at: String,
    /// Number of discovered evidence files
    pub evidence_file_count: usize,
    /// Number of computed hashes
    pub hash_count: usize,
    /// Number of sessions
    pub session_count: usize,
    /// Number of activity log entries
    pub activity_count: usize,
    /// Number of bookmarks
    pub bookmark_count: usize,
    /// Number of notes
    pub note_count: usize,
    /// Number of tabs
    pub tab_count: usize,
    /// Number of reports
    pub report_count: usize,
    /// Root path
    pub root_path: String,
    /// Examiners/users found in .cffx and .ffxdb
    pub examiners: Vec<MergeExaminerInfo>,
    /// Evidence collection summaries from .ffxdb
    pub collections: Vec<MergeCollectionSummary>,
    /// COC item summaries from .ffxdb
    pub coc_items: Vec<MergeCocSummary>,
    /// Form submission summaries from .ffxdb
    pub form_submissions: Vec<MergeFormSummary>,
    /// Evidence file summaries from .ffxdb
    pub evidence_files: Vec<MergeEvidenceFileSummary>,
}

/// Examiner/user information gathered from .cffx and .ffxdb sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeExaminerInfo {
    /// Username or examiner name
    pub name: String,
    /// Display name if available
    pub display_name: Option<String>,
    /// Where this name was found
    pub source: String,
    /// Role context (e.g. "project owner", "session user", "collecting officer")
    pub role: String,
}

/// Summary of an evidence collection record from .ffxdb
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeCollectionSummary {
    pub id: String,
    pub case_number: String,
    pub collection_date: String,
    pub collecting_officer: String,
    pub collection_location: String,
    pub status: String,
    pub item_count: usize,
}

/// Summary of a COC item from .ffxdb
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeCocSummary {
    pub id: String,
    pub coc_number: String,
    pub case_number: String,
    pub evidence_id: String,
    pub description: String,
    pub submitted_by: String,
    pub received_by: String,
    pub status: String,
}

/// Summary of a form submission from .ffxdb
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeFormSummary {
    pub id: String,
    pub template_id: String,
    pub case_number: Option<String>,
    pub status: String,
    pub created_at: String,
    /// Collecting officer (extracted from data_json for evidence_collection templates)
    pub collecting_officer: Option<String>,
    /// Collection location (extracted from data_json for evidence_collection templates)
    pub collection_location: Option<String>,
    /// Lead examiner (extracted from data_json for IAR/activity templates)
    pub lead_examiner: Option<String>,
}

/// Summary of an evidence file from .ffxdb
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeEvidenceFileSummary {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub container_type: String,
    pub total_size: u64,
}

/// Owner assignment for a source project during merge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeSourceAssignment {
    /// Source .cffx path
    pub cffx_path: String,
    /// Assigned owner/examiner name for this source project
    pub owner_name: String,
}

/// Provenance record for a merged source project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeSource {
    /// Source project ID
    pub source_project_id: String,
    /// Source project name
    pub source_project_name: String,
    /// Source .cffx file path
    pub source_cffx_path: String,
    /// Owner/Examiner assigned to this source
    pub owner_name: String,
    /// When the merge was performed
    pub merged_at: String,
    /// Record counts from this source
    pub evidence_file_count: usize,
    pub session_count: usize,
    pub activity_count: usize,
    pub bookmark_count: usize,
    pub note_count: usize,
}

/// Result of a merge operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    pub success: bool,
    /// Path to the merged .cffx file
    pub cffx_path: Option<String>,
    /// Path to the merged .ffxdb file
    pub ffxdb_path: Option<String>,
    pub error: Option<String>,
    /// Merge statistics
    pub stats: Option<MergeStats>,
    /// Provenance records for each source project
    pub sources: Option<Vec<MergeSource>>,
}

/// Statistics from the merge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeStats {
    pub projects_merged: usize,
    pub users_merged: usize,
    pub sessions_merged: usize,
    pub activity_entries_merged: usize,
    pub evidence_files_merged: usize,
    pub hashes_merged: usize,
    pub bookmarks_merged: usize,
    pub notes_merged: usize,
    pub tabs_merged: usize,
    pub reports_merged: usize,
    pub tags_merged: usize,
    pub searches_merged: usize,
    pub ffxdb_tables_merged: usize,
}

// =============================================================================
// Analyze Projects
// =============================================================================

/// Analyze a list of .cffx files and return summaries for the merge wizard.
pub fn analyze_projects(cffx_paths: &[String]) -> Vec<ProjectMergeSummary> {
    cffx_paths
        .iter()
        .filter_map(|path| {
            let cffx_path = Path::new(path);
            let ffxdb_path = cffx_path.with_extension("ffxdb");
            let project_dir = cffx_path.parent().unwrap_or(Path::new("."));

            match std::fs::read_to_string(cffx_path) {
                Ok(json) => match serde_json::from_str::<FFXProject>(&json) {
                    Ok(mut project) => {
                        // Resolve relative paths so summaries show absolute paths
                        make_paths_absolute(&mut project, project_dir);

                        let evidence_count = project
                            .evidence_cache
                            .as_ref()
                            .map(|c| c.discovered_files.len())
                            .unwrap_or(0);
                        let hash_count = project
                            .evidence_cache
                            .as_ref()
                            .map(|c| c.computed_hashes.len())
                            .unwrap_or(0);

                        // Gather examiners from .cffx
                        let mut examiners: Vec<MergeExaminerInfo> = Vec::new();
                        if let Some(ref owner) = project.owner_name {
                            if !owner.is_empty() {
                                examiners.push(MergeExaminerInfo {
                                    name: owner.clone(),
                                    display_name: None,
                                    source: "cffx".to_string(),
                                    role: "project owner".to_string(),
                                });
                            }
                        }
                        for u in &project.users {
                            if !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(&u.username)) {
                                examiners.push(MergeExaminerInfo {
                                    name: u.username.clone(),
                                    display_name: u.display_name.clone(),
                                    source: "cffx".to_string(),
                                    role: "session user".to_string(),
                                });
                            }
                        }

                        // Query .ffxdb for detailed records
                        let ffxdb_exists = ffxdb_path.exists();
                        let mut collections = Vec::new();
                        let mut coc_items_list = Vec::new();
                        let mut form_submissions = Vec::new();
                        let mut evidence_files = Vec::new();

                        if ffxdb_exists {
                            match open_ffxdb_for_analysis(&ffxdb_path) {
                            Ok(db) => {
                                let conn = &db.conn;
                                // Users/examiners from ffxdb
                                query_ffxdb_examiners(conn, &mut examiners);
                                // Evidence collections
                                collections = query_ffxdb_collections(conn);
                                // Add collecting officers as examiners
                                for c in &collections {
                                    if !c.collecting_officer.is_empty()
                                        && !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(&c.collecting_officer))
                                    {
                                        examiners.push(MergeExaminerInfo {
                                            name: c.collecting_officer.clone(),
                                            display_name: None,
                                            source: "ffxdb".to_string(),
                                            role: "collecting officer".to_string(),
                                        });
                                    }
                                }
                                // COC items
                                coc_items_list = query_ffxdb_coc_items(conn);
                                // Add COC submitted_by / received_by as examiners
                                for coc in &coc_items_list {
                                    for (name, role) in [
                                        (&coc.submitted_by, "submitted by (COC)"),
                                        (&coc.received_by, "received by (COC)"),
                                    ] {
                                        if !name.is_empty()
                                            && !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(name))
                                        {
                                            examiners.push(MergeExaminerInfo {
                                                name: name.clone(),
                                                display_name: None,
                                                source: "ffxdb".to_string(),
                                                role: role.to_string(),
                                            });
                                        }
                                    }
                                }
                                // Form submissions
                                form_submissions = query_ffxdb_forms(conn);
                                // Add form examiners (collecting officers, lead examiners) to examiner list
                                for f in &form_submissions {
                                    for (name_opt, role) in [
                                        (&f.collecting_officer, "collecting officer (form)"),
                                        (&f.lead_examiner, "lead examiner (form)"),
                                    ] {
                                        if let Some(name) = name_opt {
                                            if !name.is_empty()
                                                && !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(name))
                                            {
                                                examiners.push(MergeExaminerInfo {
                                                    name: name.clone(),
                                                    display_name: None,
                                                    source: "ffxdb".to_string(),
                                                    role: role.to_string(),
                                                });
                                            }
                                        }
                                    }
                                }
                                // Evidence files
                                evidence_files = query_ffxdb_evidence_files(conn);
                                // Processed DB examiners
                                query_ffxdb_processed_db_examiners(conn, &mut examiners);
                                // Additional clues from sessions, activity, bookmarks, notes, reports, exports, COC chain
                                query_ffxdb_additional_clues(conn, &mut examiners);

                                info!(
                                    "Merge analyze ffxdb {}: {} examiners, {} collections, {} COC, {} forms, {} evidence files",
                                    ffxdb_path.display(),
                                    examiners.len(),
                                    collections.len(),
                                    coc_items_list.len(),
                                    form_submissions.len(),
                                    evidence_files.len(),
                                );
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to open ffxdb for merge analyze: {} — {}",
                                    ffxdb_path.display(),
                                    e
                                );
                            }
                            }
                        }

                        Some(ProjectMergeSummary {
                            cffx_path: path.clone(),
                            ffxdb_path: ffxdb_path.to_string_lossy().to_string(),
                            ffxdb_exists,
                            name: project.name.clone(),
                            project_id: project.project_id.clone(),
                            owner_name: project.owner_name.clone(),
                            created_at: project.created_at.clone(),
                            saved_at: project.saved_at.clone(),
                            evidence_file_count: evidence_count,
                            hash_count,
                            session_count: project.sessions.len(),
                            activity_count: project.activity_log.len(),
                            bookmark_count: project.bookmarks.len(),
                            note_count: project.notes.len(),
                            tab_count: project.tabs.len(),
                            report_count: project.reports.len(),
                            root_path: project.root_path.clone(),
                            examiners,
                            collections,
                            coc_items: coc_items_list,
                            form_submissions,
                            evidence_files,
                        })
                    }
                    Err(e) => {
                        warn!("Failed to parse {}: {}", path, e);
                        None
                    }
                },
                Err(e) => {
                    warn!("Failed to read {}: {}", path, e);
                    None
                }
            }
        })
        .collect()
}

// =============================================================================
// .ffxdb Query Helpers (for merge analysis)
// =============================================================================

/// Query users table from .ffxdb and add to examiners list
fn query_ffxdb_examiners(conn: &rusqlite::Connection, examiners: &mut Vec<MergeExaminerInfo>) {
    let sql = "SELECT username, display_name FROM users";
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        let (username, display_name) = row;
                        if !examiners
                            .iter()
                            .any(|e| e.name.eq_ignore_ascii_case(&username))
                        {
                            examiners.push(MergeExaminerInfo {
                                name: username,
                                display_name,
                                source: "ffxdb".to_string(),
                                role: "session user".to_string(),
                            });
                        }
                    }
                }
                Err(e) => warn!("merge: query users failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare users query failed: {}", e),
    }
}

/// Query evidence_collections from .ffxdb
fn query_ffxdb_collections(conn: &rusqlite::Connection) -> Vec<MergeCollectionSummary> {
    let sql = "SELECT ec.id, ec.case_number, ec.collection_date, ec.collecting_officer, \
               ec.collection_location, ec.status, \
               (SELECT COUNT(*) FROM collected_items ci WHERE ci.collection_id = ec.id) as item_count \
               FROM evidence_collections ec ORDER BY ec.collection_date DESC";
    let mut results = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok(MergeCollectionSummary {
                    id: row.get(0)?,
                    case_number: row.get(1)?,
                    collection_date: row.get(2)?,
                    collecting_officer: row.get(3)?,
                    collection_location: row.get(4)?,
                    status: row.get(5)?,
                    item_count: row.get::<_, i64>(6).unwrap_or(0) as usize,
                })
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        results.push(row);
                    }
                }
                Err(e) => warn!("merge: query evidence_collections failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare evidence_collections query failed: {}", e),
    }
    results
}

/// Query coc_items from .ffxdb
fn query_ffxdb_coc_items(conn: &rusqlite::Connection) -> Vec<MergeCocSummary> {
    let sql = "SELECT id, coc_number, case_number, evidence_id, description, \
               submitted_by, received_by, status FROM coc_items ORDER BY created_at DESC";
    let mut results = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok(MergeCocSummary {
                    id: row.get(0)?,
                    coc_number: row.get(1)?,
                    case_number: row.get(2)?,
                    evidence_id: row.get(3)?,
                    description: row.get(4)?,
                    submitted_by: row.get(5)?,
                    received_by: row.get(6)?,
                    status: row.get(7)?,
                })
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        results.push(row);
                    }
                }
                Err(e) => warn!("merge: query coc_items failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare coc_items query failed: {}", e),
    }
    results
}

/// Query form_submissions from .ffxdb (includes data_json for detail extraction)
fn query_ffxdb_forms(conn: &rusqlite::Connection) -> Vec<MergeFormSummary> {
    let sql = "SELECT id, template_id, case_number, status, created_at, data_json \
               FROM form_submissions ORDER BY created_at DESC";
    let mut results = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                let data_json: Option<String> = row.get(5)?;
                // Extract useful fields from data_json
                let (collecting_officer, collection_location, lead_examiner) =
                    extract_form_details(data_json.as_deref());
                Ok(MergeFormSummary {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    case_number: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                    collecting_officer,
                    collection_location,
                    lead_examiner,
                })
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        results.push(row);
                    }
                }
                Err(e) => warn!("merge: query form_submissions failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare form_submissions query failed: {}", e),
    }
    results
}

/// Extract useful display fields from form_submissions.data_json
fn extract_form_details(
    data_json: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>) {
    let Some(json_str) = data_json else {
        return (None, None, None);
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return (None, None, None);
    };
    let obj = val.as_object();
    let get_str = |key: &str| -> Option<String> {
        obj.and_then(|o| o.get(key))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    };
    (
        get_str("collecting_officer"),
        get_str("collection_location"),
        get_str("lead_examiner"),
    )
}

/// Query evidence_files from .ffxdb
fn query_ffxdb_evidence_files(conn: &rusqlite::Connection) -> Vec<MergeEvidenceFileSummary> {
    let sql = "SELECT id, path, filename, container_type, total_size \
               FROM evidence_files ORDER BY filename ASC";
    let mut results = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok(MergeEvidenceFileSummary {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    filename: row.get(2)?,
                    container_type: row.get(3)?,
                    total_size: row.get::<_, i64>(4).unwrap_or(0) as u64,
                })
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        results.push(row);
                    }
                }
                Err(e) => warn!("merge: query evidence_files failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare evidence_files query failed: {}", e),
    }
    results
}

/// Query processed_databases and axiom_case_info for examiner names
fn query_ffxdb_processed_db_examiners(
    conn: &rusqlite::Connection,
    examiners: &mut Vec<MergeExaminerInfo>,
) {
    // processed_databases.examiner
    let sql = "SELECT DISTINCT examiner FROM processed_databases WHERE examiner IS NOT NULL AND examiner != ''";
    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for name in rows.flatten() {
                if !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(&name)) {
                    examiners.push(MergeExaminerInfo {
                        name,
                        display_name: None,
                        source: "ffxdb".to_string(),
                        role: "processed DB examiner".to_string(),
                    });
                }
            }
        }
    }
    // axiom_case_info.examiner
    let sql2 = "SELECT DISTINCT examiner FROM axiom_case_info WHERE examiner IS NOT NULL AND examiner != ''";
    if let Ok(mut stmt) = conn.prepare(sql2) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for name in rows.flatten() {
                if !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(&name)) {
                    examiners.push(MergeExaminerInfo {
                        name,
                        display_name: None,
                        source: "ffxdb".to_string(),
                        role: "AXIOM examiner".to_string(),
                    });
                }
            }
        }
    }
}

/// Query additional tables for user/examiner clues when owner is still unknown.
/// Checks sessions, activity_log, bookmarks, notes, reports, export_history,
/// and chain_of_custody for distinct usernames not already in the examiner list.
fn query_ffxdb_additional_clues(
    conn: &rusqlite::Connection,
    examiners: &mut Vec<MergeExaminerInfo>,
) {
    // Each tuple: (SQL, role label)
    let queries: &[(&str, &str)] = &[
        (
            "SELECT DISTINCT user FROM sessions WHERE user IS NOT NULL AND user != ''",
            "session user",
        ),
        (
            "SELECT DISTINCT user FROM activity_log WHERE user IS NOT NULL AND user != ''",
            "activity user",
        ),
        (
            "SELECT DISTINCT created_by FROM bookmarks WHERE created_by IS NOT NULL AND created_by != ''",
            "bookmark author",
        ),
        (
            "SELECT DISTINCT created_by FROM notes WHERE created_by IS NOT NULL AND created_by != ''",
            "note author",
        ),
        (
            "SELECT DISTINCT generated_by FROM reports WHERE generated_by IS NOT NULL AND generated_by != ''",
            "report author",
        ),
        (
            "SELECT DISTINCT initiated_by FROM export_history WHERE initiated_by IS NOT NULL AND initiated_by != ''",
            "export initiator",
        ),
        (
            "SELECT DISTINCT recorded_by FROM chain_of_custody WHERE recorded_by IS NOT NULL AND recorded_by != ''",
            "COC recorder",
        ),
        (
            "SELECT DISTINCT from_person FROM chain_of_custody WHERE from_person IS NOT NULL AND from_person != ''",
            "COC from",
        ),
        (
            "SELECT DISTINCT to_person FROM chain_of_custody WHERE to_person IS NOT NULL AND to_person != ''",
            "COC to",
        ),
    ];

    for (sql, role) in queries {
        if let Ok(mut stmt) = conn.prepare(sql) {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                for name in rows.flatten() {
                    if !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(&name)) {
                        examiners.push(MergeExaminerInfo {
                            name,
                            display_name: None,
                            source: "ffxdb".to_string(),
                            role: role.to_string(),
                        });
                    }
                }
            }
        }
    }
}

// =============================================================================
// Merge .cffx Projects
// =============================================================================

/// Merge multiple FFXProject structs into a single project.
///
/// Strategy:
/// - Metadata: use provided name/root_path, earliest created_at, current saved_at
/// - Users: union by username
/// - Sessions: union by session_id
/// - Activity log: union by id, sort by timestamp
/// - Evidence cache: union discovered files by path (prefer latest data)
/// - Hashes: union by (path, algorithm)
/// - Bookmarks: union by id
/// - Notes: union by id
/// - Tags: union by id
/// - Reports: union by id
/// - Searches: union by id
/// - Tabs: take from the most recent project
/// - UI state: take from the most recent project
pub fn merge_projects(projects: &[FFXProject], merged_name: &str, merged_root: &str) -> FFXProject {
    info!("Merging {} projects into '{}'", projects.len(), merged_name);

    let now = chrono::Utc::now().to_rfc3339();

    // Find earliest created_at
    let earliest_created = projects
        .iter()
        .map(|p| p.created_at.as_str())
        .min()
        .unwrap_or(&now)
        .to_string();

    // Find the most recently saved project (for UI state, tabs, etc.)
    let newest_idx = projects
        .iter()
        .enumerate()
        .max_by_key(|(_, p)| &p.saved_at)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let newest = &projects[newest_idx];

    // --- Merge users (union by username) ---
    let mut users_map: HashMap<String, ProjectUser> = HashMap::new();
    for p in projects {
        for u in &p.users {
            users_map
                .entry(u.username.clone())
                .and_modify(|existing| {
                    // Keep earliest first_access, latest last_access
                    if u.first_access < existing.first_access {
                        existing.first_access = u.first_access.clone();
                    }
                    if u.last_access > existing.last_access {
                        existing.last_access = u.last_access.clone();
                    }
                    // Prefer non-None display_name/hostname
                    if existing.display_name.is_none() && u.display_name.is_some() {
                        existing.display_name = u.display_name.clone();
                    }
                    if existing.hostname.is_none() && u.hostname.is_some() {
                        existing.hostname = u.hostname.clone();
                    }
                })
                .or_insert_with(|| u.clone());
        }
    }

    // --- Merge sessions (union by session_id) ---
    let mut sessions_map: HashMap<String, ProjectSession> = HashMap::new();
    for p in projects {
        for s in &p.sessions {
            sessions_map
                .entry(s.session_id.clone())
                .or_insert_with(|| s.clone());
        }
    }

    // --- Merge activity log (union by id, sort by timestamp) ---
    let mut activity_map: HashMap<String, ActivityLogEntry> = HashMap::new();
    for p in projects {
        for a in &p.activity_log {
            activity_map
                .entry(a.id.clone())
                .or_insert_with(|| a.clone());
        }
    }
    let mut merged_activity: Vec<ActivityLogEntry> = activity_map.into_values().collect();
    merged_activity.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    // --- Merge evidence cache (union discovered files by path) ---
    let mut discovered_map: HashMap<String, CachedDiscoveredFile> = HashMap::new();
    let mut file_info_map: HashMap<String, serde_json::Value> = HashMap::new();
    let mut computed_hashes_map: HashMap<String, CachedFileHash> = HashMap::new();
    for p in projects {
        if let Some(ref cache) = p.evidence_cache {
            for f in &cache.discovered_files {
                discovered_map
                    .entry(f.path.clone())
                    .or_insert_with(|| f.clone());
            }
            for (path, info) in &cache.file_info {
                file_info_map
                    .entry(path.clone())
                    .or_insert_with(|| info.clone());
            }
            for (path, hash) in &cache.computed_hashes {
                computed_hashes_map
                    .entry(path.clone())
                    .or_insert_with(|| hash.clone());
            }
        }
    }

    let merged_evidence_cache = if discovered_map.is_empty()
        && file_info_map.is_empty()
        && computed_hashes_map.is_empty()
    {
        None
    } else {
        Some(EvidenceCache {
            discovered_files: discovered_map.into_values().collect(),
            file_info: file_info_map,
            computed_hashes: computed_hashes_map,
            cached_at: now.clone(),
            valid: true,
        })
    };

    // --- Merge hash history (union by path+algorithm) ---
    let mut hash_history_map: HashMap<String, Vec<ProjectFileHash>> = HashMap::new();
    for p in projects {
        for (path, hashes) in &p.hash_history.files {
            let entry = hash_history_map.entry(path.clone()).or_default();
            for h in hashes {
                // Dedup by algorithm+hash_value
                let exists = entry
                    .iter()
                    .any(|e| e.algorithm == h.algorithm && e.hash_value == h.hash_value);
                if !exists {
                    entry.push(h.clone());
                }
            }
        }
    }

    // --- Merge bookmarks (union by id) ---
    let mut bookmarks_map: HashMap<String, ProjectBookmark> = HashMap::new();
    for p in projects {
        for b in &p.bookmarks {
            bookmarks_map
                .entry(b.id.clone())
                .or_insert_with(|| b.clone());
        }
    }

    // --- Merge notes (union by id) ---
    let mut notes_map: HashMap<String, ProjectNote> = HashMap::new();
    for p in projects {
        for n in &p.notes {
            notes_map.entry(n.id.clone()).or_insert_with(|| n.clone());
        }
    }

    // --- Merge tags (union by id) ---
    let mut tags_map: HashMap<String, ProjectTag> = HashMap::new();
    for p in projects {
        for t in &p.tags {
            tags_map.entry(t.id.clone()).or_insert_with(|| t.clone());
        }
    }

    // --- Merge reports (union by id) ---
    let mut reports_map: HashMap<String, ProjectReportRecord> = HashMap::new();
    for p in projects {
        for r in &p.reports {
            reports_map.entry(r.id.clone()).or_insert_with(|| r.clone());
        }
    }

    // --- Merge saved searches (union by id) ---
    let mut searches_map: HashMap<String, SavedSearch> = HashMap::new();
    for p in projects {
        for s in &p.saved_searches {
            searches_map
                .entry(s.id.clone())
                .or_insert_with(|| s.clone());
        }
    }

    // --- Merge recent searches (union by query, prefer most recent) ---
    let mut recent_map: HashMap<String, RecentSearch> = HashMap::new();
    for p in projects {
        for rs in &p.recent_searches {
            recent_map
                .entry(rs.query.clone())
                .and_modify(|existing| {
                    if rs.timestamp > existing.timestamp {
                        *existing = rs.clone();
                    }
                })
                .or_insert_with(|| rs.clone());
        }
    }

    // --- Merge open directories (union by path) ---
    let mut open_dirs_map: HashMap<String, OpenDirectory> = HashMap::new();
    for p in projects {
        for d in &p.open_directories {
            open_dirs_map
                .entry(d.path.clone())
                .or_insert_with(|| d.clone());
        }
    }

    // --- Merge recent directories (union by path) ---
    let mut recent_dirs_map: HashMap<String, RecentDirectory> = HashMap::new();
    for p in projects {
        for d in &p.recent_directories {
            recent_dirs_map
                .entry(d.path.clone())
                .and_modify(|existing| {
                    if d.last_opened > existing.last_opened {
                        *existing = d.clone();
                    }
                })
                .or_insert_with(|| d.clone());
        }
    }

    // --- Merge case documents cache (union documents by path) ---
    let mut case_docs: HashMap<String, CachedCaseDocument> = HashMap::new();
    let mut search_path = String::new();
    for p in projects {
        if let Some(ref cache) = p.case_documents_cache {
            if !cache.search_path.is_empty() {
                search_path = cache.search_path.clone();
            }
            for doc in &cache.documents {
                case_docs
                    .entry(doc.path.clone())
                    .or_insert_with(|| doc.clone());
            }
        }
    }
    let merged_case_docs = if case_docs.is_empty() {
        None
    } else {
        Some(CaseDocumentsCache {
            documents: case_docs.into_values().collect(),
            search_path,
            cached_at: now.clone(),
            valid: true,
        })
    };

    // --- Merge processed database state ---
    let mut pd_loaded: HashSet<String> = HashSet::new();
    let mut pd_integrity: HashMap<String, ProcessedDbIntegrity> = HashMap::new();
    let mut pd_cached_metadata: HashMap<String, serde_json::Value> = HashMap::new();
    let mut pd_cached_databases: Vec<serde_json::Value> = Vec::new();
    let mut pd_axiom: HashMap<String, serde_json::Value> = HashMap::new();
    let mut pd_categories: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut pd_seen_db_paths: HashSet<String> = HashSet::new();

    for p in projects {
        for path in &p.processed_databases.loaded_paths {
            pd_loaded.insert(path.clone());
        }
        for (k, v) in &p.processed_databases.integrity {
            pd_integrity.entry(k.clone()).or_insert_with(|| v.clone());
        }
        if let Some(ref meta) = p.processed_databases.cached_metadata {
            for (k, v) in meta {
                pd_cached_metadata
                    .entry(k.clone())
                    .or_insert_with(|| v.clone());
            }
        }
        if let Some(ref dbs) = p.processed_databases.cached_databases {
            for db in dbs {
                // Dedup by path field
                let db_path = db
                    .as_object()
                    .and_then(|o| o.get("path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if pd_seen_db_paths.insert(db_path) {
                    pd_cached_databases.push(db.clone());
                }
            }
        }
        if let Some(ref axiom) = p.processed_databases.cached_axiom_case_info {
            for (k, v) in axiom {
                pd_axiom.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        if let Some(ref cats) = p.processed_databases.cached_artifact_categories {
            for (k, v) in cats {
                pd_categories.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
    }

    // --- Merge locations (prefer non-None, then newest) ---
    let mut merged_locations: Option<ProjectLocations> = None;
    for p in projects {
        if let Some(ref loc) = p.locations {
            merged_locations = Some(loc.clone());
        }
    }

    // --- Merge custom_data (union keys, prefer newest) ---
    let mut custom_data: HashMap<String, serde_json::Value> = HashMap::new();
    for p in projects {
        if let Some(ref cd) = p.custom_data {
            for (k, v) in cd {
                custom_data.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
    }

    // --- Collect stats ---
    let users: Vec<ProjectUser> = users_map.into_values().collect();
    let sessions: Vec<ProjectSession> = sessions_map.into_values().collect();
    let bookmarks: Vec<ProjectBookmark> = bookmarks_map.into_values().collect();
    let notes: Vec<ProjectNote> = notes_map.into_values().collect();
    let tags: Vec<ProjectTag> = tags_map.into_values().collect();
    let reports: Vec<ProjectReportRecord> = reports_map.into_values().collect();
    let saved_searches: Vec<SavedSearch> = searches_map.into_values().collect();
    let recent_searches: Vec<RecentSearch> = recent_map.into_values().collect();
    let open_directories: Vec<OpenDirectory> = open_dirs_map.into_values().collect();
    let recent_directories: Vec<RecentDirectory> = recent_dirs_map.into_values().collect();

    info!(
        "Merge complete: {} users, {} sessions, {} activity, {} evidence files",
        users.len(),
        sessions.len(),
        merged_activity.len(),
        merged_evidence_cache
            .as_ref()
            .map(|c| c.discovered_files.len())
            .unwrap_or(0),
    );

    FFXProject {
        version: PROJECT_VERSION,
        project_id: generate_merge_id(),
        name: merged_name.to_string(),
        owner_name: None, // Set by execute_merge
        description: Some(format!(
            "Merged from {} projects on {}",
            projects.len(),
            &now[..10]
        )),
        root_path: merged_root.to_string(),
        created_at: earliest_created,
        saved_at: now.clone(),
        created_by_version: APP_VERSION.to_string(),
        saved_by_version: APP_VERSION.to_string(),
        db_path: None, // Will be set after DB merge
        users,
        current_user: newest.current_user.clone(),
        sessions,
        current_session_id: None, // Fresh session on next open
        activity_log: merged_activity,
        activity_log_limit: 5000, // Higher limit for merged project
        locations: merged_locations,
        open_directories,
        recent_directories,
        tabs: newest.tabs.clone(), // Take tabs from most recent project
        active_tab_path: newest.active_tab_path.clone(),
        center_pane_state: newest.center_pane_state.clone(),
        file_selection: FileSelectionState {
            selected_paths: Vec::new(),
            active_path: None,
            timestamp: now.clone(),
        },
        hash_history: ProjectHashHistory {
            files: hash_history_map,
        },
        evidence_cache: merged_evidence_cache,
        case_documents_cache: merged_case_docs,
        preview_cache: None, // Fresh preview cache
        processed_databases: ProcessedDatabaseState {
            loaded_paths: pd_loaded.into_iter().collect(),
            selected_path: newest.processed_databases.selected_path.clone(),
            detail_view_type: newest.processed_databases.detail_view_type.clone(),
            integrity: pd_integrity,
            cached_metadata: if pd_cached_metadata.is_empty() {
                None
            } else {
                Some(pd_cached_metadata)
            },
            cached_databases: if pd_cached_databases.is_empty() {
                None
            } else {
                Some(pd_cached_databases)
            },
            cached_axiom_case_info: if pd_axiom.is_empty() {
                None
            } else {
                Some(pd_axiom)
            },
            cached_artifact_categories: if pd_categories.is_empty() {
                None
            } else {
                Some(pd_categories)
            },
        },
        bookmarks,
        notes,
        tags,
        reports,
        saved_searches,
        recent_searches,
        filter_state: newest.filter_state.clone(),
        ui_state: newest.ui_state.clone(),
        settings: newest.settings.clone(),
        merge_sources: None, // Set by execute_merge after building provenance
        custom_data: if custom_data.is_empty() {
            None
        } else {
            Some(custom_data)
        },
        app_version: Some(APP_VERSION.to_string()),
    }
}

/// Generate a unique project ID for the merged project
fn generate_merge_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after UNIX_EPOCH")
        .as_millis();
    format!("merged_{}", timestamp)
}

// =============================================================================
// Merge .ffxdb Databases
// =============================================================================

/// Merge multiple .ffxdb databases into one using SQLite ATTACH + INSERT OR IGNORE.
///
/// The target database should already exist (created by project_db_open for the merged .cffx).
/// Source databases are attached one at a time and their data merged in.
pub fn merge_databases(
    target_db_path: &Path,
    source_db_paths: &[PathBuf],
) -> Result<MergeStats, String> {
    use rusqlite::Connection;

    let conn =
        Connection::open(target_db_path).map_err(|e| format!("Failed to open target DB: {}", e))?;

    // Enable WAL mode for performance
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("Failed to set pragmas: {}", e))?;

    let mut total_stats = MergeStats {
        projects_merged: source_db_paths.len(),
        users_merged: 0,
        sessions_merged: 0,
        activity_entries_merged: 0,
        evidence_files_merged: 0,
        hashes_merged: 0,
        bookmarks_merged: 0,
        notes_merged: 0,
        tabs_merged: 0,
        reports_merged: 0,
        tags_merged: 0,
        searches_merged: 0,
        ffxdb_tables_merged: 0,
    };

    // Tables to merge with their INSERT OR IGNORE statements
    // Each entry: (table_name, insert_sql)
    let merge_tables: Vec<(&str, &str)> = vec![
        ("users",
         "INSERT OR IGNORE INTO users SELECT * FROM source.users"),
        ("sessions",
         "INSERT OR IGNORE INTO sessions SELECT * FROM source.sessions"),
        ("activity_log",
         "INSERT OR IGNORE INTO activity_log SELECT * FROM source.activity_log"),
        ("evidence_files",
         "INSERT OR IGNORE INTO evidence_files SELECT * FROM source.evidence_files"),
        ("hashes",
         "INSERT OR IGNORE INTO hashes SELECT * FROM source.hashes"),
        ("verifications",
         "INSERT OR IGNORE INTO verifications SELECT * FROM source.verifications"),
        ("bookmarks",
         "INSERT OR IGNORE INTO bookmarks SELECT * FROM source.bookmarks"),
        ("notes",
         "INSERT OR IGNORE INTO notes SELECT * FROM source.notes"),
        ("tags",
         "INSERT OR IGNORE INTO tags SELECT * FROM source.tags"),
        ("tag_assignments",
         "INSERT OR IGNORE INTO tag_assignments SELECT * FROM source.tag_assignments"),
        ("reports",
         "INSERT OR IGNORE INTO reports SELECT * FROM source.reports"),
        ("saved_searches",
         "INSERT OR IGNORE INTO saved_searches SELECT * FROM source.saved_searches"),
        ("recent_searches",
         "INSERT OR IGNORE INTO recent_searches SELECT * FROM source.recent_searches"),
        ("case_documents",
         "INSERT OR IGNORE INTO case_documents SELECT * FROM source.case_documents"),
        ("processed_databases",
         "INSERT OR IGNORE INTO processed_databases SELECT * FROM source.processed_databases"),
        ("axiom_case_info",
         "INSERT OR IGNORE INTO axiom_case_info SELECT * FROM source.axiom_case_info"),
        ("axiom_evidence_sources",
         "INSERT OR IGNORE INTO axiom_evidence_sources SELECT * FROM source.axiom_evidence_sources"),
        ("axiom_search_results",
         "INSERT OR IGNORE INTO axiom_search_results SELECT * FROM source.axiom_search_results"),
        ("artifact_categories",
         "INSERT OR IGNORE INTO artifact_categories SELECT * FROM source.artifact_categories"),
        ("coc_items",
         "INSERT OR IGNORE INTO coc_items SELECT * FROM source.coc_items"),
        ("coc_amendments",
         "INSERT OR IGNORE INTO coc_amendments SELECT * FROM source.coc_amendments"),
        ("coc_audit_log",
         "INSERT OR IGNORE INTO coc_audit_log SELECT * FROM source.coc_audit_log"),
        ("coc_transfers",
         "INSERT OR IGNORE INTO coc_transfers SELECT * FROM source.coc_transfers"),
        ("evidence_collections",
         "INSERT OR IGNORE INTO evidence_collections SELECT * FROM source.evidence_collections"),
        ("collected_items",
         "INSERT OR IGNORE INTO collected_items SELECT * FROM source.collected_items"),
        ("form_submissions",
         "INSERT OR IGNORE INTO form_submissions SELECT * FROM source.form_submissions"),
        ("chain_of_custody",
         "INSERT OR IGNORE INTO chain_of_custody SELECT * FROM source.chain_of_custody"),
        ("export_history",
         "INSERT OR IGNORE INTO export_history SELECT * FROM source.export_history"),
        ("extraction_log",
         "INSERT OR IGNORE INTO extraction_log SELECT * FROM source.extraction_log"),
        ("viewer_history",
         "INSERT OR IGNORE INTO viewer_history SELECT * FROM source.viewer_history"),
        ("annotations",
         "INSERT OR IGNORE INTO annotations SELECT * FROM source.annotations"),
        ("evidence_relationships",
         "INSERT OR IGNORE INTO evidence_relationships SELECT * FROM source.evidence_relationships"),
        ("file_classifications",
         "INSERT OR IGNORE INTO file_classifications SELECT * FROM source.file_classifications"),
        ("processed_db_integrity",
         "INSERT OR IGNORE INTO processed_db_integrity SELECT * FROM source.processed_db_integrity"),
        ("processed_db_metrics",
         "INSERT OR IGNORE INTO processed_db_metrics SELECT * FROM source.processed_db_metrics"),
        ("ui_state",
         "INSERT OR IGNORE INTO ui_state SELECT * FROM source.ui_state"),
    ];

    // Track temp directories for WAL-replayed source DBs (kept alive until merge completes)
    let mut _temp_dirs: Vec<tempfile::TempDir> = Vec::new();

    for source_path in source_db_paths {
        if !source_path.exists() {
            info!("Skipping non-existent source DB: {:?}", source_path);
            continue;
        }

        // If source has an active WAL file, copy to temp dir first so ATTACH can read WAL data.
        // ATTACH inherits the main connection's mode; even though the target is read-write,
        // the attached database may fail to read WAL data if the WAL file can't be replayed
        // (e.g., permissions, external volume, or stale SHM).
        let wal_path = source_path.with_extension("ffxdb-wal");
        let has_active_wal =
            wal_path.exists() && wal_path.metadata().map(|m| m.len() > 0).unwrap_or(false);

        let attach_path = if has_active_wal {
            info!(
                "Source DB has active WAL, copying to temp for merge: {}",
                source_path.display()
            );
            match tempfile::tempdir() {
                Ok(temp_dir) => {
                    let temp_db = temp_dir.path().join("source_merge.ffxdb");
                    let temp_wal = temp_dir.path().join("source_merge.ffxdb-wal");
                    let temp_shm = temp_dir.path().join("source_merge.ffxdb-shm");

                    if let Err(e) = std::fs::copy(source_path, &temp_db) {
                        warn!(
                            "Failed to copy source DB to temp: {} — {}",
                            source_path.display(),
                            e
                        );
                        continue;
                    }
                    if let Err(e) = std::fs::copy(&wal_path, &temp_wal) {
                        warn!(
                            "Failed to copy source WAL to temp: {} — {}",
                            wal_path.display(),
                            e
                        );
                        continue;
                    }
                    let shm_path = source_path.with_extension("ffxdb-shm");
                    if shm_path.exists() {
                        let _ = std::fs::copy(&shm_path, &temp_shm);
                    }

                    // Open the temp copy read-write to force WAL checkpoint, then close
                    {
                        let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                            | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
                        if let Ok(temp_conn) =
                            rusqlite::Connection::open_with_flags(&temp_db, flags)
                        {
                            let _ = temp_conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
                        }
                    }

                    let path = temp_db.to_string_lossy().to_string();
                    _temp_dirs.push(temp_dir); // Keep temp dir alive
                    path
                }
                Err(e) => {
                    warn!("Failed to create temp dir for WAL merge: {}", e);
                    source_path.to_string_lossy().to_string()
                }
            }
        } else {
            source_path.to_string_lossy().to_string()
        };

        info!("Merging source DB: {}", attach_path);

        // Attach source database
        conn.execute(
            &format!(
                "ATTACH DATABASE '{}' AS source",
                attach_path.replace('\'', "''")
            ),
            [],
        )
        .map_err(|e| format!("Failed to attach {}: {}", attach_path, e))?;

        // Merge each table
        for (table_name, insert_sql) in &merge_tables {
            // Check if table exists in source
            let table_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM source.sqlite_master WHERE type='table' AND name=?1",
                    [table_name],
                    |row| row.get::<_, i64>(0),
                )
                .map(|c| c > 0)
                .unwrap_or(false);

            if !table_exists {
                continue;
            }

            // Count rows before merge for stats
            let count_before: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);

            match conn.execute(insert_sql, []) {
                Ok(inserted) => {
                    if inserted > 0 {
                        info!("  {} → {} rows merged", table_name, inserted);
                        total_stats.ffxdb_tables_merged += 1;
                        match *table_name {
                            "users" => total_stats.users_merged += inserted,
                            "sessions" => total_stats.sessions_merged += inserted,
                            "activity_log" => total_stats.activity_entries_merged += inserted,
                            "evidence_files" => total_stats.evidence_files_merged += inserted,
                            "hashes" => total_stats.hashes_merged += inserted,
                            "bookmarks" => total_stats.bookmarks_merged += inserted,
                            "notes" => total_stats.notes_merged += inserted,
                            "reports" => total_stats.reports_merged += inserted,
                            "tags" => total_stats.tags_merged += inserted,
                            "saved_searches" | "recent_searches" => {
                                total_stats.searches_merged += inserted
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    warn!("  {} → merge error (continuing): {}", table_name, e);
                }
            }

            let _ = count_before; // Used for logging context
        }

        // Detach source
        conn.execute("DETACH DATABASE source", [])
            .map_err(|e| format!("Failed to detach {}: {}", attach_path, e))?;
    }

    // Rebuild FTS indexes
    rebuild_fts_indexes(&conn);

    info!(
        "Database merge complete: {} sources processed",
        source_db_paths.len()
    );
    Ok(total_stats)
}

/// Rebuild FTS (Full-Text Search) indexes after merge
fn rebuild_fts_indexes(conn: &rusqlite::Connection) {
    let fts_rebuild_cmds = [
        "INSERT INTO fts_activity_log(fts_activity_log) VALUES('rebuild')",
        "INSERT INTO fts_bookmarks(fts_bookmarks) VALUES('rebuild')",
        "INSERT INTO fts_notes(fts_notes) VALUES('rebuild')",
    ];

    for cmd in &fts_rebuild_cmds {
        match conn.execute(cmd, []) {
            Ok(_) => {}
            Err(e) => warn!("FTS rebuild warning (non-fatal): {}", e),
        }
    }
}

// =============================================================================
// Rebase Paths
// =============================================================================

/// Rebase all paths in a project from old_base to new_base.
/// Used when relocating a project to a new directory.
pub fn rebase_paths(project: &mut FFXProject, old_base: &Path, new_base: &Path) {
    info!("Rebasing paths: {:?} → {:?}", old_base, new_base);

    let rebase = |path: &str| -> String {
        let p = Path::new(path);
        if let Ok(relative) = p.strip_prefix(old_base) {
            new_base.join(relative).to_string_lossy().to_string()
        } else {
            // Path doesn't start with old_base, leave as-is
            path.to_string()
        }
    };

    // Root path
    project.root_path = rebase(&project.root_path);

    // Locations
    if let Some(ref mut loc) = project.locations {
        loc.project_root = rebase(&loc.project_root);
        loc.evidence_path = rebase(&loc.evidence_path);
        loc.processed_db_path = rebase(&loc.processed_db_path);
        if let Some(ref path) = loc.case_documents_path {
            loc.case_documents_path = Some(rebase(path));
        }
    }

    // Tabs
    for tab in &mut project.tabs {
        tab.file_path = rebase(&tab.file_path);
        if let Some(ref path) = tab.document_path {
            tab.document_path = Some(rebase(path));
        }
        if let Some(ref path) = tab.entry_container_path {
            tab.entry_container_path = Some(rebase(path));
        }
        if let Some(ref path) = tab.processed_db_path {
            tab.processed_db_path = Some(rebase(path));
        }
    }

    // Hash history
    let mut new_hh: HashMap<String, Vec<ProjectFileHash>> = HashMap::new();
    for (path, hashes) in project.hash_history.files.drain() {
        new_hh.insert(rebase(&path), hashes);
    }
    project.hash_history.files = new_hh;

    // Evidence cache
    if let Some(ref mut cache) = project.evidence_cache {
        for f in &mut cache.discovered_files {
            f.path = rebase(&f.path);
        }
        let mut new_fi: HashMap<String, serde_json::Value> = HashMap::new();
        for (path, info) in cache.file_info.drain() {
            new_fi.insert(rebase(&path), info);
        }
        cache.file_info = new_fi;

        let mut new_ch: HashMap<String, CachedFileHash> = HashMap::new();
        for (path, hash) in cache.computed_hashes.drain() {
            new_ch.insert(rebase(&path), hash);
        }
        cache.computed_hashes = new_ch;
    }

    // Case documents cache
    if let Some(ref mut cache) = project.case_documents_cache {
        cache.search_path = rebase(&cache.search_path);
        for doc in &mut cache.documents {
            doc.path = rebase(&doc.path);
        }
    }

    // Processed databases
    project.processed_databases.loaded_paths = project
        .processed_databases
        .loaded_paths
        .iter()
        .map(|p| rebase(p))
        .collect();
    if let Some(ref path) = project.processed_databases.selected_path {
        project.processed_databases.selected_path = Some(rebase(path));
    }
    if let Some(ref mut meta) = project.processed_databases.cached_metadata {
        let mut new: HashMap<String, serde_json::Value> = HashMap::new();
        for (k, v) in meta.drain() {
            new.insert(rebase(&k), v);
        }
        *meta = new;
    }
    // Rebase integrity keys
    let mut new_integrity: HashMap<String, ProcessedDbIntegrity> = HashMap::new();
    for (k, mut v) in project.processed_databases.integrity.drain() {
        v.path = rebase(&v.path);
        new_integrity.insert(rebase(&k), v);
    }
    project.processed_databases.integrity = new_integrity;

    // Activity log file_paths
    for entry in &mut project.activity_log {
        if let Some(ref path) = entry.file_path {
            entry.file_path = Some(rebase(path));
        }
    }

    // Open directories
    for dir in &mut project.open_directories {
        dir.path = rebase(&dir.path);
    }

    // Recent directories
    for dir in &mut project.recent_directories {
        dir.path = rebase(&dir.path);
    }

    // File selection
    project.file_selection.selected_paths = project
        .file_selection
        .selected_paths
        .iter()
        .map(|p| rebase(p))
        .collect();
    if let Some(ref path) = project.file_selection.active_path {
        project.file_selection.active_path = Some(rebase(path));
    }

    // Bookmarks
    for b in &mut project.bookmarks {
        b.target_path = rebase(&b.target_path);
    }

    // Notes
    for n in &mut project.notes {
        if let Some(ref path) = n.target_path {
            n.target_path = Some(rebase(path));
        }
    }

    // Reports
    for r in &mut project.reports {
        if let Some(ref path) = r.output_path {
            r.output_path = Some(rebase(path));
        }
    }

    // UI state - case documents path
    if let Some(ref path) = project.ui_state.case_documents_path {
        project.ui_state.case_documents_path = Some(rebase(path));
    }

    // Active tab path
    if let Some(ref path) = project.active_tab_path {
        project.active_tab_path = Some(rebase(path));
    }
}

// =============================================================================
// Full Merge Pipeline
// =============================================================================

/// Execute the full merge pipeline:
/// 1. Load all .cffx files
/// 2. Merge .cffx data into one FFXProject
/// 3. Save merged .cffx to output_path
/// 4. Create/open the merged .ffxdb
/// 5. Merge all source .ffxdb files into it
/// 6. Optionally rebase paths
/// 7. Build merge provenance records
pub fn execute_merge(
    cffx_paths: &[String],
    output_path: &str,
    merged_name: &str,
    new_root: Option<&str>,
    owner_assignments: Option<&[MergeSourceAssignment]>,
) -> MergeResult {
    info!(
        "Starting merge: {} projects → {}",
        cffx_paths.len(),
        output_path
    );

    let now = chrono::Utc::now().to_rfc3339();

    // 1. Load all projects
    let mut projects: Vec<FFXProject> = Vec::new();
    let mut source_db_paths: Vec<PathBuf> = Vec::new();

    for path_str in cffx_paths {
        let cffx_path = Path::new(path_str);
        let project_dir = cffx_path.parent().unwrap_or(Path::new("."));

        match std::fs::read_to_string(cffx_path) {
            Ok(json) => match serde_json::from_str::<FFXProject>(&json) {
                Ok(mut project) => {
                    // Resolve relative paths to absolute for merging
                    make_paths_absolute(&mut project, project_dir);
                    projects.push(project);

                    // Check for companion .ffxdb
                    let ffxdb = cffx_path.with_extension("ffxdb");
                    if ffxdb.exists() {
                        source_db_paths.push(ffxdb);
                    }
                }
                Err(e) => {
                    return MergeResult {
                        success: false,
                        cffx_path: None,
                        ffxdb_path: None,
                        error: Some(format!("Failed to parse {}: {}", path_str, e)),
                        stats: None,
                        sources: None,
                    };
                }
            },
            Err(e) => {
                return MergeResult {
                    success: false,
                    cffx_path: None,
                    ffxdb_path: None,
                    error: Some(format!("Failed to read {}: {}", path_str, e)),
                    stats: None,
                    sources: None,
                };
            }
        }
    }

    if projects.is_empty() {
        return MergeResult {
            success: false,
            cffx_path: None,
            ffxdb_path: None,
            error: Some("No valid projects to merge".to_string()),
            stats: None,
            sources: None,
        };
    }

    // 2. Determine root path
    let root_path = new_root
        .map(|r| r.to_string())
        .unwrap_or_else(|| projects[0].root_path.clone());

    // 3. Merge .cffx data
    let mut merged = merge_projects(&projects, merged_name, &root_path);

    // 3b. Build merge provenance records
    let owner_map: std::collections::HashMap<String, String> = owner_assignments
        .map(|assignments| {
            assignments
                .iter()
                .map(|a| (a.cffx_path.clone(), a.owner_name.clone()))
                .collect()
        })
        .unwrap_or_default();

    let merge_sources: Vec<MergeSource> = projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
            // Get owner: user-assigned > project.owner_name > first user > "Unknown"
            let assigned_owner = cffx_paths
                .get(i)
                .and_then(|path| owner_map.get(path))
                .cloned();
            let owner = assigned_owner
                .or_else(|| p.owner_name.clone())
                .or_else(|| p.users.first().map(|u| u.username.clone()))
                .unwrap_or_else(|| "Unknown".to_string());

            MergeSource {
                source_project_id: p.project_id.clone(),
                source_project_name: p.name.clone(),
                source_cffx_path: cffx_paths.get(i).cloned().unwrap_or_default(),
                owner_name: owner,
                merged_at: now.clone(),
                evidence_file_count: p
                    .evidence_cache
                    .as_ref()
                    .map(|c| c.discovered_files.len())
                    .unwrap_or(0),
                session_count: p.sessions.len(),
                activity_count: p.activity_log.len(),
                bookmark_count: p.bookmarks.len(),
                note_count: p.notes.len(),
            }
        })
        .collect();

    // Store provenance in the merged project
    merged.merge_sources = Some(merge_sources.clone());

    // 4. Rebase paths if relocating
    if let Some(new_root_path) = new_root {
        // Find common old root from all projects
        let old_roots: Vec<&str> = projects.iter().map(|p| p.root_path.as_str()).collect();
        if let Some(common_old) = find_common_parent(&old_roots) {
            let new_base = Path::new(new_root_path)
                .parent()
                .unwrap_or(Path::new(new_root_path));
            rebase_paths(&mut merged, Path::new(&common_old), new_base);
        }
    }

    // 5. Save merged .cffx
    let output = Path::new(output_path);
    let save_result = super::save_project(&merged, Some(output_path));
    if !save_result.success {
        return MergeResult {
            success: false,
            cffx_path: None,
            ffxdb_path: None,
            error: save_result.error,
            stats: None,
            sources: None,
        };
    }

    // 6. Create and merge .ffxdb
    let target_ffxdb = output.with_extension("ffxdb");
    let mut db_stats = MergeStats {
        projects_merged: projects.len(),
        users_merged: 0,
        sessions_merged: 0,
        activity_entries_merged: 0,
        evidence_files_merged: 0,
        hashes_merged: 0,
        bookmarks_merged: 0,
        notes_merged: 0,
        tabs_merged: 0,
        reports_merged: 0,
        tags_merged: 0,
        searches_merged: 0,
        ffxdb_tables_merged: 0,
    };

    if !source_db_paths.is_empty() {
        // Create the target .ffxdb with schema by opening via ProjectDatabase
        {
            use crate::project_db::ProjectDatabase;
            match ProjectDatabase::open(&target_ffxdb) {
                Ok(db) => {
                    // Migrate .cffx data into the new database
                    if let Err(e) = db.migrate_from_project(&merged) {
                        warn!("Migration from merged .cffx had errors: {}", e);
                    }
                    // db drops here, closing the connection
                }
                Err(e) => {
                    return MergeResult {
                        success: false,
                        cffx_path: save_result.path,
                        ffxdb_path: None,
                        error: Some(format!("Failed to create target .ffxdb: {}", e)),
                        stats: None,
                        sources: None,
                    };
                }
            }
        }

        // Now merge source databases into target
        match merge_databases(&target_ffxdb, &source_db_paths) {
            Ok(stats) => {
                db_stats = stats;
            }
            Err(e) => {
                warn!("Database merge had errors: {}", e);
                // Continue — partial merge is better than no merge
            }
        }
    } else {
        // No source .ffxdb files — just create from merged .cffx
        use crate::project_db::ProjectDatabase;
        match ProjectDatabase::open(&target_ffxdb) {
            Ok(db) => {
                if let Err(e) = db.migrate_from_project(&merged) {
                    warn!("Migration from merged .cffx had errors: {}", e);
                }
            }
            Err(e) => {
                warn!("Failed to create .ffxdb: {}", e);
            }
        }
    }

    info!(
        "Merge pipeline complete: {} → {}",
        cffx_paths.len(),
        output_path
    );

    MergeResult {
        success: true,
        cffx_path: save_result.path,
        ffxdb_path: Some(target_ffxdb.to_string_lossy().to_string()),
        error: None,
        stats: Some(db_stats),
        sources: Some(merge_sources),
    }
}

/// Find the common parent directory from a list of paths.
fn find_common_parent(paths: &[&str]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }

    let first = Path::new(paths[0]);
    let mut common = first.parent().unwrap_or(first).to_path_buf();

    for path in &paths[1..] {
        let p = Path::new(path).parent().unwrap_or(Path::new(path));
        while !p.starts_with(&common) {
            if let Some(parent) = common.parent() {
                common = parent.to_path_buf();
            } else {
                return None;
            }
        }
    }

    Some(common.to_string_lossy().to_string())
}
