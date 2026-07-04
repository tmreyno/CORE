// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for activity log, session, and user operations.

use super::with_project_db;
use crate::project_db::{ActivityQuery, DbActivityEntry, DbProjectSession, DbProjectUser};

const MAX_ACTIVITY_RESPONSE_ROWS: i64 = 10_000;
const MAX_ACTIVITY_FIELD_CHARS: usize = 4096;
const MAX_ACTIVITY_DETAILS_CHARS: usize = 65_536;
const ACTIVITY_TRUNCATED_SUFFIX: &str = "... [truncated]";

// =============================================================================
// Activity Log Commands
// =============================================================================

/// Insert a new activity log entry.
#[tauri::command]
pub fn project_db_insert_activity(
    window: tauri::Window,
    entry: DbActivityEntry,
) -> Result<(), String> {
    let entry = bounded_activity_entry(entry);
    with_project_db(window.label(), |db| db.insert_activity(&entry))
}

/// Query activity log with filters.
#[tauri::command]
pub fn project_db_query_activities(
    window: tauri::Window,
    query: ActivityQuery,
) -> Result<Vec<DbActivityEntry>, String> {
    let query = bounded_activity_query(query);
    with_project_db(window.label(), |db| {
        db.query_activities(&query).map(|entries| {
            entries
                .into_iter()
                .take(MAX_ACTIVITY_RESPONSE_ROWS as usize)
                .map(bounded_activity_entry)
                .collect()
        })
    })
}

/// Get total activity count, optionally filtered by category.
#[tauri::command]
pub fn project_db_count_activities(
    window: tauri::Window,
    category: Option<String>,
) -> Result<i64, String> {
    let category = category.map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    with_project_db(window.label(), |db| {
        db.count_activities(category.as_deref())
    })
}

// =============================================================================
// Session Commands
// =============================================================================

/// Insert or update a session.
#[tauri::command]
pub fn project_db_upsert_session(
    window: tauri::Window,
    session: DbProjectSession,
) -> Result<(), String> {
    let session = bounded_project_session(session);
    with_project_db(window.label(), |db| db.upsert_session(&session))
}

/// Get all sessions.
#[tauri::command]
pub fn project_db_get_sessions(window: tauri::Window) -> Result<Vec<DbProjectSession>, String> {
    with_project_db(window.label(), |db| {
        db.get_sessions().map(|sessions| {
            sessions
                .into_iter()
                .take(MAX_ACTIVITY_RESPONSE_ROWS as usize)
                .map(bounded_project_session)
                .collect()
        })
    })
}

/// End a session (set ended_at and duration).
#[tauri::command]
pub fn project_db_end_session(
    window: tauri::Window,
    session_id: String,
    summary: Option<String>,
) -> Result<(), String> {
    let session_id = truncate_activity_text(&session_id, MAX_ACTIVITY_FIELD_CHARS);
    let summary = summary.map(|value| truncate_activity_text(&value, MAX_ACTIVITY_DETAILS_CHARS));
    with_project_db(window.label(), |db| {
        db.end_session(&session_id, summary.as_deref())
    })
}

// =============================================================================
// User Commands
// =============================================================================

/// Insert or update a user.
#[tauri::command]
pub fn project_db_upsert_user(window: tauri::Window, user: DbProjectUser) -> Result<(), String> {
    let user = bounded_project_user(user);
    with_project_db(window.label(), |db| db.upsert_user(&user))
}

/// Get all users.
#[tauri::command]
pub fn project_db_get_users(window: tauri::Window) -> Result<Vec<DbProjectUser>, String> {
    with_project_db(window.label(), |db| {
        db.get_users().map(|users| {
            users
                .into_iter()
                .take(MAX_ACTIVITY_RESPONSE_ROWS as usize)
                .map(bounded_project_user)
                .collect()
        })
    })
}

fn bounded_activity_query(mut query: ActivityQuery) -> ActivityQuery {
    query.category = query
        .category
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    query.user = query
        .user
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    query.since = query
        .since
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    query.until = query
        .until
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    query.file_path = query
        .file_path
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_DETAILS_CHARS));
    query.search = query
        .search
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    query.limit = Some(
        query
            .limit
            .unwrap_or(MAX_ACTIVITY_RESPONSE_ROWS)
            .clamp(0, MAX_ACTIVITY_RESPONSE_ROWS),
    );
    query.offset = query.offset.map(|offset| offset.max(0));
    query
}

fn bounded_activity_entry(mut entry: DbActivityEntry) -> DbActivityEntry {
    entry.id = truncate_activity_text(&entry.id, MAX_ACTIVITY_FIELD_CHARS);
    entry.timestamp = truncate_activity_text(&entry.timestamp, MAX_ACTIVITY_FIELD_CHARS);
    entry.user = truncate_activity_text(&entry.user, MAX_ACTIVITY_FIELD_CHARS);
    entry.category = truncate_activity_text(&entry.category, MAX_ACTIVITY_FIELD_CHARS);
    entry.action = truncate_activity_text(&entry.action, MAX_ACTIVITY_FIELD_CHARS);
    entry.description = truncate_activity_text(&entry.description, MAX_ACTIVITY_DETAILS_CHARS);
    entry.file_path = entry
        .file_path
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_DETAILS_CHARS));
    entry.details = entry
        .details
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_DETAILS_CHARS));
    entry
}

fn bounded_project_session(mut session: DbProjectSession) -> DbProjectSession {
    session.session_id = truncate_activity_text(&session.session_id, MAX_ACTIVITY_FIELD_CHARS);
    session.user = truncate_activity_text(&session.user, MAX_ACTIVITY_FIELD_CHARS);
    session.started_at = truncate_activity_text(&session.started_at, MAX_ACTIVITY_FIELD_CHARS);
    session.ended_at = session
        .ended_at
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    session.hostname = session
        .hostname
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    session.app_version = truncate_activity_text(&session.app_version, MAX_ACTIVITY_FIELD_CHARS);
    session.summary = session
        .summary
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_DETAILS_CHARS));
    session
}

fn bounded_project_user(mut user: DbProjectUser) -> DbProjectUser {
    user.username = truncate_activity_text(&user.username, MAX_ACTIVITY_FIELD_CHARS);
    user.display_name = user
        .display_name
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    user.hostname = user
        .hostname
        .map(|value| truncate_activity_text(&value, MAX_ACTIVITY_FIELD_CHARS));
    user.first_access = truncate_activity_text(&user.first_access, MAX_ACTIVITY_FIELD_CHARS);
    user.last_access = truncate_activity_text(&user.last_access, MAX_ACTIVITY_FIELD_CHARS);
    user
}

fn truncate_activity_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let keep_chars = max_chars.saturating_sub(ACTIVITY_TRUNCATED_SUFFIX.chars().count());
    let mut truncated: String = value.chars().take(keep_chars).collect();
    truncated.push_str(ACTIVITY_TRUNCATED_SUFFIX);
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repeated(len: usize) -> String {
        "x".repeat(len)
    }

    #[test]
    fn bounded_activity_query_caps_limit_offset_and_filter_text() {
        let query = bounded_activity_query(ActivityQuery {
            category: Some(repeated(MAX_ACTIVITY_FIELD_CHARS + 8)),
            user: Some(repeated(MAX_ACTIVITY_FIELD_CHARS + 8)),
            since: None,
            until: None,
            file_path: Some(repeated(MAX_ACTIVITY_DETAILS_CHARS + 8)),
            search: Some(repeated(MAX_ACTIVITY_FIELD_CHARS + 8)),
            limit: Some(MAX_ACTIVITY_RESPONSE_ROWS + 1),
            offset: Some(-20),
        });

        assert_eq!(query.limit, Some(MAX_ACTIVITY_RESPONSE_ROWS));
        assert_eq!(query.offset, Some(0));
        assert_eq!(
            query.category.unwrap().chars().count(),
            MAX_ACTIVITY_FIELD_CHARS
        );
        assert_eq!(
            query.file_path.unwrap().chars().count(),
            MAX_ACTIVITY_DETAILS_CHARS
        );
        assert!(query.search.unwrap().ends_with(ACTIVITY_TRUNCATED_SUFFIX));
    }

    #[test]
    fn bounded_activity_query_defaults_missing_limit() {
        let query = bounded_activity_query(ActivityQuery::default());

        assert_eq!(query.limit, Some(MAX_ACTIVITY_RESPONSE_ROWS));
        assert_eq!(query.offset, None);
    }

    #[test]
    fn bounded_activity_entry_caps_audit_payloads() {
        let entry = bounded_activity_entry(DbActivityEntry {
            id: repeated(MAX_ACTIVITY_FIELD_CHARS + 8),
            timestamp: "2026-02-16T10:00:00Z".to_string(),
            user: repeated(MAX_ACTIVITY_FIELD_CHARS + 8),
            category: "analysis".to_string(),
            action: "extract".to_string(),
            description: repeated(MAX_ACTIVITY_DETAILS_CHARS + 8),
            file_path: Some(repeated(MAX_ACTIVITY_DETAILS_CHARS + 8)),
            details: Some(repeated(MAX_ACTIVITY_DETAILS_CHARS + 8)),
        });

        assert_eq!(entry.id.chars().count(), MAX_ACTIVITY_FIELD_CHARS);
        assert_eq!(entry.user.chars().count(), MAX_ACTIVITY_FIELD_CHARS);
        assert_eq!(
            entry.description.chars().count(),
            MAX_ACTIVITY_DETAILS_CHARS
        );
        assert_eq!(
            entry.file_path.unwrap().chars().count(),
            MAX_ACTIVITY_DETAILS_CHARS
        );
        assert!(entry.details.unwrap().ends_with(ACTIVITY_TRUNCATED_SUFFIX));
    }

    #[test]
    fn bounded_session_and_user_caps_metadata() {
        let session = bounded_project_session(DbProjectSession {
            session_id: repeated(MAX_ACTIVITY_FIELD_CHARS + 8),
            user: "examiner".to_string(),
            started_at: "2026-02-16T10:00:00Z".to_string(),
            ended_at: Some(repeated(MAX_ACTIVITY_FIELD_CHARS + 8)),
            duration_seconds: Some(10),
            hostname: Some(repeated(MAX_ACTIVITY_FIELD_CHARS + 8)),
            app_version: "0.1.112".to_string(),
            summary: Some(repeated(MAX_ACTIVITY_DETAILS_CHARS + 8)),
        });
        let user = bounded_project_user(DbProjectUser {
            username: repeated(MAX_ACTIVITY_FIELD_CHARS + 8),
            display_name: Some(repeated(MAX_ACTIVITY_FIELD_CHARS + 8)),
            hostname: Some(repeated(MAX_ACTIVITY_FIELD_CHARS + 8)),
            first_access: "2026-02-16T10:00:00Z".to_string(),
            last_access: repeated(MAX_ACTIVITY_FIELD_CHARS + 8),
        });

        assert_eq!(session.session_id.chars().count(), MAX_ACTIVITY_FIELD_CHARS);
        assert_eq!(
            session.summary.unwrap().chars().count(),
            MAX_ACTIVITY_DETAILS_CHARS
        );
        assert_eq!(user.username.chars().count(), MAX_ACTIVITY_FIELD_CHARS);
        assert_eq!(
            user.display_name.unwrap().chars().count(),
            MAX_ACTIVITY_FIELD_CHARS
        );
        assert!(user.last_access.ends_with(ACTIVITY_TRUNCATED_SUFFIX));
    }
}
