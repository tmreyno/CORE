// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for activity log, session, and user operations.

use super::with_project_db;
use crate::project_db::{ActivityQuery, DbActivityEntry, DbProjectSession, DbProjectUser};

// =============================================================================
// Activity Log Commands
// =============================================================================

/// Insert a new activity log entry.
#[tauri::command]
pub fn project_db_insert_activity(entry: DbActivityEntry) -> Result<(), String> {
    with_project_db(|db| db.insert_activity(&entry))
}

/// Query activity log with filters.
#[tauri::command]
pub fn project_db_query_activities(query: ActivityQuery) -> Result<Vec<DbActivityEntry>, String> {
    with_project_db(|db| db.query_activities(&query))
}

/// Get total activity count, optionally filtered by category.
#[tauri::command]
pub fn project_db_count_activities(category: Option<String>) -> Result<i64, String> {
    with_project_db(|db| db.count_activities(category.as_deref()))
}

// =============================================================================
// Session Commands
// =============================================================================

/// Insert or update a session.
#[tauri::command]
pub fn project_db_upsert_session(session: DbProjectSession) -> Result<(), String> {
    with_project_db(|db| db.upsert_session(&session))
}

/// Get all sessions.
#[tauri::command]
pub fn project_db_get_sessions() -> Result<Vec<DbProjectSession>, String> {
    with_project_db(|db| db.get_sessions())
}

/// End a session (set ended_at and duration).
#[tauri::command]
pub fn project_db_end_session(session_id: String, summary: Option<String>) -> Result<(), String> {
    with_project_db(|db| db.end_session(&session_id, summary.as_deref()))
}

// =============================================================================
// User Commands
// =============================================================================

/// Insert or update a user.
#[tauri::command]
pub fn project_db_upsert_user(user: DbProjectUser) -> Result<(), String> {
    with_project_db(|db| db.upsert_user(&user))
}

/// Get all users.
#[tauri::command]
pub fn project_db_get_users() -> Result<Vec<DbProjectUser>, String> {
    with_project_db(|db| db.get_users())
}
