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

fn table_exists(db: &ProjectDatabase, name: &str) -> bool {
    let conn = db.conn.lock();
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        [name],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count > 0)
    .unwrap_or(false)
}

fn hex_digest(ch: char, len: usize) -> String {
    std::iter::repeat_n(ch, len).collect()
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
        source_id: None,
        source_ref_json: None,
        item_number: "EXISTING-001".to_string(),
        description: "Existing item".to_string(),
        found_location: "Existing Shelf".to_string(),
        item_type: "Existing".to_string(),
        make: None,
        model: None,
        serial_number: None,
        condition: "Good".to_string(),
        packaging: "Box".to_string(),
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

    let whole_sha256 = hex_digest('a', 64);
    let segment_sha256 = hex_digest('b', 64);
    let segment_actual_sha256 = hex_digest('c', 64);
    let blake3_digest = hex_digest('d', 64);
    let source_sha512 = hex_digest('e', 128);

    let hash = DbProjectHash {
        id: "hash_1".to_string(),
        file_id: "ev_1".to_string(),
        source_id: None,
        source_ref_json: None,
        algorithm: "SHA-256".to_string(),
        hash_value: whole_sha256.clone(),
        computed_at: "2026-02-16T10:01:00Z".to_string(),
        segment_index: None,
        segment_name: None,
        source: "computed".to_string(),
    };
    db.insert_hash(&hash).unwrap();
    let segment_hash = DbProjectHash {
        id: "hash_2".to_string(),
        file_id: "ev_1".to_string(),
        source_id: None,
        source_ref_json: None,
        algorithm: "SHA-256".to_string(),
        hash_value: segment_sha256.clone(),
        computed_at: "2026-02-16T10:02:00Z".to_string(),
        segment_index: Some(1),
        segment_name: Some("evidence.E02".to_string()),
        source: "segment-computed".to_string(),
    };
    db.insert_hash(&segment_hash).unwrap();
    let blake3_hash = DbProjectHash {
        id: "hash_3".to_string(),
        file_id: "ev_1".to_string(),
        source_id: None,
        source_ref_json: None,
        algorithm: "BLAKE3".to_string(),
        hash_value: blake3_digest,
        computed_at: "2026-02-16T10:03:00Z".to_string(),
        segment_index: None,
        segment_name: None,
        source: "computed".to_string(),
    };
    db.insert_hash(&blake3_hash).unwrap();

    let result = db
        .lookup_hash_by_path("/case/evidence.E01", "SHA-256")
        .unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().0, whole_sha256);

    let summaries = db.summarize_hashes_by_algorithm().unwrap();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].algorithm, "BLAKE3");
    assert_eq!(summaries[0].count, 1);
    assert_eq!(summaries[0].evidence_file_count, 1);
    assert_eq!(summaries[0].source_count, 1);
    assert_eq!(
        summaries[0].latest_computed_at.as_deref(),
        Some("2026-02-16T10:03:00Z")
    );
    assert_eq!(summaries[1].algorithm, "SHA-256");
    assert_eq!(summaries[1].count, 2);
    assert_eq!(summaries[1].evidence_file_count, 1);
    assert_eq!(summaries[1].source_count, 2);
    assert_eq!(
        summaries[1].latest_computed_at.as_deref(),
        Some("2026-02-16T10:02:00Z")
    );

    let source_hash = DbProjectHash {
        id: "hash_4".to_string(),
        file_id: "ev_1".to_string(),
        source_id: Some("e01:/case/evidence.E01:/docs/report.pdf".to_string()),
        source_ref_json: Some(
            r#"{"kind":"containerEntry","containerPath":"/case/evidence.E01","entryPath":"/docs/report.pdf","containerType":"e01"}"#.to_string(),
        ),
        algorithm: "SHA-512".to_string(),
        hash_value: source_sha512.clone(),
        computed_at: "2026-02-16T10:03:30Z".to_string(),
        segment_index: None,
        segment_name: None,
        source: "source-computed".to_string(),
    };
    db.insert_hash(&source_hash).unwrap();

    let source_hashes = db
        .get_hashes_for_source("e01:/case/evidence.E01:/docs/report.pdf")
        .unwrap();
    assert_eq!(source_hashes.len(), 1);
    assert_eq!(source_hashes[0].hash_value, source_sha512);
    assert!(source_hashes[0]
        .source_ref_json
        .as_deref()
        .unwrap()
        .contains("containerEntry"));

    let latest_source_hash = db
        .get_latest_hash_for_source("e01:/case/evidence.E01:/docs/report.pdf", "SHA-512")
        .unwrap()
        .unwrap();
    assert_eq!(latest_source_hash.id, "hash_4");

    let source_hash_map = db.latest_source_hash_map("SHA-512").unwrap();
    assert_eq!(
        source_hash_map.get("e01:/case/evidence.E01:/docs/report.pdf"),
        Some(&source_sha512)
    );

    let latest_file_hash = db.get_latest_hash("ev_1", "SHA-512").unwrap();
    assert!(latest_file_hash.is_none());

    let verification_ok = DbProjectVerification {
        id: "verification_1".to_string(),
        hash_id: "hash_1".to_string(),
        verified_at: "2026-02-16T10:04:00Z".to_string(),
        result: "match".to_string(),
        expected_hash: whole_sha256.clone(),
        actual_hash: whole_sha256.clone(),
    };
    db.insert_verification(&verification_ok).unwrap();
    let verification_fail = DbProjectVerification {
        id: "verification_2".to_string(),
        hash_id: "hash_2".to_string(),
        verified_at: "2026-02-16T10:05:00Z".to_string(),
        result: "mismatch".to_string(),
        expected_hash: segment_sha256,
        actual_hash: segment_actual_sha256,
    };
    db.insert_verification(&verification_fail).unwrap();
    let verification_ok_repeat = DbProjectVerification {
        id: "verification_3".to_string(),
        hash_id: "hash_1".to_string(),
        verified_at: "2026-02-16T10:06:00Z".to_string(),
        result: "match".to_string(),
        expected_hash: whole_sha256.clone(),
        actual_hash: whole_sha256,
    };
    db.insert_verification(&verification_ok_repeat).unwrap();

    let verifications = db.get_verifications_for_hash("hash_1").unwrap();
    assert_eq!(verifications.len(), 2);
    assert_eq!(verifications[0].id, "verification_3");

    let verification_summaries = db.summarize_verifications_by_result().unwrap();
    assert_eq!(verification_summaries.len(), 2);
    assert_eq!(verification_summaries[0].result, "match");
    assert_eq!(verification_summaries[0].count, 2);
    assert_eq!(verification_summaries[0].hash_count, 1);
    assert_eq!(
        verification_summaries[0].latest_verified_at.as_deref(),
        Some("2026-02-16T10:06:00Z")
    );
    assert_eq!(verification_summaries[1].result, "mismatch");
    assert_eq!(verification_summaries[1].count, 1);
    assert_eq!(verification_summaries[1].hash_count, 1);
    assert_eq!(
        verification_summaries[1].latest_verified_at.as_deref(),
        Some("2026-02-16T10:05:00Z")
    );
}

#[test]
fn test_evidence_upsert_rejects_invalid_persistence_payloads() {
    let (_dir, db) = create_test_db();

    let mut file = DbEvidenceFile {
        id: "ev_validation".to_string(),
        path: "/case/validation.E01".to_string(),
        filename: "validation.E01".to_string(),
        container_type: "e01".to_string(),
        total_size: 1024,
        segment_count: 1,
        discovered_at: "2026-02-16T10:00:00Z".to_string(),
        created: None,
        modified: None,
    };

    file.path = "   ".to_string();
    let err = db.upsert_evidence_file(&file).unwrap_err();
    assert!(err.to_string().contains("Evidence path cannot be blank"));

    file.path = "/case/validation.E01".to_string();
    file.total_size = -1;
    let err = db.upsert_evidence_file(&file).unwrap_err();
    assert!(err
        .to_string()
        .contains("Evidence total_size cannot be negative"));

    file.total_size = 1024;
    file.segment_count = 0;
    let err = db.upsert_evidence_file(&file).unwrap_err();
    assert!(err
        .to_string()
        .contains("Evidence segment_count must be at least 1"));

    file.segment_count = 1;
    file.created = Some(" ".to_string());
    let err = db.upsert_evidence_file(&file).unwrap_err();
    assert!(err.to_string().contains("Evidence created cannot be blank"));

    file.created = None;
    file.filename = "a".repeat((16 * 1024) + 1);
    let err = db.upsert_evidence_file(&file).unwrap_err();
    assert!(err.to_string().contains("Evidence filename exceeds"));

    file.filename = "validation.E01".to_string();
    db.upsert_evidence_file(&file).unwrap();
}

#[test]
fn test_batch_evidence_upsert_rejects_invalid_payload_before_writes() {
    let (_dir, db) = create_test_db();
    let valid = DbEvidenceFile {
        id: "ev_batch_valid".to_string(),
        path: "/case/batch-valid.E01".to_string(),
        filename: "batch-valid.E01".to_string(),
        container_type: "e01".to_string(),
        total_size: 1024,
        segment_count: 1,
        discovered_at: "2026-02-16T10:00:00Z".to_string(),
        created: None,
        modified: None,
    };
    let mut invalid = valid.clone();
    invalid.id = "ev_batch_invalid".to_string();
    invalid.path = "/case/batch-invalid.E01".to_string();
    invalid.segment_count = 0;

    let err = db
        .batch_upsert_evidence_files(&[valid.clone(), invalid])
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Evidence segment_count must be at least 1"));

    assert!(db
        .get_evidence_file_by_path("/case/batch-valid.E01")
        .unwrap()
        .is_none());

    assert_eq!(db.batch_upsert_evidence_files(&[valid]).unwrap(), 1);
}

#[test]
fn test_evidence_file_listing_applies_bounded_limit() {
    let (_dir, db) = create_test_db();

    let files: Vec<_> = (0..3)
        .map(|index| DbEvidenceFile {
            id: format!("ev_limit_{index}"),
            path: format!("/case/limit-{index}.E01"),
            filename: format!("limit-{index}.E01"),
            container_type: "e01".to_string(),
            total_size: 1024,
            segment_count: 1,
            discovered_at: "2026-02-16T10:00:00Z".to_string(),
            created: None,
            modified: None,
        })
        .collect();
    assert_eq!(db.batch_upsert_evidence_files(&files).unwrap(), 3);

    let limited = db.get_evidence_files_limited(Some(2)).unwrap();
    assert_eq!(limited.len(), 2);
    assert_eq!(limited[0].filename, "limit-0.E01");
    assert_eq!(limited[1].filename, "limit-1.E01");
}

#[test]
fn test_hash_upsert_rejects_invalid_persistence_payloads() {
    let (_dir, db) = create_test_db();

    let file = DbEvidenceFile {
        id: "ev_hash_validation".to_string(),
        path: "/case/hash-validation.E01".to_string(),
        filename: "hash-validation.E01".to_string(),
        container_type: "e01".to_string(),
        total_size: 1024,
        segment_count: 1,
        discovered_at: "2026-02-16T10:00:00Z".to_string(),
        created: None,
        modified: None,
    };
    db.upsert_evidence_file(&file).unwrap();

    let mut hash = DbProjectHash {
        id: "hash_validation".to_string(),
        file_id: file.id.clone(),
        source_id: Some("/case/hash-validation.E01".to_string()),
        source_ref_json: Some(
            r#"{"kind":"localFile","path":"/case/hash-validation.E01"}"#.to_string(),
        ),
        algorithm: "SHA-256".to_string(),
        hash_value: hex_digest('a', 64),
        computed_at: "2026-02-16T10:01:00Z".to_string(),
        segment_index: None,
        segment_name: None,
        source: "computed".to_string(),
    };

    hash.id = "   ".to_string();
    let err = db.insert_hash(&hash).unwrap_err();
    assert!(err.to_string().contains("Hash id cannot be blank"));

    hash.id = "hash_validation".to_string();
    hash.algorithm = "sha256".to_string();
    let err = db.insert_hash(&hash).unwrap_err();
    assert!(err
        .to_string()
        .contains("Hash algorithm must use canonical name SHA-256"));

    hash.algorithm = "ROT13".to_string();
    let err = db.insert_hash(&hash).unwrap_err();
    assert!(err.to_string().contains("Invalid hash algorithm"));

    hash.algorithm = "SHA-256".to_string();
    hash.hash_value = "not-a-digest".to_string();
    let err = db.insert_hash(&hash).unwrap_err();
    assert!(err
        .to_string()
        .contains("Hash value is not a valid SHA-256 digest"));

    hash.hash_value = hex_digest('a', 64);
    hash.source_ref_json = Some("{not-json".to_string());
    let err = db.insert_hash(&hash).unwrap_err();
    assert!(err.to_string().contains("Invalid hash source_ref_json"));

    hash.source_ref_json =
        Some(r#"{"kind":"localFile","path":"/case/hash-validation.E01"}"#.to_string());
    hash.source_id = Some("   ".to_string());
    let err = db.insert_hash(&hash).unwrap_err();
    assert!(err.to_string().contains("Hash source_id cannot be blank"));

    hash.source_id = Some("/case/hash-validation.E01".to_string());
    hash.segment_index = Some(-1);
    let err = db.insert_hash(&hash).unwrap_err();
    assert!(err
        .to_string()
        .contains("Hash segment_index cannot be negative"));

    hash.segment_index = None;
    db.insert_hash(&hash).unwrap();

    let mut verification = DbProjectVerification {
        id: "verification_validation".to_string(),
        hash_id: hash.id.clone(),
        verified_at: "2026-02-16T10:02:00Z".to_string(),
        result: "match".to_string(),
        expected_hash: hex_digest('a', 64),
        actual_hash: hex_digest('a', 64),
    };

    verification.expected_hash = "not-a-digest".to_string();
    let err = db.insert_verification(&verification).unwrap_err();
    assert!(err
        .to_string()
        .contains("Verification expected_hash is not a valid SHA-256 digest"));

    verification.expected_hash = hex_digest('a', 64);
    verification.actual_hash = hex_digest('b', 32);
    let err = db.insert_verification(&verification).unwrap_err();
    assert!(err
        .to_string()
        .contains("Verification actual_hash is not a valid SHA-256 digest"));

    verification.actual_hash = hex_digest('a', 64);
    verification.hash_id = "missing_hash".to_string();
    let err = db.insert_verification(&verification).unwrap_err();
    assert!(err
        .to_string()
        .contains("Verification references unknown hash"));

    verification.hash_id = hash.id.clone();
    db.insert_verification(&verification).unwrap();
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
    assert_eq!(stats.total_artifacts, 0);
    assert_eq!(stats.total_source_analyses, 0);
    assert!(stats.db_size_bytes > 0);
}

#[test]
fn test_annotation_crud_and_fts() {
    let (_dir, db) = create_test_db();

    let annotation = DbAnnotation {
        id: "ann_hex_1".to_string(),
        file_path: "ad1:/case/logical.ad1:/docs/report.pdf".to_string(),
        container_path: Some("/case/logical.ad1".to_string()),
        annotation_type: "hex-magic".to_string(),
        offset_start: Some(0),
        offset_end: Some(16),
        line_start: None,
        line_end: None,
        label: "Magic Bytes".to_string(),
        content: Some("Initial signature bytes: 25 50 44 46".to_string()),
        color: Some("#38bdf8".to_string()),
        created_by: "hex-viewer".to_string(),
        created_at: "2026-02-16T10:05:00Z".to_string(),
        modified_at: "2026-02-16T10:05:00Z".to_string(),
    };

    db.insert_annotation(&annotation).unwrap();

    let by_path = db
        .get_annotations_for_path("ad1:/case/logical.ad1:/docs/report.pdf")
        .unwrap();
    assert_eq!(by_path.len(), 1);
    assert_eq!(by_path[0].annotation_type, "hex-magic");
    assert_eq!(by_path[0].offset_start, Some(0));

    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_annotations, 1);

    if table_exists(&db, "fts_annotations") {
        db.rebuild_fts_indexes().unwrap();
        let fts_results = db.fts_search("signature", Some(10)).unwrap();
        let annotation_result = fts_results
            .iter()
            .find(|result| result.source == "annotations")
            .expect("annotation FTS result should be returned");

        assert_eq!(
            annotation_result.id,
            "ad1:/case/logical.ad1:/docs/report.pdf"
        );
        assert!(annotation_result.snippet.contains("<mark>signature</mark>"));
    }
}

#[test]
fn test_annotation_upsert_rejects_invalid_persistence_payloads() {
    let (_dir, db) = create_test_db();

    let mut annotation = DbAnnotation {
        id: "ann_validation".to_string(),
        file_path: "ad1:/case/logical.ad1:/docs/report.pdf".to_string(),
        container_path: Some("/case/logical.ad1".to_string()),
        annotation_type: "hex-magic".to_string(),
        offset_start: Some(0),
        offset_end: Some(16),
        line_start: None,
        line_end: None,
        label: "Magic Bytes".to_string(),
        content: Some("Initial signature bytes: 25 50 44 46".to_string()),
        color: Some("#38bdf8".to_string()),
        created_by: "hex-viewer".to_string(),
        created_at: "2026-02-16T10:05:00Z".to_string(),
        modified_at: "2026-02-16T10:05:00Z".to_string(),
    };

    annotation.file_path = "   ".to_string();
    let err = db.insert_annotation(&annotation).unwrap_err();
    assert!(err
        .to_string()
        .contains("Annotation file_path cannot be blank"));

    annotation.file_path = "ad1:/case/logical.ad1:/docs/report.pdf".to_string();
    annotation.offset_start = Some(16);
    annotation.offset_end = Some(8);
    let err = db.insert_annotation(&annotation).unwrap_err();
    assert!(err
        .to_string()
        .contains("Annotation offset range end cannot be before start"));

    annotation.offset_start = Some(-1);
    annotation.offset_end = Some(8);
    let err = db.insert_annotation(&annotation).unwrap_err();
    assert!(err
        .to_string()
        .contains("Annotation offset range cannot be negative"));

    annotation.offset_start = Some(0);
    annotation.offset_end = None;
    let err = db.insert_annotation(&annotation).unwrap_err();
    assert!(err
        .to_string()
        .contains("Annotation offset range requires both start and end"));

    annotation.offset_end = Some(16);
    annotation.label = " ".to_string();
    let err = db.insert_annotation(&annotation).unwrap_err();
    assert!(err.to_string().contains("Annotation label cannot be blank"));

    annotation.label = "Magic Bytes".to_string();
    annotation.content = Some("a".repeat((1024 * 1024) + 1));
    let err = db.insert_annotation(&annotation).unwrap_err();
    assert!(err.to_string().contains("Annotation content exceeds"));

    annotation.content = Some("Initial signature bytes: 25 50 44 46".to_string());
    db.insert_annotation(&annotation).unwrap();

    annotation.label = "Updated Magic Bytes".to_string();
    annotation.modified_at = "   ".to_string();
    let err = db.update_annotation(&annotation).unwrap_err();
    assert!(err
        .to_string()
        .contains("Annotation modified_at cannot be blank"));

    annotation.modified_at = "2026-02-16T10:06:00Z".to_string();
    db.update_annotation(&annotation).unwrap();
    let fetched = db
        .get_annotations_for_path("ad1:/case/logical.ad1:/docs/report.pdf")
        .unwrap();
    assert_eq!(fetched[0].label, "Updated Magic Bytes");
}

#[test]
fn test_annotation_listing_applies_bounded_limit() {
    let (_dir, db) = create_test_db();

    for index in 0..3 {
        db.insert_annotation(&DbAnnotation {
            id: format!("ann_limit_{index}"),
            file_path: format!("/case/file-{index}.bin"),
            container_path: None,
            annotation_type: "hex-review".to_string(),
            offset_start: Some(0),
            offset_end: Some(16),
            line_start: None,
            line_end: None,
            label: "Magic Bytes".to_string(),
            content: Some("Initial signature bytes".to_string()),
            color: Some("#38bdf8".to_string()),
            created_by: "hex-viewer".to_string(),
            created_at: format!("2026-02-16T10:0{index}:00Z"),
            modified_at: format!("2026-02-16T10:0{index}:00Z"),
        })
        .unwrap();
    }

    let limited = db.get_all_annotations_limited(Some(2)).unwrap();
    assert_eq!(limited.len(), 2);
    assert_eq!(limited[0].id, "ann_limit_2");
    assert_eq!(limited[1].id, "ann_limit_1");
}

#[test]
fn test_normalized_artifact_crud() {
    let (_dir, db) = create_test_db();

    let file = DbEvidenceFile {
        id: "ev_artifact".to_string(),
        path: "/case/evidence/logical.ad1".to_string(),
        filename: "logical.ad1".to_string(),
        container_type: "ad1".to_string(),
        total_size: 2048,
        segment_count: 1,
        discovered_at: "2026-02-16T10:00:00Z".to_string(),
        created: None,
        modified: None,
    };
    db.upsert_evidence_file(&file).unwrap();

    let artifact = DbNormalizedArtifact {
        id: "artifact_1".to_string(),
        evidence_file_id: Some("ev_artifact".to_string()),
        source_id: "/case/evidence/logical.ad1:/docs/a.txt".to_string(),
        source_ref_json:
            r#"{"kind":"containerEntry","containerPath":"/case/evidence/logical.ad1","entryPath":"/docs/a.txt","containerType":"ad1"}"#
                .to_string(),
        name: "a.txt".to_string(),
        extension: Some("txt".to_string()),
        size: 42,
        mime_type: Some("text/plain".to_string()),
        type_description: "Plain Text".to_string(),
        category: "text".to_string(),
        confidence: "high".to_string(),
        is_text: true,
        content_preview: Some("artifact text".to_string()),
        metadata_json: Some(r#"{"sourceId":"x"}"#.to_string()),
        extracted_at: "2026-02-16T10:01:00Z".to_string(),
        extractor: "test".to_string(),
    };

    db.upsert_artifact(&artifact).unwrap();
    let image_artifact = DbNormalizedArtifact {
        id: "artifact_2".to_string(),
        evidence_file_id: Some("ev_artifact".to_string()),
        source_id: "/case/evidence/logical.ad1:/media/photo.png".to_string(),
        source_ref_json:
            r#"{"kind":"containerEntry","containerPath":"/case/evidence/logical.ad1","entryPath":"/media/photo.png","containerType":"ad1"}"#
                .to_string(),
        name: "photo.png".to_string(),
        extension: Some("png".to_string()),
        size: 2048,
        mime_type: Some("image/png".to_string()),
        type_description: "PNG Image".to_string(),
        category: "image".to_string(),
        confidence: "high".to_string(),
        is_text: false,
        content_preview: None,
        metadata_json: Some(r#"{"image.dimensions":"640x480"}"#.to_string()),
        extracted_at: "2026-02-16T10:02:00Z".to_string(),
        extractor: "test".to_string(),
    };
    db.upsert_artifact(&image_artifact).unwrap();
    let orphan_artifact = DbNormalizedArtifact {
        id: "artifact_3".to_string(),
        evidence_file_id: None,
        source_id: "/case/loose/orphan.log".to_string(),
        source_ref_json: r#"{"kind":"localFile","path":"/case/loose/orphan.log"}"#.to_string(),
        name: "orphan.log".to_string(),
        extension: Some("log".to_string()),
        size: 100,
        mime_type: Some("text/plain".to_string()),
        type_description: "Log File".to_string(),
        category: "text".to_string(),
        confidence: "medium".to_string(),
        is_text: true,
        content_preview: Some("loose artifact".to_string()),
        metadata_json: Some(r#"{"sourceId":"/case/loose/orphan.log"}"#.to_string()),
        extracted_at: "2026-02-16T10:03:00Z".to_string(),
        extractor: "manual-import".to_string(),
    };
    db.upsert_artifact(&orphan_artifact).unwrap();

    let fetched = db.get_artifact("artifact_1").unwrap().unwrap();
    assert_eq!(fetched.name, "a.txt");
    assert_eq!(fetched.category, "text");
    assert!(fetched.is_text);

    let by_evidence = db.list_artifacts_for_evidence("ev_artifact").unwrap();
    assert_eq!(by_evidence.len(), 2);
    assert_eq!(by_evidence[0].id, "artifact_1");

    let all_artifacts = db.list_artifacts(None).unwrap();
    assert_eq!(all_artifacts.len(), 3);
    assert_eq!(all_artifacts[0].id, "artifact_3");
    assert_eq!(all_artifacts[1].id, "artifact_2");
    assert_eq!(all_artifacts[2].id, "artifact_1");

    let by_category = db.list_artifacts_by_category("text", Some(10)).unwrap();
    assert_eq!(by_category.len(), 2);
    assert_eq!(by_category[0].source_id, "/case/loose/orphan.log");
    assert_eq!(
        by_category[1].source_id,
        "/case/evidence/logical.ad1:/docs/a.txt"
    );

    let summaries = db.summarize_artifacts_by_category().unwrap();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].category, "image");
    assert_eq!(summaries[0].count, 1);
    assert_eq!(summaries[0].total_size, 2048);
    assert_eq!(summaries[0].text_count, 0);
    assert_eq!(
        summaries[0].latest_extracted_at.as_deref(),
        Some("2026-02-16T10:02:00Z")
    );
    assert_eq!(summaries[1].category, "text");
    assert_eq!(summaries[1].count, 2);
    assert_eq!(summaries[1].total_size, 142);
    assert_eq!(summaries[1].text_count, 2);

    let evidence_summaries = db.summarize_artifacts_by_evidence().unwrap();
    assert_eq!(evidence_summaries.len(), 2);
    assert_eq!(
        evidence_summaries[0].evidence_file_id.as_deref(),
        Some("ev_artifact")
    );
    assert_eq!(evidence_summaries[0].count, 2);
    assert_eq!(evidence_summaries[0].total_size, 2090);
    assert_eq!(evidence_summaries[0].text_count, 1);
    assert_eq!(evidence_summaries[0].category_count, 2);
    assert_eq!(
        evidence_summaries[0].latest_extracted_at.as_deref(),
        Some("2026-02-16T10:02:00Z")
    );
    assert_eq!(evidence_summaries[1].evidence_file_id, None);
    assert_eq!(evidence_summaries[1].count, 1);
    assert_eq!(evidence_summaries[1].total_size, 100);
    assert_eq!(evidence_summaries[1].text_count, 1);
    assert_eq!(evidence_summaries[1].category_count, 1);

    let extractor_summaries = db.summarize_artifacts_by_extractor().unwrap();
    assert_eq!(extractor_summaries.len(), 2);
    assert_eq!(extractor_summaries[0].extractor, "manual-import");
    assert_eq!(extractor_summaries[0].count, 1);
    assert_eq!(extractor_summaries[0].total_size, 100);
    assert_eq!(extractor_summaries[0].text_count, 1);
    assert_eq!(extractor_summaries[0].category_count, 1);
    assert_eq!(extractor_summaries[0].evidence_file_count, 0);
    assert_eq!(
        extractor_summaries[0].latest_extracted_at.as_deref(),
        Some("2026-02-16T10:03:00Z")
    );
    assert_eq!(extractor_summaries[1].extractor, "test");
    assert_eq!(extractor_summaries[1].count, 2);
    assert_eq!(extractor_summaries[1].total_size, 2090);
    assert_eq!(extractor_summaries[1].text_count, 1);
    assert_eq!(extractor_summaries[1].category_count, 2);
    assert_eq!(extractor_summaries[1].evidence_file_count, 1);

    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_artifacts, 3);
    assert_eq!(stats.total_source_analyses, 0);

    if table_exists(&db, "fts_artifacts") {
        db.rebuild_fts_indexes().unwrap();
        let fts_results = db.fts_search("dimensions", Some(10)).unwrap();
        let artifact_result = fts_results
            .iter()
            .find(|result| result.source == "artifacts")
            .expect("artifact FTS result should be returned");

        assert_eq!(
            artifact_result.id,
            "/case/evidence/logical.ad1:/media/photo.png"
        );
        assert!(artifact_result.snippet.contains("<mark>dimensions</mark>"));
    }
}

#[test]
fn test_artifact_summary_totals_saturate_on_overflow() {
    let (_dir, db) = create_test_db();

    for index in 0..2 {
        let artifact = DbNormalizedArtifact {
            id: format!("huge_artifact_{index}"),
            evidence_file_id: None,
            source_id: format!("/case/huge-{index}.bin"),
            source_ref_json: format!(r#"{{"kind":"localFile","path":"/case/huge-{index}.bin"}}"#),
            name: format!("huge-{index}.bin"),
            extension: Some("bin".to_string()),
            size: i64::MAX,
            mime_type: Some("application/octet-stream".to_string()),
            type_description: "Binary Data".to_string(),
            category: "binary".to_string(),
            confidence: "medium".to_string(),
            is_text: false,
            content_preview: None,
            metadata_json: None,
            extracted_at: format!("2026-02-16T10:0{index}:00Z"),
            extractor: "overflow-test".to_string(),
        };
        db.upsert_artifact(&artifact).unwrap();
    }

    let category = db.summarize_artifacts_by_category().unwrap();
    assert_eq!(category.len(), 1);
    assert_eq!(category[0].count, 2);
    assert_eq!(category[0].total_size, i64::MAX);

    let evidence = db.summarize_artifacts_by_evidence().unwrap();
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].count, 2);
    assert_eq!(evidence[0].total_size, i64::MAX);

    let extractor = db.summarize_artifacts_by_extractor().unwrap();
    assert_eq!(extractor.len(), 1);
    assert_eq!(extractor[0].count, 2);
    assert_eq!(extractor[0].total_size, i64::MAX);
}

#[test]
fn test_artifact_upsert_rejects_invalid_persistence_payloads() {
    let (_dir, db) = create_test_db();

    let mut artifact = DbNormalizedArtifact {
        id: "bad_artifact".to_string(),
        evidence_file_id: None,
        source_id: "/case/bad.bin".to_string(),
        source_ref_json: r#"{"kind":"localFile","path":"/case/bad.bin"}"#.to_string(),
        name: "bad.bin".to_string(),
        extension: Some("bin".to_string()),
        size: 1,
        mime_type: Some("application/octet-stream".to_string()),
        type_description: "Binary Data".to_string(),
        category: "binary".to_string(),
        confidence: "medium".to_string(),
        is_text: false,
        content_preview: None,
        metadata_json: Some(r#"{"safe":true}"#.to_string()),
        extracted_at: "2026-02-16T10:00:00Z".to_string(),
        extractor: "validation-test".to_string(),
    };

    artifact.size = -1;
    let err = db.upsert_artifact(&artifact).unwrap_err();
    assert!(err.to_string().contains("Artifact size cannot be negative"));

    artifact.size = 1;
    artifact.source_ref_json = "{not-json".to_string();
    let err = db.upsert_artifact(&artifact).unwrap_err();
    assert!(err.to_string().contains("Invalid artifact source_ref_json"));

    artifact.source_ref_json = r#"{"kind":"localFile","path":"/case/bad.bin"}"#.to_string();
    artifact.metadata_json = Some("{not-json".to_string());
    let err = db.upsert_artifact(&artifact).unwrap_err();
    assert!(err.to_string().contains("Invalid artifact metadata_json"));

    artifact.metadata_json = Some(r#"{"safe":true}"#.to_string());
    artifact.name = "   ".to_string();
    let err = db.upsert_artifact(&artifact).unwrap_err();
    assert!(err.to_string().contains("Artifact name cannot be blank"));

    artifact.name = "n".repeat((16 * 1024) + 1);
    let err = db.upsert_artifact(&artifact).unwrap_err();
    assert!(err.to_string().contains("Artifact name exceeds"));

    artifact.name = "bad.bin".to_string();
    artifact.evidence_file_id = Some("   ".to_string());
    let err = db.upsert_artifact(&artifact).unwrap_err();
    assert!(err
        .to_string()
        .contains("Artifact evidence_file_id cannot be blank"));

    artifact.evidence_file_id = None;
    artifact.source_id = "\t".to_string();
    let err = db.upsert_artifact(&artifact).unwrap_err();
    assert!(err
        .to_string()
        .contains("Artifact source_id cannot be blank"));
}

#[test]
fn test_source_analysis_crud() {
    let (_dir, db) = create_test_db();

    let file = DbEvidenceFile {
        id: "ev_analysis".to_string(),
        path: "/case/evidence/disk.E01".to_string(),
        filename: "disk.E01".to_string(),
        container_type: "e01".to_string(),
        total_size: 4096,
        segment_count: 1,
        discovered_at: "2026-02-16T10:00:00Z".to_string(),
        created: None,
        modified: None,
    };
    db.upsert_evidence_file(&file).unwrap();

    let record = DbSourceAnalysisRecord {
        id: "analysis_1".to_string(),
        evidence_file_id: Some("ev_analysis".to_string()),
        source_id: "/case/evidence/disk.E01:/docs/report.pdf".to_string(),
        source_ref_json:
            r#"{"kind":"vfsEntry","containerPath":"/case/evidence/disk.E01","entryPath":"/docs/report.pdf","containerType":"e01"}"#
                .to_string(),
        total_size: 2048,
        offset: 0,
        bytes_analyzed: 512,
        magic_hex: "25 50 44 46".to_string(),
        signature_count: 1,
        primary_signature: Some("PDF Document".to_string()),
        primary_mime_type: Some("application/pdf".to_string()),
        primary_category: Some("document".to_string()),
        entropy: 4.5,
        printable_ratio: 0.75,
        is_likely_text: true,
        ascii_preview: Some("%PDF".to_string()),
        signatures_json: Some(r#"[{"description":"PDF Document"}]"#.to_string()),
        entropy_windows_json: Some("[]".to_string()),
        histogram_json: Some("[0,1]".to_string()),
        indicators_json: Some(r#"[{"indicatorType":"email","value":"admin@example.com","offset":42,"length":17,"confidence":"medium"}]"#.to_string()),
        analyzed_at: "2026-02-16T10:01:00Z".to_string(),
        analyzer: "test-analyzer".to_string(),
    };
    db.upsert_source_analysis(&record).unwrap();

    let fetched = db.get_source_analysis("analysis_1").unwrap().unwrap();
    assert_eq!(
        fetched.source_id,
        "/case/evidence/disk.E01:/docs/report.pdf"
    );
    assert_eq!(fetched.primary_category.as_deref(), Some("document"));
    assert!(fetched
        .indicators_json
        .as_deref()
        .is_some_and(|json| json.contains("admin@example.com")));
    assert!(fetched.is_likely_text);

    let all = db.list_source_analyses(None).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, "analysis_1");

    let summaries = db.summarize_source_analyses_by_category().unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].category, "document");
    assert_eq!(summaries[0].count, 1);
    assert_eq!(summaries[0].evidence_file_count, 1);
    assert_eq!(summaries[0].text_like_count, 1);
    assert_eq!(
        summaries[0].latest_analyzed_at.as_deref(),
        Some("2026-02-16T10:01:00Z")
    );

    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_source_analyses, 1);

    if table_exists(&db, "fts_source_analyses") {
        db.rebuild_fts_indexes().unwrap();
        let fts_results = db.fts_search("application", Some(10)).unwrap();
        let source_result = fts_results
            .iter()
            .find(|result| result.source == "source_analysis")
            .expect("source-analysis FTS result should be returned");

        assert_eq!(source_result.id, "/case/evidence/disk.E01:/docs/report.pdf");
        assert!(source_result.snippet.contains("<mark>application</mark>"));

        let indicator_results = db.fts_search("admin", Some(10)).unwrap();
        let indicator_result = indicator_results
            .iter()
            .find(|result| result.source == "source_analysis")
            .expect("source-analysis indicator FTS result should be returned");
        assert_eq!(
            indicator_result.id,
            "/case/evidence/disk.E01:/docs/report.pdf"
        );
    }
}

#[test]
fn test_source_analysis_upsert_rejects_invalid_persistence_payloads() {
    let (_dir, db) = create_test_db();

    let mut record = DbSourceAnalysisRecord {
        id: "bad_analysis".to_string(),
        evidence_file_id: None,
        source_id: "/case/bad.bin".to_string(),
        source_ref_json: r#"{"kind":"localFile","path":"/case/bad.bin"}"#.to_string(),
        total_size: 10,
        offset: 0,
        bytes_analyzed: 4,
        magic_hex: "00 01 02 03".to_string(),
        signature_count: 0,
        primary_signature: None,
        primary_mime_type: None,
        primary_category: None,
        entropy: 1.5,
        printable_ratio: 0.25,
        is_likely_text: false,
        ascii_preview: None,
        signatures_json: Some("[]".to_string()),
        entropy_windows_json: Some("[]".to_string()),
        histogram_json: Some("[0,1]".to_string()),
        indicators_json: Some("[]".to_string()),
        analyzed_at: "2026-02-16T10:00:00Z".to_string(),
        analyzer: "validation-test".to_string(),
    };

    record.id = "   ".to_string();
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err
        .to_string()
        .contains("Source analysis id cannot be blank"));

    record.id = "bad_analysis".to_string();
    record.source_ref_json = "{not-json".to_string();
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid source analysis source_ref_json"));

    record.source_ref_json = r#"{"kind":"localFile","path":"/case/bad.bin"}"#.to_string();
    record.total_size = -1;
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err
        .to_string()
        .contains("Source analysis total_size cannot be negative"));

    record.total_size = 10;
    record.offset = 9;
    record.bytes_analyzed = 2;
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err
        .to_string()
        .contains("Source analysis byte range exceeds source size"));

    record.offset = 0;
    record.bytes_analyzed = 4;
    record.entropy = 9.0;
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err
        .to_string()
        .contains("Source analysis entropy must be finite"));

    record.entropy = 1.5;
    record.printable_ratio = -0.1;
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err
        .to_string()
        .contains("Source analysis printable_ratio must be finite"));

    record.printable_ratio = 0.25;
    record.indicators_json = Some("{not-json".to_string());
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid source analysis indicators_json"));

    record.indicators_json = Some("[]".to_string());
    record.primary_category = Some("   ".to_string());
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err
        .to_string()
        .contains("Source analysis primary_category cannot be blank"));

    record.primary_category = Some("binary".to_string());
    record.analyzer = "a".repeat((16 * 1024) + 1);
    let err = db.upsert_source_analysis(&record).unwrap_err();
    assert!(err.to_string().contains("Source analysis analyzer exceeds"));

    record.analyzer = "validation-test".to_string();
    db.upsert_source_analysis(&record).unwrap();
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
        baseline_hash: hex_digest('a', 64),
        baseline_timestamp: "2026-02-16T10:00:00Z".to_string(),
        current_hash: None,
        current_hash_timestamp: None,
        status: "unchanged".to_string(),
        changes_json: None,
    };
    db.upsert_processed_db_integrity(&integrity).unwrap();
    let integ = db.get_processed_db_integrity("pdb_1").unwrap();
    assert_eq!(integ.len(), 1);
    assert_eq!(integ[0].baseline_hash, hex_digest('a', 64));

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
    assert_eq!(stats.total_artifacts, 0);
    assert_eq!(stats.total_source_analyses, 0);

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
fn test_processed_records_reject_invalid_persistence_payloads() {
    let (_dir, db) = create_test_db();

    let mut processed = DbProcessedDatabase {
        id: "pdb_validation".to_string(),
        path: "/case/processed/validation".to_string(),
        name: "Validation Processed DB".to_string(),
        db_type: "MagnetAxiom".to_string(),
        case_number: None,
        examiner: None,
        created_date: None,
        total_size: 1024,
        artifact_count: Some(1),
        notes: None,
        registered_at: "2026-02-16T10:00:00Z".to_string(),
        metadata_json: Some(r#"{"safe":true}"#.to_string()),
    };

    processed.path = " ".to_string();
    let err = db.upsert_processed_database(&processed).unwrap_err();
    assert!(err.to_string().contains("Processed path cannot be blank"));

    processed.path = "/case/processed/validation".to_string();
    processed.total_size = -1;
    let err = db.upsert_processed_database(&processed).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed total_size cannot be negative"));

    processed.total_size = 1024;
    processed.artifact_count = Some(-1);
    let err = db.upsert_processed_database(&processed).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed artifact_count cannot be negative"));

    processed.artifact_count = Some(1);
    processed.metadata_json = Some("{not-json".to_string());
    let err = db.upsert_processed_database(&processed).unwrap_err();
    assert!(err.to_string().contains("Invalid processed metadata_json"));

    processed.metadata_json = Some(r#"{"safe":true}"#.to_string());
    db.upsert_processed_database(&processed).unwrap();

    let mut integrity = DbProcessedDbIntegrity {
        id: "pdi_validation".to_string(),
        processed_db_id: processed.id.clone(),
        file_path: "/case/processed/validation/Case.mfdb".to_string(),
        file_size: 1024,
        baseline_hash: hex_digest('a', 64),
        baseline_timestamp: "2026-02-16T10:00:00Z".to_string(),
        current_hash: Some(hex_digest('b', 64)),
        current_hash_timestamp: Some("2026-02-16T10:01:00Z".to_string()),
        status: "unchanged".to_string(),
        changes_json: Some("[]".to_string()),
    };

    integrity.baseline_hash = "not-a-digest".to_string();
    let err = db.upsert_processed_db_integrity(&integrity).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed baseline_hash is not a valid hex digest"));

    integrity.baseline_hash = hex_digest('a', 64);
    integrity.current_hash = Some("bad-current".to_string());
    let err = db.upsert_processed_db_integrity(&integrity).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed current_hash is not a valid hex digest"));

    integrity.current_hash = Some(hex_digest('b', 64));
    integrity.file_size = -1;
    let err = db.upsert_processed_db_integrity(&integrity).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed file_size cannot be negative"));

    integrity.file_size = 1024;
    integrity.changes_json = Some("{not-json".to_string());
    let err = db.upsert_processed_db_integrity(&integrity).unwrap_err();
    assert!(err.to_string().contains("Invalid processed changes_json"));

    integrity.changes_json = Some("[]".to_string());
    db.upsert_processed_db_integrity(&integrity).unwrap();

    let mut metrics = DbProcessedDbMetrics {
        id: "pdm_validation".to_string(),
        processed_db_id: processed.id.clone(),
        total_scans: 1,
        last_scan_date: None,
        total_jobs: 1,
        last_job_date: None,
        total_notes: 0,
        total_tagged_items: 0,
        total_users: 1,
        user_names_json: Some(r#"["analyst"]"#.to_string()),
        captured_at: "2026-02-16T10:00:00Z".to_string(),
    };

    metrics.total_jobs = -1;
    let err = db.upsert_processed_db_metrics(&metrics).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed total_jobs cannot be negative"));

    metrics.total_jobs = 1;
    metrics.user_names_json = Some("{not-json".to_string());
    let err = db.upsert_processed_db_metrics(&metrics).unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid processed user_names_json"));

    metrics.user_names_json = Some(r#"["analyst"]"#.to_string());
    db.upsert_processed_db_metrics(&metrics).unwrap();

    let mut axiom = DbAxiomCaseInfo {
        id: "axc_validation".to_string(),
        processed_db_id: processed.id.clone(),
        case_name: "Validation Case".to_string(),
        case_number: None,
        case_type: None,
        description: None,
        examiner: None,
        agency: None,
        axiom_version: None,
        search_start: None,
        search_end: None,
        search_duration: None,
        search_outcome: None,
        output_folder: None,
        total_artifacts: 1,
        case_path: None,
        captured_at: "2026-02-16T10:00:00Z".to_string(),
        keyword_info_json: Some("[]".to_string()),
    };

    axiom.case_name = " ".to_string();
    let err = db.upsert_axiom_case_info(&axiom).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed case_name cannot be blank"));

    axiom.case_name = "Validation Case".to_string();
    axiom.total_artifacts = -1;
    let err = db.upsert_axiom_case_info(&axiom).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed total_artifacts cannot be negative"));

    axiom.total_artifacts = 1;
    axiom.keyword_info_json = Some("{not-json".to_string());
    let err = db.upsert_axiom_case_info(&axiom).unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid processed keyword_info_json"));

    axiom.keyword_info_json = Some("[]".to_string());
    db.upsert_axiom_case_info(&axiom).unwrap();

    let mut source = DbAxiomEvidenceSource {
        id: "axs_validation".to_string(),
        axiom_case_id: axiom.id.clone(),
        name: "validation.E01".to_string(),
        evidence_number: None,
        source_type: "image".to_string(),
        path: Some("/case/evidence/validation.E01".to_string()),
        hash: Some("sha256:abcdef".to_string()),
        size: Some(1024),
        acquired: None,
        search_types_json: Some(r#"["full"]"#.to_string()),
    };

    source.name = " ".to_string();
    let err = db.insert_axiom_evidence_source(&source).unwrap_err();
    assert!(err.to_string().contains("Processed name cannot be blank"));

    source.name = "validation.E01".to_string();
    source.size = Some(-1);
    let err = db.insert_axiom_evidence_source(&source).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed size cannot be negative"));

    source.size = Some(1024);
    source.search_types_json = Some("{not-json".to_string());
    let err = db.insert_axiom_evidence_source(&source).unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid processed search_types_json"));

    source.search_types_json = Some(r#"["full"]"#.to_string());
    db.insert_axiom_evidence_source(&source).unwrap();

    let mut search_result = DbAxiomSearchResult {
        id: "axr_validation".to_string(),
        axiom_case_id: axiom.id.clone(),
        artifact_type: "Web Visits".to_string(),
        hit_count: 1,
    };

    search_result.hit_count = -1;
    let err = db.insert_axiom_search_result(&search_result).unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed hit_count cannot be negative"));

    search_result.hit_count = 1;
    db.insert_axiom_search_result(&search_result).unwrap();

    let valid_category = DbArtifactCategory {
        id: "cat_validation_valid".to_string(),
        processed_db_id: processed.id.clone(),
        category: "Web".to_string(),
        artifact_type: "Web Visits".to_string(),
        count: 1,
    };
    let mut invalid_category = valid_category.clone();
    invalid_category.id = "cat_validation_invalid".to_string();
    invalid_category.count = -1;

    let err = db
        .upsert_artifact_categories(&[valid_category.clone(), invalid_category])
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Processed count cannot be negative"));
    assert!(db
        .get_artifact_categories(&processed.id)
        .unwrap()
        .is_empty());

    db.upsert_artifact_categories(&[valid_category]).unwrap();
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
    assert_eq!(
        imported_coc.evidence_file_id.as_deref(),
        Some("ev-existing")
    );
    assert_eq!(
        imported_coc.storage_class.as_deref(),
        Some("evidence_locker")
    );
    assert_eq!(
        imported_coc.storage_location_detail.as_deref(),
        Some("Shelf 7")
    );

    let items = db.get_collected_items(&imported_collection.id).unwrap();
    assert_eq!(items.len(), 2);

    let preserved_item = items
        .iter()
        .find(|item| item.item_number == "ITEM-001")
        .expect("expected preserved linked item");
    assert_ne!(preserved_item.id, "item-src-1");
    assert_eq!(
        preserved_item.coc_item_id.as_deref(),
        Some(imported_coc.id.as_str())
    );
    assert_eq!(
        preserved_item.evidence_file_id.as_deref(),
        Some("ev-existing")
    );
    assert_eq!(
        preserved_item.source_id.as_deref(),
        Some("ev-existing:/Users/test/phone.E01")
    );
    assert_eq!(
        preserved_item.source_ref_json.as_deref(),
        Some(r#"{"type":"localFile","path":"/Users/test/phone.E01"}"#)
    );
    assert_eq!(preserved_item.hash_algorithm.as_deref(), Some("sha256"));
    assert_eq!(
        preserved_item.hash_value.as_deref(),
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    );
    assert_eq!(
        preserved_item.hash_computed_at.as_deref(),
        Some("2026-04-14T10:02:00Z")
    );
    assert_eq!(
        preserved_item.packaging_type.as_deref(),
        Some("faraday_bag")
    );
    assert_eq!(
        preserved_item.packaging_detail.as_deref(),
        Some("Mission Darkness bag")
    );

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
    assert_eq!(
        transfers[0].storage_class.as_deref(),
        Some("evidence_locker")
    );
    assert_eq!(
        transfers[0].storage_location_detail.as_deref(),
        Some("Shelf 7")
    );

    let amendments = db.get_coc_amendments(&imported_coc.id).unwrap();
    assert_eq!(amendments.len(), 1);
    assert_eq!(amendments[0].coc_item_id, imported_coc.id);

    let audit_log = db.get_coc_audit_log(Some(&imported_coc.id)).unwrap();
    assert_eq!(audit_log.len(), 1);
    assert_eq!(
        audit_log[0].coc_item_id.as_deref(),
        Some(imported_coc.id.as_str())
    );
    assert_eq!(audit_log[0].action, "imported");
}

#[test]
fn test_collected_item_upsert_rejects_invalid_source_and_hash_payloads() {
    let (_dir, db) = create_test_db();
    let collection = make_existing_collection("collection-validation");
    db.upsert_evidence_collection(&collection).unwrap();

    let mut item = make_existing_item("item-validation", &collection.id);
    item.source_ref_json = Some("{not-json".to_string());
    let err = db.upsert_collected_item(&item).unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid collected item source_ref_json"));

    item.source_ref_json = Some(format!("\"{}\"", "a".repeat(1024 * 1024)));
    let err = db.upsert_collected_item(&item).unwrap_err();
    assert!(err
        .to_string()
        .contains("Collected item source_ref_json exceeds"));

    item.source_ref_json = Some(r#"{"type":"localFile","path":"/case/phone.E01"}"#.to_string());
    item.hash_value =
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string());
    let err = db.upsert_collected_item(&item).unwrap_err();
    assert!(err
        .to_string()
        .contains("Collected item hash value requires hash algorithm"));

    item.hash_algorithm = Some("sha256".to_string());
    item.hash_value = None;
    let err = db.upsert_collected_item(&item).unwrap_err();
    assert!(err
        .to_string()
        .contains("Collected item hash algorithm requires hash value"));

    item.hash_value = Some("not-a-hex-digest".to_string());
    let err = db.upsert_collected_item(&item).unwrap_err();
    assert!(err
        .to_string()
        .contains("Collected item hash value is not a valid SHA-256 digest"));

    item.hash_algorithm = Some("sha999".to_string());
    item.hash_value =
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string());
    let err = db.upsert_collected_item(&item).unwrap_err();
    assert!(err
        .to_string()
        .contains("Invalid collected item hash algorithm"));

    item.hash_algorithm = Some("sha256".to_string());
    db.upsert_collected_item(&item).unwrap();

    let items = db.get_collected_items(&collection.id).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].hash_algorithm.as_deref(), Some("sha256"));
}
