// =============================================================================
// CORE-FFX - Forensic File Explorer  
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Extraction V2 - Based on libad1
//!
//! File extraction functionality matching libad1_extract.c implementation

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::Serialize;
use tracing::{debug, info, warn};

use super::reader_v2::{ItemHeader, MetadataEntry, SessionV2};
use super::types::*;

/// Extract options
#[derive(Debug, Clone)]
pub struct ExtractOptions {
    /// Output directory
    pub output_dir: PathBuf,
    /// Apply metadata (timestamps, attributes)
    pub apply_metadata: bool,
    /// Verify hashes during extraction
    pub verify_hashes: bool,
    /// Overwrite existing files
    pub overwrite: bool,
    /// Callback for progress updates
    pub progress_callback: Option<fn(usize, usize, &str)>,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            apply_metadata: true,
            verify_hashes: false,
            overwrite: false,
            progress_callback: None,
        }
    }
}

/// Extraction result
#[derive(Debug, Clone, Serialize)]
pub struct ExtractionResult {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_bytes: u64,
    pub failed: Vec<String>,
    pub verified: usize,
    pub verification_failed: Vec<String>,
}

/// Extract all files from AD1 container
///
/// Based on libad1's `extract_all()` function
pub fn extract_all<P: AsRef<Path>>(
    path: P,
    options: ExtractOptions,
) -> Result<ExtractionResult, Ad1Error> {
    let session = SessionV2::open(path)?;
    
    // Create output directory if it doesn't exist
    if !options.output_dir.exists() {
        fs::create_dir_all(&options.output_dir).map_err(|e| {
            Ad1Error::IoError(format!(
                "Failed to create output directory: {}",
                e
            ))
        })?;
    }

    let first_item_addr = session.logical_header.first_item_addr;
    if first_item_addr == 0 {
        return Ok(ExtractionResult {
            total_files: 0,
            total_dirs: 0,
            total_bytes: 0,
            failed: Vec::new(),
            verified: 0,
            verification_failed: Vec::new(),
        });
    }

    let root_item = session.read_item_at(first_item_addr)?;
    
    let mut result = ExtractionResult {
        total_files: 0,
        total_dirs: 0,
        total_bytes: 0,
        failed: Vec::new(),
        verified: 0,
        verification_failed: Vec::new(),
    };

    extract_item_recursive(
        &session,
        &root_item,
        &options.output_dir,
        "",
        &options,
        &mut result,
    )?;

    info!(
        "Extraction complete: {} files, {} dirs, {} bytes",
        result.total_files, result.total_dirs, result.total_bytes
    );

    Ok(result)
}

/// Recursively extract an item and its children
///
/// Based on libad1's `extract_file()` function
fn extract_item_recursive(
    session: &SessionV2,
    item: &ItemHeader,
    base_dir: &Path,
    parent_path: &str,
    options: &ExtractOptions,
    result: &mut ExtractionResult,
) -> Result<(), Ad1Error> {
    // Build item path
    let item_path = if parent_path.is_empty() {
        item.name.clone()
    } else {
        format!("{}/{}", parent_path, item.name)
    };

    // Build complete filesystem path
    let output_path = base_dir.join(&item_path);

    // Progress callback
    if let Some(callback) = options.progress_callback {
        callback(
            result.total_files + result.total_dirs,
            0,
            &item_path,
        );
    }

    let is_dir = item.item_type == 0x05; // AD1_FOLDER_SIGNATURE

    if is_dir {
        // Create directory
        if !output_path.exists() {
            fs::create_dir_all(&output_path).map_err(|e| {
                let err_msg = format!("Failed to create directory {}: {}", output_path.display(), e);
                warn!("{}", err_msg);
                result.failed.push(item_path.clone());
                Ad1Error::IoError(err_msg)
            })?;
            result.total_dirs += 1;
        }
    } else {
        // Extract file
        match extract_single_file(session, item, &output_path, options, result) {
            Ok(_) => {
                result.total_files += 1;
                result.total_bytes += item.decompressed_size;
            }
            Err(e) => {
                warn!("Failed to extract {}: {}", item_path, e);
                result.failed.push(item_path.clone());
            }
        }
    }

    // Apply metadata if requested
    if options.apply_metadata && item.first_metadata_addr != 0 {
        if let Ok(metadata) = session.read_metadata_chain(item.first_metadata_addr) {
            if let Err(e) = apply_metadata(&output_path, &metadata) {
                warn!("Failed to apply metadata to {}: {}", output_path.display(), e);
            }
        }
    }

    // Recurse into children
    if item.first_child_addr != 0 {
        let children = session.read_children_at(item.first_child_addr)?;
        for child in children {
            extract_item_recursive(
                session,
                &child,
                base_dir,
                &item_path,
                options,
                result,
            )?;
        }
    }

    Ok(())
}

/// Extract a single file
fn extract_single_file(
    session: &SessionV2,
    item: &ItemHeader,
    output_path: &Path,
    options: &ExtractOptions,
    result: &mut ExtractionResult,
) -> Result<(), Ad1Error> {
    // Check if file exists and handle overwrite
    if output_path.exists() && !options.overwrite {
        return Err(Ad1Error::IoError(format!(
            "File exists and overwrite disabled: {}",
            output_path.display()
        )));
    }

    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            Ad1Error::IoError(format!("Failed to create parent directory: {}", e))
        })?;
    }

    // Read and decompress file data
    let file_data = if item.decompressed_size > 0 {
        crate::ad1::operations_v2::decompress_file_data(session, item)?
    } else {
        Vec::new()
    };

    // Verify hash if requested
    if options.verify_hashes && item.decompressed_size > 0 {
        if let Ok(metadata) = session.read_metadata_chain(item.first_metadata_addr) {
            // Check MD5 if available
            if let Some(stored_md5) = metadata.iter().find_map(|m| {
                if m.category == 0x01 && m.key == 0x5001 {
                    Some(String::from_utf8_lossy(&m.data).to_string())
                } else {
                    None
                }
            }) {
                let computed_md5 = crate::ad1::hash_v2::md5_hash(&file_data);
                if !stored_md5.eq_ignore_ascii_case(&computed_md5) {
                    warn!(
                        "MD5 mismatch for {}: stored={}, computed={}",
                        item.name, stored_md5, computed_md5
                    );
                    result.verification_failed.push(item.name.clone());
                } else {
                    result.verified += 1;
                }
            }
        }
    }

    // Write file
    let mut file = File::create(output_path).map_err(|e| {
        Ad1Error::IoError(format!("Failed to create file: {}", e))
    })?;

    file.write_all(&file_data).map_err(|e| {
        Ad1Error::IoError(format!("Failed to write file: {}", e))
    })?;

    debug!("Extracted: {} ({} bytes)", output_path.display(), file_data.len());

    Ok(())
}

/// Apply metadata to extracted file/directory
///
/// Based on libad1's `apply_metadata()` function
fn apply_metadata(
    path: &Path,
    metadata: &[MetadataEntry],
) -> Result<(), Ad1Error> {
    use std::time::{SystemTime, Duration};

    let mut accessed_time: Option<SystemTime> = None;
    let mut modified_time: Option<SystemTime> = None;

    for entry in metadata {
        match (entry.category, entry.key) {
            // TIMESTAMP category
            (0x05, 0x07) => {
                // ACCESS timestamp
                if let Some(time) = parse_windows_filetime(&entry.data) {
                    accessed_time = Some(time);
                }
            }
            (0x05, 0x08) => {
                // MODIFIED timestamp
                if let Some(time) = parse_windows_filetime(&entry.data) {
                    modified_time = Some(time);
                }
            }
            _ => {}
        }
    }

    // Set file times if available
    #[cfg(unix)]
    {
        if accessed_time.is_some() || modified_time.is_some() {
            use std::os::unix::fs::MetadataExt;
            use std::time::UNIX_EPOCH;

            let metadata = fs::metadata(path).map_err(|e| {
                Ad1Error::IoError(format!("Failed to read file metadata: {}", e))
            })?;

            let atime = accessed_time
                .or_else(|| {
                    Some(UNIX_EPOCH + Duration::from_secs(metadata.atime() as u64))
                })
                .unwrap();

            let mtime = modified_time
                .or_else(|| {
                    Some(UNIX_EPOCH + Duration::from_secs(metadata.mtime() as u64))
                })
                .unwrap();

            // Use filetime crate for cross-platform support
            use filetime::{FileTime, set_file_times};
            
            let atime_ft = FileTime::from_system_time(atime);
            let mtime_ft = FileTime::from_system_time(mtime);
            
            set_file_times(path, atime_ft, mtime_ft).map_err(|e| {
                Ad1Error::IoError(format!("Failed to set file times: {}", e))
            })?;
        }
    }

    #[cfg(windows)]
    {
        // Windows-specific file time setting
        if accessed_time.is_some() || modified_time.is_some() {
            use filetime::{FileTime, set_file_times};
            
            let atime = accessed_time.unwrap_or_else(|| SystemTime::now());
            let mtime = modified_time.unwrap_or_else(|| SystemTime::now());
            
            let atime_ft = FileTime::from_system_time(atime);
            let mtime_ft = FileTime::from_system_time(mtime);
            
            set_file_times(path, atime_ft, mtime_ft).map_err(|e| {
                Ad1Error::IoError(format!("Failed to set file times: {}", e))
            })?;
        }
    }

    Ok(())
}

/// Parse Windows FILETIME (100-nanosecond intervals since 1601-01-01)
fn parse_windows_filetime(data: &[u8]) -> Option<std::time::SystemTime> {
    use std::time::{Duration, UNIX_EPOCH};
    
    if data.len() < 8 {
        return None;
    }

    let filetime = u64::from_le_bytes([
        data[0], data[1], data[2], data[3],
        data[4], data[5], data[6], data[7],
    ]);

    // Windows FILETIME epoch difference from Unix epoch
    const FILETIME_EPOCH_DIFF: u64 = 116444736000000000;

    if filetime < FILETIME_EPOCH_DIFF {
        return None;
    }

    // Convert to Unix timestamp (seconds since 1970-01-01)
    let unix_time = (filetime - FILETIME_EPOCH_DIFF) / 10000000;
    
    Some(UNIX_EPOCH + Duration::from_secs(unix_time))
}

/// Extract a single item by address (non-recursive)
pub fn extract_item_by_addr<P: AsRef<Path>>(
    container_path: P,
    item_addr: u64,
    output_path: &Path,
) -> Result<(), Ad1Error> {
    let session = SessionV2::open(container_path)?;
    let item = session.read_item_at(item_addr)?;

    if item.item_type == 0x05 {
        return Err(Ad1Error::InvalidFormat("Cannot extract directory item".to_string()));
    }

    let options = ExtractOptions {
        output_dir: output_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        apply_metadata: true,
        verify_hashes: false,
        overwrite: true,
        progress_callback: None,
    };

    let mut result = ExtractionResult {
        total_files: 0,
        total_dirs: 0,
        total_bytes: 0,
        failed: Vec::new(),
        verified: 0,
        verification_failed: Vec::new(),
    };

    extract_single_file(&session, &item, output_path, &options, &mut result)
}
