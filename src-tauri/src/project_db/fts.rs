// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Full-Text Search (FTS5) operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Full-Text Search (FTS5)
    // ========================================================================

    /// Rebuild FTS indexes by re-populating from source tables
    pub fn rebuild_fts_indexes(&self) -> SqlResult<()> {
        let conn = self.conn.lock();
        // Only rebuild if the FTS tables exist
        let has_fts: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='fts_notes'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if !has_fts {
            return Ok(());
        }

        conn.execute_batch(
            r#"
            DELETE FROM fts_notes;
            INSERT INTO fts_notes(rowid, target_path, title, content, tags)
                SELECT rowid, target_path, COALESCE(title,''), COALESCE(content,''), COALESCE(tags,'') FROM notes;
            DELETE FROM fts_bookmarks;
            INSERT INTO fts_bookmarks(rowid, target_path, label, description)
                SELECT rowid, target_path, COALESCE(label,''), COALESCE(description,'') FROM bookmarks;
            DELETE FROM fts_activity_log;
            INSERT INTO fts_activity_log(rowid, action, description, file_path, details)
                SELECT rowid, COALESCE(action,''), COALESCE(description,''), COALESCE(file_path,''), COALESCE(details,'') FROM activity_log;
            "#,
        )?;
        Ok(())
    }

    /// Full-text search across notes, bookmarks, and activity log
    pub fn fts_search(&self, query: &str, limit: Option<i64>) -> SqlResult<Vec<FtsSearchResult>> {
        let conn = self.conn.lock();
        let max = limit.unwrap_or(50);
        let mut results = Vec::new();

        // Check FTS availability
        let has_fts: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='fts_notes'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if !has_fts {
            return Ok(results);
        }

        // Search notes
        if let Ok(mut stmt) = conn.prepare(
            "SELECT target_path, title, snippet(fts_notes, 2, '<mark>', '</mark>', '...', 32), rank
             FROM fts_notes WHERE fts_notes MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "notes".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            }) {
                for r in rows.flatten() {
                    results.push(r);
                }
            }
        }

        // Search bookmarks
        if let Ok(mut stmt) = conn.prepare(
            "SELECT target_path, label, snippet(fts_bookmarks, 2, '<mark>', '</mark>', '...', 32), rank
             FROM fts_bookmarks WHERE fts_bookmarks MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "bookmarks".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            }) {
                for r in rows.flatten() {
                    results.push(r);
                }
            }
        }

        // Search activity log
        if let Ok(mut stmt) = conn.prepare(
            "SELECT action, description, snippet(fts_activity_log, 3, '<mark>', '</mark>', '...', 32), rank
             FROM fts_activity_log WHERE fts_activity_log MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "activity_log".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            }) {
                for r in rows.flatten() {
                    results.push(r);
                }
            }
        }

        // Sort by rank (ascending = better match)
        results.sort_by(|a, b| {
            a.rank
                .partial_cmp(&b.rank)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(max as usize);

        Ok(results)
    }
}
