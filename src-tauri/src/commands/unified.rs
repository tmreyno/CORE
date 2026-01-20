// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//!
//! Single unified Tauri commands for all container types.
//! These commands replace the separate container_*, archive_*, ufed_* commands.
//!
//! ## Frontend API
//!
//! ```typescript
//! // Get container summary
//! const summary = await invoke("unified_get_summary", { containerPath });
//!
//! // Get root children
//! const result = await invoke("unified_get_children", { 
//!     containerPath, 
//!     parentPath: "",  // empty for root
//!     offset: 0,
//!     limit: 100 
//! });
//!
//! // Get children at path
//! const result = await invoke("unified_get_children", { 
//!     containerPath, 
//!     parentPath: "/some/folder",
//!     offset: 0,
//!     limit: 100 
//! });
//! ```

use tracing::{debug, info};

use crate::common::lazy_loading::{LazyLoadConfig, LazyLoadResult, ContainerSummary};
use crate::containers::unified::{self, ContainerType};

// =============================================================================
// Unified Commands
// =============================================================================

/// Get container summary with lazy loading recommendation
/// 
/// This should be called FIRST when opening a container to get:
/// - Container type (ad1, ewf, zip, etc)
/// - Total size
/// - Estimated entry count
/// - Whether lazy loading is recommended
#[tauri::command]
pub async fn unified_get_summary(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<ContainerSummary, String> {
    info!("unified_get_summary: {}", containerPath);
    
    tauri::async_runtime::spawn_blocking(move || {
        unified::get_summary(&containerPath)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get container type for a path
/// 
/// Returns the detected container type as a string
#[tauri::command]
pub async fn unified_detect_type(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<String, String> {
    let container_type = ContainerType::detect(&containerPath);
    Ok(format!("{:?}", container_type).to_lowercase())
}

/// Get entry count for a container
/// 
/// This may be fast (from headers) or slow (requires scanning)
/// depending on the container type.
#[tauri::command]
pub async fn unified_get_entry_count(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<usize, String> {
    debug!("unified_get_entry_count: {}", containerPath);
    
    tauri::async_runtime::spawn_blocking(move || {
        unified::get_entry_count(&containerPath)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get children at a path within a container
/// 
/// This is the MAIN API for tree navigation. Works for all container types:
/// - AD1, E01/EWF, UFED, ZIP, 7z, TAR, RAR, Raw
/// 
/// # Arguments
/// 
/// * `containerPath` - Path to the container file
/// * `parentPath` - Path within the container ("" for root)
/// * `offset` - Pagination offset (default: 0)
/// * `limit` - Maximum entries to return (default: from config, typically 100)
/// 
/// # Returns
/// 
/// `LazyLoadResult` with:
/// - `entries`: Array of `LazyTreeEntry`
/// - `total_count`: Total children at this path
/// - `has_more`: Whether there are more entries beyond this batch
/// - `next_offset`: Offset for next batch if `has_more` is true
#[tauri::command]
pub async fn unified_get_children(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    parentPath: Option<String>,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<LazyLoadResult, String> {
    let parent = parentPath.unwrap_or_default();
    debug!("unified_get_children: {} parent='{}' offset={:?} limit={:?}", 
           containerPath, parent, offset, limit);
    
    tauri::async_runtime::spawn_blocking(move || {
        let offset = offset.map(|o| o as usize);
        let limit = limit.map(|l| l as usize);
        
        if parent.is_empty() {
            unified::get_root_children(&containerPath, offset, limit)
        } else {
            unified::get_children(&containerPath, &parent, offset, limit)
        }
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get children with explicit container type
/// 
/// Use this when you already know the container type to skip detection.
#[tauri::command]
pub async fn unified_get_children_typed(
    #[allow(non_snake_case)]
    containerType: String,
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    parentPath: Option<String>,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<LazyLoadResult, String> {
    let parent = parentPath.unwrap_or_default();
    debug!("unified_get_children_typed: {} type={} parent='{}' offset={:?} limit={:?}", 
           containerPath, containerType, parent, offset, limit);
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = match containerType.to_lowercase().as_str() {
            "ad1" => ContainerType::Ad1,
            "ewf" | "e01" | "l01" => ContainerType::Ewf,
            "ufed" | "ufd" | "ufdr" => ContainerType::Ufed,
            "zip" => ContainerType::Zip,
            "7z" | "sevenz" | "sevenzip" => ContainerType::SevenZip,
            "tar" | "tgz" | "tar.gz" => ContainerType::Tar,
            "rar" => ContainerType::Rar,
            "raw" | "img" | "dd" => ContainerType::Raw,
            _ => ContainerType::detect(&containerPath),
        };
        
        let offset = offset.map(|o| o as usize);
        let limit = limit.map(|l| l as usize);
        
        unified::get_children_typed(container_type, &containerPath, &parent, offset, limit)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get current lazy loading settings
#[tauri::command]
pub async fn unified_get_settings() -> Result<LazyLoadConfig, String> {
    Ok(crate::common::lazy_loading::get_config())
}

/// Update lazy loading settings
#[tauri::command]
pub async fn unified_update_settings(
    #[allow(non_snake_case)]
    batchSize: Option<u64>,
    #[allow(non_snake_case)]
    largeContainerThreshold: Option<u64>,
    #[allow(non_snake_case)]
    paginationThreshold: Option<u64>,
    enabled: Option<bool>,
) -> Result<LazyLoadConfig, String> {
    let mut config = crate::common::lazy_loading::get_config();
    
    if let Some(v) = batchSize {
        config.batch_size = v as usize;
    }
    if let Some(v) = largeContainerThreshold {
        config.large_container_threshold = v as usize;
    }
    if let Some(v) = paginationThreshold {
        config.pagination_threshold = v as usize;
    }
    if let Some(v) = enabled {
        config.enabled = v;
    }
    
    crate::common::lazy_loading::update_config(config);
    Ok(config)
}

// =============================================================================
// Compatibility Wrappers (for gradual migration)
// =============================================================================

/// Legacy compatibility: Get container tree
/// 
/// DEPRECATED: Use `unified_get_children` with parentPath="" instead.
/// This wrapper exists for backward compatibility during migration.
#[tauri::command]
pub async fn unified_get_tree(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<LazyLoadResult, String> {
    debug!("unified_get_tree (legacy): {}", containerPath);
    unified_get_children(containerPath, None, None, Some(10000)).await
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_container_type_parsing() {
        // Just test the type string parsing logic
        let types = vec![
            ("ad1", ContainerType::Ad1),
            ("AD1", ContainerType::Ad1),
            ("ewf", ContainerType::Ewf),
            ("e01", ContainerType::Ewf),
            ("zip", ContainerType::Zip),
            ("7z", ContainerType::SevenZip),
            ("tar", ContainerType::Tar),
            ("rar", ContainerType::Rar),
        ];
        
        for (input, expected) in types {
            let result = match input.to_lowercase().as_str() {
                "ad1" => ContainerType::Ad1,
                "ewf" | "e01" | "l01" => ContainerType::Ewf,
                "zip" => ContainerType::Zip,
                "7z" | "sevenz" | "sevenzip" => ContainerType::SevenZip,
                "tar" | "tgz" | "tar.gz" => ContainerType::Tar,
                "rar" => ContainerType::Rar,
                _ => ContainerType::Unknown,
            };
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
}
