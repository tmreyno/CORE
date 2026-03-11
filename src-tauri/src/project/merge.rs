// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project merge logic — combine multiple .cffx projects into one.

use super::merge_db::merge_databases;
use super::merge_query::*;
use super::merge_rebase::rebase_paths;
pub use super::merge_types::*;
use super::migration::make_paths_absolute;
use super::types::*;
use super::{FFXProject, APP_VERSION, PROJECT_VERSION};
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
        case_number: newest.case_number.clone(),
        case_name: newest.case_name.clone(),
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
    exclusions: Option<&MergeExclusions>,
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
        let default_exclusions = MergeExclusions::default();
        let eff_exclusions = exclusions.unwrap_or(&default_exclusions);
        match merge_databases(&target_ffxdb, &source_db_paths, eff_exclusions) {
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
