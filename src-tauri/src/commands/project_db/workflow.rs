// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for annotations.

use super::with_project_db;
use crate::project_db::DbAnnotation;

const MAX_ANNOTATION_RESPONSE_ROWS: usize = 10_000;
const MAX_ANNOTATION_FIELD_CHARS: usize = 4096;
const MAX_ANNOTATION_CONTENT_CHARS: usize = 16_384;
const ANNOTATION_TRUNCATED_SUFFIX: &str = "... [truncated]";

// =============================================================================
// Annotation Commands
// =============================================================================

/// Insert a new annotation.
#[tauri::command]
pub fn project_db_insert_annotation(
    window: tauri::Window,
    ann: DbAnnotation,
) -> Result<(), String> {
    let ann = bounded_annotation_record(ann);
    with_project_db(window.label(), |db| db.insert_annotation(&ann))
}

/// Update an annotation (label, content, color).
#[tauri::command]
pub fn project_db_update_annotation(
    window: tauri::Window,
    ann: DbAnnotation,
) -> Result<(), String> {
    let ann = bounded_annotation_record(ann);
    with_project_db(window.label(), |db| db.update_annotation(&ann))
}

/// Get annotations for a file.
#[tauri::command]
pub fn project_db_get_annotations_for_path(
    window: tauri::Window,
    file_path: String,
) -> Result<Vec<DbAnnotation>, String> {
    with_project_db(window.label(), |db| db.get_annotations_for_path(&file_path)).map(|items| {
        items
            .into_iter()
            .take(MAX_ANNOTATION_RESPONSE_ROWS)
            .map(bounded_annotation_record)
            .collect()
    })
}

/// Get all annotations.
#[tauri::command]
pub fn project_db_get_all_annotations(window: tauri::Window) -> Result<Vec<DbAnnotation>, String> {
    with_project_db(window.label(), |db| db.get_all_annotations()).map(|items| {
        items
            .into_iter()
            .take(MAX_ANNOTATION_RESPONSE_ROWS)
            .map(bounded_annotation_record)
            .collect()
    })
}

/// Delete an annotation.
#[tauri::command]
pub fn project_db_delete_annotation(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_annotation(&id))
}

fn bounded_annotation_record(mut ann: DbAnnotation) -> DbAnnotation {
    ann.id = truncate_annotation_text(&ann.id, MAX_ANNOTATION_FIELD_CHARS);
    ann.file_path = truncate_annotation_text(&ann.file_path, MAX_ANNOTATION_FIELD_CHARS);
    ann.container_path = ann
        .container_path
        .map(|value| truncate_annotation_text(&value, MAX_ANNOTATION_FIELD_CHARS));
    ann.annotation_type =
        truncate_annotation_text(&ann.annotation_type, MAX_ANNOTATION_FIELD_CHARS);
    ann.label = truncate_annotation_text(&ann.label, MAX_ANNOTATION_FIELD_CHARS);
    ann.content = ann
        .content
        .map(|value| truncate_annotation_text(&value, MAX_ANNOTATION_CONTENT_CHARS));
    ann.color = ann
        .color
        .map(|value| truncate_annotation_text(&value, MAX_ANNOTATION_FIELD_CHARS));
    ann.created_by = truncate_annotation_text(&ann.created_by, MAX_ANNOTATION_FIELD_CHARS);
    ann.created_at = truncate_annotation_text(&ann.created_at, MAX_ANNOTATION_FIELD_CHARS);
    ann.modified_at = truncate_annotation_text(&ann.modified_at, MAX_ANNOTATION_FIELD_CHARS);
    ann
}

fn truncate_annotation_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = ANNOTATION_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + ANNOTATION_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(ANNOTATION_TRUNCATED_SUFFIX);
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_annotation(content: String) -> DbAnnotation {
        DbAnnotation {
            id: "ann-1".to_string(),
            file_path: "f".repeat(MAX_ANNOTATION_FIELD_CHARS + 16),
            container_path: Some("/case/container.ad1".to_string()),
            annotation_type: "hex-review".to_string(),
            offset_start: Some(0),
            offset_end: Some(16),
            line_start: None,
            line_end: None,
            label: "Magic Bytes".to_string(),
            content: Some(content),
            color: Some("#38bdf8".to_string()),
            created_by: "analyst".to_string(),
            created_at: "2026-02-16T10:00:00Z".to_string(),
            modified_at: "2026-02-16T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn bounded_annotation_record_caps_field_and_content_text() {
        let ann = bounded_annotation_record(make_annotation(
            "é".repeat(MAX_ANNOTATION_CONTENT_CHARS + 32),
        ));

        assert_eq!(ann.file_path.chars().count(), MAX_ANNOTATION_FIELD_CHARS);
        assert!(ann.file_path.ends_with(ANNOTATION_TRUNCATED_SUFFIX));
        let content = ann.content.as_deref().unwrap();
        assert_eq!(content.chars().count(), MAX_ANNOTATION_CONTENT_CHARS);
        assert!(content.ends_with(ANNOTATION_TRUNCATED_SUFFIX));
    }

    #[test]
    fn truncate_annotation_text_allows_exact_limit() {
        let value = "x".repeat(MAX_ANNOTATION_FIELD_CHARS);

        assert_eq!(
            truncate_annotation_text(&value, MAX_ANNOTATION_FIELD_CHARS),
            value
        );
    }
}
