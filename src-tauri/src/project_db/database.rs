// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Core `ProjectDatabase` struct definition, constructor, and path helpers.

use parking_lot::Mutex;
use rusqlite::{Connection, Result as SqlResult};
use std::path::{Path, PathBuf};
use tracing::info;

/// Per-project SQLite database for forensic activity persistence.
///
/// Unlike the global `ffx.db` (which tracks app-level sessions), this database
/// lives alongside the `.cffx` manifest in the case folder and is portable.
pub struct ProjectDatabase {
    pub(crate) conn: Mutex<Connection>,
    pub(crate) path: PathBuf,
}

impl ProjectDatabase {
    /// Open or create a project database at the given path.
    ///
    /// Creates the `.ffxdb` file and initializes the schema if it doesn't exist.
    /// Runs migrations if the schema version is older than current.
    pub fn open(db_path: &Path) -> SqlResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self {
            conn: Mutex::new(conn),
            path: db_path.to_path_buf(),
        };

        db.init_schema()?;
        db.check_migrations()?;

        info!("Project database opened: {:?}", db_path);
        Ok(db)
    }

    /// Derive the `.ffxdb` path from a `.cffx` project file path.
    ///
    /// The database sits alongside the project file in the same directory.
    /// Example: `/case/project.cffx` → `/case/project.ffxdb`
    pub fn db_path_for_project(cffx_path: &Path) -> PathBuf {
        cffx_path.with_extension("ffxdb")
    }

    /// Get the file path of this database
    pub fn path(&self) -> &Path {
        &self.path
    }
}
