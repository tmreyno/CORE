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
mod bookmarks;
mod collections;
mod evidence;
mod forensic;
mod processed;
mod search;
mod utilities;
mod workflow;

pub use activity::*;
pub use bookmarks::*;
pub use collections::*;
pub use evidence::*;
pub use forensic::*;
pub use processed::*;
pub use search::*;
pub use utilities::*;
pub use workflow::*;

use crate::project_db::ProjectDatabase;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
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
pub(super) fn with_project_db<F, T>(window_label: &str, f: F) -> Result<T, String>
where
    F: FnOnce(&ProjectDatabase) -> rusqlite::Result<T>,
{
    let guard = PROJECT_DBS.lock();
    match guard.get(window_label) {
        Some(db) => f(db).map_err(|e| format!("Project DB error: {}", e)),
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
pub fn project_db_open(window: tauri::Window, cffx_path: String) -> Result<String, String> {
    let label = window.label().to_string();
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
    info!(window = %label, "Project DB opened: {} (new: {})", db_path_str, is_new);

    // Store keyed by the calling window's label
    let mut guard = PROJECT_DBS.lock();
    guard.insert(label, db);

    Ok(db_path_str)
}

/// Close the project database for the calling window.
/// Performs a WAL checkpoint before closing to ensure all data is flushed
/// to the main database file (prevents data-only-in-WAL on external volumes).
#[tauri::command]
pub fn project_db_close(window: tauri::Window) -> Result<(), String> {
    let label = window.label();
    let mut guard = PROJECT_DBS.lock();
    if let Some(db) = guard.get(label) {
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
    }
    if guard.remove(label).is_some() {
        info!(window = %label, "Project DB closed");
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
    with_project_db(window.label(), |db| Ok(db.path().to_string_lossy().to_string()))
}

/// Get project database statistics for the calling window.
#[tauri::command]
pub fn project_db_get_stats(window: tauri::Window) -> Result<crate::project_db::ProjectDbStats, String> {
    with_project_db(window.label(), |db| db.get_stats())
}
