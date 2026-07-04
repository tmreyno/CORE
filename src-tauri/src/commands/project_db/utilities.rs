// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for FTS search, database utilities, and form submissions.

use super::with_project_db;
use crate::project_db::{DbFormSubmission, FtsSearchResult};

const MAX_UTILITY_RESPONSE_ROWS: usize = 10_000;
const MAX_UTILITY_FTS_ROWS: usize = 1_000;
const MAX_UTILITY_FIELD_CHARS: usize = 4096;
const MAX_UTILITY_TEXT_CHARS: usize = 65_536;
const MAX_UTILITY_JSON_CHARS: usize = 65_536;
const MAX_UTILITY_JSON_DEPTH: usize = 4;
const MAX_UTILITY_JSON_ITEMS: usize = 256;
const UTILITY_TRUNCATED_SUFFIX: &str = "... [truncated]";

// =============================================================================
// Full-Text Search Commands
// =============================================================================

/// Rebuild FTS5 indexes from source tables.
#[tauri::command]
pub fn project_db_rebuild_fts(window: tauri::Window) -> Result<(), String> {
    with_project_db(window.label(), |db| db.rebuild_fts_indexes())
}

/// Full-text search across notes, bookmarks, and activity log.
#[tauri::command]
pub fn project_db_fts_search(
    window: tauri::Window,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<FtsSearchResult>, String> {
    let query = truncate_utility_text(&query, MAX_UTILITY_TEXT_CHARS);
    let limit = limit
        .unwrap_or(MAX_UTILITY_FTS_ROWS as i64)
        .clamp(1, MAX_UTILITY_FTS_ROWS as i64);
    with_project_db(window.label(), |db| {
        db.fts_search(&query, Some(limit)).map(|results| {
            results
                .into_iter()
                .take(MAX_UTILITY_FTS_ROWS)
                .map(bounded_fts_result)
                .collect()
        })
    })
}

// =============================================================================
// Database Utility Commands
// =============================================================================

/// Run SQLite integrity check on the project database.
#[tauri::command]
pub fn project_db_integrity_check(window: tauri::Window) -> Result<Vec<String>, String> {
    with_project_db(window.label(), |db| {
        db.integrity_check().map(|rows| {
            rows.into_iter()
                .take(MAX_UTILITY_RESPONSE_ROWS)
                .map(|row| truncate_utility_text(&row, MAX_UTILITY_TEXT_CHARS))
                .collect()
        })
    })
}

/// Force WAL checkpoint (flush write-ahead log to main DB file).
#[tauri::command]
pub fn project_db_wal_checkpoint(
    window: tauri::Window,
    mode: Option<String>,
) -> Result<(i64, i64), String> {
    let mode = mode.map(|value| truncate_utility_text(&value, MAX_UTILITY_FIELD_CHARS));
    with_project_db(window.label(), |db| match mode.as_deref() {
        Some("passive") => db.wal_checkpoint_passive(),
        _ => db.wal_checkpoint(),
    })
}

/// Create a backup copy of the project database.
#[tauri::command]
pub fn project_db_backup(window: tauri::Window, dest_path: String) -> Result<(), String> {
    let dest_path = truncate_utility_text(&dest_path, MAX_UTILITY_TEXT_CHARS);
    with_project_db(window.label(), |db| db.backup_to(&dest_path))
}

/// Vacuum the database to reclaim space.
#[tauri::command]
pub fn project_db_vacuum(window: tauri::Window) -> Result<(), String> {
    with_project_db(window.label(), |db| db.vacuum())
}

// =============================================================================
// Form Submission Commands (Generic JSON-driven forms)
// =============================================================================

/// Upsert (insert or update) a form submission.
#[tauri::command]
pub fn project_db_upsert_form_submission(
    window: tauri::Window,
    submission: DbFormSubmission,
) -> Result<(), String> {
    let submission = bounded_form_submission(submission);
    with_project_db(window.label(), |db| db.upsert_form_submission(&submission))
}

/// Get a form submission by ID.
#[tauri::command]
pub fn project_db_get_form_submission(
    window: tauri::Window,
    id: String,
) -> Result<Option<DbFormSubmission>, String> {
    let id = truncate_utility_text(&id, MAX_UTILITY_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_form_submission(&id)
            .map(|submission| submission.map(bounded_form_submission))
    })
}

/// List form submissions with optional filters.
#[tauri::command]
pub fn project_db_list_form_submissions(
    window: tauri::Window,
    template_id: Option<String>,
    case_number: Option<String>,
    status: Option<String>,
) -> Result<Vec<DbFormSubmission>, String> {
    let template_id =
        template_id.map(|value| truncate_utility_text(&value, MAX_UTILITY_FIELD_CHARS));
    let case_number =
        case_number.map(|value| truncate_utility_text(&value, MAX_UTILITY_FIELD_CHARS));
    let status = status.map(|value| truncate_utility_text(&value, MAX_UTILITY_FIELD_CHARS));
    with_project_db(window.label(), |db| {
        db.list_form_submissions(
            template_id.as_deref(),
            case_number.as_deref(),
            status.as_deref(),
        )
        .map(|submissions| {
            submissions
                .into_iter()
                .take(MAX_UTILITY_RESPONSE_ROWS)
                .map(bounded_form_submission)
                .collect()
        })
    })
}

/// Delete a form submission (only draft status).
#[tauri::command]
pub fn project_db_delete_form_submission(window: tauri::Window, id: String) -> Result<(), String> {
    let id = truncate_utility_text(&id, MAX_UTILITY_FIELD_CHARS);
    with_project_db(window.label(), |db| db.delete_form_submission(&id))
}

fn bounded_fts_result(mut result: FtsSearchResult) -> FtsSearchResult {
    result.source = truncate_utility_text(&result.source, MAX_UTILITY_FIELD_CHARS);
    result.id = truncate_utility_text(&result.id, MAX_UTILITY_FIELD_CHARS);
    result.snippet = truncate_utility_text(&result.snippet, MAX_UTILITY_TEXT_CHARS);
    result
}

fn bounded_form_submission(mut submission: DbFormSubmission) -> DbFormSubmission {
    submission.id = truncate_utility_text(&submission.id, MAX_UTILITY_FIELD_CHARS);
    submission.template_id =
        truncate_utility_text(&submission.template_id, MAX_UTILITY_FIELD_CHARS);
    submission.template_version =
        truncate_utility_text(&submission.template_version, MAX_UTILITY_FIELD_CHARS);
    submission.case_number = submission
        .case_number
        .map(|value| truncate_utility_text(&value, MAX_UTILITY_FIELD_CHARS));
    submission.data_json =
        bounded_utility_json_or_text(&submission.data_json, MAX_UTILITY_JSON_CHARS);
    submission.status = truncate_utility_text(&submission.status, MAX_UTILITY_FIELD_CHARS);
    submission.created_at = truncate_utility_text(&submission.created_at, MAX_UTILITY_FIELD_CHARS);
    submission.updated_at = truncate_utility_text(&submission.updated_at, MAX_UTILITY_FIELD_CHARS);
    submission
}

fn truncate_utility_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let keep_chars = max_chars.saturating_sub(UTILITY_TRUNCATED_SUFFIX.chars().count());
    let mut truncated: String = value.chars().take(keep_chars).collect();
    truncated.push_str(UTILITY_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_utility_json_or_text(value: &str, max_chars: usize) -> String {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(value) else {
        return truncate_utility_text(value, max_chars);
    };
    let bounded = bounded_utility_json_value(parsed, 0);
    match serde_json::to_string(&bounded) {
        Ok(serialized) => truncate_utility_text(&serialized, max_chars),
        Err(_) => truncate_utility_text(value, max_chars),
    }
}

fn bounded_utility_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_UTILITY_JSON_DEPTH {
        return serde_json::Value::String(UTILITY_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(text) => {
            serde_json::Value::String(truncate_utility_text(&text, MAX_UTILITY_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_UTILITY_JSON_ITEMS)
                .map(|value| bounded_utility_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(entries) => serde_json::Value::Object(
            entries
                .into_iter()
                .take(MAX_UTILITY_JSON_ITEMS)
                .map(|(key, value)| {
                    (
                        truncate_utility_text(&key, MAX_UTILITY_FIELD_CHARS),
                        bounded_utility_json_value(value, depth + 1),
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
    fn bounded_fts_result_caps_text_fields() {
        let result = bounded_fts_result(FtsSearchResult {
            source: repeated(MAX_UTILITY_FIELD_CHARS + 8),
            id: repeated(MAX_UTILITY_FIELD_CHARS + 8),
            snippet: repeated(MAX_UTILITY_TEXT_CHARS + 8),
            rank: 1.0,
        });

        assert_eq!(result.source.chars().count(), MAX_UTILITY_FIELD_CHARS);
        assert_eq!(result.id.chars().count(), MAX_UTILITY_FIELD_CHARS);
        assert_eq!(result.snippet.chars().count(), MAX_UTILITY_TEXT_CHARS);
        assert!(result.snippet.ends_with(UTILITY_TRUNCATED_SUFFIX));
    }

    #[test]
    fn bounded_form_submission_caps_json_and_metadata() {
        let submission = bounded_form_submission(DbFormSubmission {
            id: repeated(MAX_UTILITY_FIELD_CHARS + 8),
            template_id: repeated(MAX_UTILITY_FIELD_CHARS + 8),
            template_version: "1".to_string(),
            case_number: Some(repeated(MAX_UTILITY_FIELD_CHARS + 8)),
            data_json: serde_json::to_string(&vec![repeated(MAX_UTILITY_FIELD_CHARS + 8)]).unwrap(),
            status: "draft".to_string(),
            created_at: "2026-02-16T10:00:00Z".to_string(),
            updated_at: "2026-02-16T10:00:00Z".to_string(),
        });

        assert_eq!(submission.id.chars().count(), MAX_UTILITY_FIELD_CHARS);
        assert_eq!(
            submission.template_id.chars().count(),
            MAX_UTILITY_FIELD_CHARS
        );
        assert_eq!(
            submission.case_number.unwrap().chars().count(),
            MAX_UTILITY_FIELD_CHARS
        );
        let data: serde_json::Value = serde_json::from_str(&submission.data_json).unwrap();
        assert_eq!(
            data[0].as_str().unwrap().chars().count(),
            MAX_UTILITY_FIELD_CHARS
        );
    }

    #[test]
    fn bounded_form_submission_caps_deep_or_plain_text_payloads() {
        let deep = bounded_utility_json_or_text(
            "{\"a\":{\"b\":{\"c\":{\"d\":\"too-deep\"}}}}",
            MAX_UTILITY_JSON_CHARS,
        );
        let plain = bounded_utility_json_or_text(
            &repeated(MAX_UTILITY_JSON_CHARS + 8),
            MAX_UTILITY_JSON_CHARS,
        );

        assert!(deep.contains(UTILITY_TRUNCATED_SUFFIX));
        assert_eq!(plain.chars().count(), MAX_UTILITY_JSON_CHARS);
        assert!(plain.ends_with(UTILITY_TRUNCATED_SUFFIX));
    }
}
