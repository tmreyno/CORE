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

    // If this is a brand-new .ffxdb, migrate data from the same resolved
    // project state used by the UI so evidence IDs and paths stay aligned.
    if is_new {
        match load_resolved_project_for_db_migration(&cffx) {
            Ok(Some(project)) => {
                if let Err(e) = db.migrate_from_project(&project) {
                    warn!("Migration from .cffx had errors: {}", e);
                }
            }
            Ok(None) => {}
            Err(e) => warn!("Could not prepare .cffx migration data: {}", e),
        }
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

fn load_resolved_project_for_db_migration(
    cffx: &Path,
) -> Result<Option<crate::project::FFXProject>, String> {
    let path = cffx.to_string_lossy();
    let result = crate::project::load_project(path.as_ref());

    if result.success {
        Ok(result.project)
    } else {
        Err(result
            .error
            .unwrap_or_else(|| "project_load failed without an error message".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{CachedDiscoveredFile, EvidenceCache, FFXProject};

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

        let resolved = load_resolved_project_for_db_migration(&project_path)
            .unwrap()
            .unwrap();
        let resolved_path = &resolved.evidence_cache.as_ref().unwrap().discovered_files[0].path;

        let expected_path = evidence_path.canonicalize().unwrap();
        assert_eq!(Path::new(resolved_path), expected_path.as_path());
        assert!(Path::new(resolved_path).is_absolute());
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
