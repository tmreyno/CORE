// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Database merge operations — merges multiple .ffxdb files via SQLite ATTACH.

use super::merge_types::{MergeExclusions, MergeStats};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Map a table name to its merge category.
/// Categories allow users to skip entire groups of related tables at once.
fn table_category(table: &str) -> &str {
    match table {
        "evidence_files" | "hashes" | "verifications" | "artifacts" | "source_analyses" => {
            "evidence"
        }
        "bookmarks" | "notes" | "annotations" => "bookmarks_notes",
        "sessions" | "activity_log" | "users" => "activity",
        "reports" => "reports",
        "tags" | "tag_assignments" => "tags",
        "saved_searches" | "recent_searches" => "searches",
        "coc_items" | "coc_amendments" | "coc_audit_log" | "coc_transfers" => "coc",
        "evidence_collections" | "collected_items" | "evidence_data_alternatives" => "collections",
        "form_submissions" => "forms",
        "case_documents" => "documents",
        "export_history" => "exports",
        "processed_databases"
        | "axiom_case_info"
        | "axiom_evidence_sources"
        | "axiom_search_results"
        | "artifact_categories"
        | "processed_db_integrity"
        | "processed_db_metrics" => "processed",
        _ => "other",
    }
}

/// Build a SQL NOT IN clause from a list of IDs, suitable for appending to a WHERE clause.
/// Returns empty string if `ids` is empty.
fn build_not_in_clause(ids: &[String]) -> String {
    if ids.is_empty() {
        return String::new();
    }
    let placeholders: String = ids
        .iter()
        .map(|id| format!("'{}'", id.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");
    placeholders
}

fn source_analyses_merge_sql(conn: &rusqlite::Connection, where_clause: Option<String>) -> String {
    let has_indicators = source_table_has_column(conn, "source_analyses", "indicators_json");
    let indicator_expr = if has_indicators {
        "indicators_json"
    } else {
        "NULL AS indicators_json"
    };
    let where_sql = where_clause
        .map(|clause| format!(" WHERE {clause}"))
        .unwrap_or_default();

    format!(
        "INSERT OR IGNORE INTO source_analyses (
            id, evidence_file_id, source_id, source_ref_json, total_size, offset,
            bytes_analyzed, magic_hex, signature_count, primary_signature,
            primary_mime_type, primary_category, entropy, printable_ratio, is_likely_text,
            ascii_preview, signatures_json, entropy_windows_json, histogram_json,
            indicators_json, analyzed_at, analyzer
         )
         SELECT id, evidence_file_id, source_id, source_ref_json, total_size, offset,
            bytes_analyzed, magic_hex, signature_count, primary_signature,
            primary_mime_type, primary_category, entropy, printable_ratio, is_likely_text,
            ascii_preview, signatures_json, entropy_windows_json, histogram_json,
            {indicator_expr}, analyzed_at, analyzer
         FROM source.source_analyses{where_sql}"
    )
}

fn collected_items_merge_sql(conn: &rusqlite::Connection, where_clause: Option<String>) -> String {
    let source_id_expr = optional_source_column_expr(conn, "collected_items", "source_id");
    let source_ref_expr = optional_source_column_expr(conn, "collected_items", "source_ref_json");
    let item_collection_datetime_expr =
        optional_source_column_expr(conn, "collected_items", "item_collection_datetime");
    let item_system_datetime_expr =
        optional_source_column_expr(conn, "collected_items", "item_system_datetime");
    let item_collecting_officer_expr =
        optional_source_column_expr(conn, "collected_items", "item_collecting_officer");
    let item_authorization_expr =
        optional_source_column_expr(conn, "collected_items", "item_authorization");
    let device_type_expr = optional_source_column_expr(conn, "collected_items", "device_type");
    let device_type_other_expr =
        optional_source_column_expr(conn, "collected_items", "device_type_other");
    let storage_interface_expr =
        optional_source_column_expr(conn, "collected_items", "storage_interface");
    let storage_interface_other_expr =
        optional_source_column_expr(conn, "collected_items", "storage_interface_other");
    let brand_expr = optional_source_column_expr(conn, "collected_items", "brand");
    let color_expr = optional_source_column_expr(conn, "collected_items", "color");
    let imei_expr = optional_source_column_expr(conn, "collected_items", "imei");
    let other_identifiers_expr =
        optional_source_column_expr(conn, "collected_items", "other_identifiers");
    let building_expr = optional_source_column_expr(conn, "collected_items", "building");
    let room_expr = optional_source_column_expr(conn, "collected_items", "room");
    let location_other_expr =
        optional_source_column_expr(conn, "collected_items", "location_other");
    let image_format_expr = optional_source_column_expr(conn, "collected_items", "image_format");
    let image_format_other_expr =
        optional_source_column_expr(conn, "collected_items", "image_format_other");
    let acquisition_method_expr =
        optional_source_column_expr(conn, "collected_items", "acquisition_method");
    let acquisition_method_other_expr =
        optional_source_column_expr(conn, "collected_items", "acquisition_method_other");
    let packaging_type_expr =
        optional_source_column_expr(conn, "collected_items", "packaging_type");
    let packaging_detail_expr =
        optional_source_column_expr(conn, "collected_items", "packaging_detail");
    let hash_algorithm_expr =
        optional_source_column_expr(conn, "collected_items", "hash_algorithm");
    let hash_value_expr = optional_source_column_expr(conn, "collected_items", "hash_value");
    let hash_computed_at_expr =
        optional_source_column_expr(conn, "collected_items", "hash_computed_at");
    let storage_notes_expr = optional_source_column_expr(conn, "collected_items", "storage_notes");
    let where_sql = where_clause
        .map(|clause| format!(" WHERE {clause}"))
        .unwrap_or_default();

    format!(
        "INSERT OR IGNORE INTO collected_items (
            id, collection_id, coc_item_id, evidence_file_id, source_id, source_ref_json,
            item_number, description, found_location, item_type, make, model, serial_number,
            condition, packaging, packaging_type, packaging_detail, photo_refs_json, notes,
            item_collection_datetime, item_system_datetime, item_collecting_officer,
            item_authorization, device_type, device_type_other, storage_interface,
            storage_interface_other, brand, color, imei, other_identifiers, building, room,
            location_other, image_format, image_format_other, acquisition_method,
            acquisition_method_other, hash_algorithm, hash_value, hash_computed_at, storage_notes
         )
         SELECT id, collection_id, coc_item_id, evidence_file_id, {source_id_expr}, {source_ref_expr},
            item_number, description, found_location, item_type, make, model, serial_number,
            condition, packaging, {packaging_type_expr}, {packaging_detail_expr}, photo_refs_json, notes,
            {item_collection_datetime_expr}, {item_system_datetime_expr}, {item_collecting_officer_expr},
            {item_authorization_expr}, {device_type_expr}, {device_type_other_expr}, {storage_interface_expr},
            {storage_interface_other_expr}, {brand_expr}, {color_expr}, {imei_expr}, {other_identifiers_expr}, {building_expr}, {room_expr},
            {location_other_expr}, {image_format_expr}, {image_format_other_expr}, {acquisition_method_expr},
            {acquisition_method_other_expr}, {hash_algorithm_expr}, {hash_value_expr}, {hash_computed_at_expr},
            {storage_notes_expr}
         FROM source.collected_items{where_sql}"
    )
}

fn hashes_merge_sql(conn: &rusqlite::Connection, where_clause: Option<String>) -> String {
    let source_id_expr = optional_source_column_expr(conn, "hashes", "source_id");
    let source_ref_expr = optional_source_column_expr(conn, "hashes", "source_ref_json");
    let where_sql = where_clause
        .map(|clause| format!(" WHERE {clause}"))
        .unwrap_or_default();

    format!(
        "INSERT OR IGNORE INTO hashes (
            id, file_id, source_id, source_ref_json, algorithm, hash_value,
            computed_at, segment_index, segment_name, source
         )
         SELECT id, file_id, {source_id_expr}, {source_ref_expr}, algorithm, hash_value,
            computed_at, segment_index, segment_name, source
         FROM source.hashes{where_sql}"
    )
}

fn coc_items_merge_sql(conn: &rusqlite::Connection, where_clause: Option<String>) -> String {
    let case_title_expr = optional_source_column_expr(conn, "coc_items", "case_title");
    let office_expr = optional_source_column_expr(conn, "coc_items", "office");
    let owner_name_expr = optional_source_column_expr(conn, "coc_items", "owner_name");
    let owner_address_expr = optional_source_column_expr(conn, "coc_items", "owner_address");
    let owner_phone_expr = optional_source_column_expr(conn, "coc_items", "owner_phone");
    let source_expr = optional_source_column_expr(conn, "coc_items", "source");
    let other_contact_name_expr =
        optional_source_column_expr(conn, "coc_items", "other_contact_name");
    let other_contact_relation_expr =
        optional_source_column_expr(conn, "coc_items", "other_contact_relation");
    let other_contact_phone_expr =
        optional_source_column_expr(conn, "coc_items", "other_contact_phone");
    let collection_method_expr =
        optional_source_column_expr(conn, "coc_items", "collection_method");
    let collection_method_other_expr =
        optional_source_column_expr(conn, "coc_items", "collection_method_other");
    let collected_date_expr = optional_source_column_expr(conn, "coc_items", "collected_date");
    let storage_class_expr = optional_source_column_expr(conn, "coc_items", "storage_class");
    let storage_location_detail_expr =
        optional_source_column_expr(conn, "coc_items", "storage_location_detail");
    let disposition_by_expr = optional_source_column_expr(conn, "coc_items", "disposition_by");
    let returned_to_expr = optional_source_column_expr(conn, "coc_items", "returned_to");
    let destruction_date_expr = optional_source_column_expr(conn, "coc_items", "destruction_date");
    let status_expr =
        optional_source_column_expr_or(conn, "coc_items", "status", "'draft' AS status");
    let locked_at_expr = optional_source_column_expr(conn, "coc_items", "locked_at");
    let locked_by_expr = optional_source_column_expr(conn, "coc_items", "locked_by");
    let where_sql = where_clause
        .map(|clause| format!(" WHERE {clause}"))
        .unwrap_or_default();

    format!(
        "INSERT OR IGNORE INTO coc_items (
            id, coc_number, evidence_file_id, case_number, evidence_id, description, item_type,
            case_title, office, owner_name, owner_address, owner_phone, source,
            other_contact_name, other_contact_relation, other_contact_phone,
            collection_method, collection_method_other,
            make, model, serial_number, capacity, condition,
            acquisition_date, entered_custody_date, submitted_by, collected_date, received_by,
            received_location, storage_location, storage_class, storage_location_detail,
            reason_submitted, intake_hashes_json, notes,
            disposition, disposition_by, returned_to, destruction_date, disposition_date, disposition_notes,
            created_at, modified_at, status, locked_at, locked_by
         )
         SELECT id, coc_number, evidence_file_id, case_number, evidence_id, description, item_type,
            {case_title_expr}, {office_expr}, {owner_name_expr}, {owner_address_expr}, {owner_phone_expr}, {source_expr},
            {other_contact_name_expr}, {other_contact_relation_expr}, {other_contact_phone_expr},
            {collection_method_expr}, {collection_method_other_expr},
            make, model, serial_number, capacity, condition,
            acquisition_date, entered_custody_date, submitted_by, {collected_date_expr}, received_by,
            received_location, storage_location, {storage_class_expr}, {storage_location_detail_expr},
            reason_submitted, intake_hashes_json, notes,
            disposition, {disposition_by_expr}, {returned_to_expr}, {destruction_date_expr}, disposition_date, disposition_notes,
            created_at, modified_at, {status_expr}, {locked_at_expr}, {locked_by_expr}
         FROM source.coc_items{where_sql}"
    )
}

fn coc_transfers_merge_sql(conn: &rusqlite::Connection, where_clause: Option<String>) -> String {
    let storage_location_expr =
        optional_source_column_expr(conn, "coc_transfers", "storage_location");
    let storage_class_expr = optional_source_column_expr(conn, "coc_transfers", "storage_class");
    let storage_location_detail_expr =
        optional_source_column_expr(conn, "coc_transfers", "storage_location_detail");
    let storage_date_expr = optional_source_column_expr(conn, "coc_transfers", "storage_date");
    let where_sql = where_clause
        .map(|clause| format!(" WHERE {clause}"))
        .unwrap_or_default();

    format!(
        "INSERT OR IGNORE INTO coc_transfers (
            id, coc_item_id, timestamp, released_by, received_by, purpose, location,
            storage_location, storage_class, storage_location_detail, storage_date, method, notes
         )
         SELECT id, coc_item_id, timestamp, released_by, received_by, purpose, location,
            {storage_location_expr}, {storage_class_expr}, {storage_location_detail_expr},
            {storage_date_expr}, method, notes
         FROM source.coc_transfers{where_sql}"
    )
}

fn evidence_collections_merge_sql(
    conn: &rusqlite::Connection,
    where_clause: Option<String>,
) -> String {
    let status_expr =
        optional_source_column_expr_or(conn, "evidence_collections", "status", "'draft' AS status");
    let where_sql = where_clause
        .map(|clause| format!(" WHERE {clause}"))
        .unwrap_or_default();

    format!(
        "INSERT OR IGNORE INTO evidence_collections (
            id, case_number, collection_date, collection_location, collecting_officer,
            authorization, authorization_date, authorizing_authority, witnesses_json,
            documentation_notes, conditions, status, created_at, modified_at
         )
         SELECT id, case_number, collection_date, collection_location, collecting_officer,
            authorization, authorization_date, authorizing_authority, witnesses_json,
            documentation_notes, conditions, {status_expr}, created_at, modified_at
         FROM source.evidence_collections{where_sql}"
    )
}

fn optional_source_column_expr(conn: &rusqlite::Connection, table: &str, column: &str) -> String {
    if source_table_has_column(conn, table, column) {
        column.to_string()
    } else {
        format!("NULL AS {column}")
    }
}

fn optional_source_column_expr_or(
    conn: &rusqlite::Connection,
    table: &str,
    column: &str,
    fallback_expr: &str,
) -> String {
    if source_table_has_column(conn, table, column) {
        column.to_string()
    } else {
        fallback_expr.to_string()
    }
}

fn source_table_has_column(conn: &rusqlite::Connection, table: &str, column: &str) -> bool {
    let escaped_table = table.replace('\'', "''");
    conn.prepare(&format!("PRAGMA source.table_info('{escaped_table}')"))
        .and_then(|mut stmt| {
            stmt.query_map([], |row| row.get::<_, String>(1))
                .map(|rows| rows.filter_map(|row| row.ok()).any(|name| name == column))
        })
        .unwrap_or(false)
}

/// Merge multiple .ffxdb databases into one using SQLite ATTACH + INSERT OR IGNORE.
///
/// The target database should already exist (created by project_db_open for the merged .cffx).
/// Source databases are attached one at a time and their data merged in.
///
/// `exclusions` — controls which categories and individual items to skip during merge.
pub fn merge_databases(
    target_db_path: &Path,
    source_db_paths: &[PathBuf],
    exclusions: &MergeExclusions,
) -> Result<MergeStats, String> {
    use rusqlite::Connection;

    let conn =
        Connection::open(target_db_path).map_err(|e| format!("Failed to open target DB: {}", e))?;

    // Enable WAL mode for performance
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("Failed to set pragmas: {}", e))?;

    let mut total_stats = MergeStats {
        projects_merged: source_db_paths.len(),
        users_merged: 0,
        sessions_merged: 0,
        activity_entries_merged: 0,
        evidence_files_merged: 0,
        hashes_merged: 0,
        artifacts_merged: 0,
        source_analyses_merged: 0,
        bookmarks_merged: 0,
        notes_merged: 0,
        tabs_merged: 0,
        reports_merged: 0,
        tags_merged: 0,
        searches_merged: 0,
        ffxdb_tables_merged: 0,
    };

    // Tables to merge with their INSERT OR IGNORE statements
    // Each entry: (table_name, insert_sql)
    let merge_tables: Vec<(&str, &str)> = vec![
        ("users",
         "INSERT OR IGNORE INTO users SELECT * FROM source.users"),
        ("sessions",
         "INSERT OR IGNORE INTO sessions SELECT * FROM source.sessions"),
        ("activity_log",
         "INSERT OR IGNORE INTO activity_log SELECT * FROM source.activity_log"),
        ("evidence_files",
         "INSERT OR IGNORE INTO evidence_files SELECT * FROM source.evidence_files"),
        ("hashes", ""),
        ("verifications",
         "INSERT OR IGNORE INTO verifications SELECT * FROM source.verifications"),
        ("artifacts",
         "INSERT OR IGNORE INTO artifacts SELECT * FROM source.artifacts"),
        ("source_analyses", ""),
        ("bookmarks",
         "INSERT OR IGNORE INTO bookmarks SELECT * FROM source.bookmarks"),
        ("notes",
         "INSERT OR IGNORE INTO notes SELECT * FROM source.notes"),
        ("tags",
         "INSERT OR IGNORE INTO tags SELECT * FROM source.tags"),
        ("tag_assignments",
         "INSERT OR IGNORE INTO tag_assignments SELECT * FROM source.tag_assignments"),
        ("reports",
         "INSERT OR IGNORE INTO reports SELECT * FROM source.reports"),
        ("saved_searches",
         "INSERT OR IGNORE INTO saved_searches SELECT * FROM source.saved_searches"),
        ("recent_searches",
         "INSERT OR IGNORE INTO recent_searches SELECT * FROM source.recent_searches"),
        ("case_documents",
         "INSERT OR IGNORE INTO case_documents SELECT * FROM source.case_documents"),
        ("processed_databases",
         "INSERT OR IGNORE INTO processed_databases SELECT * FROM source.processed_databases"),
        ("axiom_case_info",
         "INSERT OR IGNORE INTO axiom_case_info SELECT * FROM source.axiom_case_info"),
        ("axiom_evidence_sources",
         "INSERT OR IGNORE INTO axiom_evidence_sources SELECT * FROM source.axiom_evidence_sources"),
        ("axiom_search_results",
         "INSERT OR IGNORE INTO axiom_search_results SELECT * FROM source.axiom_search_results"),
        ("artifact_categories",
         "INSERT OR IGNORE INTO artifact_categories SELECT * FROM source.artifact_categories"),
        ("coc_items", ""),
        ("coc_amendments",
         "INSERT OR IGNORE INTO coc_amendments SELECT * FROM source.coc_amendments"),
        ("coc_audit_log",
         "INSERT OR IGNORE INTO coc_audit_log SELECT * FROM source.coc_audit_log"),
        ("coc_transfers", ""),
        ("evidence_collections", ""),
        ("collected_items", ""),
        ("evidence_data_alternatives",
         "INSERT OR IGNORE INTO evidence_data_alternatives SELECT * FROM source.evidence_data_alternatives"),
        ("form_submissions",
         "INSERT OR IGNORE INTO form_submissions SELECT * FROM source.form_submissions"),
        ("export_history",
         "INSERT OR IGNORE INTO export_history SELECT * FROM source.export_history"),
        ("annotations",
         "INSERT OR IGNORE INTO annotations SELECT * FROM source.annotations"),
        ("processed_db_integrity",
         "INSERT OR IGNORE INTO processed_db_integrity SELECT * FROM source.processed_db_integrity"),
        ("processed_db_metrics",
         "INSERT OR IGNORE INTO processed_db_metrics SELECT * FROM source.processed_db_metrics"),
        ("ui_state",
         "INSERT OR IGNORE INTO ui_state SELECT * FROM source.ui_state"),
    ];

    // Track temp directories for WAL-replayed source DBs (kept alive until merge completes)
    let mut _temp_dirs: Vec<tempfile::TempDir> = Vec::new();

    for source_path in source_db_paths {
        if !source_path.exists() {
            info!("Skipping non-existent source DB: {:?}", source_path);
            continue;
        }

        // If source has an active WAL file, copy to temp dir first so ATTACH can read WAL data.
        // ATTACH inherits the main connection's mode; even though the target is read-write,
        // the attached database may fail to read WAL data if the WAL file can't be replayed
        // (e.g., permissions, external volume, or stale SHM).
        let wal_path = source_path.with_extension("ffxdb-wal");
        let has_active_wal =
            wal_path.exists() && wal_path.metadata().map(|m| m.len() > 0).unwrap_or(false);

        let attach_path = if has_active_wal {
            info!(
                "Source DB has active WAL, copying to temp for merge: {}",
                source_path.display()
            );
            match tempfile::tempdir() {
                Ok(temp_dir) => {
                    let temp_db = temp_dir.path().join("source_merge.ffxdb");
                    let temp_wal = temp_dir.path().join("source_merge.ffxdb-wal");
                    let temp_shm = temp_dir.path().join("source_merge.ffxdb-shm");

                    if let Err(e) = std::fs::copy(source_path, &temp_db) {
                        warn!(
                            "Failed to copy source DB to temp: {} — {}",
                            source_path.display(),
                            e
                        );
                        continue;
                    }
                    if let Err(e) = std::fs::copy(&wal_path, &temp_wal) {
                        warn!(
                            "Failed to copy source WAL to temp: {} — {}",
                            wal_path.display(),
                            e
                        );
                        continue;
                    }
                    let shm_path = source_path.with_extension("ffxdb-shm");
                    if shm_path.exists() {
                        let _ = std::fs::copy(&shm_path, &temp_shm);
                    }

                    // Open the temp copy read-write to force WAL checkpoint, then close
                    {
                        let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                            | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
                        if let Ok(temp_conn) =
                            rusqlite::Connection::open_with_flags(&temp_db, flags)
                        {
                            let _ = temp_conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
                        }
                    }

                    let path = temp_db.to_string_lossy().to_string();
                    _temp_dirs.push(temp_dir); // Keep temp dir alive
                    path
                }
                Err(e) => {
                    warn!("Failed to create temp dir for WAL merge: {}", e);
                    source_path.to_string_lossy().to_string()
                }
            }
        } else {
            source_path.to_string_lossy().to_string()
        };

        info!("Merging source DB: {}", attach_path);

        // Attach source database
        conn.execute(
            &format!(
                "ATTACH DATABASE '{}' AS source",
                attach_path.replace('\'', "''")
            ),
            [],
        )
        .map_err(|e| format!("Failed to attach {}: {}", attach_path, e))?;

        // Merge each table
        for (table_name, insert_sql) in &merge_tables {
            // Check if this table's category is skipped
            let category = table_category(table_name);
            if exclusions.skip_categories.iter().any(|c| c == category) {
                info!(
                    "  {} → skipped (category '{}' excluded)",
                    table_name, category
                );
                continue;
            }

            // Check if table exists in source
            let table_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM source.sqlite_master WHERE type='table' AND name=?1",
                    [table_name],
                    |row| row.get::<_, i64>(0),
                )
                .map(|c| c > 0)
                .unwrap_or(false);

            if !table_exists {
                continue;
            }

            // Count rows before merge for stats
            let count_before: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);

            // Build effective SQL — apply item-level exclusion filters when needed
            let effective_sql: String = match *table_name {
                // --- Evidence file exclusions ---
                "evidence_files" if !exclusions.exclude_evidence_file_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_evidence_file_ids);
                    format!(
                        "INSERT OR IGNORE INTO evidence_files \
                         SELECT * FROM source.evidence_files WHERE id NOT IN ({})",
                        ids
                    )
                }
                "hashes" if !exclusions.exclude_evidence_file_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_evidence_file_ids);
                    hashes_merge_sql(&conn, Some(format!("file_id NOT IN ({})", ids)))
                }
                "hashes" => hashes_merge_sql(&conn, None),
                "verifications" if !exclusions.exclude_evidence_file_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_evidence_file_ids);
                    format!(
                        "INSERT OR IGNORE INTO verifications \
                         SELECT * FROM source.verifications WHERE hash_id NOT IN (\
                         SELECT id FROM source.hashes WHERE file_id IN ({}))",
                        ids
                    )
                }
                "artifacts" if !exclusions.exclude_evidence_file_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_evidence_file_ids);
                    format!(
                        "INSERT OR IGNORE INTO artifacts \
                         SELECT * FROM source.artifacts WHERE evidence_file_id IS NULL OR evidence_file_id NOT IN ({})",
                        ids
                    )
                }
                "source_analyses" if !exclusions.exclude_evidence_file_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_evidence_file_ids);
                    source_analyses_merge_sql(
                        &conn,
                        Some(format!(
                            "evidence_file_id IS NULL OR evidence_file_id NOT IN ({})",
                            ids
                        )),
                    )
                }
                "source_analyses" => source_analyses_merge_sql(&conn, None),

                // --- COC item exclusions ---
                "coc_items" if !exclusions.exclude_coc_item_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_coc_item_ids);
                    coc_items_merge_sql(&conn, Some(format!("id NOT IN ({})", ids)))
                }
                "coc_items" => coc_items_merge_sql(&conn, None),
                "coc_amendments" if !exclusions.exclude_coc_item_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_coc_item_ids);
                    format!(
                        "INSERT OR IGNORE INTO coc_amendments \
                         SELECT * FROM source.coc_amendments WHERE coc_item_id NOT IN ({})",
                        ids
                    )
                }
                "coc_audit_log" if !exclusions.exclude_coc_item_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_coc_item_ids);
                    format!(
                        "INSERT OR IGNORE INTO coc_audit_log \
                         SELECT * FROM source.coc_audit_log WHERE coc_item_id NOT IN ({})",
                        ids
                    )
                }
                "coc_transfers" if !exclusions.exclude_coc_item_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_coc_item_ids);
                    coc_transfers_merge_sql(&conn, Some(format!("coc_item_id NOT IN ({})", ids)))
                }
                "coc_transfers" => coc_transfers_merge_sql(&conn, None),

                // --- Evidence collection exclusions ---
                "evidence_collections" if !exclusions.exclude_collection_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_collection_ids);
                    evidence_collections_merge_sql(&conn, Some(format!("id NOT IN ({})", ids)))
                }
                "evidence_collections" => evidence_collections_merge_sql(&conn, None),
                "collected_items" if !exclusions.exclude_collection_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_collection_ids);
                    collected_items_merge_sql(
                        &conn,
                        Some(format!("collection_id NOT IN ({})", ids)),
                    )
                }
                "collected_items" => collected_items_merge_sql(&conn, None),

                // --- Form submission exclusions ---
                "form_submissions" if !exclusions.exclude_form_submission_ids.is_empty() => {
                    let ids = build_not_in_clause(&exclusions.exclude_form_submission_ids);
                    format!(
                        "INSERT OR IGNORE INTO form_submissions \
                         SELECT * FROM source.form_submissions WHERE id NOT IN ({})",
                        ids
                    )
                }

                // --- Default: no exclusion filter ---
                _ => insert_sql.to_string(),
            };

            match conn.execute(&effective_sql, []) {
                Ok(inserted) => {
                    if inserted > 0 {
                        info!("  {} → {} rows merged", table_name, inserted);
                        total_stats.ffxdb_tables_merged += 1;
                        match *table_name {
                            "users" => total_stats.users_merged += inserted,
                            "sessions" => total_stats.sessions_merged += inserted,
                            "activity_log" => total_stats.activity_entries_merged += inserted,
                            "evidence_files" => total_stats.evidence_files_merged += inserted,
                            "hashes" => total_stats.hashes_merged += inserted,
                            "artifacts" => total_stats.artifacts_merged += inserted,
                            "source_analyses" => total_stats.source_analyses_merged += inserted,
                            "bookmarks" => total_stats.bookmarks_merged += inserted,
                            "notes" => total_stats.notes_merged += inserted,
                            "reports" => total_stats.reports_merged += inserted,
                            "tags" => total_stats.tags_merged += inserted,
                            "saved_searches" | "recent_searches" => {
                                total_stats.searches_merged += inserted
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    warn!("  {} → merge error (continuing): {}", table_name, e);
                }
            }

            let _ = count_before; // Used for logging context
        }

        // Detach source
        conn.execute("DETACH DATABASE source", [])
            .map_err(|e| format!("Failed to detach {}: {}", attach_path, e))?;
    }

    // Rebuild FTS indexes
    rebuild_fts_indexes(&conn);

    info!(
        "Database merge complete: {} sources processed",
        source_db_paths.len()
    );
    Ok(total_stats)
}

/// Rebuild FTS (Full-Text Search) indexes after merge
fn rebuild_fts_indexes(conn: &rusqlite::Connection) {
    let fts_rebuild_cmds = [
        "INSERT INTO fts_activity_log(fts_activity_log) VALUES('rebuild')",
        "INSERT INTO fts_annotations(fts_annotations) VALUES('rebuild')",
        "INSERT INTO fts_artifacts(fts_artifacts) VALUES('rebuild')",
        "INSERT INTO fts_bookmarks(fts_bookmarks) VALUES('rebuild')",
        "INSERT INTO fts_notes(fts_notes) VALUES('rebuild')",
        "INSERT INTO fts_source_analyses(fts_source_analyses) VALUES('rebuild')",
    ];

    for cmd in &fts_rebuild_cmds {
        match conn.execute(cmd, []) {
            Ok(_) => {}
            Err(e) => warn!("FTS rebuild warning (non-fatal): {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn attach_source_with_collected_items(conn: &Connection, extra_columns: &str) {
        conn.execute_batch(&format!(
            r#"
            ATTACH DATABASE ':memory:' AS source;
            CREATE TABLE source.collected_items (
                id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                coc_item_id TEXT,
                evidence_file_id TEXT,
                item_number TEXT NOT NULL,
                description TEXT NOT NULL,
                found_location TEXT NOT NULL,
                item_type TEXT NOT NULL,
                make TEXT,
                model TEXT,
                serial_number TEXT,
                condition TEXT NOT NULL,
                packaging TEXT NOT NULL,
                packaging_type TEXT,
                packaging_detail TEXT,
                photo_refs_json TEXT,
                notes TEXT,
                item_collection_datetime TEXT,
                item_system_datetime TEXT,
                item_collecting_officer TEXT,
                item_authorization TEXT,
                device_type TEXT,
                device_type_other TEXT,
                storage_interface TEXT,
                storage_interface_other TEXT,
                brand TEXT,
                color TEXT,
                imei TEXT,
                other_identifiers TEXT,
                building TEXT,
                room TEXT,
                location_other TEXT,
                image_format TEXT,
                image_format_other TEXT,
                acquisition_method TEXT,
                acquisition_method_other TEXT,
                storage_notes TEXT
                {extra_columns}
            );
            "#
        ))
        .expect("create source collected_items");
    }

    fn attach_source_with_pre_v12_collected_items(conn: &Connection) {
        conn.execute_batch(
            r#"
            ATTACH DATABASE ':memory:' AS source;
            CREATE TABLE source.collected_items (
                id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                coc_item_id TEXT,
                evidence_file_id TEXT,
                item_number TEXT NOT NULL,
                description TEXT NOT NULL,
                found_location TEXT NOT NULL,
                item_type TEXT NOT NULL,
                make TEXT,
                model TEXT,
                serial_number TEXT,
                condition TEXT NOT NULL,
                packaging TEXT NOT NULL,
                photo_refs_json TEXT,
                notes TEXT,
                item_collection_datetime TEXT,
                item_system_datetime TEXT,
                item_collecting_officer TEXT,
                item_authorization TEXT,
                device_type TEXT,
                device_type_other TEXT,
                storage_interface TEXT,
                storage_interface_other TEXT,
                brand TEXT,
                color TEXT,
                imei TEXT,
                other_identifiers TEXT,
                building TEXT,
                room TEXT,
                location_other TEXT,
                image_format TEXT,
                image_format_other TEXT,
                acquisition_method TEXT,
                acquisition_method_other TEXT,
                storage_notes TEXT
            );
            "#,
        )
        .expect("create pre-v12 source collected_items");
    }

    fn attach_source_with_pre_v8_collected_items(conn: &Connection) {
        conn.execute_batch(
            r#"
            ATTACH DATABASE ':memory:' AS source;
            CREATE TABLE source.collected_items (
                id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                coc_item_id TEXT,
                evidence_file_id TEXT,
                item_number TEXT NOT NULL,
                description TEXT NOT NULL,
                found_location TEXT NOT NULL,
                item_type TEXT NOT NULL,
                make TEXT,
                model TEXT,
                serial_number TEXT,
                condition TEXT NOT NULL,
                packaging TEXT NOT NULL,
                photo_refs_json TEXT,
                notes TEXT
            );
            "#,
        )
        .expect("create pre-v8 source collected_items");
    }

    fn create_current_collected_items_target(conn: &Connection) {
        conn.execute_batch(
            r#"
            CREATE TABLE collected_items (
                id TEXT PRIMARY KEY,
                collection_id TEXT,
                coc_item_id TEXT,
                evidence_file_id TEXT,
                source_id TEXT,
                source_ref_json TEXT,
                item_number TEXT,
                description TEXT,
                found_location TEXT,
                item_type TEXT,
                make TEXT,
                model TEXT,
                serial_number TEXT,
                condition TEXT,
                packaging TEXT,
                packaging_type TEXT,
                packaging_detail TEXT,
                photo_refs_json TEXT,
                notes TEXT,
                item_collection_datetime TEXT,
                item_system_datetime TEXT,
                item_collecting_officer TEXT,
                item_authorization TEXT,
                device_type TEXT,
                device_type_other TEXT,
                storage_interface TEXT,
                storage_interface_other TEXT,
                brand TEXT,
                color TEXT,
                imei TEXT,
                other_identifiers TEXT,
                building TEXT,
                room TEXT,
                location_other TEXT,
                image_format TEXT,
                image_format_other TEXT,
                acquisition_method TEXT,
                acquisition_method_other TEXT,
                hash_algorithm TEXT,
                hash_value TEXT,
                hash_computed_at TEXT,
                storage_notes TEXT
            );
            "#,
        )
        .expect("create current target collected_items");
    }

    fn attach_source_with_hashes(conn: &Connection, extra_columns: &str) {
        conn.execute_batch(&format!(
            r#"
            ATTACH DATABASE ':memory:' AS source;
            CREATE TABLE source.hashes (
                id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL,
                algorithm TEXT NOT NULL,
                hash_value TEXT NOT NULL,
                computed_at TEXT NOT NULL,
                segment_index INTEGER,
                segment_name TEXT,
                source TEXT NOT NULL DEFAULT 'computed'
                {extra_columns}
            );
            "#
        ))
        .expect("create source hashes");
    }

    fn attach_source_with_v4_coc_tables(conn: &Connection) {
        conn.execute_batch(
            r#"
            ATTACH DATABASE ':memory:' AS source;
            CREATE TABLE source.coc_items (
                id TEXT PRIMARY KEY,
                coc_number TEXT NOT NULL,
                evidence_file_id TEXT,
                case_number TEXT NOT NULL,
                evidence_id TEXT NOT NULL,
                description TEXT NOT NULL,
                item_type TEXT NOT NULL,
                make TEXT,
                model TEXT,
                serial_number TEXT,
                capacity TEXT,
                condition TEXT NOT NULL,
                acquisition_date TEXT NOT NULL,
                entered_custody_date TEXT NOT NULL,
                submitted_by TEXT NOT NULL,
                received_by TEXT NOT NULL,
                received_location TEXT,
                storage_location TEXT,
                reason_submitted TEXT,
                intake_hashes_json TEXT,
                notes TEXT,
                disposition TEXT,
                disposition_date TEXT,
                disposition_notes TEXT,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL
            );
            CREATE TABLE source.coc_transfers (
                id TEXT PRIMARY KEY,
                coc_item_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                released_by TEXT NOT NULL,
                received_by TEXT NOT NULL,
                purpose TEXT NOT NULL,
                location TEXT,
                method TEXT,
                notes TEXT
            );
            "#,
        )
        .expect("create v4 source coc tables");
    }

    fn attach_source_with_current_coc_tables(conn: &Connection) {
        conn.execute_batch(
            r#"
            ATTACH DATABASE ':memory:' AS source;
            CREATE TABLE source.coc_items (
                id TEXT PRIMARY KEY,
                coc_number TEXT NOT NULL,
                evidence_file_id TEXT,
                case_number TEXT NOT NULL,
                evidence_id TEXT NOT NULL,
                description TEXT NOT NULL,
                item_type TEXT NOT NULL,
                case_title TEXT,
                office TEXT,
                owner_name TEXT,
                owner_address TEXT,
                owner_phone TEXT,
                source TEXT,
                other_contact_name TEXT,
                other_contact_relation TEXT,
                other_contact_phone TEXT,
                collection_method TEXT,
                collection_method_other TEXT,
                make TEXT,
                model TEXT,
                serial_number TEXT,
                capacity TEXT,
                condition TEXT NOT NULL,
                acquisition_date TEXT NOT NULL,
                entered_custody_date TEXT NOT NULL,
                submitted_by TEXT NOT NULL,
                collected_date TEXT,
                received_by TEXT NOT NULL,
                received_location TEXT,
                storage_location TEXT,
                storage_class TEXT,
                storage_location_detail TEXT,
                reason_submitted TEXT,
                intake_hashes_json TEXT,
                notes TEXT,
                disposition TEXT,
                disposition_by TEXT,
                returned_to TEXT,
                destruction_date TEXT,
                disposition_date TEXT,
                disposition_notes TEXT,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'draft',
                locked_at TEXT,
                locked_by TEXT
            );
            CREATE TABLE source.coc_transfers (
                id TEXT PRIMARY KEY,
                coc_item_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                released_by TEXT NOT NULL,
                received_by TEXT NOT NULL,
                purpose TEXT NOT NULL,
                location TEXT,
                storage_location TEXT,
                storage_class TEXT,
                storage_location_detail TEXT,
                storage_date TEXT,
                method TEXT,
                notes TEXT
            );
            "#,
        )
        .expect("create current source coc tables");
    }

    fn attach_source_with_evidence_collections(conn: &Connection, include_status: bool) {
        let status_column = if include_status {
            ", status TEXT NOT NULL DEFAULT 'draft'"
        } else {
            ""
        };
        conn.execute_batch(&format!(
            r#"
            ATTACH DATABASE ':memory:' AS source;
            CREATE TABLE source.evidence_collections (
                id TEXT PRIMARY KEY,
                case_number TEXT NOT NULL,
                collection_date TEXT NOT NULL,
                collection_location TEXT NOT NULL,
                collecting_officer TEXT NOT NULL,
                authorization TEXT NOT NULL,
                authorization_date TEXT,
                authorizing_authority TEXT,
                witnesses_json TEXT,
                documentation_notes TEXT,
                conditions TEXT
                {status_column},
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL
            );
            "#
        ))
        .expect("create source evidence_collections");
    }

    #[test]
    fn collected_items_merge_sql_defaults_v20_columns_for_old_source() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_collected_items(&conn, "");

        let sql = collected_items_merge_sql(&conn, Some("collection_id NOT IN ('skip')".into()));

        assert!(sql.contains("NULL AS source_id"));
        assert!(sql.contains("NULL AS source_ref_json"));
        assert!(sql.contains("NULL AS hash_algorithm"));
        assert!(sql.contains("NULL AS hash_value"));
        assert!(sql.contains("NULL AS hash_computed_at"));
        assert!(sql.contains("WHERE collection_id NOT IN ('skip')"));
    }

    #[test]
    fn collected_items_merge_sql_defaults_v12_packaging_columns_for_old_source() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_pre_v12_collected_items(&conn);

        let sql = collected_items_merge_sql(&conn, None);

        assert!(sql.contains("condition, packaging, packaging_type, packaging_detail"));
        assert!(sql.contains("condition, packaging, NULL AS packaging_type"));
        assert!(sql.contains("NULL AS packaging_type"));
        assert!(sql.contains("NULL AS packaging_detail"));
        assert!(sql.contains("NULL AS source_id"));
        assert!(sql.contains("NULL AS hash_value"));
    }

    #[test]
    fn collected_items_merge_sql_defaults_v8_columns_for_old_source() {
        let conn = Connection::open_in_memory().expect("open memory db");
        create_current_collected_items_target(&conn);
        attach_source_with_pre_v8_collected_items(&conn);
        conn.execute(
            r#"
            INSERT INTO source.collected_items (
                id, collection_id, item_number, description, found_location, item_type,
                make, model, serial_number, condition, packaging, photo_refs_json, notes
            )
            VALUES (
                'item-1', 'collection-1', '001', 'Phone', 'Desk', 'Mobile Device',
                'Acme', 'One', 'SN123', 'Good', 'Bag', '[]', 'Legacy row'
            )
            "#,
            [],
        )
        .expect("insert pre-v8 collected item");

        let sql = collected_items_merge_sql(&conn, None);
        conn.execute(&sql, [])
            .expect("merge pre-v8 collected item into current target");

        assert!(sql.contains("NULL AS item_collection_datetime"));
        assert!(sql.contains("NULL AS item_system_datetime"));
        assert!(sql.contains("NULL AS item_collecting_officer"));
        assert!(sql.contains("NULL AS item_authorization"));
        assert!(sql.contains("NULL AS device_type"));
        assert!(sql.contains("NULL AS storage_interface"));
        assert!(sql.contains("NULL AS image_format"));
        assert!(sql.contains("NULL AS acquisition_method"));
        assert!(sql.contains("NULL AS storage_notes"));
        assert!(sql.contains("NULL AS packaging_type"));
        assert!(sql.contains("NULL AS source_id"));
        assert!(sql.contains("NULL AS hash_value"));

        let merged: (String, Option<String>, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT item_number, packaging_type, device_type, hash_value FROM collected_items WHERE id = 'item-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("load merged collected item");
        assert_eq!(merged, ("001".to_string(), None, None, None));
    }

    #[test]
    fn collected_items_merge_sql_preserves_v20_columns_for_current_source() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_collected_items(
            &conn,
            ",
                source_id TEXT,
                source_ref_json TEXT,
                hash_algorithm TEXT,
                hash_value TEXT,
                hash_computed_at TEXT",
        );

        let sql = collected_items_merge_sql(&conn, None);

        assert!(sql.contains("evidence_file_id, source_id, source_ref_json"));
        assert!(
            sql.contains("acquisition_method_other, hash_algorithm, hash_value, hash_computed_at")
        );
        assert!(!sql.contains("NULL AS source_id"));
        assert!(!sql.contains("NULL AS hash_value"));
    }

    #[test]
    fn hashes_merge_sql_defaults_v15_columns_for_old_source() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_hashes(&conn, "");

        let sql = hashes_merge_sql(&conn, Some("file_id NOT IN ('ev-skip')".into()));

        assert!(sql.contains("NULL AS source_id"));
        assert!(sql.contains("NULL AS source_ref_json"));
        assert!(sql.contains("WHERE file_id NOT IN ('ev-skip')"));
    }

    #[test]
    fn hashes_merge_sql_preserves_v15_columns_for_current_source() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_hashes(
            &conn,
            ",
                source_id TEXT,
                source_ref_json TEXT",
        );

        let sql = hashes_merge_sql(&conn, None);

        assert!(sql.contains("id, file_id, source_id, source_ref_json"));
        assert!(!sql.contains("NULL AS source_id"));
        assert!(!sql.contains("NULL AS source_ref_json"));
    }

    #[test]
    fn coc_items_merge_sql_defaults_added_columns_for_v4_source() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_v4_coc_tables(&conn);

        let sql = coc_items_merge_sql(&conn, Some("id NOT IN ('coc-skip')".into()));

        assert!(sql.contains("NULL AS case_title"));
        assert!(sql.contains("NULL AS storage_class"));
        assert!(sql.contains("NULL AS storage_location_detail"));
        assert!(sql.contains("'draft' AS status"));
        assert!(sql.contains("WHERE id NOT IN ('coc-skip')"));
    }

    #[test]
    fn coc_items_merge_sql_preserves_current_columns() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_current_coc_tables(&conn);

        let sql = coc_items_merge_sql(&conn, None);

        assert!(sql.contains("case_title, office, owner_name"));
        assert!(sql.contains("storage_location, storage_class, storage_location_detail"));
        assert!(sql.contains("created_at, modified_at, status, locked_at, locked_by"));
        assert!(!sql.contains("NULL AS case_title"));
        assert!(!sql.contains("'draft' AS status"));
    }

    #[test]
    fn coc_transfers_merge_sql_defaults_added_columns_for_v4_source() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_v4_coc_tables(&conn);

        let sql = coc_transfers_merge_sql(&conn, Some("coc_item_id NOT IN ('coc-skip')".into()));

        assert!(sql.contains("NULL AS storage_location"));
        assert!(sql.contains("NULL AS storage_class"));
        assert!(sql.contains("NULL AS storage_location_detail"));
        assert!(sql.contains("NULL AS storage_date"));
        assert!(sql.contains("WHERE coc_item_id NOT IN ('coc-skip')"));
    }

    #[test]
    fn coc_transfers_merge_sql_preserves_current_columns() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_current_coc_tables(&conn);

        let sql = coc_transfers_merge_sql(&conn, None);

        assert!(sql.contains("storage_location, storage_class, storage_location_detail"));
        assert!(sql.contains("storage_date, method, notes"));
        assert!(!sql.contains("NULL AS storage_class"));
    }

    #[test]
    fn evidence_collections_merge_sql_defaults_missing_status() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_evidence_collections(&conn, false);

        let sql =
            evidence_collections_merge_sql(&conn, Some("id NOT IN ('collection-skip')".into()));

        assert!(sql.contains("'draft' AS status"));
        assert!(sql.contains("WHERE id NOT IN ('collection-skip')"));
    }

    #[test]
    fn evidence_collections_merge_sql_preserves_current_status() {
        let conn = Connection::open_in_memory().expect("open memory db");
        attach_source_with_evidence_collections(&conn, true);

        let sql = evidence_collections_merge_sql(&conn, None);

        assert!(sql.contains("conditions, status, created_at"));
        assert!(!sql.contains("'draft' AS status"));
    }
}
