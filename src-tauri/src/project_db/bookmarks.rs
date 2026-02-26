// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Bookmark, note, and tag operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Bookmark Operations
    // ========================================================================

    /// Insert or update a bookmark
    pub fn upsert_bookmark(&self, b: &DbBookmark) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO bookmarks (id, target_type, target_path, name, created_by, created_at, color, notes, context)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                color = excluded.color,
                notes = excluded.notes,
                context = excluded.context",
            params![
                b.id, b.target_type, b.target_path, b.name,
                b.created_by, b.created_at, b.color, b.notes, b.context,
            ],
        )?;
        Ok(())
    }

    /// Get all bookmarks
    pub fn get_bookmarks(&self) -> SqlResult<Vec<DbBookmark>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, target_type, target_path, name, created_by, created_at, color, notes, context
             FROM bookmarks ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbBookmark {
                id: row.get(0)?,
                target_type: row.get(1)?,
                target_path: row.get(2)?,
                name: row.get(3)?,
                created_by: row.get(4)?,
                created_at: row.get(5)?,
                color: row.get(6)?,
                notes: row.get(7)?,
                context: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    /// Delete a bookmark by ID
    pub fn delete_bookmark(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        conn.execute(
            "DELETE FROM tag_assignments WHERE target_type = 'bookmark' AND target_id = ?1",
            params![id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Note Operations
    // ========================================================================

    /// Insert or update a note
    pub fn upsert_note(&self, n: &DbNote) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO notes (id, target_type, target_path, title, content, created_by, created_at, modified_at, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                modified_at = excluded.modified_at,
                priority = excluded.priority",
            params![
                n.id, n.target_type, n.target_path, n.title, n.content,
                n.created_by, n.created_at, n.modified_at, n.priority,
            ],
        )?;
        Ok(())
    }

    /// Get all notes
    pub fn get_notes(&self) -> SqlResult<Vec<DbNote>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, target_type, target_path, title, content, created_by, created_at, modified_at, priority
             FROM notes ORDER BY modified_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbNote {
                id: row.get(0)?,
                target_type: row.get(1)?,
                target_path: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
                modified_at: row.get(7)?,
                priority: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    /// Delete a note by ID
    pub fn delete_note(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        conn.execute(
            "DELETE FROM tag_assignments WHERE target_type = 'note' AND target_id = ?1",
            params![id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Tag Operations
    // ========================================================================

    /// Insert or update a tag definition
    pub fn upsert_tag(&self, t: &DbTag) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO tags (id, name, color, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                color = excluded.color,
                description = excluded.description",
            params![t.id, t.name, t.color, t.description, t.created_at],
        )?;
        Ok(())
    }

    /// Get all tags
    pub fn get_tags(&self) -> SqlResult<Vec<DbTag>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, color, description, created_at FROM tags ORDER BY name",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    /// Assign a tag to a target
    pub fn assign_tag(&self, assignment: &DbTagAssignment) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO tag_assignments (tag_id, target_type, target_id, assigned_at, assigned_by)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                assignment.tag_id,
                assignment.target_type,
                assignment.target_id,
                assignment.assigned_at,
                assignment.assigned_by,
            ],
        )?;
        Ok(())
    }

    /// Remove a tag assignment
    pub fn remove_tag(&self, tag_id: &str, target_type: &str, target_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM tag_assignments WHERE tag_id = ?1 AND target_type = ?2 AND target_id = ?3",
            params![tag_id, target_type, target_id],
        )?;
        Ok(())
    }

    /// Get tags for a specific target
    pub fn get_tags_for_target(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> SqlResult<Vec<DbTag>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.description, t.created_at
             FROM tags t
             INNER JOIN tag_assignments ta ON t.id = ta.tag_id
             WHERE ta.target_type = ?1 AND ta.target_id = ?2
             ORDER BY t.name",
        )?;

        let rows = stmt.query_map(params![target_type, target_id], |row| {
            Ok(DbTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    /// Delete a tag and all its assignments
    pub fn delete_tag(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM tag_assignments WHERE tag_id = ?1", params![id])?;
        conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        Ok(())
    }
}
