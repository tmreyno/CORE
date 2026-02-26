// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Activity log, session, and user operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Activity Log Operations
    // ========================================================================

    /// Insert a new activity log entry
    pub fn insert_activity(&self, entry: &DbActivityEntry) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO activity_log (id, timestamp, user, category, action, description, file_path, details)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id,
                entry.timestamp,
                entry.user,
                entry.category,
                entry.action,
                entry.description,
                entry.file_path,
                entry.details,
            ],
        )?;
        Ok(())
    }

    /// Query activity log with filters
    pub fn query_activities(&self, query: &ActivityQuery) -> SqlResult<Vec<DbActivityEntry>> {
        let conn = self.conn.lock();

        let mut sql = String::from(
            "SELECT id, timestamp, user, category, action, description, file_path, details
             FROM activity_log WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref cat) = query.category {
            param_values.push(Box::new(cat.clone()));
            sql.push_str(&format!(" AND category = ?{}", param_values.len()));
        }
        if let Some(ref user) = query.user {
            param_values.push(Box::new(user.clone()));
            sql.push_str(&format!(" AND user = ?{}", param_values.len()));
        }
        if let Some(ref since) = query.since {
            param_values.push(Box::new(since.clone()));
            sql.push_str(&format!(" AND timestamp >= ?{}", param_values.len()));
        }
        if let Some(ref until) = query.until {
            param_values.push(Box::new(until.clone()));
            sql.push_str(&format!(" AND timestamp <= ?{}", param_values.len()));
        }
        if let Some(ref search) = query.search {
            param_values.push(Box::new(format!("%{}%", search)));
            sql.push_str(&format!(" AND description LIKE ?{}", param_values.len()));
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = query.limit {
            param_values.push(Box::new(limit));
            sql.push_str(&format!(" LIMIT ?{}", param_values.len()));
        }
        if let Some(offset) = query.offset {
            param_values.push(Box::new(offset));
            sql.push_str(&format!(" OFFSET ?{}", param_values.len()));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(DbActivityEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                user: row.get(2)?,
                category: row.get(3)?,
                action: row.get(4)?,
                description: row.get(5)?,
                file_path: row.get(6)?,
                details: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    /// Get total activity count (for pagination)
    pub fn count_activities(&self, category: Option<&str>) -> SqlResult<i64> {
        let conn = self.conn.lock();
        if let Some(cat) = category {
            conn.query_row(
                "SELECT COUNT(*) FROM activity_log WHERE category = ?1",
                params![cat],
                |row| row.get(0),
            )
        } else {
            conn.query_row("SELECT COUNT(*) FROM activity_log", [], |row| row.get(0))
        }
    }

    // ========================================================================
    // Session Operations
    // ========================================================================

    /// Insert or update a session
    pub fn upsert_session(&self, session: &DbProjectSession) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO sessions (session_id, user, started_at, ended_at, duration_seconds, hostname, app_version, summary)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(session_id) DO UPDATE SET
                ended_at = excluded.ended_at,
                duration_seconds = excluded.duration_seconds,
                summary = excluded.summary",
            params![
                session.session_id,
                session.user,
                session.started_at,
                session.ended_at,
                session.duration_seconds,
                session.hostname,
                session.app_version,
                session.summary,
            ],
        )?;
        Ok(())
    }

    /// Get all sessions ordered by start time
    pub fn get_sessions(&self) -> SqlResult<Vec<DbProjectSession>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT session_id, user, started_at, ended_at, duration_seconds, hostname, app_version, summary
             FROM sessions ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbProjectSession {
                session_id: row.get(0)?,
                user: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                duration_seconds: row.get(4)?,
                hostname: row.get(5)?,
                app_version: row.get(6)?,
                summary: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    /// End a session (set ended_at and duration)
    pub fn end_session(&self, session_id: &str, summary: Option<&str>) -> SqlResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock();

        let started: String = conn.query_row(
            "SELECT started_at FROM sessions WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        let duration = if let (Ok(start), Ok(end)) = (
            chrono::DateTime::parse_from_rfc3339(&started),
            chrono::DateTime::parse_from_rfc3339(&now),
        ) {
            Some((end - start).num_seconds())
        } else {
            None
        };

        conn.execute(
            "UPDATE sessions SET ended_at = ?1, duration_seconds = ?2, summary = COALESCE(?3, summary) WHERE session_id = ?4",
            params![now, duration, summary, session_id],
        )?;
        Ok(())
    }

    // ========================================================================
    // User Operations
    // ========================================================================

    /// Insert or update a user
    pub fn upsert_user(&self, user: &DbProjectUser) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO users (username, display_name, hostname, first_access, last_access)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(username) DO UPDATE SET
                display_name = COALESCE(excluded.display_name, users.display_name),
                hostname = COALESCE(excluded.hostname, users.hostname),
                last_access = excluded.last_access",
            params![
                user.username,
                user.display_name,
                user.hostname,
                user.first_access,
                user.last_access,
            ],
        )?;
        Ok(())
    }

    /// Get all users
    pub fn get_users(&self) -> SqlResult<Vec<DbProjectUser>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT username, display_name, hostname, first_access, last_access FROM users ORDER BY last_access DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbProjectUser {
                username: row.get(0)?,
                display_name: row.get(1)?,
                hostname: row.get(2)?,
                first_access: row.get(3)?,
                last_access: row.get(4)?,
            })
        })?;

        rows.collect()
    }
}
