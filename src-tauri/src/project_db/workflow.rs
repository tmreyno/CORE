// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Annotation operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

const MAX_ANNOTATION_TEXT_FIELD_BYTES: usize = 16 * 1024;
const MAX_ANNOTATION_CONTENT_BYTES: usize = 1024 * 1024;

impl ProjectDatabase {
    // ========================================================================
    // Annotation Operations
    // ========================================================================

    /// Insert a new annotation
    pub fn insert_annotation(&self, ann: &DbAnnotation) -> SqlResult<()> {
        validate_annotation_insert(ann)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO annotations (id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                ann.id, ann.file_path, ann.container_path, ann.annotation_type,
                ann.offset_start, ann.offset_end, ann.line_start, ann.line_end,
                ann.label, ann.content, ann.color, ann.created_by,
                ann.created_at, ann.modified_at,
            ],
        )?;
        Ok(())
    }

    /// Update an annotation
    pub fn update_annotation(&self, ann: &DbAnnotation) -> SqlResult<()> {
        validate_annotation_update(ann)?;

        let conn = self.conn.lock();
        conn.execute(
            "UPDATE annotations SET label = ?1, content = ?2, color = ?3, modified_at = ?4 WHERE id = ?5",
            params![ann.label, ann.content, ann.color, ann.modified_at, ann.id],
        )?;
        Ok(())
    }

    /// Get annotations for a file
    pub fn get_annotations_for_path(&self, file_path: &str) -> SqlResult<Vec<DbAnnotation>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at
             FROM annotations WHERE file_path = ?1 ORDER BY COALESCE(offset_start, line_start, 0)",
        )?;
        let rows = stmt.query_map(params![file_path], Self::map_annotation)?;
        rows.collect()
    }

    /// Get all annotations
    pub fn get_all_annotations(&self) -> SqlResult<Vec<DbAnnotation>> {
        self.get_all_annotations_limited(None)
    }

    /// Get all annotations with an optional bounded limit.
    pub fn get_all_annotations_limited(&self, limit: Option<i64>) -> SqlResult<Vec<DbAnnotation>> {
        let conn = self.conn.lock();
        let limit = limit.unwrap_or(10_000).clamp(1, 100_000);
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at
             FROM annotations ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], Self::map_annotation)?;
        rows.collect()
    }

    /// Delete an annotation by ID
    pub fn delete_annotation(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM annotations WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Row mapper for DbAnnotation (14 columns)
    fn map_annotation(row: &rusqlite::Row<'_>) -> rusqlite::Result<DbAnnotation> {
        Ok(DbAnnotation {
            id: row.get(0)?,
            file_path: row.get(1)?,
            container_path: row.get(2)?,
            annotation_type: row.get(3)?,
            offset_start: row.get(4)?,
            offset_end: row.get(5)?,
            line_start: row.get(6)?,
            line_end: row.get(7)?,
            label: row.get(8)?,
            content: row.get(9)?,
            color: row.get(10)?,
            created_by: row.get(11)?,
            created_at: row.get(12)?,
            modified_at: row.get(13)?,
        })
    }
}

fn validate_annotation_insert(ann: &DbAnnotation) -> SqlResult<()> {
    validate_required_annotation_text_field("id", &ann.id, &ann.id)?;
    validate_required_annotation_text_field("file_path", &ann.file_path, &ann.id)?;
    validate_optional_annotation_text_field(
        "container_path",
        ann.container_path.as_deref(),
        &ann.id,
    )?;
    validate_required_annotation_text_field("annotation_type", &ann.annotation_type, &ann.id)?;
    validate_annotation_ranges(ann)?;
    validate_annotation_update(ann)?;
    validate_required_annotation_text_field("created_by", &ann.created_by, &ann.id)?;
    validate_required_annotation_text_field("created_at", &ann.created_at, &ann.id)
}

fn validate_annotation_update(ann: &DbAnnotation) -> SqlResult<()> {
    validate_required_annotation_text_field("id", &ann.id, &ann.id)?;
    validate_required_annotation_text_field("label", &ann.label, &ann.id)?;
    validate_optional_annotation_content("content", ann.content.as_deref(), &ann.id)?;
    validate_optional_annotation_text_field("color", ann.color.as_deref(), &ann.id)?;
    validate_required_annotation_text_field("modified_at", &ann.modified_at, &ann.id)
}

fn validate_annotation_ranges(ann: &DbAnnotation) -> SqlResult<()> {
    validate_annotation_range_pair("offset", ann.offset_start, ann.offset_end, &ann.id)?;
    validate_annotation_range_pair("line", ann.line_start, ann.line_end, &ann.id)
}

fn validate_annotation_range_pair(
    range_name: &str,
    start: Option<i64>,
    end: Option<i64>,
    annotation_id: &str,
) -> SqlResult<()> {
    match (start, end) {
        (None, None) => Ok(()),
        (Some(start), Some(end)) => {
            if start < 0 || end < 0 {
                return Err(annotation_validation_error(format!(
                    "Annotation {range_name} range cannot be negative for {}",
                    annotation_id_for_error(annotation_id)
                )));
            }
            if end < start {
                return Err(annotation_validation_error(format!(
                    "Annotation {range_name} range end cannot be before start for {}: {start}..{end}",
                    annotation_id_for_error(annotation_id)
                )));
            }
            Ok(())
        }
        _ => Err(annotation_validation_error(format!(
            "Annotation {range_name} range requires both start and end for {}",
            annotation_id_for_error(annotation_id)
        ))),
    }
}

fn validate_required_annotation_text_field(
    field_name: &str,
    value: &str,
    annotation_id: &str,
) -> SqlResult<()> {
    if value.trim().is_empty() {
        return Err(annotation_validation_error(format!(
            "Annotation {field_name} cannot be blank for {}",
            annotation_id_for_error(annotation_id)
        )));
    }

    validate_annotation_text_field_size(field_name, value, annotation_id)
}

fn validate_optional_annotation_text_field(
    field_name: &str,
    value: Option<&str>,
    annotation_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.trim().is_empty() {
        return Err(annotation_validation_error(format!(
            "Annotation {field_name} cannot be blank for {}",
            annotation_id_for_error(annotation_id)
        )));
    }

    validate_annotation_text_field_size(field_name, value, annotation_id)
}

fn validate_optional_annotation_content(
    field_name: &str,
    value: Option<&str>,
    annotation_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.len() > MAX_ANNOTATION_CONTENT_BYTES {
        return Err(annotation_validation_error(format!(
            "Annotation {field_name} exceeds {MAX_ANNOTATION_CONTENT_BYTES} bytes for {}",
            annotation_id_for_error(annotation_id)
        )));
    }

    Ok(())
}

fn validate_annotation_text_field_size(
    field_name: &str,
    value: &str,
    annotation_id: &str,
) -> SqlResult<()> {
    if value.len() > MAX_ANNOTATION_TEXT_FIELD_BYTES {
        return Err(annotation_validation_error(format!(
            "Annotation {field_name} exceeds {MAX_ANNOTATION_TEXT_FIELD_BYTES} bytes for {}",
            annotation_id_for_error(annotation_id)
        )));
    }

    Ok(())
}

fn annotation_id_for_error(annotation_id: &str) -> &str {
    if annotation_id.trim().is_empty() {
        "<blank>"
    } else {
        annotation_id
    }
}

fn annotation_validation_error(message: String) -> rusqlite::Error {
    rusqlite::Error::InvalidParameterName(message)
}
