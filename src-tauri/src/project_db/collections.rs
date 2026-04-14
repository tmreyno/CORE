// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence collection and collected item operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Connection, Result as SqlResult};
use std::collections::{HashMap, HashSet};

fn load_id_set(conn: &Connection, sql: &str) -> SqlResult<HashSet<String>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut ids = HashSet::new();
    for row in rows {
        ids.insert(row?);
    }
    Ok(ids)
}

fn choose_import_id(original_id: &str, used_ids: &mut HashSet<String>) -> String {
    let trimmed = original_id.trim();
    if !trimmed.is_empty() {
        let candidate = trimmed.to_string();
        if used_ids.insert(candidate.clone()) {
            return candidate;
        }
    }

    loop {
        let candidate = uuid::Uuid::new_v4().to_string();
        if used_ids.insert(candidate.clone()) {
            return candidate;
        }
    }
}

fn non_empty_or(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn optional_non_empty_or(value: &Option<String>, fallback: &str) -> Option<String> {
    match value
        .as_deref()
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        Some(entry) => Some(entry.to_string()),
        None if fallback.trim().is_empty() => None,
        None => Some(fallback.to_string()),
    }
}

fn default_import_label(prefix: &str, id: &str) -> String {
    let short_id = id.get(..8).unwrap_or(id);
    format!("{}-{}", prefix, short_id)
}

fn normalize_collection_status(status: &str) -> String {
    match status.trim() {
        "complete" => "complete".to_string(),
        "locked" => "locked".to_string(),
        _ => "draft".to_string(),
    }
}

fn normalize_coc_status(status: &str) -> String {
    match status.trim() {
        "locked" => "locked".to_string(),
        "voided" => "voided".to_string(),
        _ => "draft".to_string(),
    }
}

fn resolve_evidence_file_link(
    evidence_file_id: &Option<String>,
    existing_evidence_file_ids: &HashSet<String>,
    dropped_links: &mut i64,
) -> Option<String> {
    match evidence_file_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) if existing_evidence_file_ids.contains(value) => Some(value.to_string()),
        Some(_) => {
            *dropped_links += 1;
            None
        }
        None => None,
    }
}

fn remap_coc_link(
    coc_item_id: &Option<String>,
    coc_id_map: &HashMap<String, String>,
    dropped_links: &mut i64,
) -> Option<String> {
    match coc_item_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => match coc_id_map.get(value) {
            Some(mapped_id) => Some(mapped_id.clone()),
            None => {
                *dropped_links += 1;
                None
            }
        },
        None => None,
    }
}

fn insert_evidence_collection_row(conn: &Connection, col: &DbEvidenceCollection) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO evidence_collections (id, case_number, collection_date, collection_location, collecting_officer, authorization, authorization_date, authorizing_authority, witnesses_json, documentation_notes, conditions, status, created_at, modified_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            col.id,
            col.case_number,
            col.collection_date,
            col.collection_location,
            col.collecting_officer,
            col.authorization,
            col.authorization_date,
            col.authorizing_authority,
            col.witnesses_json,
            col.documentation_notes,
            col.conditions,
            col.status,
            col.created_at,
            col.modified_at,
        ],
    )?;
    Ok(())
}

fn insert_collected_item_row(conn: &Connection, item: &DbCollectedItem) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO collected_items (
            id, collection_id, coc_item_id, evidence_file_id, item_number, description,
            found_location, item_type, make, model, serial_number, condition, packaging,
            photo_refs_json, notes,
            item_collection_datetime, item_system_datetime, item_collecting_officer, item_authorization,
            device_type, device_type_other, storage_interface, storage_interface_other,
            brand, color, imei, other_identifiers,
            building, room, location_other,
            image_format, image_format_other, acquisition_method, acquisition_method_other,
            storage_notes
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                 ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30,
                 ?31, ?32, ?33, ?34, ?35)",
        params![
            item.id,
            item.collection_id,
            item.coc_item_id,
            item.evidence_file_id,
            item.item_number,
            item.description,
            item.found_location,
            item.item_type,
            item.make,
            item.model,
            item.serial_number,
            item.condition,
            item.packaging,
            item.photo_refs_json,
            item.notes,
            item.item_collection_datetime,
            item.item_system_datetime,
            item.item_collecting_officer,
            item.item_authorization,
            item.device_type,
            item.device_type_other,
            item.storage_interface,
            item.storage_interface_other,
            item.brand,
            item.color,
            item.imei,
            item.other_identifiers,
            item.building,
            item.room,
            item.location_other,
            item.image_format,
            item.image_format_other,
            item.acquisition_method,
            item.acquisition_method_other,
            item.storage_notes,
        ],
    )?;
    Ok(())
}

fn insert_coc_item_row(conn: &Connection, item: &DbCocItem) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO coc_items (
            id, coc_number, evidence_file_id, case_number, evidence_id, description, item_type,
            case_title, office, owner_name, owner_address, owner_phone, source,
            other_contact_name, other_contact_relation, other_contact_phone,
            collection_method, collection_method_other,
            make, model, serial_number, capacity, condition,
            acquisition_date, entered_custody_date, submitted_by, collected_date, received_by,
            received_location, storage_location, reason_submitted, intake_hashes_json, notes,
            disposition, disposition_by, returned_to, destruction_date, disposition_date, disposition_notes,
            created_at, modified_at, status, locked_at, locked_by
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7,
            ?8, ?9, ?10, ?11, ?12, ?13,
            ?14, ?15, ?16,
            ?17, ?18,
            ?19, ?20, ?21, ?22, ?23,
            ?24, ?25, ?26, ?27, ?28,
            ?29, ?30, ?31, ?32, ?33,
            ?34, ?35, ?36, ?37, ?38, ?39,
            ?40, ?41, ?42, ?43, ?44
         )",
        params![
            item.id,
            item.coc_number,
            item.evidence_file_id,
            item.case_number,
            item.evidence_id,
            item.description,
            item.item_type,
            item.case_title,
            item.office,
            item.owner_name,
            item.owner_address,
            item.owner_phone,
            item.source,
            item.other_contact_name,
            item.other_contact_relation,
            item.other_contact_phone,
            item.collection_method,
            item.collection_method_other,
            item.make,
            item.model,
            item.serial_number,
            item.capacity,
            item.condition,
            item.acquisition_date,
            item.entered_custody_date,
            item.submitted_by,
            item.collected_date,
            item.received_by,
            item.received_location,
            item.storage_location,
            item.reason_submitted,
            item.intake_hashes_json,
            item.notes,
            item.disposition,
            item.disposition_by,
            item.returned_to,
            item.destruction_date,
            item.disposition_date,
            item.disposition_notes,
            item.created_at,
            item.modified_at,
            item.status,
            item.locked_at,
            item.locked_by,
        ],
    )?;
    Ok(())
}

fn insert_coc_transfer_row(conn: &Connection, transfer: &DbCocTransfer) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO coc_transfers (id, coc_item_id, timestamp, released_by, received_by, purpose, location, storage_location, storage_date, method, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            transfer.id,
            transfer.coc_item_id,
            transfer.timestamp,
            transfer.released_by,
            transfer.received_by,
            transfer.purpose,
            transfer.location,
            transfer.storage_location,
            transfer.storage_date,
            transfer.method,
            transfer.notes,
        ],
    )?;
    Ok(())
}

fn insert_coc_amendment_row(conn: &Connection, amendment: &DbCocAmendment) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO coc_amendments (id, coc_item_id, field_name, old_value, new_value, amended_by_initials, amended_at, reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            amendment.id,
            amendment.coc_item_id,
            amendment.field_name,
            amendment.old_value,
            amendment.new_value,
            amendment.amended_by_initials,
            amendment.amended_at,
            amendment.reason,
        ],
    )?;
    Ok(())
}

fn insert_coc_audit_row(conn: &Connection, audit: &DbCocAuditEntry) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO coc_audit_log (id, coc_item_id, action, performed_by, performed_at, summary, details_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            audit.id,
            audit.coc_item_id,
            audit.action,
            audit.performed_by,
            audit.performed_at,
            audit.summary,
            audit.details_json,
        ],
    )?;
    Ok(())
}

impl ProjectDatabase {
    // ========================================================================
    // Evidence Collection Operations
    // ========================================================================

    /// Upsert an evidence collection record
    pub fn upsert_evidence_collection(&self, col: &DbEvidenceCollection) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO evidence_collections (id, case_number, collection_date, collection_location, collecting_officer, authorization, authorization_date, authorizing_authority, witnesses_json, documentation_notes, conditions, status, created_at, modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                case_number=excluded.case_number, collection_date=excluded.collection_date,
                collection_location=excluded.collection_location, collecting_officer=excluded.collecting_officer,
                authorization=excluded.authorization, authorization_date=excluded.authorization_date,
                authorizing_authority=excluded.authorizing_authority, witnesses_json=excluded.witnesses_json,
                documentation_notes=excluded.documentation_notes, conditions=excluded.conditions,
                status=excluded.status, modified_at=excluded.modified_at",
            params![
                col.id, col.case_number, col.collection_date, col.collection_location,
                col.collecting_officer, col.authorization, col.authorization_date,
                col.authorizing_authority, col.witnesses_json, col.documentation_notes,
                col.conditions, col.status, col.created_at, col.modified_at,
            ],
        )?;
        Ok(())
    }

    /// Get a single evidence collection by ID
    pub fn get_evidence_collection_by_id(&self, id: &str) -> SqlResult<DbEvidenceCollection> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT ec.id, ec.case_number, ec.collection_date, ec.collection_location, ec.collecting_officer, ec.authorization, ec.authorization_date, ec.authorizing_authority, ec.witnesses_json, ec.documentation_notes, ec.conditions, ec.status, ec.created_at, ec.modified_at,
                    (SELECT COUNT(*) FROM collected_items ci WHERE ci.collection_id = ec.id) as item_count
             FROM evidence_collections ec WHERE ec.id = ?1",
            params![id],
            |row| {
                Ok(DbEvidenceCollection {
                    id: row.get(0)?,
                    case_number: row.get(1)?,
                    collection_date: row.get(2)?,
                    collection_location: row.get(3)?,
                    collecting_officer: row.get(4)?,
                    authorization: row.get(5)?,
                    authorization_date: row.get(6)?,
                    authorizing_authority: row.get(7)?,
                    witnesses_json: row.get(8)?,
                    documentation_notes: row.get(9)?,
                    conditions: row.get(10)?,
                    status: row.get(11)?,
                    created_at: row.get(12)?,
                    modified_at: row.get(13)?,
                    item_count: row.get(14)?,
                })
            },
        )
    }

    /// Update evidence collection status (draft → complete → locked)
    pub fn update_evidence_collection_status(&self, id: &str, new_status: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        // Validate status transitions
        let current_status: String = conn.query_row(
            "SELECT status FROM evidence_collections WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let valid = matches!(
            (current_status.as_str(), new_status),
            ("draft", "complete") | ("draft", "locked") | ("complete", "locked")
        );
        if !valid {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        conn.execute(
            "UPDATE evidence_collections SET status = ?1, modified_at = datetime('now') WHERE id = ?2",
            params![new_status, id],
        )?;
        Ok(())
    }

    /// Get all evidence collections, optionally filtered by case number (with item counts)
    pub fn get_evidence_collections(
        &self,
        case_number: Option<&str>,
    ) -> SqlResult<Vec<DbEvidenceCollection>> {
        let conn = self.conn.lock();
        let sql = if case_number.is_some() {
            "SELECT ec.id, ec.case_number, ec.collection_date, ec.collection_location, ec.collecting_officer, ec.authorization, ec.authorization_date, ec.authorizing_authority, ec.witnesses_json, ec.documentation_notes, ec.conditions, ec.status, ec.created_at, ec.modified_at,
                    (SELECT COUNT(*) FROM collected_items ci WHERE ci.collection_id = ec.id) as item_count
             FROM evidence_collections ec WHERE ec.case_number = ?1 ORDER BY ec.collection_date DESC"
        } else {
            "SELECT ec.id, ec.case_number, ec.collection_date, ec.collection_location, ec.collecting_officer, ec.authorization, ec.authorization_date, ec.authorizing_authority, ec.witnesses_json, ec.documentation_notes, ec.conditions, ec.status, ec.created_at, ec.modified_at,
                    (SELECT COUNT(*) FROM collected_items ci WHERE ci.collection_id = ec.id) as item_count
             FROM evidence_collections ec ORDER BY ec.collection_date DESC"
        };
        let mut stmt = conn.prepare(sql)?;
        let params_slice: Vec<Box<dyn rusqlite::types::ToSql>> = if let Some(cn) = case_number {
            vec![Box::new(cn.to_string())]
        } else {
            vec![]
        };
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            params_slice.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), |row| {
            Ok(DbEvidenceCollection {
                id: row.get(0)?,
                case_number: row.get(1)?,
                collection_date: row.get(2)?,
                collection_location: row.get(3)?,
                collecting_officer: row.get(4)?,
                authorization: row.get(5)?,
                authorization_date: row.get(6)?,
                authorizing_authority: row.get(7)?,
                witnesses_json: row.get(8)?,
                documentation_notes: row.get(9)?,
                conditions: row.get(10)?,
                status: row.get(11)?,
                created_at: row.get(12)?,
                modified_at: row.get(13)?,
                item_count: row.get(14)?,
            })
        })?;
        rows.collect()
    }

    /// Delete an evidence collection (cascades to collected_items)
    pub fn delete_evidence_collection(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM evidence_collections WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Import a portable evidence collection package into the current project.
    pub fn import_evidence_collection_package(
        &self,
        package: &ImportedEvidenceCollectionPackage,
    ) -> SqlResult<EvidenceCollectionPackageImportSummary> {
        let conn = self.conn.lock();
        conn.execute_batch("BEGIN IMMEDIATE")?;

        let now = chrono::Utc::now().to_rfc3339();
        let result = (|| -> SqlResult<EvidenceCollectionPackageImportSummary> {
            let mut existing_collection_ids =
                load_id_set(&conn, "SELECT id FROM evidence_collections")?;
            let mut existing_item_ids = load_id_set(&conn, "SELECT id FROM collected_items")?;
            let existing_evidence_file_ids = load_id_set(&conn, "SELECT id FROM evidence_files")?;

            let mut coc_id_map = HashMap::with_capacity(package.coc_items.len());
            for entry in &package.coc_items {
                coc_id_map.insert(entry.item.id.clone(), uuid::Uuid::new_v4().to_string());
            }

            let mut imported_collections = 0i64;
            let mut imported_items = 0i64;
            let mut imported_coc_items = 0i64;
            let mut dropped_evidence_file_links = 0i64;
            let mut dropped_coc_links = 0i64;

            for entry in &package.coc_items {
                let mapped_coc_id = coc_id_map
                    .get(&entry.item.id)
                    .cloned()
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                let mut item = entry.item.clone();
                item.id = mapped_coc_id.clone();
                item.case_number = non_empty_or(&item.case_number, &package.source_case_number);
                item.coc_number = non_empty_or(
                    &item.coc_number,
                    &default_import_label("COC", &mapped_coc_id),
                );
                item.evidence_id = non_empty_or(
                    &item.evidence_id,
                    &default_import_label("EVID", &mapped_coc_id),
                );
                item.description = non_empty_or(&item.description, "Imported evidence item");
                item.item_type = non_empty_or(&item.item_type, "Evidence");
                item.case_title =
                    optional_non_empty_or(&item.case_title, &package.source_case_title);
                item.submitted_by = non_empty_or(&item.submitted_by, &package.source_examiner_name);
                item.received_by = non_empty_or(&item.received_by, &item.submitted_by);
                item.condition = non_empty_or(&item.condition, "Unknown");
                item.created_at = non_empty_or(&item.created_at, &now);
                item.modified_at = non_empty_or(&item.modified_at, &item.created_at);
                item.acquisition_date = non_empty_or(&item.acquisition_date, &item.created_at);
                item.entered_custody_date =
                    non_empty_or(&item.entered_custody_date, &item.acquisition_date);
                item.status = normalize_coc_status(&item.status);
                item.evidence_file_id = resolve_evidence_file_link(
                    &item.evidence_file_id,
                    &existing_evidence_file_ids,
                    &mut dropped_evidence_file_links,
                );

                insert_coc_item_row(&conn, &item)?;

                for transfer_entry in &entry.transfers {
                    let mut transfer = transfer_entry.clone();
                    transfer.id = uuid::Uuid::new_v4().to_string();
                    transfer.coc_item_id = mapped_coc_id.clone();
                    transfer.timestamp = non_empty_or(&transfer.timestamp, &item.modified_at);
                    transfer.released_by = non_empty_or(&transfer.released_by, &item.submitted_by);
                    transfer.received_by = non_empty_or(&transfer.received_by, &item.received_by);
                    transfer.purpose = non_empty_or(&transfer.purpose, "Imported transfer");
                    insert_coc_transfer_row(&conn, &transfer)?;
                }

                for amendment_entry in &entry.amendments {
                    let mut amendment = amendment_entry.clone();
                    amendment.id = uuid::Uuid::new_v4().to_string();
                    amendment.coc_item_id = mapped_coc_id.clone();
                    amendment.field_name = non_empty_or(&amendment.field_name, "notes");
                    amendment.old_value = amendment.old_value.trim().to_string();
                    amendment.new_value = amendment.new_value.trim().to_string();
                    amendment.amended_by_initials =
                        non_empty_or(&amendment.amended_by_initials, "IMP");
                    amendment.amended_at = non_empty_or(&amendment.amended_at, &item.modified_at);
                    insert_coc_amendment_row(&conn, &amendment)?;
                }

                if entry.audit_log.is_empty() {
                    let audit = DbCocAuditEntry {
                        id: uuid::Uuid::new_v4().to_string(),
                        coc_item_id: Some(mapped_coc_id.clone()),
                        action: "imported".to_string(),
                        performed_by: item.submitted_by.clone(),
                        performed_at: now.clone(),
                        summary: format!(
                            "Imported from {} evidence collection package",
                            package.source_app
                        ),
                        details_json: Some(
                            serde_json::json!({
                                "sourceCaseNumber": package.source_case_number,
                                "originalCocItemId": entry.item.id,
                            })
                            .to_string(),
                        ),
                    };
                    insert_coc_audit_row(&conn, &audit)?;
                } else {
                    for audit_entry in &entry.audit_log {
                        let mut audit = audit_entry.clone();
                        audit.id = uuid::Uuid::new_v4().to_string();
                        audit.coc_item_id = Some(mapped_coc_id.clone());
                        audit.action = non_empty_or(&audit.action, "imported");
                        audit.performed_by = non_empty_or(&audit.performed_by, &item.submitted_by);
                        audit.performed_at = non_empty_or(&audit.performed_at, &item.modified_at);
                        audit.summary = non_empty_or(
                            &audit.summary,
                            &format!(
                                "Imported from {} evidence collection package",
                                package.source_app
                            ),
                        );
                        insert_coc_audit_row(&conn, &audit)?;
                    }
                }

                imported_coc_items += 1;
            }

            for collection_entry in &package.collections {
                let mut collection = collection_entry.collection.clone();
                collection.id = choose_import_id(&collection.id, &mut existing_collection_ids);
                collection.case_number =
                    non_empty_or(&collection.case_number, &package.source_case_number);
                collection.collection_date = non_empty_or(&collection.collection_date, &now);
                collection.collection_location =
                    non_empty_or(&collection.collection_location, "Imported package");
                collection.collecting_officer = non_empty_or(
                    &collection.collecting_officer,
                    &package.source_examiner_name,
                );
                collection.authorization =
                    non_empty_or(&collection.authorization, "Imported package");
                collection.created_at = non_empty_or(&collection.created_at, &now);
                collection.modified_at =
                    non_empty_or(&collection.modified_at, &collection.created_at);
                collection.status = normalize_collection_status(&collection.status);
                collection.item_count = 0;

                insert_evidence_collection_row(&conn, &collection)?;
                imported_collections += 1;

                for (index, item_entry) in collection_entry.items.iter().enumerate() {
                    let mut item = item_entry.clone();
                    item.id = choose_import_id(&item.id, &mut existing_item_ids);
                    item.collection_id = collection.id.clone();
                    item.coc_item_id =
                        remap_coc_link(&item.coc_item_id, &coc_id_map, &mut dropped_coc_links);
                    item.evidence_file_id = resolve_evidence_file_link(
                        &item.evidence_file_id,
                        &existing_evidence_file_ids,
                        &mut dropped_evidence_file_links,
                    );
                    item.item_number =
                        non_empty_or(&item.item_number, &format!("ITEM-{:03}", index + 1));
                    item.description = non_empty_or(&item.description, "Imported item");
                    item.found_location =
                        non_empty_or(&item.found_location, &collection.collection_location);
                    item.item_type = non_empty_or(&item.item_type, "Evidence");
                    item.condition = non_empty_or(&item.condition, "Unknown");
                    item.packaging = non_empty_or(&item.packaging, "Unknown");

                    insert_collected_item_row(&conn, &item)?;
                    imported_items += 1;
                }
            }

            Ok(EvidenceCollectionPackageImportSummary {
                source_app: package.source_app.clone(),
                source_case_number: package.source_case_number.clone(),
                imported_collections,
                imported_items,
                imported_coc_items,
                dropped_evidence_file_links,
                dropped_coc_links,
            })
        })();

        match result {
            Ok(summary) => {
                conn.execute_batch("COMMIT")?;
                Ok(summary)
            }
            Err(error) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    // ========================================================================
    // Collected Item Operations
    // ========================================================================

    /// Upsert a collected item
    pub fn upsert_collected_item(&self, item: &DbCollectedItem) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO collected_items (
                id, collection_id, coc_item_id, evidence_file_id, item_number, description,
                found_location, item_type, make, model, serial_number, condition, packaging,
                photo_refs_json, notes,
                item_collection_datetime, item_system_datetime, item_collecting_officer, item_authorization,
                device_type, device_type_other, storage_interface, storage_interface_other,
                brand, color, imei, other_identifiers,
                building, room, location_other,
                image_format, image_format_other, acquisition_method, acquisition_method_other,
                storage_notes
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                     ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30,
                     ?31, ?32, ?33, ?34, ?35)
             ON CONFLICT(id) DO UPDATE SET
                collection_id=excluded.collection_id, coc_item_id=excluded.coc_item_id,
                evidence_file_id=excluded.evidence_file_id, item_number=excluded.item_number,
                description=excluded.description, found_location=excluded.found_location,
                item_type=excluded.item_type, make=excluded.make, model=excluded.model,
                serial_number=excluded.serial_number, condition=excluded.condition,
                packaging=excluded.packaging, photo_refs_json=excluded.photo_refs_json,
                notes=excluded.notes,
                item_collection_datetime=excluded.item_collection_datetime,
                item_system_datetime=excluded.item_system_datetime,
                item_collecting_officer=excluded.item_collecting_officer,
                item_authorization=excluded.item_authorization,
                device_type=excluded.device_type, device_type_other=excluded.device_type_other,
                storage_interface=excluded.storage_interface, storage_interface_other=excluded.storage_interface_other,
                brand=excluded.brand, color=excluded.color, imei=excluded.imei,
                other_identifiers=excluded.other_identifiers,
                building=excluded.building, room=excluded.room, location_other=excluded.location_other,
                image_format=excluded.image_format, image_format_other=excluded.image_format_other,
                acquisition_method=excluded.acquisition_method, acquisition_method_other=excluded.acquisition_method_other,
                storage_notes=excluded.storage_notes",
            params![
                item.id, item.collection_id, item.coc_item_id, item.evidence_file_id,
                item.item_number, item.description, item.found_location, item.item_type,
                item.make, item.model, item.serial_number, item.condition,
                item.packaging, item.photo_refs_json, item.notes,
                item.item_collection_datetime, item.item_system_datetime,
                item.item_collecting_officer, item.item_authorization,
                item.device_type, item.device_type_other,
                item.storage_interface, item.storage_interface_other,
                item.brand, item.color, item.imei, item.other_identifiers,
                item.building, item.room, item.location_other,
                item.image_format, item.image_format_other,
                item.acquisition_method, item.acquisition_method_other,
                item.storage_notes,
            ],
        )?;
        Ok(())
    }

    /// Get collected items for a collection
    pub fn get_collected_items(&self, collection_id: &str) -> SqlResult<Vec<DbCollectedItem>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, collection_id, coc_item_id, evidence_file_id, item_number, description,
                    found_location, item_type, make, model, serial_number, condition, packaging,
                    photo_refs_json, notes,
                    item_collection_datetime, item_system_datetime, item_collecting_officer, item_authorization,
                    device_type, device_type_other, storage_interface, storage_interface_other,
                    brand, color, imei, other_identifiers,
                    building, room, location_other,
                    image_format, image_format_other, acquisition_method, acquisition_method_other,
                    storage_notes
             FROM collected_items WHERE collection_id = ?1 ORDER BY item_number ASC",
        )?;
        let rows = stmt.query_map(params![collection_id], Self::map_collected_item)?;
        rows.collect()
    }

    /// Get all collected items across all collections
    pub fn get_all_collected_items(&self) -> SqlResult<Vec<DbCollectedItem>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, collection_id, coc_item_id, evidence_file_id, item_number, description,
                    found_location, item_type, make, model, serial_number, condition, packaging,
                    photo_refs_json, notes,
                    item_collection_datetime, item_system_datetime, item_collecting_officer, item_authorization,
                    device_type, device_type_other, storage_interface, storage_interface_other,
                    brand, color, imei, other_identifiers,
                    building, room, location_other,
                    image_format, image_format_other, acquisition_method, acquisition_method_other,
                    storage_notes
             FROM collected_items ORDER BY item_number ASC",
        )?;
        let rows = stmt.query_map([], Self::map_collected_item)?;
        rows.collect()
    }

    /// Delete a collected item
    pub fn delete_collected_item(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM collected_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Row mapper for DbCollectedItem (35 columns)
    fn map_collected_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<DbCollectedItem> {
        Ok(DbCollectedItem {
            id: row.get(0)?,
            collection_id: row.get(1)?,
            coc_item_id: row.get(2)?,
            evidence_file_id: row.get(3)?,
            item_number: row.get(4)?,
            description: row.get(5)?,
            found_location: row.get(6)?,
            item_type: row.get(7)?,
            make: row.get(8)?,
            model: row.get(9)?,
            serial_number: row.get(10)?,
            condition: row.get(11)?,
            packaging: row.get(12)?,
            photo_refs_json: row.get(13)?,
            notes: row.get(14)?,
            item_collection_datetime: row.get(15)?,
            item_system_datetime: row.get(16)?,
            item_collecting_officer: row.get(17)?,
            item_authorization: row.get(18)?,
            device_type: row.get(19)?,
            device_type_other: row.get(20)?,
            storage_interface: row.get(21)?,
            storage_interface_other: row.get(22)?,
            brand: row.get(23)?,
            color: row.get(24)?,
            imei: row.get(25)?,
            other_identifiers: row.get(26)?,
            building: row.get(27)?,
            room: row.get(28)?,
            location_other: row.get(29)?,
            image_format: row.get(30)?,
            image_format_other: row.get(31)?,
            acquisition_method: row.get(32)?,
            acquisition_method_other: row.get(33)?,
            storage_notes: row.get(34)?,
        })
    }

    // ========================================================================
    // Evidence Data Alternatives (Conflict Resolution)
    // ========================================================================

    /// Upsert an evidence data alternative record
    pub fn upsert_evidence_data_alternative(
        &self,
        alt: &DbEvidenceDataAlternative,
    ) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO evidence_data_alternatives (
                id, collected_item_id, evidence_file_id, field_name,
                chosen_source, user_value, container_value,
                resolved_by, resolved_at, resolution_note
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                chosen_source=excluded.chosen_source, user_value=excluded.user_value,
                container_value=excluded.container_value, resolved_by=excluded.resolved_by,
                resolved_at=excluded.resolved_at, resolution_note=excluded.resolution_note",
            params![
                alt.id,
                alt.collected_item_id,
                alt.evidence_file_id,
                alt.field_name,
                alt.chosen_source,
                alt.user_value,
                alt.container_value,
                alt.resolved_by,
                alt.resolved_at,
                alt.resolution_note,
            ],
        )?;
        Ok(())
    }

    /// Get all alternative data records for a collected item
    pub fn get_evidence_data_alternatives(
        &self,
        collected_item_id: &str,
    ) -> SqlResult<Vec<DbEvidenceDataAlternative>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, collected_item_id, evidence_file_id, field_name,
                    chosen_source, user_value, container_value,
                    resolved_by, resolved_at, resolution_note
             FROM evidence_data_alternatives
             WHERE collected_item_id = ?1
             ORDER BY field_name ASC",
        )?;
        let rows = stmt.query_map(params![collected_item_id], |row| {
            Ok(DbEvidenceDataAlternative {
                id: row.get(0)?,
                collected_item_id: row.get(1)?,
                evidence_file_id: row.get(2)?,
                field_name: row.get(3)?,
                chosen_source: row.get(4)?,
                user_value: row.get(5)?,
                container_value: row.get(6)?,
                resolved_by: row.get(7)?,
                resolved_at: row.get(8)?,
                resolution_note: row.get(9)?,
            })
        })?;
        rows.collect()
    }

    /// Get all alternative data records for a specific evidence file
    pub fn get_evidence_data_alternatives_by_file(
        &self,
        evidence_file_id: &str,
    ) -> SqlResult<Vec<DbEvidenceDataAlternative>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, collected_item_id, evidence_file_id, field_name,
                    chosen_source, user_value, container_value,
                    resolved_by, resolved_at, resolution_note
             FROM evidence_data_alternatives
             WHERE evidence_file_id = ?1
             ORDER BY collected_item_id, field_name ASC",
        )?;
        let rows = stmt.query_map(params![evidence_file_id], |row| {
            Ok(DbEvidenceDataAlternative {
                id: row.get(0)?,
                collected_item_id: row.get(1)?,
                evidence_file_id: row.get(2)?,
                field_name: row.get(3)?,
                chosen_source: row.get(4)?,
                user_value: row.get(5)?,
                container_value: row.get(6)?,
                resolved_by: row.get(7)?,
                resolved_at: row.get(8)?,
                resolution_note: row.get(9)?,
            })
        })?;
        rows.collect()
    }

    /// Delete an evidence data alternative record
    pub fn delete_evidence_data_alternative(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM evidence_data_alternatives WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Delete all alternative data records for a collected item
    pub fn delete_evidence_data_alternatives_for_item(
        &self,
        collected_item_id: &str,
    ) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM evidence_data_alternatives WHERE collected_item_id = ?1",
            params![collected_item_id],
        )?;
        Ok(())
    }
}
