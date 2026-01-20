// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED container operations (UFD, UFDR, UFDX formats).

use tracing::debug;

use crate::ufed;

/// UFED tree entry for tree display
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UfedTreeEntryResult {
    /// Path within the UFED container
    pub path: String,
    /// Filename
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// File size (0 for directories)
    pub size: u64,
    /// UFED-specific type (file, folder, extraction, etc.)
    pub entry_type: String,
    /// Associated hash if available
    pub hash: Option<String>,
    /// Modified timestamp if available
    pub modified: Option<String>,
}

/// Get the file tree for a UFED container
#[tauri::command]
pub async fn ufed_get_tree(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<Vec<UfedTreeEntryResult>, String> {
    debug!("ufed_get_tree: {}", containerPath);
    tauri::async_runtime::spawn_blocking(move || {
        let entries = ufed::get_tree(&containerPath)
            .map_err(|e| e.to_string())?;
        
        Ok(entries.into_iter().map(|e| UfedTreeEntryResult {
            path: e.path,
            name: e.name,
            is_dir: e.is_dir,
            size: e.size,
            entry_type: e.entry_type,
            hash: e.hash,
            modified: e.modified,
        }).collect())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get children at a specific path in a UFED container (LAZY LOADING)
/// 
/// For large UFED ZIP files, this enables on-demand loading as the user
/// expands folders, preventing the app from locking up on 18GB+ extractions.
#[tauri::command]
pub async fn ufed_get_children(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    parentPath: String,
) -> Result<Vec<UfedTreeEntryResult>, String> {
    debug!("ufed_get_children: {} at {}", containerPath, parentPath);
    tauri::async_runtime::spawn_blocking(move || {
        let entries = ufed::get_children(&containerPath, &parentPath)
            .map_err(|e| e.to_string())?;
        
        Ok(entries.into_iter().map(|e| UfedTreeEntryResult {
            path: e.path,
            name: e.name,
            is_dir: e.is_dir,
            size: e.size,
            entry_type: e.entry_type,
            hash: e.hash,
            modified: e.modified,
        }).collect())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get the entry count for a UFED container without loading all entries
/// 
/// Useful for showing progress or determining if lazy loading is needed.
#[tauri::command]
pub async fn ufed_get_entry_count(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<usize, String> {
    debug!("ufed_get_entry_count: {}", containerPath);
    tauri::async_runtime::spawn_blocking(move || {
        ufed::get_entry_count(&containerPath)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
