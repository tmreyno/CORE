// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Per-project SQLite database (.ffxdb) for forensic project persistence
//!
//! Each CORE-FFX project gets a companion `.ffxdb` database that lives alongside
//! the `.cffx` manifest file. This database stores all growing/queryable data:
//!
//! - Activity log (examiner actions with timestamps)
//! - Sessions (work sessions with duration tracking)
//! - Users (examiners who have accessed the project)
//! - Evidence files (discovered containers and metadata)
//! - Hashes (computed hash audit trail)
//! - Verifications (hash verification results)
//! - Bookmarks & Notes (examiner annotations)
//! - Tags (tag definitions and assignments)
//! - Tabs & UI state (for session restoration)
//! - Reports (generated report records)
//! - Searches (saved and recent searches)
//! - Case documents (cached document metadata)
//!
//! The `.cffx` file remains a lightweight JSON manifest with project metadata,
//! locations, and settings. The `.ffxdb` file is portable — it travels with
//! the case folder for forensic chain of custody.

use parking_lot::Mutex;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

// =============================================================================
// Constants
// =============================================================================

/// File extension for project databases
pub const PROJECT_DB_EXTENSION: &str = ".ffxdb";

/// Current schema version for migration tracking
/// v1: Initial schema (activity, sessions, users, evidence, hashes, verifications, bookmarks, notes, tags, tabs, ui_state, reports, searches, case_documents)
/// v2: Added processed database tables (processed_databases, processed_db_integrity, processed_db_metrics, axiom_case_info, axiom_evidence_sources, axiom_search_results, axiom_keywords, artifact_categories)
/// v3: Added forensic workflow tables (export_history, chain_of_custody, file_classifications, extraction_log, viewer_history, annotations, evidence_relationships) + FTS5
pub const SCHEMA_VERSION: u32 = 3;

/// Application name for metadata
pub const APP_NAME: &str = "CORE-FFX";

// =============================================================================
// Data Types (serializable for IPC)
// =============================================================================

/// Activity log entry — immutable audit trail of examiner actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbActivityEntry {
    pub id: String,
    pub timestamp: String,
    pub user: String,
    pub category: String,
    pub action: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// JSON-encoded additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Session record — tracks examiner work sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectSession {
    pub session_id: String,
    pub user: String,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    pub app_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// User/examiner record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectUser {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    pub first_access: String,
    pub last_access: String,
}

/// Evidence file record — discovered forensic containers
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

/// Hash record — immutable audit trail of hash computations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectHash {
    pub id: String,
    pub file_id: String,
    pub algorithm: String,
    pub hash_value: String,
    pub computed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_name: Option<String>,
    /// Source: 'computed', 'stored', 'imported'
    pub source: String,
}

/// Verification record — hash verification audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectVerification {
    pub id: String,
    pub hash_id: String,
    pub verified_at: String,
    /// 'match' or 'mismatch'
    pub result: String,
    pub expected_hash: String,
    pub actual_hash: String,
}

/// Bookmark record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbBookmark {
    pub id: String,
    /// 'file', 'artifact', 'search_result', 'location'
    pub target_type: String,
    pub target_path: String,
    pub name: String,
    pub created_by: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// JSON-encoded additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Note/annotation record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbNote {
    pub id: String,
    /// 'file', 'artifact', 'database', 'case', 'general'
    pub target_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_path: Option<String>,
    pub title: String,
    /// Supports markdown
    pub content: String,
    pub created_by: String,
    pub created_at: String,
    pub modified_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
}

/// Tag definition record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbTag {
    pub id: String,
    pub name: String,
    /// Hex color string
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
}

/// Tag assignment (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbTagAssignment {
    pub tag_id: String,
    /// 'bookmark', 'note', 'file', 'artifact'
    pub target_type: String,
    pub target_id: String,
    pub assigned_at: String,
    pub assigned_by: String,
}

/// Tab record for UI state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProjectTab {
    pub id: String,
    pub tab_type: String,
    pub file_path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub tab_order: i32,
    /// JSON-encoded extra fields (container_type, entry_path, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
}

/// Report record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbReportRecord {
    pub id: String,
    pub title: String,
    pub report_type: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    pub generated_at: String,
    pub generated_by: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// JSON-encoded config
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
}

/// Saved search record
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

/// Case document record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCaseDocument {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub document_type: String,
    pub size: i64,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    pub discovered_at: String,
}

// =============================================================================
// Processed Database Types (read-only snapshots of AXIOM, PA, etc.)
// =============================================================================

/// A registered processed database (AXIOM, Cellebrite PA, X-Ways, etc.)
/// This is a read-only record of the external database — CORE-FFX never modifies it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProcessedDatabase {
    pub id: String,
    /// Path to the processed database folder or file
    pub path: String,
    /// Display name (case name, folder name, etc.)
    pub name: String,
    /// Type: 'MagnetAxiom', 'CellebritePA', 'XWays', 'Autopsy', 'EnCase', 'FTK', 'GenericSqlite', 'Unknown'
    pub db_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examiner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    pub total_size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// When this record was registered in CORE-FFX
    pub registered_at: String,
    /// JSON-encoded metadata snapshot (full ProcessedDbInfo)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_json: Option<String>,
}

/// Integrity record for a processed database — tracks whether the external DB has changed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProcessedDbIntegrity {
    pub id: String,
    /// FK to processed_databases.id
    pub processed_db_id: String,
    /// Database file path (individual .mfdb, .db, etc.)
    pub file_path: String,
    pub file_size: i64,
    /// Hash computed when first loaded
    pub baseline_hash: String,
    pub baseline_timestamp: String,
    /// Most recent hash check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_hash_timestamp: Option<String>,
    /// 'unchanged', 'modified', 'new_baseline', 'not_verified'
    pub status: String,
    /// JSON array of detected changes since baseline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes_json: Option<String>,
}

/// Work metrics extracted from a processed database (read-only snapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbProcessedDbMetrics {
    pub id: String,
    /// FK to processed_databases.id
    pub processed_db_id: String,
    pub total_scans: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scan_date: Option<String>,
    pub total_jobs: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_job_date: Option<String>,
    pub total_notes: i32,
    pub total_tagged_items: i32,
    pub total_users: i32,
    /// JSON array of user names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_names_json: Option<String>,
    /// When these metrics were captured
    pub captured_at: String,
}

/// AXIOM-specific case information (read-only snapshot from .mcfc / Case Information.xml / .mfdb)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbAxiomCaseInfo {
    pub id: String,
    /// FK to processed_databases.id
    pub processed_db_id: String,
    pub case_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examiner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axiom_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_duration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_folder: Option<String>,
    pub total_artifacts: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_path: Option<String>,
    /// When this snapshot was captured
    pub captured_at: String,
    /// JSON-encoded keyword info (AxiomKeywordInfo)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword_info_json: Option<String>,
}

/// AXIOM evidence source (read-only snapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbAxiomEvidenceSource {
    pub id: String,
    /// FK to axiom_case_info.id
    pub axiom_case_id: String,
    /// Source name/path
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_number: Option<String>,
    /// 'image', 'mobile', 'cloud', etc.
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquired: Option<String>,
    /// JSON array of search types applied
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_types_json: Option<String>,
}

/// AXIOM search result entry (artifact type + hit count)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbAxiomSearchResult {
    pub id: String,
    /// FK to axiom_case_info.id
    pub axiom_case_id: String,
    pub artifact_type: String,
    pub hit_count: i64,
}

/// Artifact category summary (works for any processed DB type, not just AXIOM)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbArtifactCategory {
    pub id: String,
    /// FK to processed_databases.id
    pub processed_db_id: String,
    pub category: String,
    pub artifact_type: String,
    pub count: i64,
}

// =============================================================================
// v3 Forensic Workflow Types
// =============================================================================

/// Export operation record — tracks every export/archive operation for chain-of-custody
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbExportRecord {
    pub id: String,
    /// 'copy', 'forensic_export', 'archive_7z'
    pub export_type: String,
    /// JSON array of source paths
    pub source_paths_json: String,
    pub destination: String,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    pub initiated_by: String,
    /// 'pending', 'in_progress', 'completed', 'failed', 'cancelled'
    pub status: String,
    pub total_files: i64,
    pub total_bytes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_level: Option<String>,
    pub encrypted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// JSON-encoded options used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_json: Option<String>,
}

/// Chain of custody record — persists per-project custody events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbCustodyRecord {
    pub id: String,
    /// 'received', 'transferred', 'returned', 'stored', 'analyzed', 'sealed', 'opened'
    pub action: String,
    pub from_person: String,
    pub to_person: String,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// JSON array of evidence item IDs affected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ids_json: Option<String>,
    pub recorded_by: String,
    pub recorded_at: String,
}

/// File classification — examiner-assigned labels for evidence items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbFileClassification {
    pub id: String,
    /// Path to the classified file/entry
    pub file_path: String,
    /// Path to the container (if inside a container)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_path: Option<String>,
    /// 'relevant', 'privileged', 'excluded', 'reviewed', 'flagged', 'exported', 'custom'
    pub classification: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_label: Option<String>,
    pub classified_by: String,
    pub classified_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Confidence: 'high', 'medium', 'low'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
}

/// Extraction log entry — immutable record of every file extracted from a container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbExtractionRecord {
    pub id: String,
    /// Path to the container file
    pub container_path: String,
    /// Path of the entry inside the container
    pub entry_path: String,
    /// Where the file was extracted to
    pub output_path: String,
    pub extracted_by: String,
    pub extracted_at: String,
    pub entry_size: i64,
    /// 'preview', 'export', 'analysis'
    pub purpose: String,
    /// Hash of extracted file (for integrity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_algorithm: Option<String>,
    /// 'success', 'failed', 'partial'
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Viewer history entry — tracks which files were viewed during examination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbViewerHistoryEntry {
    pub id: String,
    /// Path to the file or entry that was viewed
    pub file_path: String,
    /// Container path (if viewing an entry inside a container)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_path: Option<String>,
    /// 'hex', 'text', 'document', 'image', 'pdf', 'binary', 'info'
    pub viewer_type: String,
    pub viewed_by: String,
    pub opened_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
}

/// Annotation record — hex/document viewer annotations (highlights, comments on byte ranges)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbAnnotation {
    pub id: String,
    /// Path to the annotated file/entry
    pub file_path: String,
    /// Container path (if inside a container)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_path: Option<String>,
    /// 'highlight', 'comment', 'bookmark_range', 'signature'
    pub annotation_type: String,
    /// For byte-range annotations (hex viewer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_end: Option<i64>,
    /// For text/document annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<i32>,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub modified_at: String,
}

/// Evidence relationship — links evidence files to each other
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbEvidenceRelationship {
    pub id: String,
    /// Source evidence file path
    pub source_path: String,
    /// Target evidence file path
    pub target_path: String,
    /// 'full_disk_of', 'logical_of', 'segment_of', 'related_to', 'derived_from', 'contains'
    pub relationship_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

/// Full-text search result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtsSearchResult {
    /// Source table: 'notes', 'bookmarks', 'activity_log'
    pub source: String,
    pub id: String,
    /// The matched text snippet
    pub snippet: String,
    /// BM25 relevance rank
    pub rank: f64,
}

/// Query parameters for activity log filtering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Statistics summary for the project database
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
    // v3 stats
    pub total_exports: i64,
    pub total_custody_records: i64,
    pub total_classifications: i64,
    pub total_extractions: i64,
    pub total_viewer_history: i64,
    pub total_annotations: i64,
    pub total_relationships: i64,
    pub db_size_bytes: u64,
    pub schema_version: u32,
}

// =============================================================================
// ProjectDatabase — per-project SQLite (.ffxdb)
// =============================================================================

/// Per-project SQLite database for forensic activity persistence.
///
/// Unlike the global `ffx.db` (which tracks app-level sessions), this database
/// lives alongside the `.cffx` manifest in the case folder and is portable.
pub struct ProjectDatabase {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl ProjectDatabase {
    /// Open or create a project database at the given path.
    ///
    /// Creates the `.ffxdb` file and initializes the schema if it doesn't exist.
    /// Runs migrations if the schema version is older than current.
    pub fn open(db_path: &Path) -> SqlResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self {
            conn: Mutex::new(conn),
            path: db_path.to_path_buf(),
        };

        db.init_schema()?;
        db.check_migrations()?;

        info!("Project database opened: {:?}", db_path);
        Ok(db)
    }

    /// Derive the `.ffxdb` path from a `.cffx` project file path.
    ///
    /// The database sits alongside the project file in the same directory.
    /// Example: `/case/project.cffx` → `/case/project.ffxdb`
    pub fn db_path_for_project(cffx_path: &Path) -> PathBuf {
        cffx_path.with_extension("ffxdb")
    }

    /// Get the file path of this database
    pub fn path(&self) -> &Path {
        &self.path
    }

    // ========================================================================
    // Schema Initialization
    // ========================================================================

    fn init_schema(&self) -> SqlResult<()> {
        let conn = self.conn.lock();

        conn.execute_batch(
            r#"
            -- Schema metadata
            CREATE TABLE IF NOT EXISTS schema_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Users (examiners)
            CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                display_name TEXT,
                hostname TEXT,
                first_access TEXT NOT NULL,
                last_access TEXT NOT NULL
            );

            -- Sessions (work sessions)
            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                user TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                duration_seconds INTEGER,
                hostname TEXT,
                app_version TEXT NOT NULL,
                summary TEXT,
                FOREIGN KEY (user) REFERENCES users(username) ON DELETE CASCADE
            );

            -- Activity log (immutable audit trail)
            CREATE TABLE IF NOT EXISTS activity_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                user TEXT NOT NULL,
                category TEXT NOT NULL,
                action TEXT NOT NULL,
                description TEXT NOT NULL,
                file_path TEXT,
                details TEXT
            );

            -- Evidence files (discovered containers)
            CREATE TABLE IF NOT EXISTS evidence_files (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                filename TEXT NOT NULL,
                container_type TEXT NOT NULL,
                total_size INTEGER NOT NULL,
                segment_count INTEGER NOT NULL DEFAULT 1,
                discovered_at TEXT NOT NULL,
                created TEXT,
                modified TEXT
            );

            -- Hashes (immutable hash audit trail)
            CREATE TABLE IF NOT EXISTS hashes (
                id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL,
                algorithm TEXT NOT NULL,
                hash_value TEXT NOT NULL,
                computed_at TEXT NOT NULL,
                segment_index INTEGER,
                segment_name TEXT,
                source TEXT NOT NULL DEFAULT 'computed',
                FOREIGN KEY (file_id) REFERENCES evidence_files(id) ON DELETE CASCADE
            );

            -- Verifications (hash verification audit trail)
            CREATE TABLE IF NOT EXISTS verifications (
                id TEXT PRIMARY KEY,
                hash_id TEXT NOT NULL,
                verified_at TEXT NOT NULL,
                result TEXT NOT NULL,
                expected_hash TEXT NOT NULL,
                actual_hash TEXT NOT NULL,
                FOREIGN KEY (hash_id) REFERENCES hashes(id) ON DELETE CASCADE
            );

            -- Bookmarks
            CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                target_type TEXT NOT NULL,
                target_path TEXT NOT NULL,
                name TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                color TEXT,
                notes TEXT,
                context TEXT
            );

            -- Notes
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                target_type TEXT NOT NULL,
                target_path TEXT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                priority TEXT
            );

            -- Tags (definitions)
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            );

            -- Tag assignments (many-to-many)
            CREATE TABLE IF NOT EXISTS tag_assignments (
                tag_id TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                assigned_at TEXT NOT NULL,
                assigned_by TEXT NOT NULL,
                PRIMARY KEY (tag_id, target_type, target_id),
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            -- Tabs (UI state)
            CREATE TABLE IF NOT EXISTS tabs (
                id TEXT PRIMARY KEY,
                tab_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                name TEXT NOT NULL,
                subtitle TEXT,
                tab_order INTEGER NOT NULL,
                extra TEXT
            );

            -- UI state (key-value store)
            CREATE TABLE IF NOT EXISTS ui_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Reports
            CREATE TABLE IF NOT EXISTS reports (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                report_type TEXT NOT NULL,
                format TEXT NOT NULL,
                output_path TEXT,
                generated_at TEXT NOT NULL,
                generated_by TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                error TEXT,
                config TEXT
            );

            -- Saved searches
            CREATE TABLE IF NOT EXISTS saved_searches (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                query TEXT NOT NULL,
                search_type TEXT NOT NULL,
                is_regex INTEGER NOT NULL DEFAULT 0,
                case_sensitive INTEGER NOT NULL DEFAULT 0,
                scope TEXT NOT NULL DEFAULT 'all',
                created_at TEXT NOT NULL,
                use_count INTEGER NOT NULL DEFAULT 0,
                last_used TEXT
            );

            -- Recent searches
            CREATE TABLE IF NOT EXISTS recent_searches (
                query TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                result_count INTEGER NOT NULL DEFAULT 0
            );

            -- Case documents
            CREATE TABLE IF NOT EXISTS case_documents (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                filename TEXT NOT NULL,
                document_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                format TEXT NOT NULL,
                case_number TEXT,
                evidence_id TEXT,
                modified TEXT,
                discovered_at TEXT NOT NULL
            );

            -- =================================================================
            -- Processed Database Tables (read-only snapshots of external tools)
            -- =================================================================

            -- Registry of processed databases (AXIOM, PA, etc.)
            CREATE TABLE IF NOT EXISTS processed_databases (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                db_type TEXT NOT NULL DEFAULT 'Unknown',
                case_number TEXT,
                examiner TEXT,
                created_date TEXT,
                total_size INTEGER NOT NULL DEFAULT 0,
                artifact_count INTEGER,
                notes TEXT,
                registered_at TEXT NOT NULL,
                metadata_json TEXT
            );

            -- Integrity tracking for processed database files
            CREATE TABLE IF NOT EXISTS processed_db_integrity (
                id TEXT PRIMARY KEY,
                processed_db_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
                baseline_hash TEXT NOT NULL,
                baseline_timestamp TEXT NOT NULL,
                current_hash TEXT,
                current_hash_timestamp TEXT,
                status TEXT NOT NULL DEFAULT 'not_verified',
                changes_json TEXT,
                FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
            );

            -- Work metrics snapshots from processed databases
            CREATE TABLE IF NOT EXISTS processed_db_metrics (
                id TEXT PRIMARY KEY,
                processed_db_id TEXT NOT NULL,
                total_scans INTEGER NOT NULL DEFAULT 0,
                last_scan_date TEXT,
                total_jobs INTEGER NOT NULL DEFAULT 0,
                last_job_date TEXT,
                total_notes INTEGER NOT NULL DEFAULT 0,
                total_tagged_items INTEGER NOT NULL DEFAULT 0,
                total_users INTEGER NOT NULL DEFAULT 0,
                user_names_json TEXT,
                captured_at TEXT NOT NULL,
                FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
            );

            -- AXIOM case information snapshots
            CREATE TABLE IF NOT EXISTS axiom_case_info (
                id TEXT PRIMARY KEY,
                processed_db_id TEXT NOT NULL,
                case_name TEXT NOT NULL,
                case_number TEXT,
                case_type TEXT,
                description TEXT,
                examiner TEXT,
                agency TEXT,
                axiom_version TEXT,
                search_start TEXT,
                search_end TEXT,
                search_duration TEXT,
                search_outcome TEXT,
                output_folder TEXT,
                total_artifacts INTEGER NOT NULL DEFAULT 0,
                case_path TEXT,
                captured_at TEXT NOT NULL,
                keyword_info_json TEXT,
                FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
            );

            -- AXIOM evidence sources
            CREATE TABLE IF NOT EXISTS axiom_evidence_sources (
                id TEXT PRIMARY KEY,
                axiom_case_id TEXT NOT NULL,
                name TEXT NOT NULL,
                evidence_number TEXT,
                source_type TEXT NOT NULL DEFAULT 'unknown',
                path TEXT,
                hash TEXT,
                size INTEGER,
                acquired TEXT,
                search_types_json TEXT,
                FOREIGN KEY (axiom_case_id) REFERENCES axiom_case_info(id) ON DELETE CASCADE
            );

            -- AXIOM search results (artifact type -> hit count)
            CREATE TABLE IF NOT EXISTS axiom_search_results (
                id TEXT PRIMARY KEY,
                axiom_case_id TEXT NOT NULL,
                artifact_type TEXT NOT NULL,
                hit_count INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (axiom_case_id) REFERENCES axiom_case_info(id) ON DELETE CASCADE
            );

            -- Artifact category summaries (works for any processed DB type)
            CREATE TABLE IF NOT EXISTS artifact_categories (
                id TEXT PRIMARY KEY,
                processed_db_id TEXT NOT NULL,
                category TEXT NOT NULL,
                artifact_type TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
            );

            -- =================================================================
            -- v3: Forensic Workflow Tables
            -- =================================================================

            -- Export history (every export/archive operation)
            CREATE TABLE IF NOT EXISTS export_history (
                id TEXT PRIMARY KEY,
                export_type TEXT NOT NULL,
                source_paths_json TEXT NOT NULL,
                destination TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                initiated_by TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                total_files INTEGER NOT NULL DEFAULT 0,
                total_bytes INTEGER NOT NULL DEFAULT 0,
                archive_name TEXT,
                archive_format TEXT,
                compression_level TEXT,
                encrypted INTEGER NOT NULL DEFAULT 0,
                manifest_hash TEXT,
                error TEXT,
                options_json TEXT
            );

            -- Chain of custody (per-project custody events)
            CREATE TABLE IF NOT EXISTS chain_of_custody (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                from_person TEXT NOT NULL,
                to_person TEXT NOT NULL,
                date TEXT NOT NULL,
                time TEXT,
                location TEXT,
                purpose TEXT,
                notes TEXT,
                evidence_ids_json TEXT,
                recorded_by TEXT NOT NULL,
                recorded_at TEXT NOT NULL
            );

            -- File classifications (examiner-assigned labels)
            CREATE TABLE IF NOT EXISTS file_classifications (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                container_path TEXT,
                classification TEXT NOT NULL,
                custom_label TEXT,
                classified_by TEXT NOT NULL,
                classified_at TEXT NOT NULL,
                notes TEXT,
                confidence TEXT
            );

            -- Extraction log (immutable audit trail of container extractions)
            CREATE TABLE IF NOT EXISTS extraction_log (
                id TEXT PRIMARY KEY,
                container_path TEXT NOT NULL,
                entry_path TEXT NOT NULL,
                output_path TEXT NOT NULL,
                extracted_by TEXT NOT NULL,
                extracted_at TEXT NOT NULL,
                entry_size INTEGER NOT NULL DEFAULT 0,
                purpose TEXT NOT NULL DEFAULT 'preview',
                hash_value TEXT,
                hash_algorithm TEXT,
                status TEXT NOT NULL DEFAULT 'success',
                error TEXT
            );

            -- Viewer history (files viewed during examination)
            CREATE TABLE IF NOT EXISTS viewer_history (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                container_path TEXT,
                viewer_type TEXT NOT NULL,
                viewed_by TEXT NOT NULL,
                opened_at TEXT NOT NULL,
                closed_at TEXT,
                duration_seconds INTEGER
            );

            -- Annotations (hex/document viewer highlights and comments)
            CREATE TABLE IF NOT EXISTS annotations (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                container_path TEXT,
                annotation_type TEXT NOT NULL,
                offset_start INTEGER,
                offset_end INTEGER,
                line_start INTEGER,
                line_end INTEGER,
                label TEXT NOT NULL,
                content TEXT,
                color TEXT,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL
            );

            -- Evidence relationships (links between evidence files)
            CREATE TABLE IF NOT EXISTS evidence_relationships (
                id TEXT PRIMARY KEY,
                source_path TEXT NOT NULL,
                target_path TEXT NOT NULL,
                relationship_type TEXT NOT NULL,
                description TEXT,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            -- =================================================================
            -- Indexes for common query patterns
            -- =================================================================
            CREATE INDEX IF NOT EXISTS idx_activity_timestamp ON activity_log(timestamp);
            CREATE INDEX IF NOT EXISTS idx_activity_category ON activity_log(category);
            CREATE INDEX IF NOT EXISTS idx_activity_user ON activity_log(user);
            CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user);
            CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);
            CREATE INDEX IF NOT EXISTS idx_evidence_type ON evidence_files(container_type);
            CREATE INDEX IF NOT EXISTS idx_evidence_path ON evidence_files(path);
            CREATE INDEX IF NOT EXISTS idx_hashes_file ON hashes(file_id);
            CREATE INDEX IF NOT EXISTS idx_hashes_algorithm ON hashes(algorithm);
            CREATE INDEX IF NOT EXISTS idx_verifications_hash ON verifications(hash_id);
            CREATE INDEX IF NOT EXISTS idx_bookmarks_target ON bookmarks(target_path);
            CREATE INDEX IF NOT EXISTS idx_notes_target ON notes(target_path);
            CREATE INDEX IF NOT EXISTS idx_tag_assignments_target ON tag_assignments(target_type, target_id);
            CREATE INDEX IF NOT EXISTS idx_case_docs_type ON case_documents(document_type);
            CREATE INDEX IF NOT EXISTS idx_processed_db_type ON processed_databases(db_type);
            CREATE INDEX IF NOT EXISTS idx_processed_db_path ON processed_databases(path);
            CREATE INDEX IF NOT EXISTS idx_processed_integrity_db ON processed_db_integrity(processed_db_id);
            CREATE INDEX IF NOT EXISTS idx_processed_metrics_db ON processed_db_metrics(processed_db_id);
            CREATE INDEX IF NOT EXISTS idx_axiom_case_db ON axiom_case_info(processed_db_id);
            CREATE INDEX IF NOT EXISTS idx_axiom_sources_case ON axiom_evidence_sources(axiom_case_id);
            CREATE INDEX IF NOT EXISTS idx_axiom_results_case ON axiom_search_results(axiom_case_id);
            CREATE INDEX IF NOT EXISTS idx_artifact_cats_db ON artifact_categories(processed_db_id);
            -- v3 indexes
            CREATE INDEX IF NOT EXISTS idx_export_status ON export_history(status);
            CREATE INDEX IF NOT EXISTS idx_export_started ON export_history(started_at);
            CREATE INDEX IF NOT EXISTS idx_custody_date ON chain_of_custody(date);
            CREATE INDEX IF NOT EXISTS idx_classification_path ON file_classifications(file_path);
            CREATE INDEX IF NOT EXISTS idx_classification_type ON file_classifications(classification);
            CREATE INDEX IF NOT EXISTS idx_extraction_container ON extraction_log(container_path);
            CREATE INDEX IF NOT EXISTS idx_extraction_at ON extraction_log(extracted_at);
            CREATE INDEX IF NOT EXISTS idx_viewer_path ON viewer_history(file_path);
            CREATE INDEX IF NOT EXISTS idx_viewer_opened ON viewer_history(opened_at);
            CREATE INDEX IF NOT EXISTS idx_annotation_path ON annotations(file_path);
            CREATE INDEX IF NOT EXISTS idx_relationship_source ON evidence_relationships(source_path);
            CREATE INDEX IF NOT EXISTS idx_relationship_target ON evidence_relationships(target_path);
        "#,
        )?;

        // FTS5 virtual tables for full-text search (separate batch — FTS5 uses
        // its own module and some SQLite builds require separate execute calls)
        conn.execute_batch(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS fts_notes USING fts5(
                target_path, title, content, tags,
                content='notes', content_rowid='rowid'
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS fts_bookmarks USING fts5(
                target_path, label, description,
                content='bookmarks', content_rowid='rowid'
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS fts_activity_log USING fts5(
                action, description, file_path, details,
                content='activity_log', content_rowid='rowid'
            );
            "#,
        ).unwrap_or_else(|e| {
            warn!("FTS5 tables could not be created (may not be available in this SQLite build): {}", e);
        });

        // Set schema version if not already set
        let conn2 = &*conn;
        let version: Option<String> = conn2
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .ok();

        if version.is_none() {
            conn2.execute(
                "INSERT INTO schema_meta (key, value) VALUES ('schema_version', ?1)",
                params![SCHEMA_VERSION.to_string()],
            )?;
            conn2.execute(
                "INSERT OR IGNORE INTO schema_meta (key, value) VALUES ('created_at', ?1)",
                params![chrono::Utc::now().to_rfc3339()],
            )?;
            conn2.execute(
                "INSERT OR IGNORE INTO schema_meta (key, value) VALUES ('app_name', ?1)",
                params![APP_NAME],
            )?;
        }

        Ok(())
    }

    /// Run schema migrations if needed (future-proofing)
    fn check_migrations(&self) -> SqlResult<()> {
        let conn = self.conn.lock();
        let current_version: u32 = conn
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'",
                [],
                |row| {
                    let v: String = row.get(0)?;
                    Ok(v.parse::<u32>().unwrap_or(1))
                },
            )
            .unwrap_or(1);

        if current_version < SCHEMA_VERSION {
            info!(
                "Migrating project DB from v{} to v{}",
                current_version, SCHEMA_VERSION
            );

            // v1 → v2: Add processed database tables
            if current_version < 2 {
                info!("Running v1 → v2 migration: adding processed database tables");
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS processed_databases (
                        id TEXT PRIMARY KEY,
                        path TEXT NOT NULL UNIQUE,
                        name TEXT NOT NULL,
                        db_type TEXT NOT NULL DEFAULT 'Unknown',
                        case_number TEXT,
                        examiner TEXT,
                        created_date TEXT,
                        total_size INTEGER NOT NULL DEFAULT 0,
                        artifact_count INTEGER,
                        notes TEXT,
                        registered_at TEXT NOT NULL,
                        metadata_json TEXT
                    );
                    CREATE TABLE IF NOT EXISTS processed_db_integrity (
                        id TEXT PRIMARY KEY,
                        processed_db_id TEXT NOT NULL,
                        file_path TEXT NOT NULL,
                        file_size INTEGER NOT NULL DEFAULT 0,
                        baseline_hash TEXT NOT NULL,
                        baseline_timestamp TEXT NOT NULL,
                        current_hash TEXT,
                        current_hash_timestamp TEXT,
                        status TEXT NOT NULL DEFAULT 'not_verified',
                        changes_json TEXT,
                        FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS processed_db_metrics (
                        id TEXT PRIMARY KEY,
                        processed_db_id TEXT NOT NULL,
                        total_scans INTEGER NOT NULL DEFAULT 0,
                        last_scan_date TEXT,
                        total_jobs INTEGER NOT NULL DEFAULT 0,
                        last_job_date TEXT,
                        total_notes INTEGER NOT NULL DEFAULT 0,
                        total_tagged_items INTEGER NOT NULL DEFAULT 0,
                        total_users INTEGER NOT NULL DEFAULT 0,
                        user_names_json TEXT,
                        captured_at TEXT NOT NULL,
                        FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS axiom_case_info (
                        id TEXT PRIMARY KEY,
                        processed_db_id TEXT NOT NULL,
                        case_name TEXT NOT NULL,
                        case_number TEXT,
                        case_type TEXT,
                        description TEXT,
                        examiner TEXT,
                        agency TEXT,
                        axiom_version TEXT,
                        search_start TEXT,
                        search_end TEXT,
                        search_duration TEXT,
                        search_outcome TEXT,
                        output_folder TEXT,
                        total_artifacts INTEGER NOT NULL DEFAULT 0,
                        case_path TEXT,
                        captured_at TEXT NOT NULL,
                        keyword_info_json TEXT,
                        FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS axiom_evidence_sources (
                        id TEXT PRIMARY KEY,
                        axiom_case_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        evidence_number TEXT,
                        source_type TEXT NOT NULL DEFAULT 'unknown',
                        path TEXT,
                        hash TEXT,
                        size INTEGER,
                        acquired TEXT,
                        search_types_json TEXT,
                        FOREIGN KEY (axiom_case_id) REFERENCES axiom_case_info(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS axiom_search_results (
                        id TEXT PRIMARY KEY,
                        axiom_case_id TEXT NOT NULL,
                        artifact_type TEXT NOT NULL,
                        hit_count INTEGER NOT NULL DEFAULT 0,
                        FOREIGN KEY (axiom_case_id) REFERENCES axiom_case_info(id) ON DELETE CASCADE
                    );
                    CREATE TABLE IF NOT EXISTS artifact_categories (
                        id TEXT PRIMARY KEY,
                        processed_db_id TEXT NOT NULL,
                        category TEXT NOT NULL,
                        artifact_type TEXT NOT NULL,
                        count INTEGER NOT NULL DEFAULT 0,
                        FOREIGN KEY (processed_db_id) REFERENCES processed_databases(id) ON DELETE CASCADE
                    );
                    CREATE INDEX IF NOT EXISTS idx_processed_db_type ON processed_databases(db_type);
                    CREATE INDEX IF NOT EXISTS idx_processed_db_path ON processed_databases(path);
                    CREATE INDEX IF NOT EXISTS idx_processed_integrity_db ON processed_db_integrity(processed_db_id);
                    CREATE INDEX IF NOT EXISTS idx_processed_metrics_db ON processed_db_metrics(processed_db_id);
                    CREATE INDEX IF NOT EXISTS idx_axiom_case_db ON axiom_case_info(processed_db_id);
                    CREATE INDEX IF NOT EXISTS idx_axiom_sources_case ON axiom_evidence_sources(axiom_case_id);
                    CREATE INDEX IF NOT EXISTS idx_axiom_results_case ON axiom_search_results(axiom_case_id);
                    CREATE INDEX IF NOT EXISTS idx_artifact_cats_db ON artifact_categories(processed_db_id);
                    "#,
                )?;
            }

            // v2 → v3: Add forensic workflow tables + FTS5
            if current_version < 3 {
                info!("Running v2 → v3 migration: adding forensic workflow tables");
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS export_history (
                        id TEXT PRIMARY KEY,
                        export_type TEXT NOT NULL,
                        source_paths_json TEXT NOT NULL,
                        destination TEXT NOT NULL,
                        started_at TEXT NOT NULL,
                        completed_at TEXT,
                        initiated_by TEXT NOT NULL,
                        status TEXT NOT NULL DEFAULT 'pending',
                        total_files INTEGER NOT NULL DEFAULT 0,
                        total_bytes INTEGER NOT NULL DEFAULT 0,
                        archive_name TEXT,
                        archive_format TEXT,
                        compression_level TEXT,
                        encrypted INTEGER NOT NULL DEFAULT 0,
                        manifest_hash TEXT,
                        error TEXT,
                        options_json TEXT
                    );
                    CREATE TABLE IF NOT EXISTS chain_of_custody (
                        id TEXT PRIMARY KEY,
                        action TEXT NOT NULL,
                        from_person TEXT NOT NULL,
                        to_person TEXT NOT NULL,
                        date TEXT NOT NULL,
                        time TEXT,
                        location TEXT,
                        purpose TEXT,
                        notes TEXT,
                        evidence_ids_json TEXT,
                        recorded_by TEXT NOT NULL,
                        recorded_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS file_classifications (
                        id TEXT PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        container_path TEXT,
                        classification TEXT NOT NULL,
                        custom_label TEXT,
                        classified_by TEXT NOT NULL,
                        classified_at TEXT NOT NULL,
                        notes TEXT,
                        confidence TEXT
                    );
                    CREATE TABLE IF NOT EXISTS extraction_log (
                        id TEXT PRIMARY KEY,
                        container_path TEXT NOT NULL,
                        entry_path TEXT NOT NULL,
                        output_path TEXT NOT NULL,
                        extracted_by TEXT NOT NULL,
                        extracted_at TEXT NOT NULL,
                        entry_size INTEGER NOT NULL DEFAULT 0,
                        purpose TEXT NOT NULL DEFAULT 'preview',
                        hash_value TEXT,
                        hash_algorithm TEXT,
                        status TEXT NOT NULL DEFAULT 'success',
                        error TEXT
                    );
                    CREATE TABLE IF NOT EXISTS viewer_history (
                        id TEXT PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        container_path TEXT,
                        viewer_type TEXT NOT NULL,
                        viewed_by TEXT NOT NULL,
                        opened_at TEXT NOT NULL,
                        closed_at TEXT,
                        duration_seconds INTEGER
                    );
                    CREATE TABLE IF NOT EXISTS annotations (
                        id TEXT PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        container_path TEXT,
                        annotation_type TEXT NOT NULL,
                        offset_start INTEGER,
                        offset_end INTEGER,
                        line_start INTEGER,
                        line_end INTEGER,
                        label TEXT NOT NULL,
                        content TEXT,
                        color TEXT,
                        created_by TEXT NOT NULL,
                        created_at TEXT NOT NULL,
                        modified_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS evidence_relationships (
                        id TEXT PRIMARY KEY,
                        source_path TEXT NOT NULL,
                        target_path TEXT NOT NULL,
                        relationship_type TEXT NOT NULL,
                        description TEXT,
                        created_by TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_export_status ON export_history(status);
                    CREATE INDEX IF NOT EXISTS idx_export_started ON export_history(started_at);
                    CREATE INDEX IF NOT EXISTS idx_custody_date ON chain_of_custody(date);
                    CREATE INDEX IF NOT EXISTS idx_classification_path ON file_classifications(file_path);
                    CREATE INDEX IF NOT EXISTS idx_classification_type ON file_classifications(classification);
                    CREATE INDEX IF NOT EXISTS idx_extraction_container ON extraction_log(container_path);
                    CREATE INDEX IF NOT EXISTS idx_extraction_at ON extraction_log(extracted_at);
                    CREATE INDEX IF NOT EXISTS idx_viewer_path ON viewer_history(file_path);
                    CREATE INDEX IF NOT EXISTS idx_viewer_opened ON viewer_history(opened_at);
                    CREATE INDEX IF NOT EXISTS idx_annotation_path ON annotations(file_path);
                    CREATE INDEX IF NOT EXISTS idx_relationship_source ON evidence_relationships(source_path);
                    CREATE INDEX IF NOT EXISTS idx_relationship_target ON evidence_relationships(target_path);
                    "#,
                )?;
            }

            // Future migrations go here: if current_version < 4 { ... }

            conn.execute(
                "UPDATE schema_meta SET value = ?1 WHERE key = 'schema_version'",
                params![SCHEMA_VERSION.to_string()],
            )?;
        }

        Ok(())
    }

    // ========================================================================
    // Activity Log Operations
    // ========================================================================

    /// Insert a new activity log entry
    pub fn insert_activity(&self, entry: &DbActivityEntry) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO activity_log (id, timestamp, user, category, action, description, file_path, details)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id,
                entry.timestamp,
                entry.user,
                entry.category,
                entry.action,
                entry.description,
                entry.file_path,
                entry.details,
            ],
        )?;
        Ok(())
    }

    /// Query activity log with filters
    pub fn query_activities(&self, query: &ActivityQuery) -> SqlResult<Vec<DbActivityEntry>> {
        let conn = self.conn.lock();

        let mut sql = String::from(
            "SELECT id, timestamp, user, category, action, description, file_path, details
             FROM activity_log WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref cat) = query.category {
            param_values.push(Box::new(cat.clone()));
            sql.push_str(&format!(" AND category = ?{}", param_values.len()));
        }
        if let Some(ref user) = query.user {
            param_values.push(Box::new(user.clone()));
            sql.push_str(&format!(" AND user = ?{}", param_values.len()));
        }
        if let Some(ref since) = query.since {
            param_values.push(Box::new(since.clone()));
            sql.push_str(&format!(" AND timestamp >= ?{}", param_values.len()));
        }
        if let Some(ref until) = query.until {
            param_values.push(Box::new(until.clone()));
            sql.push_str(&format!(" AND timestamp <= ?{}", param_values.len()));
        }
        if let Some(ref search) = query.search {
            param_values.push(Box::new(format!("%{}%", search)));
            sql.push_str(&format!(" AND description LIKE ?{}", param_values.len()));
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = query.limit {
            param_values.push(Box::new(limit));
            sql.push_str(&format!(" LIMIT ?{}", param_values.len()));
        }
        if let Some(offset) = query.offset {
            param_values.push(Box::new(offset));
            sql.push_str(&format!(" OFFSET ?{}", param_values.len()));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(DbActivityEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                user: row.get(2)?,
                category: row.get(3)?,
                action: row.get(4)?,
                description: row.get(5)?,
                file_path: row.get(6)?,
                details: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    /// Get total activity count (for pagination)
    pub fn count_activities(&self, category: Option<&str>) -> SqlResult<i64> {
        let conn = self.conn.lock();
        if let Some(cat) = category {
            conn.query_row(
                "SELECT COUNT(*) FROM activity_log WHERE category = ?1",
                params![cat],
                |row| row.get(0),
            )
        } else {
            conn.query_row("SELECT COUNT(*) FROM activity_log", [], |row| row.get(0))
        }
    }

    // ========================================================================
    // Session Operations
    // ========================================================================

    /// Insert or update a session
    pub fn upsert_session(&self, session: &DbProjectSession) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO sessions (session_id, user, started_at, ended_at, duration_seconds, hostname, app_version, summary)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(session_id) DO UPDATE SET
                ended_at = excluded.ended_at,
                duration_seconds = excluded.duration_seconds,
                summary = excluded.summary",
            params![
                session.session_id,
                session.user,
                session.started_at,
                session.ended_at,
                session.duration_seconds,
                session.hostname,
                session.app_version,
                session.summary,
            ],
        )?;
        Ok(())
    }

    /// Get all sessions ordered by start time
    pub fn get_sessions(&self) -> SqlResult<Vec<DbProjectSession>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT session_id, user, started_at, ended_at, duration_seconds, hostname, app_version, summary
             FROM sessions ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbProjectSession {
                session_id: row.get(0)?,
                user: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                duration_seconds: row.get(4)?,
                hostname: row.get(5)?,
                app_version: row.get(6)?,
                summary: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    /// End a session (set ended_at and duration)
    pub fn end_session(&self, session_id: &str, summary: Option<&str>) -> SqlResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock();

        // Calculate duration from started_at
        let started: String = conn.query_row(
            "SELECT started_at FROM sessions WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        let duration = if let (Ok(start), Ok(end)) = (
            chrono::DateTime::parse_from_rfc3339(&started),
            chrono::DateTime::parse_from_rfc3339(&now),
        ) {
            Some((end - start).num_seconds())
        } else {
            None
        };

        conn.execute(
            "UPDATE sessions SET ended_at = ?1, duration_seconds = ?2, summary = COALESCE(?3, summary) WHERE session_id = ?4",
            params![now, duration, summary, session_id],
        )?;
        Ok(())
    }

    // ========================================================================
    // User Operations
    // ========================================================================

    /// Insert or update a user
    pub fn upsert_user(&self, user: &DbProjectUser) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO users (username, display_name, hostname, first_access, last_access)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(username) DO UPDATE SET
                display_name = COALESCE(excluded.display_name, users.display_name),
                hostname = COALESCE(excluded.hostname, users.hostname),
                last_access = excluded.last_access",
            params![
                user.username,
                user.display_name,
                user.hostname,
                user.first_access,
                user.last_access,
            ],
        )?;
        Ok(())
    }

    /// Get all users
    pub fn get_users(&self) -> SqlResult<Vec<DbProjectUser>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT username, display_name, hostname, first_access, last_access FROM users ORDER BY last_access DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbProjectUser {
                username: row.get(0)?,
                display_name: row.get(1)?,
                hostname: row.get(2)?,
                first_access: row.get(3)?,
                last_access: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Evidence File Operations
    // ========================================================================

    /// Insert or update an evidence file
    pub fn upsert_evidence_file(&self, file: &DbEvidenceFile) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO evidence_files (id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(path) DO UPDATE SET
                filename = excluded.filename,
                container_type = excluded.container_type,
                total_size = excluded.total_size,
                segment_count = excluded.segment_count,
                created = COALESCE(excluded.created, evidence_files.created),
                modified = COALESCE(excluded.modified, evidence_files.modified)",
            params![
                file.id,
                file.path,
                file.filename,
                file.container_type,
                file.total_size,
                file.segment_count,
                file.discovered_at,
                file.created,
                file.modified,
            ],
        )?;
        Ok(())
    }

    /// Get all evidence files
    pub fn get_evidence_files(&self) -> SqlResult<Vec<DbEvidenceFile>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified
             FROM evidence_files ORDER BY filename",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbEvidenceFile {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                container_type: row.get(3)?,
                total_size: row.get(4)?,
                segment_count: row.get(5)?,
                discovered_at: row.get(6)?,
                created: row.get(7)?,
                modified: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    /// Get an evidence file by path
    pub fn get_evidence_file_by_path(&self, path: &str) -> SqlResult<Option<DbEvidenceFile>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified
             FROM evidence_files WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbEvidenceFile {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                container_type: row.get(3)?,
                total_size: row.get(4)?,
                segment_count: row.get(5)?,
                discovered_at: row.get(6)?,
                created: row.get(7)?,
                modified: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // Hash Operations
    // ========================================================================

    /// Insert a hash record (immutable — no updates)
    pub fn insert_hash(&self, hash: &DbProjectHash) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO hashes (id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                hash.id,
                hash.file_id,
                hash.algorithm,
                hash.hash_value,
                hash.computed_at,
                hash.segment_index,
                hash.segment_name,
                hash.source,
            ],
        )?;
        Ok(())
    }

    /// Get all hashes for an evidence file
    pub fn get_hashes_for_file(&self, file_id: &str) -> SqlResult<Vec<DbProjectHash>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE file_id = ?1 ORDER BY computed_at DESC",
        )?;

        let rows = stmt.query_map(params![file_id], |row| {
            Ok(DbProjectHash {
                id: row.get(0)?,
                file_id: row.get(1)?,
                algorithm: row.get(2)?,
                hash_value: row.get(3)?,
                computed_at: row.get(4)?,
                segment_index: row.get(5)?,
                segment_name: row.get(6)?,
                source: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    /// Get the latest hash for a file/algorithm combination
    pub fn get_latest_hash(
        &self,
        file_id: &str,
        algorithm: &str,
    ) -> SqlResult<Option<DbProjectHash>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE file_id = ?1 AND algorithm = ?2 AND segment_index IS NULL
             ORDER BY computed_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query(params![file_id, algorithm])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbProjectHash {
                id: row.get(0)?,
                file_id: row.get(1)?,
                algorithm: row.get(2)?,
                hash_value: row.get(3)?,
                computed_at: row.get(4)?,
                segment_index: row.get(5)?,
                segment_name: row.get(6)?,
                source: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Look up latest known hash for a file by path
    pub fn lookup_hash_by_path(&self, path: &str, algorithm: &str) -> SqlResult<Option<(String, String)>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT h.hash_value, h.source
             FROM hashes h
             INNER JOIN evidence_files f ON h.file_id = f.id
             WHERE f.path = ?1 AND h.algorithm = ?2 AND h.segment_index IS NULL
             ORDER BY h.computed_at DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![path, algorithm])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?)))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // Verification Operations
    // ========================================================================

    /// Insert a verification record (immutable)
    pub fn insert_verification(&self, v: &DbProjectVerification) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO verifications (id, hash_id, verified_at, result, expected_hash, actual_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![v.id, v.hash_id, v.verified_at, v.result, v.expected_hash, v.actual_hash],
        )?;
        Ok(())
    }

    /// Get verifications for a specific hash
    pub fn get_verifications_for_hash(&self, hash_id: &str) -> SqlResult<Vec<DbProjectVerification>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, hash_id, verified_at, result, expected_hash, actual_hash
             FROM verifications WHERE hash_id = ?1 ORDER BY verified_at DESC",
        )?;

        let rows = stmt.query_map(params![hash_id], |row| {
            Ok(DbProjectVerification {
                id: row.get(0)?,
                hash_id: row.get(1)?,
                verified_at: row.get(2)?,
                result: row.get(3)?,
                expected_hash: row.get(4)?,
                actual_hash: row.get(5)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Bookmark Operations
    // ========================================================================

    /// Insert or update a bookmark
    pub fn upsert_bookmark(&self, b: &DbBookmark) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO bookmarks (id, target_type, target_path, name, created_by, created_at, color, notes, context)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                color = excluded.color,
                notes = excluded.notes,
                context = excluded.context",
            params![
                b.id, b.target_type, b.target_path, b.name,
                b.created_by, b.created_at, b.color, b.notes, b.context,
            ],
        )?;
        Ok(())
    }

    /// Get all bookmarks
    pub fn get_bookmarks(&self) -> SqlResult<Vec<DbBookmark>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, target_type, target_path, name, created_by, created_at, color, notes, context
             FROM bookmarks ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbBookmark {
                id: row.get(0)?,
                target_type: row.get(1)?,
                target_path: row.get(2)?,
                name: row.get(3)?,
                created_by: row.get(4)?,
                created_at: row.get(5)?,
                color: row.get(6)?,
                notes: row.get(7)?,
                context: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    /// Delete a bookmark by ID
    pub fn delete_bookmark(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        // Also remove tag assignments for this bookmark
        conn.execute(
            "DELETE FROM tag_assignments WHERE target_type = 'bookmark' AND target_id = ?1",
            params![id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Note Operations
    // ========================================================================

    /// Insert or update a note
    pub fn upsert_note(&self, n: &DbNote) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO notes (id, target_type, target_path, title, content, created_by, created_at, modified_at, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                modified_at = excluded.modified_at,
                priority = excluded.priority",
            params![
                n.id, n.target_type, n.target_path, n.title, n.content,
                n.created_by, n.created_at, n.modified_at, n.priority,
            ],
        )?;
        Ok(())
    }

    /// Get all notes
    pub fn get_notes(&self) -> SqlResult<Vec<DbNote>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, target_type, target_path, title, content, created_by, created_at, modified_at, priority
             FROM notes ORDER BY modified_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbNote {
                id: row.get(0)?,
                target_type: row.get(1)?,
                target_path: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
                modified_at: row.get(7)?,
                priority: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    /// Delete a note by ID
    pub fn delete_note(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        conn.execute(
            "DELETE FROM tag_assignments WHERE target_type = 'note' AND target_id = ?1",
            params![id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Tag Operations
    // ========================================================================

    /// Insert or update a tag definition
    pub fn upsert_tag(&self, t: &DbTag) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO tags (id, name, color, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                color = excluded.color,
                description = excluded.description",
            params![t.id, t.name, t.color, t.description, t.created_at],
        )?;
        Ok(())
    }

    /// Get all tags
    pub fn get_tags(&self) -> SqlResult<Vec<DbTag>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, color, description, created_at FROM tags ORDER BY name",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    /// Assign a tag to a target
    pub fn assign_tag(&self, assignment: &DbTagAssignment) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO tag_assignments (tag_id, target_type, target_id, assigned_at, assigned_by)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                assignment.tag_id,
                assignment.target_type,
                assignment.target_id,
                assignment.assigned_at,
                assignment.assigned_by,
            ],
        )?;
        Ok(())
    }

    /// Remove a tag assignment
    pub fn remove_tag(&self, tag_id: &str, target_type: &str, target_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM tag_assignments WHERE tag_id = ?1 AND target_type = ?2 AND target_id = ?3",
            params![tag_id, target_type, target_id],
        )?;
        Ok(())
    }

    /// Get tags for a specific target
    pub fn get_tags_for_target(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> SqlResult<Vec<DbTag>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.description, t.created_at
             FROM tags t
             INNER JOIN tag_assignments ta ON t.id = ta.tag_id
             WHERE ta.target_type = ?1 AND ta.target_id = ?2
             ORDER BY t.name",
        )?;

        let rows = stmt.query_map(params![target_type, target_id], |row| {
            Ok(DbTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    /// Delete a tag and all its assignments
    pub fn delete_tag(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM tag_assignments WHERE tag_id = ?1", params![id])?;
        conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // Tab Operations (UI State)
    // ========================================================================

    /// Save all tabs (replace all)
    pub fn save_tabs(&self, tabs: &[DbProjectTab]) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM tabs", [])?;
        for tab in tabs {
            conn.execute(
                "INSERT INTO tabs (id, tab_type, file_path, name, subtitle, tab_order, extra)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    tab.id,
                    tab.tab_type,
                    tab.file_path,
                    tab.name,
                    tab.subtitle,
                    tab.tab_order,
                    tab.extra,
                ],
            )?;
        }
        Ok(())
    }

    /// Get all tabs ordered
    pub fn get_tabs(&self) -> SqlResult<Vec<DbProjectTab>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, tab_type, file_path, name, subtitle, tab_order, extra
             FROM tabs ORDER BY tab_order",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbProjectTab {
                id: row.get(0)?,
                tab_type: row.get(1)?,
                file_path: row.get(2)?,
                name: row.get(3)?,
                subtitle: row.get(4)?,
                tab_order: row.get(5)?,
                extra: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // UI State (key-value)
    // ========================================================================

    /// Set a UI state value
    pub fn set_ui_state(&self, key: &str, value: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO ui_state (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get a UI state value
    pub fn get_ui_state(&self, key: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM ui_state WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Get all UI state as a map
    pub fn get_all_ui_state(&self) -> SqlResult<HashMap<String, String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT key, value FROM ui_state")?;
        let mut map = HashMap::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            map.insert(key, value);
        }
        Ok(map)
    }

    // ========================================================================
    // Report Operations
    // ========================================================================

    /// Insert a report record
    pub fn insert_report(&self, r: &DbReportRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO reports (id, title, report_type, format, output_path, generated_at, generated_by, status, error, config)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                r.id, r.title, r.report_type, r.format, r.output_path,
                r.generated_at, r.generated_by, r.status, r.error, r.config,
            ],
        )?;
        Ok(())
    }

    /// Get all reports
    pub fn get_reports(&self) -> SqlResult<Vec<DbReportRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, title, report_type, format, output_path, generated_at, generated_by, status, error, config
             FROM reports ORDER BY generated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbReportRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                report_type: row.get(2)?,
                format: row.get(3)?,
                output_path: row.get(4)?,
                generated_at: row.get(5)?,
                generated_by: row.get(6)?,
                status: row.get(7)?,
                error: row.get(8)?,
                config: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Search Operations
    // ========================================================================

    /// Insert or update a saved search
    pub fn upsert_saved_search(&self, s: &DbSavedSearch) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO saved_searches (id, name, query, search_type, is_regex, case_sensitive, scope, created_at, use_count, last_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                query = excluded.query,
                use_count = excluded.use_count,
                last_used = excluded.last_used",
            params![
                s.id, s.name, s.query, s.search_type,
                s.is_regex as i32, s.case_sensitive as i32,
                s.scope, s.created_at, s.use_count, s.last_used,
            ],
        )?;
        Ok(())
    }

    /// Get all saved searches
    pub fn get_saved_searches(&self) -> SqlResult<Vec<DbSavedSearch>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, query, search_type, is_regex, case_sensitive, scope, created_at, use_count, last_used
             FROM saved_searches ORDER BY use_count DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let is_regex: i32 = row.get(4)?;
            let case_sensitive: i32 = row.get(5)?;
            Ok(DbSavedSearch {
                id: row.get(0)?,
                name: row.get(1)?,
                query: row.get(2)?,
                search_type: row.get(3)?,
                is_regex: is_regex != 0,
                case_sensitive: case_sensitive != 0,
                scope: row.get(6)?,
                created_at: row.get(7)?,
                use_count: row.get(8)?,
                last_used: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Case Document Operations
    // ========================================================================

    /// Insert or update a case document
    pub fn upsert_case_document(&self, d: &DbCaseDocument) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO case_documents (id, path, filename, document_type, size, format, case_number, evidence_id, modified, discovered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(path) DO UPDATE SET
                filename = excluded.filename,
                document_type = excluded.document_type,
                size = excluded.size,
                format = excluded.format,
                case_number = excluded.case_number,
                evidence_id = excluded.evidence_id,
                modified = excluded.modified",
            params![
                d.id, d.path, d.filename, d.document_type, d.size, d.format,
                d.case_number, d.evidence_id, d.modified, d.discovered_at,
            ],
        )?;
        Ok(())
    }

    /// Get all case documents
    pub fn get_case_documents(&self) -> SqlResult<Vec<DbCaseDocument>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, filename, document_type, size, format, case_number, evidence_id, modified, discovered_at
             FROM case_documents ORDER BY filename",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbCaseDocument {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                document_type: row.get(3)?,
                size: row.get(4)?,
                format: row.get(5)?,
                case_number: row.get(6)?,
                evidence_id: row.get(7)?,
                modified: row.get(8)?,
                discovered_at: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Processed Database Operations (read-only snapshots of AXIOM, PA, etc.)
    // ========================================================================

    /// Insert or update a processed database record
    pub fn upsert_processed_database(&self, db: &DbProcessedDatabase) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO processed_databases (id, path, name, db_type, case_number, examiner, created_date, total_size, artifact_count, notes, registered_at, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                db_type = excluded.db_type,
                case_number = excluded.case_number,
                examiner = excluded.examiner,
                created_date = excluded.created_date,
                total_size = excluded.total_size,
                artifact_count = excluded.artifact_count,
                notes = excluded.notes,
                metadata_json = excluded.metadata_json",
            params![
                db.id, db.path, db.name, db.db_type, db.case_number, db.examiner,
                db.created_date, db.total_size, db.artifact_count, db.notes,
                db.registered_at, db.metadata_json,
            ],
        )?;
        Ok(())
    }

    /// Get all processed databases
    pub fn get_processed_databases(&self) -> SqlResult<Vec<DbProcessedDatabase>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, name, db_type, case_number, examiner, created_date, total_size, artifact_count, notes, registered_at, metadata_json
             FROM processed_databases ORDER BY name",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbProcessedDatabase {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                db_type: row.get(3)?,
                case_number: row.get(4)?,
                examiner: row.get(5)?,
                created_date: row.get(6)?,
                total_size: row.get(7)?,
                artifact_count: row.get(8)?,
                notes: row.get(9)?,
                registered_at: row.get(10)?,
                metadata_json: row.get(11)?,
            })
        })?;

        rows.collect()
    }

    /// Get a processed database by path
    pub fn get_processed_database_by_path(&self, path: &str) -> SqlResult<Option<DbProcessedDatabase>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, name, db_type, case_number, examiner, created_date, total_size, artifact_count, notes, registered_at, metadata_json
             FROM processed_databases WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbProcessedDatabase {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                db_type: row.get(3)?,
                case_number: row.get(4)?,
                examiner: row.get(5)?,
                created_date: row.get(6)?,
                total_size: row.get(7)?,
                artifact_count: row.get(8)?,
                notes: row.get(9)?,
                registered_at: row.get(10)?,
                metadata_json: row.get(11)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete a processed database and all related records (cascades)
    pub fn delete_processed_database(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        // Delete child records first (integrity, metrics, axiom info, artifact categories)
        // Cascade should handle it, but be explicit for safety
        conn.execute("DELETE FROM artifact_categories WHERE processed_db_id = ?1", params![id])?;
        conn.execute("DELETE FROM processed_db_metrics WHERE processed_db_id = ?1", params![id])?;
        conn.execute("DELETE FROM processed_db_integrity WHERE processed_db_id = ?1", params![id])?;
        // Axiom children need two-step: find axiom_case_info ids, then delete their children
        let axiom_ids: Vec<String> = {
            let mut stmt = conn.prepare("SELECT id FROM axiom_case_info WHERE processed_db_id = ?1")?;
            let rows = stmt.query_map(params![id], |row| row.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        for axiom_id in &axiom_ids {
            conn.execute("DELETE FROM axiom_evidence_sources WHERE axiom_case_id = ?1", params![axiom_id])?;
            conn.execute("DELETE FROM axiom_search_results WHERE axiom_case_id = ?1", params![axiom_id])?;
        }
        conn.execute("DELETE FROM axiom_case_info WHERE processed_db_id = ?1", params![id])?;
        conn.execute("DELETE FROM processed_databases WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // Processed DB Integrity Operations
    // ========================================================================

    /// Insert or update an integrity record
    pub fn upsert_processed_db_integrity(&self, i: &DbProcessedDbIntegrity) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO processed_db_integrity (id, processed_db_id, file_path, file_size, baseline_hash, baseline_timestamp, current_hash, current_hash_timestamp, status, changes_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                file_size = excluded.file_size,
                current_hash = excluded.current_hash,
                current_hash_timestamp = excluded.current_hash_timestamp,
                status = excluded.status,
                changes_json = excluded.changes_json",
            params![
                i.id, i.processed_db_id, i.file_path, i.file_size,
                i.baseline_hash, i.baseline_timestamp,
                i.current_hash, i.current_hash_timestamp,
                i.status, i.changes_json,
            ],
        )?;
        Ok(())
    }

    /// Get integrity records for a processed database
    pub fn get_processed_db_integrity(&self, processed_db_id: &str) -> SqlResult<Vec<DbProcessedDbIntegrity>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, file_path, file_size, baseline_hash, baseline_timestamp, current_hash, current_hash_timestamp, status, changes_json
             FROM processed_db_integrity WHERE processed_db_id = ?1",
        )?;

        let rows = stmt.query_map(params![processed_db_id], |row| {
            Ok(DbProcessedDbIntegrity {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                file_path: row.get(2)?,
                file_size: row.get(3)?,
                baseline_hash: row.get(4)?,
                baseline_timestamp: row.get(5)?,
                current_hash: row.get(6)?,
                current_hash_timestamp: row.get(7)?,
                status: row.get(8)?,
                changes_json: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Processed DB Metrics Operations
    // ========================================================================

    /// Insert or update metrics for a processed database
    pub fn upsert_processed_db_metrics(&self, m: &DbProcessedDbMetrics) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO processed_db_metrics (id, processed_db_id, total_scans, last_scan_date, total_jobs, last_job_date, total_notes, total_tagged_items, total_users, user_names_json, captured_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                total_scans = excluded.total_scans,
                last_scan_date = excluded.last_scan_date,
                total_jobs = excluded.total_jobs,
                last_job_date = excluded.last_job_date,
                total_notes = excluded.total_notes,
                total_tagged_items = excluded.total_tagged_items,
                total_users = excluded.total_users,
                user_names_json = excluded.user_names_json,
                captured_at = excluded.captured_at",
            params![
                m.id, m.processed_db_id, m.total_scans, m.last_scan_date,
                m.total_jobs, m.last_job_date, m.total_notes, m.total_tagged_items,
                m.total_users, m.user_names_json, m.captured_at,
            ],
        )?;
        Ok(())
    }

    /// Get metrics for a processed database
    pub fn get_processed_db_metrics(&self, processed_db_id: &str) -> SqlResult<Option<DbProcessedDbMetrics>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, total_scans, last_scan_date, total_jobs, last_job_date, total_notes, total_tagged_items, total_users, user_names_json, captured_at
             FROM processed_db_metrics WHERE processed_db_id = ?1
             ORDER BY captured_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query(params![processed_db_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbProcessedDbMetrics {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                total_scans: row.get(2)?,
                last_scan_date: row.get(3)?,
                total_jobs: row.get(4)?,
                last_job_date: row.get(5)?,
                total_notes: row.get(6)?,
                total_tagged_items: row.get(7)?,
                total_users: row.get(8)?,
                user_names_json: row.get(9)?,
                captured_at: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // AXIOM Case Info Operations
    // ========================================================================

    /// Insert or update AXIOM case information
    pub fn upsert_axiom_case_info(&self, a: &DbAxiomCaseInfo) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO axiom_case_info (id, processed_db_id, case_name, case_number, case_type, description, examiner, agency, axiom_version, search_start, search_end, search_duration, search_outcome, output_folder, total_artifacts, case_path, captured_at, keyword_info_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(id) DO UPDATE SET
                case_name = excluded.case_name,
                case_number = excluded.case_number,
                case_type = excluded.case_type,
                description = excluded.description,
                examiner = excluded.examiner,
                agency = excluded.agency,
                axiom_version = excluded.axiom_version,
                search_start = excluded.search_start,
                search_end = excluded.search_end,
                search_duration = excluded.search_duration,
                search_outcome = excluded.search_outcome,
                output_folder = excluded.output_folder,
                total_artifacts = excluded.total_artifacts,
                case_path = excluded.case_path,
                captured_at = excluded.captured_at,
                keyword_info_json = excluded.keyword_info_json",
            params![
                a.id, a.processed_db_id, a.case_name, a.case_number,
                a.case_type, a.description, a.examiner, a.agency,
                a.axiom_version, a.search_start, a.search_end,
                a.search_duration, a.search_outcome, a.output_folder,
                a.total_artifacts, a.case_path, a.captured_at, a.keyword_info_json,
            ],
        )?;
        Ok(())
    }

    /// Get AXIOM case info for a processed database
    pub fn get_axiom_case_info(&self, processed_db_id: &str) -> SqlResult<Option<DbAxiomCaseInfo>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, case_name, case_number, case_type, description, examiner, agency, axiom_version, search_start, search_end, search_duration, search_outcome, output_folder, total_artifacts, case_path, captured_at, keyword_info_json
             FROM axiom_case_info WHERE processed_db_id = ?1
             ORDER BY captured_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query(params![processed_db_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbAxiomCaseInfo {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                case_name: row.get(2)?,
                case_number: row.get(3)?,
                case_type: row.get(4)?,
                description: row.get(5)?,
                examiner: row.get(6)?,
                agency: row.get(7)?,
                axiom_version: row.get(8)?,
                search_start: row.get(9)?,
                search_end: row.get(10)?,
                search_duration: row.get(11)?,
                search_outcome: row.get(12)?,
                output_folder: row.get(13)?,
                total_artifacts: row.get(14)?,
                case_path: row.get(15)?,
                captured_at: row.get(16)?,
                keyword_info_json: row.get(17)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all AXIOM case info records
    pub fn get_all_axiom_case_info(&self) -> SqlResult<Vec<DbAxiomCaseInfo>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, case_name, case_number, case_type, description, examiner, agency, axiom_version, search_start, search_end, search_duration, search_outcome, output_folder, total_artifacts, case_path, captured_at, keyword_info_json
             FROM axiom_case_info ORDER BY captured_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbAxiomCaseInfo {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                case_name: row.get(2)?,
                case_number: row.get(3)?,
                case_type: row.get(4)?,
                description: row.get(5)?,
                examiner: row.get(6)?,
                agency: row.get(7)?,
                axiom_version: row.get(8)?,
                search_start: row.get(9)?,
                search_end: row.get(10)?,
                search_duration: row.get(11)?,
                search_outcome: row.get(12)?,
                output_folder: row.get(13)?,
                total_artifacts: row.get(14)?,
                case_path: row.get(15)?,
                captured_at: row.get(16)?,
                keyword_info_json: row.get(17)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // AXIOM Evidence Source Operations
    // ========================================================================

    /// Insert an AXIOM evidence source
    pub fn insert_axiom_evidence_source(&self, s: &DbAxiomEvidenceSource) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO axiom_evidence_sources (id, axiom_case_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                s.id, s.axiom_case_id, s.name, s.evidence_number,
                s.source_type, s.path, s.hash, s.size, s.acquired, s.search_types_json,
            ],
        )?;
        Ok(())
    }

    /// Get evidence sources for an AXIOM case
    pub fn get_axiom_evidence_sources(&self, axiom_case_id: &str) -> SqlResult<Vec<DbAxiomEvidenceSource>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, axiom_case_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json
             FROM axiom_evidence_sources WHERE axiom_case_id = ?1",
        )?;

        let rows = stmt.query_map(params![axiom_case_id], |row| {
            Ok(DbAxiomEvidenceSource {
                id: row.get(0)?,
                axiom_case_id: row.get(1)?,
                name: row.get(2)?,
                evidence_number: row.get(3)?,
                source_type: row.get(4)?,
                path: row.get(5)?,
                hash: row.get(6)?,
                size: row.get(7)?,
                acquired: row.get(8)?,
                search_types_json: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // AXIOM Search Result Operations
    // ========================================================================

    /// Insert an AXIOM search result
    pub fn insert_axiom_search_result(&self, r: &DbAxiomSearchResult) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO axiom_search_results (id, axiom_case_id, artifact_type, hit_count)
             VALUES (?1, ?2, ?3, ?4)",
            params![r.id, r.axiom_case_id, r.artifact_type, r.hit_count],
        )?;
        Ok(())
    }

    /// Get search results for an AXIOM case
    pub fn get_axiom_search_results(&self, axiom_case_id: &str) -> SqlResult<Vec<DbAxiomSearchResult>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, axiom_case_id, artifact_type, hit_count
             FROM axiom_search_results WHERE axiom_case_id = ?1 ORDER BY hit_count DESC",
        )?;

        let rows = stmt.query_map(params![axiom_case_id], |row| {
            Ok(DbAxiomSearchResult {
                id: row.get(0)?,
                axiom_case_id: row.get(1)?,
                artifact_type: row.get(2)?,
                hit_count: row.get(3)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Artifact Category Operations
    // ========================================================================

    /// Insert or replace artifact categories for a processed database
    pub fn upsert_artifact_categories(&self, categories: &[DbArtifactCategory]) -> SqlResult<()> {
        let conn = self.conn.lock();
        for c in categories {
            conn.execute(
                "INSERT INTO artifact_categories (id, processed_db_id, category, artifact_type, count)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                    count = excluded.count",
                params![c.id, c.processed_db_id, c.category, c.artifact_type, c.count],
            )?;
        }
        Ok(())
    }

    /// Get artifact categories for a processed database
    pub fn get_artifact_categories(&self, processed_db_id: &str) -> SqlResult<Vec<DbArtifactCategory>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, category, artifact_type, count
             FROM artifact_categories WHERE processed_db_id = ?1 ORDER BY category, artifact_type",
        )?;

        let rows = stmt.query_map(params![processed_db_id], |row| {
            Ok(DbArtifactCategory {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                category: row.get(2)?,
                artifact_type: row.get(3)?,
                count: row.get(4)?,
            })
        })?;

        rows.collect()
    }

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
    // File Classification Operations
    // ========================================================================

    /// Insert or update a file classification
    pub fn upsert_classification(&self, record: &DbFileClassification) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO file_classifications (id, file_path, container_path, classification, custom_label, classified_by, classified_at, notes, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET classification = excluded.classification, custom_label = excluded.custom_label, notes = excluded.notes, confidence = excluded.confidence, classified_at = excluded.classified_at",
            params![
                record.id, record.file_path, record.container_path, record.classification,
                record.custom_label, record.classified_by, record.classified_at,
                record.notes, record.confidence,
            ],
        )?;
        Ok(())
    }

    /// Get classifications for a file path
    pub fn get_classifications_for_path(&self, file_path: &str) -> SqlResult<Vec<DbFileClassification>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, classification, custom_label, classified_by, classified_at, notes, confidence
             FROM file_classifications WHERE file_path = ?1 ORDER BY classified_at DESC",
        )?;
        let rows = stmt.query_map(params![file_path], |row| {
            Ok(DbFileClassification {
                id: row.get(0)?,
                file_path: row.get(1)?,
                container_path: row.get(2)?,
                classification: row.get(3)?,
                custom_label: row.get(4)?,
                classified_by: row.get(5)?,
                classified_at: row.get(6)?,
                notes: row.get(7)?,
                confidence: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    /// Get all classifications grouped by type
    pub fn get_all_classifications(&self) -> SqlResult<Vec<DbFileClassification>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, classification, custom_label, classified_by, classified_at, notes, confidence
             FROM file_classifications ORDER BY classification, classified_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DbFileClassification {
                id: row.get(0)?,
                file_path: row.get(1)?,
                container_path: row.get(2)?,
                classification: row.get(3)?,
                custom_label: row.get(4)?,
                classified_by: row.get(5)?,
                classified_at: row.get(6)?,
                notes: row.get(7)?,
                confidence: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    /// Delete a classification by ID
    pub fn delete_classification(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM file_classifications WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // Extraction Log Operations
    // ========================================================================

    /// Insert an extraction log entry
    pub fn insert_extraction(&self, record: &DbExtractionRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO extraction_log (id, container_path, entry_path, output_path, extracted_by, extracted_at, entry_size, purpose, hash_value, hash_algorithm, status, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.id, record.container_path, record.entry_path, record.output_path,
                record.extracted_by, record.extracted_at, record.entry_size, record.purpose,
                record.hash_value, record.hash_algorithm, record.status, record.error,
            ],
        )?;
        Ok(())
    }

    /// Get extraction log entries for a container
    pub fn get_extractions_for_container(&self, container_path: &str) -> SqlResult<Vec<DbExtractionRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, container_path, entry_path, output_path, extracted_by, extracted_at, entry_size, purpose, hash_value, hash_algorithm, status, error
             FROM extraction_log WHERE container_path = ?1 ORDER BY extracted_at DESC",
        )?;
        let rows = stmt.query_map(params![container_path], |row| {
            Ok(DbExtractionRecord {
                id: row.get(0)?,
                container_path: row.get(1)?,
                entry_path: row.get(2)?,
                output_path: row.get(3)?,
                extracted_by: row.get(4)?,
                extracted_at: row.get(5)?,
                entry_size: row.get(6)?,
                purpose: row.get(7)?,
                hash_value: row.get(8)?,
                hash_algorithm: row.get(9)?,
                status: row.get(10)?,
                error: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    /// Get all extraction records, most recent first
    pub fn get_all_extractions(&self, limit: Option<i64>) -> SqlResult<Vec<DbExtractionRecord>> {
        let conn = self.conn.lock();
        let sql = format!(
            "SELECT id, container_path, entry_path, output_path, extracted_by, extracted_at, entry_size, purpose, hash_value, hash_algorithm, status, error
             FROM extraction_log ORDER BY extracted_at DESC{}",
            limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default()
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(DbExtractionRecord {
                id: row.get(0)?,
                container_path: row.get(1)?,
                entry_path: row.get(2)?,
                output_path: row.get(3)?,
                extracted_by: row.get(4)?,
                extracted_at: row.get(5)?,
                entry_size: row.get(6)?,
                purpose: row.get(7)?,
                hash_value: row.get(8)?,
                hash_algorithm: row.get(9)?,
                status: row.get(10)?,
                error: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    // ========================================================================
    // Viewer History Operations
    // ========================================================================

    /// Insert a viewer history entry
    pub fn insert_viewer_history(&self, entry: &DbViewerHistoryEntry) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO viewer_history (id, file_path, container_path, viewer_type, viewed_by, opened_at, closed_at, duration_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id, entry.file_path, entry.container_path, entry.viewer_type,
                entry.viewed_by, entry.opened_at, entry.closed_at, entry.duration_seconds,
            ],
        )?;
        Ok(())
    }

    /// Update viewer history (set closed_at and duration)
    pub fn update_viewer_history_close(&self, id: &str, closed_at: &str, duration_seconds: Option<i64>) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE viewer_history SET closed_at = ?1, duration_seconds = ?2 WHERE id = ?3",
            params![closed_at, duration_seconds, id],
        )?;
        Ok(())
    }

    /// Get recent viewer history
    pub fn get_viewer_history(&self, limit: Option<i64>) -> SqlResult<Vec<DbViewerHistoryEntry>> {
        let conn = self.conn.lock();
        let sql = format!(
            "SELECT id, file_path, container_path, viewer_type, viewed_by, opened_at, closed_at, duration_seconds
             FROM viewer_history ORDER BY opened_at DESC{}",
            limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default()
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(DbViewerHistoryEntry {
                id: row.get(0)?,
                file_path: row.get(1)?,
                container_path: row.get(2)?,
                viewer_type: row.get(3)?,
                viewed_by: row.get(4)?,
                opened_at: row.get(5)?,
                closed_at: row.get(6)?,
                duration_seconds: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    // ========================================================================
    // Annotation Operations
    // ========================================================================

    /// Insert a new annotation
    pub fn insert_annotation(&self, ann: &DbAnnotation) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO annotations (id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                ann.id, ann.file_path, ann.container_path, ann.annotation_type,
                ann.offset_start, ann.offset_end, ann.line_start, ann.line_end,
                ann.label, ann.content, ann.color, ann.created_by,
                ann.created_at, ann.modified_at,
            ],
        )?;
        Ok(())
    }

    /// Update an annotation
    pub fn update_annotation(&self, ann: &DbAnnotation) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE annotations SET label = ?1, content = ?2, color = ?3, modified_at = ?4 WHERE id = ?5",
            params![ann.label, ann.content, ann.color, ann.modified_at, ann.id],
        )?;
        Ok(())
    }

    /// Get annotations for a file
    pub fn get_annotations_for_path(&self, file_path: &str) -> SqlResult<Vec<DbAnnotation>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at
             FROM annotations WHERE file_path = ?1 ORDER BY COALESCE(offset_start, line_start, 0)",
        )?;
        let rows = stmt.query_map(params![file_path], |row| {
            Ok(DbAnnotation {
                id: row.get(0)?,
                file_path: row.get(1)?,
                container_path: row.get(2)?,
                annotation_type: row.get(3)?,
                offset_start: row.get(4)?,
                offset_end: row.get(5)?,
                line_start: row.get(6)?,
                line_end: row.get(7)?,
                label: row.get(8)?,
                content: row.get(9)?,
                color: row.get(10)?,
                created_by: row.get(11)?,
                created_at: row.get(12)?,
                modified_at: row.get(13)?,
            })
        })?;
        rows.collect()
    }

    /// Get all annotations
    pub fn get_all_annotations(&self) -> SqlResult<Vec<DbAnnotation>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, container_path, annotation_type, offset_start, offset_end, line_start, line_end, label, content, color, created_by, created_at, modified_at
             FROM annotations ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DbAnnotation {
                id: row.get(0)?,
                file_path: row.get(1)?,
                container_path: row.get(2)?,
                annotation_type: row.get(3)?,
                offset_start: row.get(4)?,
                offset_end: row.get(5)?,
                line_start: row.get(6)?,
                line_end: row.get(7)?,
                label: row.get(8)?,
                content: row.get(9)?,
                color: row.get(10)?,
                created_by: row.get(11)?,
                created_at: row.get(12)?,
                modified_at: row.get(13)?,
            })
        })?;
        rows.collect()
    }

    /// Delete an annotation by ID
    pub fn delete_annotation(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM annotations WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // Evidence Relationship Operations
    // ========================================================================

    /// Insert an evidence relationship
    pub fn insert_relationship(&self, rel: &DbEvidenceRelationship) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO evidence_relationships (id, source_path, target_path, relationship_type, description, created_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                rel.id, rel.source_path, rel.target_path, rel.relationship_type,
                rel.description, rel.created_by, rel.created_at,
            ],
        )?;
        Ok(())
    }

    /// Get relationships involving a path (as source or target)
    pub fn get_relationships_for_path(&self, path: &str) -> SqlResult<Vec<DbEvidenceRelationship>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, source_path, target_path, relationship_type, description, created_by, created_at
             FROM evidence_relationships WHERE source_path = ?1 OR target_path = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![path], |row| {
            Ok(DbEvidenceRelationship {
                id: row.get(0)?,
                source_path: row.get(1)?,
                target_path: row.get(2)?,
                relationship_type: row.get(3)?,
                description: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    /// Get all relationships
    pub fn get_all_relationships(&self) -> SqlResult<Vec<DbEvidenceRelationship>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, source_path, target_path, relationship_type, description, created_by, created_at
             FROM evidence_relationships ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DbEvidenceRelationship {
                id: row.get(0)?,
                source_path: row.get(1)?,
                target_path: row.get(2)?,
                relationship_type: row.get(3)?,
                description: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    /// Delete a relationship by ID
    pub fn delete_relationship(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM evidence_relationships WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // Full-Text Search (FTS5)
    // ========================================================================

    /// Rebuild FTS indexes by re-populating from source tables
    pub fn rebuild_fts_indexes(&self) -> SqlResult<()> {
        let conn = self.conn.lock();
        // Only rebuild if the FTS tables exist
        let has_fts: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='fts_notes'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if !has_fts {
            return Ok(());
        }

        conn.execute_batch(
            r#"
            DELETE FROM fts_notes;
            INSERT INTO fts_notes(rowid, target_path, title, content, tags)
                SELECT rowid, target_path, COALESCE(title,''), COALESCE(content,''), COALESCE(tags,'') FROM notes;
            DELETE FROM fts_bookmarks;
            INSERT INTO fts_bookmarks(rowid, target_path, label, description)
                SELECT rowid, target_path, COALESCE(label,''), COALESCE(description,'') FROM bookmarks;
            DELETE FROM fts_activity_log;
            INSERT INTO fts_activity_log(rowid, action, description, file_path, details)
                SELECT rowid, COALESCE(action,''), COALESCE(description,''), COALESCE(file_path,''), COALESCE(details,'') FROM activity_log;
            "#,
        )?;
        Ok(())
    }

    /// Full-text search across notes, bookmarks, and activity log
    pub fn fts_search(&self, query: &str, limit: Option<i64>) -> SqlResult<Vec<FtsSearchResult>> {
        let conn = self.conn.lock();
        let max = limit.unwrap_or(50);
        let mut results = Vec::new();

        // Check FTS availability
        let has_fts: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='fts_notes'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if !has_fts {
            return Ok(results);
        }

        // Search notes
        if let Ok(mut stmt) = conn.prepare(
            "SELECT target_path, title, snippet(fts_notes, 2, '<mark>', '</mark>', '...', 32), rank
             FROM fts_notes WHERE fts_notes MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "notes".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            }) {
                for r in rows.flatten() {
                    results.push(r);
                }
            }
        }

        // Search bookmarks
        if let Ok(mut stmt) = conn.prepare(
            "SELECT target_path, label, snippet(fts_bookmarks, 2, '<mark>', '</mark>', '...', 32), rank
             FROM fts_bookmarks WHERE fts_bookmarks MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "bookmarks".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            }) {
                for r in rows.flatten() {
                    results.push(r);
                }
            }
        }

        // Search activity log
        if let Ok(mut stmt) = conn.prepare(
            "SELECT action, description, snippet(fts_activity_log, 3, '<mark>', '</mark>', '...', 32), rank
             FROM fts_activity_log WHERE fts_activity_log MATCH ?1 ORDER BY rank LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![query, max], |row| {
                Ok(FtsSearchResult {
                    source: "activity_log".to_string(),
                    id: row.get::<_, String>(0)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            }) {
                for r in rows.flatten() {
                    results.push(r);
                }
            }
        }

        // Sort by rank (ascending = better match)
        results.sort_by(|a, b| a.rank.partial_cmp(&b.rank).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max as usize);

        Ok(results)
    }

    // ========================================================================
    // Database Utilities
    // ========================================================================

    /// Run a SQLite integrity check
    pub fn integrity_check(&self) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("PRAGMA integrity_check")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    /// Force a WAL checkpoint (flush WAL to main DB file)
    pub fn wal_checkpoint(&self) -> SqlResult<(i64, i64)> {
        let conn = self.conn.lock();
        let (log_size, frames_checkpointed): (i64, i64) = conn.query_row(
            "PRAGMA wal_checkpoint(TRUNCATE)",
            [],
            |row| Ok((row.get(1)?, row.get(2)?)),
        )?;
        Ok((log_size, frames_checkpointed))
    }

    /// Create a backup copy of the database
    pub fn backup_to(&self, dest_path: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        let mut dest = rusqlite::Connection::open(dest_path)?;
        let backup = rusqlite::backup::Backup::new(&conn, &mut dest)?;
        backup.run_to_completion(100, std::time::Duration::from_millis(50), None)?;
        Ok(())
    }

    /// Vacuum the database to reclaim space
    pub fn vacuum(&self) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute_batch("VACUUM")?;
        Ok(())
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get project database statistics
    pub fn get_stats(&self) -> SqlResult<ProjectDbStats> {
        let conn = self.conn.lock();

        let count = |table: &str| -> SqlResult<i64> {
            conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
                row.get(0)
            })
        };

        let db_size = std::fs::metadata(&self.path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ProjectDbStats {
            total_activities: count("activity_log")?,
            total_sessions: count("sessions")?,
            total_users: count("users")?,
            total_evidence_files: count("evidence_files")?,
            total_hashes: count("hashes")?,
            total_verifications: count("verifications")?,
            total_bookmarks: count("bookmarks")?,
            total_notes: count("notes")?,
            total_tags: count("tags")?,
            total_reports: count("reports")?,
            total_saved_searches: count("saved_searches")?,
            total_case_documents: count("case_documents")?,
            total_processed_databases: count("processed_databases")?,
            total_axiom_cases: count("axiom_case_info")?,
            total_artifact_categories: count("artifact_categories")?,
            total_exports: count("export_history")?,
            total_custody_records: count("chain_of_custody")?,
            total_classifications: count("file_classifications")?,
            total_extractions: count("extraction_log")?,
            total_viewer_history: count("viewer_history")?,
            total_annotations: count("annotations")?,
            total_relationships: count("evidence_relationships")?,
            db_size_bytes: db_size,
            schema_version: SCHEMA_VERSION,
        })
    }

    // ========================================================================
    // Migration from FFXProject (one-time import from .cffx)
    // ========================================================================

    /// Migrate data from an FFXProject struct into this database.
    /// Used when opening a project that has data in the .cffx but no .ffxdb yet.
    /// This is idempotent — it uses INSERT OR IGNORE to avoid duplicates.
    pub fn migrate_from_project(&self, project: &crate::project::FFXProject) -> SqlResult<()> {
        info!(
            "Migrating project '{}' data to .ffxdb ({} activities, {} sessions, {} users)",
            project.name,
            project.activity_log.len(),
            project.sessions.len(),
            project.users.len(),
        );

        let conn = self.conn.lock();

        // --- Users ---
        for u in &project.users {
            conn.execute(
                "INSERT OR IGNORE INTO users (username, display_name, hostname, first_access, last_access)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![u.username, u.display_name, u.hostname, u.first_access, u.last_access],
            )?;
        }

        // --- Sessions ---
        for s in &project.sessions {
            conn.execute(
                "INSERT OR IGNORE INTO sessions (session_id, user, started_at, ended_at, duration_seconds, hostname, app_version, summary)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    s.session_id, s.user, s.started_at, s.ended_at,
                    s.duration_seconds.map(|d| d as i64),
                    s.hostname, s.app_version, s.summary,
                ],
            )?;
        }

        // --- Activity Log ---
        for a in &project.activity_log {
            let details_json = a
                .details
                .as_ref()
                .and_then(|d| serde_json::to_string(d).ok());
            conn.execute(
                "INSERT OR IGNORE INTO activity_log (id, timestamp, user, category, action, description, file_path, details)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    a.id, a.timestamp, a.user, a.category, a.action,
                    a.description, a.file_path, details_json,
                ],
            )?;
        }

        // --- Bookmarks ---
        for b in &project.bookmarks {
            let context_json = b
                .context
                .as_ref()
                .and_then(|c| serde_json::to_string(c).ok());
            conn.execute(
                "INSERT OR IGNORE INTO bookmarks (id, target_type, target_path, name, created_by, created_at, color, notes, context)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    b.id, b.target_type, b.target_path, b.name,
                    b.created_by, b.created_at, b.color, b.notes, context_json,
                ],
            )?;
        }

        // --- Notes ---
        for n in &project.notes {
            conn.execute(
                "INSERT OR IGNORE INTO notes (id, target_type, target_path, title, content, created_by, created_at, modified_at, priority)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    n.id, n.target_type, n.target_path, n.title, n.content,
                    n.created_by, n.created_at, n.modified_at, n.priority,
                ],
            )?;
        }

        // --- Tags ---
        for t in &project.tags {
            conn.execute(
                "INSERT OR IGNORE INTO tags (id, name, color, description, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![t.id, t.name, t.color, t.description, t.created_at],
            )?;
        }

        // --- Reports ---
        for r in &project.reports {
            let config_json = r
                .config
                .as_ref()
                .and_then(|c| serde_json::to_string(c).ok());
            conn.execute(
                "INSERT OR IGNORE INTO reports (id, title, report_type, format, output_path, generated_at, generated_by, status, error, config)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    r.id, r.title, r.report_type, r.format, r.output_path,
                    r.generated_at, r.generated_by, r.status, r.error, config_json,
                ],
            )?;
        }

        // --- Saved Searches ---
        for s in &project.saved_searches {
            conn.execute(
                "INSERT OR IGNORE INTO saved_searches (id, name, query, search_type, is_regex, case_sensitive, scope, created_at, use_count, last_used)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    s.id, s.name, s.query, s.search_type,
                    s.is_regex as i32, s.case_sensitive as i32,
                    s.scope, s.created_at, s.use_count, s.last_used,
                ],
            )?;
        }

        // --- Evidence Files from cache ---
        if let Some(ref cache) = project.evidence_cache {
            for f in &cache.discovered_files {
                let id = format!(
                    "ev_{}",
                    f.path
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .take(16)
                        .collect::<String>()
                );
                conn.execute(
                    "INSERT OR IGNORE INTO evidence_files (id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        id, f.path, f.filename, f.container_type, f.size as i64,
                        f.segment_count as i32,
                        cache.cached_at.clone(),
                        f.created, f.modified,
                    ],
                )?;
            }
        }

        // Record migration timestamp
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('migrated_from_cffx', ?1)",
            params![chrono::Utc::now().to_rfc3339()],
        )?;

        // --- Processed Databases ---
        let now_str = chrono::Utc::now().to_rfc3339();
        let pd_state = &project.processed_databases;

        // Register each loaded processed database path
        for (idx, loaded_path) in pd_state.loaded_paths.iter().enumerate() {
            let db_id = format!("pdb_{}", idx);
            let display_name = std::path::Path::new(loaded_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            // Try to extract metadata from cached_metadata if available
            let (db_type, case_number, examiner, total_size, artifact_count, metadata_json) =
                if let Some(ref meta_map) = pd_state.cached_metadata {
                    if let Some(meta_val) = meta_map.get(loaded_path) {
                        let db_type = meta_val.get("db_type").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                        let case_number = meta_val.get("case_number").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let examiner = meta_val.get("examiner").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let total_size = meta_val.get("total_size").and_then(|v| v.as_i64()).unwrap_or(0);
                        let artifact_count = meta_val.get("artifact_count").and_then(|v| v.as_i64());
                        let metadata_json = serde_json::to_string(meta_val).ok();
                        (db_type, case_number, examiner, total_size, artifact_count, metadata_json)
                    } else {
                        ("Unknown".to_string(), None, None, 0i64, None, None)
                    }
                } else {
                    ("Unknown".to_string(), None, None, 0i64, None, None)
                };

            conn.execute(
                "INSERT OR IGNORE INTO processed_databases (id, path, name, db_type, case_number, examiner, total_size, artifact_count, registered_at, metadata_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    db_id, loaded_path, display_name, db_type,
                    case_number, examiner, total_size, artifact_count,
                    now_str, metadata_json,
                ],
            )?;

            // Migrate integrity records for this path
            if let Some(integrity) = pd_state.integrity.get(loaded_path) {
                let integrity_id = format!("pdi_{}_{}", idx, 0);
                let changes_json = if integrity.changes.is_empty() {
                    None
                } else {
                    serde_json::to_string(&integrity.changes).ok()
                };
                conn.execute(
                    "INSERT OR IGNORE INTO processed_db_integrity (id, processed_db_id, file_path, file_size, baseline_hash, baseline_timestamp, current_hash, current_hash_timestamp, status, changes_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        integrity_id, db_id, &integrity.path,
                        integrity.file_size as i64, &integrity.baseline_hash,
                        &integrity.baseline_timestamp, &integrity.current_hash,
                        &integrity.current_hash_timestamp, &integrity.status, changes_json,
                    ],
                )?;

                // Migrate work metrics if present
                if let Some(ref metrics) = integrity.metrics {
                    let metrics_id = format!("pdm_{}", idx);
                    let user_names_json = serde_json::to_string(&metrics.user_names).ok();
                    conn.execute(
                        "INSERT OR IGNORE INTO processed_db_metrics (id, processed_db_id, total_scans, last_scan_date, total_jobs, last_job_date, total_notes, total_tagged_items, total_users, user_names_json, captured_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            metrics_id, db_id,
                            metrics.total_scans as i32, &metrics.last_scan_date,
                            metrics.total_jobs as i32, &metrics.last_job_date,
                            metrics.total_notes as i32, metrics.total_tagged_items as i32,
                            metrics.total_users as i32, user_names_json, &now_str,
                        ],
                    )?;
                }
            }

            // Migrate cached AXIOM case info
            if let Some(ref axiom_map) = pd_state.cached_axiom_case_info {
                if let Some(axiom_val) = axiom_map.get(loaded_path) {
                    let axiom_id = format!("axc_{}", idx);
                    let case_name = axiom_val.get("case_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                    let case_number = axiom_val.get("case_number").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let case_type = axiom_val.get("case_type").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let description = axiom_val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let examiner = axiom_val.get("examiner").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let agency = axiom_val.get("agency").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let axiom_version = axiom_val.get("axiom_version").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let search_start = axiom_val.get("search_start").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let search_end = axiom_val.get("search_end").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let search_duration = axiom_val.get("search_duration").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let search_outcome = axiom_val.get("search_outcome").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let output_folder = axiom_val.get("output_folder").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let total_artifacts = axiom_val.get("total_artifacts").and_then(|v| v.as_i64()).unwrap_or(0);
                    let case_path = axiom_val.get("case_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let keyword_info_json = axiom_val.get("keyword_info").map(|v| v.to_string());

                    conn.execute(
                        "INSERT OR IGNORE INTO axiom_case_info (id, processed_db_id, case_name, case_number, case_type, description, examiner, agency, axiom_version, search_start, search_end, search_duration, search_outcome, output_folder, total_artifacts, case_path, captured_at, keyword_info_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                        params![
                            axiom_id, db_id, case_name, case_number,
                            case_type, description, examiner, agency,
                            axiom_version, search_start, search_end,
                            search_duration, search_outcome, output_folder,
                            total_artifacts, case_path, now_str, keyword_info_json,
                        ],
                    )?;

                    // Migrate AXIOM evidence sources
                    if let Some(sources) = axiom_val.get("evidence_sources").and_then(|v| v.as_array()) {
                        for (si, source) in sources.iter().enumerate() {
                            let src_id = format!("axs_{}_{}", idx, si);
                            let name = source.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let evidence_number = source.get("evidence_number").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let source_type = source.get("source_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                            let path = source.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let hash = source.get("hash").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let size = source.get("size").and_then(|v| v.as_i64());
                            let acquired = source.get("acquired").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let search_types_json = source.get("search_types").map(|v| v.to_string());

                            conn.execute(
                                "INSERT OR IGNORE INTO axiom_evidence_sources (id, axiom_case_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json)
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                                params![src_id, axiom_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json],
                            )?;
                        }
                    }

                    // Migrate AXIOM search results
                    if let Some(results) = axiom_val.get("search_results").and_then(|v| v.as_array()) {
                        for (ri, result) in results.iter().enumerate() {
                            let res_id = format!("axr_{}_{}", idx, ri);
                            let artifact_type = result.get("artifact_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let hit_count = result.get("hit_count").and_then(|v| v.as_i64()).unwrap_or(0);

                            conn.execute(
                                "INSERT OR IGNORE INTO axiom_search_results (id, axiom_case_id, artifact_type, hit_count)
                                 VALUES (?1, ?2, ?3, ?4)",
                                params![res_id, axiom_id, artifact_type, hit_count],
                            )?;
                        }
                    }
                }
            }

            // Migrate cached artifact categories
            if let Some(ref cat_map) = pd_state.cached_artifact_categories {
                if let Some(cats) = cat_map.get(loaded_path) {
                    for (ci, cat) in cats.iter().enumerate() {
                        let cat_id = format!("cat_{}_{}", idx, ci);
                        let category = cat.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let artifact_type = cat.get("artifact_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let count = cat.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

                        conn.execute(
                            "INSERT OR IGNORE INTO artifact_categories (id, processed_db_id, category, artifact_type, count)
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![cat_id, db_id, category, artifact_type, count],
                        )?;
                    }
                }
            }
        }

        info!("Migration complete for project '{}' (including {} processed databases)", project.name, pd_state.loaded_paths.len());
        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (TempDir, ProjectDatabase) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.ffxdb");
        let db = ProjectDatabase::open(&db_path).expect("Failed to create test DB");
        (temp_dir, db)
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
        let path = Path::new("/case/folder/myproject.cffx");
        let db_path = ProjectDatabase::db_path_for_project(path);
        assert_eq!(
            db_path,
            PathBuf::from("/case/folder/myproject.ffxdb")
        );
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

        let result = db.lookup_hash_by_path("/case/evidence.E01", "SHA-256").unwrap();
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
        let found = db.get_processed_database_by_path("/case/2.Processed/AXIOM - Nov 15 2025").unwrap();
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
}
