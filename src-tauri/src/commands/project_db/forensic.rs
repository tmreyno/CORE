// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for export history, chain of custody, COC items
//! (immutability model), amendments, audit log, and COC transfers.

use super::with_project_db;
use crate::project_db::{
    DbCocAmendment, DbCocAuditEntry, DbCocItem, DbCocTransfer,
    DbCustodyRecord, DbExportRecord,
};

// =============================================================================
// Export History Commands
// =============================================================================

/// Insert an export record.
#[tauri::command]
pub fn project_db_insert_export(record: DbExportRecord) -> Result<(), String> {
    with_project_db(|db| db.insert_export(&record))
}

/// Update an export record (status, completed_at, error, etc.).
#[tauri::command]
pub fn project_db_update_export(record: DbExportRecord) -> Result<(), String> {
    with_project_db(|db| db.update_export(&record))
}

/// Get export records, most recent first.
#[tauri::command]
pub fn project_db_get_exports(limit: Option<i64>) -> Result<Vec<DbExportRecord>, String> {
    with_project_db(|db| db.get_exports(limit))
}

/// Delete an export record.
#[tauri::command]
pub fn project_db_delete_export(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_export(&id))
}

// =============================================================================
// Chain of Custody Commands
// =============================================================================

/// Insert a chain-of-custody record.
#[tauri::command]
pub fn project_db_insert_custody_record(record: DbCustodyRecord) -> Result<(), String> {
    with_project_db(|db| db.insert_custody_record(&record))
}

/// Get all custody records in chronological order.
#[tauri::command]
pub fn project_db_get_custody_records() -> Result<Vec<DbCustodyRecord>, String> {
    with_project_db(|db| db.get_custody_records())
}

/// Delete a custody record.
#[tauri::command]
pub fn project_db_delete_custody_record(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_custody_record(&id))
}

// =============================================================================
// COC Item Commands (v5 — immutability model)
// =============================================================================

/// Insert a new COC item (draft status). Fails if ID already exists.
#[tauri::command]
pub fn project_db_insert_coc_item(record: DbCocItem) -> Result<(), String> {
    with_project_db(|db| db.insert_coc_item(&record))
}

/// Insert or update a COC item (allowed ONLY for draft items).
#[tauri::command]
pub fn project_db_upsert_coc_item(record: DbCocItem) -> Result<(), String> {
    with_project_db(|db| db.upsert_coc_item(&record))
}

/// Get COC items, optionally filtered by case number.
#[tauri::command]
pub fn project_db_get_coc_items(case_number: Option<String>) -> Result<Vec<DbCocItem>, String> {
    with_project_db(|db| db.get_coc_items(case_number.as_deref()))
}

/// Lock a COC item — makes it immutable (only amendments allowed after this).
#[tauri::command]
pub fn project_db_lock_coc_item(id: String, locked_by: String) -> Result<(), String> {
    with_project_db(|db| db.lock_coc_item(&id, &locked_by))
}

/// Amend a field on a COC item (requires initials + date). Creates amendment record.
#[tauri::command]
pub fn project_db_amend_coc_item(
    coc_item_id: String,
    field_name: String,
    old_value: String,
    new_value: String,
    amended_by_initials: String,
    reason: Option<String>,
) -> Result<DbCocAmendment, String> {
    with_project_db(|db| {
        db.amend_coc_item(
            &coc_item_id,
            &field_name,
            &old_value,
            &new_value,
            &amended_by_initials,
            reason.as_deref(),
        )
    })
}

/// Soft-delete (void) a COC item. Record remains for audit trail.
#[tauri::command]
pub fn project_db_delete_coc_item(id: String, voided_by: String, reason: String) -> Result<(), String> {
    with_project_db(|db| db.delete_coc_item(&id, &voided_by, &reason))
}

/// Get amendments for a COC item.
#[tauri::command]
pub fn project_db_get_coc_amendments(coc_item_id: String) -> Result<Vec<DbCocAmendment>, String> {
    with_project_db(|db| db.get_coc_amendments(&coc_item_id))
}

/// Get audit log entries for a COC item (or all if coc_item_id is None).
#[tauri::command]
pub fn project_db_get_coc_audit_log(
    coc_item_id: Option<String>,
) -> Result<Vec<DbCocAuditEntry>, String> {
    with_project_db(|db| db.get_coc_audit_log(coc_item_id.as_deref()))
}

/// Insert a COC audit log entry.
#[tauri::command]
pub fn project_db_insert_coc_audit_entry(entry: DbCocAuditEntry) -> Result<(), String> {
    with_project_db(|db| db.insert_coc_audit_entry(&entry))
}

// =============================================================================
// COC Transfer Commands
// =============================================================================

/// Insert or update a COC transfer record.
#[tauri::command]
pub fn project_db_upsert_coc_transfer(record: DbCocTransfer) -> Result<(), String> {
    with_project_db(|db| db.upsert_coc_transfer(&record))
}

/// Get transfers for a specific COC item.
#[tauri::command]
pub fn project_db_get_coc_transfers(coc_item_id: String) -> Result<Vec<DbCocTransfer>, String> {
    with_project_db(|db| db.get_coc_transfers(&coc_item_id))
}

/// Get all COC transfers.
#[tauri::command]
pub fn project_db_get_all_coc_transfers() -> Result<Vec<DbCocTransfer>, String> {
    with_project_db(|db| db.get_all_coc_transfers())
}

/// Delete a COC transfer.
#[tauri::command]
pub fn project_db_delete_coc_transfer(id: String) -> Result<(), String> {
    with_project_db(|db| db.delete_coc_transfer(&id))
}
