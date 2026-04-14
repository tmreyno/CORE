// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for evidence collections and collected items.

use super::{with_project_db, with_project_db_result};
use crate::project::FFXProject;
use crate::project_db::{
    DbCocItem, DbCollectedItem, DbEvidenceCollection, DbEvidenceDataAlternative,
    EvidenceCollectionPackageImportSummary, ImportedEvidenceCollectionPackage,
    ImportedEvidenceCollectionPackageCocItem, ImportedEvidenceCollectionPackageCollection,
    ProjectDatabase,
};
use core_types::evidence_collection_contract::EVIDENCE_COLLECTION_PACKAGE_VERSION;
use core_types::mobile::{
    MobileEvidenceCollectionPackage, MobileEvidenceCollectionPackageCocItem,
    MobileEvidenceCollectionPackageCollection, MobileProject,
};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::warn;

fn convert_shared_type<TSrc, TDst>(value: &TSrc, label: &str) -> Result<TDst, String>
where
    TSrc: Serialize,
    TDst: DeserializeOwned,
{
    let json =
        serde_json::to_value(value).map_err(|e| format!("Failed to serialize {}: {}", label, e))?;
    serde_json::from_value(json)
        .map_err(|e| format!("Failed to convert {} to shared type: {}", label, e))
}

fn fallback_mobile_project(
    db: &ProjectDatabase,
    collection: &DbEvidenceCollection,
) -> MobileProject {
    let file_stem = db
        .path()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("project")
        .to_string();

    MobileProject {
        id: file_stem.clone(),
        case_number: if collection.case_number.is_empty() {
            file_stem.clone()
        } else {
            collection.case_number.clone()
        },
        case_title: file_stem,
        examiner_name: collection.collecting_officer.clone(),
        organization: String::new(),
        created_at: collection.created_at.clone(),
        modified_at: collection.modified_at.clone(),
        status: "active".to_string(),
    }
}

fn load_mobile_project(db: &ProjectDatabase, collection: &DbEvidenceCollection) -> MobileProject {
    let cffx_path = db.path().with_extension("cffx");
    let fallback = || fallback_mobile_project(db, collection);

    let content = match std::fs::read_to_string(&cffx_path) {
        Ok(content) => content,
        Err(error) => {
            warn!(
                path = %cffx_path.display(),
                "Falling back to minimal package project metadata: {}",
                error
            );
            return fallback();
        }
    };

    let project = match serde_json::from_str::<FFXProject>(&content) {
        Ok(project) => project,
        Err(error) => {
            warn!(
                path = %cffx_path.display(),
                "Failed to parse .cffx for package metadata, using fallback: {}",
                error
            );
            return fallback();
        }
    };

    MobileProject {
        id: project.project_id,
        case_number: project
            .case_number
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| collection.case_number.clone()),
        case_title: project
            .case_name
            .filter(|value| !value.is_empty())
            .unwrap_or(project.name),
        examiner_name: project
            .owner_name
            .filter(|value| !value.is_empty())
            .or_else(|| project.current_user.filter(|value| !value.is_empty()))
            .unwrap_or_else(|| collection.collecting_officer.clone()),
        organization: String::new(),
        created_at: project.created_at,
        modified_at: project.saved_at,
        status: "active".to_string(),
    }
}

fn build_linked_coc_items(
    db: &ProjectDatabase,
    items: &[DbCollectedItem],
) -> Result<Vec<MobileEvidenceCollectionPackageCocItem>, String> {
    let mut linked_coc_ids = Vec::new();
    let mut seen_ids = HashSet::new();

    for item in items {
        if let Some(coc_item_id) = item.coc_item_id.as_ref() {
            if seen_ids.insert(coc_item_id.clone()) {
                linked_coc_ids.push(coc_item_id.clone());
            }
        }
    }

    if linked_coc_ids.is_empty() {
        return Ok(Vec::new());
    }

    let coc_items = db
        .get_coc_items(None)
        .map_err(|e| format!("Failed to load linked COC items: {}", e))?;
    let coc_item_map: HashMap<String, DbCocItem> = coc_items
        .into_iter()
        .filter(|item| seen_ids.contains(&item.id))
        .map(|item| (item.id.clone(), item))
        .collect();

    let mut export_items = Vec::with_capacity(linked_coc_ids.len());
    for coc_item_id in linked_coc_ids {
        let Some(coc_item) = coc_item_map.get(&coc_item_id) else {
            warn!(
                coc_item_id = %coc_item_id,
                "Collected item references missing COC item during package export"
            );
            continue;
        };

        let transfers: Vec<core_types::coc::DbCocTransfer> = convert_shared_type(
            &db.get_coc_transfers(&coc_item_id)
                .map_err(|e| format!("Failed to load COC transfers for {}: {}", coc_item_id, e))?,
            "COC transfers",
        )?;
        let amendments: Vec<core_types::coc::DbCocAmendment> = convert_shared_type(
            &db.get_coc_amendments(&coc_item_id)
                .map_err(|e| format!("Failed to load COC amendments for {}: {}", coc_item_id, e))?,
            "COC amendments",
        )?;
        let audit_log: Vec<core_types::coc::DbCocAuditEntry> = convert_shared_type(
            &db.get_coc_audit_log(Some(&coc_item_id))
                .map_err(|e| format!("Failed to load COC audit log for {}: {}", coc_item_id, e))?,
            "COC audit log",
        )?;

        export_items.push(MobileEvidenceCollectionPackageCocItem {
            item: convert_shared_type(coc_item, "COC item")?,
            transfers,
            amendments,
            audit_log,
        });
    }

    Ok(export_items)
}

fn convert_import_package(
    package: MobileEvidenceCollectionPackage,
) -> Result<ImportedEvidenceCollectionPackage, String> {
    let collections = package
        .collections
        .into_iter()
        .map(|entry| {
            Ok(ImportedEvidenceCollectionPackageCollection {
                collection: convert_shared_type(&entry.collection, "package collection")?,
                items: convert_shared_type(&entry.items, "package collected items")?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let coc_items = package
        .coc_items
        .into_iter()
        .map(|entry| {
            Ok(ImportedEvidenceCollectionPackageCocItem {
                item: convert_shared_type(&entry.item, "package COC item")?,
                transfers: convert_shared_type(&entry.transfers, "package COC transfers")?,
                amendments: convert_shared_type(&entry.amendments, "package COC amendments")?,
                audit_log: convert_shared_type(&entry.audit_log, "package COC audit log")?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(ImportedEvidenceCollectionPackage {
        source_app: package.source_app,
        source_case_number: package.project.case_number,
        source_case_title: package.project.case_title,
        source_examiner_name: package.project.examiner_name,
        collections,
        coc_items,
    })
}

fn build_evidence_collection_package(
    db: &ProjectDatabase,
    collection_id: &str,
    source_app: &str,
) -> Result<MobileEvidenceCollectionPackage, String> {
    let collection = db
        .get_evidence_collection_by_id(collection_id)
        .map_err(|e| {
            format!(
                "Failed to load evidence collection {}: {}",
                collection_id, e
            )
        })?;
    let items = db.get_collected_items(collection_id).map_err(|e| {
        format!(
            "Failed to load collected items for {}: {}",
            collection_id, e
        )
    })?;

    Ok(MobileEvidenceCollectionPackage {
        export_version: EVIDENCE_COLLECTION_PACKAGE_VERSION.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        source_app: source_app.to_string(),
        project: load_mobile_project(db, &collection),
        collections: vec![MobileEvidenceCollectionPackageCollection {
            collection: convert_shared_type(&collection, "evidence collection")?,
            items: convert_shared_type(&items, "collected items")?,
        }],
        coc_items: build_linked_coc_items(db, &items)?,
    })
}

// =============================================================================
// Evidence Collection Commands
// =============================================================================

/// Insert or update an evidence collection record.
#[tauri::command]
pub fn project_db_upsert_evidence_collection(
    window: tauri::Window,
    record: DbEvidenceCollection,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_evidence_collection(&record))
}

/// Get evidence collections, optionally filtered by case number.
#[tauri::command]
pub fn project_db_get_evidence_collections(
    window: tauri::Window,
    case_number: Option<String>,
) -> Result<Vec<DbEvidenceCollection>, String> {
    with_project_db(window.label(), |db| {
        db.get_evidence_collections(case_number.as_deref())
    })
}

/// Delete an evidence collection.
#[tauri::command]
pub fn project_db_delete_evidence_collection(
    window: tauri::Window,
    id: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_evidence_collection(&id))
}

/// Get a single evidence collection by ID (with item count).
#[tauri::command]
pub fn project_db_get_evidence_collection_by_id(
    window: tauri::Window,
    id: String,
) -> Result<DbEvidenceCollection, String> {
    with_project_db(window.label(), |db| db.get_evidence_collection_by_id(&id))
}

/// Export a single evidence collection as the canonical portable package JSON.
#[tauri::command]
pub fn project_db_export_evidence_collection_package(
    window: tauri::Window,
    collection_id: String,
    output_path: String,
) -> Result<String, String> {
    with_project_db_result(window.label(), |db| {
        let package = build_evidence_collection_package(db, &collection_id, "CORE-FFX")?;

        let json = serde_json::to_string_pretty(&package)
            .map_err(|e| format!("Failed to serialize evidence collection package: {}", e))?;
        std::fs::write(&output_path, json)
            .map_err(|e| format!("Failed to write evidence collection package: {}", e))?;

        Ok(output_path)
    })
}

/// Import a portable evidence collection package into the current project.
#[tauri::command]
pub fn project_db_import_evidence_collection_package(
    window: tauri::Window,
    input_path: String,
) -> Result<EvidenceCollectionPackageImportSummary, String> {
    let content = std::fs::read_to_string(&input_path)
        .map_err(|e| format!("Failed to read evidence collection package: {}", e))?;
    let package = serde_json::from_str::<MobileEvidenceCollectionPackage>(&content)
        .map_err(|e| format!("Failed to parse evidence collection package: {}", e))?;
    let imported = convert_import_package(package)?;

    with_project_db_result(window.label(), |db| {
        db.import_evidence_collection_package(&imported)
            .map_err(|e| format!("Failed to import evidence collection package: {}", e))
    })
}

/// Update evidence collection status (draft → complete → locked).
#[tauri::command]
pub fn project_db_update_evidence_collection_status(
    window: tauri::Window,
    id: String,
    new_status: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.update_evidence_collection_status(&id, &new_status)
    })
}

// =============================================================================
// Collected Item Commands
// =============================================================================

/// Insert or update a collected item.
#[tauri::command]
pub fn project_db_upsert_collected_item(
    window: tauri::Window,
    record: DbCollectedItem,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_collected_item(&record))
}

/// Get collected items for a specific collection.
#[tauri::command]
pub fn project_db_get_collected_items(
    window: tauri::Window,
    collection_id: String,
) -> Result<Vec<DbCollectedItem>, String> {
    with_project_db(window.label(), |db| db.get_collected_items(&collection_id))
}

/// Get all collected items.
#[tauri::command]
pub fn project_db_get_all_collected_items(
    window: tauri::Window,
) -> Result<Vec<DbCollectedItem>, String> {
    with_project_db(window.label(), |db| db.get_all_collected_items())
}

/// Delete a collected item.
#[tauri::command]
pub fn project_db_delete_collected_item(window: tauri::Window, id: String) -> Result<(), String> {
    with_project_db(window.label(), |db| db.delete_collected_item(&id))
}

// =============================================================================
// Evidence Data Alternative Commands
// =============================================================================

/// Insert or update an evidence data alternative record.
#[tauri::command]
pub fn project_db_upsert_evidence_data_alternative(
    window: tauri::Window,
    record: DbEvidenceDataAlternative,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.upsert_evidence_data_alternative(&record)
    })
}

/// Get all evidence data alternatives for a collected item.
#[tauri::command]
pub fn project_db_get_evidence_data_alternatives(
    window: tauri::Window,
    collected_item_id: String,
) -> Result<Vec<DbEvidenceDataAlternative>, String> {
    with_project_db(window.label(), |db| {
        db.get_evidence_data_alternatives(&collected_item_id)
    })
}

/// Get all evidence data alternatives for a specific evidence file.
#[tauri::command]
pub fn project_db_get_evidence_data_alternatives_by_file(
    window: tauri::Window,
    evidence_file_id: String,
) -> Result<Vec<DbEvidenceDataAlternative>, String> {
    with_project_db(window.label(), |db| {
        db.get_evidence_data_alternatives_by_file(&evidence_file_id)
    })
}

/// Delete a single evidence data alternative record.
#[tauri::command]
pub fn project_db_delete_evidence_data_alternative(
    window: tauri::Window,
    id: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.delete_evidence_data_alternative(&id)
    })
}

/// Delete all evidence data alternatives for a collected item.
#[tauri::command]
pub fn project_db_delete_evidence_data_alternatives_for_item(
    window: tauri::Window,
    collected_item_id: String,
) -> Result<(), String> {
    with_project_db(window.label(), |db| {
        db.delete_evidence_data_alternatives_for_item(&collected_item_id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_db::{
        DbCocAmendment, DbCocTransfer, DbEvidenceFile, ImportedEvidenceCollectionPackageCocItem,
        ImportedEvidenceCollectionPackageCollection,
    };
    use serde_json::Value;
    use tempfile::TempDir;

    fn create_test_db() -> (TempDir, ProjectDatabase) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.ffxdb");
        let db = ProjectDatabase::open(&db_path).expect("Failed to create test DB");
        (temp_dir, db)
    }

    fn make_import_package() -> ImportedEvidenceCollectionPackage {
        ImportedEvidenceCollectionPackage {
            source_app: "CORE-EVD".to_string(),
            source_case_number: "CASE-IMPORT-1".to_string(),
            source_case_title: "Imported Package Case".to_string(),
            source_examiner_name: "Examiner Import".to_string(),
            collections: vec![ImportedEvidenceCollectionPackageCollection {
                collection: DbEvidenceCollection {
                    id: "collection-src-1".to_string(),
                    case_number: String::new(),
                    collection_date: "2026-04-14".to_string(),
                    collection_location: "Scene A".to_string(),
                    collecting_officer: "Officer Import".to_string(),
                    authorization: "Consent".to_string(),
                    authorization_date: None,
                    authorizing_authority: None,
                    witnesses_json: None,
                    documentation_notes: None,
                    conditions: None,
                    status: "complete".to_string(),
                    created_at: "2026-04-14T10:00:00Z".to_string(),
                    modified_at: "2026-04-14T10:00:00Z".to_string(),
                    item_count: 0,
                },
                items: vec![
                    DbCollectedItem {
                        id: "item-src-1".to_string(),
                        collection_id: "collection-src-1".to_string(),
                        coc_item_id: Some("coc-src-1".to_string()),
                        evidence_file_id: Some("ev-existing".to_string()),
                        item_number: "ITEM-001".to_string(),
                        description: "Imported iPhone".to_string(),
                        found_location: "Desk".to_string(),
                        item_type: "Phone".to_string(),
                        make: None,
                        model: None,
                        serial_number: None,
                        condition: "Good".to_string(),
                        packaging: "Bag".to_string(),
                        photo_refs_json: None,
                        notes: None,
                        item_collection_datetime: None,
                        item_system_datetime: None,
                        item_collecting_officer: None,
                        item_authorization: None,
                        device_type: None,
                        device_type_other: None,
                        storage_interface: None,
                        storage_interface_other: None,
                        brand: None,
                        color: None,
                        imei: None,
                        other_identifiers: None,
                        building: None,
                        room: None,
                        location_other: None,
                        image_format: None,
                        image_format_other: None,
                        acquisition_method: None,
                        acquisition_method_other: None,
                        storage_notes: None,
                    },
                    DbCollectedItem {
                        id: "item-src-2".to_string(),
                        collection_id: "collection-src-1".to_string(),
                        coc_item_id: Some("coc-missing".to_string()),
                        evidence_file_id: Some("ev-missing".to_string()),
                        item_number: "ITEM-002".to_string(),
                        description: String::new(),
                        found_location: String::new(),
                        item_type: String::new(),
                        make: None,
                        model: None,
                        serial_number: None,
                        condition: String::new(),
                        packaging: String::new(),
                        photo_refs_json: None,
                        notes: None,
                        item_collection_datetime: None,
                        item_system_datetime: None,
                        item_collecting_officer: None,
                        item_authorization: None,
                        device_type: None,
                        device_type_other: None,
                        storage_interface: None,
                        storage_interface_other: None,
                        brand: None,
                        color: None,
                        imei: None,
                        other_identifiers: None,
                        building: None,
                        room: None,
                        location_other: None,
                        image_format: None,
                        image_format_other: None,
                        acquisition_method: None,
                        acquisition_method_other: None,
                        storage_notes: None,
                    },
                ],
            }],
            coc_items: vec![ImportedEvidenceCollectionPackageCocItem {
                item: DbCocItem {
                    id: "coc-src-1".to_string(),
                    coc_number: "COC-001".to_string(),
                    evidence_file_id: Some("ev-existing".to_string()),
                    case_number: String::new(),
                    evidence_id: "EVID-001".to_string(),
                    description: "Imported phone custody".to_string(),
                    item_type: "Device".to_string(),
                    case_title: None,
                    office: None,
                    owner_name: None,
                    owner_address: None,
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
                    condition: "Good".to_string(),
                    acquisition_date: "2026-04-14T10:00:00Z".to_string(),
                    entered_custody_date: "2026-04-14T10:05:00Z".to_string(),
                    submitted_by: "Examiner Import".to_string(),
                    collected_date: None,
                    received_by: "Evidence Room".to_string(),
                    received_location: None,
                    storage_location: None,
                    reason_submitted: None,
                    intake_hashes_json: None,
                    notes: Some("Imported note".to_string()),
                    disposition: None,
                    disposition_by: None,
                    returned_to: None,
                    destruction_date: None,
                    disposition_date: None,
                    disposition_notes: None,
                    created_at: "2026-04-14T10:00:00Z".to_string(),
                    modified_at: "2026-04-14T10:05:00Z".to_string(),
                    status: "locked".to_string(),
                    locked_at: Some("2026-04-14T10:05:00Z".to_string()),
                    locked_by: Some("EX".to_string()),
                },
                transfers: vec![DbCocTransfer {
                    id: "transfer-src-1".to_string(),
                    coc_item_id: "coc-src-1".to_string(),
                    timestamp: "2026-04-14T10:06:00Z".to_string(),
                    released_by: "Examiner Import".to_string(),
                    received_by: "Evidence Room".to_string(),
                    purpose: "Storage".to_string(),
                    location: Some("Locker A".to_string()),
                    storage_location: Some("Shelf 7".to_string()),
                    storage_date: Some("2026-04-14".to_string()),
                    method: Some("Hand-delivered".to_string()),
                    notes: None,
                }],
                amendments: vec![DbCocAmendment {
                    id: "amendment-src-1".to_string(),
                    coc_item_id: "coc-src-1".to_string(),
                    field_name: "description".to_string(),
                    old_value: "Phone".to_string(),
                    new_value: "Imported phone custody".to_string(),
                    amended_by_initials: "EX".to_string(),
                    amended_at: "2026-04-14T10:07:00Z".to_string(),
                    reason: Some("Clarified wording".to_string()),
                }],
                audit_log: Vec::new(),
            }],
        }
    }

    #[test]
    fn build_evidence_collection_package_preserves_flattened_export_shape_after_import() {
        let (_dir, db) = create_test_db();

        db.upsert_evidence_file(&DbEvidenceFile {
            id: "ev-existing".to_string(),
            path: "/case/evidence/phone.E01".to_string(),
            filename: "phone.E01".to_string(),
            container_type: "e01".to_string(),
            total_size: 1_024,
            segment_count: 1,
            discovered_at: "2026-04-14T09:55:00Z".to_string(),
            created: None,
            modified: None,
        })
        .unwrap();

        db.import_evidence_collection_package(&make_import_package())
            .unwrap();

        let collection = db
            .get_evidence_collections(None)
            .unwrap()
            .into_iter()
            .find(|entry| entry.collection_location == "Scene A")
            .expect("expected imported collection");

        let package = build_evidence_collection_package(&db, &collection.id, "CORE-FFX").unwrap();
        assert_eq!(package.export_version, EVIDENCE_COLLECTION_PACKAGE_VERSION);
        assert_eq!(package.source_app, "CORE-FFX");
        assert_eq!(package.project.case_number, "CASE-IMPORT-1");
        assert_eq!(package.collections.len(), 1);
        assert_eq!(package.collections[0].items.len(), 2);
        assert_eq!(package.coc_items.len(), 1);

        let json = serde_json::to_value(&package).unwrap();
        let collection_json = &json["collections"][0];
        assert!(collection_json.get("collection").is_none());
        assert_eq!(
            collection_json.get("caseNumber").and_then(Value::as_str),
            Some("CASE-IMPORT-1")
        );

        let items = collection_json["items"].as_array().expect("items array");
        assert_eq!(items.len(), 2);
        let preserved_item = items
            .iter()
            .find(|item| item.get("itemNumber").and_then(Value::as_str) == Some("ITEM-001"))
            .expect("preserved item");
        assert_eq!(
            preserved_item.get("evidenceFileId").and_then(Value::as_str),
            Some("ev-existing")
        );
        assert_ne!(
            preserved_item
                .get("cocItemId")
                .and_then(Value::as_str)
                .expect("remapped coc id"),
            "coc-src-1"
        );

        let dropped_item = items
            .iter()
            .find(|item| item.get("itemNumber").and_then(Value::as_str) == Some("ITEM-002"))
            .expect("dropped-link item");
        assert_eq!(
            dropped_item.get("cocItemId").map(Value::is_null),
            Some(true)
        );
        assert_eq!(
            dropped_item.get("evidenceFileId").map(Value::is_null),
            Some(true)
        );

        let coc_json = &json["cocItems"][0];
        assert!(coc_json.get("item").is_none());
        assert_eq!(
            coc_json.get("cocNumber").and_then(Value::as_str),
            Some("COC-001")
        );
        assert_eq!(coc_json["transfers"].as_array().map(Vec::len), Some(1));
        assert_eq!(coc_json["amendments"].as_array().map(Vec::len), Some(1));
        assert_eq!(coc_json["auditLog"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            coc_json["auditLog"][0]
                .get("action")
                .and_then(Value::as_str),
            Some("imported")
        );
    }
}
