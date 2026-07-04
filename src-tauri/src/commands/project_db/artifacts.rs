// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for normalized artifact persistence.

use super::with_project_db;
use crate::commands::artifacts::artifact_extract_source;
use crate::commands::hash::{open_hash_source, HashSourceInput};
use crate::common::{ArtifactExtractionOptions, EvidenceByteSource, NormalizedArtifact};
use crate::project_db::{
    DbArtifactCategorySummary, DbArtifactEvidenceSummary, DbArtifactExtractorSummary,
    DbEvidenceFile, DbNormalizedArtifact,
};
use crate::viewer::document::database_viewer::get_database_info;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

const SQLITE_ARTIFACT_SOURCE_MAX_BYTES: u64 = 512 * 1024 * 1024;
const SQLITE_ARTIFACT_COPY_CHUNK_BYTES: usize = 1024 * 1024;
const SQLITE_METADATA_NAME_LIMIT: usize = 12;
const DEFAULT_ARTIFACT_EXTRACTOR: &str = "core-artifact-extractor";
const MAX_ARTIFACT_EXTRACTOR_CHARS: usize = 128;
const MAX_ARTIFACT_RESPONSE_ROWS: usize = 10_000;
const MAX_ARTIFACT_FIELD_CHARS: usize = 4096;
const MAX_ARTIFACT_PREVIEW_CHARS: usize = 16_384;
const MAX_ARTIFACT_JSON_CHARS: usize = 65_536;
const MAX_ARTIFACT_JSON_DEPTH: usize = 4;
const MAX_ARTIFACT_JSON_ITEMS: usize = 256;
const MAX_ARTIFACT_METADATA_ENTRIES: usize = 96;
const MAX_ARTIFACT_METADATA_VALUE_CHARS: usize = 384;
const ARTIFACT_TRUNCATED_SUFFIX: &str = "... [truncated]";

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbExtractArtifactRequest {
    pub source: HashSourceInput,
    pub options: Option<ArtifactExtractionOptions>,
    pub evidence_file_id: Option<String>,
    pub evidence_file: Option<DbEvidenceFile>,
    pub extractor: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbExtractArtifactResult {
    pub artifact: NormalizedArtifact,
    pub record: DbNormalizedArtifact,
}

/// Insert or replace a normalized artifact record.
#[tauri::command]
pub fn project_db_upsert_artifact(
    window: tauri::Window,
    artifact: DbNormalizedArtifact,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_artifact(&artifact))
}

/// Get a normalized artifact by ID.
#[tauri::command]
pub fn project_db_get_artifact(
    window: tauri::Window,
    id: String,
) -> Result<Option<DbNormalizedArtifact>, String> {
    with_project_db(window.label(), |db| db.get_artifact(&id))
        .map(|artifact| artifact.map(bounded_artifact_record_for_response))
}

/// List normalized artifacts across the active project.
#[tauri::command]
pub fn project_db_list_artifacts(
    window: tauri::Window,
    limit: Option<i64>,
) -> Result<Vec<DbNormalizedArtifact>, String> {
    with_project_db(window.label(), |db| db.list_artifacts(limit)).map(|artifacts| {
        artifacts
            .into_iter()
            .take(MAX_ARTIFACT_RESPONSE_ROWS)
            .map(bounded_artifact_record_for_response)
            .collect()
    })
}

/// List normalized artifacts for an evidence file.
#[tauri::command]
pub fn project_db_list_artifacts_for_evidence(
    window: tauri::Window,
    evidence_file_id: String,
) -> Result<Vec<DbNormalizedArtifact>, String> {
    with_project_db(window.label(), |db| {
        db.list_artifacts_for_evidence(&evidence_file_id)
    })
    .map(|artifacts| {
        artifacts
            .into_iter()
            .take(MAX_ARTIFACT_RESPONSE_ROWS)
            .map(bounded_artifact_record_for_response)
            .collect()
    })
}

/// List normalized artifacts by category.
#[tauri::command]
pub fn project_db_list_artifacts_by_category(
    window: tauri::Window,
    category: String,
    limit: Option<i64>,
) -> Result<Vec<DbNormalizedArtifact>, String> {
    with_project_db(window.label(), |db| {
        db.list_artifacts_by_category(&category, limit)
    })
    .map(|artifacts| {
        artifacts
            .into_iter()
            .take(MAX_ARTIFACT_RESPONSE_ROWS)
            .map(bounded_artifact_record_for_response)
            .collect()
    })
}

/// Summarize normalized artifacts by category.
#[tauri::command]
pub fn project_db_summarize_artifacts_by_category(
    window: tauri::Window,
) -> Result<Vec<DbArtifactCategorySummary>, String> {
    with_project_db(window.label(), |db| db.summarize_artifacts_by_category())
}

/// Summarize normalized artifacts by evidence file.
#[tauri::command]
pub fn project_db_summarize_artifacts_by_evidence(
    window: tauri::Window,
) -> Result<Vec<DbArtifactEvidenceSummary>, String> {
    with_project_db(window.label(), |db| db.summarize_artifacts_by_evidence())
}

/// Summarize normalized artifacts by extractor engine.
#[tauri::command]
pub fn project_db_summarize_artifacts_by_extractor(
    window: tauri::Window,
) -> Result<Vec<DbArtifactExtractorSummary>, String> {
    with_project_db(window.label(), |db| db.summarize_artifacts_by_extractor())
}

/// Extract a normalized artifact from a source and persist it to the active
/// project database.
#[tauri::command]
pub async fn project_db_extract_artifact_source(
    window: tauri::Window,
    request: ProjectDbExtractArtifactRequest,
) -> Result<ProjectDbExtractArtifactResult, String> {
    let ProjectDbExtractArtifactRequest {
        source,
        options,
        evidence_file_id,
        evidence_file,
        extractor,
    } = request;

    let source_for_enrichment = source.clone();
    let resolved_evidence_id = evidence_file
        .as_ref()
        .map(|file| file.id.clone())
        .or(evidence_file_id);
    let mut artifact = artifact_extract_source(source, options).await?;
    enrich_sqlite_artifact_metadata(&source_for_enrichment, &mut artifact).await?;
    let record = normalized_to_db_artifact(
        &artifact,
        resolved_evidence_id,
        normalize_artifact_extractor(extractor),
    )?;

    with_project_db(window.label(), |db| {
        if let Some(file) = &evidence_file {
            db.upsert_evidence_file(file)?;
        }
        db.upsert_artifact(&record)?;
        Ok(())
    })?;

    Ok(ProjectDbExtractArtifactResult { artifact, record })
}

async fn enrich_sqlite_artifact_metadata(
    source: &HashSourceInput,
    artifact: &mut NormalizedArtifact,
) -> Result<(), String> {
    if !is_sqlite_artifact(artifact) {
        return Ok(());
    }

    let source = source.clone();
    let metadata_result =
        tauri::async_runtime::spawn_blocking(move || sqlite_artifact_metadata_from_source(&source))
            .await
            .map_err(|e| format!("SQLite artifact metadata task failed: {e}"))?;

    match metadata_result {
        Ok(metadata) => {
            artifact.metadata.extend(metadata);
        }
        Err(error) => {
            artifact
                .metadata
                .insert("sqlite.schemaStatus".to_string(), "unavailable".to_string());
            artifact.metadata.insert(
                "sqlite.schemaError".to_string(),
                truncate_metadata_value(&error, 180),
            );
        }
    }

    Ok(())
}

fn is_sqlite_artifact(artifact: &NormalizedArtifact) -> bool {
    artifact.category == "database"
        || artifact.mime_type.as_deref() == Some("application/x-sqlite3")
        || matches!(
            artifact.extension.as_deref(),
            Some("db" | "sqlite" | "sqlite3" | "sqlitedb")
        )
}

fn sqlite_artifact_metadata_from_source(
    source: &HashSourceInput,
) -> Result<BTreeMap<String, String>, String> {
    with_sqlite_artifact_source(source, |path, _source_id| {
        let info = get_database_info(path).map_err(|e| e.to_string())?;
        let mut metadata = BTreeMap::new();

        let user_tables: Vec<_> = info
            .tables
            .iter()
            .filter(|table| !table.is_system)
            .collect();
        let system_tables: Vec<_> = info.tables.iter().filter(|table| table.is_system).collect();
        let total_rows = sqlite_total_rows(&info.tables);

        metadata.insert(
            "sqlite.tableCount".to_string(),
            info.tables.len().to_string(),
        );
        metadata.insert("sqlite.viewCount".to_string(), info.views.len().to_string());
        metadata.insert(
            "sqlite.userTableCount".to_string(),
            user_tables.len().to_string(),
        );
        metadata.insert(
            "sqlite.systemTableCount".to_string(),
            system_tables.len().to_string(),
        );
        metadata.insert("sqlite.totalRows".to_string(), total_rows.to_string());
        metadata.insert("sqlite.journalMode".to_string(), info.journal_mode);
        metadata.insert("sqlite.sqliteVersion".to_string(), info.sqlite_version);

        if !info.tables.is_empty() {
            metadata.insert(
                "sqlite.tableNames".to_string(),
                limited_names(info.tables.iter().map(|table| table.name.as_str())),
            );
            metadata.insert("sqlite.tables".to_string(), table_summaries(&info.tables));
        }
        if !info.views.is_empty() {
            metadata.insert(
                "sqlite.viewNames".to_string(),
                limited_names(info.views.iter().map(String::as_str)),
            );
        }
        if let Some(largest) = info.tables.iter().max_by_key(|table| table.row_count) {
            metadata.insert(
                "sqlite.largestTable".to_string(),
                format!("{} ({} rows)", largest.name, largest.row_count),
            );
        }

        Ok(metadata)
    })
}

fn with_sqlite_artifact_source<T>(
    source: &HashSourceInput,
    operation: impl FnOnce(&Path, String) -> Result<T, String>,
) -> Result<T, String> {
    let byte_source = open_hash_source(source)?;
    let source_id = byte_source.source_ref().display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    if size > SQLITE_ARTIFACT_SOURCE_MAX_BYTES {
        return Err(format!(
            "SQLite artifact source is too large for schema extraction: {size} bytes > {SQLITE_ARTIFACT_SOURCE_MAX_BYTES} bytes"
        ));
    }

    let mut temp = tempfile::Builder::new()
        .prefix("core-ffx-artifact-db-")
        .suffix(".sqlite")
        .tempfile()
        .map_err(|e| format!("Failed to create temporary SQLite artifact copy: {e}"))?;
    copy_sqlite_artifact_source(byte_source.as_ref(), size, &mut temp)?;
    temp.flush()
        .map_err(|e| format!("Failed to flush temporary SQLite artifact copy: {e}"))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| format!("Failed to sync temporary SQLite artifact copy: {e}"))?;

    operation(temp.path(), source_id)
}

fn copy_sqlite_artifact_source(
    byte_source: &dyn EvidenceByteSource,
    expected_size: u64,
    writer: &mut impl Write,
) -> Result<(), String> {
    let source_id = byte_source.source_ref().display_id();
    let mut offset = 0u64;

    while offset < expected_size {
        let remaining = expected_size - offset;
        let read_size = remaining.min(SQLITE_ARTIFACT_COPY_CHUNK_BYTES as u64) as usize;
        let chunk = byte_source.read_range(offset, read_size).map_err(|e| {
            format!("Failed to read SQLite artifact source {source_id} at offset {offset}: {e}")
        })?;

        if chunk.is_empty() {
            return Err(format!(
                "Short read materializing SQLite artifact source {source_id}: expected {expected_size} bytes but read {offset} bytes"
            ));
        }
        if chunk.len() as u64 > remaining {
            return Err(format!(
                "Invalid oversized read materializing SQLite artifact source {source_id}: {} bytes returned with {remaining} bytes remaining",
                chunk.len()
            ));
        }

        writer.write_all(&chunk).map_err(|e| {
            format!("Failed to write SQLite artifact source {source_id} at offset {offset}: {e}")
        })?;
        offset = checked_sqlite_copy_offset_add(offset, chunk.len(), &source_id)?;
    }

    Ok(())
}

fn checked_sqlite_copy_offset_add(
    offset: u64,
    bytes_read: usize,
    source_id: &str,
) -> Result<u64, String> {
    let bytes_read = u64::try_from(bytes_read).map_err(|_| {
        format!(
            "SQLite artifact source {source_id} returned a chunk length that does not fit in u64"
        )
    })?;
    offset.checked_add(bytes_read).ok_or_else(|| {
        format!(
            "SQLite artifact copy offset overflow for {source_id}: offset {offset} + {bytes_read} bytes"
        )
    })
}

fn table_summaries(tables: &[crate::viewer::document::database_viewer::TableSummary]) -> String {
    let mut values: Vec<String> = tables
        .iter()
        .take(SQLITE_METADATA_NAME_LIMIT)
        .map(|table| {
            format!(
                "{} ({} rows, {} cols)",
                table.name, table.row_count, table.column_count
            )
        })
        .collect();
    if tables.len() > SQLITE_METADATA_NAME_LIMIT {
        values.push(format!(
            "{} more table(s)",
            tables.len() - SQLITE_METADATA_NAME_LIMIT
        ));
    }
    values.join(", ")
}

fn sqlite_total_rows(tables: &[crate::viewer::document::database_viewer::TableSummary]) -> i64 {
    tables.iter().fold(0i64, |total, table| {
        total.saturating_add(table.row_count.max(0))
    })
}

fn limited_names<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let values: Vec<&str> = names.take(SQLITE_METADATA_NAME_LIMIT).collect();
    values.join(", ")
}

fn truncate_metadata_value(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("...");
    truncated
}

fn normalized_to_db_artifact(
    artifact: &NormalizedArtifact,
    evidence_file_id: Option<String>,
    extractor: String,
) -> Result<DbNormalizedArtifact, String> {
    let extractor = normalize_artifact_extractor(Some(extractor));
    let source_ref_json = serde_json::to_string(&artifact.source_ref)
        .map_err(|e| format!("Failed to serialize artifact source ref: {e}"))?;
    let metadata = bounded_artifact_metadata(&artifact.metadata);
    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| format!("Failed to serialize artifact metadata: {e}"))?;

    let record = DbNormalizedArtifact {
        id: truncate_chars_with_suffix(&artifact.id, MAX_ARTIFACT_FIELD_CHARS),
        evidence_file_id: evidence_file_id
            .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS)),
        source_id: truncate_chars_with_suffix(&artifact.source_id, MAX_ARTIFACT_FIELD_CHARS),
        source_ref_json: bounded_artifact_json_text(&source_ref_json, MAX_ARTIFACT_JSON_CHARS),
        name: truncate_chars_with_suffix(&artifact.name, MAX_ARTIFACT_FIELD_CHARS),
        extension: artifact
            .extension
            .as_ref()
            .map(|value| truncate_chars_with_suffix(value, MAX_ARTIFACT_FIELD_CHARS)),
        size: artifact_size_to_i64(artifact.size)?,
        mime_type: artifact
            .mime_type
            .as_ref()
            .map(|value| truncate_chars_with_suffix(value, MAX_ARTIFACT_FIELD_CHARS)),
        type_description: truncate_chars_with_suffix(
            &artifact.type_description,
            MAX_ARTIFACT_FIELD_CHARS,
        ),
        category: truncate_chars_with_suffix(&artifact.category, MAX_ARTIFACT_FIELD_CHARS),
        confidence: truncate_chars_with_suffix(&artifact.confidence, MAX_ARTIFACT_FIELD_CHARS),
        is_text: artifact.is_text,
        content_preview: artifact
            .content_preview
            .as_ref()
            .map(|value| truncate_chars_with_suffix(value, MAX_ARTIFACT_PREVIEW_CHARS)),
        metadata_json: Some(bounded_artifact_json_text(
            &metadata_json,
            MAX_ARTIFACT_JSON_CHARS,
        )),
        extracted_at: chrono::Utc::now().to_rfc3339(),
        extractor,
    };

    Ok(record)
}

fn artifact_size_to_i64(size: u64) -> Result<i64, String> {
    i64::try_from(size).map_err(|_| format!("Artifact size exceeds project DB range: {size} bytes"))
}

fn normalize_artifact_extractor(extractor: Option<String>) -> String {
    let value = extractor
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_ARTIFACT_EXTRACTOR);

    truncate_chars(value, MAX_ARTIFACT_EXTRACTOR_CHARS)
}

fn bounded_artifact_record_for_response(
    mut artifact: DbNormalizedArtifact,
) -> DbNormalizedArtifact {
    artifact.id = truncate_chars_with_suffix(&artifact.id, MAX_ARTIFACT_FIELD_CHARS);
    artifact.evidence_file_id = artifact
        .evidence_file_id
        .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS));
    artifact.source_id = truncate_chars_with_suffix(&artifact.source_id, MAX_ARTIFACT_FIELD_CHARS);
    artifact.source_ref_json =
        bounded_artifact_json_text(&artifact.source_ref_json, MAX_ARTIFACT_JSON_CHARS);
    artifact.name = truncate_chars_with_suffix(&artifact.name, MAX_ARTIFACT_FIELD_CHARS);
    artifact.extension = artifact
        .extension
        .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS));
    artifact.mime_type = artifact
        .mime_type
        .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS));
    artifact.type_description =
        truncate_chars_with_suffix(&artifact.type_description, MAX_ARTIFACT_FIELD_CHARS);
    artifact.category = truncate_chars_with_suffix(&artifact.category, MAX_ARTIFACT_FIELD_CHARS);
    artifact.confidence =
        truncate_chars_with_suffix(&artifact.confidence, MAX_ARTIFACT_FIELD_CHARS);
    artifact.content_preview = artifact
        .content_preview
        .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_PREVIEW_CHARS));
    artifact.metadata_json = artifact
        .metadata_json
        .map(|value| bounded_artifact_json_text(&value, MAX_ARTIFACT_JSON_CHARS));
    artifact.extracted_at =
        truncate_chars_with_suffix(&artifact.extracted_at, MAX_ARTIFACT_FIELD_CHARS);
    artifact.extractor = truncate_chars_with_suffix(&artifact.extractor, MAX_ARTIFACT_FIELD_CHARS);
    artifact
}

fn bounded_artifact_metadata(metadata: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    metadata
        .iter()
        .take(MAX_ARTIFACT_METADATA_ENTRIES)
        .map(|(key, value)| {
            (
                truncate_chars_with_suffix(key, MAX_ARTIFACT_FIELD_CHARS),
                truncate_chars_with_suffix(value, MAX_ARTIFACT_METADATA_VALUE_CHARS),
            )
        })
        .collect()
}

fn truncate_chars_with_suffix(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = ARTIFACT_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + ARTIFACT_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(ARTIFACT_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_artifact_json_text(value: &str, max_chars: usize) -> String {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return if value.chars().count() <= max_chars {
            value.to_string()
        } else {
            truncate_chars_with_suffix(value, max_chars)
        };
    };
    let bounded = bounded_artifact_json_value(json, 0);
    let Ok(serialized) = serde_json::to_string(&bounded) else {
        return "{\"truncated\":true}".to_string();
    };

    if serialized.chars().count() <= max_chars {
        serialized
    } else {
        "{\"truncated\":true}".to_string()
    }
}

fn bounded_artifact_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_ARTIFACT_JSON_DEPTH {
        return serde_json::Value::String(ARTIFACT_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(value) => {
            serde_json::Value::String(truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_ARTIFACT_JSON_ITEMS)
                .map(|value| bounded_artifact_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(object) => {
            let mut bounded = serde_json::Map::new();
            for (key, value) in object.into_iter().take(MAX_ARTIFACT_JSON_ITEMS) {
                bounded.insert(
                    truncate_chars_with_suffix(&key, MAX_ARTIFACT_FIELD_CHARS),
                    bounded_artifact_json_value(value, depth + 1),
                );
            }
            serde_json::Value::Object(bounded)
        }
        value @ (serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)) => value,
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let end = value
        .char_indices()
        .nth(max_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    value[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{EvidenceSourceError, EvidenceSourceRef, EvidenceSourceResult};
    use rusqlite::Connection;
    use std::collections::BTreeMap;

    struct TestByteSource {
        source_ref: EvidenceSourceRef,
        declared_len: u64,
        data: Vec<u8>,
        max_chunk: usize,
    }

    impl TestByteSource {
        fn new(declared_len: u64, data: &[u8], max_chunk: usize) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: "test-source.sqlite".to_string(),
                },
                declared_len,
                data: data.to_vec(),
                max_chunk,
            }
        }
    }

    impl EvidenceByteSource for TestByteSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.declared_len)
        }

        fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
            if offset > self.declared_len {
                return Err(EvidenceSourceError::InvalidRange {
                    source_id: self.source_ref.display_id(),
                    offset,
                    size: self.declared_len,
                });
            }

            let start = offset as usize;
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            let read_size = size.min(self.max_chunk);
            let end = start.saturating_add(read_size).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    #[test]
    fn normalized_to_db_artifact_preserves_core_fields() {
        let artifact = NormalizedArtifact {
            id: "artifact-1".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/a.txt".to_string(),
            },
            source_id: "/case/a.txt".to_string(),
            name: "a.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 42,
            mime_type: Some("text/plain".to_string()),
            type_description: "Plain Text".to_string(),
            category: "text".to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("hello".to_string()),
            metadata: BTreeMap::from([("k".to_string(), "v".to_string())]),
        };

        let record = normalized_to_db_artifact(
            &artifact,
            Some("ev-1".to_string()),
            "test-extractor".to_string(),
        )
        .unwrap();

        assert_eq!(record.id, "artifact-1");
        assert_eq!(record.evidence_file_id.as_deref(), Some("ev-1"));
        assert_eq!(record.source_id, "/case/a.txt");
        assert_eq!(record.category, "text");
        assert_eq!(record.extractor, "test-extractor");
        assert!(record.source_ref_json.contains("localFile"));
        assert!(record.metadata_json.unwrap().contains("\"k\":\"v\""));
    }

    #[test]
    fn normalized_to_db_artifact_rejects_oversized_artifact_size() {
        let artifact = NormalizedArtifact {
            id: "huge-artifact".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/huge.bin".to_string(),
            },
            source_id: "/case/huge.bin".to_string(),
            name: "huge.bin".to_string(),
            extension: Some("bin".to_string()),
            size: u64::MAX,
            mime_type: Some("application/octet-stream".to_string()),
            type_description: "Binary Data".to_string(),
            category: "binary".to_string(),
            confidence: "medium".to_string(),
            is_text: false,
            content_preview: None,
            metadata: BTreeMap::new(),
        };

        let err = normalized_to_db_artifact(&artifact, None, "test-extractor".to_string())
            .expect_err("artifact conversion should reject unrepresentable size");

        assert!(err.contains("Artifact size exceeds project DB range"));
    }

    #[test]
    fn normalized_to_db_artifact_normalizes_extractor_label() {
        let artifact = NormalizedArtifact {
            id: "artifact-1".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/a.txt".to_string(),
            },
            source_id: "/case/a.txt".to_string(),
            name: "a.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 42,
            mime_type: Some("text/plain".to_string()),
            type_description: "Plain Text".to_string(),
            category: "text".to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("hello".to_string()),
            metadata: BTreeMap::new(),
        };
        let oversized = format!("  {}é  ", "x".repeat(MAX_ARTIFACT_EXTRACTOR_CHARS + 64));

        let record = normalized_to_db_artifact(&artifact, None, oversized)
            .expect("artifact conversion should succeed");
        assert_eq!(
            record.extractor.chars().count(),
            MAX_ARTIFACT_EXTRACTOR_CHARS
        );
        assert!(record.extractor.starts_with('x'));

        let defaulted = normalized_to_db_artifact(&artifact, None, "   ".to_string())
            .expect("artifact conversion should succeed");
        assert_eq!(defaulted.extractor, DEFAULT_ARTIFACT_EXTRACTOR);
    }

    #[test]
    fn normalized_to_db_artifact_bounds_preview_and_metadata() {
        let mut metadata = BTreeMap::new();
        for index in 0..(MAX_ARTIFACT_METADATA_ENTRIES + 25) {
            metadata.insert(
                format!("key-{index}"),
                "é".repeat(MAX_ARTIFACT_FIELD_CHARS + 32),
            );
        }
        let artifact = NormalizedArtifact {
            id: "artifact-1".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/a.txt".to_string(),
            },
            source_id: "/case/a.txt".to_string(),
            name: "a.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 42,
            mime_type: Some("text/plain".to_string()),
            type_description: "Plain Text".to_string(),
            category: "text".to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("p".repeat(MAX_ARTIFACT_PREVIEW_CHARS + 32)),
            metadata,
        };

        let record = normalized_to_db_artifact(&artifact, None, "test-extractor".to_string())
            .expect("artifact conversion should succeed");

        let preview = record.content_preview.as_deref().unwrap();
        assert_eq!(preview.chars().count(), MAX_ARTIFACT_PREVIEW_CHARS);
        assert!(preview.ends_with(ARTIFACT_TRUNCATED_SUFFIX));

        let metadata: BTreeMap<String, String> =
            serde_json::from_str(record.metadata_json.as_deref().unwrap()).unwrap();
        assert_eq!(metadata.len(), MAX_ARTIFACT_METADATA_ENTRIES);
        assert!(metadata
            .values()
            .all(|value| value.chars().count() == MAX_ARTIFACT_METADATA_VALUE_CHARS));
        assert!(metadata
            .values()
            .all(|value| value.ends_with(ARTIFACT_TRUNCATED_SUFFIX)));
    }

    #[test]
    fn bounded_artifact_record_for_response_caps_payloads_and_preserves_json() {
        let artifact = DbNormalizedArtifact {
            id: "artifact-1".to_string(),
            evidence_file_id: Some("ev-1".to_string()),
            source_id: "s".repeat(MAX_ARTIFACT_FIELD_CHARS + 32),
            source_ref_json: serde_json::json!({
                "kind": "localFile",
                "path": "x".repeat(MAX_ARTIFACT_JSON_CHARS + 32)
            })
            .to_string(),
            name: "n".repeat(MAX_ARTIFACT_FIELD_CHARS + 32),
            extension: Some("txt".to_string()),
            size: 42,
            mime_type: Some("text/plain".to_string()),
            type_description: "Plain Text".to_string(),
            category: "text".to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("p".repeat(MAX_ARTIFACT_PREVIEW_CHARS + 32)),
            metadata_json: Some(
                serde_json::json!({
                    "large": "m".repeat(MAX_ARTIFACT_JSON_CHARS + 32)
                })
                .to_string(),
            ),
            extracted_at: "2026-02-16T10:00:00Z".to_string(),
            extractor: "test-extractor".to_string(),
        };

        let bounded = bounded_artifact_record_for_response(artifact);

        assert_eq!(bounded.source_id.chars().count(), MAX_ARTIFACT_FIELD_CHARS);
        assert!(bounded.source_id.ends_with(ARTIFACT_TRUNCATED_SUFFIX));
        assert_eq!(bounded.name.chars().count(), MAX_ARTIFACT_FIELD_CHARS);
        assert_eq!(
            bounded.content_preview.as_deref().unwrap().chars().count(),
            MAX_ARTIFACT_PREVIEW_CHARS
        );
        assert!(serde_json::from_str::<serde_json::Value>(&bounded.source_ref_json).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(
            bounded.metadata_json.as_deref().unwrap()
        )
        .is_ok());
    }

    #[test]
    fn bounded_artifact_json_value_caps_arrays_and_depth() {
        let json = serde_json::json!({
            "items": (0..(MAX_ARTIFACT_JSON_ITEMS + 10)).collect::<Vec<_>>(),
            "nested": {"a": {"b": {"c": {"d": "too deep"}}}}
        })
        .to_string();

        let bounded = bounded_artifact_json_text(&json, MAX_ARTIFACT_JSON_CHARS);
        let value: serde_json::Value = serde_json::from_str(&bounded).unwrap();

        assert_eq!(
            value["items"].as_array().unwrap().len(),
            MAX_ARTIFACT_JSON_ITEMS
        );
        assert!(bounded.contains(ARTIFACT_TRUNCATED_SUFFIX));
    }

    #[test]
    fn copy_sqlite_artifact_source_accepts_chunked_reads() {
        let source = TestByteSource::new(10, b"0123456789", 3);
        let mut output = Vec::new();

        copy_sqlite_artifact_source(&source, source.len().unwrap(), &mut output).unwrap();

        assert_eq!(output, b"0123456789");
    }

    #[test]
    fn copy_sqlite_artifact_source_rejects_short_reads() {
        let source = TestByteSource::new(8, b"abc", 8);
        let mut output = Vec::new();

        let err =
            copy_sqlite_artifact_source(&source, source.len().unwrap(), &mut output).unwrap_err();

        assert!(err.contains("Short read materializing SQLite artifact source"));
        assert!(err.contains("expected 8 bytes but read 3 bytes"));
        assert_eq!(output, b"abc");
    }

    #[test]
    fn checked_sqlite_copy_offset_add_rejects_overflow() {
        let err = checked_sqlite_copy_offset_add(u64::MAX, 1, "test-source.sqlite").unwrap_err();

        assert!(err.contains("SQLite artifact copy offset overflow"));
        assert!(err.contains("test-source.sqlite"));
    }

    #[test]
    fn sqlite_artifact_metadata_from_source_extracts_schema_summary() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(tmp.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE contacts (id INTEGER PRIMARY KEY, name TEXT, email TEXT);
             CREATE TABLE logs (id INTEGER PRIMARY KEY, level TEXT, message TEXT);
             INSERT INTO contacts VALUES (1, 'Alice', 'alice@example.com');
             INSERT INTO contacts VALUES (2, 'Bob', 'bob@example.com');
             INSERT INTO logs VALUES (1, 'info', 'started');
             CREATE VIEW contact_names AS SELECT name FROM contacts;",
        )
        .unwrap();
        drop(conn);

        let source = HashSourceInput {
            path: Some(tmp.path().to_string_lossy().to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: Some("disk".to_string()),
            size: Some(std::fs::metadata(tmp.path()).unwrap().len()),
        };

        let metadata = sqlite_artifact_metadata_from_source(&source).unwrap();

        assert_eq!(
            metadata.get("sqlite.tableCount").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("sqlite.viewCount").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata.get("sqlite.totalRows").map(String::as_str),
            Some("3")
        );
        assert_eq!(
            metadata.get("sqlite.tableNames").map(String::as_str),
            Some("contacts, logs")
        );
        assert!(metadata
            .get("sqlite.tables")
            .is_some_and(|value| value.contains("contacts (2 rows, 3 cols)")));
        assert_eq!(
            metadata.get("sqlite.largestTable").map(String::as_str),
            Some("contacts (2 rows)")
        );
    }

    #[test]
    fn sqlite_total_rows_clamps_negative_counts_and_saturates() {
        let tables = vec![
            crate::viewer::document::database_viewer::TableSummary {
                name: "negative".to_string(),
                row_count: -25,
                column_count: 1,
                is_system: false,
            },
            crate::viewer::document::database_viewer::TableSummary {
                name: "huge".to_string(),
                row_count: i64::MAX,
                column_count: 1,
                is_system: false,
            },
            crate::viewer::document::database_viewer::TableSummary {
                name: "extra".to_string(),
                row_count: 42,
                column_count: 1,
                is_system: false,
            },
        ];

        assert_eq!(sqlite_total_rows(&tables), i64::MAX);
    }
}
