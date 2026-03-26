// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Annotation operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Annotation Operations
    // ========================================================================

    /// Insert a new annotation
    pub fn insert_annotation(&self, ann: &DbAnnotation) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO annotations (id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                ann.id, ann.file_path, ann.container_path, ann.annotation_type,
                ann.offset_start, ann.offset_end, ann.line_start, ann.line_end,
                ann.label, ann.content, ann.color, ann.created_by,
                ann.created_at, ann.modified_at,
            ],
        )?;
        Ok(())
    }

    /// Update an annotation
    pub fn update_annotation(&self, ann: &DbAnnotation) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE annotations SET label = ?1, content = ?2, color = ?3, modified_at = ?4 WHERE id = ?5",
            params![ann.label, ann.content, ann.color, ann.modified_at, ann.id],
        )?;
        Ok(())
    }

    /// Get annotations for a file
    pub fn get_annotations_for_path(&self, file_path: &str) -> SqlResult<Vec<DbAnnotation>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at
             FROM annotations WHERE file_path = ?1 ORDER BY COALESCE(offset_start, line_start, 0)",
        )?;
        let rows = stmt.query_map(params![file_path], Self::map_annotation)?;
        rows.collect()
    }

    /// Get all annotations
    pub fn get_all_annotations(&self) -> SqlResult<Vec<DbAnnotation>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at
             FROM annotations ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], Self::map_annotation)?;
        rows.collect()
    }

    /// Delete an annotation by ID
    pub fn delete_annotation(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM annotations WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Row mapper for DbAnnotation (14 columns)
    fn map_annotation(row: &rusqlite::Row<'_>) -> rusqlite::Result<DbAnnotation> {
        Ok(DbAnnotation {
            id: row.get(0)?,
            file_path: row.get(1)?,
            container_path: row.get(2)?,
            annotation_type: row.get(3)?,
            offset_start: row.get(4)?,
            offset_end: row.get(5)?,
            line_start: row.get(6)?,
            line_end: row.get(7)?,
            label: row.get(8)?,
            content: row.get(9)?,
            color: row.get(10)?,
            created_by: row.get(11)?,
            created_at: row.get(12)?,
            modified_at: row.get(13)?,
        })
    }
}
