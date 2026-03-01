// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive metadata and tree listing operations.
//!
//! Provides quick metadata extraction and full directory tree listing for
//! ZIP, 7z, TAR, RAR, and DMG archives. Optimized for performance with
//! safety limits for large containers.

use tracing::debug;

use crate::archive;
use crate::common::filesystem::FilesystemDriver;

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
    #[allow(non_snake_case)] containerPath: String,
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
                // Check for encryption before attempting to open
                let is_encrypted = crate::common::filesystem::DmgDriver::is_encrypted(&containerPath);

                if is_encrypted {
                    Ok(ArchiveQuickMetadata {
                        entry_count: None,
                        archive_size,
                        format: "dmg".to_string(),
                        encrypted: true,
                        error: Some("DMG is encrypted (AES-256)".to_string()),
                    })
                } else {
                    match crate::common::filesystem::DmgDriver::open(&containerPath) {
                        Ok(dmg) => {
                            let partition_count = dmg.partition_count() as u32;
                            Ok(ArchiveQuickMetadata {
                                entry_count: Some(partition_count),
                                archive_size,
                                format: "dmg".to_string(),
                                encrypted: false,
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
            }
            _ => Ok(ArchiveQuickMetadata {
                entry_count: None,
                archive_size,
                format: "unknown".to_string(),
                encrypted: false,
                error: Some(format!(
                    "Unknown archive format: .{}. Supported archive formats: ZIP, 7z, TAR, GZ, BZ2, XZ, RAR, DMG, ISO. \
                     The file may be corrupted, encrypted, or use an unsupported archive format.",
                    extension
                )),
            }),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get the file tree for an archive container (ZIP, 7z, etc.)
///
/// Supports ZIP, 7z, TAR (and compressed variants), RAR, and DMG archives.
/// For DMG files, recursively reads entire HFS+ directory tree with safety limits.
#[tauri::command]
pub async fn archive_get_tree(
    #[allow(non_snake_case)] containerPath: String,
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
                let dmg_start = std::time::Instant::now();

                match crate::common::filesystem::DmgDriver::open(&containerPath) {
                    Ok(dmg) => {
                        debug!("archive_get_tree: DMG opened in {:.2}s", dmg_start.elapsed().as_secs_f32());

                        // Find the HFS+ partition
                        if let Some(hfs_idx) = dmg.find_hfs_partition() {
                            debug!("archive_get_tree: found HFS+ partition at index {}", hfs_idx);

                            let partition_start = std::time::Instant::now();
                            // Get partition as block device
                            match dmg.partition_device(hfs_idx) {
                                Ok(device) => {
                                    debug!("archive_get_tree: partition decompressed in {:.2}s", partition_start.elapsed().as_secs_f32());
                                    let size = device.size();

                                    let mount_start = std::time::Instant::now();
                                    // Mount HFS+ filesystem
                                    match crate::common::filesystem::HfsPlusDriver::new(device, 0, size) {
                                        Ok(hfs) => {
                                            debug!("archive_get_tree: HFS+ mounted in {:.2}s", mount_start.elapsed().as_secs_f32());
                                            debug!("archive_get_tree: starting recursive DMG directory scan");
                                            let start_time = std::time::Instant::now();

                                            // Recursively read entire directory tree with safety limits
                                            let mut all_entries = Vec::new();
                                            let mut dirs_to_process = vec![("/".to_string(), 0u32)]; // (path, depth)
                                            let max_depth = 50; // Prevent infinite recursion
                                            let max_entries = 100_000; // Prevent memory issues
                                            let mut dirs_processed = 0;

                                            while let Some((current_dir, depth)) = dirs_to_process.pop() {
                                                // Safety check: depth limit
                                                if depth > max_depth {
                                                    debug!("archive_get_tree: skipping {} - max depth {} exceeded", current_dir, max_depth);
                                                    continue;
                                                }

                                                // Safety check: entry count limit
                                                if all_entries.len() >= max_entries {
                                                    debug!("archive_get_tree: stopped at {} entries - max limit reached", max_entries);
                                                    break;
                                                }

                                                dirs_processed += 1;
                                                if dirs_processed % 100 == 0 {
                                                    debug!("archive_get_tree: processed {} directories, {} entries so far ({:.1}s)",
                                                           dirs_processed, all_entries.len(), start_time.elapsed().as_secs_f32());
                                                }

                                                match hfs.readdir(&current_dir) {
                                                    Ok(entries) => {
                                                        for entry in entries {
                                                            // Build full path
                                                            let full_path = if current_dir == "/" {
                                                                entry.name.clone()
                                                            } else {
                                                                format!("{}/{}", current_dir.trim_end_matches('/'), entry.name)
                                                            };

                                                            // Get file size for files (only if not too many entries yet)
                                                            let entry_size = if !entry.is_directory && all_entries.len() < max_entries {
                                                                let attr_path = if current_dir == "/" {
                                                                    format!("/{}", entry.name)
                                                                } else {
                                                                    format!("{}/{}", current_dir.trim_end_matches('/'), entry.name)
                                                                };
                                                                hfs.getattr(&attr_path)
                                                                    .map(|a| a.size)
                                                                    .unwrap_or(0)
                                                            } else {
                                                                0
                                                            };

                                                            all_entries.push(ArchiveTreeEntry {
                                                                path: full_path.clone(),
                                                                name: entry.name.clone(),
                                                                is_dir: entry.is_directory,
                                                                size: entry_size,
                                                                compressed_size: 0,
                                                                crc32: 0,
                                                                modified: String::new(),
                                                            });

                                                            // Add subdirectories to queue with incremented depth
                                                            if entry.is_directory {
                                                                let subdir_path = if current_dir == "/" {
                                                                    format!("/{}", entry.name)
                                                                } else {
                                                                    format!("{}/{}", current_dir.trim_end_matches('/'), entry.name)
                                                                };
                                                                dirs_to_process.push((subdir_path, depth + 1));
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        debug!("archive_get_tree: failed to read directory {}: {}", current_dir, e);
                                                        // Continue processing other directories
                                                    }
                                                }
                                            }

                                            let elapsed = start_time.elapsed();
                                            debug!("archive_get_tree: DMG scan complete - {} entries, {} directories, {:.2}s",
                                                   all_entries.len(), dirs_processed, elapsed.as_secs_f32());
                                            Ok(all_entries)
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
                            path: format!("(Unsupported archive format: .{})", extension),
                            name: format!(
                                "(Unable to read archive contents — .{} is not a recognized format. \
                                 Supported: ZIP, 7z, TAR, GZ, BZ2, XZ, RAR, DMG, ISO)",
                                extension
                            ),
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
