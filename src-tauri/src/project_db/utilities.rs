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
        let conn = self.conn.lock();
        let (log_size, frames_checkpointed): (i64, i64) = conn.query_row(
            "PRAGMA wal_checkpoint(TRUNCATE)",
            [],
            |row| Ok((row.get(1)?, row.get(2)?)),
        )?;
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

        let db_size = std::fs::metadata(&self.path)
            .map(|m| m.len())
            .unwrap_or(0);

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
            total_custody_records: count("chain_of_custody")?,
            total_classifications: count("file_classifications")?,
            total_extractions: count("extraction_log")?,
            total_viewer_history: count("viewer_history")?,
            total_annotations: count("annotations")?,
            total_relationships: count("evidence_relationships")?,
            total_coc_items: count("coc_items")?,
            total_coc_transfers: count("coc_transfers")?,
            total_evidence_collections: count("evidence_collections")?,
            total_collected_items: count("collected_items")?,
            total_coc_amendments: count("coc_amendments")?,
            total_coc_audit_entries: count("coc_audit_log")?,
            db_size_bytes: db_size,
            schema_version: SCHEMA_VERSION,
        })
    }
}
