// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for evidence files, hashes, and verifications.

use super::with_project_db;
use crate::commands::hash::{hash_source, HashSourceInput, HashSourceResult};
use crate::project_db::{
    DbEvidenceFile, DbHashAlgorithmSummary, DbProjectHash, DbProjectVerification,
    DbVerificationResultSummary,
};

const MAX_EVIDENCE_RESPONSE_ROWS: usize = 10_000;
const MAX_HASH_RESPONSE_ROWS: usize = 10_000;
const MAX_VERIFICATION_RESPONSE_ROWS: usize = 10_000;
const MAX_EVIDENCE_FIELD_CHARS: usize = 4096;
const MAX_HASH_FIELD_CHARS: usize = 4096;
const MAX_HASH_VALUE_CHARS: usize = 1024;
const MAX_HASH_SOURCE_REF_JSON_CHARS: usize = 65_536;
const MAX_HASH_SOURCE_REF_JSON_DEPTH: usize = 4;
const MAX_HASH_SOURCE_REF_JSON_ITEMS: usize = 256;
const HASH_TRUNCATED_SUFFIX: &str = "... [truncated]";

/// Request to compute a source-aware hash and persist it to the active
/// project database in one backend operation.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbHashSourceRequest {
    pub source: HashSourceInput,
    pub algorithm: String,
    /// Existing evidence_files.id. Required unless `evidence_file` is provided.
    pub file_id: Option<String>,
    /// Evidence record to upsert before inserting the hash.
    pub evidence_file: Option<DbEvidenceFile>,
    /// Stored in hashes.source. Defaults to "computed".
    pub hash_record_source: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbHashSourceResult {
    pub hash_result: HashSourceResult,
    pub hash_record: DbProjectHash,
}

fn build_computed_hash_record(
    file_id: String,
    hash_result: &HashSourceResult,
    hash_record_source: Option<String>,
) -> Result<DbProjectHash, String> {
    let source_ref_json = serde_json::to_string(&hash_result.source_ref)
        .map_err(|e| format!("Failed to serialize hash source reference: {e}"))?;

    Ok(bounded_hash_record(DbProjectHash {
        id: uuid::Uuid::new_v4().to_string(),
        file_id: truncate_hash_text(&file_id, MAX_HASH_FIELD_CHARS),
        source_id: Some(truncate_hash_text(
            &hash_result.source_id,
            MAX_EVIDENCE_FIELD_CHARS,
        )),
        source_ref_json: Some(source_ref_json),
        algorithm: truncate_hash_text(&hash_result.algorithm, MAX_HASH_FIELD_CHARS),
        hash_value: truncate_hash_text(&hash_result.hash, MAX_HASH_VALUE_CHARS),
        computed_at: chrono::Utc::now().to_rfc3339(),
        segment_index: None,
        segment_name: None,
        source: hash_record_source.unwrap_or_else(|| "computed".to_string()),
    }))
}

// =============================================================================
// Evidence File Commands
// =============================================================================

/// Insert or update an evidence file record.
#[tauri::command]
pub fn project_db_upsert_evidence_file(
    window: tauri::Window,
    file: DbEvidenceFile,
) -> Result<(), String> {
    let file = bounded_evidence_file(file);
    with_project_db(window.label(), |db| db.upsert_evidence_file(&file))
}

/// Batch insert or update evidence files in a single transaction.
/// Accepts an array of files and inserts them all within one transaction,
/// reducing IPC overhead from N calls to 1 call.
#[tauri::command]
pub fn project_db_batch_upsert_evidence_files(
    window: tauri::Window,
    files: Vec<DbEvidenceFile>,
) -> Result<usize, String> {
    let files: Vec<_> = files.into_iter().map(bounded_evidence_file).collect();
    with_project_db(window.label(), |db| db.batch_upsert_evidence_files(&files))
}

/// Get all evidence files.
#[tauri::command]
pub fn project_db_get_evidence_files(window: tauri::Window) -> Result<Vec<DbEvidenceFile>, String> {
    with_project_db(window.label(), |db| db.get_evidence_files()).map(|files| {
        files
            .into_iter()
            .take(MAX_EVIDENCE_RESPONSE_ROWS)
            .map(bounded_evidence_file)
            .collect()
    })
}

/// Get an evidence file by path.
#[tauri::command]
pub fn project_db_get_evidence_file_by_path(
    window: tauri::Window,
    path: String,
) -> Result<Option<DbEvidenceFile>, String> {
    with_project_db(window.label(), |db| db.get_evidence_file_by_path(&path))
        .map(|file| file.map(bounded_evidence_file))
}

// =============================================================================
// Hash Commands
// =============================================================================

/// Insert a hash record.
#[tauri::command]
pub fn project_db_insert_hash(window: tauri::Window, hash: DbProjectHash) -> Result<(), String> {
    let hash = bounded_hash_record(hash);
    with_project_db(window.label(), |db| db.insert_hash(&hash))
}

/// Compute a hash from a local file or container entry and persist it to the
/// active project database.
#[tauri::command]
pub async fn project_db_hash_source_and_insert(
    window: tauri::Window,
    app: tauri::AppHandle,
    request: ProjectDbHashSourceRequest,
) -> Result<ProjectDbHashSourceResult, String> {
    let ProjectDbHashSourceRequest {
        source,
        algorithm,
        file_id,
        evidence_file,
        hash_record_source,
    } = request;

    let resolved_file_id = evidence_file
        .as_ref()
        .map(|file| file.id.clone())
        .or(file_id)
        .ok_or_else(|| "Hash persistence requires fileId or evidenceFile".to_string())?;

    let hash_result = hash_source(source, algorithm, app).await?;
    let hash_record =
        build_computed_hash_record(resolved_file_id, &hash_result, hash_record_source)?;

    with_project_db(window.label(), |db| {
        if let Some(file) = &evidence_file {
            db.upsert_evidence_file(file)?;
        }
        db.insert_hash(&hash_record)?;
        Ok(())
    })?;

    Ok(ProjectDbHashSourceResult {
        hash_result,
        hash_record: bounded_hash_record(hash_record),
    })
}

/// Get all hashes for an evidence file.
#[tauri::command]
pub fn project_db_get_hashes_for_file(
    window: tauri::Window,
    file_id: String,
) -> Result<Vec<DbProjectHash>, String> {
    with_project_db(window.label(), |db| db.get_hashes_for_file(&file_id)).map(|hashes| {
        hashes
            .into_iter()
            .take(MAX_HASH_RESPONSE_ROWS)
            .map(bounded_hash_record)
            .collect()
    })
}

/// Get all hashes for a source id.
#[tauri::command]
pub fn project_db_get_hashes_for_source(
    window: tauri::Window,
    source_id: String,
) -> Result<Vec<DbProjectHash>, String> {
    with_project_db(window.label(), |db| db.get_hashes_for_source(&source_id)).map(|hashes| {
        hashes
            .into_iter()
            .take(MAX_HASH_RESPONSE_ROWS)
            .map(bounded_hash_record)
            .collect()
    })
}

/// Summarize hashes by algorithm.
#[tauri::command]
pub fn project_db_summarize_hashes_by_algorithm(
    window: tauri::Window,
) -> Result<Vec<DbHashAlgorithmSummary>, String> {
    with_project_db(window.label(), |db| db.summarize_hashes_by_algorithm())
}

/// Get the latest hash for a file/algorithm.
#[tauri::command]
pub fn project_db_get_latest_hash(
    window: tauri::Window,
    file_id: String,
    algorithm: String,
) -> Result<Option<DbProjectHash>, String> {
    with_project_db(window.label(), |db| {
        db.get_latest_hash(&file_id, &algorithm)
    })
    .map(|hash| hash.map(bounded_hash_record))
}

/// Get the latest hash for a source id/algorithm.
#[tauri::command]
pub fn project_db_get_latest_hash_for_source(
    window: tauri::Window,
    source_id: String,
    algorithm: String,
) -> Result<Option<DbProjectHash>, String> {
    with_project_db(window.label(), |db| {
        db.get_latest_hash_for_source(&source_id, &algorithm)
    })
    .map(|hash| hash.map(bounded_hash_record))
}

/// Look up latest hash by file path and algorithm.
#[tauri::command]
pub fn project_db_lookup_hash_by_path(
    window: tauri::Window,
    path: String,
    algorithm: String,
) -> Result<Option<(String, String)>, String> {
    with_project_db(window.label(), |db| {
        db.lookup_hash_by_path(&path, &algorithm)
    })
}

// =============================================================================
// Verification Commands
// =============================================================================

/// Insert a verification record.
#[tauri::command]
pub fn project_db_insert_verification(
    window: tauri::Window,
    v: DbProjectVerification,
) -> Result<(), String> {
    let v = bounded_verification_record(v);
    with_project_db(window.label(), |db| db.insert_verification(&v))
}

/// Get verifications for a hash.
#[tauri::command]
pub fn project_db_get_verifications_for_hash(
    window: tauri::Window,
    hash_id: String,
) -> Result<Vec<DbProjectVerification>, String> {
    with_project_db(window.label(), |db| db.get_verifications_for_hash(&hash_id)).map(|items| {
        items
            .into_iter()
            .take(MAX_VERIFICATION_RESPONSE_ROWS)
            .map(bounded_verification_record)
            .collect()
    })
}

/// Summarize hash verification results by status.
#[tauri::command]
pub fn project_db_summarize_verifications_by_result(
    window: tauri::Window,
) -> Result<Vec<DbVerificationResultSummary>, String> {
    with_project_db(window.label(), |db| db.summarize_verifications_by_result())
}

fn bounded_evidence_file(mut file: DbEvidenceFile) -> DbEvidenceFile {
    file.id = truncate_hash_text(&file.id, MAX_EVIDENCE_FIELD_CHARS);
    file.path = truncate_hash_text(&file.path, MAX_EVIDENCE_FIELD_CHARS);
    file.filename = truncate_hash_text(&file.filename, MAX_EVIDENCE_FIELD_CHARS);
    file.container_type = truncate_hash_text(&file.container_type, MAX_EVIDENCE_FIELD_CHARS);
    file.discovered_at = truncate_hash_text(&file.discovered_at, MAX_EVIDENCE_FIELD_CHARS);
    file.created = file
        .created
        .map(|value| truncate_hash_text(&value, MAX_EVIDENCE_FIELD_CHARS));
    file.modified = file
        .modified
        .map(|value| truncate_hash_text(&value, MAX_EVIDENCE_FIELD_CHARS));
    file
}

fn bounded_hash_record(mut hash: DbProjectHash) -> DbProjectHash {
    hash.id = truncate_hash_text(&hash.id, MAX_HASH_FIELD_CHARS);
    hash.file_id = truncate_hash_text(&hash.file_id, MAX_HASH_FIELD_CHARS);
    hash.source_id = hash
        .source_id
        .map(|value| truncate_hash_text(&value, MAX_EVIDENCE_FIELD_CHARS));
    hash.source_ref_json = hash
        .source_ref_json
        .map(|value| bounded_hash_json_text(&value, MAX_HASH_SOURCE_REF_JSON_CHARS));
    hash.algorithm = truncate_hash_text(&hash.algorithm, MAX_HASH_FIELD_CHARS);
    hash.hash_value = truncate_hash_text(&hash.hash_value, MAX_HASH_VALUE_CHARS);
    hash.computed_at = truncate_hash_text(&hash.computed_at, MAX_HASH_FIELD_CHARS);
    hash.segment_name = hash
        .segment_name
        .map(|value| truncate_hash_text(&value, MAX_HASH_FIELD_CHARS));
    hash.source = truncate_hash_text(&hash.source, MAX_HASH_FIELD_CHARS);
    hash
}

fn bounded_verification_record(mut v: DbProjectVerification) -> DbProjectVerification {
    v.id = truncate_hash_text(&v.id, MAX_HASH_FIELD_CHARS);
    v.hash_id = truncate_hash_text(&v.hash_id, MAX_HASH_FIELD_CHARS);
    v.verified_at = truncate_hash_text(&v.verified_at, MAX_HASH_FIELD_CHARS);
    v.result = truncate_hash_text(&v.result, MAX_HASH_FIELD_CHARS);
    v.expected_hash = truncate_hash_text(&v.expected_hash, MAX_HASH_VALUE_CHARS);
    v.actual_hash = truncate_hash_text(&v.actual_hash, MAX_HASH_VALUE_CHARS);
    v
}

fn truncate_hash_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = HASH_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + HASH_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(HASH_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_hash_json_text(value: &str, max_chars: usize) -> String {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return if value.chars().count() <= max_chars {
            value.to_string()
        } else {
            truncate_hash_text(value, max_chars)
        };
    };
    let bounded = bounded_hash_json_value(json, 0);
    let Ok(serialized) = serde_json::to_string(&bounded) else {
        return "{\"truncated\":true}".to_string();
    };

    if serialized.chars().count() <= max_chars {
        serialized
    } else {
        "{\"truncated\":true}".to_string()
    }
}

fn bounded_hash_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_HASH_SOURCE_REF_JSON_DEPTH {
        return serde_json::Value::String(HASH_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(value) => {
            serde_json::Value::String(truncate_hash_text(&value, MAX_EVIDENCE_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_HASH_SOURCE_REF_JSON_ITEMS)
                .map(|value| bounded_hash_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(object) => {
            let mut bounded = serde_json::Map::new();
            for (key, value) in object.into_iter().take(MAX_HASH_SOURCE_REF_JSON_ITEMS) {
                bounded.insert(
                    truncate_hash_text(&key, MAX_HASH_FIELD_CHARS),
                    bounded_hash_json_value(value, depth + 1),
                );
            }
            serde_json::Value::Object(bounded)
        }
        value @ (serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)) => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hash_result() -> HashSourceResult {
        HashSourceResult {
            source_ref: crate::common::EvidenceSourceRef::LocalFile {
                path: "/case/item.bin".to_string(),
            },
            source_id: "/case/item.bin".to_string(),
            path: Some("/case/item.bin".to_string()),
            container_path: None,
            entry_path: None,
            container_type: Some("disk".to_string()),
            algorithm: "SHA-256".to_string(),
            hash: "abcdef123456".to_string(),
            bytes_hashed: 12,
            duration_ms: 3,
            throughput_mbs: Some(4.0),
        }
    }

    #[test]
    fn build_computed_hash_record_maps_hash_result() {
        let hash_result = sample_hash_result();
        let record = build_computed_hash_record("ev_1".to_string(), &hash_result, None).unwrap();

        assert_eq!(record.file_id, "ev_1");
        assert_eq!(record.source_id.as_deref(), Some("/case/item.bin"));
        assert!(record
            .source_ref_json
            .as_deref()
            .unwrap()
            .contains("localFile"));
        assert_eq!(record.algorithm, "SHA-256");
        assert_eq!(record.hash_value, "abcdef123456");
        assert_eq!(record.source, "computed");
        assert!(record.segment_index.is_none());
        assert!(record.segment_name.is_none());
        assert!(chrono::DateTime::parse_from_rfc3339(&record.computed_at).is_ok());
    }

    #[test]
    fn build_computed_hash_record_uses_custom_source_label() {
        let hash_result = sample_hash_result();
        let record = build_computed_hash_record(
            "ev_1".to_string(),
            &hash_result,
            Some("artifact-computed".to_string()),
        )
        .unwrap();

        assert_eq!(record.source, "artifact-computed");
    }

    #[test]
    fn bounded_evidence_file_caps_path_like_fields() {
        let file = DbEvidenceFile {
            id: "ev-1".to_string(),
            path: "p".repeat(MAX_EVIDENCE_FIELD_CHARS + 32),
            filename: "f".repeat(MAX_EVIDENCE_FIELD_CHARS + 32),
            container_type: "ad1".to_string(),
            total_size: 42,
            segment_count: 1,
            discovered_at: "2026-02-16T10:00:00Z".to_string(),
            created: Some("c".repeat(MAX_EVIDENCE_FIELD_CHARS + 32)),
            modified: None,
        };

        let bounded = bounded_evidence_file(file);

        assert_eq!(bounded.path.chars().count(), MAX_EVIDENCE_FIELD_CHARS);
        assert!(bounded.path.ends_with(HASH_TRUNCATED_SUFFIX));
        assert_eq!(bounded.filename.chars().count(), MAX_EVIDENCE_FIELD_CHARS);
        assert_eq!(
            bounded.created.as_deref().unwrap().chars().count(),
            MAX_EVIDENCE_FIELD_CHARS
        );
    }

    #[test]
    fn bounded_hash_record_caps_fields_and_preserves_json() {
        let hash = DbProjectHash {
            id: "hash-1".to_string(),
            file_id: "ev-1".to_string(),
            source_id: Some("s".repeat(MAX_EVIDENCE_FIELD_CHARS + 32)),
            source_ref_json: Some(
                serde_json::json!({
                    "kind": "localFile",
                    "path": "x".repeat(MAX_HASH_SOURCE_REF_JSON_CHARS + 32),
                    "parts": (0..(MAX_HASH_SOURCE_REF_JSON_ITEMS + 10)).collect::<Vec<_>>()
                })
                .to_string(),
            ),
            algorithm: "SHA-256".to_string(),
            hash_value: "a".repeat(MAX_HASH_VALUE_CHARS + 32),
            computed_at: "2026-02-16T10:00:00Z".to_string(),
            segment_index: None,
            segment_name: Some("segment".to_string()),
            source: "computed".to_string(),
        };

        let bounded = bounded_hash_record(hash);

        assert_eq!(
            bounded.source_id.as_deref().unwrap().chars().count(),
            MAX_EVIDENCE_FIELD_CHARS
        );
        assert_eq!(bounded.hash_value.chars().count(), MAX_HASH_VALUE_CHARS);
        assert!(bounded.hash_value.ends_with(HASH_TRUNCATED_SUFFIX));

        let source_ref: serde_json::Value =
            serde_json::from_str(bounded.source_ref_json.as_deref().unwrap()).unwrap();
        assert_eq!(
            source_ref["parts"].as_array().unwrap().len(),
            MAX_HASH_SOURCE_REF_JSON_ITEMS
        );
    }

    #[test]
    fn bounded_verification_record_caps_hash_values() {
        let record = DbProjectVerification {
            id: "verification-1".to_string(),
            hash_id: "hash-1".to_string(),
            verified_at: "2026-02-16T10:00:00Z".to_string(),
            result: "mismatch".to_string(),
            expected_hash: "e".repeat(MAX_HASH_VALUE_CHARS + 32),
            actual_hash: "a".repeat(MAX_HASH_VALUE_CHARS + 32),
        };

        let bounded = bounded_verification_record(record);

        assert_eq!(bounded.expected_hash.chars().count(), MAX_HASH_VALUE_CHARS);
        assert_eq!(bounded.actual_hash.chars().count(), MAX_HASH_VALUE_CHARS);
        assert!(bounded.expected_hash.ends_with(HASH_TRUNCATED_SUFFIX));
        assert!(bounded.actual_hash.ends_with(HASH_TRUNCATED_SUFFIX));
    }

    #[test]
    fn bounded_hash_json_text_truncates_depth() {
        let value = serde_json::json!({
            "a": {
                "b": {
                    "c": {
                        "d": {
                            "e": "too deep"
                        }
                    }
                }
            }
        })
        .to_string();

        let bounded = bounded_hash_json_text(&value, MAX_HASH_SOURCE_REF_JSON_CHARS);

        assert!(bounded.contains(HASH_TRUNCATED_SUFFIX));
        assert!(serde_json::from_str::<serde_json::Value>(&bounded).is_ok());
    }
}
