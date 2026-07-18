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

const PROJECT_DB_WAL_AUTOCHECKPOINT_PAGES: i64 = 256;
const PROJECT_DB_WAL_SIZE_LIMIT_BYTES: i64 = 8 * 1024 * 1024;

/// Per-project SQLite database for forensic activity persistence.
///
/// Unlike the global `ffx.db` (which tracks app-level sessions), this database
/// lives alongside the `.cffx` manifest in the case folder and is portable.
pub struct ProjectDatabase {
    pub(crate) conn: Mutex<Connection>,
    pub(crate) path: PathBuf,
    pub(crate) last_wal_checkpoint: Mutex<Option<super::types::ProjectDbWalCheckpointStatus>>,
}

impl ProjectDatabase {
    /// Open or create a project database at the given path.
    ///
    /// Creates the `.ffxdb` file and initializes the schema if it doesn't exist.
    /// Runs migrations if the schema version is older than current.
    pub fn open(db_path: &Path) -> SqlResult<Self> {
        let t0 = std::time::Instant::now();

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| rusqlite::Error::InvalidPath(parent.to_path_buf()))?;
        }

        let conn = Connection::open(db_path)?;
        info!("  DB Connection::open took {:?}", t0.elapsed());

        // Keep WAL mode for concurrent reads, but checkpoint more aggressively
        // during active use and cap how much WAL SQLite retains after resets.
        conn.execute_batch(&format!(
            "PRAGMA journal_mode=WAL;\nPRAGMA foreign_keys=ON;\nPRAGMA wal_autocheckpoint={};\nPRAGMA journal_size_limit={};",
            PROJECT_DB_WAL_AUTOCHECKPOINT_PAGES,
            PROJECT_DB_WAL_SIZE_LIMIT_BYTES,
        ))?;
        info!(
            "  DB pragmas took {:?} total (wal_autocheckpoint={} pages, journal_size_limit={} bytes)",
            t0.elapsed(),
            PROJECT_DB_WAL_AUTOCHECKPOINT_PAGES,
            PROJECT_DB_WAL_SIZE_LIMIT_BYTES,
        );

        // If an existing WAL file is large, checkpoint it now.
        // This replays WAL pages into the main DB and truncates the WAL file,
        // reducing WAL index construction time on subsequent opens.
        let wal_path = db_path.with_extension("ffxdb-wal");
        if wal_path.exists() {
            if let Ok(meta) = std::fs::metadata(&wal_path) {
                if meta.len() > 32 * 1024 {
                    let wal_kb = meta.len() / 1024;
                    match conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);") {
                        Ok(()) => info!("  DB: checkpointed large WAL ({} KB)", wal_kb),
                        Err(e) => info!("  DB: WAL checkpoint skipped: {}", e),
                    }
                }
            }
        }

        let db = Self {
            conn: Mutex::new(conn),
            path: db_path.to_path_buf(),
            last_wal_checkpoint: Mutex::new(None),
        };

        // Skip the expensive init_schema() DDL batch for existing databases.
        // If schema_meta already has a version, all tables exist — running 70+
        // CREATE TABLE IF NOT EXISTS is wasted work (especially on slow I/O).
        let needs_init = {
            let c = db.conn.lock();
            c.query_row(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'",
                [],
                |row| row.get::<_, String>(0),
            )
            .is_err()
        };
        info!(
            "  DB schema check took {:?} total (needs_init: {})",
            t0.elapsed(),
            needs_init
        );

        if needs_init {
            db.init_schema()?;
            info!("  DB init_schema took {:?} total", t0.elapsed());
        }

        db.check_migrations()?;
        info!("  DB check_migrations took {:?} total", t0.elapsed());

        info!(
            "Project database opened: {:?} (total: {:?})",
            db_path,
            t0.elapsed()
        );
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
