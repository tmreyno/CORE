// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project recovery commands.

use crate::project_recovery;

/// Create a backup of the project file
#[tauri::command]
pub async fn project_create_backup(
    project_path: String,
    backup_type: project_recovery::BackupType,
    user: Option<String>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(project_path);
        project_recovery::create_backup(&path, backup_type, user)
            .map(|p| p.to_string_lossy().to_string())
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task failed: {}", e)))
}

/// Create a versioned backup
#[tauri::command]
pub async fn project_create_version(project_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(project_path);
        project_recovery::create_version_backup(&path).map(|p| p.to_string_lossy().to_string())
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task failed: {}", e)))
}

/// List all version backups for a project
#[tauri::command]
pub async fn project_list_versions(
    project_path: String,
) -> Result<Vec<project_recovery::BackupFile>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(project_path);
        project_recovery::list_version_backups(&path)
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task failed: {}", e)))
}

/// Check if recovery is available
#[tauri::command]
pub async fn project_check_recovery(project_path: String) -> project_recovery::RecoveryInfo {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(project_path);
        project_recovery::check_recovery(&path)
    })
    .await
    .unwrap_or(project_recovery::RecoveryInfo {
        has_autosave: false,
        autosave_path: None,
        autosave_age_seconds: None,
        autosave_is_newer: false,
        has_backup: false,
        backup_path: None,
    })
}

/// Recover project from autosave
#[tauri::command]
pub async fn project_recover_autosave(
    project_path: String,
) -> Result<crate::project::FFXProject, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(project_path);
        project_recovery::recover_from_autosave(&path)
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task failed: {}", e)))
}

/// Clear autosave file
#[tauri::command]
pub async fn project_clear_autosave(project_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(project_path);
        project_recovery::clear_autosave(&path)
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task failed: {}", e)))
}

/// Check project health
#[tauri::command]
pub async fn project_check_health(
    project_path: String,
) -> Result<project_recovery::ProjectHealth, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(project_path);
        project_recovery::check_project_health(&path)
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task failed: {}", e)))
}
