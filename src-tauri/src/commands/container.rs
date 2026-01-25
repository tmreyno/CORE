// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 container operations including tree navigation, data reading, and verification.

use tracing::{debug, info, warn, error};

use crate::ad1;
use crate::containers;

// =============================================================================
// V1 Container Commands (Original Implementation)
// =============================================================================

#[tauri::command]
pub fn logical_info(
    #[allow(non_snake_case)]
    inputPath: String,
    #[allow(non_snake_case)]
    includeTree: bool,
) -> Result<containers::ContainerInfo, String> {
    containers::info(&inputPath, includeTree)
}

/// Fast info - only reads headers, doesn't parse full item trees
/// Use this for quick container listing/display
#[tauri::command]
pub fn logical_info_fast(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<containers::ContainerInfo, String> {
    containers::info_fast(&inputPath)
}

/// Get only stored hashes - minimal parsing, fastest option
/// Use this for pre-loading hash values during project setup
#[tauri::command]
pub fn get_stored_hashes_only(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<Vec<containers::StoredHash>, String> {
    containers::get_stored_hashes_only(&inputPath)
}

/// Get just the tree entries for a container - faster than full info
/// Use this for populating the evidence tree UI
#[tauri::command]
pub async fn container_get_tree(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<Vec<ad1::TreeEntry>, String> {
    info!("container_get_tree called for: {}", containerPath);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            let result = ad1::get_tree(&containerPath);
            match &result {
                Ok(entries) => info!("get_tree returned {} entries", entries.len()),
                Err(e) => error!("get_tree failed: {}", e),
            }
            result.map_err(|e| e.to_string())
        } else {
            let msg = format!("Container type not supported for tree listing: {}", containerPath);
            warn!("{}", msg);
            Err(msg)
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get immediate children at a specific path within a container (lazy loading)
/// Pass empty string for parentPath to get root-level entries
#[tauri::command]
pub async fn container_get_children(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    parentPath: String,
) -> Result<Vec<ad1::TreeEntry>, String> {
    debug!("container_get_children: {} at '{}'", containerPath, parentPath);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::get_children(&containerPath, &parentPath).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get immediate children at a direct address (fastest method - lazy loading, no full tree parsing)
#[tauri::command]
pub async fn container_get_children_at_addr(
    #[allow(non_snake_case)]
    containerPath: String,
    addr: u64,
    #[allow(non_snake_case)]
    parentPath: Option<String>,
) -> Result<Vec<ad1::TreeEntry>, String> {
    debug!("container_get_children_at_addr: {} addr={}", containerPath, addr);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            // Use lazy loading for fast performance on large containers
            let parent = parentPath.unwrap_or_default();
            ad1::get_children_at_addr_lazy(&containerPath, addr, &parent).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read file data by direct address (fastest method - no path parsing)
#[tauri::command]
pub async fn container_read_entry_by_addr(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    dataAddr: u64,
    size: u64,
) -> Result<Vec<u8>, String> {
    debug!("container_read_entry_by_addr: {} addr={} size={}", containerPath, dataAddr, size);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::read_entry_data_by_addr(&containerPath, dataAddr, size).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read file content from within a forensic container (AD1, etc.)
/// Returns raw bytes of the file entry - uses lazy loading for speed
#[tauri::command]
pub async fn container_read_entry(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    entryPath: String,
) -> Result<Vec<u8>, String> {
    debug!("container_read_entry: {} from {}", entryPath, containerPath);
    tauri::async_runtime::spawn_blocking(move || {
        // Detect container type and read accordingly
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            // Use lazy read for better performance
            ad1::read_entry_data(&containerPath, &entryPath).map_err(|e| e.to_string())
        } else {
            Err(format!("Unsupported container type for: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read a chunk of file content from within a forensic container
#[tauri::command]
pub async fn container_read_entry_chunk(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    entryPath: String,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::read_entry_chunk(&containerPath, &entryPath, offset, size).map_err(|e| e.to_string())
        } else {
            Err(format!("Unsupported container type for: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Extract an entry from a container to a temp file for preview
/// 
/// This is a unified command that handles AD1, archives, and VFS containers.
/// Returns the path to the extracted temp file which can then be used with
/// document viewers (PDF, DOCX, etc.)
#[tauri::command]
pub async fn container_extract_entry_to_temp(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    entryPath: String,
    #[allow(non_snake_case)]
    entrySize: u64,
    #[allow(non_snake_case)]
    isVfsEntry: bool,
    #[allow(non_snake_case)]
    isArchiveEntry: bool,
    #[allow(non_snake_case)]
    dataAddr: Option<u64>,
) -> Result<String, String> {
    debug!("container_extract_entry_to_temp: {} / {} (size={}, vfs={}, archive={}, addr={:?})", 
           containerPath, entryPath, entrySize, isVfsEntry, isArchiveEntry, dataAddr);
    
    tauri::async_runtime::spawn_blocking(move || {
        use std::io::Write;
        
        // Create temp directory for extracted files
        let temp_dir = std::env::temp_dir().join("core-ffx-preview");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        
        // Generate output filename from entry path (preserving extension)
        let entry_filename = std::path::Path::new(&entryPath)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("preview");
        
        // Add unique prefix to avoid collisions
        let unique_name = format!("{}_{}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            entry_filename
        );
        
        let output_path = temp_dir.join(&unique_name);
        
        // Read content based on container type
        let data = if isArchiveEntry {
            // Archive entry - use libarchive unified backend
            use crate::archive;
            
            archive::libarchive_read_file(&containerPath, &entryPath)
                .map_err(|e| format!("Failed to read archive entry: {}", e))?
        } else if isVfsEntry {
            // VFS entry (E01/Raw) - use VFS read
            use crate::ewf;
            use crate::raw;
            use crate::common::vfs::VirtualFileSystem;
            
            if ewf::is_ewf(&containerPath).unwrap_or(false) {
                let vfs = ewf::vfs::EwfVfs::open(&containerPath)
                    .map_err(|e| format!("Failed to open E01: {:?}", e))?;
                vfs.read(&entryPath, 0, entrySize as usize)
                    .map_err(|e| format!("Failed to read VFS file: {:?}", e))?
            } else if raw::is_raw(&containerPath).unwrap_or(false) {
                let vfs = raw::vfs::RawVfs::open_filesystem(&containerPath)
                    .or_else(|_| raw::vfs::RawVfs::open(&containerPath))
                    .map_err(|e| format!("Failed to open raw: {:?}", e))?;
                vfs.read(&entryPath, 0, entrySize as usize)
                    .map_err(|e| format!("Failed to read raw file: {:?}", e))?
            } else {
                return Err(format!("Unsupported VFS container: {}", containerPath));
            }
        } else if ad1::is_ad1(&containerPath).unwrap_or(false) {
            // AD1 entry - prefer address-based read if available
            if let Some(addr) = dataAddr {
                ad1::read_entry_data_by_addr(&containerPath, addr, entrySize)
                    .map_err(|e| format!("Failed to read AD1 by address: {}", e))?
            } else {
                ad1::read_entry_data(&containerPath, &entryPath)
                    .map_err(|e| format!("Failed to read AD1 entry: {}", e))?
            }
        } else {
            return Err(format!("Unsupported container type: {}", containerPath));
        };
        
        // Write to temp file
        let mut file = std::fs::File::create(&output_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        file.write_all(&data)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        
        debug!("Extracted {} bytes to: {:?}", data.len(), output_path);
        
        Ok(output_path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get metadata for a specific entry in a forensic container
#[tauri::command]
pub fn container_get_entry_info(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    entryPath: String,
) -> Result<ad1::TreeEntry, String> {
    if ad1::is_ad1(&containerPath).unwrap_or(false) {
        ad1::get_entry_info(&containerPath, &entryPath).map_err(|e| e.to_string())
    } else {
        Err(format!("Unsupported container type for: {}", containerPath))
    }
}

// =============================================================================
// V2 Container Commands (Based on libad1 C implementation)
// =============================================================================

/// Get root children using V2 implementation (improved from libad1)
#[tauri::command]
pub async fn container_get_root_children_v2(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<Vec<ad1::TreeEntry>, String> {
    debug!("container_get_root_children_v2: {}", containerPath);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::get_root_children_v2(&containerPath).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get children at address using V2 implementation
#[tauri::command]
pub async fn container_get_children_at_addr_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    addr: u64,
    #[allow(non_snake_case)]
    parentPath: String,
) -> Result<Vec<ad1::TreeEntry>, String> {
    debug!("container_get_children_at_addr_v2: {} addr={} parent={}", 
           containerPath, addr, parentPath);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::get_children_at_addr_v2(&containerPath, addr, &parentPath)
                .map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read file data using V2 implementation
#[tauri::command]
pub async fn container_read_file_data_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    itemAddr: u64,
) -> Result<Vec<u8>, String> {
    debug!("container_read_file_data_v2: {} addr={}", containerPath, itemAddr);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::read_file_data_v2(&containerPath, itemAddr).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get item info using V2 implementation
#[tauri::command]
pub async fn container_get_item_info_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    addr: u64,
) -> Result<ad1::TreeEntry, String> {
    debug!("container_get_item_info_v2: {} addr={}", containerPath, addr);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::get_item_info_v2(&containerPath, addr).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get item metadata on-demand (V2 implementation)
/// Used for lazy loading of hashes, timestamps, and attributes
#[tauri::command]
pub async fn container_get_item_metadata_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    itemAddr: u64,
) -> Result<ad1::ItemMetadata, String> {
    debug!("container_get_item_metadata_v2: {} addr={}", containerPath, itemAddr);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::get_item_metadata_v2(&containerPath, itemAddr).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get metadata for multiple items on-demand (V2 implementation - batch)
/// More efficient than calling container_get_item_metadata_v2 multiple times
#[tauri::command]
pub async fn container_get_items_metadata_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    itemAddrs: Vec<u64>,
) -> Result<Vec<ad1::ItemMetadata>, String> {
    debug!("container_get_items_metadata_v2: {} count={}", containerPath, itemAddrs.len());
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::get_items_metadata_v2(&containerPath, &itemAddrs).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get container status (segment availability for partial AD1 support)
#[tauri::command]
pub async fn container_get_status_v2(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<ad1::ContainerStatus, String> {
    debug!("container_get_status_v2: {}", containerPath);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::get_container_status_v2(&containerPath).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Verify item hash using V2 implementation
#[tauri::command]
pub async fn container_verify_item_hash_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    itemAddr: u64,
) -> Result<bool, String> {
    debug!("container_verify_item_hash_v2: {} addr={}", containerPath, itemAddr);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::verify_item_hash_v2(&containerPath, itemAddr).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Verify all items in container (V2)
#[tauri::command]
pub async fn container_verify_all_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    hashType: String,
) -> Result<Vec<ad1::ItemVerifyResult>, String> {
    debug!("container_verify_all_v2: {} type={}", containerPath, hashType);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            let hash_type = match hashType.to_lowercase().as_str() {
                "md5" => ad1::HashType::Md5,
                "sha1" => ad1::HashType::Sha1,
                _ => return Err(format!("Invalid hash type: {}", hashType)),
            };
            ad1::verify_all_items(&containerPath, hash_type).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get container info (V2)
#[tauri::command]
pub async fn container_get_info_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    includeTree: bool,
) -> Result<ad1::Ad1InfoV2, String> {
    debug!("container_get_info_v2: {} tree={}", containerPath, includeTree);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::get_container_info_v2(&containerPath, includeTree).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Extract all files from container (V2)
#[tauri::command]
pub async fn container_extract_all_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    outputDir: String,
    #[allow(non_snake_case)]
    applyMetadata: bool,
    #[allow(non_snake_case)]
    verifyHashes: bool,
) -> Result<ad1::ExtractionResult, String> {
    debug!("container_extract_all_v2: {} -> {}", containerPath, outputDir);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            let options = ad1::ExtractOptions {
                output_dir: std::path::PathBuf::from(outputDir),
                apply_metadata: applyMetadata,
                verify_hashes: verifyHashes,
                overwrite: false,
                progress_callback: None,
            };
            ad1::extract_all_v2(&containerPath, options).map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Extract single item by address (V2)
#[tauri::command]
pub async fn container_extract_item_v2(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    itemAddr: u64,
    #[allow(non_snake_case)]
    outputPath: String,
) -> Result<(), String> {
    debug!("container_extract_item_v2: {} addr={} -> {}", containerPath, itemAddr, outputPath);
    tauri::async_runtime::spawn_blocking(move || {
        if ad1::is_ad1(&containerPath).unwrap_or(false) {
            ad1::extract_item_by_addr_v2(
                &containerPath,
                itemAddr,
                std::path::Path::new(&outputPath),
            )
            .map_err(|e| e.to_string())
        } else {
            Err(format!("Container type not supported: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// =============================================================================
// Verification Commands
// =============================================================================

#[tauri::command]
pub fn logical_verify(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
) -> Result<Vec<containers::VerifyEntry>, String> {
    containers::verify(&inputPath, &algorithm)
}

/// Hash all AD1 segment files to produce a single hash of the container image.
/// This is different from logical_verify which verifies internal file hashes.
#[tauri::command]
pub async fn ad1_hash_segments(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use tauri::Emitter;
    use super::VerifyProgress;
    
    let path_for_closure = inputPath.clone();
    // Run on blocking thread pool to prevent UI freeze
    tauri::async_runtime::spawn_blocking(move || {
        ad1::hash_segments_with_progress(&inputPath, &algorithm, |current, total| {
            let percent = if total > 0 { (current as f64 / total as f64) * 100.0 } else { 0.0 };
            let _ = app.emit("verify-progress", VerifyProgress {
                path: path_for_closure.clone(),
                current: current as usize,
                total: total as usize,
                percent,
            });
        }).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// =============================================================================
// Extraction Commands
// =============================================================================

#[tauri::command]
pub fn logical_extract(
    #[allow(non_snake_case)]
    inputPath: String,
    #[allow(non_snake_case)]
    outputDir: String,
) -> Result<(), String> {
    containers::extract(&inputPath, &outputDir)
}
