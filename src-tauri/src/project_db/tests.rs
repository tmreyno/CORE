// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tests for ProjectDatabase operations.

use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_db() -> (TempDir, ProjectDatabase) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.ffxdb");
    let db = ProjectDatabase::open(&db_path).expect("Failed to create test DB");
    (temp_dir, db)
}

fn make_existing_collection(id: &str) -> DbEvidenceCollection {
    DbEvidenceCollection {
        id: id.to_string(),
        case_number: "CASE-EXISTING".to_string(),
        collection_date: "2026-04-13".to_string(),
        collection_location: "Existing Shelf".to_string(),
        collecting_officer: "Existing Officer".to_string(),
        authorization: "Existing Authorization".to_string(),
        authorization_date: None,
        authorizing_authority: None,
        witnesses_json: None,
        documentation_notes: None,
        conditions: None,
        status: "draft".to_string(),
        created_at: "2026-04-13T09:00:00Z".to_string(),
        modified_at: "2026-04-13T09:00:00Z".to_string(),
        item_count: 0,
    }
}

fn make_existing_item(id: &str, collection_id: &str) -> DbCollectedItem {
    DbCollectedItem {
        id: id.to_string(),
        collection_id: collection_id.to_string(),
        coc_item_id: None,
        evidence_file_id: None,
        item_number: "EXISTING-001".to_string(),
        description: "Existing item".to_string(),
        found_location: "Existing Shelf".to_string(),
        item_type: "Existing".to_string(),
        make: None,
        model: None,
        serial_number: None,
        condition: "Good".to_string(),
        packaging: "Box".to_string(),
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
    }
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
fn test_db_creation() {
    let (_dir, db) = create_test_db();
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.schema_version, SCHEMA_VERSION);
    assert_eq!(stats.total_activities, 0);
    assert_eq!(stats.total_users, 0);
}

#[test]
fn test_db_path_derivation() {
    let path = std::path::Path::new("/case/folder/myproject.cffx");
    let db_path = ProjectDatabase::db_path_for_project(path);
    assert_eq!(db_path, PathBuf::from("/case/folder/myproject.ffxdb"));
}

#[test]
fn test_activity_log() {
    let (_dir, db) = create_test_db();

    let entry = DbActivityEntry {
        id: "act_1".to_string(),
        timestamp: "2026-02-16T10:00:00Z".to_string(),
        user: "examiner1".to_string(),
        category: "hash".to_string(),
        action: "compute_hash".to_string(),
        description: "Computed SHA-256 for evidence.E01".to_string(),
        file_path: Some("/case/evidence.E01".to_string()),
        details: None,
    };

    db.insert_activity(&entry).unwrap();

    let results = db
        .query_activities(&ActivityQuery {
            category: Some("hash".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].action, "compute_hash");
}

#[test]
fn test_user_and_session() {
    let (_dir, db) = create_test_db();

    let user = DbProjectUser {
        username: "examiner1".to_string(),
        display_name: Some("Jane Doe".to_string()),
        hostname: Some("forensic-ws-01".to_string()),
        first_access: "2026-02-16T10:00:00Z".to_string(),
        last_access: "2026-02-16T10:00:00Z".to_string(),
    };
    db.upsert_user(&user).unwrap();

    let session = DbProjectSession {
        session_id: "sess_1".to_string(),
        user: "examiner1".to_string(),
        started_at: "2026-02-16T10:00:00Z".to_string(),
        ended_at: None,
        duration_seconds: None,
        hostname: Some("forensic-ws-01".to_string()),
        app_version: "0.1.0".to_string(),
        summary: None,
    };
    db.upsert_session(&session).unwrap();

    let users = db.get_users().unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].display_name, Some("Jane Doe".to_string()));

    let sessions = db.get_sessions().unwrap();
    assert_eq!(sessions.len(), 1);
}

#[test]
fn test_bookmarks_and_tags() {
    let (_dir, db) = create_test_db();

    let tag = DbTag {
        id: "tag_1".to_string(),
        name: "suspicious".to_string(),
        color: "#ff0000".to_string(),
        description: Some("Flagged for review".to_string()),
        created_at: "2026-02-16T10:00:00Z".to_string(),
    };
    db.upsert_tag(&tag).unwrap();

    let bookmark = DbBookmark {
        id: "bm_1".to_string(),
        target_type: "file".to_string(),
        target_path: "/case/evidence.E01".to_string(),
        name: "Suspicious E01".to_string(),
        created_by: "examiner1".to_string(),
        created_at: "2026-02-16T10:00:00Z".to_string(),
        color: None,
        notes: Some("Needs further analysis".to_string()),
        context: None,
    };
    db.upsert_bookmark(&bookmark).unwrap();

    let assignment = DbTagAssignment {
        tag_id: "tag_1".to_string(),
        target_type: "bookmark".to_string(),
        target_id: "bm_1".to_string(),
        assigned_at: "2026-02-16T10:00:00Z".to_string(),
        assigned_by: "examiner1".to_string(),
    };
    db.assign_tag(&assignment).unwrap();

    let tags = db.get_tags_for_target("bookmark", "bm_1").unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "suspicious");
}

#[test]
fn test_evidence_and_hashes() {
    let (_dir, db) = create_test_db();

    let file = DbEvidenceFile {
        id: "ev_1".to_string(),
        path: "/case/evidence.E01".to_string(),
        filename: "evidence.E01".to_string(),
        container_type: "e01".to_string(),
        total_size: 1_073_741_824,
        segment_count: 3,
        discovered_at: "2026-02-16T10:00:00Z".to_string(),
        created: None,
        modified: None,
    };
    db.upsert_evidence_file(&file).unwrap();

    let hash = DbProjectHash {
        id: "hash_1".to_string(),
        file_id: "ev_1".to_string(),
        algorithm: "SHA-256".to_string(),
        hash_value: "abc123def456".to_string(),
        computed_at: "2026-02-16T10:01:00Z".to_string(),
        segment_index: None,
        segment_name: None,
        source: "computed".to_string(),
    };
    db.insert_hash(&hash).unwrap();

    let result = db
        .lookup_hash_by_path("/case/evidence.E01", "SHA-256")
        .unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().0, "abc123def456");
}

#[test]
fn test_stats() {
    let (_dir, db) = create_test_db();

    let user = DbProjectUser {
        username: "test".to_string(),
        display_name: None,
        hostname: None,
        first_access: "2026-02-16T10:00:00Z".to_string(),
        last_access: "2026-02-16T10:00:00Z".to_string(),
    };
    db.upsert_user(&user).unwrap();

    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_users, 1);
    assert_eq!(stats.total_processed_databases, 0);
    assert_eq!(stats.total_axiom_cases, 0);
    assert_eq!(stats.total_artifact_categories, 0);
    assert!(stats.db_size_bytes > 0);
}

#[test]
fn test_processed_database_crud() {
    let (_dir, db) = create_test_db();

    // Insert a processed database
    let pdb = DbProcessedDatabase {
        id: "pdb_1".to_string(),
        path: "/case/2.Processed/AXIOM - Nov 15 2025".to_string(),
        name: "AXIOM - Nov 15 2025".to_string(),
        db_type: "MagnetAxiom".to_string(),
        case_number: Some("24-048".to_string()),
        examiner: Some("Jane Doe".to_string()),
        created_date: Some("2025-11-15T10:00:00Z".to_string()),
        total_size: 5_000_000_000,
        artifact_count: Some(12345),
        notes: None,
        registered_at: "2026-02-16T10:00:00Z".to_string(),
        metadata_json: None,
    };
    db.upsert_processed_database(&pdb).unwrap();

    // Verify retrieval
    let all = db.get_processed_databases().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].db_type, "MagnetAxiom");
    assert_eq!(all[0].case_number, Some("24-048".to_string()));

    // By path
    let found = db
        .get_processed_database_by_path("/case/2.Processed/AXIOM - Nov 15 2025")
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().artifact_count, Some(12345));

    // Insert integrity
    let integrity = DbProcessedDbIntegrity {
        id: "pdi_1".to_string(),
        processed_db_id: "pdb_1".to_string(),
        file_path: "/case/2.Processed/AXIOM/Case.mfdb".to_string(),
        file_size: 1_000_000,
        baseline_hash: "abc123".to_string(),
        baseline_timestamp: "2026-02-16T10:00:00Z".to_string(),
        current_hash: None,
        current_hash_timestamp: None,
        status: "unchanged".to_string(),
        changes_json: None,
    };
    db.upsert_processed_db_integrity(&integrity).unwrap();
    let integ = db.get_processed_db_integrity("pdb_1").unwrap();
    assert_eq!(integ.len(), 1);
    assert_eq!(integ[0].baseline_hash, "abc123");

    // Insert metrics
    let metrics = DbProcessedDbMetrics {
        id: "pdm_1".to_string(),
        processed_db_id: "pdb_1".to_string(),
        total_scans: 3,
        last_scan_date: Some("2025-11-15T10:00:00Z".to_string()),
        total_jobs: 5,
        last_job_date: Some("2025-11-16T10:00:00Z".to_string()),
        total_notes: 12,
        total_tagged_items: 47,
        total_users: 2,
        user_names_json: Some(r#"["Jane","John"]"#.to_string()),
        captured_at: "2026-02-16T10:00:00Z".to_string(),
    };
    db.upsert_processed_db_metrics(&metrics).unwrap();
    let m = db.get_processed_db_metrics("pdb_1").unwrap();
    assert!(m.is_some());
    assert_eq!(m.unwrap().total_tagged_items, 47);

    // Insert AXIOM case info
    let axiom = DbAxiomCaseInfo {
        id: "axc_1".to_string(),
        processed_db_id: "pdb_1".to_string(),
        case_name: "Wilson Investigation".to_string(),
        case_number: Some("24-048".to_string()),
        case_type: Some("Other".to_string()),
        description: None,
        examiner: Some("Jane Doe".to_string()),
        agency: Some("CORE Lab".to_string()),
        axiom_version: Some("7.5.0.0".to_string()),
        search_start: Some("2025-11-15T10:00:00Z".to_string()),
        search_end: Some("2025-11-15T16:00:00Z".to_string()),
        search_duration: Some("6h 0m".to_string()),
        search_outcome: Some("Completed".to_string()),
        output_folder: Some("/case/2.Processed/AXIOM".to_string()),
        total_artifacts: 12345,
        case_path: Some("/case/2.Processed/AXIOM".to_string()),
        captured_at: "2026-02-16T10:00:00Z".to_string(),
        keyword_info_json: None,
    };
    db.upsert_axiom_case_info(&axiom).unwrap();
    let a = db.get_axiom_case_info("pdb_1").unwrap();
    assert!(a.is_some());
    assert_eq!(a.unwrap().case_name, "Wilson Investigation");

    // Insert evidence source
    let src = DbAxiomEvidenceSource {
        id: "axs_1".to_string(),
        axiom_case_id: "axc_1".to_string(),
        name: "2020JimmyWilson.E01".to_string(),
        evidence_number: Some("EV-001".to_string()),
        source_type: "image".to_string(),
        path: Some("/case/1.Evidence/2020JimmyWilson.E01".to_string()),
        hash: Some("sha256:abcdef".to_string()),
        size: Some(50_000_000_000),
        acquired: Some("2025-01-01T00:00:00Z".to_string()),
        search_types_json: Some(r#"["full","keyword"]"#.to_string()),
    };
    db.insert_axiom_evidence_source(&src).unwrap();
    let sources = db.get_axiom_evidence_sources("axc_1").unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].name, "2020JimmyWilson.E01");

    // Insert search results
    let result = DbAxiomSearchResult {
        id: "axr_1".to_string(),
        axiom_case_id: "axc_1".to_string(),
        artifact_type: "Web Browser - Chrome - Web Visits".to_string(),
        hit_count: 1234,
    };
    db.insert_axiom_search_result(&result).unwrap();
    let results = db.get_axiom_search_results("axc_1").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].hit_count, 1234);

    // Insert artifact categories
    let cats = vec![
        DbArtifactCategory {
            id: "cat_1".to_string(),
            processed_db_id: "pdb_1".to_string(),
            category: "Web Related".to_string(),
            artifact_type: "Chrome Web Visits".to_string(),
            count: 500,
        },
        DbArtifactCategory {
            id: "cat_2".to_string(),
            processed_db_id: "pdb_1".to_string(),
            category: "Communication".to_string(),
            artifact_type: "Email Messages".to_string(),
            count: 200,
        },
    ];
    db.upsert_artifact_categories(&cats).unwrap();
    let fetched = db.get_artifact_categories("pdb_1").unwrap();
    assert_eq!(fetched.len(), 2);

    // Verify stats
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_processed_databases, 1);
    assert_eq!(stats.total_axiom_cases, 1);
    assert_eq!(stats.total_artifact_categories, 2);

    // Delete cascade
    db.delete_processed_database("pdb_1").unwrap();
    let all = db.get_processed_databases().unwrap();
    assert_eq!(all.len(), 0);
    let integ = db.get_processed_db_integrity("pdb_1").unwrap();
    assert_eq!(integ.len(), 0);
    let results = db.get_axiom_search_results("axc_1").unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_import_evidence_collection_package_remaps_links_and_preserves_valid_references() {
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

    let existing_collection = make_existing_collection("collection-src-1");
    db.upsert_evidence_collection(&existing_collection).unwrap();
    db.upsert_collected_item(&make_existing_item("item-src-1", &existing_collection.id))
        .unwrap();

    let summary = db
        .import_evidence_collection_package(&make_import_package())
        .unwrap();

    assert_eq!(summary.source_app, "CORE-EVD");
    assert_eq!(summary.source_case_number, "CASE-IMPORT-1");
    assert_eq!(summary.imported_collections, 1);
    assert_eq!(summary.imported_items, 2);
    assert_eq!(summary.imported_coc_items, 1);
    assert_eq!(summary.dropped_evidence_file_links, 1);
    assert_eq!(summary.dropped_coc_links, 1);

    let collections = db.get_evidence_collections(None).unwrap();
    assert_eq!(collections.len(), 2);

    let imported_collection = collections
        .iter()
        .find(|collection| {
            collection.id != existing_collection.id && collection.collection_location == "Scene A"
        })
        .expect("expected imported collection");
    assert_eq!(imported_collection.case_number, "CASE-IMPORT-1");
    assert_eq!(imported_collection.status, "complete");
    assert_eq!(imported_collection.item_count, 2);

    let imported_coc_items = db.get_coc_items(Some("CASE-IMPORT-1")).unwrap();
    assert_eq!(imported_coc_items.len(), 1);
    let imported_coc = &imported_coc_items[0];
    assert_ne!(imported_coc.id, "coc-src-1");
    assert_eq!(imported_coc.case_number, "CASE-IMPORT-1");
    assert_eq!(imported_coc.status, "locked");
    assert_eq!(imported_coc.evidence_file_id.as_deref(), Some("ev-existing"));

    let items = db.get_collected_items(&imported_collection.id).unwrap();
    assert_eq!(items.len(), 2);

    let preserved_item = items
        .iter()
        .find(|item| item.item_number == "ITEM-001")
        .expect("expected preserved linked item");
    assert_ne!(preserved_item.id, "item-src-1");
    assert_eq!(preserved_item.coc_item_id.as_deref(), Some(imported_coc.id.as_str()));
    assert_eq!(preserved_item.evidence_file_id.as_deref(), Some("ev-existing"));

    let dropped_item = items
        .iter()
        .find(|item| item.item_number == "ITEM-002")
        .expect("expected item with dropped links");
    assert_eq!(dropped_item.coc_item_id, None);
    assert_eq!(dropped_item.evidence_file_id, None);
    assert_eq!(dropped_item.description, "Imported item");
    assert_eq!(dropped_item.found_location, "Scene A");
    assert_eq!(dropped_item.item_type, "Evidence");
    assert_eq!(dropped_item.condition, "Unknown");
    assert_eq!(dropped_item.packaging, "Unknown");

    let transfers = db.get_coc_transfers(&imported_coc.id).unwrap();
    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].coc_item_id, imported_coc.id);

    let amendments = db.get_coc_amendments(&imported_coc.id).unwrap();
    assert_eq!(amendments.len(), 1);
    assert_eq!(amendments[0].coc_item_id, imported_coc.id);

    let audit_log = db.get_coc_audit_log(Some(&imported_coc.id)).unwrap();
    assert_eq!(audit_log.len(), 1);
    assert_eq!(audit_log[0].coc_item_id.as_deref(), Some(imported_coc.id.as_str()));
    assert_eq!(audit_log[0].action, "imported");
}
