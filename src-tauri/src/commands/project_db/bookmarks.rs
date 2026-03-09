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
pub fn project_db_upsert_bookmark(
    window: tauri::Window,
    bookmark: DbBookmark,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_bookmark(&bookmark))
}

/// Get all bookmarks.
#[tauri::command]
pub fn project_db_get_bookmarks(window: tauri::Window) -> Result<Vec<DbBookmark>, String> {
    with_project_db(window.label(), |db| db.get_bookmarks())
}

/// Delete a bookmark.
#[tauri::command]
pub fn project_db_delete_bookmark(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_bookmark(&id))
}

// =============================================================================
// Note Commands
// =============================================================================

/// Insert or update a note.
#[tauri::command]
pub fn project_db_upsert_note(window: tauri::Window, note: DbNote) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_note(&note))
}

/// Get all notes.
#[tauri::command]
pub fn project_db_get_notes(window: tauri::Window) -> Result<Vec<DbNote>, String> {
    with_project_db(window.label(), |db| db.get_notes())
}

/// Delete a note.
#[tauri::command]
pub fn project_db_delete_note(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_note(&id))
}

// =============================================================================
// Tag Commands
// =============================================================================

/// Insert or update a tag definition.
#[tauri::command]
pub fn project_db_upsert_tag(window: tauri::Window, tag: DbTag) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_tag(&tag))
}

/// Get all tags.
#[tauri::command]
pub fn project_db_get_tags(window: tauri::Window) -> Result<Vec<DbTag>, String> {
    with_project_db(window.label(), |db| db.get_tags())
}

/// Delete a tag and its assignments.
#[tauri::command]
pub fn project_db_delete_tag(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_tag(&id))
}

/// Assign a tag to a target.
#[tauri::command]
pub fn project_db_assign_tag(
    window: tauri::Window,
    assignment: DbTagAssignment,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.assign_tag(&assignment))
}

/// Remove a tag assignment.
#[tauri::command]
pub fn project_db_remove_tag(
    window: tauri::Window,
    tag_id: String,
    target_type: String,
    target_id: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.remove_tag(&tag_id, &target_type, &target_id)
    })
}

/// Get tags for a specific target.
#[tauri::command]
pub fn project_db_get_tags_for_target(
    window: tauri::Window,
    target_type: String,
    target_id: String,
) -> Result<Vec<DbTag>, String> {
    with_project_db(window.label(), |db| {
        db.get_tags_for_target(&target_type, &target_id)
    })
}
