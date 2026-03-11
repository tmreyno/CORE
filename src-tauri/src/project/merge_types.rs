// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Types used by the project merge system.

use serde::{Deserialize, Serialize};

/// Summary of a project to be merged (returned by analyze_projects)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMergeSummary {
    /// .cffx file path
    pub cffx_path: String,
    /// Companion .ffxdb path (may not exist)
    pub ffxdb_path: String,
    /// Whether the .ffxdb file exists
    pub ffxdb_exists: bool,
    /// Project name
    pub name: String,
    /// Project ID
    pub project_id: String,
    /// Owner/Examiner name (from project owner_name field)
    pub owner_name: Option<String>,
    /// When created
    pub created_at: String,
    /// When last saved
    pub saved_at: String,
    /// Number of discovered evidence files
    pub evidence_file_count: usize,
    /// Number of computed hashes
    pub hash_count: usize,
    /// Number of sessions
    pub session_count: usize,
    /// Number of activity log entries
    pub activity_count: usize,
    /// Number of bookmarks
    pub bookmark_count: usize,
    /// Number of notes
    pub note_count: usize,
    /// Number of tabs
    pub tab_count: usize,
    /// Number of reports
    pub report_count: usize,
    /// Root path
    pub root_path: String,
    /// Examiners/users found in .cffx and .ffxdb
    pub examiners: Vec<MergeExaminerInfo>,
    /// Evidence collection summaries from .ffxdb
    pub collections: Vec<MergeCollectionSummary>,
    /// COC item summaries from .ffxdb
    pub coc_items: Vec<MergeCocSummary>,
    /// Form submission summaries from .ffxdb
    pub form_submissions: Vec<MergeFormSummary>,
    /// Evidence file summaries from .ffxdb
    pub evidence_files: Vec<MergeEvidenceFileSummary>,
}

/// Examiner/user information gathered from .cffx and .ffxdb sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeExaminerInfo {
    /// Username or examiner name
    pub name: String,
    /// Display name if available
    pub display_name: Option<String>,
    /// Where this name was found
    pub source: String,
    /// Role context (e.g. "project owner", "session user", "collecting officer")
    pub role: String,
}

/// Summary of an evidence collection record from .ffxdb
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeCollectionSummary {
    pub id: String,
    pub case_number: String,
    pub collection_date: String,
    pub collecting_officer: String,
    pub collection_location: String,
    pub status: String,
    pub item_count: usize,
}

/// Summary of a COC item from .ffxdb
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeCocSummary {
    pub id: String,
    pub coc_number: String,
    pub case_number: String,
    pub evidence_id: String,
    pub description: String,
    pub submitted_by: String,
    pub received_by: String,
    pub status: String,
}

/// Summary of a form submission from .ffxdb
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeFormSummary {
    pub id: String,
    pub template_id: String,
    pub case_number: Option<String>,
    pub status: String,
    pub created_at: String,
    /// Collecting officer (extracted from data_json for evidence_collection templates)
    pub collecting_officer: Option<String>,
    /// Collection location (extracted from data_json for evidence_collection templates)
    pub collection_location: Option<String>,
    /// Lead examiner (extracted from data_json for IAR/activity templates)
    pub lead_examiner: Option<String>,
}

/// Summary of an evidence file from .ffxdb
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeEvidenceFileSummary {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub container_type: String,
    pub total_size: u64,
}

/// Owner assignment for a source project during merge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeSourceAssignment {
    /// Source .cffx path
    pub cffx_path: String,
    /// Assigned owner/examiner name for this source project
    pub owner_name: String,
}

/// Provenance record for a merged source project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeSource {
    /// Source project ID
    pub source_project_id: String,
    /// Source project name
    pub source_project_name: String,
    /// Source .cffx file path
    pub source_cffx_path: String,
    /// Owner/Examiner assigned to this source
    pub owner_name: String,
    /// When the merge was performed
    pub merged_at: String,
    /// Record counts from this source
    pub evidence_file_count: usize,
    pub session_count: usize,
    pub activity_count: usize,
    pub bookmark_count: usize,
    pub note_count: usize,
}

/// Result of a merge operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    pub success: bool,
    /// Path to the merged .cffx file
    pub cffx_path: Option<String>,
    /// Path to the merged .ffxdb file
    pub ffxdb_path: Option<String>,
    pub error: Option<String>,
    /// Merge statistics
    pub stats: Option<MergeStats>,
    /// Provenance records for each source project
    pub sources: Option<Vec<MergeSource>>,
}

/// Exclusion configuration for selective merging.
///
/// Allows users to skip entire data categories or individual items by ID.
/// Category names map to groups of related tables (see `table_category()` in merge_db.rs).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MergeExclusions {
    /// Table category names to skip entirely.
    /// Valid values: "evidence", "bookmarks_notes", "activity", "reports", "tags",
    /// "searches", "coc", "collections", "forms", "documents", "exports", "processed"
    #[serde(default)]
    pub skip_categories: Vec<String>,
    /// Individual evidence file IDs to exclude (also excludes their hashes/verifications)
    #[serde(default)]
    pub exclude_evidence_file_ids: Vec<String>,
    /// Individual COC item IDs to exclude (also excludes their amendments/audit/transfers)
    #[serde(default)]
    pub exclude_coc_item_ids: Vec<String>,
    /// Individual evidence collection IDs to exclude (also excludes their collected_items)
    #[serde(default)]
    pub exclude_collection_ids: Vec<String>,
    /// Individual form submission IDs to exclude
    #[serde(default)]
    pub exclude_form_submission_ids: Vec<String>,
}

/// Statistics from the merge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeStats {
    pub projects_merged: usize,
    pub users_merged: usize,
    pub sessions_merged: usize,
    pub activity_entries_merged: usize,
    pub evidence_files_merged: usize,
    pub hashes_merged: usize,
    pub bookmarks_merged: usize,
    pub notes_merged: usize,
    pub tabs_merged: usize,
    pub reports_merged: usize,
    pub tags_merged: usize,
    pub searches_merged: usize,
    pub ffxdb_tables_merged: usize,
}
