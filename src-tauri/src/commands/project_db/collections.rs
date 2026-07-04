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
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::warn;

const PROJECT_METADATA_READ_MAX_BYTES: u64 = 16 * 1024 * 1024;
const EVIDENCE_COLLECTION_PACKAGE_READ_MAX_BYTES: u64 = 64 * 1024 * 1024;
const MAX_COLLECTION_RESPONSE_ROWS: usize = 10_000;
const MAX_COLLECTION_FIELD_CHARS: usize = 4096;
const MAX_COLLECTION_NOTE_CHARS: usize = 16_384;
const MAX_COLLECTION_HASH_CHARS: usize = 1024;
const MAX_COLLECTION_JSON_CHARS: usize = 65_536;
const MAX_COLLECTION_JSON_DEPTH: usize = 4;
const MAX_COLLECTION_JSON_ITEMS: usize = 256;
const COLLECTION_TRUNCATED_SUFFIX: &str = "... [truncated]";

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

fn read_utf8_file_with_limit(
    path: impl AsRef<Path>,
    max_bytes: u64,
    label: &str,
) -> Result<String, String> {
    let path = path.as_ref();
    if let Ok(metadata) = std::fs::metadata(path) {
        let size = metadata.len();
        if size > max_bytes {
            return Err(format!(
                "{} is too large: {} bytes > {} bytes",
                label, size, max_bytes
            ));
        }
    }

    let file = File::open(path)
        .map_err(|e| format!("Failed to open {} {}: {}", label, path.display(), e))?;
    let mut limited = file.take(max_bytes.saturating_add(1));
    let mut bytes = Vec::new();
    limited
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read {} {}: {}", label, path.display(), e))?;

    if bytes.len() as u64 > max_bytes {
        return Err(format!(
            "{} is too large: read more than {} bytes from {}",
            label,
            max_bytes,
            path.display()
        ));
    }

    String::from_utf8(bytes).map_err(|e| {
        format!(
            "Failed to decode {} {} as UTF-8: {}",
            label,
            path.display(),
            e
        )
    })
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

    let content = match read_utf8_file_with_limit(
        &cffx_path,
        PROJECT_METADATA_READ_MAX_BYTES,
        ".cffx project metadata",
    ) {
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

    Ok(bounded_import_package(ImportedEvidenceCollectionPackage {
        source_app: package.source_app,
        source_case_number: package.project.case_number,
        source_case_title: package.project.case_title,
        source_examiner_name: package.project.examiner_name,
        collections,
        coc_items,
    }))
}

fn build_evidence_collection_package(
    db: &ProjectDatabase,
    collection_id: &str,
    source_app: &str,
) -> Result<MobileEvidenceCollectionPackage, String> {
    let collection = bounded_evidence_collection(
        db.get_evidence_collection_by_id(collection_id)
            .map_err(|e| {
                format!(
                    "Failed to load evidence collection {}: {}",
                    collection_id, e
                )
            })?,
    );
    let items: Vec<_> = db
        .get_collected_items(collection_id)
        .map_err(|e| {
            format!(
                "Failed to load collected items for {}: {}",
                collection_id, e
            )
        })?
        .into_iter()
        .take(MAX_COLLECTION_RESPONSE_ROWS)
        .map(bounded_collected_item)
        .collect();

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
    let record = bounded_evidence_collection(record);
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
    .map(|collections| {
        collections
            .into_iter()
            .take(MAX_COLLECTION_RESPONSE_ROWS)
            .map(bounded_evidence_collection)
            .collect()
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
        .map(bounded_evidence_collection)
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
    let content = read_utf8_file_with_limit(
        &input_path,
        EVIDENCE_COLLECTION_PACKAGE_READ_MAX_BYTES,
        "evidence collection package",
    )?;
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
    let record = bounded_collected_item(record);
    with_project_db(window.label(), |db| db.upsert_collected_item(&record))
}

/// Get collected items for a specific collection.
#[tauri::command]
pub fn project_db_get_collected_items(
    window: tauri::Window,
    collection_id: String,
) -> Result<Vec<DbCollectedItem>, String> {
    with_project_db(window.label(), |db| db.get_collected_items(&collection_id)).map(|items| {
        items
            .into_iter()
            .take(MAX_COLLECTION_RESPONSE_ROWS)
            .map(bounded_collected_item)
            .collect()
    })
}

/// Get all collected items.
#[tauri::command]
pub fn project_db_get_all_collected_items(
    window: tauri::Window,
) -> Result<Vec<DbCollectedItem>, String> {
    with_project_db(window.label(), |db| db.get_all_collected_items()).map(|items| {
        items
            .into_iter()
            .take(MAX_COLLECTION_RESPONSE_ROWS)
            .map(bounded_collected_item)
            .collect()
    })
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
    let record = bounded_evidence_data_alternative(record);
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
    .map(|items| {
        items
            .into_iter()
            .take(MAX_COLLECTION_RESPONSE_ROWS)
            .map(bounded_evidence_data_alternative)
            .collect()
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
    .map(|items| {
        items
            .into_iter()
            .take(MAX_COLLECTION_RESPONSE_ROWS)
            .map(bounded_evidence_data_alternative)
            .collect()
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

fn bounded_import_package(
    mut package: ImportedEvidenceCollectionPackage,
) -> ImportedEvidenceCollectionPackage {
    package.source_app = truncate_collection_text(&package.source_app, MAX_COLLECTION_FIELD_CHARS);
    package.source_case_number =
        truncate_collection_text(&package.source_case_number, MAX_COLLECTION_FIELD_CHARS);
    package.source_case_title =
        truncate_collection_text(&package.source_case_title, MAX_COLLECTION_FIELD_CHARS);
    package.source_examiner_name =
        truncate_collection_text(&package.source_examiner_name, MAX_COLLECTION_FIELD_CHARS);
    package.collections = package
        .collections
        .into_iter()
        .take(MAX_COLLECTION_RESPONSE_ROWS)
        .map(|mut entry| {
            entry.collection = bounded_evidence_collection(entry.collection);
            entry.items = entry
                .items
                .into_iter()
                .take(MAX_COLLECTION_RESPONSE_ROWS)
                .map(bounded_collected_item)
                .collect();
            entry
        })
        .collect();
    package.coc_items = package
        .coc_items
        .into_iter()
        .take(MAX_COLLECTION_RESPONSE_ROWS)
        .collect();
    package
}

fn bounded_evidence_collection(mut record: DbEvidenceCollection) -> DbEvidenceCollection {
    record.id = truncate_collection_text(&record.id, MAX_COLLECTION_FIELD_CHARS);
    record.case_number = truncate_collection_text(&record.case_number, MAX_COLLECTION_FIELD_CHARS);
    record.collection_date =
        truncate_collection_text(&record.collection_date, MAX_COLLECTION_FIELD_CHARS);
    record.collection_location =
        truncate_collection_text(&record.collection_location, MAX_COLLECTION_FIELD_CHARS);
    record.collecting_officer =
        truncate_collection_text(&record.collecting_officer, MAX_COLLECTION_FIELD_CHARS);
    record.authorization =
        truncate_collection_text(&record.authorization, MAX_COLLECTION_FIELD_CHARS);
    record.authorization_date =
        truncate_collection_option(record.authorization_date, MAX_COLLECTION_FIELD_CHARS);
    record.authorizing_authority =
        truncate_collection_option(record.authorizing_authority, MAX_COLLECTION_FIELD_CHARS);
    record.witnesses_json = record
        .witnesses_json
        .map(|value| bounded_collection_json_text(&value, MAX_COLLECTION_JSON_CHARS));
    record.documentation_notes =
        truncate_collection_option(record.documentation_notes, MAX_COLLECTION_NOTE_CHARS);
    record.conditions = truncate_collection_option(record.conditions, MAX_COLLECTION_NOTE_CHARS);
    record.status = truncate_collection_text(&record.status, MAX_COLLECTION_FIELD_CHARS);
    record.created_at = truncate_collection_text(&record.created_at, MAX_COLLECTION_FIELD_CHARS);
    record.modified_at = truncate_collection_text(&record.modified_at, MAX_COLLECTION_FIELD_CHARS);
    record
}

fn bounded_collected_item(mut item: DbCollectedItem) -> DbCollectedItem {
    item.id = truncate_collection_text(&item.id, MAX_COLLECTION_FIELD_CHARS);
    item.collection_id = truncate_collection_text(&item.collection_id, MAX_COLLECTION_FIELD_CHARS);
    item.coc_item_id = truncate_collection_option(item.coc_item_id, MAX_COLLECTION_FIELD_CHARS);
    item.evidence_file_id =
        truncate_collection_option(item.evidence_file_id, MAX_COLLECTION_FIELD_CHARS);
    item.source_id = truncate_collection_option(item.source_id, MAX_COLLECTION_FIELD_CHARS);
    item.source_ref_json = item
        .source_ref_json
        .map(|value| bounded_collection_json_text(&value, MAX_COLLECTION_JSON_CHARS));
    item.item_number = truncate_collection_text(&item.item_number, MAX_COLLECTION_FIELD_CHARS);
    item.description = truncate_collection_text(&item.description, MAX_COLLECTION_NOTE_CHARS);
    item.found_location =
        truncate_collection_text(&item.found_location, MAX_COLLECTION_FIELD_CHARS);
    item.item_type = truncate_collection_text(&item.item_type, MAX_COLLECTION_FIELD_CHARS);
    item.make = truncate_collection_option(item.make, MAX_COLLECTION_FIELD_CHARS);
    item.model = truncate_collection_option(item.model, MAX_COLLECTION_FIELD_CHARS);
    item.serial_number = truncate_collection_option(item.serial_number, MAX_COLLECTION_FIELD_CHARS);
    item.condition = truncate_collection_text(&item.condition, MAX_COLLECTION_FIELD_CHARS);
    item.packaging = truncate_collection_text(&item.packaging, MAX_COLLECTION_FIELD_CHARS);
    item.packaging_type =
        truncate_collection_option(item.packaging_type, MAX_COLLECTION_FIELD_CHARS);
    item.packaging_detail =
        truncate_collection_option(item.packaging_detail, MAX_COLLECTION_FIELD_CHARS);
    item.photo_refs_json = item
        .photo_refs_json
        .map(|value| bounded_collection_json_text(&value, MAX_COLLECTION_JSON_CHARS));
    item.notes = truncate_collection_option(item.notes, MAX_COLLECTION_NOTE_CHARS);
    item.item_collection_datetime =
        truncate_collection_option(item.item_collection_datetime, MAX_COLLECTION_FIELD_CHARS);
    item.item_system_datetime =
        truncate_collection_option(item.item_system_datetime, MAX_COLLECTION_FIELD_CHARS);
    item.item_collecting_officer =
        truncate_collection_option(item.item_collecting_officer, MAX_COLLECTION_FIELD_CHARS);
    item.item_authorization =
        truncate_collection_option(item.item_authorization, MAX_COLLECTION_FIELD_CHARS);
    item.device_type = truncate_collection_option(item.device_type, MAX_COLLECTION_FIELD_CHARS);
    item.device_type_other =
        truncate_collection_option(item.device_type_other, MAX_COLLECTION_FIELD_CHARS);
    item.storage_interface =
        truncate_collection_option(item.storage_interface, MAX_COLLECTION_FIELD_CHARS);
    item.storage_interface_other =
        truncate_collection_option(item.storage_interface_other, MAX_COLLECTION_FIELD_CHARS);
    item.brand = truncate_collection_option(item.brand, MAX_COLLECTION_FIELD_CHARS);
    item.color = truncate_collection_option(item.color, MAX_COLLECTION_FIELD_CHARS);
    item.imei = truncate_collection_option(item.imei, MAX_COLLECTION_FIELD_CHARS);
    item.other_identifiers =
        truncate_collection_option(item.other_identifiers, MAX_COLLECTION_FIELD_CHARS);
    item.building = truncate_collection_option(item.building, MAX_COLLECTION_FIELD_CHARS);
    item.room = truncate_collection_option(item.room, MAX_COLLECTION_FIELD_CHARS);
    item.location_other =
        truncate_collection_option(item.location_other, MAX_COLLECTION_FIELD_CHARS);
    item.image_format = truncate_collection_option(item.image_format, MAX_COLLECTION_FIELD_CHARS);
    item.image_format_other =
        truncate_collection_option(item.image_format_other, MAX_COLLECTION_FIELD_CHARS);
    item.acquisition_method =
        truncate_collection_option(item.acquisition_method, MAX_COLLECTION_FIELD_CHARS);
    item.acquisition_method_other =
        truncate_collection_option(item.acquisition_method_other, MAX_COLLECTION_FIELD_CHARS);
    item.hash_algorithm =
        truncate_collection_option(item.hash_algorithm, MAX_COLLECTION_FIELD_CHARS);
    item.hash_value = truncate_collection_option(item.hash_value, MAX_COLLECTION_HASH_CHARS);
    item.hash_computed_at =
        truncate_collection_option(item.hash_computed_at, MAX_COLLECTION_FIELD_CHARS);
    item.storage_notes = truncate_collection_option(item.storage_notes, MAX_COLLECTION_NOTE_CHARS);
    item
}

fn bounded_evidence_data_alternative(
    mut record: DbEvidenceDataAlternative,
) -> DbEvidenceDataAlternative {
    record.id = truncate_collection_text(&record.id, MAX_COLLECTION_FIELD_CHARS);
    record.collected_item_id =
        truncate_collection_text(&record.collected_item_id, MAX_COLLECTION_FIELD_CHARS);
    record.evidence_file_id =
        truncate_collection_option(record.evidence_file_id, MAX_COLLECTION_FIELD_CHARS);
    record.field_name = truncate_collection_text(&record.field_name, MAX_COLLECTION_FIELD_CHARS);
    record.chosen_source =
        truncate_collection_text(&record.chosen_source, MAX_COLLECTION_FIELD_CHARS);
    record.user_value = truncate_collection_option(record.user_value, MAX_COLLECTION_NOTE_CHARS);
    record.container_value =
        truncate_collection_option(record.container_value, MAX_COLLECTION_NOTE_CHARS);
    record.resolved_by = truncate_collection_option(record.resolved_by, MAX_COLLECTION_FIELD_CHARS);
    record.resolved_at = truncate_collection_text(&record.resolved_at, MAX_COLLECTION_FIELD_CHARS);
    record.resolution_note =
        truncate_collection_option(record.resolution_note, MAX_COLLECTION_NOTE_CHARS);
    record
}

fn truncate_collection_option(value: Option<String>, max_chars: usize) -> Option<String> {
    value.map(|value| truncate_collection_text(&value, max_chars))
}

fn truncate_collection_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = COLLECTION_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + COLLECTION_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(COLLECTION_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_collection_json_text(value: &str, max_chars: usize) -> String {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return if value.chars().count() <= max_chars {
            value.to_string()
        } else {
            truncate_collection_text(value, max_chars)
        };
    };
    let bounded = bounded_collection_json_value(json, 0);
    let Ok(serialized) = serde_json::to_string(&bounded) else {
        return "{\"truncated\":true}".to_string();
    };

    if serialized.chars().count() <= max_chars {
        serialized
    } else {
        "{\"truncated\":true}".to_string()
    }
}

fn bounded_collection_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_COLLECTION_JSON_DEPTH {
        return serde_json::Value::String(COLLECTION_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(value) => {
            serde_json::Value::String(truncate_collection_text(&value, MAX_COLLECTION_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_COLLECTION_JSON_ITEMS)
                .map(|value| bounded_collection_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(object) => {
            let mut bounded = serde_json::Map::new();
            for (key, value) in object.into_iter().take(MAX_COLLECTION_JSON_ITEMS) {
                bounded.insert(
                    truncate_collection_text(&key, MAX_COLLECTION_FIELD_CHARS),
                    bounded_collection_json_value(value, depth + 1),
                );
            }
            serde_json::Value::Object(bounded)
        }
        value @ (serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)) => value,
    }
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

    #[test]
    fn read_utf8_file_with_limit_accepts_bounded_text() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("package.json");
        std::fs::write(&path, "bounded package").unwrap();

        let content = read_utf8_file_with_limit(&path, 64, "test package").unwrap();

        assert_eq!(content, "bounded package");
    }

    #[test]
    fn read_utf8_file_with_limit_rejects_oversize_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("oversize-package.json");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(65).unwrap();

        let err = read_utf8_file_with_limit(&path, 64, "test package").unwrap_err();

        assert!(err.contains("test package is too large"));
        assert!(err.contains("65 bytes > 64 bytes"));
    }

    #[test]
    fn read_utf8_file_with_limit_rejects_invalid_utf8() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("invalid-package.json");
        std::fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();

        let err = read_utf8_file_with_limit(&path, 64, "test package").unwrap_err();

        assert!(err.contains("Failed to decode test package"));
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
                        source_id: Some("ev-existing:/Users/test/phone.E01".to_string()),
                        source_ref_json: Some(
                            r#"{"type":"localFile","path":"/Users/test/phone.E01"}"#.to_string(),
                        ),
                        item_number: "ITEM-001".to_string(),
                        description: "Imported iPhone".to_string(),
                        found_location: "Desk".to_string(),
                        item_type: "Phone".to_string(),
                        make: None,
                        model: None,
                        serial_number: None,
                        condition: "Good".to_string(),
                        packaging: "Bag".to_string(),
                        packaging_type: Some("faraday_bag".to_string()),
                        packaging_detail: Some("Mission Darkness bag".to_string()),
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
                        hash_algorithm: Some("sha256".to_string()),
                        hash_value: Some(
                            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                                .to_string(),
                        ),
                        hash_computed_at: Some("2026-04-14T10:02:00Z".to_string()),
                        storage_notes: None,
                    },
                    DbCollectedItem {
                        id: "item-src-2".to_string(),
                        collection_id: "collection-src-1".to_string(),
                        coc_item_id: Some("coc-missing".to_string()),
                        evidence_file_id: Some("ev-missing".to_string()),
                        source_id: None,
                        source_ref_json: None,
                        item_number: "ITEM-002".to_string(),
                        description: String::new(),
                        found_location: String::new(),
                        item_type: String::new(),
                        make: None,
                        model: None,
                        serial_number: None,
                        condition: String::new(),
                        packaging: String::new(),
                        packaging_type: None,
                        packaging_detail: None,
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
                        hash_algorithm: None,
                        hash_value: None,
                        hash_computed_at: None,
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
                    storage_location: Some("Shelf 7".to_string()),
                    storage_class: Some("evidence_locker".to_string()),
                    storage_location_detail: Some("Shelf 7".to_string()),
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
                    storage_class: Some("evidence_locker".to_string()),
                    storage_location_detail: Some("Shelf 7".to_string()),
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

    fn make_collected_item() -> DbCollectedItem {
        DbCollectedItem {
            id: "item-1".to_string(),
            collection_id: "collection-1".to_string(),
            coc_item_id: Some("coc-1".to_string()),
            evidence_file_id: Some("ev-1".to_string()),
            source_id: Some("source-1".to_string()),
            source_ref_json: Some(r#"{"kind":"localFile","path":"/case/item.dd"}"#.to_string()),
            item_number: "ITEM-001".to_string(),
            description: "Collected item".to_string(),
            found_location: "Desk".to_string(),
            item_type: "Drive".to_string(),
            make: Some("CORE".to_string()),
            model: Some("Model".to_string()),
            serial_number: Some("SERIAL".to_string()),
            condition: "Good".to_string(),
            packaging: "Bag".to_string(),
            packaging_type: Some("evidence_bag".to_string()),
            packaging_detail: Some("Tamper evident".to_string()),
            photo_refs_json: Some(r#"["IMG_001"]"#.to_string()),
            notes: Some("notes".to_string()),
            item_collection_datetime: None,
            item_system_datetime: None,
            item_collecting_officer: None,
            item_authorization: None,
            device_type: Some("external_drive".to_string()),
            device_type_other: None,
            storage_interface: Some("usb".to_string()),
            storage_interface_other: None,
            brand: None,
            color: None,
            imei: None,
            other_identifiers: None,
            building: None,
            room: None,
            location_other: None,
            image_format: Some("e01".to_string()),
            image_format_other: None,
            acquisition_method: Some("physical".to_string()),
            acquisition_method_other: None,
            hash_algorithm: Some("sha256".to_string()),
            hash_value: Some("a".repeat(64)),
            hash_computed_at: Some("2026-04-14T10:02:00Z".to_string()),
            storage_notes: None,
        }
    }

    #[test]
    fn bounded_evidence_collection_caps_text_and_json() {
        let collection = DbEvidenceCollection {
            id: "collection-1".to_string(),
            case_number: "c".repeat(MAX_COLLECTION_FIELD_CHARS + 32),
            collection_date: "2026-04-14".to_string(),
            collection_location: "Scene A".to_string(),
            collecting_officer: "Officer".to_string(),
            authorization: "Consent".to_string(),
            authorization_date: None,
            authorizing_authority: None,
            witnesses_json: Some(
                serde_json::json!((0..(MAX_COLLECTION_JSON_ITEMS + 10)).collect::<Vec<_>>())
                    .to_string(),
            ),
            documentation_notes: Some("n".repeat(MAX_COLLECTION_NOTE_CHARS + 32)),
            conditions: None,
            status: "draft".to_string(),
            created_at: "2026-04-14T10:00:00Z".to_string(),
            modified_at: "2026-04-14T10:00:00Z".to_string(),
            item_count: 0,
        };

        let bounded = bounded_evidence_collection(collection);

        assert_eq!(
            bounded.case_number.chars().count(),
            MAX_COLLECTION_FIELD_CHARS
        );
        assert!(bounded.case_number.ends_with(COLLECTION_TRUNCATED_SUFFIX));
        assert_eq!(
            bounded
                .documentation_notes
                .as_deref()
                .unwrap()
                .chars()
                .count(),
            MAX_COLLECTION_NOTE_CHARS
        );
        let witnesses: serde_json::Value =
            serde_json::from_str(bounded.witnesses_json.as_deref().unwrap()).unwrap();
        assert_eq!(
            witnesses.as_array().unwrap().len(),
            MAX_COLLECTION_JSON_ITEMS
        );
    }

    #[test]
    fn bounded_collected_item_caps_source_refs_notes_and_hashes() {
        let mut item = make_collected_item();
        item.source_id = Some("s".repeat(MAX_COLLECTION_FIELD_CHARS + 32));
        item.source_ref_json = Some(
            serde_json::json!({
                "kind": "localFile",
                "path": "x".repeat(MAX_COLLECTION_JSON_CHARS + 32)
            })
            .to_string(),
        );
        item.photo_refs_json = Some(
            serde_json::json!((0..(MAX_COLLECTION_JSON_ITEMS + 10)).collect::<Vec<_>>())
                .to_string(),
        );
        item.notes = Some("n".repeat(MAX_COLLECTION_NOTE_CHARS + 32));
        item.hash_value = Some("h".repeat(MAX_COLLECTION_HASH_CHARS + 32));

        let bounded = bounded_collected_item(item);

        assert_eq!(
            bounded.source_id.as_deref().unwrap().chars().count(),
            MAX_COLLECTION_FIELD_CHARS
        );
        assert_eq!(
            bounded.notes.as_deref().unwrap().chars().count(),
            MAX_COLLECTION_NOTE_CHARS
        );
        assert_eq!(
            bounded.hash_value.as_deref().unwrap().chars().count(),
            MAX_COLLECTION_HASH_CHARS
        );
        assert!(serde_json::from_str::<serde_json::Value>(
            bounded.source_ref_json.as_deref().unwrap()
        )
        .is_ok());
        let photo_refs: serde_json::Value =
            serde_json::from_str(bounded.photo_refs_json.as_deref().unwrap()).unwrap();
        assert_eq!(
            photo_refs.as_array().unwrap().len(),
            MAX_COLLECTION_JSON_ITEMS
        );
    }

    #[test]
    fn bounded_evidence_data_alternative_caps_values() {
        let alternative = DbEvidenceDataAlternative {
            id: "alt-1".to_string(),
            collected_item_id: "item-1".to_string(),
            evidence_file_id: Some("ev-1".to_string()),
            field_name: "serial_number".to_string(),
            chosen_source: "user".to_string(),
            user_value: Some("u".repeat(MAX_COLLECTION_NOTE_CHARS + 32)),
            container_value: Some("c".repeat(MAX_COLLECTION_NOTE_CHARS + 32)),
            resolved_by: Some("analyst".to_string()),
            resolved_at: "2026-04-14T10:00:00Z".to_string(),
            resolution_note: Some("r".repeat(MAX_COLLECTION_NOTE_CHARS + 32)),
        };

        let bounded = bounded_evidence_data_alternative(alternative);

        assert_eq!(
            bounded.user_value.as_deref().unwrap().chars().count(),
            MAX_COLLECTION_NOTE_CHARS
        );
        assert_eq!(
            bounded.container_value.as_deref().unwrap().chars().count(),
            MAX_COLLECTION_NOTE_CHARS
        );
        assert_eq!(
            bounded.resolution_note.as_deref().unwrap().chars().count(),
            MAX_COLLECTION_NOTE_CHARS
        );
    }

    #[test]
    fn bounded_import_package_caps_collection_and_item_counts() {
        let mut package = make_import_package();
        package.source_app = "a".repeat(MAX_COLLECTION_FIELD_CHARS + 32);
        let item_heavy_collection = ImportedEvidenceCollectionPackageCollection {
            collection: package.collections[0].collection.clone(),
            items: vec![make_collected_item(); MAX_COLLECTION_RESPONSE_ROWS + 1],
        };
        package.collections = vec![
            ImportedEvidenceCollectionPackageCollection {
                collection: package.collections[0].collection.clone(),
                items: vec![make_collected_item()],
            };
            MAX_COLLECTION_RESPONSE_ROWS + 1
        ];
        package.collections[0] = item_heavy_collection;

        let bounded = bounded_import_package(package);

        assert_eq!(
            bounded.source_app.chars().count(),
            MAX_COLLECTION_FIELD_CHARS
        );
        assert_eq!(bounded.collections.len(), MAX_COLLECTION_RESPONSE_ROWS);
        assert_eq!(
            bounded.collections[0].items.len(),
            MAX_COLLECTION_RESPONSE_ROWS
        );
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
        assert_eq!(
            preserved_item.get("sourceId").and_then(Value::as_str),
            Some("ev-existing:/Users/test/phone.E01")
        );
        assert_eq!(
            serde_json::from_str::<Value>(
                preserved_item
                    .get("sourceRefJson")
                    .and_then(Value::as_str)
                    .expect("source ref json")
            )
            .unwrap(),
            serde_json::json!({"type":"localFile","path":"/Users/test/phone.E01"})
        );
        assert_eq!(
            preserved_item.get("hashAlgorithm").and_then(Value::as_str),
            Some("sha256")
        );
        assert_eq!(
            preserved_item.get("hashValue").and_then(Value::as_str),
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        );
        assert_eq!(
            preserved_item.get("hashComputedAt").and_then(Value::as_str),
            Some("2026-04-14T10:02:00Z")
        );
        assert_eq!(
            preserved_item.get("packagingType").and_then(Value::as_str),
            Some("faraday_bag")
        );
        assert_eq!(
            preserved_item
                .get("packagingDetail")
                .and_then(Value::as_str),
            Some("Mission Darkness bag")
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
        assert_eq!(
            coc_json.get("storageClass").and_then(Value::as_str),
            Some("evidence_locker")
        );
        assert_eq!(
            coc_json
                .get("storageLocationDetail")
                .and_then(Value::as_str),
            Some("Shelf 7")
        );
        assert_eq!(coc_json["transfers"].as_array().map(Vec::len), Some(1));
        assert_eq!(coc_json["amendments"].as_array().map(Vec::len), Some(1));
        assert_eq!(coc_json["auditLog"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            coc_json["transfers"][0]
                .get("storageClass")
                .and_then(Value::as_str),
            Some("evidence_locker")
        );
        assert_eq!(
            coc_json["transfers"][0]
                .get("storageLocationDetail")
                .and_then(Value::as_str),
            Some("Shelf 7")
        );
        assert_eq!(
            coc_json["auditLog"][0]
                .get("action")
                .and_then(Value::as_str),
            Some("imported")
        );
    }
}
