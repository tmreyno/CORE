// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for bookmarks, notes, and tags.

use super::with_project_db;
use crate::project_db::{DbBookmark, DbNote, DbTag, DbTagAssignment};

// =============================================================================
// Bookmark Commands
// =============================================================================

/// Insert or update a bookmark.
#[tauri::command]
pub fn project_db_upsert_bookmark(bookmark: DbBookmark) -> Result<(), String> {
    with_project_db(|db| db.upsert_bookmark(&bookmark))
}

/// Get all bookmarks.
#[tauri::command]
pub fn project_db_get_bookmarks() -> Result<Vec<DbBookmark>, String> {
    with_project_db(|db| db.get_bookmarks())
}

/// Delete a bookmark.
#[tauri::command]
pub fn project_db_delete_bookmark(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_bookmark(&id))
}

// =============================================================================
// Note Commands
// =============================================================================

/// Insert or update a note.
#[tauri::command]
pub fn project_db_upsert_note(note: DbNote) -> Result<(), String> {
    with_project_db(|db| db.upsert_note(&note))
}

/// Get all notes.
#[tauri::command]
pub fn project_db_get_notes() -> Result<Vec<DbNote>, String> {
    with_project_db(|db| db.get_notes())
}

/// Delete a note.
#[tauri::command]
pub fn project_db_delete_note(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_note(&id))
}

// =============================================================================
// Tag Commands
// =============================================================================

/// Insert or update a tag definition.
#[tauri::command]
pub fn project_db_upsert_tag(tag: DbTag) -> Result<(), String> {
    with_project_db(|db| db.upsert_tag(&tag))
}

/// Get all tags.
#[tauri::command]
pub fn project_db_get_tags() -> Result<Vec<DbTag>, String> {
    with_project_db(|db| db.get_tags())
}

/// Delete a tag and its assignments.
#[tauri::command]
pub fn project_db_delete_tag(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_tag(&id))
}

/// Assign a tag to a target.
#[tauri::command]
pub fn project_db_assign_tag(assignment: DbTagAssignment) -> Result<(), String> {
    with_project_db(|db| db.assign_tag(&assignment))
}

/// Remove a tag assignment.
#[tauri::command]
pub fn project_db_remove_tag(
    tag_id: String,
    target_type: String,
    target_id: String,
) -> Result<(), String> {
    with_project_db(|db| db.remove_tag(&tag_id, &target_type, &target_id))
}

/// Get tags for a specific target.
#[tauri::command]
pub fn project_db_get_tags_for_target(
    target_type: String,
    target_id: String,
) -> Result<Vec<DbTag>, String> {
    with_project_db(|db| db.get_tags_for_target(&target_type, &target_id))
}
