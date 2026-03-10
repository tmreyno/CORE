// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Data types, structs, and constants for the project database module.

use serde::{Deserialize, Serialize};

// =============================================================================
// Constants
// =============================================================================

/// File extension for project databases
pub const PROJECT_DB_EXTENSION: &str = ".ffxdb";

/// Current schema version for migration tracking
/// v1: Initial schema (activity, sessions, users, evidence, hashes, verifications, bookmarks, notes, tags, tabs, ui_state, reports, searches, case_documents)
/// v2: Added processed database tables (processed_databases, processed_db_integrity, processed_db_metrics, axiom_case_info, axiom_evidence_sources, axiom_search_results, axiom_keywords, artifact_categories)
/// v3: Added forensic workflow tables (export_history, chain_of_custody, file_classifications, extraction_log, viewer_history, annotations, evidence_relationships) + FTS5
/// v4: Added COC items & evidence collection tables (coc_items, coc_transfers, evidence_collections, collected_items)
/// v5: COC immutability model (coc_amendments, coc_audit_log, status/locked_at/locked_by on coc_items)
/// v7: Evidence collection status lifecycle (status column on evidence_collections)
/// v9: Form 7-01 COC alignment (15 new coc_items columns, 2 new coc_transfers columns)
pub const SCHEMA_VERSION: u32 = 10;

/// Application name for metadata
pub const APP_NAME: &str = "CORE-FFX";

// =============================================================================
// Core Data Types
// =============================================================================

/// Activity log entry (immutable audit trail of examiner actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbActivityEntry {
    pub id: String,
    pub timestamp: String,
    pub user: String,
    pub category: String,
    pub action: String,
    pub description: String,
    pub file_path: Option<String>,
    pub details: Option<String>,
}

/// Work session record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectSession {
    pub session_id: String,
    pub user: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub hostname: Option<String>,
    pub app_version: String,
    pub summary: Option<String>,
}

/// User/examiner record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectUser {
    pub username: String,
    pub display_name: Option<String>,
    pub hostname: Option<String>,
    pub first_access: String,
    pub last_access: String,
}

/// Evidence file (discovered container) record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbEvidenceFile {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub container_type: String,
    pub total_size: i64,
    pub segment_count: i32,
    pub discovered_at: String,
    pub created: Option<String>,
    pub modified: Option<String>,
}

/// Hash record (immutable audit trail)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectHash {
    pub id: String,
    pub file_id: String,
    pub algorithm: String,
    pub hash_value: String,
    pub computed_at: String,
    pub segment_index: Option<i32>,
    pub segment_name: Option<String>,
    pub source: String,
}

/// Hash verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectVerification {
    pub id: String,
    pub hash_id: String,
    pub verified_at: String,
    pub result: String,
    pub expected_hash: String,
    pub actual_hash: String,
}

/// Bookmark (examiner-placed marker)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbBookmark {
    pub id: String,
    pub target_type: String,
    pub target_path: String,
    pub name: String,
    pub created_by: String,
    pub created_at: String,
    pub color: Option<String>,
    pub notes: Option<String>,
    pub context: Option<String>,
}

/// Note (examiner annotation)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbNote {
    pub id: String,
    pub target_type: String,
    pub target_path: Option<String>,
    pub title: String,
    pub content: String,
    pub created_by: String,
    pub created_at: String,
    pub modified_at: String,
    pub priority: Option<String>,
}

/// Tag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbTag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// Tag assignment (many-to-many link)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbTagAssignment {
    pub tag_id: String,
    pub target_type: String,
    pub target_id: String,
    pub assigned_at: String,
    pub assigned_by: String,
}

/// Open tab (UI state)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectTab {
    pub id: String,
    pub tab_type: String,
    pub file_path: String,
    pub name: String,
    pub subtitle: Option<String>,
    pub tab_order: i32,
    pub extra: Option<String>,
}

/// Generated report record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbReportRecord {
    pub id: String,
    pub title: String,
    pub report_type: String,
    pub format: String,
    pub output_path: Option<String>,
    pub generated_at: String,
    pub generated_by: String,
    pub status: String,
    pub error: Option<String>,
    pub config: Option<String>,
}

/// Saved search definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbSavedSearch {
    pub id: String,
    pub name: String,
    pub query: String,
    pub search_type: String,
    pub is_regex: bool,
    pub case_sensitive: bool,
    pub scope: String,
    pub created_at: String,
    pub use_count: i32,
    pub last_used: Option<String>,
}

/// Recent search entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbRecentSearch {
    pub query: String,
    pub timestamp: String,
    pub result_count: i32,
}

/// Case document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCaseDocument {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub document_type: String,
    pub size: i64,
    pub format: String,
    pub case_number: Option<String>,
    pub evidence_id: Option<String>,
    pub modified: Option<String>,
    pub discovered_at: String,
}

// =============================================================================
// Processed Database Types
// =============================================================================

/// Processed database registry entry (AXIOM, PA, Autopsy, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProcessedDatabase {
    pub id: String,
    pub path: String,
    pub name: String,
    pub db_type: String,
    pub case_number: Option<String>,
    pub examiner: Option<String>,
    pub created_date: Option<String>,
    pub total_size: i64,
    pub artifact_count: Option<i64>,
    pub notes: Option<String>,
    pub registered_at: String,
    pub metadata_json: Option<String>,
}

/// Processed database file integrity tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProcessedDbIntegrity {
    pub id: String,
    pub processed_db_id: String,
    pub file_path: String,
    pub file_size: i64,
    pub baseline_hash: String,
    pub baseline_timestamp: String,
    pub current_hash: Option<String>,
    pub current_hash_timestamp: Option<String>,
    pub status: String,
    pub changes_json: Option<String>,
}

/// Processed database work metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProcessedDbMetrics {
    pub id: String,
    pub processed_db_id: String,
    pub total_scans: i32,
    pub last_scan_date: Option<String>,
    pub total_jobs: i32,
    pub last_job_date: Option<String>,
    pub total_notes: i32,
    pub total_tagged_items: i32,
    pub total_users: i32,
    pub user_names_json: Option<String>,
    pub captured_at: String,
}

/// AXIOM case information snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbAxiomCaseInfo {
    pub id: String,
    pub processed_db_id: String,
    pub case_name: String,
    pub case_number: Option<String>,
    pub case_type: Option<String>,
    pub description: Option<String>,
    pub examiner: Option<String>,
    pub agency: Option<String>,
    pub axiom_version: Option<String>,
    pub search_start: Option<String>,
    pub search_end: Option<String>,
    pub search_duration: Option<String>,
    pub search_outcome: Option<String>,
    pub output_folder: Option<String>,
    pub total_artifacts: i64,
    pub case_path: Option<String>,
    pub captured_at: String,
    pub keyword_info_json: Option<String>,
}

/// AXIOM evidence source entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbAxiomEvidenceSource {
    pub id: String,
    pub axiom_case_id: String,
    pub name: String,
    pub evidence_number: Option<String>,
    pub source_type: String,
    pub path: Option<String>,
    pub hash: Option<String>,
    pub size: Option<i64>,
    pub acquired: Option<String>,
    pub search_types_json: Option<String>,
}

/// AXIOM search result (artifact type → hit count)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbAxiomSearchResult {
    pub id: String,
    pub axiom_case_id: String,
    pub artifact_type: String,
    pub hit_count: i64,
}

/// Artifact category summary (works for any processed DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbArtifactCategory {
    pub id: String,
    pub processed_db_id: String,
    pub category: String,
    pub artifact_type: String,
    pub count: i64,
}

// =============================================================================
// Forensic Workflow Types
// =============================================================================

/// Export history record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbExportRecord {
    pub id: String,
    pub export_type: String,
    pub source_paths_json: String,
    pub destination: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub initiated_by: String,
    pub status: String,
    pub total_files: i64,
    pub total_bytes: i64,
    pub archive_name: Option<String>,
    pub archive_format: Option<String>,
    pub compression_level: Option<String>,
    pub encrypted: Option<bool>,
    pub manifest_hash: Option<String>,
    pub error: Option<String>,
    pub options_json: Option<String>,
}

/// Chain of custody event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCustodyRecord {
    pub id: String,
    pub action: String,
    pub from_person: String,
    pub to_person: String,
    pub date: String,
    pub time: Option<String>,
    pub location: Option<String>,
    pub purpose: Option<String>,
    pub notes: Option<String>,
    pub evidence_ids_json: Option<String>,
    pub recorded_by: String,
    pub recorded_at: String,
}

// =============================================================================
// COC Item Types (v4-v5 — immutability model)
// =============================================================================

/// Per-evidence Chain of Custody item (EPA CID OCEFT Form 7-01 aligned)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCocItem {
    pub id: String,
    pub coc_number: String,
    pub evidence_file_id: Option<String>,
    pub case_number: String,
    pub evidence_id: String,
    pub description: String,
    pub item_type: String,

    // ── Form 7-01 Header ──
    pub case_title: Option<String>,
    pub office: Option<String>,

    // ── Owner / Source / Contact ──
    pub owner_name: Option<String>,
    pub owner_address: Option<String>,
    pub owner_phone: Option<String>,
    pub source: Option<String>,
    pub other_contact_name: Option<String>,
    pub other_contact_relation: Option<String>,
    pub other_contact_phone: Option<String>,

    // ── Collection Method ──
    pub collection_method: Option<String>,
    pub collection_method_other: Option<String>,

    // ── Item Details ──
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub capacity: Option<String>,
    pub condition: String,

    // ── Custody / Collection ──
    pub acquisition_date: String,
    pub entered_custody_date: String,
    pub submitted_by: String,
    pub collected_date: Option<String>,
    pub received_by: String,
    pub received_location: Option<String>,
    pub storage_location: Option<String>,
    pub reason_submitted: Option<String>,
    pub intake_hashes_json: Option<String>,
    pub notes: Option<String>,

    // ── Final Disposition (Form 7-01) ──
    pub disposition: Option<String>,
    pub disposition_by: Option<String>,
    pub returned_to: Option<String>,
    pub destruction_date: Option<String>,
    pub disposition_date: Option<String>,
    pub disposition_notes: Option<String>,

    // ── Timestamps + Immutability ──
    pub created_at: String,
    pub modified_at: String,
    #[serde(default = "default_coc_status")]
    pub status: String,
    pub locked_at: Option<String>,
    pub locked_by: Option<String>,
}

fn default_coc_status() -> String {
    "draft".to_string()
}

/// COC amendment record (append-only field change audit)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCocAmendment {
    pub id: String,
    pub coc_item_id: String,
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
    pub amended_by_initials: String,
    pub amended_at: String,
    pub reason: Option<String>,
}

/// COC audit log entry (immutable action history)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCocAuditEntry {
    pub id: String,
    pub coc_item_id: Option<String>,
    pub action: String,
    pub performed_by: String,
    pub performed_at: String,
    pub summary: String,
    pub details_json: Option<String>,
}

/// Generic form submission record (JSON schema-driven)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbFormSubmission {
    pub id: String,
    pub template_id: String,
    pub template_version: String,
    pub case_number: Option<String>,
    pub data_json: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// COC transfer record (Form 7-01: Relinquished to / Storage Location)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCocTransfer {
    pub id: String,
    pub coc_item_id: String,
    pub timestamp: String,
    pub released_by: String,
    pub received_by: String,
    pub purpose: String,
    pub location: Option<String>,
    pub storage_location: Option<String>,
    pub storage_date: Option<String>,
    pub method: Option<String>,
    pub notes: Option<String>,
}

// =============================================================================
// Evidence Collection Types
// =============================================================================

/// Evidence collection event record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbEvidenceCollection {
    pub id: String,
    pub case_number: String,
    pub collection_date: String,
    pub collection_location: String,
    pub collecting_officer: String,
    pub authorization: String,
    pub authorization_date: Option<String>,
    pub authorizing_authority: Option<String>,
    pub witnesses_json: Option<String>,
    pub documentation_notes: Option<String>,
    pub conditions: Option<String>,
    #[serde(default = "default_draft_status")]
    pub status: String,
    pub created_at: String,
    pub modified_at: String,
    /// Virtual field: count of collected items (populated by query)
    #[serde(default)]
    pub item_count: i64,
}

fn default_draft_status() -> String {
    "draft".to_string()
}

/// Individual collected item within a collection event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCollectedItem {
    pub id: String,
    pub collection_id: String,
    pub coc_item_id: Option<String>,
    pub evidence_file_id: Option<String>,
    pub item_number: String,
    pub description: String,
    pub found_location: String,
    pub item_type: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub condition: String,
    pub packaging: String,
    pub photo_refs_json: Option<String>,
    pub notes: Option<String>,
    // --- Per-item collection fields (v8) ---
    pub item_collection_datetime: Option<String>,
    pub item_system_datetime: Option<String>,
    pub item_collecting_officer: Option<String>,
    pub item_authorization: Option<String>,
    // --- Device identification ---
    pub device_type: Option<String>,
    pub device_type_other: Option<String>,
    pub storage_interface: Option<String>,
    pub storage_interface_other: Option<String>,
    pub brand: Option<String>,
    pub color: Option<String>,
    pub imei: Option<String>,
    pub other_identifiers: Option<String>,
    // --- Detailed location ---
    pub building: Option<String>,
    pub room: Option<String>,
    pub location_other: Option<String>,
    // --- Forensic imaging ---
    pub image_format: Option<String>,
    pub image_format_other: Option<String>,
    pub acquisition_method: Option<String>,
    pub acquisition_method_other: Option<String>,
    pub storage_notes: Option<String>,
}

// =============================================================================
// Additional Forensic Workflow Types
// =============================================================================

/// Alternative data record — stores non-selected field values when user resolves
/// conflicts between container metadata and manual entries. Preserves both sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbEvidenceDataAlternative {
    pub id: String,
    /// FK to collected_items.id
    pub collected_item_id: String,
    /// FK to evidence_files.id (the matched container)
    pub evidence_file_id: Option<String>,
    /// The field name that had a conflict (e.g., "serial_number", "model")
    pub field_name: String,
    /// The value that was chosen ("user" or "container")
    pub chosen_source: String,
    /// The value the user manually entered
    pub user_value: Option<String>,
    /// The value extracted from the container metadata
    pub container_value: Option<String>,
    /// Who resolved the conflict
    pub resolved_by: Option<String>,
    /// When the conflict was resolved
    pub resolved_at: String,
    /// Optional note about why one value was preferred
    pub resolution_note: Option<String>,
}

/// File classification record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbFileClassification {
    pub id: String,
    pub file_path: String,
    pub container_path: Option<String>,
    pub classification: String,
    pub custom_label: Option<String>,
    pub classified_by: String,
    pub classified_at: String,
    pub notes: Option<String>,
    pub confidence: Option<String>,
}

/// Extraction log entry (audit trail)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbExtractionRecord {
    pub id: String,
    pub container_path: String,
    pub entry_path: String,
    pub output_path: String,
    pub extracted_by: String,
    pub extracted_at: String,
    pub entry_size: i64,
    pub purpose: String,
    pub hash_value: Option<String>,
    pub hash_algorithm: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

/// Viewer history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbViewerHistoryEntry {
    pub id: String,
    pub file_path: String,
    pub container_path: Option<String>,
    pub viewer_type: String,
    pub viewed_by: String,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub duration_seconds: Option<i64>,
}

/// Annotation (hex/document viewer highlight)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbAnnotation {
    pub id: String,
    pub file_path: String,
    pub container_path: Option<String>,
    pub annotation_type: String,
    pub offset_start: Option<i64>,
    pub offset_end: Option<i64>,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub label: String,
    pub content: Option<String>,
    pub color: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub modified_at: String,
}

/// Evidence relationship (link between files)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbEvidenceRelationship {
    pub id: String,
    pub source_path: String,
    pub target_path: String,
    pub relationship_type: String,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

// =============================================================================
// Query & Result Types
// =============================================================================

/// Full-text search result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtsSearchResult {
    pub source: String,
    pub id: String,
    pub snippet: String,
    pub rank: f64,
}

/// Activity log query filter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityQuery {
    pub category: Option<String>,
    pub user: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub file_path: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Project database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbStats {
    pub total_activities: i64,
    pub total_sessions: i64,
    pub total_users: i64,
    pub total_evidence_files: i64,
    pub total_hashes: i64,
    pub total_verifications: i64,
    pub total_bookmarks: i64,
    pub total_notes: i64,
    pub total_tags: i64,
    pub total_reports: i64,
    pub total_saved_searches: i64,
    pub total_case_documents: i64,
    pub total_processed_databases: i64,
    pub total_axiom_cases: i64,
    pub total_artifact_categories: i64,
    pub total_exports: i64,
    pub total_custody_records: i64,
    pub total_classifications: i64,
    pub total_extractions: i64,
    pub total_viewer_history: i64,
    pub total_annotations: i64,
    pub total_relationships: i64,
    pub total_coc_items: i64,
    pub total_coc_transfers: i64,
    pub total_evidence_collections: i64,
    pub total_collected_items: i64,
    pub total_coc_amendments: i64,
    pub total_coc_audit_entries: i64,
    pub db_size_bytes: u64,
    pub schema_version: u32,
}
