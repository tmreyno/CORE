// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive tree listing and extraction commands for ZIP, 7z, RAR, TAR, DMG formats.

use tracing::debug;
use tauri::{Window, Emitter};

use crate::archive;
use crate::common::filesystem::FilesystemDriver;  // For HFS+ readdir method

/// Archive entry for tree display (matches ArchiveEntry from extraction.rs)
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveTreeEntry {
    /// Path within the archive
    pub path: String,
    /// Filename only
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Uncompressed size
    pub size: u64,
    /// Compressed size
    pub compressed_size: u64,
    /// CRC32 checksum
    pub crc32: u32,
    /// Last modified timestamp
    pub modified: String,
}

/// Quick archive metadata (fast - only reads headers, not full listing)
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveQuickMetadata {
    /// Estimated entry count (from archive headers)
    pub entry_count: Option<u32>,
    /// Archive file size on disk
    pub archive_size: u64,
    /// Archive format detected
    pub format: String,
    /// Whether archive appears encrypted
    pub encrypted: bool,
    /// Error message if metadata parsing failed
    pub error: Option<String>,
}

/// Get quick metadata for an archive without listing all entries.
/// This is FAST (<100ms for any size) - only reads headers.
/// Use before archive_get_tree to show loading state with entry count.
#[tauri::command]
pub async fn archive_get_metadata(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<ArchiveQuickMetadata, String> {
    debug!("archive_get_metadata called: {}", containerPath);
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::Path::new(&containerPath);
        
        // Get file size
        let archive_size = path.metadata()
            .map(|m| m.len())
            .unwrap_or(0);
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Determine format
        let format = if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            "tar.gz".to_string()
        } else if filename.ends_with(".tar.bz2") || filename.ends_with(".tbz2") {
            "tar.bz2".to_string()
        } else if filename.ends_with(".tar.xz") || filename.ends_with(".txz") {
            "tar.xz".to_string()
        } else if filename.ends_with(".tar.zst") || filename.ends_with(".tzst") {
            "tar.zst".to_string()
        } else if filename.contains(".7z.") && extension.chars().all(|c| c.is_ascii_digit()) {
            "7z-split".to_string()
        } else {
            extension.clone()
        };
        
        match format.as_str() {
            "zip" => {
                match archive::zip::parse_metadata(&containerPath) {
                    Ok(meta) => Ok(ArchiveQuickMetadata {
                        entry_count: meta.entry_count,
                        archive_size,
                        format: "zip".to_string(),
                        encrypted: meta.aes_encrypted || meta.encrypted_headers,
                        error: None,
                    }),
                    Err(e) => Ok(ArchiveQuickMetadata {
                        entry_count: None,
                        archive_size,
                        format: "zip".to_string(),
                        encrypted: false,
                        error: Some(e.to_string()),
                    }),
                }
            }
            "7z" | "7z-split" => {
                match archive::sevenz::parse_metadata(&containerPath) {
                    Ok(meta) => Ok(ArchiveQuickMetadata {
                        entry_count: None, // 7z doesn't store count in headers
                        archive_size,
                        format: format.clone(),
                        encrypted: meta.encrypted,
                        error: None,
                    }),
                    Err(e) => Ok(ArchiveQuickMetadata {
                        entry_count: None,
                        archive_size,
                        format: format.clone(),
                        encrypted: false,
                        error: Some(e.to_string()),
                    }),
                }
            }
            "rar" => {
                // RAR metadata parsing is more complex, just return basic info
                Ok(ArchiveQuickMetadata {
                    entry_count: None, // RAR doesn't have quick header count
                    archive_size,
                    format: "rar".to_string(),
                    encrypted: false,
                    error: None,
                })
            }
            "tar" | "tar.gz" | "tar.bz2" | "tar.xz" | "tar.zst" | "gz" => {
                // TAR/compressed formats don't have quick entry count
                Ok(ArchiveQuickMetadata {
                    entry_count: None,
                    archive_size,
                    format: format.clone(),
                    encrypted: false,
                    error: None,
                })
            }
            "dmg" => {
                // DMG - Apple Disk Image
                match crate::common::filesystem::DmgDriver::open(&containerPath) {
                    Ok(dmg) => {
                        let partition_count = dmg.partition_count() as u32;
                        Ok(ArchiveQuickMetadata {
                            entry_count: Some(partition_count),
                            archive_size,
                            format: "dmg".to_string(),
                            encrypted: false, // TODO: detect encrypted DMGs
                            error: None,
                        })
                    }
                    Err(e) => Ok(ArchiveQuickMetadata {
                        entry_count: None,
                        archive_size,
                        format: "dmg".to_string(),
                        encrypted: false,
                        error: Some(e.to_string()),
                    }),
                }
            }
            _ => Ok(ArchiveQuickMetadata {
                entry_count: None,
                archive_size,
                format: "unknown".to_string(),
                encrypted: false,
                error: Some(format!("Unknown archive format: {}", extension)),
            }),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get the file tree for an archive container (ZIP, 7z, etc.)
/// 
/// Supports ZIP, 7z, TAR (and compressed variants), and RAR archives.
#[tauri::command]
pub async fn archive_get_tree(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<Vec<ArchiveTreeEntry>, String> {
    debug!("archive_get_tree called: {}", containerPath);
    tauri::async_runtime::spawn_blocking(move || {
        let path_str = containerPath.clone();
        let path = std::path::Path::new(&containerPath);
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        
        // Check for compound extensions (.tar.gz, .tar.bz2, etc.)
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        debug!("archive_get_tree: extension={}, filename={}", extension, filename);
        
        // Helper to convert ArchiveEntry to ArchiveTreeEntry
        let convert_entries = |entries: Vec<archive::ArchiveEntry>| -> Vec<ArchiveTreeEntry> {
            entries.into_iter().map(|e| {
                let name = std::path::Path::new(&e.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| e.path.clone());
                ArchiveTreeEntry {
                    path: e.path,
                    name,
                    is_dir: e.is_directory,
                    size: e.size,
                    compressed_size: e.compressed_size,
                    crc32: e.crc32,
                    modified: e.last_modified,
                }
            }).collect()
        };
        
        // Detect archive format and handle accordingly
        // Check compound extensions first
        if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") ||
           filename.ends_with(".tar.bz2") || filename.ends_with(".tbz2") ||
           filename.ends_with(".tar.xz") || filename.ends_with(".txz") ||
           filename.ends_with(".tar.zst") || filename.ends_with(".tzst") {
            // Compressed TAR archives
            debug!("archive_get_tree: handling compressed TAR format");
            let entries = archive::tar::list_entries(&path_str)
                .map_err(|e| e.to_string())?;
            return Ok(convert_entries(entries));
        }
        
        // Check for segmented 7z archives (.7z.001, .7z.002, etc.)
        if filename.contains(".7z.") && extension.chars().all(|c| c.is_ascii_digit()) {
            // This is a split 7z archive part
            debug!("archive_get_tree: handling segmented 7z format (.{})", extension);
            match archive::sevenz::list_entries(&containerPath) {
                Ok(entries) => {
                    debug!("archive_get_tree: got {} segmented 7z entries", entries.len());
                    return Ok(convert_entries(entries));
                }
                Err(e) => {
                    debug!("archive_get_tree: segmented 7z listing failed: {}", e);
                    return Ok(vec![ArchiveTreeEntry {
                        path: format!("(Segmented 7z archive: {})", e),
                        name: "(Listing failed)".to_string(),
                        is_dir: false,
                        size: 0,
                        compressed_size: 0,
                        crc32: 0,
                        modified: String::new(),
                    }]);
                }
            }
        }
        
        match extension.as_str() {
            "zip" => {
                // ZIP archives - fully supported
                debug!("archive_get_tree: handling ZIP format");
                let entries = archive::list_zip_entries(&containerPath)
                    .map_err(|e| e.to_string())?;
                debug!("archive_get_tree: got {} entries", entries.len());
                Ok(convert_entries(entries))
            }
            "7z" => {
                // 7-Zip archives - full listing with sevenz-rust
                debug!("archive_get_tree: handling 7z format");
                match archive::sevenz::list_entries(&containerPath) {
                    Ok(entries) => {
                        debug!("archive_get_tree: got {} 7z entries", entries.len());
                        Ok(convert_entries(entries))
                    }
                    Err(e) => {
                        // Fallback to metadata display if listing fails (e.g., encrypted)
                        debug!("archive_get_tree: 7z listing failed, using metadata: {}", e);
                        let meta = archive::sevenz::parse_metadata(&containerPath)
                            .map_err(|e| e.to_string())?;
                        
                        let version_str = meta.version.as_deref().unwrap_or("unknown");
                        let status = if meta.encrypted {
                            "encrypted headers - listing unavailable"
                        } else {
                            "listing failed"
                        };
                        
                        Ok(vec![ArchiveTreeEntry {
                            path: format!("(7z archive v{}: {})", version_str, status),
                            name: format!("({})", status),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }])
                    }
                }
            }
            "tar" => {
                // Uncompressed TAR archives
                debug!("archive_get_tree: handling TAR format");
                let entries = archive::tar::list_entries(&containerPath)
                    .map_err(|e| e.to_string())?;
                Ok(convert_entries(entries))
            }
            "gz" => {
                // Standalone gzip file (not .tar.gz)
                debug!("archive_get_tree: handling standalone gzip");
                if archive::tar::is_standalone_gzip(&containerPath) {
                    let entries = archive::tar::list_gzip_entry(&containerPath)
                        .map_err(|e| e.to_string())?;
                    Ok(convert_entries(entries))
                } else {
                    // Might be misnamed tar.gz
                    let entries = archive::tar::list_entries(&containerPath)
                        .map_err(|e| e.to_string())?;
                    Ok(convert_entries(entries))
                }
            }
            "bz2" | "xz" | "zst" | "lz4" => {
                // Compressed files - try as TAR first
                debug!("archive_get_tree: handling compressed format ({})", extension);
                match archive::tar::list_entries(&containerPath) {
                    Ok(entries) => Ok(convert_entries(entries)),
                    Err(_) => {
                        // Single compressed file
                        Ok(vec![ArchiveTreeEntry {
                            path: format!("(Compressed {} file)", extension.to_uppercase()),
                            name: "(Single compressed file - not a TAR archive)".to_string(),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }])
                    }
                }
            }
            "rar" | "r00" | "r01" => {
                // RAR archives
                debug!("archive_get_tree: handling RAR format");
                match archive::rar::list_entries(&containerPath) {
                    Ok(entries) => {
                        debug!("archive_get_tree: got {} RAR entries", entries.len());
                        Ok(convert_entries(entries))
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        let message = if error_str.contains("encrypted") || error_str.contains("password") {
                            "encrypted headers - listing unavailable"
                        } else if error_str.contains("library") || error_str.contains("unrar") {
                            "unrar library not installed"
                        } else {
                            "listing failed"
                        };
                        
                        Ok(vec![ArchiveTreeEntry {
                            path: format!("(RAR archive: {})", message),
                            name: format!("({})", message),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }])
                    }
                }
            }
            "dmg" => {
                // DMG (Apple Disk Image) - parse HFS+ filesystem inside
                debug!("archive_get_tree: handling DMG format");
                match crate::common::filesystem::DmgDriver::open(&containerPath) {
                    Ok(dmg) => {
                        // Find the HFS+ partition
                        if let Some(hfs_idx) = dmg.find_hfs_partition() {
                            debug!("archive_get_tree: found HFS+ partition at index {}", hfs_idx);
                            
                            // Get partition as block device
                            match dmg.partition_device(hfs_idx) {
                                Ok(device) => {
                                    let size = device.size();
                                    
                                    // Mount HFS+ filesystem
                                    match crate::common::filesystem::HfsPlusDriver::new(device, 0, size) {
                                        Ok(hfs) => {
                                            // Read root directory
                                            match hfs.readdir("/") {
                                                Ok(entries) => {
                                                    let tree_entries: Vec<ArchiveTreeEntry> = entries.iter().map(|e| {
                                                        // Get file size via getattr if needed
                                                        let entry_size = if e.is_directory {
                                                            0
                                                        } else {
                                                            hfs.getattr(&format!("/{}", e.name))
                                                                .map(|a| a.size)
                                                                .unwrap_or(0)
                                                        };
                                                        ArchiveTreeEntry {
                                                            path: e.name.clone(),
                                                            name: e.name.clone(),
                                                            is_dir: e.is_directory,
                                                            size: entry_size,
                                                            compressed_size: 0,
                                                            crc32: 0,
                                                            modified: String::new(),
                                                        }
                                                    }).collect();
                                                    debug!("archive_get_tree: got {} HFS+ entries from DMG", tree_entries.len());
                                                    Ok(tree_entries)
                                                }
                                                Err(e) => {
                                                    debug!("archive_get_tree: failed to read HFS+ root: {}", e);
                                                    Ok(vec![ArchiveTreeEntry {
                                                        path: format!("(DMG HFS+ filesystem: failed to read - {})", e),
                                                        name: "(Failed to read filesystem)".to_string(),
                                                        is_dir: false,
                                                        size: 0,
                                                        compressed_size: 0,
                                                        crc32: 0,
                                                        modified: String::new(),
                                                    }])
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            debug!("archive_get_tree: failed to mount HFS+: {}", e);
                                            Ok(vec![ArchiveTreeEntry {
                                                path: format!("(DMG: HFS+ mount failed - {})", e),
                                                name: "(HFS+ mount failed)".to_string(),
                                                is_dir: false,
                                                size: 0,
                                                compressed_size: 0,
                                                crc32: 0,
                                                modified: String::new(),
                                            }])
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("archive_get_tree: failed to get partition device: {}", e);
                                    Ok(vec![ArchiveTreeEntry {
                                        path: format!("(DMG: partition read failed - {})", e),
                                        name: "(Partition read failed)".to_string(),
                                        is_dir: false,
                                        size: 0,
                                        compressed_size: 0,
                                        crc32: 0,
                                        modified: String::new(),
                                    }])
                                }
                            }
                        } else {
                            // No HFS+ partition found, list partitions
                            let partition_entries: Vec<ArchiveTreeEntry> = dmg.partitions()
                                .iter()
                                .enumerate()
                                .map(|(i, name)| {
                                    let info = dmg.partition_info(i);
                                    ArchiveTreeEntry {
                                        path: format!("Partition_{}", i),
                                        name: name.clone(),
                                        is_dir: true,
                                        size: info.map(|i| i.size).unwrap_or(0),
                                        compressed_size: 0,
                                        crc32: 0,
                                        modified: String::new(),
                                    }
                                })
                                .collect();
                            
                            if partition_entries.is_empty() {
                                Ok(vec![ArchiveTreeEntry {
                                    path: "(DMG: no partitions found)".to_string(),
                                    name: "(No partitions)".to_string(),
                                    is_dir: false,
                                    size: 0,
                                    compressed_size: 0,
                                    crc32: 0,
                                    modified: String::new(),
                                }])
                            } else {
                                Ok(partition_entries)
                            }
                        }
                    }
                    Err(e) => {
                        debug!("archive_get_tree: failed to open DMG: {}", e);
                        Ok(vec![ArchiveTreeEntry {
                            path: format!("(DMG: failed to open - {})", e),
                            name: "(DMG open failed)".to_string(),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }])
                    }
                }
            }
            _ => {
                // Try ZIP format as fallback (some archives use non-standard extensions)
                debug!("archive_get_tree: trying ZIP fallback for unknown extension");
                match archive::list_zip_entries(&containerPath) {
                    Ok(entries) => Ok(convert_entries(entries)),
                    Err(_) => {
                        // Return empty with helpful message
                        Ok(vec![ArchiveTreeEntry {
                            path: format!("(Unknown archive format: .{})", extension),
                            name: "(Unable to read archive contents)".to_string(),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }])
                    }
                }
            }
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Extract a single entry from an archive to a temp file
/// 
/// Used for opening nested containers (containers inside archives)
/// Returns the path to the extracted temp file
#[tauri::command]
pub async fn archive_extract_entry(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    entryPath: String,
) -> Result<String, String> {
    debug!("archive_extract_entry: container={}, entry={}", containerPath, entryPath);
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_path = std::path::Path::new(&containerPath);
        let extension = container_path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        
        // Create temp directory for extracted file
        let temp_dir = std::env::temp_dir().join("core-ffx-nested");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        
        // Generate output filename from entry path (preserving extension)
        let entry_filename = std::path::Path::new(&entryPath)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("extracted");
        
        // Add unique prefix to avoid collisions
        let unique_name = format!("{}_{}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            entry_filename
        );
        
        let output_path = temp_dir.join(&unique_name);
        let output_str = output_path.to_string_lossy().to_string();
        
        match extension.as_str() {
            "zip" => {
                archive::extract_zip_entry(&containerPath, &entryPath, &output_str)
                    .map_err(|e| e.to_string())?;
                Ok(output_str)
            }
            "7z" => {
                // 7z requires external tool - for now return error
                Err("7z nested container extraction not yet supported. Please extract the archive first.".to_string())
            }
            "rar" | "r00" | "r01" => {
                // RAR requires external tool
                Err("RAR nested container extraction not yet supported. Please extract the archive first.".to_string())
            }
            _ => {
                // Try ZIP as fallback
                match archive::extract_zip_entry(&containerPath, &entryPath, &output_str) {
                    Ok(_) => Ok(output_str),
                    Err(e) => Err(format!("Extraction not supported for this archive type: {}", e))
                }
            }
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read a chunk of bytes from an entry within an archive
/// 
/// Used for HexViewer to show archive file contents without extracting the entire file.
/// For most archive formats, we need to decompress the entire entry first, then slice it.
/// The data is cached in memory to allow efficient scrolling through large files.
#[tauri::command]
pub async fn archive_read_entry_chunk(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    entryPath: String,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, String> {
    debug!("archive_read_entry_chunk: container={}, entry='{}', offset={}, size={}", 
           containerPath, entryPath, offset, size);
    
    tauri::async_runtime::spawn_blocking(move || {
        // Read the full entry content using libarchive
        // Note: Most archive formats require sequential decompression,
        // so we read the whole file and slice it. For very large files,
        // consider extracting to temp and memory-mapping.
        let data = archive::libarchive_read_file(&containerPath, &entryPath)
            .map_err(|e| {
                // Log available entries for debugging
                if let Ok(entries) = archive::libarchive_list_all(&containerPath) {
                    let paths: Vec<_> = entries.iter().take(10).map(|e| e.path.as_str()).collect();
                    debug!("archive_read_entry_chunk: Entry '{}' not found. First 10 entries in archive: {:?}", 
                           entryPath, paths);
                }
                format!("Failed to read archive entry '{}': {}", entryPath, e)
            })?;
        
        let total_size = data.len() as u64;
        debug!("archive_read_entry_chunk: Read {} bytes from entry", total_size);
        
        // Bounds checking
        if offset >= total_size {
            return Ok(Vec::new());
        }
        
        let start = offset as usize;
        let end = std::cmp::min(start + size as usize, data.len());
        
        Ok(data[start..end].to_vec())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read a chunk of bytes from an entry within a nested archive
/// 
/// Used for HexViewer to show nested archive file contents.
/// The entry path format is "nestedArchivePath::entryPath"
/// 
/// For example: "inner.zip::readme.txt" reads readme.txt from inner.zip
/// which is itself inside the parent container.
#[tauri::command]
pub async fn nested_archive_read_entry_chunk(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    nestedArchivePath: String,
    #[allow(non_snake_case)]
    entryPath: String,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, String> {
    debug!("nested_archive_read_entry_chunk: container={}, nestedArchive='{}', entry='{}', offset={}, size={}", 
           containerPath, nestedArchivePath, entryPath, offset, size);
    
    // Use the existing get_or_create_nested_temp helper (defined later in this file)
    // We need to run this in a blocking context since it's sync
    tauri::async_runtime::spawn_blocking(move || {
        // First, extract the nested archive to a temp file (or get from cache)
        let temp_path = get_or_create_nested_temp(&containerPath, &nestedArchivePath)?;
        
        debug!("nested_archive_read_entry_chunk: Using nested archive at {}", temp_path);
        
        // Now read from the extracted nested archive
        let data = archive::libarchive_read_file(&temp_path, &entryPath)
            .map_err(|e| format!("Failed to read nested archive entry '{}': {}", entryPath, e))?;
        
        let total_size = data.len() as u64;
        debug!("nested_archive_read_entry_chunk: Read {} bytes from nested entry", total_size);
        
        // Bounds checking
        if offset >= total_size {
            return Ok(Vec::new());
        }
        
        let start = offset as usize;
        let end = std::cmp::min(start + size as usize, data.len());
        
        Ok(data[start..end].to_vec())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// =============================================================================
// Nested Container Support - Inline Expansion
// =============================================================================

/// Nested container entry information
/// Unified type that works for any nested container type (archive, AD1, forensic image)
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NestedContainerEntry {
    /// Path within the nested container
    pub path: String,
    /// Filename only
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Uncompressed size
    pub size: u64,
    /// Hash/checksum if available (CRC32 for archives, MD5/SHA for AD1)
    pub hash: Option<String>,
    /// Last modified timestamp
    pub modified: Option<String>,
    /// Type of nested container this entry is from
    pub source_type: String,
    /// Whether this entry is itself a nested container
    pub is_nested_container: bool,
    /// Container type if this is a nested container
    pub nested_type: Option<String>,
}

/// Nested container info returned when first accessed
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NestedContainerInfo {
    /// Type of container (zip, ad1, e01, etc.)
    pub container_type: String,
    /// Total entry count
    pub entry_count: usize,
    /// Total size (uncompressed)
    pub total_size: u64,
    /// Whether container is encrypted
    pub encrypted: bool,
    /// Path where container was extracted (for forensic logging)
    pub temp_path: String,
    /// Original path within parent container
    pub original_path: String,
}

/// Cache for extracted nested containers (avoids re-extraction)
static NESTED_CONTAINER_CACHE: std::sync::LazyLock<parking_lot::Mutex<std::collections::HashMap<String, String>>> =
    std::sync::LazyLock::new(|| parking_lot::Mutex::new(std::collections::HashMap::new()));

/// Get or create the temp path for a nested container
fn get_or_create_nested_temp(parent_path: &str, nested_path: &str) -> Result<String, String> {
    let cache_key = format!("{}::{}", parent_path, nested_path);
    
    // Check cache first
    {
        let cache = NESTED_CONTAINER_CACHE.lock();
        if let Some(temp_path) = cache.get(&cache_key) {
            // Verify file still exists
            if std::path::Path::new(temp_path).exists() {
                return Ok(temp_path.clone());
            }
        }
    }
    
    // Extract to temp
    let temp_dir = std::env::temp_dir().join("core-ffx-nested");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    
    // Generate unique filename based on simple hash of path
    // Use FNV-1a hash for fast, deterministic hashing
    let hash = {
        let mut h: u64 = 0xcbf29ce484222325; // FNV offset basis
        for byte in cache_key.as_bytes() {
            h ^= *byte as u64;
            h = h.wrapping_mul(0x100000001b3); // FNV prime
        }
        format!("{:016x}", h)
    };
    let nested_filename = std::path::Path::new(nested_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("nested");
    let unique_name = format!("{}_{}", &hash[..8], nested_filename);
    let temp_path = temp_dir.join(&unique_name);
    let temp_str = temp_path.to_string_lossy().to_string();
    
    // Extract the nested container
    let parent_ext = std::path::Path::new(parent_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    
    match parent_ext.as_str() {
        "zip" => {
            archive::extract_zip_entry(parent_path, nested_path, &temp_str)
                .map_err(|e| e.to_string())?;
        }
        "7z" => {
            // Use libarchive for 7z
            let data = archive::libarchive_read_file(parent_path, nested_path)
                .map_err(|e| e.to_string())?;
            std::fs::write(&temp_path, data)
                .map_err(|e| format!("Failed to write extracted file: {}", e))?;
        }
        "rar" | "r00" | "r01" => {
            // Use libarchive for RAR
            let data = archive::libarchive_read_file(parent_path, nested_path)
                .map_err(|e| e.to_string())?;
            std::fs::write(&temp_path, data)
                .map_err(|e| format!("Failed to write extracted file: {}", e))?;
        }
        "tar" | "tgz" | "tar.gz" | "tar.bz2" | "tar.xz" => {
            // Use libarchive for tar variants
            let data = archive::libarchive_read_file(parent_path, nested_path)
                .map_err(|e| e.to_string())?;
            std::fs::write(&temp_path, data)
                .map_err(|e| format!("Failed to write extracted file: {}", e))?;
        }
        _ => {
            // Try ZIP as fallback, then libarchive
            match archive::extract_zip_entry(parent_path, nested_path, &temp_str) {
                Ok(_) => {}
                Err(_) => {
                    let data = archive::libarchive_read_file(parent_path, nested_path)
                        .map_err(|e| format!("Unsupported parent archive format: {}", e))?;
                    std::fs::write(&temp_path, data)
                        .map_err(|e| format!("Failed to write extracted file: {}", e))?;
                }
            }
        }
    }
    
    // Log forensic access
    tracing::info!(
        parent = %parent_path,
        nested = %nested_path,
        temp = %temp_str,
        "Extracted nested container for inline viewing"
    );
    
    // Cache the result
    {
        let mut cache = NESTED_CONTAINER_CACHE.lock();
        cache.insert(cache_key, temp_str.clone());
    }
    
    Ok(temp_str)
}

/// Detect container type from filename extension
fn detect_nested_container_type(filename: &str) -> Option<String> {
    let lower = filename.to_lowercase();
    
    // Forensic containers
    if lower.ends_with(".ad1") { return Some("ad1".to_string()); }
    if lower.ends_with(".e01") || lower.ends_with(".ex01") { return Some("e01".to_string()); }
    if lower.ends_with(".l01") || lower.ends_with(".lx01") { return Some("l01".to_string()); }
    
    // Archives
    if lower.ends_with(".zip") { return Some("zip".to_string()); }
    if lower.ends_with(".7z") { return Some("7z".to_string()); }
    if lower.ends_with(".rar") { return Some("rar".to_string()); }
    if lower.ends_with(".tar") || lower.ends_with(".tar.gz") || lower.ends_with(".tgz") 
       || lower.ends_with(".tar.bz2") || lower.ends_with(".tar.xz") { 
        return Some("tar".to_string()); 
    }
    
    // UFED
    if lower.ends_with(".ufd") || lower.ends_with(".ufdr") || lower.ends_with(".ufdx") {
        return Some("ufed".to_string());
    }
    
    // Disk images
    if lower.ends_with(".dmg") { return Some("dmg".to_string()); }
    if lower.ends_with(".iso") { return Some("iso".to_string()); }
    if lower.ends_with(".raw") || lower.ends_with(".dd") || lower.ends_with(".img") {
        return Some("raw".to_string());
    }
    
    None
}

/// Check if a filename is a recognized container type
fn is_container_filename(filename: &str) -> bool {
    detect_nested_container_type(filename).is_some()
}

/// Get the tree of entries for a nested container
/// 
/// This extracts the nested container from its parent (caching the extraction)
/// and then lists its entries. Used for inline expansion in the evidence tree.
/// 
/// # Arguments
/// * `parentContainerPath` - Path to the outer container (e.g., "/path/to/outer.zip")
/// * `nestedEntryPath` - Path within the outer container (e.g., "folder/inner.ad1")
/// 
/// # Returns
/// Vector of entries from the nested container, with nested container detection
#[tauri::command]
pub async fn nested_container_get_tree(
    #[allow(non_snake_case)]
    parentContainerPath: String,
    #[allow(non_snake_case)]
    nestedEntryPath: String,
) -> Result<Vec<NestedContainerEntry>, String> {
    debug!("nested_container_get_tree: parent={}, nested={}", parentContainerPath, nestedEntryPath);
    
    tauri::async_runtime::spawn_blocking(move || {
        // Extract nested container to temp (or get from cache)
        let temp_path = get_or_create_nested_temp(&parentContainerPath, &nestedEntryPath)?;
        
        // Detect the nested container type
        let nested_type = detect_nested_container_type(&nestedEntryPath)
            .unwrap_or_else(|| "unknown".to_string());
        
        // List entries based on container type
        let entries: Vec<NestedContainerEntry> = match nested_type.as_str() {
            "ad1" => {
                // Use AD1 module
                match crate::ad1::get_tree(&temp_path) {
                    Ok(tree) => tree.into_iter().map(|e| NestedContainerEntry {
                        path: e.path.clone(),
                        name: e.name.clone(),
                        is_dir: e.is_dir,
                        size: e.size,
                        // AD1 has md5_hash/sha1_hash, use md5 preferentially
                        hash: e.md5_hash.clone().or(e.sha1_hash.clone()),
                        modified: e.modified.clone(),
                        source_type: "ad1".to_string(),
                        is_nested_container: is_container_filename(&e.name),
                        nested_type: detect_nested_container_type(&e.name),
                    }).collect(),
                    Err(e) => return Err(format!("Failed to read AD1 container: {}", e)),
                }
            }
            "zip" => {
                match archive::list_zip_entries(&temp_path) {
                    Ok(entries) => entries.into_iter().map(|e| {
                        let name = std::path::Path::new(&e.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&e.path)
                            .to_string();
                        NestedContainerEntry {
                            path: e.path.clone(),
                            name: name.clone(),
                            is_dir: e.is_directory,
                            size: e.size,
                            hash: Some(format!("{:08X}", e.crc32)),
                            modified: Some(e.last_modified),
                            source_type: "zip".to_string(),
                            is_nested_container: is_container_filename(&name),
                            nested_type: detect_nested_container_type(&name),
                        }
                    }).collect(),
                    Err(e) => return Err(format!("Failed to read ZIP container: {}", e)),
                }
            }
            "7z" => {
                match archive::sevenz::list_entries(&temp_path) {
                    Ok(entries) => entries.into_iter().map(|e| {
                        let name = std::path::Path::new(&e.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&e.path)
                            .to_string();
                        NestedContainerEntry {
                            path: e.path.clone(),
                            name: name.clone(),
                            is_dir: e.is_directory,
                            size: e.size,
                            hash: Some(format!("{:08X}", e.crc32)),
                            modified: Some(e.last_modified),
                            source_type: "7z".to_string(),
                            is_nested_container: is_container_filename(&name),
                            nested_type: detect_nested_container_type(&name),
                        }
                    }).collect(),
                    Err(e) => return Err(format!("Failed to read 7z container: {}", e)),
                }
            }
            "rar" => {
                match archive::rar::list_entries(&temp_path) {
                    Ok(entries) => entries.into_iter().map(|e| {
                        let name = std::path::Path::new(&e.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&e.path)
                            .to_string();
                        NestedContainerEntry {
                            path: e.path.clone(),
                            name: name.clone(),
                            is_dir: e.is_directory,
                            size: e.size,
                            hash: Some(format!("{:08X}", e.crc32)),
                            modified: Some(e.last_modified),
                            source_type: "rar".to_string(),
                            is_nested_container: is_container_filename(&name),
                            nested_type: detect_nested_container_type(&name),
                        }
                    }).collect(),
                    Err(e) => return Err(format!("Failed to read RAR container: {}", e)),
                }
            }
            "tar" => {
                match archive::tar::list_entries(&temp_path) {
                    Ok(entries) => entries.into_iter().map(|e| {
                        let name = std::path::Path::new(&e.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&e.path)
                            .to_string();
                        NestedContainerEntry {
                            path: e.path.clone(),
                            name: name.clone(),
                            is_dir: e.is_directory,
                            size: e.size,
                            hash: None, // TAR doesn't have checksums
                            modified: Some(e.last_modified),
                            source_type: "tar".to_string(),
                            is_nested_container: is_container_filename(&name),
                            nested_type: detect_nested_container_type(&name),
                        }
                    }).collect(),
                    Err(e) => return Err(format!("Failed to read TAR container: {}", e)),
                }
            }
            "ufed" | "ufd" | "ufdr" | "ufdx" => {
                match crate::ufed::get_tree(&temp_path) {
                    Ok(entries) => entries.into_iter().map(|e| NestedContainerEntry {
                        path: e.path.clone(),
                        name: e.name.clone(),
                        is_dir: e.is_dir,
                        size: e.size,
                        hash: e.hash.clone(),
                        modified: e.modified.clone(),
                        source_type: "ufed".to_string(),
                        is_nested_container: is_container_filename(&e.name),
                        nested_type: detect_nested_container_type(&e.name),
                    }).collect(),
                    Err(e) => return Err(format!("Failed to read UFED container: {}", e)),
                }
            }
            _ => {
                // Try libarchive as universal fallback
                match archive::libarchive_list_all(&temp_path) {
                    Ok(entries) => entries.into_iter().map(|e| NestedContainerEntry {
                        path: e.path.clone(),
                        name: e.name.clone(),
                        is_dir: e.is_dir,
                        size: e.size,
                        hash: None,
                        modified: e.mtime.map(|t| {
                            chrono::DateTime::from_timestamp(t, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_default()
                        }),
                        source_type: nested_type.clone(),
                        is_nested_container: is_container_filename(&e.name),
                        nested_type: detect_nested_container_type(&e.name),
                    }).collect(),
                    Err(e) => return Err(format!("Unsupported nested container type '{}': {}", nested_type, e)),
                }
            }
        };
        
        debug!("nested_container_get_tree: found {} entries", entries.len());
        Ok(entries)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get info about a nested container without listing all entries
#[tauri::command]
pub async fn nested_container_get_info(
    #[allow(non_snake_case)]
    parentContainerPath: String,
    #[allow(non_snake_case)]
    nestedEntryPath: String,
) -> Result<NestedContainerInfo, String> {
    debug!("nested_container_get_info: parent={}, nested={}", parentContainerPath, nestedEntryPath);
    
    tauri::async_runtime::spawn_blocking(move || {
        // Extract nested container to temp (or get from cache)
        let temp_path = get_or_create_nested_temp(&parentContainerPath, &nestedEntryPath)?;
        
        // Detect the nested container type
        let container_type = detect_nested_container_type(&nestedEntryPath)
            .unwrap_or_else(|| "unknown".to_string());
        
        // Get quick info based on type
        let (entry_count, total_size, encrypted) = match container_type.as_str() {
            "zip" => {
                let meta = archive::extraction::get_zip_entry_count(&temp_path)
                    .map_err(|e| e.to_string())?;
                (meta, 0u64, false)
            }
            "7z" => {
                match archive::sevenz::list_entries(&temp_path) {
                    Ok(entries) => {
                        let total: u64 = entries.iter().map(|e| e.size).sum();
                        (entries.len(), total, false)
                    }
                    Err(_) => (0, 0, false)
                }
            }
            "ad1" => {
                // Use get_stats which gathers container statistics
                match crate::ad1::get_stats(&temp_path) {
                    Ok(stats) => (stats.total_items as usize, stats.total_size, false),
                    Err(_) => (0, 0, false)
                }
            }
            _ => (0, 0, false)
        };
        
        Ok(NestedContainerInfo {
            container_type,
            entry_count,
            total_size,
            encrypted,
            temp_path,
            original_path: nestedEntryPath,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Clear the nested container cache (called on app exit or manually)
#[tauri::command]
pub async fn nested_container_clear_cache() -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut cache = NESTED_CONTAINER_CACHE.lock();
        let count = cache.len();
        
        // Delete temp files
        for (_key, path) in cache.iter() {
            if let Err(e) = std::fs::remove_file(path) {
                tracing::warn!("Failed to remove temp file {}: {}", path, e);
            }
        }
        
        cache.clear();
        tracing::info!("Cleared {} nested container cache entries", count);
        Ok(count)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// =============================================================================
// NEW FEATURES: Archive Testing, Repair, Enhanced Errors, LZMA, Native Encryption
// =============================================================================

use seven_zip::SevenZip;
use seven_zip::advanced;
use seven_zip::EncryptionContext;

/// Test archive integrity without extracting
#[tauri::command]
pub async fn test_7z_archive(
    archive_path: String,
    password: Option<String>,
    window: Window,
) -> Result<bool, String> {
    tracing::info!("Testing archive integrity: {}", archive_path);
    
    let window_clone = window.clone();
    let archive_path_clone = archive_path.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new()
            .map_err(|e| format!("Failed to initialize 7z library: {}", e))?;
        
        // Emit starting status
        let _ = window_clone.emit("archive-test-progress", serde_json::json!({
            "archive_path": archive_path_clone,
            "status": "Testing archive integrity...",
            "percent": 0.0,
        }));
        
        // Test archive
        sz.test_archive(
            &archive_path_clone,
            password.as_deref(),
            None, // No progress callback for now
        ).map_err(|e| format!("Archive test failed: {}", e))?;
        
        // Emit completion
        let _ = window_clone.emit("archive-test-progress", serde_json::json!({
            "archive_path": archive_path_clone,
            "status": "Archive is valid",
            "percent": 100.0,
        }));
        
        tracing::info!("Archive test passed: {}", archive_path);
        Ok(true)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Repair corrupted archive
#[tauri::command]
pub async fn repair_7z_archive(
    corrupted_path: String,
    repaired_path: String,
    window: Window,
) -> Result<String, String> {
    tracing::info!("Repairing archive: {} -> {}", corrupted_path, repaired_path);
    
    let window_clone = window.clone();
    let repaired_clone = repaired_path.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new()
            .map_err(|e| format!("Failed to initialize 7z library: {}", e))?;
        
        // Emit start status
        let _ = window_clone.emit("archive-repair-progress", serde_json::json!({
            "percent": 0.0,
            "status": "Repairing archive...",
        }));
        
        // Repair archive
        sz.repair_archive(
            &corrupted_path,
            &repaired_clone,
            None, // No progress callback for now
        ).map_err(|e| format!("Archive repair failed: {}", e))?;
        
        // Emit completion
        let _ = window_clone.emit("archive-repair-progress", serde_json::json!({
            "percent": 100.0,
            "status": "Archive repaired successfully",
        }));
        
        tracing::info!("Archive repaired successfully: {}", repaired_clone);
        Ok(repaired_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get detailed information about the last archive error
#[tauri::command]
pub fn get_last_archive_error() -> Result<serde_json::Value, String> {
    advanced::DetailedError::get_last()
        .map(|err| serde_json::json!({
            "code": err.code,
            "message": err.message,
            "file_context": err.file_context,
            "position": err.position,
            "suggestion": err.suggestion,
        }))
        .map_err(|e| format!("Failed to get error details: {}", e))
}

/// Clear last error
#[tauri::command]
pub fn clear_last_archive_error() {
    advanced::DetailedError::clear();
}

// TODO: LZMA compress/decompress commands disabled until sevenzip-ffi library is updated with these functions
// The functions exist in the C library but Rust wrappers need to be added to the embedded sevenzip-ffi copy

/*
/// Compress a single file to .lzma format
#[tauri::command]
pub async fn compress_to_lzma(
    input_path: String,
    output_path: String,
    compression_level: u8,
    window: Window,
) -> Result<String, String> {
    tracing::info!("Compressing to LZMA: {} -> {}", input_path, output_path);
    
    let window_clone = window.clone();
    let output_clone = output_path.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let level = match compression_level {
            0 => CompressionLevel::Store,
            1 => CompressionLevel::Fastest,
            2..=3 => CompressionLevel::Fast,
            4..=6 => CompressionLevel::Normal,
            7..=8 => CompressionLevel::Maximum,
            9 => CompressionLevel::Ultra,
            _ => CompressionLevel::Normal,
        };
        
        advanced::compress_lzma(
            &input_path,
            &output_path,
            level,
            Some(Box::new(move |completed: u64, total: u64, _user_data: *mut std::ffi::c_void| {
                let percent = if total > 0 {
                    (completed as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                
                let _ = window_clone.emit("lzma-compress-progress", serde_json::json!({
                    "percent": percent,
                    "bytes_processed": completed,
                    "bytes_total": total,
                }));
            }))
        ).map_err(|e| format!("LZMA compression failed: {}", e))?;
        
        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Decompress a .lzma file
#[tauri::command]
pub async fn decompress_lzma(
    lzma_path: String,
    output_path: String,
    window: Window,
) -> Result<String, String> {
    tracing::info!("Decompressing LZMA: {} -> {}", lzma_path, output_path);
    
    let window_clone = window.clone();
    let output_clone = output_path.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        advanced::decompress_lzma(
            &lzma_path,
            &output_path,
            Some(Box::new(move |completed: u64, total: u64, _user_data: *mut std::ffi::c_void| {
                let percent = if total > 0 {
                    (completed as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                
                let _ = window_clone.emit("lzma-decompress-progress", serde_json::json!({
                    "percent": percent,
                }));
            }))
        ).map_err(|e| format!("LZMA decompression failed: {}", e))?;
        
        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
*/

/// Encrypt data using native Rust AES-256
#[tauri::command]
pub fn encrypt_data_native(
    data: Vec<u8>,
    password: String,
) -> Result<Vec<u8>, String> {
    let mut ctx = EncryptionContext::new(&password)
        .map_err(|e| format!("Failed to initialize encryption: {}", e))?;
    
    ctx.encrypt(&data)
        .map_err(|e| format!("Encryption failed: {}", e))
}

/// Decrypt data using native Rust AES-256
#[tauri::command]
pub fn decrypt_data_native(
    encrypted_data: Vec<u8>,
    password: String,
) -> Result<Vec<u8>, String> {
    let mut ctx = EncryptionContext::new(&password)
        .map_err(|e| format!("Failed to initialize decryption: {}", e))?;
    
    ctx.decrypt(&encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Extract split/multi-volume archive
#[tauri::command]
pub async fn extract_split_7z_archive(
    first_volume_path: String,
    output_dir: String,
    password: Option<String>,
    window: Window,
) -> Result<String, String> {
    tracing::info!("Extracting split archive: {} to {}", first_volume_path, output_dir);
    
    let window_clone = window.clone();
    let output_clone = output_dir.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        // Emit start status
        let _ = window_clone.emit("split-extract-progress", serde_json::json!({
            "status": "Extracting split archive...",
            "percent": 0.0,
        }));
        
        // Extract split archive (simple version without progress callback)
        advanced::extract_split_archive(
            &first_volume_path,
            &output_dir,
            password.as_deref(),
        ).map_err(|e| format!("Split archive extraction failed: {}", e))?;
        
        // Emit completion
        let _ = window_clone.emit("split-extract-progress", serde_json::json!({
            "status": "Split archive extracted successfully",
            "percent": 100.0,
        }));
        
        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Validate archive with detailed error reporting
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveValidationResult {
    pub is_valid: bool,
    pub error_message: Option<String>,
    pub file_context: Option<String>,
    pub suggestion: Option<String>,
}

#[tauri::command]
pub async fn validate_7z_archive(
    archive_path: String,
) -> Result<ArchiveValidationResult, String> {
    tracing::info!("Validating archive: {}", archive_path);
    
    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new()
            .map_err(|e| format!("Failed to initialize 7z: {}", e))?;
        
        match sz.validate_archive(&archive_path) {
            Ok(_) => Ok(ArchiveValidationResult {
                is_valid: true,
                error_message: None,
                file_context: None,
                suggestion: None,
            }),
            Err(e) => {
                // Try to get detailed error info
                let error_msg = e.to_string();
                Ok(ArchiveValidationResult {
                    is_valid: false,
                    error_message: Some(error_msg.clone()),
                    file_context: Some(archive_path.clone()),
                    suggestion: Some(match error_msg.as_str() {
                        msg if msg.contains("CRC") => "Archive may be corrupted. Try repair_7z_archive.".to_string(),
                        msg if msg.contains("password") => "Archive requires password or password is incorrect.".to_string(),
                        msg if msg.contains("header") => "Archive headers are damaged. Try repair_7z_archive.".to_string(),
                        _ => "Archive validation failed. Check file integrity.".to_string(),
                    }),
                })
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
