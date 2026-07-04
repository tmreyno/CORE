// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Normalized artifact persistence operations.

use super::database::ProjectDatabase;
use super::types::{
    DbArtifactCategorySummary, DbArtifactEvidenceSummary, DbArtifactExtractorSummary,
    DbNormalizedArtifact,
};
use rusqlite::{params, Result as SqlResult};

const MAX_ARTIFACT_JSON_BYTES: usize = 1024 * 1024;
const MAX_ARTIFACT_PREVIEW_BYTES: usize = 1024 * 1024;
const MAX_ARTIFACT_TEXT_FIELD_BYTES: usize = 16 * 1024;

impl ProjectDatabase {
    /// Insert or replace a normalized artifact record.
    pub fn upsert_artifact(&self, artifact: &DbNormalizedArtifact) -> SqlResult<()> {
        validate_artifact_record(artifact)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO artifacts (
                id, evidence_file_id, source_id, source_ref_json, name, extension, size,
                mime_type, type_description, category, confidence, is_text, content_preview,
                metadata_json, extracted_at, extractor
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                artifact.id,
                artifact.evidence_file_id,
                artifact.source_id,
                artifact.source_ref_json,
                artifact.name,
                artifact.extension,
                artifact.size,
                artifact.mime_type,
                artifact.type_description,
                artifact.category,
                artifact.confidence,
                artifact.is_text,
                artifact.content_preview,
                artifact.metadata_json,
                artifact.extracted_at,
                artifact.extractor,
            ],
        )?;
        Ok(())
    }

    /// Get a normalized artifact by ID.
    pub fn get_artifact(&self, id: &str) -> SqlResult<Option<DbNormalizedArtifact>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, evidence_file_id, source_id, source_ref_json, name, extension, size,
                    mime_type, type_description, category, confidence, is_text, content_preview,
                    metadata_json, extracted_at, extractor
             FROM artifacts WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_artifact(row)?))
        } else {
            Ok(None)
        }
    }

    /// List normalized artifacts across the project.
    pub fn list_artifacts(&self, limit: Option<i64>) -> SqlResult<Vec<DbNormalizedArtifact>> {
        let conn = self.conn.lock();
        let limit = limit.unwrap_or(10_000).clamp(1, 100_000);
        let mut stmt = conn.prepare(
            "SELECT id, evidence_file_id, source_id, source_ref_json, name, extension, size,
                    mime_type, type_description, category, confidence, is_text, content_preview,
                    metadata_json, extracted_at, extractor
             FROM artifacts ORDER BY extracted_at DESC, name LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], row_to_artifact)?;
        rows.collect()
    }

    /// List normalized artifacts for a specific evidence file.
    pub fn list_artifacts_for_evidence(
        &self,
        evidence_file_id: &str,
    ) -> SqlResult<Vec<DbNormalizedArtifact>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, evidence_file_id, source_id, source_ref_json, name, extension, size,
                    mime_type, type_description, category, confidence, is_text, content_preview,
                    metadata_json, extracted_at, extractor
             FROM artifacts WHERE evidence_file_id = ?1 ORDER BY name",
        )?;

        let rows = stmt.query_map(params![evidence_file_id], row_to_artifact)?;
        rows.collect()
    }

    /// List normalized artifacts by category.
    pub fn list_artifacts_by_category(
        &self,
        category: &str,
        limit: Option<i64>,
    ) -> SqlResult<Vec<DbNormalizedArtifact>> {
        let conn = self.conn.lock();
        let limit = limit.unwrap_or(500).clamp(1, 10_000);
        let mut stmt = conn.prepare(
            "SELECT id, evidence_file_id, source_id, source_ref_json, name, extension, size,
                    mime_type, type_description, category, confidence, is_text, content_preview,
                    metadata_json, extracted_at, extractor
             FROM artifacts WHERE category = ?1 ORDER BY extracted_at DESC LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![category, limit], row_to_artifact)?;
        rows.collect()
    }

    /// Summarize normalized artifacts by category.
    pub fn summarize_artifacts_by_category(&self) -> SqlResult<Vec<DbArtifactCategorySummary>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT category,
                    COUNT(*) AS count,
                    CASE
                        WHEN TOTAL(size) >= 9223372036854775807 THEN 9223372036854775807
                        ELSE CAST(TOTAL(size) AS INTEGER)
                    END AS total_size,
                    COALESCE(SUM(CASE WHEN is_text THEN 1 ELSE 0 END), 0) AS text_count,
                    MAX(extracted_at) AS latest_extracted_at
             FROM artifacts
             GROUP BY category
             ORDER BY category",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbArtifactCategorySummary {
                category: row.get(0)?,
                count: row.get(1)?,
                total_size: row.get(2)?,
                text_count: row.get(3)?,
                latest_extracted_at: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    /// Summarize normalized artifacts by evidence file.
    pub fn summarize_artifacts_by_evidence(&self) -> SqlResult<Vec<DbArtifactEvidenceSummary>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT evidence_file_id,
                    COUNT(*) AS count,
                    CASE
                        WHEN TOTAL(size) >= 9223372036854775807 THEN 9223372036854775807
                        ELSE CAST(TOTAL(size) AS INTEGER)
                    END AS total_size,
                    COALESCE(SUM(CASE WHEN is_text THEN 1 ELSE 0 END), 0) AS text_count,
                    COUNT(DISTINCT category) AS category_count,
                    MAX(extracted_at) AS latest_extracted_at
             FROM artifacts
             GROUP BY evidence_file_id
             ORDER BY evidence_file_id IS NULL, evidence_file_id",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbArtifactEvidenceSummary {
                evidence_file_id: row.get(0)?,
                count: row.get(1)?,
                total_size: row.get(2)?,
                text_count: row.get(3)?,
                category_count: row.get(4)?,
                latest_extracted_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    /// Summarize normalized artifacts by extractor engine.
    pub fn summarize_artifacts_by_extractor(&self) -> SqlResult<Vec<DbArtifactExtractorSummary>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT extractor,
                    COUNT(*) AS count,
                    CASE
                        WHEN TOTAL(size) >= 9223372036854775807 THEN 9223372036854775807
                        ELSE CAST(TOTAL(size) AS INTEGER)
                    END AS total_size,
                    COALESCE(SUM(CASE WHEN is_text THEN 1 ELSE 0 END), 0) AS text_count,
                    COUNT(DISTINCT category) AS category_count,
                    COUNT(DISTINCT evidence_file_id) AS evidence_file_count,
                    MAX(extracted_at) AS latest_extracted_at
             FROM artifacts
             GROUP BY extractor
             ORDER BY extractor",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbArtifactExtractorSummary {
                extractor: row.get(0)?,
                count: row.get(1)?,
                total_size: row.get(2)?,
                text_count: row.get(3)?,
                category_count: row.get(4)?,
                evidence_file_count: row.get(5)?,
                latest_extracted_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }
}

fn validate_artifact_record(artifact: &DbNormalizedArtifact) -> SqlResult<()> {
    validate_required_text_field("id", &artifact.id, &artifact.id)?;
    validate_optional_text_field(
        "evidence_file_id",
        artifact.evidence_file_id.as_deref(),
        &artifact.id,
    )?;
    validate_required_text_field("source_id", &artifact.source_id, &artifact.id)?;
    validate_required_text_field("name", &artifact.name, &artifact.id)?;
    validate_optional_text_field("extension", artifact.extension.as_deref(), &artifact.id)?;
    validate_optional_text_field("mime_type", artifact.mime_type.as_deref(), &artifact.id)?;
    validate_required_text_field("type_description", &artifact.type_description, &artifact.id)?;
    validate_required_text_field("category", &artifact.category, &artifact.id)?;
    validate_required_text_field("confidence", &artifact.confidence, &artifact.id)?;
    validate_required_text_field("extracted_at", &artifact.extracted_at, &artifact.id)?;
    validate_required_text_field("extractor", &artifact.extractor, &artifact.id)?;

    if artifact.size < 0 {
        return Err(artifact_validation_error(format!(
            "Artifact size cannot be negative for {}: {}",
            artifact.id, artifact.size
        )));
    }

    validate_json_field("source_ref_json", &artifact.source_ref_json)?;
    if artifact.source_ref_json.len() > MAX_ARTIFACT_JSON_BYTES {
        return Err(artifact_validation_error(format!(
            "Artifact source_ref_json exceeds {MAX_ARTIFACT_JSON_BYTES} bytes for {}",
            artifact.id
        )));
    }

    if let Some(metadata_json) = &artifact.metadata_json {
        validate_json_field("metadata_json", metadata_json)?;
        if metadata_json.len() > MAX_ARTIFACT_JSON_BYTES {
            return Err(artifact_validation_error(format!(
                "Artifact metadata_json exceeds {MAX_ARTIFACT_JSON_BYTES} bytes for {}",
                artifact.id
            )));
        }
    }

    if artifact
        .content_preview
        .as_ref()
        .is_some_and(|preview| preview.len() > MAX_ARTIFACT_PREVIEW_BYTES)
    {
        return Err(artifact_validation_error(format!(
            "Artifact content_preview exceeds {MAX_ARTIFACT_PREVIEW_BYTES} bytes for {}",
            artifact.id
        )));
    }

    Ok(())
}

fn validate_required_text_field(field_name: &str, value: &str, artifact_id: &str) -> SqlResult<()> {
    if value.trim().is_empty() {
        return Err(artifact_validation_error(format!(
            "Artifact {field_name} cannot be blank for {}",
            artifact_id_for_error(artifact_id)
        )));
    }

    validate_text_field_size(field_name, value, artifact_id)
}

fn validate_optional_text_field(
    field_name: &str,
    value: Option<&str>,
    artifact_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.trim().is_empty() {
        return Err(artifact_validation_error(format!(
            "Artifact {field_name} cannot be blank for {}",
            artifact_id_for_error(artifact_id)
        )));
    }

    validate_text_field_size(field_name, value, artifact_id)
}

fn validate_text_field_size(field_name: &str, value: &str, artifact_id: &str) -> SqlResult<()> {
    if value.len() > MAX_ARTIFACT_TEXT_FIELD_BYTES {
        return Err(artifact_validation_error(format!(
            "Artifact {field_name} exceeds {MAX_ARTIFACT_TEXT_FIELD_BYTES} bytes for {}",
            artifact_id_for_error(artifact_id)
        )));
    }

    Ok(())
}

fn validate_json_field(field_name: &str, value: &str) -> SqlResult<()> {
    serde_json::from_str::<serde_json::Value>(value)
        .map(|_| ())
        .map_err(|e| artifact_validation_error(format!("Invalid artifact {field_name}: {e}")))
}

fn artifact_id_for_error(artifact_id: &str) -> &str {
    if artifact_id.trim().is_empty() {
        "<blank>"
    } else {
        artifact_id
    }
}

fn artifact_validation_error(message: String) -> rusqlite::Error {
    rusqlite::Error::InvalidParameterName(message)
}

fn row_to_artifact(row: &rusqlite::Row<'_>) -> SqlResult<DbNormalizedArtifact> {
    Ok(DbNormalizedArtifact {
        id: row.get(0)?,
        evidence_file_id: row.get(1)?,
        source_id: row.get(2)?,
        source_ref_json: row.get(3)?,
        name: row.get(4)?,
        extension: row.get(5)?,
        size: row.get(6)?,
        mime_type: row.get(7)?,
        type_description: row.get(8)?,
        category: row.get(9)?,
        confidence: row.get(10)?,
        is_text: row.get(11)?,
        content_preview: row.get(12)?,
        metadata_json: row.get(13)?,
        extracted_at: row.get(14)?,
        extractor: row.get(15)?,
    })
}
