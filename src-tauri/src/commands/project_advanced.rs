// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project recovery commands.

use crate::project_recovery;

/// Create a backup of the project file
#[tauri::command]
pub fn project_create_backup(
    project_path: String,
    backup_type: project_recovery::BackupType,
    user: Option<String>,
) -> Result<String, String> {
    let path = std::path::Path::new(&project_path);
    project_recovery::create_backup(path, backup_type, user)
        .map(|p| p.to_string_lossy().to_string())
}

/// Create a versioned backup
#[tauri::command]
pub fn project_create_version(project_path: String) -> Result<String, String> {
    let path = std::path::Path::new(&project_path);
    project_recovery::create_version_backup(path)
        .map(|p| p.to_string_lossy().to_string())
}

/// List all version backups for a project
#[tauri::command]
pub fn project_list_versions(
    project_path: String,
) -> Result<Vec<project_recovery::BackupFile>, String> {
    let path = std::path::Path::new(&project_path);
    project_recovery::list_version_backups(path)
}

/// Check if recovery is available
#[tauri::command]
pub fn project_check_recovery(project_path: String) -> project_recovery::RecoveryInfo {
    let path = std::path::Path::new(&project_path);
    project_recovery::check_recovery(path)
}

/// Recover project from autosave
#[tauri::command]
pub fn project_recover_autosave(
    project_path: String,
) -> Result<crate::project::FFXProject, String> {
    let path = std::path::Path::new(&project_path);
    project_recovery::recover_from_autosave(path)
}

/// Clear autosave file
#[tauri::command]
pub fn project_clear_autosave(project_path: String) -> Result<(), String> {
    let path = std::path::Path::new(&project_path);
    project_recovery::clear_autosave(path)
}

/// Check project health
#[tauri::command]
pub fn project_check_health(
    project_path: String,
) -> Result<project_recovery::ProjectHealth, String> {
    let path = std::path::Path::new(&project_path);
    project_recovery::check_project_health(path)
}
