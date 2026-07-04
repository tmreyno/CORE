// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for reports, saved searches, recent searches, case documents,
//! and UI state.

use super::with_project_db;
use crate::project_db::{DbCaseDocument, DbRecentSearch, DbReportRecord, DbSavedSearch};

const MAX_SEARCH_RESPONSE_ROWS: usize = 10_000;
const MAX_SEARCH_FIELD_CHARS: usize = 4096;
const MAX_SEARCH_BODY_CHARS: usize = 16_384;
const MAX_SEARCH_CONFIG_CHARS: usize = 65_536;
const MAX_SEARCH_JSON_DEPTH: usize = 4;
const MAX_SEARCH_JSON_ITEMS: usize = 256;
const SEARCH_TRUNCATED_SUFFIX: &str = "... [truncated]";

// =============================================================================
// Report Commands
// =============================================================================

/// Insert a report record.
#[tauri::command]
pub fn project_db_insert_report(
    window: tauri::Window,
    report: DbReportRecord,
) -> Result<(), String> {
    let report = bounded_report_record(report);
    with_project_db(window.label(), |db| db.insert_report(&report))
}

/// Get all reports.
#[tauri::command]
pub fn project_db_get_reports(window: tauri::Window) -> Result<Vec<DbReportRecord>, String> {
    with_project_db(window.label(), |db| db.get_reports()).map(|reports| {
        reports
            .into_iter()
            .take(MAX_SEARCH_RESPONSE_ROWS)
            .map(bounded_report_record)
            .collect()
    })
}

// =============================================================================
// Search Commands
// =============================================================================

/// Insert or update a saved search.
#[tauri::command]
pub fn project_db_upsert_saved_search(
    window: tauri::Window,
    search: DbSavedSearch,
) -> Result<(), String> {
    let search = bounded_saved_search(search);
    with_project_db(window.label(), |db| db.upsert_saved_search(&search))
}

/// Get all saved searches.
#[tauri::command]
pub fn project_db_get_saved_searches(window: tauri::Window) -> Result<Vec<DbSavedSearch>, String> {
    with_project_db(window.label(), |db| db.get_saved_searches()).map(|searches| {
        searches
            .into_iter()
            .take(MAX_SEARCH_RESPONSE_ROWS)
            .map(bounded_saved_search)
            .collect()
    })
}

/// Insert or update a recent search.
#[tauri::command]
pub fn project_db_insert_recent_search(
    window: tauri::Window,
    search: DbRecentSearch,
) -> Result<(), String> {
    let search = bounded_recent_search(search);
    with_project_db(window.label(), |db| db.insert_recent_search(&search))
}

// =============================================================================
// Case Document Commands
// =============================================================================

/// Insert or update a case document.
#[tauri::command]
pub fn project_db_upsert_case_document(
    window: tauri::Window,
    doc: DbCaseDocument,
) -> Result<(), String> {
    let doc = bounded_case_document(doc);
    with_project_db(window.label(), |db| db.upsert_case_document(&doc))
}

/// Get all case documents.
#[tauri::command]
pub fn project_db_get_case_documents(window: tauri::Window) -> Result<Vec<DbCaseDocument>, String> {
    with_project_db(window.label(), |db| db.get_case_documents()).map(|docs| {
        docs.into_iter()
            .take(MAX_SEARCH_RESPONSE_ROWS)
            .map(bounded_case_document)
            .collect()
    })
}

// =============================================================================
// UI State Commands
// =============================================================================

/// Set a UI state value.
#[tauri::command]
pub fn project_db_set_ui_state(
    window: tauri::Window,
    key: String,
    value: String,
) -> Result<(), String> {
    let key = truncate_search_text(&key, MAX_SEARCH_FIELD_CHARS);
    let value = bounded_search_json_or_text(&value, MAX_SEARCH_CONFIG_CHARS);
    with_project_db(window.label(), |db| db.set_ui_state(&key, &value))
}

/// Get a UI state value.
#[tauri::command]
pub fn project_db_get_ui_state(
    window: tauri::Window,
    key: String,
) -> Result<Option<String>, String> {
    with_project_db(window.label(), |db| db.get_ui_state(&key)).map(|value| {
        value.map(|value| bounded_search_json_or_text(&value, MAX_SEARCH_CONFIG_CHARS))
    })
}

fn bounded_report_record(mut report: DbReportRecord) -> DbReportRecord {
    report.id = truncate_search_text(&report.id, MAX_SEARCH_FIELD_CHARS);
    report.title = truncate_search_text(&report.title, MAX_SEARCH_FIELD_CHARS);
    report.report_type = truncate_search_text(&report.report_type, MAX_SEARCH_FIELD_CHARS);
    report.format = truncate_search_text(&report.format, MAX_SEARCH_FIELD_CHARS);
    report.output_path = report
        .output_path
        .map(|value| truncate_search_text(&value, MAX_SEARCH_FIELD_CHARS));
    report.generated_at = truncate_search_text(&report.generated_at, MAX_SEARCH_FIELD_CHARS);
    report.generated_by = truncate_search_text(&report.generated_by, MAX_SEARCH_FIELD_CHARS);
    report.status = truncate_search_text(&report.status, MAX_SEARCH_FIELD_CHARS);
    report.error = report
        .error
        .map(|value| truncate_search_text(&value, MAX_SEARCH_BODY_CHARS));
    report.config = report
        .config
        .map(|value| bounded_search_json_or_text(&value, MAX_SEARCH_CONFIG_CHARS));
    report
}

fn bounded_saved_search(mut search: DbSavedSearch) -> DbSavedSearch {
    search.id = truncate_search_text(&search.id, MAX_SEARCH_FIELD_CHARS);
    search.name = truncate_search_text(&search.name, MAX_SEARCH_FIELD_CHARS);
    search.query = truncate_search_text(&search.query, MAX_SEARCH_BODY_CHARS);
    search.search_type = truncate_search_text(&search.search_type, MAX_SEARCH_FIELD_CHARS);
    search.scope = bounded_search_json_or_text(&search.scope, MAX_SEARCH_CONFIG_CHARS);
    search.created_at = truncate_search_text(&search.created_at, MAX_SEARCH_FIELD_CHARS);
    search.last_used = search
        .last_used
        .map(|value| truncate_search_text(&value, MAX_SEARCH_FIELD_CHARS));
    search
}

fn bounded_recent_search(mut search: DbRecentSearch) -> DbRecentSearch {
    search.query = truncate_search_text(&search.query, MAX_SEARCH_BODY_CHARS);
    search.timestamp = truncate_search_text(&search.timestamp, MAX_SEARCH_FIELD_CHARS);
    search
}

fn bounded_case_document(mut doc: DbCaseDocument) -> DbCaseDocument {
    doc.id = truncate_search_text(&doc.id, MAX_SEARCH_FIELD_CHARS);
    doc.path = truncate_search_text(&doc.path, MAX_SEARCH_FIELD_CHARS);
    doc.filename = truncate_search_text(&doc.filename, MAX_SEARCH_FIELD_CHARS);
    doc.document_type = truncate_search_text(&doc.document_type, MAX_SEARCH_FIELD_CHARS);
    doc.format = truncate_search_text(&doc.format, MAX_SEARCH_FIELD_CHARS);
    doc.case_number = doc
        .case_number
        .map(|value| truncate_search_text(&value, MAX_SEARCH_FIELD_CHARS));
    doc.evidence_id = doc
        .evidence_id
        .map(|value| truncate_search_text(&value, MAX_SEARCH_FIELD_CHARS));
    doc.modified = doc
        .modified
        .map(|value| truncate_search_text(&value, MAX_SEARCH_FIELD_CHARS));
    doc.discovered_at = truncate_search_text(&doc.discovered_at, MAX_SEARCH_FIELD_CHARS);
    doc
}

fn truncate_search_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = SEARCH_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + SEARCH_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(SEARCH_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_search_json_or_text(value: &str, max_chars: usize) -> String {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return truncate_search_text(value, max_chars);
    };
    let bounded = bounded_search_json_value(json, 0);
    let Ok(serialized) = serde_json::to_string(&bounded) else {
        return "{\"truncated\":true}".to_string();
    };

    if serialized.chars().count() <= max_chars {
        serialized
    } else {
        "{\"truncated\":true}".to_string()
    }
}

fn bounded_search_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_SEARCH_JSON_DEPTH {
        return serde_json::Value::String(SEARCH_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(value) => {
            serde_json::Value::String(truncate_search_text(&value, MAX_SEARCH_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_SEARCH_JSON_ITEMS)
                .map(|value| bounded_search_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(object) => {
            let mut bounded = serde_json::Map::new();
            for (key, value) in object.into_iter().take(MAX_SEARCH_JSON_ITEMS) {
                bounded.insert(
                    truncate_search_text(&key, MAX_SEARCH_FIELD_CHARS),
                    bounded_search_json_value(value, depth + 1),
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

    #[test]
    fn bounded_report_record_caps_config_error_and_paths() {
        let report = DbReportRecord {
            id: "report-1".to_string(),
            title: "t".repeat(MAX_SEARCH_FIELD_CHARS + 32),
            report_type: "forensic".to_string(),
            format: "pdf".to_string(),
            output_path: Some("p".repeat(MAX_SEARCH_FIELD_CHARS + 32)),
            generated_at: "2026-02-16T10:00:00Z".to_string(),
            generated_by: "analyst".to_string(),
            status: "complete".to_string(),
            error: Some("e".repeat(MAX_SEARCH_BODY_CHARS + 32)),
            config: Some(
                serde_json::json!({
                    "sections": (0..(MAX_SEARCH_JSON_ITEMS + 10)).collect::<Vec<_>>(),
                    "deep": {"a": {"b": {"c": {"d": "too deep"}}}}
                })
                .to_string(),
            ),
        };

        let bounded = bounded_report_record(report);

        assert_eq!(bounded.title.chars().count(), MAX_SEARCH_FIELD_CHARS);
        assert_eq!(
            bounded.output_path.as_deref().unwrap().chars().count(),
            MAX_SEARCH_FIELD_CHARS
        );
        assert_eq!(
            bounded.error.as_deref().unwrap().chars().count(),
            MAX_SEARCH_BODY_CHARS
        );
        let config: serde_json::Value =
            serde_json::from_str(bounded.config.as_deref().unwrap()).unwrap();
        assert_eq!(
            config["sections"].as_array().unwrap().len(),
            MAX_SEARCH_JSON_ITEMS
        );
        assert!(bounded
            .config
            .as_deref()
            .unwrap()
            .contains(SEARCH_TRUNCATED_SUFFIX));
    }

    #[test]
    fn bounded_saved_and_recent_search_cap_queries() {
        let saved = DbSavedSearch {
            id: "search-1".to_string(),
            name: "n".repeat(MAX_SEARCH_FIELD_CHARS + 32),
            query: "q".repeat(MAX_SEARCH_BODY_CHARS + 32),
            search_type: "fts".to_string(),
            is_regex: false,
            case_sensitive: false,
            scope: serde_json::json!((0..(MAX_SEARCH_JSON_ITEMS + 10)).collect::<Vec<_>>())
                .to_string(),
            created_at: "2026-02-16T10:00:00Z".to_string(),
            use_count: 0,
            last_used: None,
        };
        let recent = DbRecentSearch {
            query: "r".repeat(MAX_SEARCH_BODY_CHARS + 32),
            timestamp: "2026-02-16T10:00:00Z".to_string(),
            result_count: 10,
        };

        let bounded_saved = bounded_saved_search(saved);
        let bounded_recent = bounded_recent_search(recent);

        assert_eq!(bounded_saved.name.chars().count(), MAX_SEARCH_FIELD_CHARS);
        assert_eq!(bounded_saved.query.chars().count(), MAX_SEARCH_BODY_CHARS);
        assert_eq!(bounded_recent.query.chars().count(), MAX_SEARCH_BODY_CHARS);
        let scope: serde_json::Value = serde_json::from_str(&bounded_saved.scope).unwrap();
        assert_eq!(scope.as_array().unwrap().len(), MAX_SEARCH_JSON_ITEMS);
    }

    #[test]
    fn bounded_case_document_caps_path_fields() {
        let doc = DbCaseDocument {
            id: "doc-1".to_string(),
            path: "p".repeat(MAX_SEARCH_FIELD_CHARS + 32),
            filename: "f".repeat(MAX_SEARCH_FIELD_CHARS + 32),
            document_type: "report".to_string(),
            size: 1024,
            format: "pdf".to_string(),
            case_number: Some("c".repeat(MAX_SEARCH_FIELD_CHARS + 32)),
            evidence_id: Some("ev-1".to_string()),
            modified: None,
            discovered_at: "2026-02-16T10:00:00Z".to_string(),
        };

        let bounded = bounded_case_document(doc);

        assert_eq!(bounded.path.chars().count(), MAX_SEARCH_FIELD_CHARS);
        assert_eq!(bounded.filename.chars().count(), MAX_SEARCH_FIELD_CHARS);
        assert_eq!(
            bounded.case_number.as_deref().unwrap().chars().count(),
            MAX_SEARCH_FIELD_CHARS
        );
    }

    #[test]
    fn bounded_ui_state_preserves_valid_json_and_caps_text() {
        let json = serde_json::json!({
            "items": (0..(MAX_SEARCH_JSON_ITEMS + 10)).collect::<Vec<_>>()
        })
        .to_string();
        let bounded_json = bounded_search_json_or_text(&json, MAX_SEARCH_CONFIG_CHARS);
        let value: serde_json::Value = serde_json::from_str(&bounded_json).unwrap();

        assert_eq!(
            value["items"].as_array().unwrap().len(),
            MAX_SEARCH_JSON_ITEMS
        );

        let text = bounded_search_json_or_text(
            &"x".repeat(MAX_SEARCH_CONFIG_CHARS + 32),
            MAX_SEARCH_CONFIG_CHARS,
        );
        assert_eq!(text.chars().count(), MAX_SEARCH_CONFIG_CHARS);
        assert!(text.ends_with(SEARCH_TRUNCATED_SUFFIX));
    }
}
