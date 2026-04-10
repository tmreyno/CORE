// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Database utility operations: integrity check, WAL checkpoint, backup,
//! vacuum, and statistics.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::Result as SqlResult;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

#[derive(Clone, Copy)]
enum WalCheckpointMode {
    Passive,
    Truncate,
}

impl WalCheckpointMode {
    fn pragma_name(self) -> &'static str {
        match self {
            Self::Passive => "PASSIVE",
            Self::Truncate => "TRUNCATE",
        }
    }

    fn log_label(self) -> &'static str {
        match self {
            Self::Passive => "passive",
            Self::Truncate => "truncate",
        }
    }
}

impl ProjectDatabase {
    // ========================================================================
    // Database Utilities
    // ========================================================================

    /// Run a SQLite integrity check
    pub fn integrity_check(&self) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("PRAGMA integrity_check")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    /// Force a WAL checkpoint (flush WAL to main DB file)
    pub fn wal_checkpoint(&self) -> SqlResult<(i64, i64)> {
        self.wal_checkpoint_with_mode(WalCheckpointMode::Truncate)
    }

    /// Run a non-blocking WAL checkpoint suitable for active sessions.
    pub fn wal_checkpoint_passive(&self) -> SqlResult<(i64, i64)> {
        self.wal_checkpoint_with_mode(WalCheckpointMode::Passive)
    }

    fn wal_checkpoint_with_mode(&self, mode: WalCheckpointMode) -> SqlResult<(i64, i64)> {
        let conn = self.conn.lock();
        let (busy, log_size, frames_checkpointed): (i64, i64, i64) = conn.query_row(
            &format!("PRAGMA wal_checkpoint({})", mode.pragma_name()),
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        drop(conn);

        let blocked = busy > 0 || frames_checkpointed < log_size;
        let completed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0))
            .ok()
            .flatten()
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        *self.last_wal_checkpoint.lock() = Some(ProjectDbWalCheckpointStatus {
            mode: mode.log_label().to_string(),
            busy_readers: busy,
            log_pages: log_size,
            checkpointed_pages: frames_checkpointed,
            blocked,
            completed_at,
        });

        if blocked {
            warn!(
                db_path = %self.path.display(),
                mode = mode.log_label(),
                busy,
                log_pages = log_size,
                checkpointed_pages = frames_checkpointed,
                "WAL checkpoint could not fully truncate; active readers may still be pinning frames"
            );
        } else {
            info!(
                db_path = %self.path.display(),
                mode = mode.log_label(),
                log_pages = log_size,
                checkpointed_pages = frames_checkpointed,
                "WAL checkpoint completed"
            );
        }

        Ok((log_size, frames_checkpointed))
    }

    /// Create a backup copy of the database
    pub fn backup_to(&self, dest_path: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        let mut dest = rusqlite::Connection::open(dest_path)?;
        let backup = rusqlite::backup::Backup::new(&conn, &mut dest)?;
        backup.run_to_completion(100, std::time::Duration::from_millis(50), None)?;
        Ok(())
    }

    /// Vacuum the database to reclaim space
    pub fn vacuum(&self) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute_batch("VACUUM")?;
        Ok(())
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get project database statistics
    pub fn get_stats(&self) -> SqlResult<ProjectDbStats> {
        let conn = self.conn.lock();

        let count = |table: &str| -> SqlResult<i64> {
            conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
                row.get(0)
            })
        };

        let db_size = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        let wal_path = self.path.with_extension("ffxdb-wal");
        let wal_size = std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);
        let wal_exists = wal_path.exists();
        let last_wal_checkpoint = self.last_wal_checkpoint.lock().clone();

        Ok(ProjectDbStats {
            total_activities: count("activity_log")?,
            total_sessions: count("sessions")?,
            total_users: count("users")?,
            total_evidence_files: count("evidence_files")?,
            total_hashes: count("hashes")?,
            total_verifications: count("verifications")?,
            total_bookmarks: count("bookmarks")?,
            total_notes: count("notes")?,
            total_tags: count("tags")?,
            total_reports: count("reports")?,
            total_saved_searches: count("saved_searches")?,
            total_case_documents: count("case_documents")?,
            total_processed_databases: count("processed_databases")?,
            total_axiom_cases: count("axiom_case_info")?,
            total_artifact_categories: count("artifact_categories")?,
            total_exports: count("export_history")?,
            total_annotations: count("annotations")?,
            total_coc_items: count("coc_items")?,
            total_coc_transfers: count("coc_transfers")?,
            total_evidence_collections: count("evidence_collections")?,
            total_collected_items: count("collected_items")?,
            total_coc_amendments: count("coc_amendments")?,
            total_coc_audit_entries: count("coc_audit_log")?,
            db_size_bytes: db_size,
            wal_exists,
            wal_size_bytes: wal_size,
            last_wal_checkpoint,
            schema_version: SCHEMA_VERSION,
        })
    }
}
