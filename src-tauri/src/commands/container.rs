// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 container operations including tree navigation, data reading, and verification.

use tracing::debug;

use crate::ad1;
use crate::containers;
use crate::ewf;
use crate::raw;

// =============================================================================
// V1 Container Commands
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
#[tauri::command]
pub fn logical_info_fast(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<containers::ContainerInfo, String> {
    containers::info_fast(&inputPath)
}

/// Get only stored hashes - minimal parsing, fastest option
#[tauri::command]
pub fn get_stored_hashes_only(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<Vec<containers::StoredHash>, String> {
    containers::get_stored_hashes_only(&inputPath)
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
        } else if ewf::is_l01_file(&containerPath).unwrap_or(false) {
            // L01 logical evidence - read chunk using ltree offsets
            let tree = ewf::parse_l01_file_tree(&containerPath)
                .map_err(|e| format!("Failed to parse L01 file tree: {}", e))?;
            let entry = tree.entry_at_path(&entryPath)
                .ok_or_else(|| format!("Entry not found in L01: {}", entryPath))?;
            let mut handle = ewf::EwfHandle::open(&containerPath)
                .map_err(|e| format!("Failed to open L01 handle: {}", e))?;
            let read_offset = entry.data_offset + offset;
            let max_size = if entry.size > offset { (entry.size - offset) as usize } else { 0 };
            let actual_size = std::cmp::min(size, max_size);
            if actual_size == 0 {
                return Ok(Vec::new());
            }
            handle.read_at(read_offset, actual_size)
                .map_err(|e| format!("Failed to read L01 chunk: {}", e))
        } else if ewf::is_ewf(&containerPath).unwrap_or(false) {
            // Fallback: VFS entry reached container_read_entry_chunk without isVfsEntry flag
            use crate::common::vfs::VirtualFileSystem;
            let vfs = ewf::vfs::EwfVfs::open(&containerPath)
                .map_err(|e| format!("Failed to open E01: {:?}", e))?;
            vfs.read(&entryPath, offset, size)
                .map_err(|e| format!("Failed to read VFS file: {:?}", e))
        } else if raw::is_raw(&containerPath).unwrap_or(false) {
            // Fallback: VFS entry reached container_read_entry_chunk without isVfsEntry flag
            use crate::common::vfs::VirtualFileSystem;
            let vfs = raw::vfs::RawVfs::open_filesystem(&containerPath)
                .or_else(|_| raw::vfs::RawVfs::open(&containerPath))
                .map_err(|e| format!("Failed to open raw: {:?}", e))?;
            vfs.read(&entryPath, offset, size)
                .map_err(|e| format!("Failed to read raw file: {:?}", e))
        } else {
            Err(format!("Unsupported container type for: {}", containerPath))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Extract an entry from a container to a temp file for preview
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
        // Use explicit flags first, then auto-detect by file type as fallback.
        // This is defensive: VFS entries should have isVfsEntry=true, but if the
        // flag is missing/false, we still try to read based on the container format.
        let is_ewf = ewf::is_ewf(&containerPath).unwrap_or(false);
        let is_l01 = ewf::is_l01_file(&containerPath).unwrap_or(false);
        let is_raw = raw::is_raw(&containerPath).unwrap_or(false);
        let is_ad1 = ad1::is_ad1(&containerPath).unwrap_or(false);
        
        let data = if isArchiveEntry {
            // Archive entry - use libarchive unified backend
            use crate::archive;
            
            // Check for nested archive entries: entryPath contains "::" separator
            // e.g., "/Partition2_NTFS/path/to/ARCHIVE.ZIP::inner/file.mdb"
            // This means: extract ARCHIVE.ZIP from the parent container first,
            // then read inner/file.mdb from the extracted archive.
            if let Some(sep_pos) = entryPath.find("::") {
                let nested_archive_path = &entryPath[..sep_pos];
                let inner_entry_path = &entryPath[sep_pos + 2..];
                
                debug!("Nested archive extraction: archive='{}', inner='{}'", 
                       nested_archive_path, inner_entry_path);
                
                // Step 1: Extract the archive from the parent container to temp
                let temp_archive_path = crate::commands::archive::nested::get_or_create_nested_temp(
                    &containerPath, nested_archive_path
                )?;
                
                // Step 2: Read the inner entry from the extracted archive
                archive::libarchive_read_file(&temp_archive_path, inner_entry_path)
                    .map_err(|e| format!("Failed to read nested archive entry '{}': {}", inner_entry_path, e))?
            } else if entryPath.starts_with("(Compressed") {
                // Single-file compressed format (BZ2, GZ, XZ, etc.) with synthetic entry name
                // The tree shows "(Compressed BZ2 file)" but the actual entry has a different name.
                // Use libarchive to read the first (only) entry directly.
                let entries = archive::libarchive_list_all(&containerPath)
                    .map_err(|e| format!("Failed to list compressed file entries: {}", e))?;
                
                if let Some(first_entry) = entries.first() {
                    archive::libarchive_read_file(&containerPath, &first_entry.path)
                        .map_err(|e| format!("Failed to decompress file: {}", e))?
                } else {
                    return Err("Compressed file contains no entries".to_string());
                }
            } else {
                // Regular archive entry (containerPath IS the archive)
                archive::libarchive_read_file(&containerPath, &entryPath)
                    .map_err(|e| format!("Failed to read archive entry: {}", e))?
            }
        } else if is_l01 {
            // L01 logical evidence - read file data using ltree offsets
            let tree = ewf::parse_l01_file_tree(&containerPath)
                .map_err(|e| format!("Failed to parse L01 file tree: {}", e))?;
            
            let entry = tree.entry_at_path(&entryPath)
                .ok_or_else(|| format!("Entry not found in L01: {}", entryPath))?;
            
            if entry.is_directory {
                return Err("Cannot extract directory entries".to_string());
            }
            
            // Read the file data from the EWF data stream at the entry's offset
            let mut handle = ewf::EwfHandle::open(&containerPath)
                .map_err(|e| format!("Failed to open L01 handle: {}", e))?;
            
            let read_size = if entry.size > 0 { entry.size as usize } else { entry.data_size as usize };
            handle.read_at(entry.data_offset, read_size)
                .map_err(|e| format!("Failed to read L01 file data: {}", e))?
        } else if isVfsEntry || is_ewf || is_raw {
            // VFS entry (E01/Raw) - use VFS read
            use crate::common::vfs::VirtualFileSystem;
            
            if is_ewf {
                let vfs = ewf::vfs::EwfVfs::open(&containerPath)
                    .map_err(|e| format!("Failed to open E01: {:?}", e))?;
                // If entrySize is 0, query the actual file size from the VFS.
                // The frontend may report size=0 when getattr() failed during directory listing.
                let read_size = if entrySize == 0 {
                    vfs.file_size(&entryPath)
                        .map(|s| s as usize)
                        .unwrap_or(0)
                } else {
                    entrySize as usize
                };
                vfs.read(&entryPath, 0, read_size)
                    .map_err(|e| format!("Failed to read VFS file: {:?}", e))?
            } else if is_raw {
                let vfs = raw::vfs::RawVfs::open_filesystem(&containerPath)
                    .or_else(|_| raw::vfs::RawVfs::open(&containerPath))
                    .map_err(|e| format!("Failed to open raw: {:?}", e))?;
                let read_size = if entrySize == 0 {
                    vfs.file_size(&entryPath)
                        .map(|s| s as usize)
                        .unwrap_or(0)
                } else {
                    entrySize as usize
                };
                vfs.read(&entryPath, 0, read_size)
                    .map_err(|e| format!("Failed to read raw file: {:?}", e))?
            } else {
                return Err(format!("Unsupported VFS container: {}", containerPath));
            }
        } else if is_ad1 {
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

// =============================================================================
// Helper Functions (testable without Tauri runtime)
// =============================================================================

/// Classify how a container entry should be extracted based on flags and file types.
/// Returns a classification string used for routing extraction logic.
#[cfg(test)]
pub(crate) fn classify_extraction_route(
    container_path: &str,
    entry_path: &str,
    is_vfs_entry: bool,
    is_archive_entry: bool,
) -> &'static str {
    let is_ewf = ewf::is_ewf(container_path).unwrap_or(false);
    let is_raw = raw::is_raw(container_path).unwrap_or(false);
    let is_ad1 = ad1::is_ad1(container_path).unwrap_or(false);

    if is_archive_entry {
        if entry_path.contains("::") {
            "archive-nested"
        } else if entry_path.starts_with("(Compressed") {
            "archive-compressed"
        } else {
            "archive-regular"
        }
    } else if is_vfs_entry || is_ewf || is_raw {
        if is_ewf {
            "vfs-ewf"
        } else {
            "vfs-raw"
        }
    } else if is_ad1 {
        "ad1"
    } else {
        "unsupported"
    }
}

/// Parse a nested archive entry path with `::` separator.
/// Returns `(archive_path, inner_entry_path)`.
#[cfg(test)]
pub(crate) fn parse_nested_archive_path(entry_path: &str) -> Option<(&str, &str)> {
    entry_path.find("::").map(|pos| {
        (&entry_path[..pos], &entry_path[pos + 2..])
    })
}

/// Check if an entry path represents a single-file compressed format.
#[cfg(test)]
pub(crate) fn is_compressed_synthetic_entry(entry_path: &str) -> bool {
    entry_path.starts_with("(Compressed")
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

// =============================================================================
// Verification Commands
// =============================================================================

/// Hash all AD1 segment files to produce a single hash of the container image.
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
                current,
                total,
                percent,
            });
        }).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse_nested_archive_path tests ====================

    #[test]
    fn test_parse_nested_with_separator() {
        let result = parse_nested_archive_path("/Partition2/path/to/ARCHIVE.ZIP::inner/file.mdb");
        assert!(result.is_some());
        let (archive, inner) = result.unwrap();
        assert_eq!(archive, "/Partition2/path/to/ARCHIVE.ZIP");
        assert_eq!(inner, "inner/file.mdb");
    }

    #[test]
    fn test_parse_nested_no_separator() {
        let result = parse_nested_archive_path("some/normal/path.txt");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_nested_separator_at_start() {
        let result = parse_nested_archive_path("::file.txt");
        assert!(result.is_some());
        let (archive, inner) = result.unwrap();
        assert_eq!(archive, "");
        assert_eq!(inner, "file.txt");
    }

    #[test]
    fn test_parse_nested_multiple_separators() {
        // Should split at the first `::`
        let result = parse_nested_archive_path("outer.zip::middle.zip::inner.txt");
        assert!(result.is_some());
        let (archive, inner) = result.unwrap();
        assert_eq!(archive, "outer.zip");
        assert_eq!(inner, "middle.zip::inner.txt");
    }

    // ==================== is_compressed_synthetic_entry tests ====================

    #[test]
    fn test_compressed_synthetic_bz2() {
        assert!(is_compressed_synthetic_entry("(Compressed BZ2 file)"));
    }

    #[test]
    fn test_compressed_synthetic_gz() {
        assert!(is_compressed_synthetic_entry("(Compressed GZ file)"));
    }

    #[test]
    fn test_compressed_synthetic_xz() {
        assert!(is_compressed_synthetic_entry("(Compressed XZ file)"));
    }

    #[test]
    fn test_not_compressed_regular_path() {
        assert!(!is_compressed_synthetic_entry("normal/path/file.txt"));
    }

    #[test]
    fn test_not_compressed_partial_match() {
        assert!(!is_compressed_synthetic_entry("file_(Compressed).dat"));
    }

    // ==================== classify_extraction_route tests ====================

    #[test]
    fn test_classify_archive_nested() {
        let result = classify_extraction_route(
            "/tmp/test.zip",
            "archive.zip::inner/file.txt",
            false,
            true,
        );
        assert_eq!(result, "archive-nested");
    }

    #[test]
    fn test_classify_archive_compressed() {
        let result = classify_extraction_route(
            "/tmp/test.bz2",
            "(Compressed BZ2 file)",
            false,
            true,
        );
        assert_eq!(result, "archive-compressed");
    }

    #[test]
    fn test_classify_archive_regular() {
        let result = classify_extraction_route(
            "/tmp/test.zip",
            "documents/report.pdf",
            false,
            true,
        );
        assert_eq!(result, "archive-regular");
    }

    #[test]
    fn test_classify_unsupported() {
        let result = classify_extraction_route(
            "/tmp/unknown_file.xyz",
            "some_entry",
            false,
            false,
        );
        assert_eq!(result, "unsupported");
    }

    #[test]
    fn test_archive_flag_takes_precedence() {
        // Even if entryPath has ::, if isArchiveEntry is false, it's not archive-nested
        let result = classify_extraction_route(
            "/tmp/unknown.xyz",
            "path::with::colons",
            false,
            false,
        );
        assert_eq!(result, "unsupported");
    }
}
