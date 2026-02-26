// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Form submission operations (generic JSON-driven forms).

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Form Submission Operations (Generic JSON-driven forms)
    // ========================================================================

    /// Upsert a form submission (insert or update)
    pub fn upsert_form_submission(&self, sub: &DbFormSubmission) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO form_submissions (id, template_id, template_version, case_number, data_json, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                template_version=excluded.template_version,
                case_number=excluded.case_number,
                data_json=excluded.data_json,
                status=excluded.status,
                updated_at=excluded.updated_at",
            params![
                sub.id, sub.template_id, sub.template_version,
                sub.case_number, sub.data_json, sub.status,
                sub.created_at, sub.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Get a form submission by ID
    pub fn get_form_submission(&self, id: &str) -> SqlResult<Option<DbFormSubmission>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, template_id, template_version, case_number, data_json, status, created_at, updated_at
             FROM form_submissions WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], |row| {
            Ok(DbFormSubmission {
                id: row.get(0)?,
                template_id: row.get(1)?,
                template_version: row.get(2)?,
                case_number: row.get(3)?,
                data_json: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        });
        match result {
            Ok(sub) => Ok(Some(sub)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// List form submissions, optionally filtered by template_id and/or case_number
    pub fn list_form_submissions(
        &self,
        template_id: Option<&str>,
        case_number: Option<&str>,
        status: Option<&str>,
    ) -> SqlResult<Vec<DbFormSubmission>> {
        let conn = self.conn.lock();
        let mut sql = String::from(
            "SELECT id, template_id, template_version, case_number, data_json, status, created_at, updated_at
             FROM form_submissions WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(tid) = template_id {
            param_values.push(Box::new(tid.to_string()));
            sql.push_str(&format!(" AND template_id = ?{}", param_values.len()));
        }
        if let Some(cn) = case_number {
            param_values.push(Box::new(cn.to_string()));
            sql.push_str(&format!(" AND case_number = ?{}", param_values.len()));
        }
        if let Some(s) = status {
            param_values.push(Box::new(s.to_string()));
            sql.push_str(&format!(" AND status = ?{}", param_values.len()));
        }
        sql.push_str(" ORDER BY updated_at DESC");

        let mut stmt = conn.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), |row| {
            Ok(DbFormSubmission {
                id: row.get(0)?,
                template_id: row.get(1)?,
                template_version: row.get(2)?,
                case_number: row.get(3)?,
                data_json: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Delete a form submission by ID (only if status is 'draft')
    pub fn delete_form_submission(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        let status: String = conn.query_row(
            "SELECT status FROM form_submissions WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        if status != "draft" {
            return Err(rusqlite::Error::QueryReturnedNoRows); // Cannot delete non-draft
        }
        conn.execute("DELETE FROM form_submissions WHERE id = ?1", params![id])?;
        Ok(())
    }
}
