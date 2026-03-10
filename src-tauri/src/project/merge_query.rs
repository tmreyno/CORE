// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Query helpers for extracting data from .ffxdb databases during merge analysis.

use super::merge_types::*;
use tracing::warn;

/// Query users table from .ffxdb and add to examiners list
pub(super) fn query_ffxdb_examiners(
    conn: &rusqlite::Connection,
    examiners: &mut Vec<MergeExaminerInfo>,
) {
    let sql = "SELECT username, display_name FROM users";
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        let (username, display_name) = row;
                        if !examiners
                            .iter()
                            .any(|e| e.name.eq_ignore_ascii_case(&username))
                        {
                            examiners.push(MergeExaminerInfo {
                                name: username,
                                display_name,
                                source: "ffxdb".to_string(),
                                role: "session user".to_string(),
                            });
                        }
                    }
                }
                Err(e) => warn!("merge: query users failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare users query failed: {}", e),
    }
}

/// Query evidence_collections from .ffxdb
pub(super) fn query_ffxdb_collections(conn: &rusqlite::Connection) -> Vec<MergeCollectionSummary> {
    let sql = "SELECT ec.id, ec.case_number, ec.collection_date, ec.collecting_officer, \
               ec.collection_location, ec.status, \
               (SELECT COUNT(*) FROM collected_items ci WHERE ci.collection_id = ec.id) as item_count \
               FROM evidence_collections ec ORDER BY ec.collection_date DESC";
    let mut results = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok(MergeCollectionSummary {
                    id: row.get(0)?,
                    case_number: row.get(1)?,
                    collection_date: row.get(2)?,
                    collecting_officer: row.get(3)?,
                    collection_location: row.get(4)?,
                    status: row.get(5)?,
                    item_count: row.get::<_, i64>(6).unwrap_or(0) as usize,
                })
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        results.push(row);
                    }
                }
                Err(e) => warn!("merge: query evidence_collections failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare evidence_collections query failed: {}", e),
    }
    results
}

/// Query coc_items from .ffxdb
pub(super) fn query_ffxdb_coc_items(conn: &rusqlite::Connection) -> Vec<MergeCocSummary> {
    let sql = "SELECT id, coc_number, case_number, evidence_id, description, \
               submitted_by, received_by, status FROM coc_items ORDER BY created_at DESC";
    let mut results = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok(MergeCocSummary {
                    id: row.get(0)?,
                    coc_number: row.get(1)?,
                    case_number: row.get(2)?,
                    evidence_id: row.get(3)?,
                    description: row.get(4)?,
                    submitted_by: row.get(5)?,
                    received_by: row.get(6)?,
                    status: row.get(7)?,
                })
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        results.push(row);
                    }
                }
                Err(e) => warn!("merge: query coc_items failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare coc_items query failed: {}", e),
    }
    results
}

/// Query form_submissions from .ffxdb (includes data_json for detail extraction)
pub(super) fn query_ffxdb_forms(conn: &rusqlite::Connection) -> Vec<MergeFormSummary> {
    let sql = "SELECT id, template_id, case_number, status, created_at, data_json \
               FROM form_submissions ORDER BY created_at DESC";
    let mut results = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                let data_json: Option<String> = row.get(5)?;
                // Extract useful fields from data_json
                let (collecting_officer, collection_location, lead_examiner) =
                    extract_form_details(data_json.as_deref());
                Ok(MergeFormSummary {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    case_number: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                    collecting_officer,
                    collection_location,
                    lead_examiner,
                })
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        results.push(row);
                    }
                }
                Err(e) => warn!("merge: query form_submissions failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare form_submissions query failed: {}", e),
    }
    results
}

/// Extract useful display fields from form_submissions.data_json
pub(super) fn extract_form_details(
    data_json: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>) {
    let Some(json_str) = data_json else {
        return (None, None, None);
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return (None, None, None);
    };
    let obj = val.as_object();
    let get_str = |key: &str| -> Option<String> {
        obj.and_then(|o| o.get(key))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    };
    (
        get_str("collecting_officer"),
        get_str("collection_location"),
        get_str("lead_examiner"),
    )
}

/// Query evidence_files from .ffxdb
pub(super) fn query_ffxdb_evidence_files(
    conn: &rusqlite::Connection,
) -> Vec<MergeEvidenceFileSummary> {
    let sql = "SELECT id, path, filename, container_type, total_size \
               FROM evidence_files ORDER BY filename ASC";
    let mut results = Vec::new();
    match conn.prepare(sql) {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok(MergeEvidenceFileSummary {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    filename: row.get(2)?,
                    container_type: row.get(3)?,
                    total_size: row.get::<_, i64>(4).unwrap_or(0) as u64,
                })
            }) {
                Ok(rows) => {
                    for row in rows.flatten() {
                        results.push(row);
                    }
                }
                Err(e) => warn!("merge: query evidence_files failed: {}", e),
            }
        }
        Err(e) => warn!("merge: prepare evidence_files query failed: {}", e),
    }
    results
}

/// Query processed_databases and axiom_case_info for examiner names
pub(super) fn query_ffxdb_processed_db_examiners(
    conn: &rusqlite::Connection,
    examiners: &mut Vec<MergeExaminerInfo>,
) {
    // processed_databases.examiner
    let sql = "SELECT DISTINCT examiner FROM processed_databases WHERE examiner IS NOT NULL AND examiner != ''";
    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for name in rows.flatten() {
                if !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(&name)) {
                    examiners.push(MergeExaminerInfo {
                        name,
                        display_name: None,
                        source: "ffxdb".to_string(),
                        role: "processed DB examiner".to_string(),
                    });
                }
            }
        }
    }
    // axiom_case_info.examiner
    let sql2 = "SELECT DISTINCT examiner FROM axiom_case_info WHERE examiner IS NOT NULL AND examiner != ''";
    if let Ok(mut stmt) = conn.prepare(sql2) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for name in rows.flatten() {
                if !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(&name)) {
                    examiners.push(MergeExaminerInfo {
                        name,
                        display_name: None,
                        source: "ffxdb".to_string(),
                        role: "AXIOM examiner".to_string(),
                    });
                }
            }
        }
    }
}

/// Query additional tables for user/examiner clues when owner is still unknown.
/// Checks sessions, activity_log, bookmarks, notes, reports, export_history,
/// and chain_of_custody for distinct usernames not already in the examiner list.
pub(super) fn query_ffxdb_additional_clues(
    conn: &rusqlite::Connection,
    examiners: &mut Vec<MergeExaminerInfo>,
) {
    // Each tuple: (SQL, role label)
    let queries: &[(&str, &str)] = &[
        (
            "SELECT DISTINCT user FROM sessions WHERE user IS NOT NULL AND user != ''",
            "session user",
        ),
        (
            "SELECT DISTINCT user FROM activity_log WHERE user IS NOT NULL AND user != ''",
            "activity user",
        ),
        (
            "SELECT DISTINCT created_by FROM bookmarks WHERE created_by IS NOT NULL AND created_by != ''",
            "bookmark author",
        ),
        (
            "SELECT DISTINCT created_by FROM notes WHERE created_by IS NOT NULL AND created_by != ''",
            "note author",
        ),
        (
            "SELECT DISTINCT generated_by FROM reports WHERE generated_by IS NOT NULL AND generated_by != ''",
            "report author",
        ),
        (
            "SELECT DISTINCT initiated_by FROM export_history WHERE initiated_by IS NOT NULL AND initiated_by != ''",
            "export initiator",
        ),
        (
            "SELECT DISTINCT recorded_by FROM chain_of_custody WHERE recorded_by IS NOT NULL AND recorded_by != ''",
            "COC recorder",
        ),
        (
            "SELECT DISTINCT from_person FROM chain_of_custody WHERE from_person IS NOT NULL AND from_person != ''",
            "COC from",
        ),
        (
            "SELECT DISTINCT to_person FROM chain_of_custody WHERE to_person IS NOT NULL AND to_person != ''",
            "COC to",
        ),
    ];

    for (sql, role) in queries {
        if let Ok(mut stmt) = conn.prepare(sql) {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                for name in rows.flatten() {
                    if !examiners.iter().any(|e| e.name.eq_ignore_ascii_case(&name)) {
                        examiners.push(MergeExaminerInfo {
                            name,
                            display_name: None,
                            source: "ffxdb".to_string(),
                            role: role.to_string(),
                        });
                    }
                }
            }
        }
    }
}
