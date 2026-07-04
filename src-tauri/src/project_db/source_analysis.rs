// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Persisted source-analysis operations.

use super::database::ProjectDatabase;
use super::types::{DbSourceAnalysisCategorySummary, DbSourceAnalysisRecord};
use rusqlite::{params, Result as SqlResult};

const MAX_SOURCE_ANALYSIS_JSON_BYTES: usize = 1024 * 1024;
const MAX_SOURCE_ANALYSIS_PREVIEW_BYTES: usize = 1024 * 1024;
const MAX_SOURCE_ANALYSIS_TEXT_FIELD_BYTES: usize = 16 * 1024;

impl ProjectDatabase {
    /// Insert or replace a bounded source-analysis record.
    pub fn upsert_source_analysis(&self, record: &DbSourceAnalysisRecord) -> SqlResult<()> {
        validate_source_analysis_record(record)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO source_analyses (
                id, evidence_file_id, source_id, source_ref_json, total_size, offset,
                bytes_analyzed, magic_hex, signature_count, primary_signature,
                primary_mime_type, primary_category, entropy, printable_ratio, is_likely_text,
                ascii_preview, signatures_json, entropy_windows_json, histogram_json,
                indicators_json, analyzed_at, analyzer
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                       ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
            params![
                record.id,
                record.evidence_file_id,
                record.source_id,
                record.source_ref_json,
                record.total_size,
                record.offset,
                record.bytes_analyzed,
                record.magic_hex,
                record.signature_count,
                record.primary_signature,
                record.primary_mime_type,
                record.primary_category,
                record.entropy,
                record.printable_ratio,
                record.is_likely_text,
                record.ascii_preview,
                record.signatures_json,
                record.entropy_windows_json,
                record.histogram_json,
                record.indicators_json,
                record.analyzed_at,
                record.analyzer,
            ],
        )?;
        Ok(())
    }

    /// Get one source-analysis record by ID.
    pub fn get_source_analysis(&self, id: &str) -> SqlResult<Option<DbSourceAnalysisRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, evidence_file_id, source_id, source_ref_json, total_size, offset,
                    bytes_analyzed, magic_hex, signature_count, primary_signature,
                    primary_mime_type, primary_category, entropy, printable_ratio, is_likely_text,
                    ascii_preview, signatures_json, entropy_windows_json, histogram_json,
                    indicators_json, analyzed_at, analyzer
             FROM source_analyses WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_source_analysis(row)?))
        } else {
            Ok(None)
        }
    }

    /// List source-analysis records across the project.
    pub fn list_source_analyses(
        &self,
        limit: Option<i64>,
    ) -> SqlResult<Vec<DbSourceAnalysisRecord>> {
        let conn = self.conn.lock();
        let limit = limit.unwrap_or(10_000).clamp(1, 100_000);
        let mut stmt = conn.prepare(
            "SELECT id, evidence_file_id, source_id, source_ref_json, total_size, offset,
                    bytes_analyzed, magic_hex, signature_count, primary_signature,
                    primary_mime_type, primary_category, entropy, printable_ratio, is_likely_text,
                    ascii_preview, signatures_json, entropy_windows_json, histogram_json,
                    indicators_json, analyzed_at, analyzer
             FROM source_analyses ORDER BY analyzed_at DESC, source_id LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], row_to_source_analysis)?;
        rows.collect()
    }

    /// Summarize persisted source analyses by primary signature category.
    pub fn summarize_source_analyses_by_category(
        &self,
    ) -> SqlResult<Vec<DbSourceAnalysisCategorySummary>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT COALESCE(primary_category, 'unknown') AS category,
                    COUNT(*) AS count,
                    COUNT(DISTINCT evidence_file_id) AS evidence_file_count,
                    COALESCE(AVG(entropy), 0.0) AS avg_entropy,
                    COALESCE(SUM(CASE WHEN is_likely_text THEN 1 ELSE 0 END), 0) AS text_like_count,
                    MAX(analyzed_at) AS latest_analyzed_at
             FROM source_analyses
             GROUP BY COALESCE(primary_category, 'unknown')
             ORDER BY category",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbSourceAnalysisCategorySummary {
                category: row.get(0)?,
                count: row.get(1)?,
                evidence_file_count: row.get(2)?,
                avg_entropy: row.get(3)?,
                text_like_count: row.get(4)?,
                latest_analyzed_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }
}

fn validate_source_analysis_record(record: &DbSourceAnalysisRecord) -> SqlResult<()> {
    validate_required_text_field("id", &record.id, &record.id)?;
    validate_optional_text_field(
        "evidence_file_id",
        record.evidence_file_id.as_deref(),
        &record.id,
    )?;
    validate_required_text_field("source_id", &record.source_id, &record.id)?;
    validate_text_field_size("magic_hex", &record.magic_hex, &record.id)?;
    validate_optional_text_field(
        "primary_signature",
        record.primary_signature.as_deref(),
        &record.id,
    )?;
    validate_optional_text_field(
        "primary_mime_type",
        record.primary_mime_type.as_deref(),
        &record.id,
    )?;
    validate_optional_text_field(
        "primary_category",
        record.primary_category.as_deref(),
        &record.id,
    )?;
    validate_required_text_field("analyzed_at", &record.analyzed_at, &record.id)?;
    validate_required_text_field("analyzer", &record.analyzer, &record.id)?;

    validate_nonnegative_i64("total_size", record.total_size, &record.id)?;
    validate_nonnegative_i64("offset", record.offset, &record.id)?;
    validate_nonnegative_i64("bytes_analyzed", record.bytes_analyzed, &record.id)?;
    validate_nonnegative_i64("signature_count", record.signature_count, &record.id)?;
    let analysis_end = record
        .offset
        .checked_add(record.bytes_analyzed)
        .ok_or_else(|| {
            source_analysis_validation_error(format!(
                "Source analysis byte range overflows for {}: offset {} + {} bytes",
                record_id_for_error(&record.id),
                record.offset,
                record.bytes_analyzed
            ))
        })?;
    if analysis_end > record.total_size {
        return Err(source_analysis_validation_error(format!(
            "Source analysis byte range exceeds source size for {}: offset {} + {} bytes > {} bytes",
            record_id_for_error(&record.id),
            record.offset,
            record.bytes_analyzed,
            record.total_size
        )));
    }

    validate_ratio("entropy", record.entropy, 0.0, 8.0, &record.id)?;
    validate_ratio(
        "printable_ratio",
        record.printable_ratio,
        0.0,
        1.0,
        &record.id,
    )?;

    validate_json_field("source_ref_json", &record.source_ref_json)?;
    if record.source_ref_json.len() > MAX_SOURCE_ANALYSIS_JSON_BYTES {
        return Err(source_analysis_validation_error(format!(
            "Source analysis source_ref_json exceeds {MAX_SOURCE_ANALYSIS_JSON_BYTES} bytes for {}",
            record_id_for_error(&record.id)
        )));
    }

    validate_optional_json_field(
        "signatures_json",
        record.signatures_json.as_deref(),
        &record.id,
    )?;
    validate_optional_json_field(
        "entropy_windows_json",
        record.entropy_windows_json.as_deref(),
        &record.id,
    )?;
    validate_optional_json_field(
        "histogram_json",
        record.histogram_json.as_deref(),
        &record.id,
    )?;
    validate_optional_json_field(
        "indicators_json",
        record.indicators_json.as_deref(),
        &record.id,
    )?;

    if record
        .ascii_preview
        .as_ref()
        .is_some_and(|preview| preview.len() > MAX_SOURCE_ANALYSIS_PREVIEW_BYTES)
    {
        return Err(source_analysis_validation_error(format!(
            "Source analysis ascii_preview exceeds {MAX_SOURCE_ANALYSIS_PREVIEW_BYTES} bytes for {}",
            record_id_for_error(&record.id)
        )));
    }

    Ok(())
}

fn validate_required_text_field(field_name: &str, value: &str, record_id: &str) -> SqlResult<()> {
    if value.trim().is_empty() {
        return Err(source_analysis_validation_error(format!(
            "Source analysis {field_name} cannot be blank for {}",
            record_id_for_error(record_id)
        )));
    }

    validate_text_field_size(field_name, value, record_id)
}

fn validate_optional_text_field(
    field_name: &str,
    value: Option<&str>,
    record_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.trim().is_empty() {
        return Err(source_analysis_validation_error(format!(
            "Source analysis {field_name} cannot be blank for {}",
            record_id_for_error(record_id)
        )));
    }

    validate_text_field_size(field_name, value, record_id)
}

fn validate_text_field_size(field_name: &str, value: &str, record_id: &str) -> SqlResult<()> {
    if value.len() > MAX_SOURCE_ANALYSIS_TEXT_FIELD_BYTES {
        return Err(source_analysis_validation_error(format!(
            "Source analysis {field_name} exceeds {MAX_SOURCE_ANALYSIS_TEXT_FIELD_BYTES} bytes for {}",
            record_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn validate_nonnegative_i64(field_name: &str, value: i64, record_id: &str) -> SqlResult<()> {
    if value < 0 {
        return Err(source_analysis_validation_error(format!(
            "Source analysis {field_name} cannot be negative for {}: {value}",
            record_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn validate_ratio(
    field_name: &str,
    value: f64,
    min: f64,
    max: f64,
    record_id: &str,
) -> SqlResult<()> {
    if !value.is_finite() || value < min || value > max {
        return Err(source_analysis_validation_error(format!(
            "Source analysis {field_name} must be finite and between {min} and {max} for {}: {value}",
            record_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn validate_json_field(field_name: &str, value: &str) -> SqlResult<()> {
    serde_json::from_str::<serde_json::Value>(value)
        .map(|_| ())
        .map_err(|e| {
            source_analysis_validation_error(format!("Invalid source analysis {field_name}: {e}"))
        })
}

fn validate_optional_json_field(
    field_name: &str,
    value: Option<&str>,
    record_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    validate_json_field(field_name, value)?;
    if value.len() > MAX_SOURCE_ANALYSIS_JSON_BYTES {
        return Err(source_analysis_validation_error(format!(
            "Source analysis {field_name} exceeds {MAX_SOURCE_ANALYSIS_JSON_BYTES} bytes for {}",
            record_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn record_id_for_error(record_id: &str) -> &str {
    if record_id.trim().is_empty() {
        "<blank>"
    } else {
        record_id
    }
}

fn source_analysis_validation_error(message: String) -> rusqlite::Error {
    rusqlite::Error::InvalidParameterName(message)
}

fn row_to_source_analysis(row: &rusqlite::Row<'_>) -> SqlResult<DbSourceAnalysisRecord> {
    Ok(DbSourceAnalysisRecord {
        id: row.get(0)?,
        evidence_file_id: row.get(1)?,
        source_id: row.get(2)?,
        source_ref_json: row.get(3)?,
        total_size: row.get(4)?,
        offset: row.get(5)?,
        bytes_analyzed: row.get(6)?,
        magic_hex: row.get(7)?,
        signature_count: row.get(8)?,
        primary_signature: row.get(9)?,
        primary_mime_type: row.get(10)?,
        primary_category: row.get(11)?,
        entropy: row.get(12)?,
        printable_ratio: row.get(13)?,
        is_likely_text: row.get(14)?,
        ascii_preview: row.get(15)?,
        signatures_json: row.get(16)?,
        entropy_windows_json: row.get(17)?,
        histogram_json: row.get(18)?,
        indicators_json: row.get(19)?,
        analyzed_at: row.get(20)?,
        analyzer: row.get(21)?,
    })
}
