// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for bookmarks, notes, and tags.

use super::with_project_db;
use crate::project_db::{DbBookmark, DbNote, DbTag, DbTagAssignment};

const MAX_WORKFLOW_RESPONSE_ROWS: usize = 10_000;
const MAX_WORKFLOW_FIELD_CHARS: usize = 4096;
const MAX_WORKFLOW_BODY_CHARS: usize = 16_384;
const WORKFLOW_TRUNCATED_SUFFIX: &str = "... [truncated]";

// =============================================================================
// Bookmark Commands
// =============================================================================

/// Insert or update a bookmark.
#[tauri::command]
pub fn project_db_upsert_bookmark(
    window: tauri::Window,
    bookmark: DbBookmark,
) -> Result<(), String> {
    let bookmark = bounded_bookmark(bookmark);
    with_project_db(window.label(), |db| db.upsert_bookmark(&bookmark))
}

/// Get all bookmarks.
#[tauri::command]
pub fn project_db_get_bookmarks(window: tauri::Window) -> Result<Vec<DbBookmark>, String> {
    with_project_db(window.label(), |db| db.get_bookmarks()).map(|bookmarks| {
        bookmarks
            .into_iter()
            .take(MAX_WORKFLOW_RESPONSE_ROWS)
            .map(bounded_bookmark)
            .collect()
    })
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
    let note = bounded_note(note);
    with_project_db(window.label(), |db| db.upsert_note(&note))
}

/// Get all notes.
#[tauri::command]
pub fn project_db_get_notes(window: tauri::Window) -> Result<Vec<DbNote>, String> {
    with_project_db(window.label(), |db| db.get_notes()).map(|notes| {
        notes
            .into_iter()
            .take(MAX_WORKFLOW_RESPONSE_ROWS)
            .map(bounded_note)
            .collect()
    })
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
    let tag = bounded_tag(tag);
    with_project_db(window.label(), |db| db.upsert_tag(&tag))
}

/// Get all tags.
#[tauri::command]
pub fn project_db_get_tags(window: tauri::Window) -> Result<Vec<DbTag>, String> {
    with_project_db(window.label(), |db| db.get_tags()).map(|tags| {
        tags.into_iter()
            .take(MAX_WORKFLOW_RESPONSE_ROWS)
            .map(bounded_tag)
            .collect()
    })
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
    let assignment = bounded_tag_assignment(assignment);
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
    .map(|tags| {
        tags.into_iter()
            .take(MAX_WORKFLOW_RESPONSE_ROWS)
            .map(bounded_tag)
            .collect()
    })
}

fn bounded_bookmark(mut bookmark: DbBookmark) -> DbBookmark {
    bookmark.id = truncate_workflow_text(&bookmark.id, MAX_WORKFLOW_FIELD_CHARS);
    bookmark.target_type = truncate_workflow_text(&bookmark.target_type, MAX_WORKFLOW_FIELD_CHARS);
    bookmark.target_path = truncate_workflow_text(&bookmark.target_path, MAX_WORKFLOW_FIELD_CHARS);
    bookmark.name = truncate_workflow_text(&bookmark.name, MAX_WORKFLOW_FIELD_CHARS);
    bookmark.created_by = truncate_workflow_text(&bookmark.created_by, MAX_WORKFLOW_FIELD_CHARS);
    bookmark.created_at = truncate_workflow_text(&bookmark.created_at, MAX_WORKFLOW_FIELD_CHARS);
    bookmark.color = bookmark
        .color
        .map(|value| truncate_workflow_text(&value, MAX_WORKFLOW_FIELD_CHARS));
    bookmark.notes = bookmark
        .notes
        .map(|value| truncate_workflow_text(&value, MAX_WORKFLOW_BODY_CHARS));
    bookmark.context = bookmark
        .context
        .map(|value| truncate_workflow_text(&value, MAX_WORKFLOW_BODY_CHARS));
    bookmark
}

fn bounded_note(mut note: DbNote) -> DbNote {
    note.id = truncate_workflow_text(&note.id, MAX_WORKFLOW_FIELD_CHARS);
    note.target_type = truncate_workflow_text(&note.target_type, MAX_WORKFLOW_FIELD_CHARS);
    note.target_path = note
        .target_path
        .map(|value| truncate_workflow_text(&value, MAX_WORKFLOW_FIELD_CHARS));
    note.title = truncate_workflow_text(&note.title, MAX_WORKFLOW_FIELD_CHARS);
    note.content = truncate_workflow_text(&note.content, MAX_WORKFLOW_BODY_CHARS);
    note.created_by = truncate_workflow_text(&note.created_by, MAX_WORKFLOW_FIELD_CHARS);
    note.created_at = truncate_workflow_text(&note.created_at, MAX_WORKFLOW_FIELD_CHARS);
    note.modified_at = truncate_workflow_text(&note.modified_at, MAX_WORKFLOW_FIELD_CHARS);
    note.priority = note
        .priority
        .map(|value| truncate_workflow_text(&value, MAX_WORKFLOW_FIELD_CHARS));
    note
}

fn bounded_tag(mut tag: DbTag) -> DbTag {
    tag.id = truncate_workflow_text(&tag.id, MAX_WORKFLOW_FIELD_CHARS);
    tag.name = truncate_workflow_text(&tag.name, MAX_WORKFLOW_FIELD_CHARS);
    tag.color = truncate_workflow_text(&tag.color, MAX_WORKFLOW_FIELD_CHARS);
    tag.description = tag
        .description
        .map(|value| truncate_workflow_text(&value, MAX_WORKFLOW_BODY_CHARS));
    tag.created_at = truncate_workflow_text(&tag.created_at, MAX_WORKFLOW_FIELD_CHARS);
    tag
}

fn bounded_tag_assignment(mut assignment: DbTagAssignment) -> DbTagAssignment {
    assignment.tag_id = truncate_workflow_text(&assignment.tag_id, MAX_WORKFLOW_FIELD_CHARS);
    assignment.target_type =
        truncate_workflow_text(&assignment.target_type, MAX_WORKFLOW_FIELD_CHARS);
    assignment.target_id = truncate_workflow_text(&assignment.target_id, MAX_WORKFLOW_FIELD_CHARS);
    assignment.assigned_at =
        truncate_workflow_text(&assignment.assigned_at, MAX_WORKFLOW_FIELD_CHARS);
    assignment.assigned_by =
        truncate_workflow_text(&assignment.assigned_by, MAX_WORKFLOW_FIELD_CHARS);
    assignment
}

fn truncate_workflow_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = WORKFLOW_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + WORKFLOW_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(WORKFLOW_TRUNCATED_SUFFIX);
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_bookmark_caps_path_notes_and_context() {
        let bookmark = DbBookmark {
            id: "bm-1".to_string(),
            target_type: "hex".to_string(),
            target_path: "p".repeat(MAX_WORKFLOW_FIELD_CHARS + 32),
            name: "Magic Bytes".to_string(),
            created_by: "analyst".to_string(),
            created_at: "2026-02-16T10:00:00Z".to_string(),
            color: Some("#38bdf8".to_string()),
            notes: Some("n".repeat(MAX_WORKFLOW_BODY_CHARS + 32)),
            context: Some("c".repeat(MAX_WORKFLOW_BODY_CHARS + 32)),
        };

        let bounded = bounded_bookmark(bookmark);

        assert_eq!(
            bounded.target_path.chars().count(),
            MAX_WORKFLOW_FIELD_CHARS
        );
        assert!(bounded.target_path.ends_with(WORKFLOW_TRUNCATED_SUFFIX));
        assert_eq!(
            bounded.notes.as_deref().unwrap().chars().count(),
            MAX_WORKFLOW_BODY_CHARS
        );
        assert_eq!(
            bounded.context.as_deref().unwrap().chars().count(),
            MAX_WORKFLOW_BODY_CHARS
        );
    }

    #[test]
    fn bounded_note_caps_target_title_and_content() {
        let note = DbNote {
            id: "note-1".to_string(),
            target_type: "source".to_string(),
            target_path: Some("t".repeat(MAX_WORKFLOW_FIELD_CHARS + 32)),
            title: "x".repeat(MAX_WORKFLOW_FIELD_CHARS + 32),
            content: "é".repeat(MAX_WORKFLOW_BODY_CHARS + 32),
            created_by: "analyst".to_string(),
            created_at: "2026-02-16T10:00:00Z".to_string(),
            modified_at: "2026-02-16T10:00:00Z".to_string(),
            priority: Some("high".to_string()),
        };

        let bounded = bounded_note(note);

        assert_eq!(
            bounded.target_path.as_deref().unwrap().chars().count(),
            MAX_WORKFLOW_FIELD_CHARS
        );
        assert_eq!(bounded.title.chars().count(), MAX_WORKFLOW_FIELD_CHARS);
        assert_eq!(bounded.content.chars().count(), MAX_WORKFLOW_BODY_CHARS);
        assert!(bounded.content.ends_with(WORKFLOW_TRUNCATED_SUFFIX));
    }

    #[test]
    fn bounded_tag_caps_description_and_assignment_fields() {
        let tag = DbTag {
            id: "tag-1".to_string(),
            name: "important".to_string(),
            color: "#ef4444".to_string(),
            description: Some("d".repeat(MAX_WORKFLOW_BODY_CHARS + 32)),
            created_at: "2026-02-16T10:00:00Z".to_string(),
        };
        let assignment = DbTagAssignment {
            tag_id: "tag-1".to_string(),
            target_type: "bookmark".to_string(),
            target_id: "t".repeat(MAX_WORKFLOW_FIELD_CHARS + 32),
            assigned_at: "2026-02-16T10:00:00Z".to_string(),
            assigned_by: "analyst".to_string(),
        };

        let bounded_tag = bounded_tag(tag);
        let bounded_assignment = bounded_tag_assignment(assignment);

        assert_eq!(
            bounded_tag.description.as_deref().unwrap().chars().count(),
            MAX_WORKFLOW_BODY_CHARS
        );
        assert_eq!(
            bounded_assignment.target_id.chars().count(),
            MAX_WORKFLOW_FIELD_CHARS
        );
    }

    #[test]
    fn truncate_workflow_text_allows_exact_limit() {
        let value = "x".repeat(MAX_WORKFLOW_FIELD_CHARS);

        assert_eq!(
            truncate_workflow_text(&value, MAX_WORKFLOW_FIELD_CHARS),
            value
        );
    }
}
