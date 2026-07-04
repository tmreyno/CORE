// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for processed databases, integrity, metrics, AXIOM case info,
//! evidence sources, search results, and artifact categories.

use super::with_project_db;
use crate::project_db::{
    DbArtifactCategory, DbAxiomCaseInfo, DbAxiomEvidenceSource, DbAxiomSearchResult,
    DbProcessedDatabase, DbProcessedDbIntegrity, DbProcessedDbMetrics,
};

const MAX_PROCESSED_RESPONSE_ROWS: usize = 10_000;
const MAX_PROCESSED_FIELD_CHARS: usize = 4096;
const MAX_PROCESSED_TEXT_CHARS: usize = 65_536;
const MAX_PROCESSED_JSON_CHARS: usize = 65_536;
const MAX_PROCESSED_JSON_DEPTH: usize = 4;
const MAX_PROCESSED_JSON_ITEMS: usize = 256;
const PROCESSED_TRUNCATED_SUFFIX: &str = "... [truncated]";

// =============================================================================
// Processed Database Commands
// =============================================================================

/// Insert or update a processed database record.
#[tauri::command]
pub fn project_db_upsert_processed_database(
    window: tauri::Window,
    db: DbProcessedDatabase,
) -> Result<(), String> {
    let db = bounded_processed_database(db);
    with_project_db(window.label(), |pdb| pdb.upsert_processed_database(&db))
}

/// Get all processed databases.
#[tauri::command]
pub fn project_db_get_processed_databases(
    window: tauri::Window,
) -> Result<Vec<DbProcessedDatabase>, String> {
    with_project_db(window.label(), |db| {
        db.get_processed_databases().map(|records| {
            records
                .into_iter()
                .take(MAX_PROCESSED_RESPONSE_ROWS)
                .map(bounded_processed_database)
                .collect()
        })
    })
}

/// Get a processed database by path.
#[tauri::command]
pub fn project_db_get_processed_database_by_path(
    window: tauri::Window,
    path: String,
) -> Result<Option<DbProcessedDatabase>, String> {
    let path = truncate_processed_text(&path, MAX_PROCESSED_TEXT_CHARS);
    with_project_db(window.label(), |db| {
        db.get_processed_database_by_path(&path)
            .map(|record| record.map(bounded_processed_database))
    })
}

/// Delete a processed database and all related records.
#[tauri::command]
pub fn project_db_delete_processed_database(
    window: tauri::Window,
    id: String,
) -> Result<(), String> {
    let id = truncate_processed_text(&id, MAX_PROCESSED_FIELD_CHARS);
    with_project_db(window.label(), |db| db.delete_processed_database(&id))
}

// =============================================================================
// Processed DB Integrity Commands
// =============================================================================

/// Insert or update a processed database integrity record.
#[tauri::command]
pub fn project_db_upsert_processed_db_integrity(
    window: tauri::Window,
    integrity: DbProcessedDbIntegrity,
) -> Result<(), String> {
    let integrity = bounded_processed_integrity(integrity);
    with_project_db(window.label(), |db| {
        db.upsert_processed_db_integrity(&integrity)
    })
}

/// Get integrity records for a processed database.
#[tauri::command]
pub fn project_db_get_processed_db_integrity(
    window: tauri::Window,
    processed_db_id: String,
) -> Result<Vec<DbProcessedDbIntegrity>, String> {
    let processed_db_id = truncate_processed_text(&processed_db_id, MAX_PROCESSED_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_processed_db_integrity(&processed_db_id)
            .map(|records| {
                records
                    .into_iter()
                    .take(MAX_PROCESSED_RESPONSE_ROWS)
                    .map(bounded_processed_integrity)
                    .collect()
            })
    })
}

// =============================================================================
// Processed DB Metrics Commands
// =============================================================================

/// Insert or update metrics for a processed database.
#[tauri::command]
pub fn project_db_upsert_processed_db_metrics(
    window: tauri::Window,
    metrics: DbProcessedDbMetrics,
) -> Result<(), String> {
    let metrics = bounded_processed_metrics(metrics);
    with_project_db(window.label(), |db| {
        db.upsert_processed_db_metrics(&metrics)
    })
}

/// Get metrics for a processed database.
#[tauri::command]
pub fn project_db_get_processed_db_metrics(
    window: tauri::Window,
    processed_db_id: String,
) -> Result<Option<DbProcessedDbMetrics>, String> {
    let processed_db_id = truncate_processed_text(&processed_db_id, MAX_PROCESSED_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_processed_db_metrics(&processed_db_id)
            .map(|metrics| metrics.map(bounded_processed_metrics))
    })
}

// =============================================================================
// AXIOM Case Info Commands
// =============================================================================

/// Insert or update AXIOM case information.
#[tauri::command]
pub fn project_db_upsert_axiom_case_info(
    window: tauri::Window,
    info: DbAxiomCaseInfo,
) -> Result<(), String> {
    let info = bounded_axiom_case_info(info);
    with_project_db(window.label(), |db| db.upsert_axiom_case_info(&info))
}

/// Get AXIOM case info for a processed database.
#[tauri::command]
pub fn project_db_get_axiom_case_info(
    window: tauri::Window,
    processed_db_id: String,
) -> Result<Option<DbAxiomCaseInfo>, String> {
    let processed_db_id = truncate_processed_text(&processed_db_id, MAX_PROCESSED_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_axiom_case_info(&processed_db_id)
            .map(|info| info.map(bounded_axiom_case_info))
    })
}

/// Get all AXIOM case info records.
#[tauri::command]
pub fn project_db_get_all_axiom_case_info(
    window: tauri::Window,
) -> Result<Vec<DbAxiomCaseInfo>, String> {
    with_project_db(window.label(), |db| {
        db.get_all_axiom_case_info().map(|records| {
            records
                .into_iter()
                .take(MAX_PROCESSED_RESPONSE_ROWS)
                .map(bounded_axiom_case_info)
                .collect()
        })
    })
}

// =============================================================================
// AXIOM Evidence Source Commands
// =============================================================================

/// Insert an AXIOM evidence source.
#[tauri::command]
pub fn project_db_insert_axiom_evidence_source(
    window: tauri::Window,
    source: DbAxiomEvidenceSource,
) -> Result<(), String> {
    let source = bounded_axiom_evidence_source(source);
    with_project_db(window.label(), |db| {
        db.insert_axiom_evidence_source(&source)
    })
}

/// Get evidence sources for an AXIOM case.
#[tauri::command]
pub fn project_db_get_axiom_evidence_sources(
    window: tauri::Window,
    axiom_case_id: String,
) -> Result<Vec<DbAxiomEvidenceSource>, String> {
    let axiom_case_id = truncate_processed_text(&axiom_case_id, MAX_PROCESSED_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_axiom_evidence_sources(&axiom_case_id)
            .map(|records| {
                records
                    .into_iter()
                    .take(MAX_PROCESSED_RESPONSE_ROWS)
                    .map(bounded_axiom_evidence_source)
                    .collect()
            })
    })
}

// =============================================================================
// AXIOM Search Result Commands
// =============================================================================

/// Insert an AXIOM search result.
#[tauri::command]
pub fn project_db_insert_axiom_search_result(
    window: tauri::Window,
    result: DbAxiomSearchResult,
) -> Result<(), String> {
    let result = bounded_axiom_search_result(result);
    with_project_db(window.label(), |db| db.insert_axiom_search_result(&result))
}

/// Get search results for an AXIOM case.
#[tauri::command]
pub fn project_db_get_axiom_search_results(
    window: tauri::Window,
    axiom_case_id: String,
) -> Result<Vec<DbAxiomSearchResult>, String> {
    let axiom_case_id = truncate_processed_text(&axiom_case_id, MAX_PROCESSED_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_axiom_search_results(&axiom_case_id).map(|records| {
            records
                .into_iter()
                .take(MAX_PROCESSED_RESPONSE_ROWS)
                .map(bounded_axiom_search_result)
                .collect()
        })
    })
}

// =============================================================================
// Artifact Category Commands
// =============================================================================

/// Insert or replace artifact categories for a processed database.
#[tauri::command]
pub fn project_db_upsert_artifact_categories(
    window: tauri::Window,
    categories: Vec<DbArtifactCategory>,
) -> Result<(), String> {
    let categories: Vec<DbArtifactCategory> = categories
        .into_iter()
        .take(MAX_PROCESSED_RESPONSE_ROWS)
        .map(bounded_artifact_category)
        .collect();
    with_project_db(window.label(), |db| {
        db.upsert_artifact_categories(&categories)
    })
}

/// Get artifact categories for a processed database.
#[tauri::command]
pub fn project_db_get_artifact_categories(
    window: tauri::Window,
    processed_db_id: String,
) -> Result<Vec<DbArtifactCategory>, String> {
    let processed_db_id = truncate_processed_text(&processed_db_id, MAX_PROCESSED_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_artifact_categories(&processed_db_id).map(|records| {
            records
                .into_iter()
                .take(MAX_PROCESSED_RESPONSE_ROWS)
                .map(bounded_artifact_category)
                .collect()
        })
    })
}

fn bounded_processed_database(mut db: DbProcessedDatabase) -> DbProcessedDatabase {
    db.id = truncate_processed_text(&db.id, MAX_PROCESSED_FIELD_CHARS);
    db.path = truncate_processed_text(&db.path, MAX_PROCESSED_TEXT_CHARS);
    db.name = truncate_processed_text(&db.name, MAX_PROCESSED_FIELD_CHARS);
    db.db_type = truncate_processed_text(&db.db_type, MAX_PROCESSED_FIELD_CHARS);
    db.case_number = opt_processed_text(db.case_number, MAX_PROCESSED_FIELD_CHARS);
    db.examiner = opt_processed_text(db.examiner, MAX_PROCESSED_FIELD_CHARS);
    db.created_date = opt_processed_text(db.created_date, MAX_PROCESSED_FIELD_CHARS);
    db.notes = opt_processed_text(db.notes, MAX_PROCESSED_TEXT_CHARS);
    db.registered_at = truncate_processed_text(&db.registered_at, MAX_PROCESSED_FIELD_CHARS);
    db.metadata_json = db
        .metadata_json
        .map(|value| bounded_processed_json_or_text(&value, MAX_PROCESSED_JSON_CHARS));
    db
}

fn bounded_processed_integrity(mut integrity: DbProcessedDbIntegrity) -> DbProcessedDbIntegrity {
    integrity.id = truncate_processed_text(&integrity.id, MAX_PROCESSED_FIELD_CHARS);
    integrity.processed_db_id =
        truncate_processed_text(&integrity.processed_db_id, MAX_PROCESSED_FIELD_CHARS);
    integrity.file_path = truncate_processed_text(&integrity.file_path, MAX_PROCESSED_TEXT_CHARS);
    integrity.baseline_hash =
        truncate_processed_text(&integrity.baseline_hash, MAX_PROCESSED_FIELD_CHARS);
    integrity.baseline_timestamp =
        truncate_processed_text(&integrity.baseline_timestamp, MAX_PROCESSED_FIELD_CHARS);
    integrity.current_hash = opt_processed_text(integrity.current_hash, MAX_PROCESSED_FIELD_CHARS);
    integrity.current_hash_timestamp =
        opt_processed_text(integrity.current_hash_timestamp, MAX_PROCESSED_FIELD_CHARS);
    integrity.status = truncate_processed_text(&integrity.status, MAX_PROCESSED_FIELD_CHARS);
    integrity.changes_json = integrity
        .changes_json
        .map(|value| bounded_processed_json_or_text(&value, MAX_PROCESSED_JSON_CHARS));
    integrity
}

fn bounded_processed_metrics(mut metrics: DbProcessedDbMetrics) -> DbProcessedDbMetrics {
    metrics.id = truncate_processed_text(&metrics.id, MAX_PROCESSED_FIELD_CHARS);
    metrics.processed_db_id =
        truncate_processed_text(&metrics.processed_db_id, MAX_PROCESSED_FIELD_CHARS);
    metrics.user_names_json = metrics
        .user_names_json
        .map(|value| bounded_processed_json_or_text(&value, MAX_PROCESSED_JSON_CHARS));
    metrics.captured_at = truncate_processed_text(&metrics.captured_at, MAX_PROCESSED_FIELD_CHARS);
    metrics
}

fn bounded_axiom_case_info(mut info: DbAxiomCaseInfo) -> DbAxiomCaseInfo {
    info.id = truncate_processed_text(&info.id, MAX_PROCESSED_FIELD_CHARS);
    info.processed_db_id =
        truncate_processed_text(&info.processed_db_id, MAX_PROCESSED_FIELD_CHARS);
    info.case_name = truncate_processed_text(&info.case_name, MAX_PROCESSED_FIELD_CHARS);
    info.case_number = opt_processed_text(info.case_number, MAX_PROCESSED_FIELD_CHARS);
    info.case_type = opt_processed_text(info.case_type, MAX_PROCESSED_FIELD_CHARS);
    info.description = opt_processed_text(info.description, MAX_PROCESSED_TEXT_CHARS);
    info.examiner = opt_processed_text(info.examiner, MAX_PROCESSED_FIELD_CHARS);
    info.agency = opt_processed_text(info.agency, MAX_PROCESSED_FIELD_CHARS);
    info.axiom_version = opt_processed_text(info.axiom_version, MAX_PROCESSED_FIELD_CHARS);
    info.search_start = opt_processed_text(info.search_start, MAX_PROCESSED_FIELD_CHARS);
    info.search_end = opt_processed_text(info.search_end, MAX_PROCESSED_FIELD_CHARS);
    info.search_duration = opt_processed_text(info.search_duration, MAX_PROCESSED_FIELD_CHARS);
    info.search_outcome = opt_processed_text(info.search_outcome, MAX_PROCESSED_FIELD_CHARS);
    info.output_folder = opt_processed_text(info.output_folder, MAX_PROCESSED_TEXT_CHARS);
    info.case_path = opt_processed_text(info.case_path, MAX_PROCESSED_TEXT_CHARS);
    info.captured_at = truncate_processed_text(&info.captured_at, MAX_PROCESSED_FIELD_CHARS);
    info.keyword_info_json = info
        .keyword_info_json
        .map(|value| bounded_processed_json_or_text(&value, MAX_PROCESSED_JSON_CHARS));
    info
}

fn bounded_axiom_evidence_source(mut source: DbAxiomEvidenceSource) -> DbAxiomEvidenceSource {
    source.id = truncate_processed_text(&source.id, MAX_PROCESSED_FIELD_CHARS);
    source.axiom_case_id =
        truncate_processed_text(&source.axiom_case_id, MAX_PROCESSED_FIELD_CHARS);
    source.name = truncate_processed_text(&source.name, MAX_PROCESSED_FIELD_CHARS);
    source.evidence_number = opt_processed_text(source.evidence_number, MAX_PROCESSED_FIELD_CHARS);
    source.source_type = truncate_processed_text(&source.source_type, MAX_PROCESSED_FIELD_CHARS);
    source.path = opt_processed_text(source.path, MAX_PROCESSED_TEXT_CHARS);
    source.hash = opt_processed_text(source.hash, MAX_PROCESSED_FIELD_CHARS);
    source.acquired = opt_processed_text(source.acquired, MAX_PROCESSED_FIELD_CHARS);
    source.search_types_json = source
        .search_types_json
        .map(|value| bounded_processed_json_or_text(&value, MAX_PROCESSED_JSON_CHARS));
    source
}

fn bounded_axiom_search_result(mut result: DbAxiomSearchResult) -> DbAxiomSearchResult {
    result.id = truncate_processed_text(&result.id, MAX_PROCESSED_FIELD_CHARS);
    result.axiom_case_id =
        truncate_processed_text(&result.axiom_case_id, MAX_PROCESSED_FIELD_CHARS);
    result.artifact_type =
        truncate_processed_text(&result.artifact_type, MAX_PROCESSED_FIELD_CHARS);
    result
}

fn bounded_artifact_category(mut category: DbArtifactCategory) -> DbArtifactCategory {
    category.id = truncate_processed_text(&category.id, MAX_PROCESSED_FIELD_CHARS);
    category.processed_db_id =
        truncate_processed_text(&category.processed_db_id, MAX_PROCESSED_FIELD_CHARS);
    category.category = truncate_processed_text(&category.category, MAX_PROCESSED_FIELD_CHARS);
    category.artifact_type =
        truncate_processed_text(&category.artifact_type, MAX_PROCESSED_FIELD_CHARS);
    category
}

fn opt_processed_text(value: Option<String>, max_chars: usize) -> Option<String> {
    value.map(|value| truncate_processed_text(&value, max_chars))
}

fn truncate_processed_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let keep_chars = max_chars.saturating_sub(PROCESSED_TRUNCATED_SUFFIX.chars().count());
    let mut truncated: String = value.chars().take(keep_chars).collect();
    truncated.push_str(PROCESSED_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_processed_json_or_text(value: &str, max_chars: usize) -> String {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(value) else {
        return truncate_processed_text(value, max_chars);
    };
    let bounded = bounded_processed_json_value(parsed, 0);
    match serde_json::to_string(&bounded) {
        Ok(serialized) => truncate_processed_text(&serialized, max_chars),
        Err(_) => truncate_processed_text(value, max_chars),
    }
}

fn bounded_processed_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_PROCESSED_JSON_DEPTH {
        return serde_json::Value::String(PROCESSED_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(text) => {
            serde_json::Value::String(truncate_processed_text(&text, MAX_PROCESSED_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_PROCESSED_JSON_ITEMS)
                .map(|value| bounded_processed_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(entries) => serde_json::Value::Object(
            entries
                .into_iter()
                .take(MAX_PROCESSED_JSON_ITEMS)
                .map(|(key, value)| {
                    (
                        truncate_processed_text(&key, MAX_PROCESSED_FIELD_CHARS),
                        bounded_processed_json_value(value, depth + 1),
                    )
                })
                .collect(),
        ),
        scalar => scalar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repeated(len: usize) -> String {
        "x".repeat(len)
    }

    #[test]
    fn bounded_processed_database_caps_metadata_json_and_notes() {
        let record = bounded_processed_database(DbProcessedDatabase {
            id: repeated(MAX_PROCESSED_FIELD_CHARS + 8),
            path: repeated(MAX_PROCESSED_TEXT_CHARS + 8),
            name: repeated(MAX_PROCESSED_FIELD_CHARS + 8),
            db_type: "axiom".to_string(),
            case_number: Some(repeated(MAX_PROCESSED_FIELD_CHARS + 8)),
            examiner: Some("examiner".to_string()),
            created_date: None,
            total_size: 100,
            artifact_count: Some(10),
            notes: Some(repeated(MAX_PROCESSED_TEXT_CHARS + 8)),
            registered_at: "2026-02-16T10:00:00Z".to_string(),
            metadata_json: Some(
                serde_json::to_string(&vec![repeated(MAX_PROCESSED_FIELD_CHARS + 8)]).unwrap(),
            ),
        });

        assert_eq!(record.id.chars().count(), MAX_PROCESSED_FIELD_CHARS);
        assert_eq!(record.path.chars().count(), MAX_PROCESSED_TEXT_CHARS);
        assert_eq!(
            record.notes.unwrap().chars().count(),
            MAX_PROCESSED_TEXT_CHARS
        );
        assert_eq!(
            record.case_number.unwrap().chars().count(),
            MAX_PROCESSED_FIELD_CHARS
        );
        let metadata: serde_json::Value =
            serde_json::from_str(&record.metadata_json.unwrap()).unwrap();
        assert_eq!(
            metadata[0].as_str().unwrap().chars().count(),
            MAX_PROCESSED_FIELD_CHARS
        );
    }

    #[test]
    fn bounded_integrity_and_metrics_cap_json_fields() {
        let integrity = bounded_processed_integrity(DbProcessedDbIntegrity {
            id: "integrity-1".to_string(),
            processed_db_id: "processed-1".to_string(),
            file_path: repeated(MAX_PROCESSED_TEXT_CHARS + 8),
            file_size: 100,
            baseline_hash: repeated(MAX_PROCESSED_FIELD_CHARS + 8),
            baseline_timestamp: "2026-02-16T10:00:00Z".to_string(),
            current_hash: Some(repeated(MAX_PROCESSED_FIELD_CHARS + 8)),
            current_hash_timestamp: None,
            status: "changed".to_string(),
            changes_json: Some(
                "{\"outer\":{\"inner\":{\"deeper\":{\"too\":\"deep\"}}}}".to_string(),
            ),
        });
        let metrics = bounded_processed_metrics(DbProcessedDbMetrics {
            id: "metrics-1".to_string(),
            processed_db_id: "processed-1".to_string(),
            total_scans: 1,
            last_scan_date: None,
            total_jobs: 2,
            last_job_date: None,
            total_notes: 3,
            total_tagged_items: 4,
            total_users: 5,
            user_names_json: Some(
                serde_json::to_string(&vec![repeated(MAX_PROCESSED_FIELD_CHARS + 8)]).unwrap(),
            ),
            captured_at: "2026-02-16T10:00:00Z".to_string(),
        });

        assert_eq!(
            integrity.file_path.chars().count(),
            MAX_PROCESSED_TEXT_CHARS
        );
        assert_eq!(
            integrity.baseline_hash.chars().count(),
            MAX_PROCESSED_FIELD_CHARS
        );
        assert!(integrity
            .changes_json
            .unwrap()
            .contains(PROCESSED_TRUNCATED_SUFFIX));
        let users: serde_json::Value =
            serde_json::from_str(&metrics.user_names_json.unwrap()).unwrap();
        assert_eq!(
            users[0].as_str().unwrap().chars().count(),
            MAX_PROCESSED_FIELD_CHARS
        );
    }

    #[test]
    fn bounded_axiom_records_and_categories_cap_payloads() {
        let info = bounded_axiom_case_info(DbAxiomCaseInfo {
            id: "case-1".to_string(),
            processed_db_id: "processed-1".to_string(),
            case_name: repeated(MAX_PROCESSED_FIELD_CHARS + 8),
            case_number: None,
            case_type: None,
            description: Some(repeated(MAX_PROCESSED_TEXT_CHARS + 8)),
            examiner: None,
            agency: None,
            axiom_version: None,
            search_start: None,
            search_end: None,
            search_duration: None,
            search_outcome: None,
            output_folder: Some(repeated(MAX_PROCESSED_TEXT_CHARS + 8)),
            total_artifacts: 10,
            case_path: Some(repeated(MAX_PROCESSED_TEXT_CHARS + 8)),
            captured_at: "2026-02-16T10:00:00Z".to_string(),
            keyword_info_json: Some(
                serde_json::to_string(&vec![repeated(MAX_PROCESSED_FIELD_CHARS + 8)]).unwrap(),
            ),
        });
        let source = bounded_axiom_evidence_source(DbAxiomEvidenceSource {
            id: "source-1".to_string(),
            axiom_case_id: "case-1".to_string(),
            name: repeated(MAX_PROCESSED_FIELD_CHARS + 8),
            evidence_number: None,
            source_type: "image".to_string(),
            path: Some(repeated(MAX_PROCESSED_TEXT_CHARS + 8)),
            hash: Some(repeated(MAX_PROCESSED_FIELD_CHARS + 8)),
            size: Some(100),
            acquired: None,
            search_types_json: Some(
                serde_json::to_string(&vec![repeated(MAX_PROCESSED_FIELD_CHARS + 8)]).unwrap(),
            ),
        });
        let result = bounded_axiom_search_result(DbAxiomSearchResult {
            id: "result-1".to_string(),
            axiom_case_id: "case-1".to_string(),
            artifact_type: repeated(MAX_PROCESSED_FIELD_CHARS + 8),
            hit_count: 10,
        });
        let category = bounded_artifact_category(DbArtifactCategory {
            id: "category-1".to_string(),
            processed_db_id: "processed-1".to_string(),
            category: repeated(MAX_PROCESSED_FIELD_CHARS + 8),
            artifact_type: repeated(MAX_PROCESSED_FIELD_CHARS + 8),
            count: 10,
        });

        assert_eq!(info.case_name.chars().count(), MAX_PROCESSED_FIELD_CHARS);
        assert_eq!(
            info.description.unwrap().chars().count(),
            MAX_PROCESSED_TEXT_CHARS
        );
        assert_eq!(source.name.chars().count(), MAX_PROCESSED_FIELD_CHARS);
        assert_eq!(
            source.path.unwrap().chars().count(),
            MAX_PROCESSED_TEXT_CHARS
        );
        assert_eq!(
            result.artifact_type.chars().count(),
            MAX_PROCESSED_FIELD_CHARS
        );
        assert_eq!(
            category.artifact_type.chars().count(),
            MAX_PROCESSED_FIELD_CHARS
        );
    }
}
