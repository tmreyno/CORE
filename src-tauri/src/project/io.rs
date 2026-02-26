// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project file I/O — save, load, and path helper functions.

use super::migration::{make_paths_absolute, make_paths_relative, migrate_project};
use super::types::{ProjectLoadResult, ProjectSaveResult};
use super::{FFXProject, PROJECT_EXTENSION, PROJECT_VERSION};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

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

/// Check if a project exists and return its path
pub fn check_project_exists(root_path: &str) -> Option<String> {
    project_exists(root_path).map(|p| p.to_string_lossy().to_string())
}

/// Save a project to the specified path
/// Paths are converted to relative for portability across computers.
pub fn save_project(project: &FFXProject, path: Option<&str>) -> ProjectSaveResult {
    let save_path = match path {
        Some(p) => PathBuf::from(p),
        None => project.default_save_path(),
    };
    
    info!("Saving project to: {:?}", save_path);
    
    // Get the directory containing the project file for relative path calculation
    let project_dir = save_path.parent().unwrap_or(Path::new("."));
    
    // Clone and convert to relative paths for portability
    let mut portable_project = project.clone();
    make_paths_relative(&mut portable_project, project_dir);
    
    // Serialize to pretty JSON
    match serde_json::to_string_pretty(&portable_project) {
        Ok(json) => {
            match fs::write(&save_path, &json) {
                Ok(_) => {
                    info!("Project saved successfully with portable paths: {} bytes", json.len());
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
            warnings: None,
        };
    }
    
    // Get the directory containing the project file for resolving relative paths
    let project_dir = path.parent().unwrap_or(Path::new("."));
    
    match fs::read_to_string(path) {
        Ok(json) => {
            match serde_json::from_str::<FFXProject>(&json) {
                Ok(mut project) => {
                    let mut warnings: Vec<String> = Vec::new();
                    
                    // Convert relative paths to absolute for this machine
                    make_paths_absolute(&mut project, project_dir);
                    
                    // Handle version migration
                    if project.version < PROJECT_VERSION {
                        info!("Migrating project from version {} to {}", project.version, PROJECT_VERSION);
                        migrate_project(&mut project);
                        warnings.push(format!(
                            "Project was migrated from version {} to {}. Save to update the file.",
                            project.version, PROJECT_VERSION
                        ));
                        project.version = PROJECT_VERSION;
                    } else if project.version > PROJECT_VERSION {
                        warn!("Project file version {} is newer than supported version {}", 
                              project.version, PROJECT_VERSION);
                        warnings.push(format!(
                            "This project was created with a newer version of CORE-FFX (v{}). Some features may not work correctly.",
                            project.version
                        ));
                    }
                    
                    info!("Project loaded with resolved paths: {} ({} tabs)", project.name, project.tabs.len());
                    ProjectLoadResult {
                        success: true,
                        project: Some(project),
                        error: None,
                        warnings: if warnings.is_empty() { None } else { Some(warnings) },
                    }
                }
                Err(e) => {
                    warn!("Failed to parse project file: {}", e);
                    ProjectLoadResult {
                        success: false,
                        project: None,
                        error: Some(format!("Failed to parse project: {}", e)),
                        warnings: None,
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
                warnings: None,
            }
        }
    }
}
