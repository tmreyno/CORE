// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project type definitions — all subordinate structs used by FFXProject.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

// =============================================================================
// Serde Deserializer Helpers
// =============================================================================

/// Deserialize a bool that may be JSON `null` — treats null as `false`.
/// Use with `#[serde(default, deserialize_with = "deserialize_bool_or_null")]`
pub(crate) fn deserialize_bool_or_null<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<bool>::deserialize(deserializer).map(|opt| opt.unwrap_or(false))
}

/// Deserialize a bool that may be JSON `null` — treats null as `true`.
/// Use with `#[serde(default = "default_true", deserialize_with = "deserialize_bool_or_null_true")]`
pub(crate) fn deserialize_bool_or_null_true<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<bool>::deserialize(deserializer).map(|opt| opt.unwrap_or(true))
}

/// Deserialize a u32 that may be JSON `null` — treats null as `0`.
/// Use with `#[serde(default, deserialize_with = "deserialize_u32_or_null")]`
pub(crate) fn deserialize_u32_or_null<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<u32>::deserialize(deserializer).map(|opt| opt.unwrap_or(0))
}

// =============================================================================
// Users & Sessions Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUser {
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    pub first_access: String,
    pub last_access: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSession {
    pub session_id: String,
    pub user: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    pub app_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    pub id: String,
    pub timestamp: String,
    pub user: String,
    pub category: String,
    pub action: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

// =============================================================================
// Evidence & File State Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTab {
    /// Unique tab identifier
    #[serde(default = "default_tab_id")]
    pub id: String,
    /// Tab type (evidence, document, entry, export, processed)
    #[serde(rename = "type", default = "default_tab_type")]
    pub tab_type: String,
    /// File path (absolute) - for evidence files
    pub file_path: String,
    /// Display name
    pub name: String,
    /// Subtitle (e.g., container type)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Tab order (0-based)
    pub order: u32,
    /// Container type - for evidence tabs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_type: Option<String>,
    /// Document path - for case document tabs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_path: Option<String>,
    /// Container entry path - for entry tabs (files inside containers)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_path: Option<String>,
    /// Parent container path - for entry tabs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_container_path: Option<String>,
    /// Entry name - for entry tabs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_name: Option<String>,
    /// Processed database path - for processed db tabs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_db_path: Option<String>,
    /// Processed database type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_db_type: Option<String>,
    /// Scroll position in file list
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_position: Option<f64>,
    /// Last viewed timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_viewed: Option<String>,
}

fn default_tab_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after UNIX_EPOCH")
        .as_millis();
    format!("tab_{}", timestamp)
}

fn default_tab_type() -> String {
    "evidence".to_string()
}

/// Center pane state for tab management
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CenterPaneState {
    /// Active tab ID
    pub active_tab_id: Option<String>,
    /// Current view mode
    #[serde(default = "default_view_mode")]
    pub view_mode: String,
}

fn default_view_mode() -> String {
    "info".to_string()
}

/// Evidence cache for discovered files, loaded info, computed hashes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceCache {
    /// Discovered evidence files
    #[serde(default)]
    pub discovered_files: Vec<CachedDiscoveredFile>,
    /// File info cache (path -> container info)
    #[serde(default)]
    pub file_info: HashMap<String, serde_json::Value>,
    /// Computed hashes cache
    #[serde(default)]
    pub computed_hashes: HashMap<String, CachedFileHash>,
    /// When cache was created
    #[serde(default)]
    pub cached_at: String,
    /// Whether cache is valid
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub valid: bool,
}

/// Cached discovered file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDiscoveredFile {
    pub path: String,
    pub filename: String,
    pub container_type: String,
    pub size: u64,
    #[serde(default, deserialize_with = "deserialize_u32_or_null")]
    pub segment_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

/// Cached file hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFileHash {
    pub algorithm: String,
    pub hash: String,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub verified: bool,
    pub computed_at: String,
}

/// Case documents cache
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaseDocumentsCache {
    /// Discovered case documents
    #[serde(default)]
    pub documents: Vec<CachedCaseDocument>,
    /// Path that was searched
    #[serde(default)]
    pub search_path: String,
    /// When cache was created
    #[serde(default)]
    pub cached_at: String,
    /// Whether cache is valid
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub valid: bool,
}

/// Cached case document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCaseDocument {
    pub path: String,
    pub filename: String,
    #[serde(default)]
    pub document_type: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

/// Preview cache for extracted container files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreviewCache {
    /// Cache entries
    #[serde(default)]
    pub entries: Vec<PreviewCacheEntry>,
    /// When cache was updated
    #[serde(default)]
    pub cached_at: String,
    /// Project cache directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,
}

/// Preview cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewCacheEntry {
    pub key: String,
    pub container_path: String,
    pub entry_path: String,
    pub temp_path: String,
    pub entry_size: u64,
    pub extracted_at: String,
    #[serde(default)]
    pub valid: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileSelectionState {
    #[serde(default)]
    pub selected_paths: Vec<String>,
    pub active_path: Option<String>,
    pub timestamp: String,
}

// =============================================================================
// Hash History Types
// =============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectHashHistory {
    #[serde(default)]
    pub files: HashMap<String, Vec<ProjectFileHash>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFileHash {
    pub algorithm: String,
    pub hash_value: String,
    pub computed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<ProjectVerification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectVerification {
    pub result: String,
    pub verified_against: String,
    pub verified_at: String,
}

// =============================================================================
// Processed Database Types
// =============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessedDatabaseState {
    #[serde(default)]
    pub loaded_paths: Vec<String>,
    pub selected_path: Option<String>,
    pub detail_view_type: Option<String>,
    #[serde(default)]
    pub integrity: HashMap<String, ProcessedDbIntegrity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_metadata: Option<HashMap<String, serde_json::Value>>,
    /// Full cached database objects for complete restoration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_databases: Option<Vec<serde_json::Value>>,
    /// AXIOM case info cache
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_axiom_case_info: Option<HashMap<String, serde_json::Value>>,
    /// Artifact categories cache
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_artifact_categories: Option<HashMap<String, Vec<serde_json::Value>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDbIntegrity {
    pub path: String,
    pub file_size: u64,
    pub baseline_hash: String,
    pub baseline_timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_hash_timestamp: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<ProcessedDbWorkMetrics>,
    #[serde(default)]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDbWorkMetrics {
    pub total_scans: u32,
    pub last_scan_date: Option<String>,
    pub total_jobs: u32,
    pub last_job_date: Option<String>,
    pub total_notes: u32,
    pub total_tagged_items: u32,
    pub total_users: u32,
    pub user_names: Vec<String>,
}

// =============================================================================
// Bookmarks & Notes Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBookmark {
    pub id: String,
    pub target_type: String,
    pub target_path: String,
    pub name: String,
    pub created_by: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNote {
    pub id: String,
    pub target_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_path: Option<String>,
    pub title: String,
    pub content: String,
    pub created_by: String,
    pub created_at: String,
    pub modified_at: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTag {
    pub id: String,
    pub name: String,
    pub color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
}

// =============================================================================
// Reports Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReportRecord {
    pub id: String,
    pub title: String,
    pub report_type: String,
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    pub generated_at: String,
    pub generated_by: String,
    #[serde(default)]
    pub included_items: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// =============================================================================
// Search & Filter Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: String,
    pub name: String,
    pub query: String,
    pub search_type: String,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub is_regex: bool,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub case_sensitive: bool,
    pub scope: String,
    pub created_at: String,
    pub use_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentSearch {
    pub query: String,
    pub timestamp: String,
    pub result_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterState {
    pub type_filter: Option<String>,
    pub status_filter: Option<String>,
    pub search_query: Option<String>,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_sort_direction")]
    pub sort_direction: String,
}

fn default_sort_by() -> String {
    "name".to_string()
}

fn default_sort_direction() -> String {
    "asc".to_string()
}

// =============================================================================
// Directories & Paths Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLocations {
    pub project_root: String,
    pub evidence_path: String,
    pub processed_db_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_documents_path: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub auto_discovered: bool,
    pub configured_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_file_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_db_count: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub load_stored_hashes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDirectory {
    pub path: String,
    pub opened_at: String,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub recursive: bool,
    pub file_count: u32,
    pub total_size: u64,
    pub last_scanned: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentDirectory {
    pub path: String,
    pub open_count: u32,
    pub last_opened: String,
    pub name: String,
}

// =============================================================================
// UI State Types
// =============================================================================

/// Tree expansion state for restoring which containers/folders are expanded.
/// This captures the expansion state across all tree types for project restoration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TreeExpansionState {
    /// Expanded container paths (top-level)
    #[serde(default)]
    pub containers: Vec<String>,
    /// Expanded VFS paths (E01, Raw, L01)
    #[serde(default)]
    pub vfs: Vec<String>,
    /// Expanded archive paths (ZIP, 7z, TAR)
    #[serde(default)]
    pub archive: Vec<String>,
    /// Expanded lazy-loaded paths (UFED, large containers)
    #[serde(default)]
    pub lazy: Vec<String>,
    /// Expanded AD1 directory entries
    #[serde(default)]
    pub ad1: Vec<String>,
    /// Currently selected entry key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUIState {
    #[serde(default = "default_panel_width")]
    pub left_panel_width: u32,
    #[serde(default = "default_panel_width")]
    pub right_panel_width: u32,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub left_panel_collapsed: bool,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub right_panel_collapsed: bool,
    #[serde(default = "default_left_panel_tab")]
    pub left_panel_tab: String,
    #[serde(default = "default_detail_view_mode")]
    pub detail_view_mode: String,
    #[serde(default)]
    pub tree_state: Vec<TreeNodeState>,
    #[serde(default)]
    pub scroll_positions: HashMap<String, f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_dimensions: Option<WindowDimensions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferences: Option<UIPreferences>,
    /// Comprehensive tree expansion state for all container types
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_expansion_state: Option<TreeExpansionState>,
    /// Selected container entry for restoration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_entry: Option<SelectedEntryState>,
    /// Entry content view mode (auto, hex, text, document)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_content_view_mode: Option<String>,
    /// Case documents search path
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_documents_path: Option<String>,
}

/// Selected container entry for UI restoration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedEntryState {
    pub container_path: String,
    pub entry_path: String,
    pub name: String,
}

impl Default for ProjectUIState {
    fn default() -> Self {
        Self {
            left_panel_width: default_panel_width(),
            right_panel_width: default_panel_width(),
            left_panel_collapsed: false,
            right_panel_collapsed: false,
            left_panel_tab: default_left_panel_tab(),
            detail_view_mode: default_detail_view_mode(),
            tree_state: Vec::new(),
            scroll_positions: HashMap::new(),
            window_dimensions: None,
            preferences: None,
            tree_expansion_state: None,
            selected_entry: None,
            entry_content_view_mode: None,
            case_documents_path: None,
        }
    }
}

fn default_panel_width() -> u32 {
    300
}

fn default_left_panel_tab() -> String {
    "evidence".to_string()
}

fn default_detail_view_mode() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNodeState {
    pub path: String,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub expanded: bool,
    #[serde(default)]
    pub children: Vec<TreeNodeState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPreferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub show_hidden_files: bool,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub confirm_on_close: bool,
}

// =============================================================================
// Settings Type
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_or_null_true"
    )]
    pub auto_save: bool,
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval: u32,
    #[serde(default = "default_hash_algorithm")]
    pub default_hash_algorithm: String,
    #[serde(default, deserialize_with = "deserialize_bool_or_null")]
    pub verify_hashes_on_load: bool,
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_or_null_true"
    )]
    pub track_activity: bool,
    #[serde(default = "default_max_recent")]
    pub max_recent_items: u32,
}

pub(crate) fn default_true() -> bool {
    true
}

fn default_auto_save_interval() -> u32 {
    300000 // 5 minutes in milliseconds
}

fn default_hash_algorithm() -> String {
    "SHA-256".to_string()
}

fn default_max_recent() -> u32 {
    20
}

// =============================================================================
// Result Types
// =============================================================================

/// Result of loading a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectLoadResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<super::FFXProject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl ProjectLoadResult {
    /// Create a successful load result
    #[inline]
    pub fn success(project: super::FFXProject) -> Self {
        Self {
            success: true,
            project: Some(project),
            error: None,
            warnings: None,
        }
    }

    /// Create a failed load result
    #[inline]
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            project: None,
            error: Some(error.into()),
            warnings: None,
        }
    }

    /// Create a successful load result with warnings
    #[inline]
    pub fn success_with_warnings(project: super::FFXProject, warnings: Vec<String>) -> Self {
        Self {
            success: true,
            project: Some(project),
            error: None,
            warnings: if warnings.is_empty() {
                None
            } else {
                Some(warnings)
            },
        }
    }
}

/// Result of saving a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectSaveResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ProjectSaveResult {
    /// Create a successful save result
    #[inline]
    pub fn success(path: impl Into<String>) -> Self {
        Self {
            success: true,
            path: Some(path.into()),
            error: None,
        }
    }

    /// Create a failed save result
    #[inline]
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            path: None,
            error: Some(error.into()),
        }
    }
}
