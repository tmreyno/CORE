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

mod activity;
mod artifacts;
mod bookmarks;
mod collections;
mod evidence;
mod forensic;
mod processed;
mod search;
mod source_analysis;
mod utilities;
mod workflow;

pub use activity::*;
pub use artifacts::*;
pub use bookmarks::*;
pub use collections::*;
pub use evidence::*;
pub use forensic::*;
pub use processed::*;
pub use search::*;
pub use source_analysis::*;
pub use utilities::*;
pub use workflow::*;

use crate::project_db::ProjectDatabase;
use parking_lot::Mutex;
use rusqlite::params;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tracing::{info, warn};

// =============================================================================
// Per-Window Project Database State
// =============================================================================

/// Project databases keyed by window label.
///
/// Each Tauri window can have its own project open independently.
/// When a command is invoked, Tauri automatically injects the calling
/// window — its label is used to look up the correct database.
static PROJECT_DBS: LazyLock<Mutex<HashMap<String, ProjectDatabase>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Helper: execute a closure with the project database for a specific window.
/// Accessible to sibling command modules within this directory.
pub(crate) fn with_project_db<F, T>(window_label: &str, f: F) -> Result<T, String>
where
    F: FnOnce(&ProjectDatabase) -> rusqlite::Result<T>,
{
    let guard = PROJECT_DBS.lock();
    match guard.get(window_label) {
        Some(db) => f(db).map_err(|e| format!("Project DB error: {}", e)),
        None => Err("No project database is open. Open or create a project first.".to_string()),
    }
}

/// Helper: execute a closure with the project database for a specific window
/// when the closure needs to return a custom `Result<T, String>`.
pub(crate) fn with_project_db_result<F, T>(window_label: &str, f: F) -> Result<T, String>
where
    F: FnOnce(&ProjectDatabase) -> Result<T, String>,
{
    let guard = PROJECT_DBS.lock();
    match guard.get(window_label) {
        Some(db) => f(db),
        None => Err("No project database is open. Open or create a project first.".to_string()),
    }
}

// =============================================================================
// Lifecycle Commands
// =============================================================================

/// Open or create a project database for a .cffx project file.
/// If the .ffxdb doesn't exist, it will be created and data migrated from the .cffx.
///
/// The database is associated with the calling window's label so each window
/// can have its own project open independently.
#[tauri::command]
pub async fn project_db_open(window: tauri::Window, cffx_path: String) -> Result<String, String> {
    let label = window.label().to_string();
    tauri::async_runtime::spawn_blocking(move || project_db_open_blocking(label, cffx_path))
        .await
        .map_err(|e| format!("Project DB open task failed: {}", e))?
}

fn project_db_open_blocking(label: String, cffx_path: String) -> Result<String, String> {
    let t0 = std::time::Instant::now();
    let cffx = PathBuf::from(&cffx_path);
    let db_path = ProjectDatabase::db_path_for_project(&cffx);

    let is_new = !db_path.exists();
    info!(window = %label, "project_db_open: starting (is_new: {}, path: {})", is_new, db_path.display());

    let db =
        ProjectDatabase::open(&db_path).map_err(|e| format!("Failed to open project DB: {}", e))?;
    info!(window = %label, "project_db_open: ProjectDatabase::open took {:?}", t0.elapsed());

    // Use the same resolved project state as the UI so .ffxdb evidence IDs
    // and paths stay aligned with project_load().
    match load_project_states_for_db_open(&cffx) {
        Ok(Some((raw_project, resolved_project))) => {
            if is_new {
                if let Err(e) = db.migrate_from_project(&resolved_project) {
                    warn!("Migration from .cffx had errors: {}", e);
                }
            } else if let Err(e) =
                repair_existing_project_db_paths(&db, &raw_project, &resolved_project)
            {
                warn!("Existing .ffxdb path repair had errors: {}", e);
            }
        }
        Ok(None) => {}
        Err(e) => warn!("Could not prepare .cffx migration data: {}", e),
    }

    let db_path_str = db_path.to_string_lossy().to_string();
    info!(window = %label, "Project DB opened: {} (new: {})", db_path_str, is_new);

    // Start project-scoped audit logging alongside the project files
    if let Some(project_dir) = db_path.parent() {
        crate::logging::set_project_log_dir(project_dir);
    }

    // Store keyed by the calling window's label
    let old_db = {
        let mut guard = PROJECT_DBS.lock();
        guard.insert(label.clone(), db)
    };
    if let Some(old_db) = old_db {
        match old_db.wal_checkpoint() {
            Ok((log_size, frames)) => {
                info!(
                    window = %label,
                    "Previous project DB checkpointed before replacement: {} log pages, {} frames checkpointed",
                    log_size,
                    frames
                );
            }
            Err(e) => {
                warn!(
                    window = %label,
                    "Previous project DB checkpoint before replacement failed (non-fatal): {}",
                    e
                );
            }
        }
    }

    Ok(db_path_str)
}

fn load_project_states_for_db_open(
    cffx: &Path,
) -> Result<Option<(crate::project::FFXProject, crate::project::FFXProject)>, String> {
    let content = crate::project::read_project_json_with_limit(cffx, "project file")?;
    let raw_project = serde_json::from_str::<crate::project::FFXProject>(&content)
        .map_err(|e| format!("Failed to parse project file: {e}"))?;
    let path = cffx.to_string_lossy();
    let result = crate::project::load_project(path.as_ref());

    if result.success {
        Ok(result
            .project
            .map(|resolved_project| (raw_project, resolved_project)))
    } else {
        Err(result
            .error
            .unwrap_or_else(|| "project_load failed without an error message".to_string()))
    }
}

fn repair_existing_project_db_paths(
    db: &ProjectDatabase,
    raw_project: &crate::project::FFXProject,
    resolved_project: &crate::project::FFXProject,
) -> rusqlite::Result<usize> {
    let Some(raw_cache) = raw_project.evidence_cache.as_ref() else {
        return Ok(0);
    };
    let Some(resolved_cache) = resolved_project.evidence_cache.as_ref() else {
        return Ok(0);
    };

    let conn = db.conn.lock();
    conn.execute_batch("BEGIN IMMEDIATE")?;

    let repair_result = (|| -> rusqlite::Result<usize> {
        let mut repaired = 0usize;

        for (raw_file, resolved_file) in raw_cache
            .discovered_files
            .iter()
            .zip(resolved_cache.discovered_files.iter())
        {
            if raw_file.path == resolved_file.path {
                continue;
            }

            conn.execute(
                "INSERT INTO evidence_files (id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(path) DO UPDATE SET
                filename = excluded.filename,
                container_type = excluded.container_type,
                total_size = excluded.total_size,
                segment_count = excluded.segment_count,
                created = COALESCE(excluded.created, evidence_files.created),
                modified = COALESCE(excluded.modified, evidence_files.modified)",
                params![
                    resolved_file.path,
                    resolved_file.path,
                    resolved_file.filename,
                    resolved_file.container_type,
                    resolved_file.size as i64,
                    resolved_file.segment_count as i64,
                    resolved_cache.cached_at,
                    resolved_file.created,
                    resolved_file.modified,
                ],
            )?;

            let legacy_ids =
                legacy_evidence_ids_for_path(&conn, &raw_file.path, &resolved_file.path)?;
            let old_id = &raw_file.path;
            let new_id = &resolved_file.path;
            for legacy_id in &legacy_ids {
                conn.execute(
                    "UPDATE hashes SET file_id = ?2 WHERE file_id = ?1",
                    params![legacy_id, new_id],
                )?;
                conn.execute(
                    "UPDATE artifacts SET evidence_file_id = ?2 WHERE evidence_file_id = ?1",
                    params![legacy_id, new_id],
                )?;
                conn.execute(
                    "UPDATE source_analyses SET evidence_file_id = ?2 WHERE evidence_file_id = ?1",
                    params![legacy_id, new_id],
                )?;
                conn.execute(
                    "UPDATE coc_items SET evidence_file_id = ?2 WHERE evidence_file_id = ?1",
                    params![legacy_id, new_id],
                )?;
                conn.execute(
                    "UPDATE collected_items SET evidence_file_id = ?2 WHERE evidence_file_id = ?1",
                    params![legacy_id, new_id],
                )?;
                conn.execute(
                    "UPDATE evidence_data_alternatives SET evidence_file_id = ?2 WHERE evidence_file_id = ?1",
                    params![legacy_id, new_id],
                )?;
            }
            conn.execute(
                "UPDATE hashes SET source_id = ?2 WHERE source_id = ?1",
                params![old_id, new_id],
            )?;
            conn.execute(
                "UPDATE artifacts SET source_id = ?2 WHERE source_id = ?1",
                params![old_id, new_id],
            )?;
            conn.execute(
                "UPDATE source_analyses SET source_id = ?2 WHERE source_id = ?1",
                params![old_id, new_id],
            )?;
            conn.execute(
                "UPDATE collected_items SET source_id = ?2 WHERE source_id = ?1",
                params![old_id, new_id],
            )?;
            for legacy_id in legacy_ids {
                conn.execute(
                    "DELETE FROM evidence_files WHERE id = ?1 AND id <> ?2",
                    params![legacy_id, new_id],
                )?;
            }

            repaired += 1;
        }

        Ok(repaired)
    })();

    match repair_result {
        Ok(repaired) => {
            conn.execute_batch("COMMIT")?;
            Ok(repaired)
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(err)
        }
    }
}

fn legacy_evidence_ids_for_path(
    conn: &rusqlite::Connection,
    raw_path: &str,
    resolved_path: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut ids = Vec::new();
    let mut stmt = conn.prepare("SELECT id FROM evidence_files WHERE path = ?1 OR id = ?1")?;
    let rows = stmt.query_map(params![raw_path], |row| row.get::<_, String>(0))?;
    for row in rows {
        let id = row?;
        if id != resolved_path && !ids.contains(&id) {
            ids.push(id);
        }
    }
    if !ids.iter().any(|id| id == raw_path) {
        ids.push(raw_path.to_string());
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{CachedDiscoveredFile, EvidenceCache, FFXProject};
    use crate::project_db::{DbEvidenceFile, DbProjectHash};

    #[test]
    fn db_migration_project_loader_resolves_relative_evidence_paths() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = temp_dir.path().join("case.cffx");
        let evidence_path = temp_dir.path().join("1.Evidence").join("drive.E01");
        std::fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        std::fs::write(&evidence_path, b"ewf").unwrap();

        let mut project = FFXProject::new(".");
        project.name = "case".to_string();
        project.evidence_cache = Some(EvidenceCache {
            discovered_files: vec![CachedDiscoveredFile {
                path: "./1.Evidence/drive.E01".to_string(),
                filename: "drive.E01".to_string(),
                container_type: "EnCase (E01)".to_string(),
                size: 3,
                segment_count: 1,
                created: None,
                modified: None,
            }],
            cached_at: "2026-07-05T00:00:00Z".to_string(),
            valid: true,
            ..EvidenceCache::default()
        });
        std::fs::write(&project_path, serde_json::to_string(&project).unwrap()).unwrap();

        let (_, resolved) = load_project_states_for_db_open(&project_path)
            .unwrap()
            .unwrap();
        let resolved_path = &resolved.evidence_cache.as_ref().unwrap().discovered_files[0].path;

        let expected_path = evidence_path.canonicalize().unwrap();
        assert_eq!(Path::new(resolved_path), expected_path.as_path());
        assert!(Path::new(resolved_path).is_absolute());
    }

    #[test]
    fn existing_db_path_repair_rekeys_evidence_and_hash_records() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("case.ffxdb");
        let db = ProjectDatabase::open(&db_path).unwrap();

        let relative_path = "./1.Evidence/drive.E01";
        let absolute_path = temp_dir
            .path()
            .join("1.Evidence")
            .join("drive.E01")
            .to_string_lossy()
            .to_string();

        let legacy_generated_id = "ev_1Evidencedrive";
        db.upsert_evidence_file(&DbEvidenceFile {
            id: legacy_generated_id.to_string(),
            path: relative_path.to_string(),
            filename: "drive.E01".to_string(),
            container_type: "EnCase (E01)".to_string(),
            total_size: 3,
            segment_count: 1,
            discovered_at: "2026-07-05T00:00:00Z".to_string(),
            created: None,
            modified: None,
        })
        .unwrap();
        db.insert_hash(&DbProjectHash {
            id: "hash-1".to_string(),
            file_id: legacy_generated_id.to_string(),
            source_id: Some(relative_path.to_string()),
            source_ref_json: None,
            algorithm: "MD5".to_string(),
            hash_value: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
            computed_at: "2026-07-05T00:00:00Z".to_string(),
            segment_index: None,
            segment_name: None,
            source: "computed".to_string(),
        })
        .unwrap();

        let mut raw_project = FFXProject::new(".");
        raw_project.evidence_cache = Some(EvidenceCache {
            discovered_files: vec![CachedDiscoveredFile {
                path: relative_path.to_string(),
                filename: "drive.E01".to_string(),
                container_type: "EnCase (E01)".to_string(),
                size: 3,
                segment_count: 1,
                created: None,
                modified: None,
            }],
            cached_at: "2026-07-05T00:00:00Z".to_string(),
            valid: true,
            ..EvidenceCache::default()
        });
        let mut resolved_project = raw_project.clone();
        resolved_project
            .evidence_cache
            .as_mut()
            .unwrap()
            .discovered_files[0]
            .path = absolute_path.clone();

        assert_eq!(
            repair_existing_project_db_paths(&db, &raw_project, &resolved_project).unwrap(),
            1
        );
        assert!(db
            .get_evidence_file_by_path(relative_path)
            .unwrap()
            .is_none());
        let stale_count: i64 = db
            .conn
            .lock()
            .query_row(
                "SELECT COUNT(*) FROM evidence_files WHERE id = ?1",
                params![legacy_generated_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stale_count, 0);
        assert!(db
            .get_evidence_file_by_path(&absolute_path)
            .unwrap()
            .is_some());
        assert_eq!(
            db.lookup_hash_by_path(&absolute_path, "MD5").unwrap(),
            Some((
                "d41d8cd98f00b204e9800998ecf8427e".to_string(),
                "computed".to_string()
            ))
        );
    }
}

/// Close the project database for the calling window.
/// Performs a WAL checkpoint before closing to ensure all data is flushed
/// to the main database file (prevents data-only-in-WAL on external volumes).
#[tauri::command]
pub async fn project_db_close(window: tauri::Window) -> Result<(), String> {
    let label = window.label().to_string();
    tauri::async_runtime::spawn_blocking(move || project_db_close_blocking(label))
        .await
        .map_err(|e| format!("Project DB close task failed: {}", e))?
}

fn project_db_close_blocking(label: String) -> Result<(), String> {
    let db = {
        let mut guard = PROJECT_DBS.lock();
        guard.remove(&label)
    };
    if let Some(db) = db {
        // Checkpoint WAL before closing — best-effort, don't fail the close
        match db.wal_checkpoint() {
            Ok((log_size, frames)) => {
                info!(
                    "WAL checkpoint on close: {} log pages, {} frames checkpointed",
                    log_size, frames
                );
            }
            Err(e) => {
                warn!("WAL checkpoint on close failed (non-fatal): {}", e);
            }
        }
        info!(window = %label, "Project DB closed");
        // Stop project-scoped audit logging
        crate::logging::clear_project_log();
    }
    Ok(())
}

/// Check if the calling window has a project database open.
#[tauri::command]
pub fn project_db_is_open(window: tauri::Window) -> bool {
    let guard = PROJECT_DBS.lock();
    guard.contains_key(window.label())
}

/// Get the file path of the calling window's project database.
#[tauri::command]
pub fn project_db_path(window: tauri::Window) -> Result<String, String> {
    with_project_db(window.label(), |db| {
        Ok(db.path().to_string_lossy().to_string())
    })
}

/// Get project database statistics for the calling window.
#[tauri::command]
pub async fn project_db_get_stats(
    window: tauri::Window,
) -> Result<crate::project_db::ProjectDbStats, String> {
    let label = window.label().to_string();
    tauri::async_runtime::spawn_blocking(move || with_project_db(&label, |db| db.get_stats()))
        .await
        .map_err(|e| format!("Project DB stats task failed: {}", e))?
}

// =============================================================================
// Window Lifecycle Cleanup
// =============================================================================

/// Clean up the project database for a destroyed window.
///
/// Called from `on_window_event(WindowEvent::Destroyed)` in `lib.rs`.
/// This is a safety net — normally the frontend calls `project_db_close`
/// before the window closes, but if the window is force-closed or crashes,
/// this ensures the database connection is dropped and WAL is checkpointed.
pub fn cleanup_window_project_db(label: &str) {
    let db = {
        let mut guard = PROJECT_DBS.lock();
        guard.remove(label)
    };
    if let Some(db) = db {
        match db.wal_checkpoint() {
            Ok((log_size, frames)) => {
                info!(
                    "WAL checkpoint on window destroy: {} log pages, {} frames checkpointed",
                    log_size, frames
                );
            }
            Err(e) => {
                warn!("WAL checkpoint on window destroy failed (non-fatal): {}", e);
            }
        }
        info!(window = %label, "Project DB cleaned up on window destroy");
        // Stop project-scoped audit logging (safety net)
        crate::logging::clear_project_log();
    }
}
