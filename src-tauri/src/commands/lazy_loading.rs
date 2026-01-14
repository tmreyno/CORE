// =============================================================================
// CORE-FFX - Lazy Loading Commands
// =============================================================================

//! Unified lazy loading commands for all container types.
//! Provides a standardized interface with configurable batch sizes.

use tracing::debug;

use crate::ad1;
use crate::archive;
use crate::common::lazy_loading::{LazyLoadConfig, LazyTreeEntry, LazyLoadResult, ContainerSummary};
use crate::ufed;

/// Detect container type from file extension and path
pub fn detect_container_type(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    
    // Check for UFED types first (they can be inside folders)
    if lower.ends_with(".ufd") || lower.ends_with(".ufdr") || lower.ends_with(".ufdx") {
        return "ufed";
    }
    
    // UFED can also be detected by folder contents
    let path_obj = std::path::Path::new(path);
    if path_obj.is_dir() {
        // Check for UFED XML file
        let ufdr_xml = path_obj.join("report.xml");
        if ufdr_xml.exists() {
            return "ufed";
        }
    }
    
    // Check for ZIP inside UFED folder
    if ufed::is_ufed(path) {
        return "ufed";
    }
    
    // AD1
    if lower.ends_with(".ad1") {
        return "ad1";
    }
    
    // EWF formats
    if lower.ends_with(".e01") || lower.ends_with(".l01") || 
       lower.ends_with(".ex01") || lower.ends_with(".lx01") {
        return "ewf";
    }
    
    // Archive formats
    if lower.ends_with(".zip") {
        return "zip";
    }
    if lower.ends_with(".7z") || (lower.contains(".7z.") && lower.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false)) {
        return "7z";
    }
    if lower.ends_with(".rar") {
        return "rar";
    }
    if lower.ends_with(".tar") || lower.ends_with(".tar.gz") || lower.ends_with(".tgz") ||
       lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") || lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        return "tar";
    }
    
    "unknown"
}

/// Get container summary with lazy loading recommendation
/// Call this FIRST to determine if lazy loading should be used
#[tauri::command]
pub async fn lazy_get_container_summary(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<ContainerSummary, String> {
    debug!("lazy_get_container_summary: {}", containerPath);
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = detect_container_type(&containerPath);
        
        // Get file size
        let total_size = std::fs::metadata(&containerPath)
            .map(|m| m.len())
            .unwrap_or(0);
        
        match container_type {
            "ad1" => {
                // AD1 - estimate count from file size (rough approximation)
                // ~500 bytes per entry average for AD1
                let estimated_count = (total_size / 500) as usize;
                Ok(ContainerSummary::new(&containerPath, "ad1", total_size, estimated_count))
            }
            "ufed" => {
                let count = ufed::get_entry_count(&containerPath).unwrap_or(0);
                Ok(ContainerSummary::new(&containerPath, "ufed", total_size, count))
            }
            "zip" => {
                let count = archive::extraction::get_zip_entry_count(&containerPath).unwrap_or(0);
                Ok(ContainerSummary::new(&containerPath, "zip", total_size, count))
            }
            "ewf" => {
                // EWF - always recommend lazy loading for disk images
                let mut summary = ContainerSummary::new(&containerPath, "ewf", total_size, 0);
                summary.lazy_loading_recommended = true;
                Ok(summary)
            }
            "7z" | "rar" | "tar" => {
                // These formats don't have quick count - recommend lazy
                let mut summary = ContainerSummary::new(&containerPath, container_type, total_size, 0);
                summary.lazy_loading_recommended = true;
                Ok(summary)
            }
            _ => Err(format!("Unknown container type: {}", containerPath)),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get root-level entries for a container (first level only)
/// Use this for initial tree population
#[tauri::command]
pub async fn lazy_get_root_children(
    #[allow(non_snake_case)]
    containerPath: String,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<LazyLoadResult, String> {
    debug!("lazy_get_root_children: {} offset={:?} limit={:?}", containerPath, offset, limit);
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = detect_container_type(&containerPath);
        let config = crate::common::lazy_loading::get_config();
        let batch_size = limit.map(|l| l as usize).unwrap_or(config.batch_size);
        let skip = offset.unwrap_or(0) as usize;
        
        match container_type {
            "ad1" => {
                // AD1 - use get_children with empty parent path for root
                let children = ad1::get_children(&containerPath, "")
                    .map_err(|e| e.to_string())?;
                
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.into_iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        if c.is_dir {
                            let mut entry = LazyTreeEntry::directory(
                                c.item_addr.map(|a| format!("ad1_{:x}", a)).unwrap_or_else(|| c.path.clone()),
                                c.name.clone(),
                                c.path.clone(),
                            );
                            if let Some(modified) = c.modified {
                                entry = entry.with_modified(modified);
                            }
                            entry
                        } else {
                            let mut entry = LazyTreeEntry::file(
                                c.item_addr.map(|a| format!("ad1_{:x}", a)).unwrap_or_else(|| c.path.clone()),
                                c.name.clone(),
                                c.path.clone(),
                                c.size,
                            );
                            if let Some(modified) = c.modified {
                                entry = entry.with_modified(modified);
                            }
                            entry
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "ufed" => {
                let children = ufed::get_root_children(&containerPath)
                    .map_err(|e| e.to_string())?;
                
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.into_iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        if c.is_dir {
                            LazyTreeEntry::directory(
                                c.path.clone(),
                                c.name.clone(),
                                c.path.clone(),
                            )
                        } else {
                            LazyTreeEntry::file(
                                c.path.clone(),
                                c.name.clone(),
                                c.path.clone(),
                                c.size,
                            )
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "zip" => {
                let children = archive::extraction::get_zip_root_entries(&containerPath)
                    .map_err(|e| e.to_string())?;
                
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.into_iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        // Extract name from path
                        let name = std::path::Path::new(&c.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&c.path)
                            .to_string();
                        
                        if c.is_directory {
                            LazyTreeEntry::directory(
                                c.path.clone(),
                                name,
                                c.path.clone(),
                            ).with_modified(c.last_modified.clone())
                        } else {
                            LazyTreeEntry::file(
                                c.path.clone(),
                                name,
                                c.path.clone(),
                                c.size,
                            ).with_modified(c.last_modified.clone())
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            _ => Err(format!("Lazy loading not yet implemented for: {}", container_type)),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get children of a specific path within a container (lazy loading)
/// Use this when user expands a folder in the tree
#[tauri::command]
pub async fn lazy_get_children(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    parentPath: String,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<LazyLoadResult, String> {
    debug!("lazy_get_children: {} parent={} offset={:?} limit={:?}", containerPath, parentPath, offset, limit);
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = detect_container_type(&containerPath);
        let config = crate::common::lazy_loading::get_config();
        let batch_size = limit.map(|l| l as usize).unwrap_or(config.batch_size);
        let skip = offset.unwrap_or(0) as usize;
        
        match container_type {
            "ad1" => {
                let children = ad1::get_children(&containerPath, &parentPath)
                    .map_err(|e| e.to_string())?;
                
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.into_iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        if c.is_dir {
                            let mut entry = LazyTreeEntry::directory(
                                c.item_addr.map(|a| format!("ad1_{:x}", a)).unwrap_or_else(|| c.path.clone()),
                                c.name.clone(),
                                c.path.clone(),
                            );
                            if let Some(modified) = c.modified {
                                entry = entry.with_modified(modified);
                            }
                            entry
                        } else {
                            let mut entry = LazyTreeEntry::file(
                                c.item_addr.map(|a| format!("ad1_{:x}", a)).unwrap_or_else(|| c.path.clone()),
                                c.name.clone(),
                                c.path.clone(),
                                c.size,
                            );
                            if let Some(modified) = c.modified {
                                entry = entry.with_modified(modified);
                            }
                            entry
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "ufed" => {
                let children = ufed::get_children(&containerPath, &parentPath)
                    .map_err(|e| e.to_string())?;
                
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.into_iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        if c.is_dir {
                            LazyTreeEntry::directory(
                                c.path.clone(),
                                c.name.clone(),
                                c.path.clone(),
                            )
                        } else {
                            LazyTreeEntry::file(
                                c.path.clone(),
                                c.name.clone(),
                                c.path.clone(),
                                c.size,
                            )
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "zip" => {
                let children = archive::extraction::get_zip_children_at_path(&containerPath, &parentPath)
                    .map_err(|e| e.to_string())?;
                
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.into_iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        // Extract name from path
                        let name = std::path::Path::new(&c.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&c.path)
                            .to_string();
                        
                        if c.is_directory {
                            LazyTreeEntry::directory(
                                c.path.clone(),
                                name,
                                c.path.clone(),
                            ).with_modified(c.last_modified.clone())
                        } else {
                            LazyTreeEntry::file(
                                c.path.clone(),
                                name,
                                c.path.clone(),
                                c.size,
                            ).with_modified(c.last_modified.clone())
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            _ => Err(format!("Lazy loading not yet implemented for: {}", container_type)),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get current lazy loading settings
#[tauri::command]
pub async fn lazy_get_settings() -> Result<LazyLoadConfig, String> {
    Ok(crate::common::lazy_loading::get_config())
}

/// Update lazy loading settings
#[tauri::command]
pub async fn lazy_update_settings(
    #[allow(non_snake_case)]
    batchSize: Option<u64>,
    #[allow(non_snake_case)]
    largeContainerThreshold: Option<u64>,
    #[allow(non_snake_case)]
    paginationThreshold: Option<u64>,
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
    
    crate::common::lazy_loading::update_config(config.clone());
    Ok(config)
}
