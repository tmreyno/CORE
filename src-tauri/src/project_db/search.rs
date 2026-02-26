// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Report, saved search, recent search, and case document operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Report Operations
    // ========================================================================

    /// Insert a report record
    pub fn insert_report(&self, r: &DbReportRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO reports (id, title, report_type, format, output_path, generated_at, generated_by, status, error, config)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                r.id, r.title, r.report_type, r.format, r.output_path,
                r.generated_at, r.generated_by, r.status, r.error, r.config,
            ],
        )?;
        Ok(())
    }

    /// Get all reports
    pub fn get_reports(&self) -> SqlResult<Vec<DbReportRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, title, report_type, format, output_path, generated_at, generated_by, status, error, config
             FROM reports ORDER BY generated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbReportRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                report_type: row.get(2)?,
                format: row.get(3)?,
                output_path: row.get(4)?,
                generated_at: row.get(5)?,
                generated_by: row.get(6)?,
                status: row.get(7)?,
                error: row.get(8)?,
                config: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Search Operations
    // ========================================================================

    /// Insert or update a saved search
    pub fn upsert_saved_search(&self, s: &DbSavedSearch) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO saved_searches (id, name, query, search_type, is_regex, case_sensitive, scope, created_at, use_count, last_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                query = excluded.query,
                use_count = excluded.use_count,
                last_used = excluded.last_used",
            params![
                s.id, s.name, s.query, s.search_type,
                s.is_regex as i32, s.case_sensitive as i32,
                s.scope, s.created_at, s.use_count, s.last_used,
            ],
        )?;
        Ok(())
    }

    /// Get all saved searches
    pub fn get_saved_searches(&self) -> SqlResult<Vec<DbSavedSearch>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, query, search_type, is_regex, case_sensitive, scope, created_at, use_count, last_used
             FROM saved_searches ORDER BY use_count DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let is_regex: i32 = row.get(4)?;
            let case_sensitive: i32 = row.get(5)?;
            Ok(DbSavedSearch {
                id: row.get(0)?,
                name: row.get(1)?,
                query: row.get(2)?,
                search_type: row.get(3)?,
                is_regex: is_regex != 0,
                case_sensitive: case_sensitive != 0,
                scope: row.get(6)?,
                created_at: row.get(7)?,
                use_count: row.get(8)?,
                last_used: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    /// Insert or update a recent search
    pub fn insert_recent_search(&self, s: &DbRecentSearch) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO recent_searches (query, timestamp, result_count)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(query) DO UPDATE SET
                timestamp = excluded.timestamp,
                result_count = excluded.result_count",
            params![s.query, s.timestamp, s.result_count],
        )?;
        Ok(())
    }

    // ========================================================================
    // Case Document Operations
    // ========================================================================

    /// Insert or update a case document
    pub fn upsert_case_document(&self, d: &DbCaseDocument) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO case_documents (id, path, filename, document_type, size, format, case_number, evidence_id, modified, discovered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(path) DO UPDATE SET
                filename = excluded.filename,
                document_type = excluded.document_type,
                size = excluded.size,
                format = excluded.format,
                case_number = excluded.case_number,
                evidence_id = excluded.evidence_id,
                modified = excluded.modified",
            params![
                d.id, d.path, d.filename, d.document_type, d.size, d.format,
                d.case_number, d.evidence_id, d.modified, d.discovered_at,
            ],
        )?;
        Ok(())
    }

    /// Get all case documents
    pub fn get_case_documents(&self) -> SqlResult<Vec<DbCaseDocument>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, filename, document_type, size, format, case_number, evidence_id, modified, discovered_at
             FROM case_documents ORDER BY filename",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbCaseDocument {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                document_type: row.get(3)?,
                size: row.get(4)?,
                format: row.get(5)?,
                case_number: row.get(6)?,
                evidence_id: row.get(7)?,
                modified: row.get(8)?,
                discovered_at: row.get(9)?,
            })
        })?;

        rows.collect()
    }
}
