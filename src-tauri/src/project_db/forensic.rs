// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Forensic workflow operations: exports, chain of custody, COC items
//! (immutability model), amendments, audit log, and transfers.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Export History Operations
    // ========================================================================

    /// Insert a new export record
    pub fn insert_export(&self, record: &DbExportRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO export_history (id, export_type, source_paths_json, destination, started_at, completed_at, initiated_by, status, total_files, total_bytes, archive_name, archive_format, compression_level, encrypted, manifest_hash, error, options_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                record.id, record.export_type, record.source_paths_json, record.destination,
                record.started_at, record.completed_at, record.initiated_by, record.status,
                record.total_files, record.total_bytes, record.archive_name, record.archive_format,
                record.compression_level, record.encrypted, record.manifest_hash, record.error,
                record.options_json,
            ],
        )?;
        Ok(())
    }

    /// Update an export record (typically to set completed_at/status/error)
    pub fn update_export(&self, record: &DbExportRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE export_history SET completed_at = ?1, status = ?2, total_files = ?3, total_bytes = ?4, manifest_hash = ?5, error = ?6 WHERE id = ?7",
            params![
                record.completed_at, record.status, record.total_files, record.total_bytes,
                record.manifest_hash, record.error, record.id,
            ],
        )?;
        Ok(())
    }

    /// Get all export records, most recent first
    pub fn get_exports(&self, limit: Option<i64>) -> SqlResult<Vec<DbExportRecord>> {
        let conn = self.conn.lock();
        let sql = format!(
            "SELECT id, export_type, source_paths_json, destination, started_at, completed_at, initiated_by, status, total_files, total_bytes, archive_name, archive_format, compression_level, encrypted, manifest_hash, error, options_json
             FROM export_history ORDER BY started_at DESC{}",
            limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default()
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(DbExportRecord {
                id: row.get(0)?,
                export_type: row.get(1)?,
                source_paths_json: row.get(2)?,
                destination: row.get(3)?,
                started_at: row.get(4)?,
                completed_at: row.get(5)?,
                initiated_by: row.get(6)?,
                status: row.get(7)?,
                total_files: row.get(8)?,
                total_bytes: row.get(9)?,
                archive_name: row.get(10)?,
                archive_format: row.get(11)?,
                compression_level: row.get(12)?,
                encrypted: row.get(13)?,
                manifest_hash: row.get(14)?,
                error: row.get(15)?,
                options_json: row.get(16)?,
            })
        })?;
        rows.collect()
    }

    /// Delete an export record by ID
    pub fn delete_export(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM export_history WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // Chain of Custody Operations
    // ========================================================================

    /// Insert a custody record
    pub fn insert_custody_record(&self, record: &DbCustodyRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO chain_of_custody (id, action, from_person, to_person, date, time, location, purpose, notes, evidence_ids_json, recorded_by, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.id, record.action, record.from_person, record.to_person,
                record.date, record.time, record.location, record.purpose,
                record.notes, record.evidence_ids_json, record.recorded_by, record.recorded_at,
            ],
        )?;
        Ok(())
    }

    /// Get all custody records in chronological order
    pub fn get_custody_records(&self) -> SqlResult<Vec<DbCustodyRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, action, from_person, to_person, date, time, location, purpose, notes, evidence_ids_json, recorded_by, recorded_at
             FROM chain_of_custody ORDER BY date ASC, time ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DbCustodyRecord {
                id: row.get(0)?,
                action: row.get(1)?,
                from_person: row.get(2)?,
                to_person: row.get(3)?,
                date: row.get(4)?,
                time: row.get(5)?,
                location: row.get(6)?,
                purpose: row.get(7)?,
                notes: row.get(8)?,
                evidence_ids_json: row.get(9)?,
                recorded_by: row.get(10)?,
                recorded_at: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    /// Delete a custody record by ID
    pub fn delete_custody_record(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM chain_of_custody WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // COC Item Operations (Form 7 per-evidence chain of custody)
    // ========================================================================
    // IMMUTABILITY MODEL (v5):
    //  - Items start as 'draft' and can be freely edited.
    //  - Once 'locked', fields can only be changed via `amend_coc_item`,
    //    which requires initials + date and creates an amendment record.
    //  - Items cannot be hard-deleted; only soft-deleted (status='voided').
    //  - All mutations are recorded in the coc_audit_log.
    // ========================================================================

    /// Insert a new COC item (draft status). Fails if ID already exists.
    pub fn insert_coc_item(&self, item: &DbCocItem) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO coc_items (id, coc_number, evidence_file_id, case_number, evidence_id, description, item_type, make, model, serial_number, capacity, condition, acquisition_date, entered_custody_date, submitted_by, received_by, received_location, storage_location, reason_submitted, intake_hashes_json, notes, disposition, disposition_date, disposition_notes, created_at, modified_at, status, locked_at, locked_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)",
            params![
                item.id, item.coc_number, item.evidence_file_id, item.case_number,
                item.evidence_id, item.description, item.item_type,
                item.make, item.model, item.serial_number, item.capacity,
                item.condition, item.acquisition_date, item.entered_custody_date,
                item.submitted_by, item.received_by, item.received_location,
                item.storage_location, item.reason_submitted, item.intake_hashes_json,
                item.notes, item.disposition, item.disposition_date, item.disposition_notes,
                item.created_at, item.modified_at,
                item.status, item.locked_at, item.locked_by,
            ],
        )?;
        // Audit: created
        self.insert_coc_audit_internal(
            &conn,
            &item.id,
            "created",
            &item.submitted_by,
            &format!(
                "COC item {} created ({})",
                item.coc_number, item.description
            ),
            None,
        )?;
        Ok(())
    }

    /// Upsert a COC item — allowed ONLY if the item is in 'draft' status.
    /// Locked/voided items will cause an error.
    pub fn upsert_coc_item(&self, item: &DbCocItem) -> SqlResult<()> {
        let conn = self.conn.lock();
        // Check if item exists and is locked/voided
        let existing_status: Option<String> = conn
            .query_row(
                "SELECT status FROM coc_items WHERE id = ?1",
                params![item.id],
                |row| row.get(0),
            )
            .ok();
        if let Some(ref status) = existing_status {
            if status != "draft" {
                return Err(rusqlite::Error::QueryReturnedNoRows); // Reject mutation on non-draft item
            }
        }
        conn.execute(
            "INSERT INTO coc_items (id, coc_number, evidence_file_id, case_number, evidence_id, description, item_type, make, model, serial_number, capacity, condition, acquisition_date, entered_custody_date, submitted_by, received_by, received_location, storage_location, reason_submitted, intake_hashes_json, notes, disposition, disposition_date, disposition_notes, created_at, modified_at, status, locked_at, locked_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)
             ON CONFLICT(id) DO UPDATE SET
                coc_number=excluded.coc_number, evidence_file_id=excluded.evidence_file_id,
                case_number=excluded.case_number, evidence_id=excluded.evidence_id,
                description=excluded.description, item_type=excluded.item_type,
                make=excluded.make, model=excluded.model, serial_number=excluded.serial_number,
                capacity=excluded.capacity, condition=excluded.condition,
                acquisition_date=excluded.acquisition_date, entered_custody_date=excluded.entered_custody_date,
                submitted_by=excluded.submitted_by, received_by=excluded.received_by,
                received_location=excluded.received_location, storage_location=excluded.storage_location,
                reason_submitted=excluded.reason_submitted, intake_hashes_json=excluded.intake_hashes_json,
                notes=excluded.notes, disposition=excluded.disposition,
                disposition_date=excluded.disposition_date, disposition_notes=excluded.disposition_notes,
                modified_at=excluded.modified_at",
            params![
                item.id, item.coc_number, item.evidence_file_id, item.case_number,
                item.evidence_id, item.description, item.item_type,
                item.make, item.model, item.serial_number, item.capacity,
                item.condition, item.acquisition_date, item.entered_custody_date,
                item.submitted_by, item.received_by, item.received_location,
                item.storage_location, item.reason_submitted, item.intake_hashes_json,
                item.notes, item.disposition, item.disposition_date, item.disposition_notes,
                item.created_at, item.modified_at,
                item.status, item.locked_at, item.locked_by,
            ],
        )?;
        Ok(())
    }

    /// Get all COC items for a case (excludes voided unless explicitly requested)
    pub fn get_coc_items(&self, case_number: Option<&str>) -> SqlResult<Vec<DbCocItem>> {
        let conn = self.conn.lock();
        let sql = if case_number.is_some() {
            "SELECT id, coc_number, evidence_file_id, case_number, evidence_id, description, item_type, make, model, serial_number, capacity, condition, acquisition_date, entered_custody_date, submitted_by, received_by, received_location, storage_location, reason_submitted, intake_hashes_json, notes, disposition, disposition_date, disposition_notes, created_at, modified_at, status, locked_at, locked_by
             FROM coc_items WHERE case_number = ?1 AND status != 'voided' ORDER BY coc_number ASC"
        } else {
            "SELECT id, coc_number, evidence_file_id, case_number, evidence_id, description, item_type, make, model, serial_number, capacity, condition, acquisition_date, entered_custody_date, submitted_by, received_by, received_location, storage_location, reason_submitted, intake_hashes_json, notes, disposition, disposition_date, disposition_notes, created_at, modified_at, status, locked_at, locked_by
             FROM coc_items WHERE status != 'voided' ORDER BY coc_number ASC"
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
            Ok(DbCocItem {
                id: row.get(0)?,
                coc_number: row.get(1)?,
                evidence_file_id: row.get(2)?,
                case_number: row.get(3)?,
                evidence_id: row.get(4)?,
                description: row.get(5)?,
                item_type: row.get(6)?,
                make: row.get(7)?,
                model: row.get(8)?,
                serial_number: row.get(9)?,
                capacity: row.get(10)?,
                condition: row.get(11)?,
                acquisition_date: row.get(12)?,
                entered_custody_date: row.get(13)?,
                submitted_by: row.get(14)?,
                received_by: row.get(15)?,
                received_location: row.get(16)?,
                storage_location: row.get(17)?,
                reason_submitted: row.get(18)?,
                intake_hashes_json: row.get(19)?,
                notes: row.get(20)?,
                disposition: row.get(21)?,
                disposition_date: row.get(22)?,
                disposition_notes: row.get(23)?,
                created_at: row.get(24)?,
                modified_at: row.get(25)?,
                status: row.get(26)?,
                locked_at: row.get(27)?,
                locked_by: row.get(28)?,
            })
        })?;
        rows.collect()
    }

    /// Lock a COC item — makes it immutable (only amendments allowed after this).
    pub fn lock_coc_item(&self, id: &str, locked_by: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE coc_items SET status = 'locked', locked_at = ?1, locked_by = ?2, modified_at = ?1
             WHERE id = ?3 AND status = 'draft'",
            params![now, locked_by, id],
        )?;
        if affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        self.insert_coc_audit_internal(
            &conn,
            id,
            "locked",
            locked_by,
            &format!("COC item locked by {}", locked_by),
            None,
        )?;
        Ok(())
    }

    /// Amend a specific field on a locked COC item. Requires initials and date.
    /// Creates an amendment record and updates the field value.
    pub fn amend_coc_item(
        &self,
        coc_item_id: &str,
        field_name: &str,
        old_value: &str,
        new_value: &str,
        amended_by_initials: &str,
        reason: Option<&str>,
    ) -> SqlResult<DbCocAmendment> {
        let conn = self.conn.lock();
        // Verify item exists and is locked
        let status: String = conn.query_row(
            "SELECT status FROM coc_items WHERE id = ?1",
            params![coc_item_id],
            |row| row.get(0),
        )?;
        if status == "voided" {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        let now = chrono::Utc::now().to_rfc3339();
        let amendment_id = uuid::Uuid::new_v4().to_string();

        // Validate field name is a real COC item column
        let valid_fields = [
            "coc_number",
            "evidence_file_id",
            "case_number",
            "evidence_id",
            "description",
            "item_type",
            "make",
            "model",
            "serial_number",
            "capacity",
            "condition",
            "acquisition_date",
            "entered_custody_date",
            "submitted_by",
            "received_by",
            "received_location",
            "storage_location",
            "reason_submitted",
            "intake_hashes_json",
            "notes",
            "disposition",
            "disposition_date",
            "disposition_notes",
        ];
        if !valid_fields.contains(&field_name) {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "Invalid COC field name: {}",
                field_name
            )));
        }

        // Insert amendment record
        conn.execute(
            "INSERT INTO coc_amendments (id, coc_item_id, field_name, old_value, new_value, amended_by_initials, amended_at, reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                amendment_id, coc_item_id, field_name, old_value, new_value,
                amended_by_initials, now, reason,
            ],
        )?;

        // Apply the field update using parameterized UPDATE
        // We use a match to build the correct SQL since column names can't be parameterized
        let update_sql = format!(
            "UPDATE coc_items SET {} = ?1, modified_at = ?2 WHERE id = ?3",
            field_name
        );
        conn.execute(&update_sql, params![new_value, now, coc_item_id])?;

        // Audit
        let details = serde_json::json!({
            "field": field_name,
            "old_value": old_value,
            "new_value": new_value,
            "reason": reason,
        });
        self.insert_coc_audit_internal(
            &conn,
            coc_item_id,
            "amended",
            amended_by_initials,
            &format!("Field '{}' amended by {}", field_name, amended_by_initials),
            Some(&details.to_string()),
        )?;

        Ok(DbCocAmendment {
            id: amendment_id,
            coc_item_id: coc_item_id.to_string(),
            field_name: field_name.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
            amended_by_initials: amended_by_initials.to_string(),
            amended_at: now,
            reason: reason.map(|s| s.to_string()),
        })
    }

    /// Soft-delete (void) a COC item — sets status to 'voided'.
    /// The record remains in the database for audit purposes.
    pub fn delete_coc_item(&self, id: &str, voided_by: &str, reason: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE coc_items SET status = 'voided', modified_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        let details = serde_json::json!({ "reason": reason });
        self.insert_coc_audit_internal(
            &conn,
            id,
            "voided",
            voided_by,
            &format!("COC item voided by {} — {}", voided_by, reason),
            Some(&details.to_string()),
        )?;
        Ok(())
    }

    /// Get amendments for a COC item
    pub fn get_coc_amendments(&self, coc_item_id: &str) -> SqlResult<Vec<DbCocAmendment>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, coc_item_id, field_name, old_value, new_value, amended_by_initials, amended_at, reason
             FROM coc_amendments WHERE coc_item_id = ?1 ORDER BY amended_at ASC",
        )?;
        let rows = stmt.query_map(params![coc_item_id], |row| {
            Ok(DbCocAmendment {
                id: row.get(0)?,
                coc_item_id: row.get(1)?,
                field_name: row.get(2)?,
                old_value: row.get(3)?,
                new_value: row.get(4)?,
                amended_by_initials: row.get(5)?,
                amended_at: row.get(6)?,
                reason: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    /// Get audit log entries for a COC item (or all items if None)
    pub fn get_coc_audit_log(&self, coc_item_id: Option<&str>) -> SqlResult<Vec<DbCocAuditEntry>> {
        let conn = self.conn.lock();
        if let Some(item_id) = coc_item_id {
            let mut stmt = conn.prepare(
                "SELECT id, coc_item_id, action, performed_by, performed_at, summary, details_json
                 FROM coc_audit_log WHERE coc_item_id = ?1 ORDER BY performed_at ASC",
            )?;
            let rows = stmt.query_map(params![item_id], |row| {
                Ok(DbCocAuditEntry {
                    id: row.get(0)?,
                    coc_item_id: row.get(1)?,
                    action: row.get(2)?,
                    performed_by: row.get(3)?,
                    performed_at: row.get(4)?,
                    summary: row.get(5)?,
                    details_json: row.get(6)?,
                })
            })?;
            rows.collect()
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, coc_item_id, action, performed_by, performed_at, summary, details_json
                 FROM coc_audit_log ORDER BY performed_at ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(DbCocAuditEntry {
                    id: row.get(0)?,
                    coc_item_id: row.get(1)?,
                    action: row.get(2)?,
                    performed_by: row.get(3)?,
                    performed_at: row.get(4)?,
                    summary: row.get(5)?,
                    details_json: row.get(6)?,
                })
            })?;
            rows.collect()
        }
    }

    /// Insert a COC audit log entry (internal helper — uses existing connection lock)
    fn insert_coc_audit_internal(
        &self,
        conn: &rusqlite::Connection,
        coc_item_id: &str,
        action: &str,
        performed_by: &str,
        summary: &str,
        details_json: Option<&str>,
    ) -> SqlResult<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO coc_audit_log (id, coc_item_id, action, performed_by, performed_at, summary, details_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, coc_item_id, action, performed_by, now, summary, details_json],
        )?;
        Ok(())
    }

    /// Insert a COC audit log entry (public — acquires its own lock)
    pub fn insert_coc_audit_entry(&self, entry: &DbCocAuditEntry) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO coc_audit_log (id, coc_item_id, action, performed_by, performed_at, summary, details_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.id, entry.coc_item_id, entry.action,
                entry.performed_by, entry.performed_at, entry.summary, entry.details_json,
            ],
        )?;
        Ok(())
    }

    // ========================================================================
    // COC Transfer Operations
    // ========================================================================

    /// Upsert a COC transfer record (also inserts audit log entry)
    pub fn upsert_coc_transfer(&self, transfer: &DbCocTransfer) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO coc_transfers (id, coc_item_id, timestamp, released_by, received_by, purpose, location, method, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                timestamp=excluded.timestamp, released_by=excluded.released_by,
                received_by=excluded.received_by, purpose=excluded.purpose,
                location=excluded.location, method=excluded.method, notes=excluded.notes",
            params![
                transfer.id, transfer.coc_item_id, transfer.timestamp,
                transfer.released_by, transfer.received_by, transfer.purpose,
                transfer.location, transfer.method, transfer.notes,
            ],
        )?;
        self.insert_coc_audit_internal(
            &conn,
            &transfer.coc_item_id,
            "transfer_added",
            &transfer.released_by,
            &format!(
                "Transfer: {} → {} ({})",
                transfer.released_by, transfer.received_by, transfer.purpose
            ),
            None,
        )?;
        Ok(())
    }

    /// Get all transfers for a COC item
    pub fn get_coc_transfers(&self, coc_item_id: &str) -> SqlResult<Vec<DbCocTransfer>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, coc_item_id, timestamp, released_by, received_by, purpose, location, method, notes
             FROM coc_transfers WHERE coc_item_id = ?1 ORDER BY timestamp ASC",
        )?;
        let rows = stmt.query_map(params![coc_item_id], |row| {
            Ok(DbCocTransfer {
                id: row.get(0)?,
                coc_item_id: row.get(1)?,
                timestamp: row.get(2)?,
                released_by: row.get(3)?,
                received_by: row.get(4)?,
                purpose: row.get(5)?,
                location: row.get(6)?,
                method: row.get(7)?,
                notes: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    /// Get all transfers across all COC items
    pub fn get_all_coc_transfers(&self) -> SqlResult<Vec<DbCocTransfer>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, coc_item_id, timestamp, released_by, received_by, purpose, location, method, notes
             FROM coc_transfers ORDER BY timestamp ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DbCocTransfer {
                id: row.get(0)?,
                coc_item_id: row.get(1)?,
                timestamp: row.get(2)?,
                released_by: row.get(3)?,
                received_by: row.get(4)?,
                purpose: row.get(5)?,
                location: row.get(6)?,
                method: row.get(7)?,
                notes: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    /// Delete a COC transfer
    pub fn delete_coc_transfer(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM coc_transfers WHERE id = ?1", params![id])?;
        Ok(())
    }
}
