// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Full-Text Search (FTS5) operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

const DEFAULT_FTS_LIMIT: i64 = 50;
const MAX_FTS_LIMIT: i64 = 1_000;

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

        for table in [
            "fts_notes",
            "fts_bookmarks",
            "fts_activity_log",
            "fts_annotations",
            "fts_artifacts",
            "fts_source_analyses",
        ] {
            let table_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;

            if table_exists {
                conn.execute(
                    &format!("INSERT INTO {table}({table}) VALUES('rebuild')"),
                    [],
                )?;
            }
        }
        Ok(())
    }

    /// Full-text search across notes, bookmarks, activity log, annotations, artifacts, and source analysis.
    pub fn fts_search(&self, query: &str, limit: Option<i64>) -> SqlResult<Vec<FtsSearchResult>> {
        let conn = self.conn.lock();
        let max = normalized_fts_limit(limit);
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
            "SELECT target_path, name, snippet(fts_bookmarks, -1, '<mark>', '</mark>', '...', 32), rank
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

        // Search annotations and hex review findings
        if let Ok(mut stmt) = conn.prepare(
            "SELECT file_path, snippet(fts_annotations, -1, '<mark>', '</mark>', '...', 32), rank
             FROM fts_annotations WHERE fts_annotations MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "annotations".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(1)?,
                    rank: row.get(2)?,
                })
            }) {
                for r in rows.flatten() {
                    results.push(r);
                }
            }
        }

        // Search normalized artifacts extracted from evidence byte sources
        if let Ok(mut stmt) = conn.prepare(
            "SELECT source_id, snippet(fts_artifacts, -1, '<mark>', '</mark>', '...', 32), rank
             FROM fts_artifacts WHERE fts_artifacts MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "artifacts".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(1)?,
                    rank: row.get(2)?,
                })
            }) {
                for r in rows.flatten() {
                    results.push(r);
                }
            }
        }

        // Search persisted source-analysis facts from hex/data review engines
        if let Ok(mut stmt) = conn.prepare(
            "SELECT source_id, snippet(fts_source_analyses, -1, '<mark>', '</mark>', '...', 32), rank
             FROM fts_source_analyses WHERE fts_source_analyses MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "source_analysis".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(1)?,
                    rank: row.get(2)?,
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

fn normalized_fts_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(DEFAULT_FTS_LIMIT).clamp(1, MAX_FTS_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fts_limit_is_bounded() {
        assert_eq!(normalized_fts_limit(None), DEFAULT_FTS_LIMIT);
        assert_eq!(normalized_fts_limit(Some(-1)), 1);
        assert_eq!(normalized_fts_limit(Some(0)), 1);
        assert_eq!(normalized_fts_limit(Some(25)), 25);
        assert_eq!(normalized_fts_limit(Some(MAX_FTS_LIMIT + 1)), MAX_FTS_LIMIT);
    }
}
