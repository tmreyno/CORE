// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for export history, COC items
//! (immutability model), amendments, audit log, and COC transfers.

use super::with_project_db;
use crate::project_db::{
    DbCocAmendment, DbCocAuditEntry, DbCocItem, DbCocTransfer, DbExportRecord,
};

const MAX_FORENSIC_RESPONSE_ROWS: usize = 10_000;
const MAX_FORENSIC_FIELD_CHARS: usize = 4096;
const MAX_FORENSIC_TEXT_CHARS: usize = 65_536;
const MAX_FORENSIC_JSON_CHARS: usize = 65_536;
const MAX_FORENSIC_JSON_DEPTH: usize = 4;
const MAX_FORENSIC_JSON_ITEMS: usize = 256;
const FORENSIC_TRUNCATED_SUFFIX: &str = "... [truncated]";

// =============================================================================
// Export History Commands
// =============================================================================

/// Insert an export record.
#[tauri::command]
pub fn project_db_insert_export(
    window: tauri::Window,
    record: DbExportRecord,
) -> Result<(), String> {
    let record = bounded_export_record(record);
    with_project_db(window.label(), |db| db.insert_export(&record))
}

/// Update an export record (status, completed_at, error, etc.).
#[tauri::command]
pub fn project_db_update_export(
    window: tauri::Window,
    record: DbExportRecord,
) -> Result<(), String> {
    let record = bounded_export_record(record);
    with_project_db(window.label(), |db| db.update_export(&record))
}

/// Get export records, most recent first.
#[tauri::command]
pub fn project_db_get_exports(
    window: tauri::Window,
    limit: Option<i64>,
) -> Result<Vec<DbExportRecord>, String> {
    let limit = limit
        .unwrap_or(MAX_FORENSIC_RESPONSE_ROWS as i64)
        .clamp(0, MAX_FORENSIC_RESPONSE_ROWS as i64);
    with_project_db(window.label(), |db| {
        db.get_exports(Some(limit)).map(|records| {
            records
                .into_iter()
                .take(MAX_FORENSIC_RESPONSE_ROWS)
                .map(bounded_export_record)
                .collect()
        })
    })
}

/// Delete an export record.
#[tauri::command]
pub fn project_db_delete_export(window: tauri::Window, id: String) -> Result<(), String> {
    let id = truncate_forensic_text(&id, MAX_FORENSIC_FIELD_CHARS);
    with_project_db(window.label(), |db| db.delete_export(&id))
}

// =============================================================================
// COC Item Commands (v5 — immutability model)
// =============================================================================

/// Insert a new COC item (draft status). Fails if ID already exists.
#[tauri::command]
pub fn project_db_insert_coc_item(window: tauri::Window, record: DbCocItem) -> Result<(), String> {
    let record = bounded_coc_item(record);
    with_project_db(window.label(), |db| db.insert_coc_item(&record))
}

/// Insert or update a COC item (allowed ONLY for draft items).
#[tauri::command]
pub fn project_db_upsert_coc_item(window: tauri::Window, record: DbCocItem) -> Result<(), String> {
    let record = bounded_coc_item(record);
    with_project_db(window.label(), |db| db.upsert_coc_item(&record))
}

/// Get COC items, optionally filtered by case number.
#[tauri::command]
pub fn project_db_get_coc_items(
    window: tauri::Window,
    case_number: Option<String>,
) -> Result<Vec<DbCocItem>, String> {
    let case_number =
        case_number.map(|value| truncate_forensic_text(&value, MAX_FORENSIC_FIELD_CHARS));
    with_project_db(window.label(), |db| {
        db.get_coc_items(case_number.as_deref()).map(|items| {
            items
                .into_iter()
                .take(MAX_FORENSIC_RESPONSE_ROWS)
                .map(bounded_coc_item)
                .collect()
        })
    })
}

/// Lock a COC item — makes it immutable (only amendments allowed after this).
#[tauri::command]
pub fn project_db_lock_coc_item(
    window: tauri::Window,
    id: String,
    locked_by: String,
) -> Result<(), String> {
    let id = truncate_forensic_text(&id, MAX_FORENSIC_FIELD_CHARS);
    let locked_by = truncate_forensic_text(&locked_by, MAX_FORENSIC_FIELD_CHARS);
    with_project_db(window.label(), |db| db.lock_coc_item(&id, &locked_by))
}

/// Amend a field on a COC item (requires initials + date). Creates amendment record.
#[tauri::command]
pub fn project_db_amend_coc_item(
    window: tauri::Window,
    coc_item_id: String,
    field_name: String,
    old_value: String,
    new_value: String,
    amended_by_initials: String,
    reason: Option<String>,
) -> Result<DbCocAmendment, String> {
    let coc_item_id = truncate_forensic_text(&coc_item_id, MAX_FORENSIC_FIELD_CHARS);
    let field_name = truncate_forensic_text(&field_name, MAX_FORENSIC_FIELD_CHARS);
    let old_value = truncate_forensic_text(&old_value, MAX_FORENSIC_TEXT_CHARS);
    let new_value = truncate_forensic_text(&new_value, MAX_FORENSIC_TEXT_CHARS);
    let amended_by_initials =
        truncate_forensic_text(&amended_by_initials, MAX_FORENSIC_FIELD_CHARS);
    let reason = reason.map(|value| truncate_forensic_text(&value, MAX_FORENSIC_TEXT_CHARS));
    with_project_db(window.label(), |db| {
        db.amend_coc_item(
            &coc_item_id,
            &field_name,
            &old_value,
            &new_value,
            &amended_by_initials,
            reason.as_deref(),
        )
        .map(bounded_coc_amendment)
    })
}

/// Soft-delete (void) a COC item. Record remains for audit trail.
#[tauri::command]
pub fn project_db_delete_coc_item(
    window: tauri::Window,
    id: String,
    voided_by: String,
    reason: String,
) -> Result<(), String> {
    let id = truncate_forensic_text(&id, MAX_FORENSIC_FIELD_CHARS);
    let voided_by = truncate_forensic_text(&voided_by, MAX_FORENSIC_FIELD_CHARS);
    let reason = truncate_forensic_text(&reason, MAX_FORENSIC_TEXT_CHARS);
    with_project_db(window.label(), |db| {
        db.delete_coc_item(&id, &voided_by, &reason)
    })
}

/// Get amendments for a COC item.
#[tauri::command]
pub fn project_db_get_coc_amendments(
    window: tauri::Window,
    coc_item_id: String,
) -> Result<Vec<DbCocAmendment>, String> {
    let coc_item_id = truncate_forensic_text(&coc_item_id, MAX_FORENSIC_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_coc_amendments(&coc_item_id).map(|amendments| {
            amendments
                .into_iter()
                .take(MAX_FORENSIC_RESPONSE_ROWS)
                .map(bounded_coc_amendment)
                .collect()
        })
    })
}

/// Get audit log entries for a COC item (or all if coc_item_id is None).
#[tauri::command]
pub fn project_db_get_coc_audit_log(
    window: tauri::Window,
    coc_item_id: Option<String>,
) -> Result<Vec<DbCocAuditEntry>, String> {
    let coc_item_id =
        coc_item_id.map(|value| truncate_forensic_text(&value, MAX_FORENSIC_FIELD_CHARS));
    with_project_db(window.label(), |db| {
        db.get_coc_audit_log(coc_item_id.as_deref()).map(|entries| {
            entries
                .into_iter()
                .take(MAX_FORENSIC_RESPONSE_ROWS)
                .map(bounded_coc_audit_entry)
                .collect()
        })
    })
}

/// Insert a COC audit log entry.
#[tauri::command]
pub fn project_db_insert_coc_audit_entry(
    window: tauri::Window,
    entry: DbCocAuditEntry,
) -> Result<(), String> {
    let entry = bounded_coc_audit_entry(entry);
    with_project_db(window.label(), |db| db.insert_coc_audit_entry(&entry))
}

// =============================================================================
// COC Transfer Commands
// =============================================================================

/// Insert or update a COC transfer record.
#[tauri::command]
pub fn project_db_upsert_coc_transfer(
    window: tauri::Window,
    record: DbCocTransfer,
) -> Result<(), String> {
    let record = bounded_coc_transfer(record);
    with_project_db(window.label(), |db| db.upsert_coc_transfer(&record))
}

/// Get transfers for a specific COC item.
#[tauri::command]
pub fn project_db_get_coc_transfers(
    window: tauri::Window,
    coc_item_id: String,
) -> Result<Vec<DbCocTransfer>, String> {
    let coc_item_id = truncate_forensic_text(&coc_item_id, MAX_FORENSIC_FIELD_CHARS);
    with_project_db(window.label(), |db| {
        db.get_coc_transfers(&coc_item_id).map(|transfers| {
            transfers
                .into_iter()
                .take(MAX_FORENSIC_RESPONSE_ROWS)
                .map(bounded_coc_transfer)
                .collect()
        })
    })
}

/// Get all COC transfers.
#[tauri::command]
pub fn project_db_get_all_coc_transfers(
    window: tauri::Window,
) -> Result<Vec<DbCocTransfer>, String> {
    with_project_db(window.label(), |db| {
        db.get_all_coc_transfers().map(|transfers| {
            transfers
                .into_iter()
                .take(MAX_FORENSIC_RESPONSE_ROWS)
                .map(bounded_coc_transfer)
                .collect()
        })
    })
}

/// Delete a COC transfer.
#[tauri::command]
pub fn project_db_delete_coc_transfer(window: tauri::Window, id: String) -> Result<(), String> {
    let id = truncate_forensic_text(&id, MAX_FORENSIC_FIELD_CHARS);
    with_project_db(window.label(), |db| db.delete_coc_transfer(&id))
}

fn bounded_export_record(mut record: DbExportRecord) -> DbExportRecord {
    record.id = truncate_forensic_text(&record.id, MAX_FORENSIC_FIELD_CHARS);
    record.export_type = truncate_forensic_text(&record.export_type, MAX_FORENSIC_FIELD_CHARS);
    record.source_paths_json =
        bounded_forensic_json_or_text(&record.source_paths_json, MAX_FORENSIC_JSON_CHARS);
    record.destination = truncate_forensic_text(&record.destination, MAX_FORENSIC_TEXT_CHARS);
    record.started_at = truncate_forensic_text(&record.started_at, MAX_FORENSIC_FIELD_CHARS);
    record.completed_at = opt_forensic_text(record.completed_at, MAX_FORENSIC_FIELD_CHARS);
    record.initiated_by = truncate_forensic_text(&record.initiated_by, MAX_FORENSIC_FIELD_CHARS);
    record.status = truncate_forensic_text(&record.status, MAX_FORENSIC_FIELD_CHARS);
    record.archive_name = opt_forensic_text(record.archive_name, MAX_FORENSIC_FIELD_CHARS);
    record.archive_format = opt_forensic_text(record.archive_format, MAX_FORENSIC_FIELD_CHARS);
    record.compression_level =
        opt_forensic_text(record.compression_level, MAX_FORENSIC_FIELD_CHARS);
    record.manifest_hash = opt_forensic_text(record.manifest_hash, MAX_FORENSIC_FIELD_CHARS);
    record.error = opt_forensic_text(record.error, MAX_FORENSIC_TEXT_CHARS);
    record.options_json = record
        .options_json
        .map(|value| bounded_forensic_json_or_text(&value, MAX_FORENSIC_JSON_CHARS));
    record
}

fn bounded_coc_item(mut item: DbCocItem) -> DbCocItem {
    item.id = truncate_forensic_text(&item.id, MAX_FORENSIC_FIELD_CHARS);
    item.coc_number = truncate_forensic_text(&item.coc_number, MAX_FORENSIC_FIELD_CHARS);
    item.evidence_file_id = opt_forensic_text(item.evidence_file_id, MAX_FORENSIC_FIELD_CHARS);
    item.case_number = truncate_forensic_text(&item.case_number, MAX_FORENSIC_FIELD_CHARS);
    item.evidence_id = truncate_forensic_text(&item.evidence_id, MAX_FORENSIC_FIELD_CHARS);
    item.description = truncate_forensic_text(&item.description, MAX_FORENSIC_TEXT_CHARS);
    item.item_type = truncate_forensic_text(&item.item_type, MAX_FORENSIC_FIELD_CHARS);
    item.case_title = opt_forensic_text(item.case_title, MAX_FORENSIC_FIELD_CHARS);
    item.office = opt_forensic_text(item.office, MAX_FORENSIC_FIELD_CHARS);
    item.owner_name = opt_forensic_text(item.owner_name, MAX_FORENSIC_FIELD_CHARS);
    item.owner_address = opt_forensic_text(item.owner_address, MAX_FORENSIC_TEXT_CHARS);
    item.owner_phone = opt_forensic_text(item.owner_phone, MAX_FORENSIC_FIELD_CHARS);
    item.source = opt_forensic_text(item.source, MAX_FORENSIC_FIELD_CHARS);
    item.other_contact_name = opt_forensic_text(item.other_contact_name, MAX_FORENSIC_FIELD_CHARS);
    item.other_contact_relation =
        opt_forensic_text(item.other_contact_relation, MAX_FORENSIC_FIELD_CHARS);
    item.other_contact_phone =
        opt_forensic_text(item.other_contact_phone, MAX_FORENSIC_FIELD_CHARS);
    item.collection_method = opt_forensic_text(item.collection_method, MAX_FORENSIC_FIELD_CHARS);
    item.collection_method_other =
        opt_forensic_text(item.collection_method_other, MAX_FORENSIC_TEXT_CHARS);
    item.make = opt_forensic_text(item.make, MAX_FORENSIC_FIELD_CHARS);
    item.model = opt_forensic_text(item.model, MAX_FORENSIC_FIELD_CHARS);
    item.serial_number = opt_forensic_text(item.serial_number, MAX_FORENSIC_FIELD_CHARS);
    item.capacity = opt_forensic_text(item.capacity, MAX_FORENSIC_FIELD_CHARS);
    item.condition = truncate_forensic_text(&item.condition, MAX_FORENSIC_FIELD_CHARS);
    item.acquisition_date =
        truncate_forensic_text(&item.acquisition_date, MAX_FORENSIC_FIELD_CHARS);
    item.entered_custody_date =
        truncate_forensic_text(&item.entered_custody_date, MAX_FORENSIC_FIELD_CHARS);
    item.submitted_by = truncate_forensic_text(&item.submitted_by, MAX_FORENSIC_FIELD_CHARS);
    item.collected_date = opt_forensic_text(item.collected_date, MAX_FORENSIC_FIELD_CHARS);
    item.received_by = truncate_forensic_text(&item.received_by, MAX_FORENSIC_FIELD_CHARS);
    item.received_location = opt_forensic_text(item.received_location, MAX_FORENSIC_FIELD_CHARS);
    item.storage_location = opt_forensic_text(item.storage_location, MAX_FORENSIC_FIELD_CHARS);
    item.storage_class = opt_forensic_text(item.storage_class, MAX_FORENSIC_FIELD_CHARS);
    item.storage_location_detail =
        opt_forensic_text(item.storage_location_detail, MAX_FORENSIC_TEXT_CHARS);
    item.reason_submitted = opt_forensic_text(item.reason_submitted, MAX_FORENSIC_TEXT_CHARS);
    item.intake_hashes_json = item
        .intake_hashes_json
        .map(|value| bounded_forensic_json_or_text(&value, MAX_FORENSIC_JSON_CHARS));
    item.notes = opt_forensic_text(item.notes, MAX_FORENSIC_TEXT_CHARS);
    item.disposition = opt_forensic_text(item.disposition, MAX_FORENSIC_FIELD_CHARS);
    item.disposition_by = opt_forensic_text(item.disposition_by, MAX_FORENSIC_FIELD_CHARS);
    item.returned_to = opt_forensic_text(item.returned_to, MAX_FORENSIC_FIELD_CHARS);
    item.destruction_date = opt_forensic_text(item.destruction_date, MAX_FORENSIC_FIELD_CHARS);
    item.disposition_date = opt_forensic_text(item.disposition_date, MAX_FORENSIC_FIELD_CHARS);
    item.disposition_notes = opt_forensic_text(item.disposition_notes, MAX_FORENSIC_TEXT_CHARS);
    item.created_at = truncate_forensic_text(&item.created_at, MAX_FORENSIC_FIELD_CHARS);
    item.modified_at = truncate_forensic_text(&item.modified_at, MAX_FORENSIC_FIELD_CHARS);
    item.status = truncate_forensic_text(&item.status, MAX_FORENSIC_FIELD_CHARS);
    item.locked_at = opt_forensic_text(item.locked_at, MAX_FORENSIC_FIELD_CHARS);
    item.locked_by = opt_forensic_text(item.locked_by, MAX_FORENSIC_FIELD_CHARS);
    item
}

fn bounded_coc_amendment(mut amendment: DbCocAmendment) -> DbCocAmendment {
    amendment.id = truncate_forensic_text(&amendment.id, MAX_FORENSIC_FIELD_CHARS);
    amendment.coc_item_id =
        truncate_forensic_text(&amendment.coc_item_id, MAX_FORENSIC_FIELD_CHARS);
    amendment.field_name = truncate_forensic_text(&amendment.field_name, MAX_FORENSIC_FIELD_CHARS);
    amendment.old_value = truncate_forensic_text(&amendment.old_value, MAX_FORENSIC_TEXT_CHARS);
    amendment.new_value = truncate_forensic_text(&amendment.new_value, MAX_FORENSIC_TEXT_CHARS);
    amendment.amended_by_initials =
        truncate_forensic_text(&amendment.amended_by_initials, MAX_FORENSIC_FIELD_CHARS);
    amendment.amended_at = truncate_forensic_text(&amendment.amended_at, MAX_FORENSIC_FIELD_CHARS);
    amendment.reason = opt_forensic_text(amendment.reason, MAX_FORENSIC_TEXT_CHARS);
    amendment
}

fn bounded_coc_audit_entry(mut entry: DbCocAuditEntry) -> DbCocAuditEntry {
    entry.id = truncate_forensic_text(&entry.id, MAX_FORENSIC_FIELD_CHARS);
    entry.coc_item_id = opt_forensic_text(entry.coc_item_id, MAX_FORENSIC_FIELD_CHARS);
    entry.action = truncate_forensic_text(&entry.action, MAX_FORENSIC_FIELD_CHARS);
    entry.performed_by = truncate_forensic_text(&entry.performed_by, MAX_FORENSIC_FIELD_CHARS);
    entry.performed_at = truncate_forensic_text(&entry.performed_at, MAX_FORENSIC_FIELD_CHARS);
    entry.summary = truncate_forensic_text(&entry.summary, MAX_FORENSIC_TEXT_CHARS);
    entry.details_json = entry
        .details_json
        .map(|value| bounded_forensic_json_or_text(&value, MAX_FORENSIC_JSON_CHARS));
    entry
}

fn bounded_coc_transfer(mut transfer: DbCocTransfer) -> DbCocTransfer {
    transfer.id = truncate_forensic_text(&transfer.id, MAX_FORENSIC_FIELD_CHARS);
    transfer.coc_item_id = truncate_forensic_text(&transfer.coc_item_id, MAX_FORENSIC_FIELD_CHARS);
    transfer.timestamp = truncate_forensic_text(&transfer.timestamp, MAX_FORENSIC_FIELD_CHARS);
    transfer.released_by = truncate_forensic_text(&transfer.released_by, MAX_FORENSIC_FIELD_CHARS);
    transfer.received_by = truncate_forensic_text(&transfer.received_by, MAX_FORENSIC_FIELD_CHARS);
    transfer.purpose = truncate_forensic_text(&transfer.purpose, MAX_FORENSIC_TEXT_CHARS);
    transfer.location = opt_forensic_text(transfer.location, MAX_FORENSIC_FIELD_CHARS);
    transfer.storage_location =
        opt_forensic_text(transfer.storage_location, MAX_FORENSIC_FIELD_CHARS);
    transfer.storage_class = opt_forensic_text(transfer.storage_class, MAX_FORENSIC_FIELD_CHARS);
    transfer.storage_location_detail =
        opt_forensic_text(transfer.storage_location_detail, MAX_FORENSIC_TEXT_CHARS);
    transfer.storage_date = opt_forensic_text(transfer.storage_date, MAX_FORENSIC_FIELD_CHARS);
    transfer.method = opt_forensic_text(transfer.method, MAX_FORENSIC_FIELD_CHARS);
    transfer.notes = opt_forensic_text(transfer.notes, MAX_FORENSIC_TEXT_CHARS);
    transfer
}

fn opt_forensic_text(value: Option<String>, max_chars: usize) -> Option<String> {
    value.map(|value| truncate_forensic_text(&value, max_chars))
}

fn truncate_forensic_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let keep_chars = max_chars.saturating_sub(FORENSIC_TRUNCATED_SUFFIX.chars().count());
    let mut truncated: String = value.chars().take(keep_chars).collect();
    truncated.push_str(FORENSIC_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_forensic_json_or_text(value: &str, max_chars: usize) -> String {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(value) else {
        return truncate_forensic_text(value, max_chars);
    };
    let bounded = bounded_forensic_json_value(parsed, 0);
    match serde_json::to_string(&bounded) {
        Ok(serialized) => truncate_forensic_text(&serialized, max_chars),
        Err(_) => truncate_forensic_text(value, max_chars),
    }
}

fn bounded_forensic_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_FORENSIC_JSON_DEPTH {
        return serde_json::Value::String(FORENSIC_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(text) => {
            serde_json::Value::String(truncate_forensic_text(&text, MAX_FORENSIC_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_FORENSIC_JSON_ITEMS)
                .map(|value| bounded_forensic_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(entries) => serde_json::Value::Object(
            entries
                .into_iter()
                .take(MAX_FORENSIC_JSON_ITEMS)
                .map(|(key, value)| {
                    (
                        truncate_forensic_text(&key, MAX_FORENSIC_FIELD_CHARS),
                        bounded_forensic_json_value(value, depth + 1),
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
    fn bounded_export_record_caps_json_and_text_fields() {
        let record = bounded_export_record(DbExportRecord {
            id: repeated(MAX_FORENSIC_FIELD_CHARS + 8),
            export_type: "portable".to_string(),
            source_paths_json: serde_json::to_string(&vec![repeated(MAX_FORENSIC_FIELD_CHARS + 8)])
                .unwrap(),
            destination: repeated(MAX_FORENSIC_TEXT_CHARS + 8),
            started_at: "2026-02-16T10:00:00Z".to_string(),
            completed_at: None,
            initiated_by: "examiner".to_string(),
            status: "complete".to_string(),
            total_files: 1,
            total_bytes: 10,
            archive_name: Some(repeated(MAX_FORENSIC_FIELD_CHARS + 8)),
            archive_format: Some("zip".to_string()),
            compression_level: None,
            encrypted: Some(false),
            manifest_hash: None,
            error: Some(repeated(MAX_FORENSIC_TEXT_CHARS + 8)),
            options_json: Some(
                "{\"nested\":{\"inner\":{\"deeper\":{\"too\":\"deep\"}}}}".to_string(),
            ),
        });

        assert_eq!(record.id.chars().count(), MAX_FORENSIC_FIELD_CHARS);
        assert_eq!(record.destination.chars().count(), MAX_FORENSIC_TEXT_CHARS);
        assert_eq!(
            record.error.unwrap().chars().count(),
            MAX_FORENSIC_TEXT_CHARS
        );
        assert!(record
            .archive_name
            .unwrap()
            .ends_with(FORENSIC_TRUNCATED_SUFFIX));
        assert!(record
            .options_json
            .unwrap()
            .contains(FORENSIC_TRUNCATED_SUFFIX));
        let paths: serde_json::Value = serde_json::from_str(&record.source_paths_json).unwrap();
        assert_eq!(
            paths[0].as_str().unwrap().chars().count(),
            MAX_FORENSIC_FIELD_CHARS
        );
    }

    #[test]
    fn bounded_coc_item_caps_notes_and_hash_json() {
        let item = bounded_coc_item(DbCocItem {
            id: "item-1".to_string(),
            coc_number: "COC-1".to_string(),
            evidence_file_id: Some(repeated(MAX_FORENSIC_FIELD_CHARS + 8)),
            case_number: "case".to_string(),
            evidence_id: "evidence".to_string(),
            description: repeated(MAX_FORENSIC_TEXT_CHARS + 8),
            item_type: "drive".to_string(),
            case_title: None,
            office: None,
            owner_name: None,
            owner_address: Some(repeated(MAX_FORENSIC_TEXT_CHARS + 8)),
            owner_phone: None,
            source: None,
            other_contact_name: None,
            other_contact_relation: None,
            other_contact_phone: None,
            collection_method: None,
            collection_method_other: None,
            make: None,
            model: None,
            serial_number: None,
            capacity: None,
            condition: "sealed".to_string(),
            acquisition_date: "2026-02-16".to_string(),
            entered_custody_date: "2026-02-16".to_string(),
            submitted_by: "examiner".to_string(),
            collected_date: None,
            received_by: "custodian".to_string(),
            received_location: None,
            storage_location: None,
            storage_class: None,
            storage_location_detail: None,
            reason_submitted: None,
            intake_hashes_json: Some(
                serde_json::to_string(&vec![repeated(MAX_FORENSIC_FIELD_CHARS + 8)]).unwrap(),
            ),
            notes: Some(repeated(MAX_FORENSIC_TEXT_CHARS + 8)),
            disposition: None,
            disposition_by: None,
            returned_to: None,
            destruction_date: None,
            disposition_date: None,
            disposition_notes: None,
            created_at: "2026-02-16T10:00:00Z".to_string(),
            modified_at: "2026-02-16T10:00:00Z".to_string(),
            status: "draft".to_string(),
            locked_at: None,
            locked_by: None,
        });

        assert_eq!(item.description.chars().count(), MAX_FORENSIC_TEXT_CHARS);
        assert_eq!(
            item.owner_address.unwrap().chars().count(),
            MAX_FORENSIC_TEXT_CHARS
        );
        assert_eq!(item.notes.unwrap().chars().count(), MAX_FORENSIC_TEXT_CHARS);
        assert_eq!(
            item.evidence_file_id.unwrap().chars().count(),
            MAX_FORENSIC_FIELD_CHARS
        );
        let hashes: serde_json::Value =
            serde_json::from_str(&item.intake_hashes_json.unwrap()).unwrap();
        assert_eq!(
            hashes[0].as_str().unwrap().chars().count(),
            MAX_FORENSIC_FIELD_CHARS
        );
    }

    #[test]
    fn bounded_amendment_audit_and_transfer_cap_payloads() {
        let amendment = bounded_coc_amendment(DbCocAmendment {
            id: "amend-1".to_string(),
            coc_item_id: "item-1".to_string(),
            field_name: "notes".to_string(),
            old_value: repeated(MAX_FORENSIC_TEXT_CHARS + 8),
            new_value: repeated(MAX_FORENSIC_TEXT_CHARS + 8),
            amended_by_initials: "TR".to_string(),
            amended_at: "2026-02-16T10:00:00Z".to_string(),
            reason: Some(repeated(MAX_FORENSIC_TEXT_CHARS + 8)),
        });
        let audit = bounded_coc_audit_entry(DbCocAuditEntry {
            id: "audit-1".to_string(),
            coc_item_id: Some("item-1".to_string()),
            action: "amend".to_string(),
            performed_by: "examiner".to_string(),
            performed_at: "2026-02-16T10:00:00Z".to_string(),
            summary: repeated(MAX_FORENSIC_TEXT_CHARS + 8),
            details_json: Some(
                serde_json::to_string(&vec![repeated(MAX_FORENSIC_FIELD_CHARS + 8)]).unwrap(),
            ),
        });
        let transfer = bounded_coc_transfer(DbCocTransfer {
            id: "transfer-1".to_string(),
            coc_item_id: "item-1".to_string(),
            timestamp: "2026-02-16T10:00:00Z".to_string(),
            released_by: "examiner".to_string(),
            received_by: "custodian".to_string(),
            purpose: repeated(MAX_FORENSIC_TEXT_CHARS + 8),
            location: None,
            storage_location: None,
            storage_class: None,
            storage_location_detail: Some(repeated(MAX_FORENSIC_TEXT_CHARS + 8)),
            storage_date: None,
            method: None,
            notes: Some(repeated(MAX_FORENSIC_TEXT_CHARS + 8)),
        });

        assert_eq!(amendment.old_value.chars().count(), MAX_FORENSIC_TEXT_CHARS);
        assert_eq!(
            amendment.reason.unwrap().chars().count(),
            MAX_FORENSIC_TEXT_CHARS
        );
        assert_eq!(audit.summary.chars().count(), MAX_FORENSIC_TEXT_CHARS);
        assert!(audit
            .details_json
            .unwrap()
            .contains(FORENSIC_TRUNCATED_SUFFIX));
        assert_eq!(transfer.purpose.chars().count(), MAX_FORENSIC_TEXT_CHARS);
        assert_eq!(
            transfer.notes.unwrap().chars().count(),
            MAX_FORENSIC_TEXT_CHARS
        );
    }
}
