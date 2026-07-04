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

const ARCHIVE_TREE_MAX_ENTRIES: usize = 100_000;
const ARCHIVE_TREE_FIELD_MAX_CHARS: usize = 4096;
const ARCHIVE_TREE_ERROR_MAX_CHARS: usize = 8192;
const ARCHIVE_TREE_TRUNCATED_SUFFIX: &str = "... [truncated]";

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
                        error: Some(truncate_archive_tree_text(
                            &e.to_string(),
                            ARCHIVE_TREE_ERROR_MAX_CHARS,
                        )),
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
                        error: Some(truncate_archive_tree_text(
                            &e.to_string(),
                            ARCHIVE_TREE_ERROR_MAX_CHARS,
                        )),
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
                            error: Some(truncate_archive_tree_text(
                                &e.to_string(),
                                ARCHIVE_TREE_ERROR_MAX_CHARS,
                            )),
                        }),
                    }
                }
            }
            "iso" => Ok(ArchiveQuickMetadata {
                entry_count: None,
                archive_size,
                format: "iso".to_string(),
                encrypted: false,
                error: None,
            }),
            _ => Ok(ArchiveQuickMetadata {
                entry_count: None,
                archive_size,
                format: "unknown".to_string(),
                encrypted: false,
                error: Some(truncate_archive_tree_text(
                    &format!(
                        "Unknown archive format: .{}. Supported archive formats: ZIP, 7z, TAR, GZ, BZ2, XZ, RAR, DMG, ISO. \
                     The file may be corrupted, encrypted, or use an unsupported archive format.",
                        extension
                    ),
                    ARCHIVE_TREE_ERROR_MAX_CHARS,
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
            bounded_archive_tree_entries(entries.into_iter().map(|e| {
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
            }))
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
                    return Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
                        path: format!("(Segmented 7z archive: {})", e),
                        name: "(Listing failed)".to_string(),
                        is_dir: false,
                        size: 0,
                        compressed_size: 0,
                        crc32: 0,
                        modified: String::new(),
                    }]));
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

                        Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
                            path: format!("(7z archive v{}: {})", version_str, status),
                            name: format!("({})", status),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }]))
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
                        Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
                            path: format!("(Compressed {} file)", extension.to_uppercase()),
                            name: "(Single compressed file - not a TAR archive)".to_string(),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }]))
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

                        Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
                            path: format!("(RAR archive: {})", message),
                            name: format!("({})", message),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }]))
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
                                            let max_entries = ARCHIVE_TREE_MAX_ENTRIES; // Prevent memory issues
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
                                            Ok(bounded_archive_tree_entries(all_entries))
                                        }
                                        Err(e) => {
                                            debug!("archive_get_tree: failed to mount HFS+: {}", e);
                                            Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
                                                path: format!("(DMG: HFS+ mount failed - {})", e),
                                                name: "(HFS+ mount failed)".to_string(),
                                                is_dir: false,
                                                size: 0,
                                                compressed_size: 0,
                                                crc32: 0,
                                                modified: String::new(),
                                            }]))
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("archive_get_tree: failed to get partition device: {}", e);
                                    Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
                                        path: format!("(DMG: partition read failed - {})", e),
                                        name: "(Partition read failed)".to_string(),
                                        is_dir: false,
                                        size: 0,
                                        compressed_size: 0,
                                        crc32: 0,
                                        modified: String::new(),
                                    }]))
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
                                Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
                                    path: "(DMG: no partitions found)".to_string(),
                                    name: "(No partitions)".to_string(),
                                    is_dir: false,
                                    size: 0,
                                    compressed_size: 0,
                                    crc32: 0,
                                    modified: String::new(),
                                }]))
                            } else {
                                Ok(bounded_archive_tree_entries(partition_entries))
                            }
                        }
                    }
                    Err(e) => {
                        debug!("archive_get_tree: failed to open DMG: {}", e);
                        Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
                            path: format!("(DMG: failed to open - {})", e),
                            name: "(DMG open failed)".to_string(),
                            is_dir: false,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            modified: String::new(),
                        }]))
                    }
                }
            }
            "iso" => {
                debug!("archive_get_tree: handling ISO format");
                let entries = archive::libarchive_list_all(&containerPath)
                    .map_err(|e| e.to_string())?;
                Ok(bounded_archive_tree_entries(entries.into_iter().map(|entry| ArchiveTreeEntry {
                        path: entry.path,
                        name: entry.name,
                        is_dir: entry.is_dir,
                        size: entry.size,
                        compressed_size: 0,
                        crc32: 0,
                        modified: entry
                            .mtime
                            .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_default(),
                    })))
            }
            _ => {
                // Try ZIP format as fallback (some archives use non-standard extensions)
                debug!("archive_get_tree: trying ZIP fallback for unknown extension");
                match archive::list_zip_entries(&containerPath) {
                    Ok(entries) => Ok(convert_entries(entries)),
                    Err(_) => {
                        // Return empty with helpful message
                        Ok(bounded_archive_tree_entries(vec![ArchiveTreeEntry {
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
                        }]))
                    }
                }
            }
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

fn bounded_archive_tree_entries(
    entries: impl IntoIterator<Item = ArchiveTreeEntry>,
) -> Vec<ArchiveTreeEntry> {
    entries
        .into_iter()
        .take(ARCHIVE_TREE_MAX_ENTRIES)
        .map(bounded_archive_tree_entry)
        .collect()
}

fn bounded_archive_tree_entry(mut entry: ArchiveTreeEntry) -> ArchiveTreeEntry {
    entry.path = truncate_archive_tree_text(&entry.path, ARCHIVE_TREE_FIELD_MAX_CHARS);
    entry.name = truncate_archive_tree_text(&entry.name, ARCHIVE_TREE_FIELD_MAX_CHARS);
    entry.modified = truncate_archive_tree_text(&entry.modified, ARCHIVE_TREE_FIELD_MAX_CHARS);
    entry
}

fn truncate_archive_tree_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let keep_chars = max_chars.saturating_sub(ARCHIVE_TREE_TRUNCATED_SUFFIX.chars().count());
    let mut truncated: String = value.chars().take(keep_chars).collect();
    truncated.push_str(ARCHIVE_TREE_TRUNCATED_SUFFIX);
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entry(index: usize) -> ArchiveTreeEntry {
        ArchiveTreeEntry {
            path: format!("folder/file-{index}.txt"),
            name: format!("file-{index}.txt"),
            is_dir: false,
            size: index as u64,
            compressed_size: 0,
            crc32: 0,
            modified: "2026-02-16 10:00:00".to_string(),
        }
    }

    #[test]
    fn bounded_archive_tree_entries_caps_result_count() {
        let entries = (0..ARCHIVE_TREE_MAX_ENTRIES + 1).map(test_entry);
        let bounded = bounded_archive_tree_entries(entries);

        assert_eq!(bounded.len(), ARCHIVE_TREE_MAX_ENTRIES);
        assert_eq!(bounded.last().unwrap().path, "folder/file-99999.txt");
    }

    #[test]
    fn bounded_archive_tree_entry_caps_long_strings() {
        let long = "x".repeat(ARCHIVE_TREE_FIELD_MAX_CHARS + 8);
        let bounded = bounded_archive_tree_entry(ArchiveTreeEntry {
            path: long.clone(),
            name: long.clone(),
            is_dir: false,
            size: 1,
            compressed_size: 0,
            crc32: 0,
            modified: long,
        });

        assert_eq!(bounded.path.chars().count(), ARCHIVE_TREE_FIELD_MAX_CHARS);
        assert_eq!(bounded.name.chars().count(), ARCHIVE_TREE_FIELD_MAX_CHARS);
        assert_eq!(
            bounded.modified.chars().count(),
            ARCHIVE_TREE_FIELD_MAX_CHARS
        );
        assert!(bounded.path.ends_with(ARCHIVE_TREE_TRUNCATED_SUFFIX));
    }

    #[test]
    fn truncate_archive_tree_text_preserves_short_text() {
        assert_eq!(truncate_archive_tree_text("short", 16), "short");
    }
}
