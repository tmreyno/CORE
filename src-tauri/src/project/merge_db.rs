// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Database merge operations — merges multiple .ffxdb files via SQLite ATTACH.

use super::merge_types::MergeStats;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

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
        ("evidence_data_alternatives",
         "INSERT OR IGNORE INTO evidence_data_alternatives SELECT * FROM source.evidence_data_alternatives"),
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
