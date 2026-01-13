// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project file handling for FFX
//!
//! Saves and loads `.ffxproj` files which contain:
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
pub const PROJECT_VERSION: u32 = 1;

/// Project file extension
pub const PROJECT_EXTENSION: &str = ".ffxproj";

/// Application version (from Cargo.toml)
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A saved FFX project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFXProject {
    /// Project file format version
    pub version: u32,
    /// Project name (derived from directory name)
    pub name: String,
    /// Root directory path that was opened
    pub root_path: String,
    /// When the project was created
    pub created_at: String,
    /// When the project was last saved
    pub saved_at: String,
    /// Application version that created/saved this project
    pub app_version: String,
    /// Open tabs state
    pub tabs: Vec<ProjectTab>,
    /// Active tab file path (if any)
    pub active_tab_path: Option<String>,
    /// Hash computation history
    pub hash_history: ProjectHashHistory,
    /// UI state preferences
    pub ui_state: ProjectUIState,
    /// User notes/annotations (future feature)
    #[serde(default)]
    pub notes: Option<String>,
    /// Custom tags/labels (future feature)
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A tab in the saved project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTab {
    /// File path (can be absolute or relative to root_path)
    pub file_path: String,
    /// Tab order (0-based)
    pub order: u32,
}

/// Hash history for the project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectHashHistory {
    /// Map of file path to hash records
    pub files: HashMap<String, Vec<ProjectFileHash>>,
}

/// A single hash record for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFileHash {
    /// Algorithm used (e.g., "SHA-256", "MD5", "BLAKE3")
    pub algorithm: String,
    /// Computed hash value (hex string)
    pub hash_value: String,
    /// When computed (ISO 8601)
    pub computed_at: String,
    /// Verification status if verified
    #[serde(default)]
    pub verified: Option<ProjectVerification>,
}

/// Verification result for a hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectVerification {
    /// Result of verification
    pub result: String, // "match" or "mismatch"
    /// When verified (ISO 8601)
    pub verified_at: String,
}

/// UI state to restore
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectUIState {
    /// Panel sizes (if adjustable)
    #[serde(default)]
    pub panel_sizes: Vec<f64>,
    /// Expanded tree nodes (paths)
    #[serde(default)]
    pub expanded_paths: Vec<String>,
    /// Scroll positions by panel ID
    #[serde(default)]
    pub scroll_positions: HashMap<String, f64>,
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
            version: PROJECT_VERSION,
            name,
            root_path: root_path.to_string(),
            created_at: now.clone(),
            saved_at: now,
            app_version: APP_VERSION.to_string(),
            tabs: Vec::new(),
            active_tab_path: None,
            hash_history: ProjectHashHistory::default(),
            ui_state: ProjectUIState::default(),
            notes: None,
            tags: Vec::new(),
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
        self.app_version = APP_VERSION.to_string();
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
        assert_eq!(project.app_version, APP_VERSION);
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
        assert!(save_path.to_string_lossy().contains("case-001.ffxproj"));
    }

    #[test]
    fn test_get_default_project_path() {
        let path = get_default_project_path("/evidence/case-2024");
        assert!(path.to_string_lossy().contains("case-2024.ffxproj"));
    }

    #[test]
    fn test_project_touch() {
        let mut project = FFXProject::new("/test/case");
        let original_saved = project.saved_at.clone();
        
        // Small delay to ensure timestamp differs
        std::thread::sleep(std::time::Duration::from_millis(10));
        project.touch();
        
        assert_ne!(project.saved_at, original_saved);
        assert_eq!(project.app_version, APP_VERSION);
    }

    #[test]
    fn test_project_serialization_roundtrip() {
        let mut project = FFXProject::new("/test/case");
        project.tabs.push(ProjectTab {
            file_path: "evidence.E01".to_string(),
            order: 0,
        });
        project.active_tab_path = Some("evidence.E01".to_string());
        project.notes = Some("Test notes".to_string());
        project.tags.push("important".to_string());
        
        // Add hash history
        project.hash_history.files.insert(
            "evidence.E01".to_string(),
            vec![ProjectFileHash {
                algorithm: "SHA-256".to_string(),
                hash_value: "abc123".to_string(),
                computed_at: chrono::Utc::now().to_rfc3339(),
                verified: None,
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
        assert_eq!(deserialized.notes, Some("Test notes".to_string()));
        assert_eq!(deserialized.tags, vec!["important"]);
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
        assert!(ui_state.panel_sizes.is_empty());
        assert!(ui_state.expanded_paths.is_empty());
        assert!(ui_state.scroll_positions.is_empty());
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
            path: Some("/test/case.ffxproj".to_string()),
            error: None,
        };
        assert!(result.success);
        assert!(result.path.is_some());
    }

    #[test]
    fn test_project_constants() {
        assert_eq!(PROJECT_EXTENSION, ".ffxproj");
        // Verify version is set (compile-time constant)
        assert_eq!(PROJECT_VERSION, 1);
        assert!(!APP_VERSION.is_empty());
    }
}
