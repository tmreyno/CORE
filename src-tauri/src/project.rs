// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project file handling for FFX
//!
//! Saves and loads `.cffx` files which contain:
//! - Open tabs and their order
//! - Hash computation history
//! - UI state preferences
//! - Project metadata

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Current project file format version
pub const PROJECT_VERSION: u32 = 2;

/// Project file extension
pub const PROJECT_EXTENSION: &str = ".cffx";

/// Application version (from Cargo.toml)
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A saved FFX project (v2 schema with full state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFXProject {
    // === Metadata ===
    /// Project file format version
    pub version: u32,
    /// Project unique identifier
    #[serde(default = "generate_id")]
    pub project_id: String,
    /// Project name
    pub name: String,
    /// Project description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Root directory path
    pub root_path: String,
    /// When the project was created
    pub created_at: String,
    /// When the project was last saved
    pub saved_at: String,
    /// App version that created this project
    #[serde(default = "default_app_version")]
    pub created_by_version: String,
    /// App version that last saved this project
    #[serde(default = "default_app_version")]
    pub saved_by_version: String,

    // === Users & Sessions ===
    /// Users who have accessed this project
    #[serde(default)]
    pub users: Vec<ProjectUser>,
    /// Current user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_user: Option<String>,
    /// Session history
    #[serde(default)]
    pub sessions: Vec<ProjectSession>,
    /// Current session ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_session_id: Option<String>,
    /// Activity log
    #[serde(default)]
    pub activity_log: Vec<ActivityLogEntry>,
    /// Max activity log entries to keep
    #[serde(default = "default_activity_log_limit")]
    pub activity_log_limit: u32,

    // === Evidence State ===
    /// Project locations (evidence, processed databases)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locations: Option<ProjectLocations>,
    /// Open directories
    #[serde(default)]
    pub open_directories: Vec<OpenDirectory>,
    /// Recent directories
    #[serde(default)]
    pub recent_directories: Vec<RecentDirectory>,
    /// Open tabs
    #[serde(default)]
    pub tabs: Vec<ProjectTab>,
    /// Active tab path
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_tab_path: Option<String>,
    /// File selection state
    #[serde(default)]
    pub file_selection: FileSelectionState,
    /// Hash computation history
    #[serde(default)]
    pub hash_history: ProjectHashHistory,

    // === Processed Databases ===
    /// Processed database state
    #[serde(default)]
    pub processed_databases: ProcessedDatabaseState,

    // === Bookmarks & Notes ===
    /// Bookmarks
    #[serde(default)]
    pub bookmarks: Vec<ProjectBookmark>,
    /// Notes
    #[serde(default)]
    pub notes: Vec<ProjectNote>,
    /// Tag definitions
    #[serde(default)]
    pub tags: Vec<ProjectTag>,

    // === Reports ===
    /// Generated report history
    #[serde(default)]
    pub reports: Vec<ProjectReportRecord>,

    // === Searches ===
    /// Saved searches
    #[serde(default)]
    pub saved_searches: Vec<SavedSearch>,
    /// Recent searches
    #[serde(default)]
    pub recent_searches: Vec<RecentSearch>,
    /// Current filter state
    #[serde(default)]
    pub filter_state: FilterState,

    // === UI State ===
    /// UI state for restoration
    #[serde(default)]
    pub ui_state: ProjectUIState,

    // === Settings ===
    /// Project-specific settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<ProjectSettings>,

    // === Custom Data ===
    /// Custom key-value data for extensibility
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<HashMap<String, serde_json::Value>>,

    // === Legacy v1 fields (for backward compatibility) ===
    /// Application version (legacy field, mapped to saved_by_version)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
}

// Helper functions for serde defaults
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("proj_{}", timestamp)
}

fn default_app_version() -> String {
    APP_VERSION.to_string()
}

fn default_activity_log_limit() -> u32 {
    1000
}

// === Users & Sessions Types ===

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

// === Evidence & File State Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTab {
    pub file_path: String,
    pub name: String,
    pub order: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_position: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_viewed: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileSelectionState {
    #[serde(default)]
    pub selected_paths: Vec<String>,
    pub active_path: Option<String>,
    pub timestamp: String,
}

// === Hash History Types ===

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

// === Processed Database Types ===

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

// === Bookmarks & Notes Types ===

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

// === Reports Types ===

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

// === Search & Filter Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: String,
    pub name: String,
    pub query: String,
    pub search_type: String,
    pub is_regex: bool,
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

// === Directories & Paths Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLocations {
    pub project_root: String,
    pub evidence_path: String,
    pub processed_db_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_documents_path: Option<String>,
    pub auto_discovered: bool,
    pub configured_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_file_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_db_count: Option<u32>,
    #[serde(default)]
    pub load_stored_hashes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDirectory {
    pub path: String,
    pub opened_at: String,
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

// === UI State Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUIState {
    #[serde(default = "default_panel_width")]
    pub left_panel_width: u32,
    #[serde(default = "default_panel_width")]
    pub right_panel_width: u32,
    #[serde(default)]
    pub left_panel_collapsed: bool,
    #[serde(default)]
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
    #[serde(default)]
    pub show_hidden_files: bool,
    #[serde(default)]
    pub confirm_on_close: bool,
}

// === Settings Type ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    #[serde(default = "default_true")]
    pub auto_save: bool,
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval: u32,
    #[serde(default = "default_hash_algorithm")]
    pub default_hash_algorithm: String,
    #[serde(default)]
    pub verify_hashes_on_load: bool,
    #[serde(default = "default_true")]
    pub track_activity: bool,
    #[serde(default = "default_max_recent")]
    pub max_recent_items: u32,
}

fn default_true() -> bool {
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

/// Result of loading a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLoadResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<FFXProject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result of saving a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSaveResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl FFXProject {
    /// Create a new project for a given root directory
    pub fn new(root_path: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let name = Path::new(root_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();
        
        Self {
            // Metadata
            version: PROJECT_VERSION,
            project_id: generate_id(),
            name,
            description: None,
            root_path: root_path.to_string(),
            created_at: now.clone(),
            saved_at: now.clone(),
            created_by_version: APP_VERSION.to_string(),
            saved_by_version: APP_VERSION.to_string(),

            // Users & Sessions
            users: Vec::new(),
            current_user: None,
            sessions: Vec::new(),
            current_session_id: None,
            activity_log: Vec::new(),
            activity_log_limit: default_activity_log_limit(),

            // Evidence State
            locations: None,
            open_directories: Vec::new(),
            recent_directories: Vec::new(),
            tabs: Vec::new(),
            active_tab_path: None,
            file_selection: FileSelectionState {
                selected_paths: Vec::new(),
                active_path: None,
                timestamp: now.clone(),
            },
            hash_history: ProjectHashHistory::default(),

            // Processed Databases
            processed_databases: ProcessedDatabaseState::default(),

            // Bookmarks & Notes
            bookmarks: Vec::new(),
            notes: Vec::new(),
            tags: Vec::new(),

            // Reports
            reports: Vec::new(),

            // Searches
            saved_searches: Vec::new(),
            recent_searches: Vec::new(),
            filter_state: FilterState::default(),

            // UI State
            ui_state: ProjectUIState::default(),

            // Settings
            settings: None,

            // Custom Data
            custom_data: None,

            // Legacy
            app_version: Some(APP_VERSION.to_string()),
        }
    }
    
    /// Get the default save path for this project (parent directory)
    pub fn default_save_path(&self) -> PathBuf {
        let root = Path::new(&self.root_path);
        let parent = root.parent().unwrap_or(root);
        let filename = format!("{}{}", self.name, PROJECT_EXTENSION);
        parent.join(filename)
    }
    
    /// Update the saved_at timestamp
    pub fn touch(&mut self) {
        self.saved_at = chrono::Utc::now().to_rfc3339();
        self.saved_by_version = APP_VERSION.to_string();
        self.app_version = Some(APP_VERSION.to_string());
    }
}

/// Get the default project file path for a given root directory
pub fn get_default_project_path(root_path: &str) -> PathBuf {
    let root = Path::new(root_path);
    let name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled");
    let parent = root.parent().unwrap_or(root);
    let filename = format!("{}{}", name, PROJECT_EXTENSION);
    parent.join(filename)
}

/// Check if a project file exists for the given root directory
pub fn project_exists(root_path: &str) -> Option<PathBuf> {
    let project_path = get_default_project_path(root_path);
    if project_path.exists() {
        Some(project_path)
    } else {
        None
    }
}

/// Save a project to the specified path
pub fn save_project(project: &FFXProject, path: Option<&str>) -> ProjectSaveResult {
    let save_path = match path {
        Some(p) => PathBuf::from(p),
        None => project.default_save_path(),
    };
    
    info!("Saving project to: {:?}", save_path);
    
    // Serialize to pretty JSON
    match serde_json::to_string_pretty(project) {
        Ok(json) => {
            match fs::write(&save_path, &json) {
                Ok(_) => {
                    info!("Project saved successfully: {} bytes", json.len());
                    ProjectSaveResult {
                        success: true,
                        path: Some(save_path.to_string_lossy().to_string()),
                        error: None,
                    }
                }
                Err(e) => {
                    warn!("Failed to write project file: {}", e);
                    ProjectSaveResult {
                        success: false,
                        path: None,
                        error: Some(format!("Failed to write file: {}", e)),
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to serialize project: {}", e);
            ProjectSaveResult {
                success: false,
                path: None,
                error: Some(format!("Failed to serialize: {}", e)),
            }
        }
    }
}

/// Load a project from the specified path
pub fn load_project(path: &str) -> ProjectLoadResult {
    info!("Loading project from: {}", path);
    
    let path = Path::new(path);
    if !path.exists() {
        return ProjectLoadResult {
            success: false,
            project: None,
            error: Some("Project file not found".to_string()),
        };
    }
    
    match fs::read_to_string(path) {
        Ok(json) => {
            match serde_json::from_str::<FFXProject>(&json) {
                Ok(project) => {
                    // Handle version migration if needed
                    if project.version > PROJECT_VERSION {
                        warn!("Project file version {} is newer than supported version {}", 
                              project.version, PROJECT_VERSION);
                    }
                    
                    info!("Project loaded: {} ({} tabs)", project.name, project.tabs.len());
                    ProjectLoadResult {
                        success: true,
                        project: Some(project),
                        error: None,
                    }
                }
                Err(e) => {
                    warn!("Failed to parse project file: {}", e);
                    ProjectLoadResult {
                        success: false,
                        project: None,
                        error: Some(format!("Failed to parse project: {}", e)),
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to read project file: {}", e);
            ProjectLoadResult {
                success: false,
                project: None,
                error: Some(format!("Failed to read file: {}", e)),
            }
        }
    }
}

/// Check if a project exists and return its path
pub fn check_project_exists(root_path: &str) -> Option<String> {
    project_exists(root_path).map(|p| p.to_string_lossy().to_string())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = FFXProject::new("/test/case-folder");
        assert_eq!(project.version, PROJECT_VERSION);
        assert_eq!(project.name, "case-folder");
        assert_eq!(project.root_path, "/test/case-folder");
        assert!(project.tabs.is_empty());
        assert!(project.active_tab_path.is_none());
        assert_eq!(project.app_version, Some(APP_VERSION.to_string()));
    }

    #[test]
    fn test_project_new_with_trailing_slash() {
        let project = FFXProject::new("/test/my-case");
        assert_eq!(project.name, "my-case");
    }

    #[test]
    fn test_project_default_save_path() {
        let project = FFXProject::new("/home/user/cases/case-001");
        let save_path = project.default_save_path();
        assert!(save_path.to_string_lossy().contains("case-001.cffx"));
    }

    #[test]
    fn test_get_default_project_path() {
        let path = get_default_project_path("/evidence/case-2024");
        assert!(path.to_string_lossy().contains("case-2024.cffx"));
    }

    #[test]
    fn test_project_touch() {
        let mut project = FFXProject::new("/test/case");
        let original_saved = project.saved_at.clone();
        
        // Small delay to ensure timestamp differs
        std::thread::sleep(std::time::Duration::from_millis(10));
        project.touch();
        
        assert_ne!(project.saved_at, original_saved);
        assert_eq!(project.app_version, Some(APP_VERSION.to_string()));
    }

    #[test]
    fn test_project_serialization_roundtrip() {
        let mut project = FFXProject::new("/test/case");
        project.tabs.push(ProjectTab {
            file_path: "evidence.E01".to_string(),
            name: "evidence.E01".to_string(),
            order: 0,
            container_type: Some("E01".to_string()),
            scroll_position: None,
            last_viewed: None,
        });
        project.active_tab_path = Some("evidence.E01".to_string());
        project.notes.push(ProjectNote {
            id: "note1".to_string(),
            target_type: "file".to_string(),
            target_path: Some("evidence.E01".to_string()),
            title: "Test Note".to_string(),
            content: "Test notes".to_string(),
            created_by: "tester".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["important".to_string()],
            priority: None,
        });
        project.tags.push(ProjectTag {
            id: "tag1".to_string(),
            name: "important".to_string(),
            color: "#ff0000".to_string(),
            description: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
        
        // Add hash history
        project.hash_history.files.insert(
            "evidence.E01".to_string(),
            vec![ProjectFileHash {
                algorithm: "SHA-256".to_string(),
                hash_value: "abc123".to_string(),
                computed_at: chrono::Utc::now().to_rfc3339(),
                verification: None,
            }],
        );
        
        // Serialize and deserialize
        let json = serde_json::to_string(&project).unwrap();
        let deserialized: FFXProject = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.version, project.version);
        assert_eq!(deserialized.name, project.name);
        assert_eq!(deserialized.root_path, project.root_path);
        assert_eq!(deserialized.tabs.len(), 1);
        assert_eq!(deserialized.active_tab_path, Some("evidence.E01".to_string()));
        assert_eq!(deserialized.notes.len(), 1);
        assert_eq!(deserialized.tags.len(), 1);
        assert!(deserialized.hash_history.files.contains_key("evidence.E01"));
    }

    #[test]
    fn test_project_hash_history_default() {
        let history = ProjectHashHistory::default();
        assert!(history.files.is_empty());
    }

    #[test]
    fn test_project_ui_state_default() {
        let ui_state = ProjectUIState::default();
        assert_eq!(ui_state.left_panel_width, 300);
        assert_eq!(ui_state.right_panel_width, 300);
        assert!(!ui_state.left_panel_collapsed);
        assert!(!ui_state.right_panel_collapsed);
    }

    #[test]
    fn test_project_load_result_success() {
        let result = ProjectLoadResult {
            success: true,
            project: Some(FFXProject::new("/test")),
            error: None,
        };
        assert!(result.success);
        assert!(result.project.is_some());
    }

    #[test]
    fn test_project_load_result_failure() {
        let result = ProjectLoadResult {
            success: false,
            project: None,
            error: Some("File not found".to_string()),
        };
        assert!(!result.success);
        assert!(result.project.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_project_save_result_success() {
        let result = ProjectSaveResult {
            success: true,
            path: Some("/test/case.cffx".to_string()),
            error: None,
        };
        assert!(result.success);
        assert!(result.path.is_some());
    }

    #[test]
    fn test_project_constants() {
        assert_eq!(PROJECT_EXTENSION, ".cffx");
        // Verify version is set (compile-time constant)
        assert_eq!(PROJECT_VERSION, 2);
        assert!(!APP_VERSION.is_empty());
    }
}
