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

/// Helper: execute a closure with the active project database.
/// Accessible to sibling command modules within this directory.
pub(super) fn with_project_db<F, T>(f: F) -> Result<T, String>
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
/// Performs a WAL checkpoint before closing to ensure all data is flushed
/// to the main database file (prevents data-only-in-WAL on external volumes).
#[tauri::command]
pub fn project_db_close() -> Result<(), String> {
    let mut guard = get_project_db_lock().lock();
    if let Some(ref db) = *guard {
        // Checkpoint WAL before closing — best-effort, don't fail the close
        match db.wal_checkpoint() {
            Ok((log_size, frames)) => {
                info!("WAL checkpoint on close: {} log pages, {} frames checkpointed", log_size, frames);
            }
            Err(e) => {
                warn!("WAL checkpoint on close failed (non-fatal): {}", e);
            }
        }
    }
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

/// Get project database statistics.
#[tauri::command]
pub fn project_db_get_stats() -> Result<crate::project_db::ProjectDbStats, String> {
    with_project_db(|db| db.get_stats())
}
