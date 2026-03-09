// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project file handling commands (.cffx files).

use crate::project;

/// Get the default project file path for a root directory
#[tauri::command]
pub fn project_get_default_path(root_path: String) -> String {
    project::get_default_project_path(&root_path)
        .to_string_lossy()
        .to_string()
}

/// Check if a project file exists for the given root directory
#[tauri::command]
pub fn project_check_exists(root_path: String) -> Option<String> {
    project::check_project_exists(&root_path)
}

/// Save a project to the specified path (or default if not provided)
#[tauri::command]
pub fn project_save(
    project: project::FFXProject,
    path: Option<String>,
) -> project::ProjectSaveResult {
    let mut proj = project;
    proj.touch(); // Update saved_at timestamp
    project::save_project(&proj, path.as_deref())
}

/// Load a project from the specified path
#[tauri::command]
pub fn project_load(path: String) -> project::ProjectLoadResult {
    project::load_project(&path)
}

/// Create a new project for a root directory
#[tauri::command]
pub fn project_create(
    root_path: String,
    owner_name: Option<String>,
    case_number: Option<String>,
    case_name: Option<String>,
) -> project::FFXProject {
    let mut project = project::FFXProject::new(&root_path);
    project.owner_name = owner_name;
    project.case_number = case_number;
    project.case_name = case_name;
    project
}
