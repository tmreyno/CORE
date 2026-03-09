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

mod io;
pub mod merge;
pub(crate) mod migration;
#[cfg(test)]
mod tests;
pub mod types;

// Re-export everything for backwards compatibility — all external code uses
// `crate::project::FFXProject`, `crate::project::ProjectTab`, etc.
pub use io::*;
pub use types::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Current project file format version
pub const PROJECT_VERSION: u32 = 2;

/// Project file extension
pub const PROJECT_EXTENSION: &str = ".cffx";

/// Application version (from Cargo.toml)
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// Helper functions for serde defaults (used by FFXProject's serde attributes)
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after UNIX_EPOCH")
        .as_millis();
    format!("proj_{}", timestamp)
}

fn default_app_version() -> String {
    APP_VERSION.to_string()
}

fn default_activity_log_limit() -> u32 {
    1000
}

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
    /// Owner/Examiner name (person responsible for this project)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_name: Option<String>,
    /// Parent case number (the larger case this project belongs to)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    /// Parent case name/title (the larger case this project belongs to)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_name: Option<String>,
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

    // === Database ===
    /// Relative path to the companion .ffxdb database (e.g. ".ffx/project.ffxdb")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub db_path: Option<String>,

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
    /// Center pane state for proper restoration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center_pane_state: Option<CenterPaneState>,
    /// File selection state
    #[serde(default)]
    pub file_selection: FileSelectionState,
    /// Hash computation history
    #[serde(default)]
    pub hash_history: ProjectHashHistory,
    /// Evidence cache (discovered files, loaded info, computed hashes) to avoid re-scan/re-load
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_cache: Option<EvidenceCache>,
    /// Case documents cache to avoid re-discovery on load
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_documents_cache: Option<CaseDocumentsCache>,
    /// Preview cache for extracted container files
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_cache: Option<PreviewCache>,

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

    // === Merge Provenance ===
    /// Source projects that were merged into this project
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_sources: Option<Vec<merge::MergeSource>>,

    // === Custom Data ===
    /// Custom key-value data for extensibility
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<HashMap<String, serde_json::Value>>,

    // === Legacy v1 fields (for backward compatibility) ===
    /// Application version (legacy field, mapped to saved_by_version)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
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
            owner_name: None,
            case_number: None,
            case_name: None,
            description: None,
            root_path: root_path.to_string(),
            created_at: now.clone(),
            saved_at: now.clone(),
            created_by_version: APP_VERSION.to_string(),
            saved_by_version: APP_VERSION.to_string(),

            // Database
            db_path: None,

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
            center_pane_state: None,
            file_selection: FileSelectionState {
                selected_paths: Vec::new(),
                active_path: None,
                timestamp: now.clone(),
            },
            hash_history: ProjectHashHistory::default(),
            evidence_cache: None,
            case_documents_cache: None,
            preview_cache: None,

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

            // Merge Provenance
            merge_sources: None,

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
