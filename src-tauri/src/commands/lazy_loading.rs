// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified lazy loading commands for all container types.
//! Provides a standardized interface with configurable batch sizes.

use tracing::debug;

use crate::ad1;
use crate::archive;
use crate::common::lazy_loading::{LazyLoadConfig, LazyTreeEntry, LazyLoadResult, ContainerSummary};
use crate::ufed;

/// Format a POSIX timestamp (i64 seconds) to ISO-like string
fn format_timestamp(secs: i64) -> String {
    use chrono::{DateTime, Utc};
    DateTime::<Utc>::from_timestamp(secs, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Detect container type from file extension and path
pub fn detect_container_type(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    
    // Check for UFED types first (they can be inside folders)
    if lower.ends_with(".ufd") || lower.ends_with(".ufdr") || lower.ends_with(".ufdx") {
        debug!("detect_container_type: {} -> ufed (by extension)", path);
        return "ufed";
    }
    
    // UFED can also be detected by folder contents
    let path_obj = std::path::Path::new(path);
    if path_obj.is_dir() {
        // Check for UFED XML file
        let ufdr_xml = path_obj.join("report.xml");
        if ufdr_xml.exists() {
            debug!("detect_container_type: {} -> ufed (report.xml)", path);
            return "ufed";
        }
    }
    
    // Check for ZIP inside UFED folder (with sibling UFD)
    if ufed::is_ufed(path) {
        debug!("detect_container_type: {} -> ufed (is_ufed check)", path);
        return "ufed";
    }
    
    // AD1
    if lower.ends_with(".ad1") {
        debug!("detect_container_type: {} -> ad1", path);
        return "ad1";
    }
    
    // EWF formats - separate L01 (logical) from E01 (physical)
    if lower.ends_with(".l01") || lower.ends_with(".lx01") {
        debug!("detect_container_type: {} -> l01", path);
        return "l01";
    }
    if lower.ends_with(".e01") || lower.ends_with(".ex01") {
        debug!("detect_container_type: {} -> ewf", path);
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
    
    // Memory dumps (check before raw - more specific patterns)
    if lower.contains("_mem.raw") || lower.ends_with(".mem") || lower.ends_with(".vmem") ||
       lower.ends_with(".dmp") || lower.contains("_memdump") ||
       lower.contains(".hiberfil") || lower.contains(".pagefile") {
        debug!("detect_container_type: {} -> memory", path);
        return "memory";
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
            "l01" => {
                // L01 - logical evidence file, parse ltree for entry count
                let count = match crate::ewf::parse_l01_file_tree(&containerPath) {
                    Ok(tree) => tree.entries.len(),
                    Err(_) => 0,
                };
                let mut summary = ContainerSummary::new(&containerPath, "l01", total_size, count);
                summary.lazy_loading_recommended = count > 1000;
                Ok(summary)
            }
            "7z" | "rar" | "tar" => {
                // Use libarchive to get actual entry count
                let count = archive::libarchive_list_all(&containerPath)
                    .map(|entries| entries.len())
                    .unwrap_or(0);
                let mut summary = ContainerSummary::new(&containerPath, container_type, total_size, count);
                summary.lazy_loading_recommended = count > 1000;
                Ok(summary)
            }
            "memory" => {
                // Memory dumps — single flat binary, no tree structure
                Ok(ContainerSummary::new(&containerPath, "memory", total_size, 1))
            }
            _ => {
                // Unknown or unsupported — return basic summary with 0 entries
                Ok(ContainerSummary::new(&containerPath, container_type, total_size, 0))
            }
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
                debug!("lazy_get_root_children: ufed returned {} children", total);
                
                let entries: Vec<LazyTreeEntry> = children.into_iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        debug!("  child: {} is_dir={} size={}", c.name, c.is_dir, c.size);
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
                
                debug!("lazy_get_root_children: returning {} entries", entries.len());
                Ok(LazyLoadResult::new(entries, total))
            }
            "zip" => {
                // Use fast ZipIndex for O(1) lookups
                let index = archive::ZipIndex::get_or_create(&containerPath)
                    .map_err(|e| e.to_string())?;
                
                let children = index.get_root_entries();
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        if c.is_directory {
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
            "7z" | "rar" | "tar" => {
                // Use libarchive for non-ZIP archive formats
                let all_entries = archive::libarchive_list_all(&containerPath)
                    .map_err(|e| e.to_string())?;
                
                // Filter root-level entries (no separator in path after trimming)
                let root_entries: Vec<_> = all_entries.iter()
                    .filter(|e| e.parent.is_empty() || e.parent == "/")
                    .collect();
                
                let total = root_entries.len();
                let entries: Vec<LazyTreeEntry> = root_entries.iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|e| {
                        if e.is_dir {
                            LazyTreeEntry::directory(
                                e.path.clone(),
                                e.name.clone(),
                                e.path.clone(),
                            )
                        } else {
                            LazyTreeEntry::file(
                                e.path.clone(),
                                e.name.clone(),
                                e.path.clone(),
                                e.size,
                            )
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "ewf" => {
                // EWF containers - use VFS to list root
                let vfs = crate::ewf::vfs::EwfVfs::open(&containerPath)
                    .map_err(|e| format!("Failed to open EWF VFS: {}", e))?;
                
                let dir_entries = crate::common::vfs::VirtualFileSystem::readdir(&vfs, "/")
                    .map_err(|e| format!("Failed to read EWF root: {}", e))?;
                
                let total = dir_entries.len();
                let entries: Vec<LazyTreeEntry> = dir_entries.iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|e| {
                        if e.is_directory {
                            LazyTreeEntry::directory(
                                format!("/{}", e.name),
                                e.name.clone(),
                                format!("/{}", e.name),
                            )
                        } else {
                            LazyTreeEntry::file(
                                format!("/{}", e.name),
                                e.name.clone(),
                                format!("/{}", e.name),
                                0, // Size not available from DirEntry
                            )
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "l01" => {
                // L01 logical evidence - parse ltree for file tree
                let tree = crate::ewf::parse_l01_file_tree(&containerPath)
                    .map_err(|e| format!("Failed to parse L01 file tree: {}", e))?;
                
                let root_entries = tree.root_entries();
                let total = root_entries.len();
                let entries: Vec<LazyTreeEntry> = root_entries.iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|e| {
                        if e.is_directory {
                            let mut entry = LazyTreeEntry::directory(
                                e.path.clone(),
                                e.name.clone(),
                                e.path.clone(),
                            );
                            if e.modification_time != 0 {
                                entry = entry.with_modified(format_timestamp(e.modification_time));
                            }
                            entry
                        } else {
                            let mut entry = LazyTreeEntry::file(
                                e.path.clone(),
                                e.name.clone(),
                                e.path.clone(),
                                e.size,
                            );
                            if e.modification_time != 0 {
                                entry = entry.with_modified(format_timestamp(e.modification_time));
                            }
                            if let Some(ref hash) = e.md5_hash {
                                entry = entry.with_hash(hash.clone());
                            }
                            entry
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "memory" => {
                // Memory dumps are flat binary blobs — no directory structure to browse.
                // Return a single virtual entry representing the raw dump.
                let filename = std::path::Path::new(&containerPath)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("memory.raw")
                    .to_string();
                let total_size = std::fs::metadata(&containerPath)
                    .map(|m| m.len())
                    .unwrap_or(0);
                let entry = LazyTreeEntry::file(
                    containerPath.clone(),
                    filename,
                    "/".to_string(),
                    total_size,
                );
                Ok(LazyLoadResult::new(vec![entry], 1))
            }
            _ => {
                // Unknown format — return empty result rather than an error
                // so the UI can still display the container with no children.
                debug!("lazy_get_root_children: unknown container type '{}' for {}", container_type, containerPath);
                Ok(LazyLoadResult::new(Vec::new(), 0))
            }
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
                // Use fast ZipIndex for O(1) lookups
                let index = archive::ZipIndex::get_or_create(&containerPath)
                    .map_err(|e| e.to_string())?;
                
                let children = index.get_children(&parentPath)
                    .cloned()
                    .unwrap_or_default();
                
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.into_iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|c| {
                        if c.is_directory {
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
            "7z" | "rar" | "tar" => {
                // Use libarchive for non-ZIP archive formats
                let all_entries = archive::libarchive_list_all(&containerPath)
                    .map_err(|e| e.to_string())?;
                
                // Normalize parent path for comparison
                let parent_normalized = parentPath.trim_end_matches('/');
                
                // Filter children of the given parent
                let children: Vec<_> = all_entries.iter()
                    .filter(|e| {
                        let entry_parent = e.parent.trim_end_matches('/');
                        entry_parent == parent_normalized
                    })
                    .collect();
                
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|e| {
                        if e.is_dir {
                            LazyTreeEntry::directory(
                                e.path.clone(),
                                e.name.clone(),
                                e.path.clone(),
                            )
                        } else {
                            LazyTreeEntry::file(
                                e.path.clone(),
                                e.name.clone(),
                                e.path.clone(),
                                e.size,
                            )
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "ewf" => {
                // EWF - use VFS for children
                let vfs = crate::ewf::vfs::EwfVfs::open(&containerPath)
                    .map_err(|e| format!("Failed to open EWF VFS: {}", e))?;
                
                let dir_entries = crate::common::vfs::VirtualFileSystem::readdir(&vfs, &parentPath)
                    .map_err(|e| format!("Failed to read EWF directory: {}", e))?;
                
                let total = dir_entries.len();
                let entries: Vec<LazyTreeEntry> = dir_entries.iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|e| {
                        let full_path = format!("{}/{}", parentPath.trim_end_matches('/'), e.name);
                        if e.is_directory {
                            LazyTreeEntry::directory(
                                full_path.clone(),
                                e.name.clone(),
                                full_path,
                            )
                        } else {
                            LazyTreeEntry::file(
                                full_path.clone(),
                                e.name.clone(),
                                full_path,
                                0,
                            )
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            "l01" => {
                // L01 logical evidence - parse ltree for children at path
                let tree = crate::ewf::parse_l01_file_tree(&containerPath)
                    .map_err(|e| format!("Failed to parse L01 file tree: {}", e))?;
                
                let children = tree.children_at_path(&parentPath);
                let total = children.len();
                let entries: Vec<LazyTreeEntry> = children.iter()
                    .skip(skip)
                    .take(batch_size)
                    .map(|e| {
                        if e.is_directory {
                            let mut entry = LazyTreeEntry::directory(
                                e.path.clone(),
                                e.name.clone(),
                                e.path.clone(),
                            );
                            if e.modification_time != 0 {
                                entry = entry.with_modified(format_timestamp(e.modification_time));
                            }
                            entry
                        } else {
                            let mut entry = LazyTreeEntry::file(
                                e.path.clone(),
                                e.name.clone(),
                                e.path.clone(),
                                e.size,
                            );
                            if e.modification_time != 0 {
                                entry = entry.with_modified(format_timestamp(e.modification_time));
                            }
                            if let Some(ref hash) = e.md5_hash {
                                entry = entry.with_hash(hash.clone());
                            }
                            entry
                        }
                    })
                    .collect();
                
                Ok(LazyLoadResult::new(entries, total))
            }
            _ => {
                debug!("lazy_get_children: no children for type '{}' at {}", container_type, parentPath);
                Ok(LazyLoadResult::new(Vec::new(), 0))
            }
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
    
    crate::common::lazy_loading::update_config(config);
    Ok(config)
}
