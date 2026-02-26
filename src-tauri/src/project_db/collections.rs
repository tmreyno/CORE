// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence collection and collected item operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

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
        let valid = match (current_status.as_str(), new_status) {
            ("draft", "complete") | ("draft", "locked") | ("complete", "locked") => true,
            _ => false,
        };
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
    pub fn get_evidence_collections(&self, case_number: Option<&str>) -> SqlResult<Vec<DbEvidenceCollection>> {
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
        let refs: Vec<&dyn rusqlite::types::ToSql> = params_slice.iter().map(|p| p.as_ref()).collect();
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
        conn.execute("DELETE FROM evidence_collections WHERE id = ?1", params![id])?;
        Ok(())
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
}
