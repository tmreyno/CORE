// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! FFX - Forensic File Xplorer Library
//!
//! This library provides parsing, verification, and extraction capabilities
//! for forensic disk image and logical evidence container formats.
//!
//! ## Supported Formats
//!
//! | Format | Module    | Description                                    |
//! |--------|-----------|------------------------------------------------|
//! | AD1    | `ad1`     | AccessData Logical Image (FTK)                 |
//! | E01    | `ewf`     | Expert Witness Format / EnCase (physical)      |
//! | L01    | `ewf`     | EnCase Logical Evidence File                   |
//! | Ex01   | `ewf`     | Expert Witness Format v2 (physical)            |
//! | Lx01   | `ewf`     | EnCase Logical Evidence v2                     |
//! | RAW    | `raw`     | dd-style images (.dd, .raw, .img, .001)        |
//! | 7z     | `archive` | 7-Zip archives (metadata only)                 |
//! | ZIP    | `archive` | ZIP/ZIP64 archives (metadata only)             |
//! | RAR    | `archive` | RAR4/RAR5 archives (metadata only)             |
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  Tauri Commands (lib.rs)                                    │
//! │   - logical_info, logical_verify, logical_extract           │
//! │   - scan_directory, scan_directory_streaming                │
//! │   - e01_v3_*, raw_*, raw_verify_segments                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Container Abstraction (containers.rs)                      │
//! │   - ContainerInfo, ContainerKind                            │
//! │   - Auto-detection by signature and extension               │
//! │   - Unified info/verify/extract interface                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Format Parsers                                             │
//! │   ┌─────────┬─────────┬─────────┬─────────┬─────────┐      │
//! │   │  ad1.rs │  ewf.rs │  l01.rs │  raw.rs │archive.rs│     │
//! │   │  ufed.rs│         │         │         │         │      │
//! │   └─────────┴─────────┴─────────┴─────────┴─────────┘      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Common Utilities (common/)                                 │
//! │   - hash.rs:     Streaming hashers (SHA, BLAKE3, XXH3)     │
//! │   - binary.rs:   Little-endian binary reading              │
//! │   - segments.rs: Multi-segment file discovery              │
//! │   - io_pool.rs:  File handle pooling                       │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Tauri Integration
//!
//! All parsing operations run on blocking threads to prevent UI freezing.
//! Progress events are emitted via Tauri's event system:
//!
//! - `scan-file-found`: File discovered during directory scan
//! - `verify-progress`: Hash verification progress (current, total, percent)
//! - `segment-verify-progress`: Per-segment verification progress
//!
//! ## Performance Considerations
//!
//! - **Streaming I/O**: Large files use buffered/memory-mapped I/O
//! - **Parallel hashing**: BLAKE3 uses rayon for parallel processing
//! - **Pipelined I/O**: Separate reader/hasher threads for throughput
//! - **LRU caching**: AD1 decompressed blocks cached for random access
//!
//! ## Forensic Standards Compliance
//!
//! This library prioritizes forensic correctness:
//! - Read-only operations (never modifies source files)
//! - Hash verification against stored checksums
//! - Companion log parsing for chain-of-custody metadata
//! - Byte-accurate extraction preserving timestamps

pub mod ad1;  // AccessData Logical Image (FTK)
pub mod archive;  // Archive formats (7z, ZIP, RAR, etc.)
pub mod common;  // Shared utilities (hash, binary, segments)
pub mod containers;  // Container abstraction layer
pub mod database;  // SQLite persistence layer
pub mod ewf;  // Expert Witness Format (E01/L01/Ex01/Lx01) parser
pub mod formats;  // Centralized format definitions and detection
pub mod logging;  // Logging and tracing configuration
pub mod processed;  // Processed forensic databases (AXIOM, PA, etc.)
pub mod project;  // Project file handling (.ffxproj)
pub mod raw;  // Raw disk images (.dd, .raw, .img, .001, etc.)
pub mod report;  // Forensic report generation (PDF, DOCX, HTML)
pub mod ufed;  // UFED containers (UFD, UFDR, UFDX)
pub mod viewer;  // Hex/text file viewer

use tauri::Emitter;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::thread;
use tracing::{debug, info, warn, error, instrument};

#[tauri::command]
fn logical_info(
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
fn logical_info_fast(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<containers::ContainerInfo, String> {
    containers::info_fast(&inputPath)
}

/// Get just the tree entries for a container - faster than full info
/// Use this for populating the evidence tree UI
#[tauri::command]
async fn container_get_tree(
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
async fn container_get_children(
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
async fn container_get_children_at_addr(
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
async fn container_read_entry_by_addr(
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

// ============================================================================
// V2 Container Commands (Based on libad1 C implementation)
// ============================================================================

/// Get root children using V2 implementation (improved from libad1)
#[tauri::command]
async fn container_get_root_children_v2(
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
async fn container_get_children_at_addr_v2(
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
async fn container_read_file_data_v2(
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
async fn container_get_item_info_v2(
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

/// Verify item hash using V2 implementation
#[tauri::command]
async fn container_verify_item_hash_v2(
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
async fn container_verify_all_v2(
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
async fn container_get_info_v2(
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
async fn container_extract_all_v2(
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
async fn container_extract_item_v2(
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

// ============================================================================
// Archive Tree Listing
// ============================================================================

/// Archive entry for tree display (matches ArchiveEntry from extraction.rs)
#[derive(Debug, Clone, serde::Serialize)]
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
async fn archive_get_metadata(
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
async fn archive_get_tree(
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
async fn archive_extract_entry(
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

// ============================================================================
// UFED Tree Listing
// ============================================================================

/// UFED tree entry for tree display
#[derive(Debug, Clone, serde::Serialize)]
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
async fn ufed_get_tree(
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

#[tauri::command]
fn logical_verify(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
) -> Result<Vec<containers::VerifyEntry>, String> {
    containers::verify(&inputPath, &algorithm)
}

/// Hash all AD1 segment files to produce a single hash of the container image.
/// This is different from logical_verify which verifies internal file hashes.
#[tauri::command]
async fn ad1_hash_segments(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
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

/// Read file content from within a forensic container (AD1, etc.)
/// Returns raw bytes of the file entry - uses lazy loading for speed
#[tauri::command]
async fn container_read_entry(
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
async fn container_read_entry_chunk(
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

/// Get metadata for a specific entry in a forensic container
#[tauri::command]
fn container_get_entry_info(
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

#[tauri::command]
fn logical_extract(
    #[allow(non_snake_case)]
    inputPath: String,
    #[allow(non_snake_case)]
    outputDir: String,
) -> Result<(), String> {
    containers::extract(&inputPath, &outputDir)
}

#[tauri::command]
fn scan_directory(
    #[allow(non_snake_case)]
    dirPath: String,
) -> Result<Vec<containers::DiscoveredFile>, String> {
    containers::scan_directory(&dirPath).map_err(|e| e.to_string())
}

#[tauri::command]
fn scan_directory_recursive(
    #[allow(non_snake_case)]
    dirPath: String,
) -> Result<Vec<containers::DiscoveredFile>, String> {
    containers::scan_directory_recursive(&dirPath).map_err(|e| e.to_string())
}

#[tauri::command]
#[instrument(skip(window), fields(path = %dirPath, recursive))]
async fn scan_directory_streaming(
    window: tauri::Window,
    #[allow(non_snake_case)]
    dirPath: String,
    recursive: bool,
) -> Result<usize, String> {
    use tokio::sync::mpsc;
    
    info!("Starting directory scan");
    let (tx, mut rx) = mpsc::unbounded_channel::<containers::DiscoveredFile>();
    
    // Spawn blocking directory scan in background thread
    let dir_path_clone = dirPath.clone();
    let scan_handle = tauri::async_runtime::spawn_blocking(move || {
        containers::scan_directory_streaming(&dir_path_clone, recursive, |file| {
            let _ = tx.send(file.clone());
        })
    });
    
    // Stream results to frontend as they arrive
    let mut emitted = 0usize;
    while let Some(file) = rx.recv().await {
        debug!(file = %file.filename, "Found file");
        let _ = window.emit("scan-file-found", &file);
        emitted += 1;
    }
    
    // Wait for scan to complete and return count
    let result = scan_handle.await.map_err(|e| format!("Task failed: {e}"))?;
    info!(count = emitted, "Scan complete");
    result.map_err(|e| e.to_string())
}

// ============================================================================
// Path and Discovery Utilities - for Project Setup Wizard
// ============================================================================

/// Check if a path exists (file or directory)
#[tauri::command]
fn path_exists(path: String) -> Result<bool, String> {
    let path = std::path::PathBuf::from(&path);
    Ok(path.exists())
}

/// Check if a path is a directory
#[tauri::command]
fn path_is_directory(path: String) -> Result<bool, String> {
    let path = std::path::PathBuf::from(&path);
    Ok(path.is_dir())
}

/// Discover evidence files (E01, AD1, L01, etc.) in a directory
/// Returns just the file paths for quick discovery
#[tauri::command]
fn discover_evidence_files(
    #[allow(non_snake_case)]
    dirPath: String,
    recursive: bool,
) -> Result<Vec<String>, String> {
    let path = std::path::PathBuf::from(&dirPath);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }
    
    let files = if recursive {
        containers::scan_directory_recursive(&dirPath)?
    } else {
        containers::scan_directory(&dirPath)?
    };
    
    Ok(files.into_iter().map(|f| f.path).collect())
}

/// Scan for processed databases (AXIOM, Cellebrite, etc.) and return them
/// Returns ProcessedDbInfo directly (can be converted to ProcessedDatabase in frontend)
#[tauri::command]
fn scan_for_processed_databases(
    #[allow(non_snake_case)]
    dirPath: String,
) -> Result<Vec<processed::types::ProcessedDbInfo>, String> {
    use std::path::PathBuf;
    
    let path = PathBuf::from(&dirPath);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    // Use the processed database scanner
    let dbs = processed::detection::scan_for_processed_dbs(&path, true);
    
    Ok(dbs)
}

// EWF Commands - Expert Witness Format implementation (E01/L01/Ex01/Lx01)
#[tauri::command]
async fn e01_v3_info(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<ewf::EwfInfo, String> {
    // Run on blocking thread pool to prevent UI freeze during file parsing
    tauri::async_runtime::spawn_blocking(move || {
        ewf::info(&inputPath)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
    .map_err(|e| e.to_string())
}

#[derive(Clone, serde::Serialize)]
struct VerifyProgress {
    path: String,
    current: usize,
    total: usize,
    percent: f64,
}

#[tauri::command]
async fn e01_v3_verify(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let path_for_closure = inputPath.clone();
    // Run on blocking thread pool to prevent UI freeze
    tauri::async_runtime::spawn_blocking(move || {
        ewf::verify_with_progress(&inputPath, &algorithm, |current, total| {
            let percent = (current as f64 / total as f64) * 100.0;
            let _ = app.emit("verify-progress", VerifyProgress {
                path: path_for_closure.clone(),
                current,
                total,
                percent,
            });
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
    .map_err(|e| e.to_string())
}

/// Read bytes from E01 image at arbitrary offset (for filesystem browsing)
#[tauri::command]
async fn e01_read_at(
    #[allow(non_snake_case)]
    inputPath: String,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut handle = ewf::EwfHandle::open(&inputPath)?;
        handle.read_at(offset, length)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
    .map_err(|e| e.to_string())
}

/// Get media info for E01 (size, sector size, chunk size)
#[tauri::command]
async fn e01_media_info(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<E01MediaInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let handle = ewf::EwfHandle::open(&inputPath)?;
        Ok(E01MediaInfo {
            media_size: handle.get_media_size(),
            chunk_size: handle.get_chunk_size(),
            sector_size: handle.get_volume_info().bytes_per_sector,
            sector_count: handle.get_volume_info().sector_count,
            chunk_count: handle.get_chunk_count() as u64,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[derive(Clone, serde::Serialize)]
struct E01MediaInfo {
    media_size: u64,
    chunk_size: u32,
    sector_size: u32,
    sector_count: u64,
    chunk_count: u64,
}

// RAW Commands - Raw disk image implementation (.dd, .raw, .img, .001)
#[tauri::command]
async fn raw_info(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<raw::RawInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        raw::info(&inputPath).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
async fn raw_verify(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let path_for_closure = inputPath.clone();
    tauri::async_runtime::spawn_blocking(move || {
        raw::verify_with_progress(&inputPath, &algorithm, |current, total| {
            let percent = (current as f64 / total as f64) * 100.0;
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

/// Verify individual segments of a raw image, comparing against stored hashes
#[derive(Clone, serde::Serialize)]
struct SegmentVerifyProgress {
    segment_name: String,
    segment_number: u32,
    percent: f64,
    segments_completed: usize,
    segments_total: usize,
}

#[derive(Clone, serde::Serialize)]
struct SegmentHashResult {
    segment_name: String,
    segment_number: u32,
    segment_path: String,
    algorithm: String,
    computed_hash: String,
    expected_hash: Option<String>,
    verified: Option<bool>,  // None = no expected, true = match, false = mismatch
    size: u64,
    duration_secs: f64,
}

#[tauri::command]
async fn raw_verify_segments(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    #[allow(non_snake_case)]
    expectedHashes: Vec<containers::SegmentHash>,  // Optional: stored hashes from companion log
    app: tauri::AppHandle,
) -> Result<Vec<SegmentHashResult>, String> {
    use std::sync::Mutex;
    use std::time::Instant;
    
    // Get all segment paths
    let segment_paths = raw::get_segment_paths(&inputPath).map_err(|e| e.to_string())?;
    let num_segments = segment_paths.len();
    
    if num_segments == 0 {
        return Err("No segments found".to_string());
    }
    
    // Build expected hash lookup (by segment name, case-insensitive)
    let expected_map: std::collections::HashMap<String, String> = expectedHashes
        .iter()
        .map(|h| (h.segment_name.to_lowercase(), h.hash.clone()))
        .collect();
    
    // Use rayon for parallel processing
    let num_cpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let parallelism = num_cpus.min(num_segments);
    
    let segments_completed = Arc::new(AtomicUsize::new(0));
    let results: Arc<Mutex<Vec<SegmentHashResult>>> = Arc::new(Mutex::new(Vec::with_capacity(num_segments)));
    let app = Arc::new(app);
    let algorithm = Arc::new(algorithm);
    let expected_map = Arc::new(expected_map);
    
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(parallelism)
        .build()
        .map_err(|e| format!("Failed to create thread pool: {}", e))?;
    
    pool.scope(|s| {
        for (idx, seg_path) in segment_paths.into_iter().enumerate() {
            let segments_completed = Arc::clone(&segments_completed);
            let results = Arc::clone(&results);
            let app = Arc::clone(&app);
            let algorithm = Arc::clone(&algorithm);
            let expected_map = Arc::clone(&expected_map);
            let segment_number = (idx + 1) as u32;
            
            s.spawn(move |_| {
                let segment_name: String = seg_path.file_name()
                    .map(|f: &std::ffi::OsStr| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("segment_{}", segment_number));
                let seg_path_str = seg_path.to_string_lossy().to_string();
                
                // Emit start event
                let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                    segment_name: segment_name.clone(),
                    segment_number,
                    percent: 0.0,
                    segments_completed: segments_completed.load(Ordering::Relaxed),
                    segments_total: num_segments,
                });
                
                let start_time = Instant::now();
                
                // Hash the segment
                let hash_result = raw::hash_single_segment(&seg_path_str, &algorithm, |current, total| {
                    let percent = (current as f64 / total as f64) * 100.0;
                    let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                        segment_name: segment_name.clone(),
                        segment_number,
                        percent,
                        segments_completed: segments_completed.load(Ordering::Relaxed),
                        segments_total: num_segments,
                    });
                }).map_err(|e| e.to_string());
                
                let duration = start_time.elapsed().as_secs_f64();
                let completed = segments_completed.fetch_add(1, Ordering::SeqCst) + 1;
                
                // Get file size
                let size = std::fs::metadata(&seg_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                
                match hash_result {
                    Ok(computed_hash) => {
                        // Look up expected hash
                        let expected_hash = expected_map.get(&segment_name.to_lowercase()).cloned();
                        let verified = expected_hash.as_ref().map(|expected| {
                            computed_hash.to_lowercase() == expected.to_lowercase()
                        });
                        
                        let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                            segment_name: segment_name.clone(),
                            segment_number,
                            percent: 100.0,
                            segments_completed: completed,
                            segments_total: num_segments,
                        });
                        
                        // Use if-let to handle potential mutex poisoning gracefully
                        if let Ok(mut guard) = results.lock() {
                            guard.push(SegmentHashResult {
                                segment_name,
                                segment_number,
                                segment_path: seg_path_str,
                                algorithm: algorithm.to_uppercase(),
                                computed_hash,
                                expected_hash,
                                verified,
                                size,
                                duration_secs: duration,
                            });
                        } else {
                            tracing::error!("Mutex poisoned while storing hash result for segment {}", segment_number);
                        }
                    }
                    Err(e) => {
                        // Return error result for this segment
                        if let Ok(mut guard) = results.lock() {
                            guard.push(SegmentHashResult {
                                segment_name,
                                segment_number,
                                segment_path: seg_path_str,
                                algorithm: algorithm.to_uppercase(),
                                computed_hash: format!("ERROR: {}", e),
                                expected_hash: None,
                                verified: None,
                                size,
                                duration_secs: duration,
                            });
                        } else {
                            tracing::error!("Mutex poisoned while storing error result for segment {}", segment_number);
                        }
                    }
                }
            });
        }
    });
    
    let mut final_results = Arc::try_unwrap(results)
        .map_err(|_| "Failed to unwrap results")?
        .into_inner()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    // Sort by segment number
    final_results.sort_by_key(|r| r.segment_number);
    
    Ok(final_results)
}

/// Verify individual E01 segment files by hashing each .E01, .E02, etc. file
#[tauri::command]
async fn e01_verify_segments(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    #[allow(non_snake_case)]
    expectedHashes: Vec<containers::SegmentHash>,
    app: tauri::AppHandle,
) -> Result<Vec<SegmentHashResult>, String> {
    use std::sync::Mutex;
    use std::time::Instant;
    
    // Get all segment paths
    let segment_paths = ewf::get_segment_paths(&inputPath)?;
    let num_segments = segment_paths.len();
    
    if num_segments == 0 {
        return Err("No E01 segments found".to_string());
    }
    
    // Build expected hash lookup (by segment name, case-insensitive)
    let expected_map: std::collections::HashMap<String, String> = expectedHashes
        .iter()
        .map(|h| (h.segment_name.to_lowercase(), h.hash.clone()))
        .collect();
    
    let num_cpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let parallelism = num_cpus.min(num_segments);
    
    let segments_completed = Arc::new(AtomicUsize::new(0));
    let results: Arc<Mutex<Vec<SegmentHashResult>>> = Arc::new(Mutex::new(Vec::with_capacity(num_segments)));
    let app = Arc::new(app);
    let algorithm = Arc::new(algorithm);
    let expected_map = Arc::new(expected_map);
    
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(parallelism)
        .build()
        .map_err(|e| format!("Failed to create thread pool: {}", e))?;
    
    pool.scope(|s| {
        for (idx, seg_path) in segment_paths.into_iter().enumerate() {
            let segments_completed = Arc::clone(&segments_completed);
            let results = Arc::clone(&results);
            let app = Arc::clone(&app);
            let algorithm = Arc::clone(&algorithm);
            let expected_map = Arc::clone(&expected_map);
            let segment_number = (idx + 1) as u32;
            
            s.spawn(move |_| {
                let segment_name: String = seg_path.file_name()
                    .map(|f: &std::ffi::OsStr| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("segment_{}", segment_number));
                let seg_path_str = seg_path.to_string_lossy().to_string();
                
                // Emit start event
                let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                    segment_name: segment_name.clone(),
                    segment_number,
                    percent: 0.0,
                    segments_completed: segments_completed.load(Ordering::Relaxed),
                    segments_total: num_segments,
                });
                
                let start_time = Instant::now();
                
                // Hash the segment
                let hash_result = ewf::hash_single_segment(&seg_path_str, &algorithm, |current, total| {
                    let percent = (current as f64 / total as f64) * 100.0;
                    let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                        segment_name: segment_name.clone(),
                        segment_number,
                        percent,
                        segments_completed: segments_completed.load(Ordering::Relaxed),
                        segments_total: num_segments,
                    });
                });
                
                let duration = start_time.elapsed().as_secs_f64();
                let completed = segments_completed.fetch_add(1, Ordering::Relaxed) + 1;
                let size = std::fs::metadata(&seg_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                
                match hash_result {
                    Ok(computed_hash) => {
                        let expected_hash = expected_map.get(&segment_name.to_lowercase()).cloned();
                        let verified = expected_hash.as_ref().map(|expected| {
                            computed_hash.to_lowercase() == expected.to_lowercase()
                        });
                        
                        let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                            segment_name: segment_name.clone(),
                            segment_number,
                            percent: 100.0,
                            segments_completed: completed,
                            segments_total: num_segments,
                        });
                        
                        if let Ok(mut guard) = results.lock() {
                            guard.push(SegmentHashResult {
                                segment_name,
                                segment_number,
                                segment_path: seg_path_str,
                                algorithm: algorithm.to_uppercase(),
                                computed_hash,
                                expected_hash,
                                verified,
                                size,
                                duration_secs: duration,
                            });
                        } else {
                            tracing::error!("Mutex poisoned while storing E01 hash result for segment {}", segment_number);
                        }
                    }
                    Err(e) => {
                        if let Ok(mut guard) = results.lock() {
                            guard.push(SegmentHashResult {
                                segment_name,
                                segment_number,
                                segment_path: seg_path_str,
                                algorithm: algorithm.to_uppercase(),
                                computed_hash: format!("ERROR: {}", e),
                                expected_hash: None,
                                verified: None,
                                size,
                                duration_secs: duration,
                            });
                        } else {
                            tracing::error!("Mutex poisoned while storing E01 error result for segment {}", segment_number);
                        }
                    }
                }
            });
        }
    });
    
    let mut final_results = Arc::try_unwrap(results)
        .map_err(|_| "Failed to unwrap results")?
        .into_inner()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    final_results.sort_by_key(|r| r.segment_number);
    
    Ok(final_results)
}

// =============================================================================
// VFS Commands - Virtual Filesystem for disk image mounting
// =============================================================================

/// VFS entry returned to the frontend
#[derive(Clone, serde::Serialize)]
pub struct VfsEntry {
    /// Entry name
    pub name: String,
    /// Full path within the VFS
    pub path: String,
    /// Is this a directory?
    pub is_dir: bool,
    /// File size (0 for directories)
    pub size: u64,
    /// File type hint (from extension or magic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
}

/// Partition information for mounted disk images
#[derive(Clone, serde::Serialize)]
pub struct VfsPartitionInfo {
    /// Partition number (1-based)
    pub number: u32,
    /// Mount name (e.g., "Partition1_NTFS")
    pub mount_name: String,
    /// Filesystem type (NTFS, FAT32, etc.)
    pub fs_type: String,
    /// Partition size in bytes
    pub size: u64,
    /// Start offset in the disk image
    pub start_offset: u64,
}

/// Information about a mounted disk image
#[derive(Clone, serde::Serialize)]
pub struct VfsMountInfo {
    /// Container path
    pub container_path: String,
    /// Container type (e01, raw, etc.)
    pub container_type: String,
    /// Total disk size
    pub disk_size: u64,
    /// Detected partitions
    pub partitions: Vec<VfsPartitionInfo>,
    /// Mount mode (physical or filesystem)
    pub mode: String,
}

/// Mount a disk image (E01/Raw) and return partition information
#[tauri::command]
async fn vfs_mount_image(
    #[allow(non_snake_case)]
    containerPath: String,
) -> Result<VfsMountInfo, String> {
    use common::vfs::VirtualFileSystem;
    
    debug!("[vfs_mount_image] Starting mount for: {}", containerPath);
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = if ewf::is_ewf(&containerPath).unwrap_or(false) {
            "e01"
        } else if raw::is_raw(&containerPath).unwrap_or(false) {
            "raw"
        } else {
            return Err(format!("Unsupported container type for VFS: {}", containerPath));
        };
        
        debug!("[vfs_mount_image] Container type: {}", container_type);
        
        // Try to mount with filesystem mode first
        let (partitions, mode, disk_size) = if container_type == "e01" {
            debug!("[vfs_mount_image] Opening E01 in filesystem mode...");
            match ewf::vfs::EwfVfs::open_filesystem(&containerPath) {
                Ok(vfs) => {
                    debug!("[vfs_mount_image] E01 opened successfully in filesystem mode");
                    
                    // Get disk size from the VFS
                    let disk_size = vfs.disk_size().unwrap_or(0);
                    debug!("[vfs_mount_image] Disk size from vfs.disk_size(): {} bytes", disk_size);
                    
                    // Get partitions from readdir
                    let root_entries = vfs.readdir("/").unwrap_or_default();
                    debug!("[vfs_mount_image] Root entries count: {}", root_entries.len());
                    for entry in &root_entries {
                        debug!("[vfs_mount_image] Root entry: {} (is_dir: {})", entry.name, entry.is_directory);
                    }
                    
                    let parts: Vec<VfsPartitionInfo> = root_entries
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.is_directory)
                        .map(|(idx, e)| {
                            // Extract fs type from name (e.g., "Partition1_NTFS" -> "NTFS")
                            let fs_type = e.name.split('_').next_back().unwrap_or("Unknown").to_string();
                            // Get partition size from the mounted partition info
                            let part_size = vfs.get_partition_size(&e.name).unwrap_or(0);
                            debug!("[vfs_mount_image] Partition {}: {} size={}", idx + 1, e.name, part_size);
                            VfsPartitionInfo {
                                number: (idx + 1) as u32,
                                mount_name: e.name.clone(),
                                fs_type,
                                size: part_size,
                                start_offset: 0,
                            }
                        })
                        .collect();
                    
                    let mode = if parts.is_empty() { "physical" } else { "filesystem" };
                    debug!("[vfs_mount_image] Mode: {}, Partitions: {}, Disk size: {}", mode, parts.len(), disk_size);
                    (parts, mode.to_string(), disk_size)
                }
                Err(e) => {
                    debug!("[vfs_mount_image] Filesystem mode failed: {:?}, falling back to physical mode", e);
                    // Fall back to physical mode
                    match ewf::vfs::EwfVfs::open_physical(&containerPath) {
                        Ok(vfs) => {
                            debug!("[vfs_mount_image] Physical mode opened");
                            // Get disk size from disk_size() method
                            let disk_size = vfs.disk_size().unwrap_or(0);
                            debug!("[vfs_mount_image] Physical mode disk size: {} bytes", disk_size);
                            (Vec::new(), "physical".to_string(), disk_size)
                        }
                        Err(e) => return Err(format!("Failed to mount E01: {:?}", e)),
                    }
                }
            }
        } else {
            // Raw image
            match raw::vfs::RawVfs::open_filesystem(&containerPath) {
                Ok(vfs) => {
                    let parts: Vec<VfsPartitionInfo> = vfs.readdir("/")
                        .unwrap_or_default()
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.is_directory)
                        .map(|(idx, e)| {
                            let fs_type = e.name.split('_').next_back().unwrap_or("Unknown").to_string();
                            VfsPartitionInfo {
                                number: (idx + 1) as u32,
                                mount_name: e.name.clone(),
                                fs_type,
                                size: 0,
                                start_offset: 0,
                            }
                        })
                        .collect();
                    
                    let mode = if parts.is_empty() { "physical" } else { "filesystem" };
                    let disk_size = vfs.getattr("/")
                        .map(|a| a.size)
                        .unwrap_or(0);
                    (parts, mode.to_string(), disk_size)
                }
                Err(_) => {
                    match raw::vfs::RawVfs::open(&containerPath) {
                        Ok(_vfs) => (Vec::new(), "physical".to_string(), 0),
                        Err(e) => return Err(format!("Failed to mount raw image: {:?}", e)),
                    }
                }
            }
        };
        
        Ok(VfsMountInfo {
            container_path: containerPath,
            container_type: container_type.to_string(),
            disk_size,
            partitions,
            mode,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// List directory contents in a mounted VFS
#[tauri::command]
async fn vfs_list_dir(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    dirPath: String,
) -> Result<Vec<VfsEntry>, String> {
    use common::vfs::VirtualFileSystem;
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = if ewf::is_ewf(&containerPath).unwrap_or(false) {
            "e01"
        } else if raw::is_raw(&containerPath).unwrap_or(false) {
            "raw"
        } else {
            return Err(format!("Unsupported container type: {}", containerPath));
        };
        
        let entries = if container_type == "e01" {
            let vfs = ewf::vfs::EwfVfs::open(&containerPath)
                .map_err(|e| format!("Failed to open E01: {:?}", e))?;
            
            let dir_entries = vfs.readdir(&dirPath)
                .map_err(|e| format!("Failed to read directory: {:?}", e))?;
            
            dir_entries.into_iter().map(|e| {
                let full_path = if dirPath == "/" {
                    format!("/{}", e.name)
                } else {
                    format!("{}/{}", dirPath, e.name)
                };
                
                let size = vfs.getattr(&full_path)
                    .map(|a| a.size)
                    .unwrap_or(0);
                
                VfsEntry {
                    name: e.name,
                    path: full_path,
                    is_dir: e.is_directory,
                    size,
                    file_type: None,
                }
            }).collect()
        } else {
            let vfs = raw::vfs::RawVfs::open_filesystem(&containerPath)
                .or_else(|_| raw::vfs::RawVfs::open(&containerPath))
                .map_err(|e| format!("Failed to open raw: {:?}", e))?;
            
            let dir_entries = vfs.readdir(&dirPath)
                .map_err(|e| format!("Failed to read directory: {:?}", e))?;
            
            dir_entries.into_iter().map(|e| {
                let full_path = if dirPath == "/" {
                    format!("/{}", e.name)
                } else {
                    format!("{}/{}", dirPath, e.name)
                };
                
                let size = vfs.getattr(&full_path)
                    .map(|a| a.size)
                    .unwrap_or(0);
                
                VfsEntry {
                    name: e.name,
                    path: full_path,
                    is_dir: e.is_directory,
                    size,
                    file_type: None,
                }
            }).collect()
        };
        
        Ok(entries)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read file content from a mounted VFS
#[tauri::command]
async fn vfs_read_file(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    filePath: String,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    use common::vfs::VirtualFileSystem;
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = if ewf::is_ewf(&containerPath).unwrap_or(false) {
            "e01"
        } else if raw::is_raw(&containerPath).unwrap_or(false) {
            "raw"
        } else {
            return Err(format!("Unsupported container type: {}", containerPath));
        };
        
        if container_type == "e01" {
            let vfs = ewf::vfs::EwfVfs::open(&containerPath)
                .map_err(|e| format!("Failed to open E01: {:?}", e))?;
            vfs.read(&filePath, offset, length)
                .map_err(|e| format!("Failed to read file: {:?}", e))
        } else {
            let vfs = raw::vfs::RawVfs::open_filesystem(&containerPath)
                .or_else(|_| raw::vfs::RawVfs::open(&containerPath))
                .map_err(|e| format!("Failed to open raw: {:?}", e))?;
            vfs.read(&filePath, offset, length)
                .map_err(|e| format!("Failed to read file: {:?}", e))
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get file attributes from a mounted VFS
#[tauri::command]
async fn vfs_get_attr(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    filePath: String,
) -> Result<VfsEntry, String> {
    use common::vfs::VirtualFileSystem;
    
    tauri::async_runtime::spawn_blocking(move || {
        let container_type = if ewf::is_ewf(&containerPath).unwrap_or(false) {
            "e01"
        } else if raw::is_raw(&containerPath).unwrap_or(false) {
            "raw"
        } else {
            return Err(format!("Unsupported container type: {}", containerPath));
        };
        
        let attr = if container_type == "e01" {
            let vfs = ewf::vfs::EwfVfs::open(&containerPath)
                .map_err(|e| format!("Failed to open E01: {:?}", e))?;
            vfs.getattr(&filePath)
                .map_err(|e| format!("Failed to get attributes: {:?}", e))?
        } else {
            let vfs = raw::vfs::RawVfs::open_filesystem(&containerPath)
                .or_else(|_| raw::vfs::RawVfs::open(&containerPath))
                .map_err(|e| format!("Failed to open raw: {:?}", e))?;
            vfs.getattr(&filePath)
                .map_err(|e| format!("Failed to get attributes: {:?}", e))?
        };
        
        let name = filePath.split('/').next_back().unwrap_or(&filePath).to_string();
        
        Ok(VfsEntry {
            name,
            path: filePath,
            is_dir: attr.is_directory,
            size: attr.size,
            file_type: None,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// =============================================================================
// End VFS Commands
// =============================================================================

// Batch hashing result for a single file
#[derive(Clone, serde::Serialize)]
struct BatchHashResult {
    path: String,
    algorithm: String,
    hash: Option<String>,
    error: Option<String>,
    duration_ms: Option<u64>,
    throughput_mbs: Option<f64>,
}

// Progress update for batch hashing - includes hash result when completed
#[derive(Clone, serde::Serialize)]
struct BatchProgress {
    path: String,
    status: String,  // "started", "progress", "completed", "error"
    percent: f64,
    files_completed: usize,
    files_total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    // Decompression progress (for E01/compressed containers)
    #[serde(skip_serializing_if = "Option::is_none")]
    chunks_processed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chunks_total: Option<usize>,
}

/// Hash multiple files in parallel with smart scheduling
/// 
/// Optimizations:
/// Simple sequential batch hash - hash files one by one with progress updates
#[tauri::command]
#[instrument(skip(files, app), fields(num_files = files.len(), algorithm = %algorithm))]
async fn batch_hash(
    files: Vec<BatchFileInput>,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<Vec<BatchHashResult>, String> {
    let num_files = files.len();
    info!("Starting parallel batch hash");
    if num_files == 0 {
        return Ok(Vec::new());
    }
    
    // Determine parallelism based on available CPU cores
    // Use all available cores for maximum throughput
    let num_cpus = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);
    // Allow processing up to num_cpus files concurrently (or fewer for small batches)
    let max_concurrent = num_cpus.min(num_files);
    debug!(max_concurrent, num_cpus, "Parallel file limit set based on CPU cores");
    
    // Use a semaphore to limit concurrent file processing
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    
    // Spawn all file processing tasks
    let mut handles = Vec::with_capacity(num_files);
    
    for (idx, file) in files.into_iter().enumerate() {
        let path = file.path.clone();
        let container_type = file.container_type.to_lowercase();
        let algo = algorithm.clone();
        let app_clone = app.clone();
        let sem = semaphore.clone();
        
        // Emit progress: queued
        let _ = app.emit("batch-progress", BatchProgress {
            path: path.clone(),
            status: "queued".to_string(),
            percent: 0.0,
            files_completed: 0,
            files_total: num_files,
            hash: None,
            algorithm: None,
            error: None,
            chunks_processed: None,
            chunks_total: None,
        });
        
        let handle = tauri::async_runtime::spawn(async move {
            // Acquire semaphore permit (limits concurrent files)
            let _permit = sem.acquire_owned().await.map_err(|e| format!("Semaphore error: {}", e))?;
            
            debug!(idx = idx + 1, total = num_files, path = %path, "File started");
            
            // Emit progress: started
            let _ = app_clone.emit("batch-progress", BatchProgress {
                path: path.clone(),
                status: "started".to_string(),
                percent: 0.0,
                files_completed: idx,
                files_total: num_files,
                hash: None,
                algorithm: None,
                error: None,
                chunks_processed: None,
                chunks_total: None,
            });
            
            let path_for_hash = path.clone();
            let algo_for_hash = algo.clone();
            let container_for_hash = container_type.clone();
            let app_for_hash = app_clone.clone();
            
            // Run blocking hash in spawn_blocking
            let hash_result = tauri::async_runtime::spawn_blocking(move || {
                let start_time = std::time::Instant::now();
                let file_size = std::fs::metadata(&path_for_hash).map(|m| m.len()).unwrap_or(0);
                debug!(idx = idx + 1, size_mb = file_size / 1024 / 1024, "Processing file");
                
                // Progress counters
                let progress_current = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
                let progress_total = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(1)); // Start with 1 to avoid div by zero
                let done_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                
                // Progress reporter thread
                let progress_current_clone = progress_current.clone();
                let progress_total_clone = progress_total.clone();
                let done_flag_clone = done_flag.clone();
                let app_for_timer = app_for_hash.clone();
                let path_for_timer = path_for_hash.clone();
                let progress_thread = std::thread::spawn(move || {
                    let mut last_percent = 0u32;
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        if done_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }
                        let current = progress_current_clone.load(std::sync::atomic::Ordering::Relaxed);
                        let total = progress_total_clone.load(std::sync::atomic::Ordering::Relaxed);
                        if total > 1 {
                            let percent = ((current as f64 / total as f64) * 100.0) as u32;
                            // Only emit if percent increased (prevents bouncing)
                            if percent > last_percent {
                                let _ = app_for_timer.emit("batch-progress", BatchProgress {
                                    path: path_for_timer.clone(),
                                    status: "progress".to_string(),
                                    percent: percent as f64,
                                    files_completed: idx,
                                    files_total: num_files,
                                    hash: None,
                                    algorithm: None,
                                    error: None,
                                    chunks_processed: Some(current),
                                    chunks_total: Some(total),
                                });
                                last_percent = percent;
                            }
                        }
                    }
                });
                
                // Hash based on container type
                let result: Result<String, String> = if container_for_hash.contains("e01") || container_for_hash.contains("encase") || container_for_hash.contains("ex01") {
                    ewf::verify_with_progress(&path_for_hash, &algo_for_hash, |current: usize, total: usize| {
                        progress_total.store(total, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                } else if container_for_hash.contains("raw") || container_for_hash.contains("dd") {
                    raw::verify_with_progress(&path_for_hash, &algo_for_hash, |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                } else if container_for_hash.contains("ufed") || container_for_hash.contains("zip") || container_for_hash.contains("archive") || container_for_hash.contains("tar") || container_for_hash.contains("7z") {
                    // UFED containers, archives (ZIP, TAR, 7z) - hash the file directly
                    raw::verify_with_progress(&path_for_hash, &algo_for_hash, |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                } else if container_for_hash.contains("ad1") {
                    // AD1 containers - hash the segment files (image-level hash)
                    ad1::hash_segments_with_progress(&path_for_hash, &algo_for_hash, |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                } else if container_for_hash.contains("l01") {
                    // L01 containers - verify and return hash from message
                    containers::verify(&path_for_hash, &algo_for_hash)
                        .map(|entries| {
                            entries.first()
                                .and_then(|e| e.message.clone())
                                .unwrap_or_else(|| "Verified".to_string())
                        })
                } else {
                    // Unknown - try raw verification
                    raw::verify_with_progress(&path_for_hash, &algo_for_hash, |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                };
                
                // Stop progress thread
                done_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                let _ = progress_thread.join();
                
                let duration = start_time.elapsed();
                let duration_ms = duration.as_millis() as u64;
                let throughput_mbs = if duration_ms > 0 && file_size > 0 {
                    Some((file_size as f64 / (1024.0 * 1024.0)) / (duration_ms as f64 / 1000.0))
                } else {
                    None
                };
                
                (result, duration_ms, throughput_mbs)
            }).await.map_err(|e| format!("Task error: {}", e))?;
            
            let (result, duration_ms, throughput_mbs) = hash_result;
            
            // Build result
            let batch_result = match result {
                Ok(hash) => {
                    debug!(idx = idx + 1, hash_prefix = %&hash[..8.min(hash.len())], "File completed");
                    let _ = app_clone.emit("batch-progress", BatchProgress {
                        path: path.clone(),
                        status: "completed".to_string(),
                        percent: 100.0,
                        files_completed: idx + 1,
                        files_total: num_files,
                        hash: Some(hash.clone()),
                        algorithm: Some(algo.to_uppercase()),
                        error: None,
                        chunks_processed: None,
                        chunks_total: None,
                    });
                    BatchHashResult {
                        path,
                        algorithm: algo.to_uppercase(),
                        hash: Some(hash),
                        error: None,
                        duration_ms: Some(duration_ms),
                        throughput_mbs,
                    }
                }
                Err(e) => {
                    debug!(idx = idx + 1, error = %e, "File error");
                    let _ = app_clone.emit("batch-progress", BatchProgress {
                        path: path.clone(),
                        status: "error".to_string(),
                        percent: 0.0,
                        files_completed: idx + 1,
                        files_total: num_files,
                        hash: None,
                        algorithm: None,
                        error: Some(e.clone()),
                        chunks_processed: None,
                        chunks_total: None,
                    });
                    BatchHashResult {
                        path,
                        algorithm: algo.to_uppercase(),
                        hash: None,
                        error: Some(e),
                        duration_ms: Some(duration_ms),
                        throughput_mbs: None,
                    }
                }
            };
            
            Ok::<BatchHashResult, String>(batch_result)
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete and collect results
    let mut results = Vec::with_capacity(num_files);
    for handle in handles {
        match handle.await {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(e)) => {
                debug!(error = %e, "Task error");
                // Error already emitted in the task
            }
            Err(e) => {
                debug!(error = %e, "Join error");
            }
        }
    }
    
    info!(num_files, results = results.len(), "Batch hash complete");
    Ok(results)
}

#[derive(Clone, serde::Deserialize)]
struct BatchFileInput {
    path: String,
    container_type: String,
}

// System Stats Command
#[derive(Clone, serde::Serialize)]
struct SystemStats {
    cpu_usage: f32,
    memory_used: u64,
    memory_total: u64,
    memory_percent: f32,
    // App-specific stats
    app_cpu_usage: f32,
    app_memory: u64,
    app_threads: usize,
    cpu_cores: usize,
}

use std::sync::{OnceLock, Mutex as StdMutex};

static SYSTEM: OnceLock<StdMutex<sysinfo::System>> = OnceLock::new();

fn get_system() -> &'static StdMutex<sysinfo::System> {
    SYSTEM.get_or_init(|| {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        StdMutex::new(sys)
    })
}

fn collect_system_stats() -> SystemStats {
    let Ok(mut sys) = get_system().lock() else {
        // Return default stats if lock is poisoned
        tracing::warn!("System stats lock poisoned, returning defaults");
        return SystemStats {
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            memory_percent: 0.0,
            app_cpu_usage: 0.0,
            app_memory: 0,
            app_threads: 0,
            cpu_cores: 0,
        };
    };
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    
    let cpu_usage = sys.global_cpu_usage();
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let memory_percent = if memory_total > 0 {
        (memory_used as f32 / memory_total as f32) * 100.0
    } else {
        0.0
    };
    
    // Get app-specific stats
    let pid = sysinfo::Pid::from_u32(std::process::id());
    let (app_cpu_usage, app_memory, app_threads) = if let Some(process) = sys.process(pid) {
        // process.tasks() is not supported on macOS, use rayon thread count as worker threads
        let threads = process.tasks()
            .map(|t| t.len())
            .unwrap_or_else(rayon::current_num_threads);
        (process.cpu_usage(), process.memory(), threads)
    } else {
        (0.0, 0, rayon::current_num_threads())
    };
    
    let cpu_cores = sys.cpus().len();
    
    SystemStats {
        cpu_usage,
        memory_used,
        memory_total,
        memory_percent,
        app_cpu_usage,
        app_memory,
        app_threads,
        cpu_cores,
    }
}

#[tauri::command]
fn get_system_stats() -> SystemStats {
    collect_system_stats()
}

// =============================================================================
// Data Viewing & Analysis Commands
// =============================================================================

/// Read raw bytes from a file at specified offset
/// 
/// Returns up to `length` bytes starting at `offset`.
/// Useful for previewing file contents without full extraction.
#[tauri::command]
fn read_file_bytes(
    path: String,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    
    let mut file = File::open(&path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let file_size = file.metadata()
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();
    
    if offset >= file_size {
        return Ok(Vec::new());
    }
    
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("Failed to seek: {}", e))?;
    
    let read_len = length.min((file_size - offset) as usize);
    let mut buffer = vec![0u8; read_len];
    
    file.read_exact(&mut buffer)
        .map_err(|e| format!("Failed to read: {}", e))?;
    
    Ok(buffer)
}

/// Get hex dump of file contents
///
/// Reads bytes from file and returns formatted hex dump string.
#[tauri::command]
fn hex_dump(
    path: String,
    offset: u64,
    length: usize,
    #[allow(non_snake_case)]
    showAscii: Option<bool>,
    #[allow(non_snake_case)]
    bytesPerLine: Option<usize>,
) -> Result<common::hex::HexDumpResult, String> {
    let data = read_file_bytes(path, offset, length)?;
    
    let options = common::hex::HexDumpOptions {
        show_ascii: showAscii.unwrap_or(true),
        bytes_per_line: bytesPerLine.unwrap_or(16),
        show_offset: true,
        uppercase: true,
        group_size: 1,
        start_offset: offset,
    };
    
    Ok(common::hex::create_hex_dump(&data, &options))
}

/// Detect file type from magic signature
///
/// Reads file header and identifies type based on magic bytes.
/// Returns None if file type cannot be determined.
#[tauri::command]
fn detect_file_type(path: String) -> Result<Option<common::magic::FileType>, String> {
    // Read first 64 bytes for magic detection
    let header = read_file_bytes(path, 0, 64)?;
    Ok(common::magic::detect_file_type(&header))
}

/// Analyze entropy of a file or portion of a file
///
/// Returns entropy statistics useful for detecting encryption.
#[tauri::command]
fn analyze_file_entropy(
    path: String,
    offset: Option<u64>,
    length: Option<usize>,
) -> Result<common::entropy::EntropyResult, String> {
    let start = offset.unwrap_or(0);
    // Default to 1MB sample for entropy analysis
    let len = length.unwrap_or(1024 * 1024);
    
    let data = read_file_bytes(path, start, len)?;
    
    Ok(common::entropy::EntropyResult::new(&data)
        .with_offset(start))
}

/// Analyze entropy across file blocks
///
/// Useful for finding encrypted regions in disk images.
#[tauri::command]
fn analyze_entropy_blocks(
    path: String,
    #[allow(non_snake_case)]
    blockSize: Option<usize>,
    #[allow(non_snake_case)]
    maxBlocks: Option<usize>,
) -> Result<common::entropy::BlockEntropyAnalysis, String> {
    use std::fs::File;
    use std::io::Read;
    
    let block_size = blockSize.unwrap_or(4096);
    let max_blocks = maxBlocks.unwrap_or(1000);
    
    let mut file = File::open(&path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let file_size = file.metadata()
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();
    
    // Limit data read to prevent memory issues
    let max_bytes = block_size * max_blocks;
    let read_len = (file_size as usize).min(max_bytes);
    
    let mut data = vec![0u8; read_len];
    file.read_exact(&mut data)
        .map_err(|e| format!("Failed to read: {}", e))?;
    
    Ok(common::entropy::analyze_blocks(&data, block_size))
}

/// Compare two hash values
///
/// Returns detailed comparison result including case-sensitivity.
#[tauri::command]
fn compare_hashes(
    hash1: String,
    hash2: String,
) -> common::hash::HashMatchResult {
    common::hash::compare_hashes(&hash1, &hash2)
}

/// Verify a file's hash against expected value
#[tauri::command]
fn verify_file_hash(
    path: String,
    expected: String,
    algorithm: String,
) -> Result<common::hash::HashVerificationResult, String> {
    let algo: common::hash::HashAlgorithm = algorithm.parse().map_err(|e: containers::ContainerError| e.to_string())?;
    common::hash::verify_file_hash(std::path::Path::new(&path), &expected, algo)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Database Commands
// ============================================================================

/// Get or create a session for a directory path
#[tauri::command]
fn db_get_or_create_session(root_path: String) -> Result<database::Session, String> {
    let db = database::get_db();
    db.get_or_create_session(&root_path).map_err(|e| e.to_string())
}

/// Get recent sessions
#[tauri::command]
fn db_get_recent_sessions(limit: i32) -> Result<Vec<database::Session>, String> {
    let db = database::get_db();
    db.get_recent_sessions(limit).map_err(|e| e.to_string())
}

/// Get the last opened session
#[tauri::command]
fn db_get_last_session() -> Result<Option<database::Session>, String> {
    let db = database::get_db();
    db.get_last_session().map_err(|e| e.to_string())
}

/// Save or update a file record
#[tauri::command]
fn db_upsert_file(file: database::FileRecord) -> Result<(), String> {
    let db = database::get_db();
    db.upsert_file(&file).map_err(|e| e.to_string())
}

/// Get all files for a session
#[tauri::command]
fn db_get_files_for_session(session_id: String) -> Result<Vec<database::FileRecord>, String> {
    let db = database::get_db();
    db.get_files_for_session(&session_id).map_err(|e| e.to_string())
}

/// Get a file by path
#[tauri::command]
fn db_get_file_by_path(session_id: String, path: String) -> Result<Option<database::FileRecord>, String> {
    let db = database::get_db();
    db.get_file_by_path(&session_id, &path).map_err(|e| e.to_string())
}

/// Insert a hash record
#[tauri::command]
fn db_insert_hash(hash: database::HashRecord) -> Result<(), String> {
    let db = database::get_db();
    db.insert_hash(&hash).map_err(|e| e.to_string())
}

/// Get all hashes for a file
#[tauri::command]
fn db_get_hashes_for_file(file_id: String) -> Result<Vec<database::HashRecord>, String> {
    let db = database::get_db();
    db.get_hashes_for_file(&file_id).map_err(|e| e.to_string())
}

/// Get the latest hash for a file/algorithm/segment combo
#[tauri::command]
fn db_get_latest_hash(
    file_id: String,
    algorithm: String,
    segment_index: Option<i32>,
) -> Result<Option<database::HashRecord>, String> {
    let db = database::get_db();
    db.get_latest_hash(&file_id, &algorithm, segment_index).map_err(|e| e.to_string())
}

/// Insert a verification record
#[tauri::command]
fn db_insert_verification(verification: database::VerificationRecord) -> Result<(), String> {
    let db = database::get_db();
    db.insert_verification(&verification).map_err(|e| e.to_string())
}

/// Get verifications for a file
#[tauri::command]
fn db_get_verifications_for_file(file_id: String) -> Result<Vec<database::VerificationRecord>, String> {
    let db = database::get_db();
    db.get_verifications_for_file(&file_id).map_err(|e| e.to_string())
}

/// Save open tabs for a session
#[tauri::command]
fn db_save_open_tabs(session_id: String, tabs: Vec<database::OpenTabRecord>) -> Result<(), String> {
    let db = database::get_db();
    db.save_open_tabs(&session_id, &tabs).map_err(|e| e.to_string())
}

/// Get open tabs for a session
#[tauri::command]
fn db_get_open_tabs(session_id: String) -> Result<Vec<database::OpenTabRecord>, String> {
    let db = database::get_db();
    db.get_open_tabs(&session_id).map_err(|e| e.to_string())
}

/// Set a setting value
#[tauri::command]
fn db_set_setting(key: String, value: String) -> Result<(), String> {
    let db = database::get_db();
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

/// Get a setting value
#[tauri::command]
fn db_get_setting(key: String) -> Result<Option<String>, String> {
    let db = database::get_db();
    db.get_setting(&key).map_err(|e| e.to_string())
}

// ============================================================================
// Project File Commands
// ============================================================================

/// Get the default project file path for a root directory
#[tauri::command]
fn project_get_default_path(root_path: String) -> String {
    project::get_default_project_path(&root_path)
        .to_string_lossy()
        .to_string()
}

/// Check if a project file exists for the given root directory
#[tauri::command]
fn project_check_exists(root_path: String) -> Option<String> {
    project::check_project_exists(&root_path)
}

/// Save a project to the specified path (or default if not provided)
#[tauri::command]
fn project_save(project: project::FFXProject, path: Option<String>) -> project::ProjectSaveResult {
    let mut proj = project;
    proj.touch(); // Update saved_at timestamp
    project::save_project(&proj, path.as_deref())
}

/// Load a project from the specified path
#[tauri::command]
fn project_load(path: String) -> project::ProjectLoadResult {
    project::load_project(&path)
}

/// Create a new project for a root directory
#[tauri::command]
fn project_create(root_path: String) -> project::FFXProject {
    project::FFXProject::new(&root_path)
}

// ============================================================================
// File Viewer Commands
// ============================================================================

/// Read a chunk of a file for hex viewing
#[tauri::command]
fn viewer_read_chunk(
    path: String, 
    offset: u64, 
    size: Option<usize>
) -> Result<viewer::FileChunk, String> {
    viewer::read_file_chunk(&path, offset, size).map_err(|e| e.to_string())
}

/// Detect file type from magic bytes and extension
#[tauri::command]
fn viewer_detect_type(path: String) -> Result<viewer::FileTypeInfo, String> {
    viewer::detect_file_type(&path).map_err(|e| e.to_string())
}

/// Parse file header and extract metadata with regions for hex highlighting
#[tauri::command]
fn viewer_parse_header(path: String) -> Result<viewer::ParsedMetadata, String> {
    viewer::parse_file_header(&path).map_err(|e| e.to_string())
}

/// Read file as text for text viewer
#[tauri::command]
fn viewer_read_text(path: String, offset: u64, max_chars: usize) -> Result<String, String> {
    viewer::read_file_text(&path, offset, max_chars).map_err(|e| e.to_string())
}

/// Start background system stats monitoring - emits "system-stats" events every 2 seconds
fn start_system_stats_monitor(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let stats = collect_system_stats();
            let _ = app_handle.emit("system-stats", stats);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(report::commands::ReportState::default())
        .setup(|app| {
            // Initialize database early (in background thread to not block startup)
            std::thread::spawn(|| {
                let _ = database::get_db();  // This triggers lazy initialization
                tracing::info!("Database initialized");
            });
            
            // Start background system stats monitoring
            start_system_stats_monitor(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            logical_info,
            logical_info_fast,
            container_get_tree,
            container_get_children,
            container_get_children_at_addr,
            container_read_entry,
            container_read_entry_by_addr,
            // V2 container commands (based on libad1)
            container_get_root_children_v2,
            container_get_children_at_addr_v2,
            container_read_file_data_v2,
            container_get_item_info_v2,
            container_verify_item_hash_v2,
            container_verify_all_v2,
            container_get_info_v2,
            container_extract_all_v2,
            container_extract_item_v2,
            // Archive and UFED tree listing
            archive_get_tree,
            archive_get_metadata,
            archive_extract_entry,
            ufed_get_tree,
            logical_verify,
            ad1_hash_segments,
            logical_extract,
            scan_directory,
            scan_directory_recursive,
            scan_directory_streaming,
            // Path and discovery utilities
            path_exists,
            path_is_directory,
            discover_evidence_files,
            scan_for_processed_databases,
            e01_v3_info,
            e01_v3_verify,
            e01_verify_segments,
            e01_read_at,
            e01_media_info,
            raw_info,
            raw_verify,
            raw_verify_segments,
            // VFS (Virtual Filesystem) commands for disk image mounting
            vfs_mount_image,
            vfs_list_dir,
            vfs_read_file,
            vfs_get_attr,
            batch_hash,
            get_system_stats,
            // Data viewing & analysis
            read_file_bytes,
            hex_dump,
            detect_file_type,
            analyze_file_entropy,
            analyze_entropy_blocks,
            compare_hashes,
            verify_file_hash,
            // Container entry reading (view files inside AD1, etc.)
            container_read_entry_chunk,
            container_get_entry_info,
            // Database operations
            db_get_or_create_session,
            db_get_recent_sessions,
            db_get_last_session,
            db_upsert_file,
            db_get_files_for_session,
            db_get_file_by_path,
            db_insert_hash,
            db_get_hashes_for_file,
            db_get_latest_hash,
            db_insert_verification,
            db_get_verifications_for_file,
            db_save_open_tabs,
            db_get_open_tabs,
            db_set_setting,
            db_get_setting,
            // Project file operations
            project_get_default_path,
            project_check_exists,
            project_save,
            project_load,
            project_create,
            // File viewer commands
            viewer_read_chunk,
            viewer_detect_type,
            viewer_parse_header,
            viewer_read_text,
            // Report generation commands
            report::commands::generate_report,
            report::commands::preview_report,
            report::commands::get_output_formats,
            report::commands::create_new_report,
            report::commands::validate_report,
            report::commands::add_evidence_to_report,
            report::commands::add_finding_to_report,
            report::commands::export_report_json,
            report::commands::import_report_json,
            report::commands::extract_evidence_from_containers,
            report::commands::create_evidence_from_container,
            report::commands::get_report_template,
            // AI commands
            report::commands::ai_commands::is_ai_available,
            report::commands::ai_commands::get_ai_providers,
            report::commands::ai_commands::generate_ai_narrative,
            report::commands::ai_commands::check_ollama_connection,
            // Processed database commands
            processed::commands::scan_processed_databases,
            processed::commands::get_processed_db_details,
            processed::commands::is_processed_database,
            processed::commands::get_processed_db_summary,
            // AXIOM-specific commands
            processed::commands::get_axiom_case_info,
            processed::commands::get_axiom_artifact_categories,
            processed::commands::query_axiom_artifacts_cmd,
            processed::commands::list_axiom_db_tables
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
