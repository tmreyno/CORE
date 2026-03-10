// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Schema migration logic for the project database.
//!
//! Contains the `check_migrations()` method that upgrades `.ffxdb` databases
//! from older schema versions to the current version (v10).

use super::database::ProjectDatabase;
use super::types::SCHEMA_VERSION;
use rusqlite::{params, Result as SqlResult};
use tracing::info;

impl ProjectDatabase {
    /// Run schema migrations if needed (future-proofing)
    pub(crate) fn check_migrations(&self) -> SqlResult<()> {
        let conn = self.conn.lock();
        let current_version: u32 = conn
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'",
                [],
                |row| {
                    let v: String = row.get(0)?;
                    Ok(v.parse::<u32>().unwrap_or(1))
                },
            )
            .unwrap_or(1);

        if current_version < SCHEMA_VERSION {
            info!(
                "Migrating project DB from v{} to v{}",
                current_version, SCHEMA_VERSION
            );

            // v1 → v2: Add processed database tables
            if current_version < 2 {
                info!("Running v1 → v2 migration: adding processed database tables");
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS processed_databases (
                        id TEXT PRIMARY KEY,
                        path TEXT NOT NULL UNIQUE,
                        name TEXT NOT NULL,
                        db_type TEXT NOT NULL DEFAULT 'Unknown',
                        case_number TEXT,
                        examiner TEXT,
                        created_date TEXT,
                        total_size INTEGER NOT NULL DEFAULT 0,
                        artifact_count INTEGER,
                        notes TEXT,
                        registered_at TEXT NOT NULL,
                        metadata_json TEXT
                    );
                    CREATE TABLE IF NOT EXISTS processed_db_integrity (
                        id TEXT PRIMARY KEY,
                        processed_db_id TEXT NOT NULL,
                        file_path TEXT NOT NULL,
                        file_size INTEGER NOT NULL DEFAULT 0,
                        baseline_hash TEXT NOT NULL,
                        baseline_timestamp TEXT NOT NULL,
                        current_hash TEXT,
                        current_hash_timestamp TEXT,
                        status TEXT NOT NULL DEFAULT 'not_verified',
                        changes_json TEXT,
                        FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS processed_db_metrics (
                        id TEXT PRIMARY KEY,
                        processed_db_id TEXT NOT NULL,
                        total_scans INTEGER NOT NULL DEFAULT 0,
                        last_scan_date TEXT,
                        total_jobs INTEGER NOT NULL DEFAULT 0,
                        last_job_date TEXT,
                        total_notes INTEGER NOT NULL DEFAULT 0,
                        total_tagged_items INTEGER NOT NULL DEFAULT 0,
                        total_users INTEGER NOT NULL DEFAULT 0,
                        user_names_json TEXT,
                        captured_at TEXT NOT NULL,
                        FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS axiom_case_info (
                        id TEXT PRIMARY KEY,
                        processed_db_id TEXT NOT NULL,
                        case_name TEXT NOT NULL,
                        case_number TEXT,
                        case_type TEXT,
                        description TEXT,
                        examiner TEXT,
                        agency TEXT,
                        axiom_version TEXT,
                        search_start TEXT,
                        search_end TEXT,
                        search_duration TEXT,
                        search_outcome TEXT,
                        output_folder TEXT,
                        total_artifacts INTEGER NOT NULL DEFAULT 0,
                        case_path TEXT,
                        captured_at TEXT NOT NULL,
                        keyword_info_json TEXT,
                        FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS axiom_evidence_sources (
                        id TEXT PRIMARY KEY,
                        axiom_case_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        evidence_number TEXT,
                        source_type TEXT NOT NULL DEFAULT 'unknown',
                        path TEXT,
                        hash TEXT,
                        size INTEGER,
                        acquired TEXT,
                        search_types_json TEXT,
                        FOREIGN KEY (axiom_case_id) REFERENCES axiom_case_info(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS axiom_search_results (
                        id TEXT PRIMARY KEY,
                        axiom_case_id TEXT NOT NULL,
                        artifact_type TEXT NOT NULL,
                        hit_count INTEGER NOT NULL DEFAULT 0,
                        FOREIGN KEY (axiom_case_id) REFERENCES axiom_case_info(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS artifact_categories (
                        id TEXT PRIMARY KEY,
                        processed_db_id TEXT NOT NULL,
                        category TEXT NOT NULL,
                        artifact_type TEXT NOT NULL,
                        count INTEGER NOT NULL DEFAULT 0,
                        FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
                    );
                    CREATE INDEX IF NOT EXISTS idx_processed_db_type ON processed_databases(db_type);
                    CREATE INDEX IF NOT EXISTS idx_processed_db_path ON processed_databases(path);
                    CREATE INDEX IF NOT EXISTS idx_processed_integrity_db ON processed_db_integrity(processed_db_id);
                    CREATE INDEX IF NOT EXISTS idx_processed_metrics_db ON processed_db_metrics(processed_db_id);
                    CREATE INDEX IF NOT EXISTS idx_axiom_case_db ON axiom_case_info(processed_db_id);
                    CREATE INDEX IF NOT EXISTS idx_axiom_sources_case ON axiom_evidence_sources(axiom_case_id);
                    CREATE INDEX IF NOT EXISTS idx_axiom_results_case ON axiom_search_results(axiom_case_id);
                    CREATE INDEX IF NOT EXISTS idx_artifact_cats_db ON artifact_categories(processed_db_id);
                    "#,
                )?;
            }

            // v2 → v3: Add forensic workflow tables + FTS5
            if current_version < 3 {
                info!("Running v2 → v3 migration: adding forensic workflow tables");
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS export_history (
                        id TEXT PRIMARY KEY,
                        export_type TEXT NOT NULL,
                        source_paths_json TEXT NOT NULL,
                        destination TEXT NOT NULL,
                        started_at TEXT NOT NULL,
                        completed_at TEXT,
                        initiated_by TEXT NOT NULL,
                        status TEXT NOT NULL DEFAULT 'pending',
                        total_files INTEGER NOT NULL DEFAULT 0,
                        total_bytes INTEGER NOT NULL DEFAULT 0,
                        archive_name TEXT,
                        archive_format TEXT,
                        compression_level TEXT,
                        encrypted INTEGER NOT NULL DEFAULT 0,
                        manifest_hash TEXT,
                        error TEXT,
                        options_json TEXT
                    );
                    CREATE TABLE IF NOT EXISTS chain_of_custody (
                        id TEXT PRIMARY KEY,
                        action TEXT NOT NULL,
                        from_person TEXT NOT NULL,
                        to_person TEXT NOT NULL,
                        date TEXT NOT NULL,
                        time TEXT,
                        location TEXT,
                        purpose TEXT,
                        notes TEXT,
                        evidence_ids_json TEXT,
                        recorded_by TEXT NOT NULL,
                        recorded_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS file_classifications (
                        id TEXT PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        container_path TEXT,
                        classification TEXT NOT NULL,
                        custom_label TEXT,
                        classified_by TEXT NOT NULL,
                        classified_at TEXT NOT NULL,
                        notes TEXT,
                        confidence TEXT
                    );
                    CREATE TABLE IF NOT EXISTS extraction_log (
                        id TEXT PRIMARY KEY,
                        container_path TEXT NOT NULL,
                        entry_path TEXT NOT NULL,
                        output_path TEXT NOT NULL,
                        extracted_by TEXT NOT NULL,
                        extracted_at TEXT NOT NULL,
                        entry_size INTEGER NOT NULL DEFAULT 0,
                        purpose TEXT NOT NULL DEFAULT 'preview',
                        hash_value TEXT,
                        hash_algorithm TEXT,
                        status TEXT NOT NULL DEFAULT 'success',
                        error TEXT
                    );
                    CREATE TABLE IF NOT EXISTS viewer_history (
                        id TEXT PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        container_path TEXT,
                        viewer_type TEXT NOT NULL,
                        viewed_by TEXT NOT NULL,
                        opened_at TEXT NOT NULL,
                        closed_at TEXT,
                        duration_seconds INTEGER
                    );
                    CREATE TABLE IF NOT EXISTS annotations (
                        id TEXT PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        container_path TEXT,
                        annotation_type TEXT NOT NULL,
                        offset_start INTEGER,
                        offset_end INTEGER,
                        line_start INTEGER,
                        line_end INTEGER,
                        label TEXT NOT NULL,
                        content TEXT,
                        color TEXT,
                        created_by TEXT NOT NULL,
                        created_at TEXT NOT NULL,
                        modified_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS evidence_relationships (
                        id TEXT PRIMARY KEY,
                        source_path TEXT NOT NULL,
                        target_path TEXT NOT NULL,
                        relationship_type TEXT NOT NULL,
                        description TEXT,
                        created_by TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_export_status ON export_history(status);
                    CREATE INDEX IF NOT EXISTS idx_export_started ON export_history(started_at);
                    CREATE INDEX IF NOT EXISTS idx_custody_date ON chain_of_custody(date);
                    CREATE INDEX IF NOT EXISTS idx_classification_path ON file_classifications(file_path);
                    CREATE INDEX IF NOT EXISTS idx_classification_type ON file_classifications(classification);
                    CREATE INDEX IF NOT EXISTS idx_extraction_container ON extraction_log(container_path);
                    CREATE INDEX IF NOT EXISTS idx_extraction_at ON extraction_log(extracted_at);
                    CREATE INDEX IF NOT EXISTS idx_viewer_path ON viewer_history(file_path);
                    CREATE INDEX IF NOT EXISTS idx_viewer_opened ON viewer_history(opened_at);
                    CREATE INDEX IF NOT EXISTS idx_annotation_path ON annotations(file_path);
                    CREATE INDEX IF NOT EXISTS idx_relationship_source ON evidence_relationships(source_path);
                    CREATE INDEX IF NOT EXISTS idx_relationship_target ON evidence_relationships(target_path);
                    "#,
                )?;
            }

            // v3 → v4: Add COC items and evidence collection tables
            if current_version < 4 {
                info!("Running v3 → v4 migration: adding COC items & evidence collection tables");
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS coc_items (
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
                        modified_at TEXT NOT NULL,
                        FOREIGN KEY (evidence_file_id) REFERENCES evidence_files(id) ON DELETE SET NULL
                    );
                    CREATE TABLE IF NOT EXISTS coc_transfers (
                        id TEXT PRIMARY KEY,
                        coc_item_id TEXT NOT NULL,
                        timestamp TEXT NOT NULL,
                        released_by TEXT NOT NULL,
                        received_by TEXT NOT NULL,
                        purpose TEXT NOT NULL,
                        location TEXT,
                        method TEXT,
                        notes TEXT,
                        FOREIGN KEY (coc_item_id) REFERENCES coc_items(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS evidence_collections (
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
                        conditions TEXT,
                        status TEXT NOT NULL DEFAULT 'draft',
                        created_at TEXT NOT NULL,
                        modified_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS collected_items (
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
                        storage_notes TEXT,
                        FOREIGN KEY (collection_id) REFERENCES evidence_collections(id) ON DELETE CASCADE,
                        FOREIGN KEY (coc_item_id) REFERENCES coc_items(id) ON DELETE SET NULL,
                        FOREIGN KEY (evidence_file_id) REFERENCES evidence_files(id) ON DELETE SET NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_coc_items_case ON coc_items(case_number);
                    CREATE INDEX IF NOT EXISTS idx_coc_items_evidence ON coc_items(evidence_file_id);
                    CREATE INDEX IF NOT EXISTS idx_coc_transfers_item ON coc_transfers(coc_item_id);
                    CREATE INDEX IF NOT EXISTS idx_collected_items_collection ON collected_items(collection_id);
                    CREATE INDEX IF NOT EXISTS idx_collected_items_coc ON collected_items(coc_item_id);
                    CREATE INDEX IF NOT EXISTS idx_evidence_collections_case ON evidence_collections(case_number);
                    "#,
                )?;
            }

            // v4 → v5: COC immutability model
            if current_version < 5 {
                info!("Running v4 → v5 migration: adding COC immutability model");
                conn.execute_batch(
                    r#"
                    ALTER TABLE coc_items ADD COLUMN status TEXT NOT NULL DEFAULT 'draft';
                    ALTER TABLE coc_items ADD COLUMN locked_at TEXT;
                    ALTER TABLE coc_items ADD COLUMN locked_by TEXT;

                    CREATE TABLE IF NOT EXISTS coc_amendments (
                        id TEXT PRIMARY KEY,
                        coc_item_id TEXT NOT NULL,
                        field_name TEXT NOT NULL,
                        old_value TEXT NOT NULL,
                        new_value TEXT NOT NULL,
                        amended_by_initials TEXT NOT NULL,
                        amended_at TEXT NOT NULL,
                        reason TEXT,
                        FOREIGN KEY (coc_item_id) REFERENCES coc_items(id) ON DELETE RESTRICT
                    );

                    CREATE TABLE IF NOT EXISTS coc_audit_log (
                        id TEXT PRIMARY KEY,
                        coc_item_id TEXT,
                        action TEXT NOT NULL,
                        performed_by TEXT NOT NULL,
                        performed_at TEXT NOT NULL,
                        summary TEXT NOT NULL,
                        details_json TEXT,
                        FOREIGN KEY (coc_item_id) REFERENCES coc_items(id) ON DELETE RESTRICT
                    );

                    CREATE INDEX IF NOT EXISTS idx_coc_amendments_item ON coc_amendments(coc_item_id);
                    CREATE INDEX IF NOT EXISTS idx_coc_amendments_at ON coc_amendments(amended_at);
                    CREATE INDEX IF NOT EXISTS idx_coc_audit_item ON coc_audit_log(coc_item_id);
                    CREATE INDEX IF NOT EXISTS idx_coc_audit_action ON coc_audit_log(action);
                    CREATE INDEX IF NOT EXISTS idx_coc_audit_at ON coc_audit_log(performed_at);
                    "#,
                )?;
            }

            // v5 → v6: Generic form submissions
            if current_version < 6 {
                info!("Running v5 → v6 migration: adding form_submissions table");
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS form_submissions (
                        id TEXT PRIMARY KEY,
                        template_id TEXT NOT NULL,
                        template_version TEXT NOT NULL,
                        case_number TEXT,
                        data_json TEXT NOT NULL DEFAULT '{}',
                        status TEXT NOT NULL DEFAULT 'draft',
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_form_submissions_template ON form_submissions(template_id);
                    CREATE INDEX IF NOT EXISTS idx_form_submissions_case ON form_submissions(case_number);
                    CREATE INDEX IF NOT EXISTS idx_form_submissions_status ON form_submissions(status);
                    "#,
                )?;
            }

            // v6 → v7: Evidence collection status lifecycle
            // NOTE: The v3→v4 migration already creates evidence_collections WITH status column.
            // Only add the column if it's genuinely missing (e.g., a DB created between v4 and v6
            // that somehow lacks it). SQLite has no ALTER TABLE ADD COLUMN IF NOT EXISTS syntax,
            // so we check via PRAGMA table_info.
            if current_version < 7 {
                let has_status: bool = conn
                    .prepare("SELECT COUNT(*) FROM pragma_table_info('evidence_collections') WHERE name = 'status'")?
                    .query_row([], |row| row.get::<_, i64>(0))
                    .map(|count| count > 0)
                    .unwrap_or(false);
                if !has_status {
                    info!(
                        "Running v6 → v7 migration: adding status column to evidence_collections"
                    );
                    conn.execute_batch(
                        r#"
                        ALTER TABLE evidence_collections ADD COLUMN status TEXT NOT NULL DEFAULT 'draft';
                        "#,
                    )?;
                } else {
                    info!("v6 → v7 migration: status column already exists on evidence_collections, skipping");
                }
            }

            // v7 → v8: Expand collected_items with full device/forensic/per-item fields
            // NOTE: The v3→v4 migration already creates collected_items WITH these columns.
            // Only add columns that are genuinely missing.
            if current_version < 8 {
                let columns_to_add = vec![
                    "item_collection_datetime",
                    "item_system_datetime",
                    "item_collecting_officer",
                    "item_authorization",
                    "device_type",
                    "device_type_other",
                    "storage_interface",
                    "storage_interface_other",
                    "brand",
                    "color",
                    "imei",
                    "other_identifiers",
                    "building",
                    "room",
                    "location_other",
                    "image_format",
                    "image_format_other",
                    "acquisition_method",
                    "acquisition_method_other",
                    "storage_notes",
                ];

                let existing_columns: Vec<String> = conn
                    .prepare("SELECT name FROM pragma_table_info('collected_items')")?
                    .query_map([], |row| row.get::<_, String>(0))?
                    .filter_map(|r| r.ok())
                    .collect();

                let mut added = 0;
                for col in &columns_to_add {
                    if !existing_columns.iter().any(|c| c == col) {
                        conn.execute(
                            &format!("ALTER TABLE collected_items ADD COLUMN {} TEXT", col),
                            [],
                        )?;
                        added += 1;
                    }
                }
                if added > 0 {
                    info!(
                        "Running v7 → v8 migration: added {} new columns to collected_items",
                        added
                    );
                } else {
                    info!(
                        "v7 → v8 migration: all columns already exist on collected_items, skipping"
                    );
                }
            }

            // v8 → v9: Form 7-01 COC alignment — new columns on coc_items and coc_transfers
            if current_version < 9 {
                let coc_columns_to_add = vec![
                    "case_title",
                    "office",
                    "owner_name",
                    "owner_address",
                    "owner_phone",
                    "source",
                    "other_contact_name",
                    "other_contact_relation",
                    "other_contact_phone",
                    "collection_method",
                    "collection_method_other",
                    "collected_date",
                    "disposition_by",
                    "returned_to",
                    "destruction_date",
                ];

                let existing_coc_cols: Vec<String> = conn
                    .prepare("SELECT name FROM pragma_table_info('coc_items')")?
                    .query_map([], |row| row.get::<_, String>(0))?
                    .filter_map(|r| r.ok())
                    .collect();

                let mut coc_added = 0;
                for col in &coc_columns_to_add {
                    if !existing_coc_cols.iter().any(|c| c == col) {
                        conn.execute(
                            &format!("ALTER TABLE coc_items ADD COLUMN {} TEXT", col),
                            [],
                        )?;
                        coc_added += 1;
                    }
                }

                // coc_transfers: add storage_location and storage_date
                let existing_transfer_cols: Vec<String> = conn
                    .prepare("SELECT name FROM pragma_table_info('coc_transfers')")?
                    .query_map([], |row| row.get::<_, String>(0))?
                    .filter_map(|r| r.ok())
                    .collect();

                let transfer_columns_to_add = vec!["storage_location", "storage_date"];
                let mut transfer_added = 0;
                for col in &transfer_columns_to_add {
                    if !existing_transfer_cols.iter().any(|c| c == col) {
                        conn.execute(
                            &format!("ALTER TABLE coc_transfers ADD COLUMN {} TEXT", col),
                            [],
                        )?;
                        transfer_added += 1;
                    }
                }

                if coc_added > 0 || transfer_added > 0 {
                    info!(
                        "Running v8 → v9 migration: added {} columns to coc_items, {} to coc_transfers (Form 7-01)",
                        coc_added, transfer_added
                    );
                } else {
                    info!("v8 → v9 migration: all Form 7-01 columns already exist, skipping");
                }
            }

            // v9 → v10: Evidence Data Alternatives (conflict resolution)
            if current_version < 10 {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS evidence_data_alternatives (
                        id TEXT PRIMARY KEY,
                        collected_item_id TEXT NOT NULL,
                        evidence_file_id TEXT,
                        field_name TEXT NOT NULL,
                        chosen_source TEXT NOT NULL DEFAULT 'user',
                        user_value TEXT,
                        container_value TEXT,
                        resolved_by TEXT,
                        resolved_at TEXT NOT NULL,
                        resolution_note TEXT,
                        FOREIGN KEY (collected_item_id) REFERENCES collected_items(id) ON DELETE CASCADE,
                        FOREIGN KEY (evidence_file_id) REFERENCES evidence_files(id) ON DELETE SET NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_eda_collected_item ON evidence_data_alternatives(collected_item_id);
                    CREATE INDEX IF NOT EXISTS idx_eda_evidence_file ON evidence_data_alternatives(evidence_file_id);",
                )?;
                info!("Running v9 → v10 migration: created evidence_data_alternatives table");
            }

            conn.execute(
                "UPDATE schema_meta SET value = ?1 WHERE key = 'schema_version'",
                params![SCHEMA_VERSION.to_string()],
            )?;
        }

        Ok(())
    }
}
