// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File classification, extraction log, viewer history, annotation, and
//! evidence relationship operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // File Classification Operations
    // ========================================================================

    /// Insert or update a file classification
    pub fn upsert_classification(&self, record: &DbFileClassification) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO file_classifications (id, file_path, container_path, classification, custom_label, classified_by, classified_at, notes, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET classification = excluded.classification, custom_label = excluded.custom_label, notes = excluded.notes, confidence = excluded.confidence, classified_at = excluded.classified_at",
            params![
                record.id, record.file_path, record.container_path, record.classification,
                record.custom_label, record.classified_by, record.classified_at,
                record.notes, record.confidence,
            ],
        )?;
        Ok(())
    }

    /// Get classifications for a file path
    pub fn get_classifications_for_path(&self, file_path: &str) -> SqlResult<Vec<DbFileClassification>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, classification, custom_label, classified_by, classified_at, notes, confidence
             FROM file_classifications WHERE file_path = ?1 ORDER BY classified_at DESC",
        )?;
        let rows = stmt.query_map(params![file_path], |row| {
            Ok(DbFileClassification {
                id: row.get(0)?,
                file_path: row.get(1)?,
                container_path: row.get(2)?,
                classification: row.get(3)?,
                custom_label: row.get(4)?,
                classified_by: row.get(5)?,
                classified_at: row.get(6)?,
                notes: row.get(7)?,
                confidence: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    /// Get all classifications grouped by type
    pub fn get_all_classifications(&self) -> SqlResult<Vec<DbFileClassification>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, classification, custom_label, classified_by, classified_at, notes, confidence
             FROM file_classifications ORDER BY classification, classified_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DbFileClassification {
                id: row.get(0)?,
                file_path: row.get(1)?,
                container_path: row.get(2)?,
                classification: row.get(3)?,
                custom_label: row.get(4)?,
                classified_by: row.get(5)?,
                classified_at: row.get(6)?,
                notes: row.get(7)?,
                confidence: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    /// Delete a classification by ID
    pub fn delete_classification(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM file_classifications WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // Extraction Log Operations
    // ========================================================================

    /// Insert an extraction log entry
    pub fn insert_extraction(&self, record: &DbExtractionRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO extraction_log (id, container_path, entry_path, output_path, extracted_by, extracted_at, entry_size, purpose, hash_value, hash_algorithm, status, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.id, record.container_path, record.entry_path, record.output_path,
                record.extracted_by, record.extracted_at, record.entry_size, record.purpose,
                record.hash_value, record.hash_algorithm, record.status, record.error,
            ],
        )?;
        Ok(())
    }

    /// Get extraction log entries for a container
    pub fn get_extractions_for_container(&self, container_path: &str) -> SqlResult<Vec<DbExtractionRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_path, entry_path, output_path, extracted_by, extracted_at, entry_size, purpose, hash_value, hash_algorithm, status, error
             FROM extraction_log WHERE container_path = ?1 ORDER BY extracted_at DESC",
        )?;
        let rows = stmt.query_map(params![container_path], |row| {
            Ok(DbExtractionRecord {
                id: row.get(0)?,
                container_path: row.get(1)?,
                entry_path: row.get(2)?,
                output_path: row.get(3)?,
                extracted_by: row.get(4)?,
                extracted_at: row.get(5)?,
                entry_size: row.get(6)?,
                purpose: row.get(7)?,
                hash_value: row.get(8)?,
                hash_algorithm: row.get(9)?,
                status: row.get(10)?,
                error: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    /// Get all extraction records, most recent first
    pub fn get_all_extractions(&self, limit: Option<i64>) -> SqlResult<Vec<DbExtractionRecord>> {
        let conn = self.conn.lock();
        let sql = format!(
            "SELECT id, container_path, entry_path, output_path, extracted_by, extracted_at, entry_size, purpose, hash_value, hash_algorithm, status, error
             FROM extraction_log ORDER BY extracted_at DESC{}",
            limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default()
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(DbExtractionRecord {
                id: row.get(0)?,
                container_path: row.get(1)?,
                entry_path: row.get(2)?,
                output_path: row.get(3)?,
                extracted_by: row.get(4)?,
                extracted_at: row.get(5)?,
                entry_size: row.get(6)?,
                purpose: row.get(7)?,
                hash_value: row.get(8)?,
                hash_algorithm: row.get(9)?,
                status: row.get(10)?,
                error: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    // ========================================================================
    // Viewer History Operations
    // ========================================================================

    /// Insert a viewer history entry
    pub fn insert_viewer_history(&self, entry: &DbViewerHistoryEntry) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO viewer_history (id, file_path, container_path, viewer_type, viewed_by, opened_at, closed_at, duration_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id, entry.file_path, entry.container_path, entry.viewer_type,
                entry.viewed_by, entry.opened_at, entry.closed_at, entry.duration_seconds,
            ],
        )?;
        Ok(())
    }

    /// Update viewer history (set closed_at and duration)
    pub fn update_viewer_history_close(&self, id: &str, closed_at: &str, duration_seconds: Option<i64>) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE viewer_history SET closed_at = ?1, duration_seconds = ?2 WHERE id = ?3",
            params![closed_at, duration_seconds, id],
        )?;
        Ok(())
    }

    /// Get recent viewer history
    pub fn get_viewer_history(&self, limit: Option<i64>) -> SqlResult<Vec<DbViewerHistoryEntry>> {
        let conn = self.conn.lock();
        let sql = format!(
            "SELECT id, file_path, container_path, viewer_type, viewed_by, opened_at, closed_at, duration_seconds
             FROM viewer_history ORDER BY opened_at DESC{}",
            limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default()
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(DbViewerHistoryEntry {
                id: row.get(0)?,
                file_path: row.get(1)?,
                container_path: row.get(2)?,
                viewer_type: row.get(3)?,
                viewed_by: row.get(4)?,
                opened_at: row.get(5)?,
                closed_at: row.get(6)?,
                duration_seconds: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    // ========================================================================
    // Annotation Operations
    // ========================================================================

    /// Insert a new annotation
    pub fn insert_annotation(&self, ann: &DbAnnotation) -> SqlResult<()> {
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
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at
             FROM annotations ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], Self::map_annotation)?;
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

    // ========================================================================
    // Evidence Relationship Operations
    // ========================================================================

    /// Insert an evidence relationship
    pub fn insert_relationship(&self, rel: &DbEvidenceRelationship) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO evidence_relationships (id, source_path, target_path, relationship_type, description, created_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                rel.id, rel.source_path, rel.target_path, rel.relationship_type,
                rel.description, rel.created_by, rel.created_at,
            ],
        )?;
        Ok(())
    }

    /// Get relationships involving a path (as source or target)
    pub fn get_relationships_for_path(&self, path: &str) -> SqlResult<Vec<DbEvidenceRelationship>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, source_path, target_path, relationship_type, description, created_by, created_at
             FROM evidence_relationships WHERE source_path = ?1 OR target_path = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![path], Self::map_relationship)?;
        rows.collect()
    }

    /// Get all relationships
    pub fn get_all_relationships(&self) -> SqlResult<Vec<DbEvidenceRelationship>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, source_path, target_path, relationship_type, description, created_by, created_at
             FROM evidence_relationships ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], Self::map_relationship)?;
        rows.collect()
    }

    /// Delete a relationship by ID
    pub fn delete_relationship(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM evidence_relationships WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Row mapper for DbEvidenceRelationship (7 columns)
    fn map_relationship(row: &rusqlite::Row<'_>) -> rusqlite::Result<DbEvidenceRelationship> {
        Ok(DbEvidenceRelationship {
            id: row.get(0)?,
            source_path: row.get(1)?,
            target_path: row.get(2)?,
            relationship_type: row.get(3)?,
            description: row.get(4)?,
            created_by: row.get(5)?,
            created_at: row.get(6)?,
        })
    }
}
