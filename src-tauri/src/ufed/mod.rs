// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED container support (UFD, UFDR, UFDX)
//!
//! This module provides detection and metadata extraction for UFED
//! (Universal Forensic Extraction Data) containers commonly used in 
//! mobile device forensics.
//!
//! ## Supported Formats
//! - **UFD**: Metadata file (INI format with case/device/hash info)
//! - **UFDR**: Standalone extraction file (often large)
//! - **UFDX**: Collection index/metadata file (XML format)
//!
//! ## Typical Structure
//! ```text
//! 02606-0900_1E_BTPLJM/                     # Evidence number folder
//! └── UFED Apple iPhone SE (A2275) 2024_08_26 (001)/
//!     └── AdvancedLogical File System 01/
//!         ├── Apple_iPhone SE (A2275)/      # Extracted file system
//!         ├── Apple_iPhone SE (A2275).ufd   # UFD metadata (INI format)
//!         ├── Apple_iPhone SE (A2275).zip   # Compressed extraction
//!         └── SummaryReport.pdf             # Report
//! ```
//!
//! ## UFD File Format
//! UFD files are INI-style configuration files containing:
//! - `\[Crime Case\]`: Case identifier, examiner, evidence number
//! - `\[DeviceInfo\]`: IMEI, model, OS version, vendor
//! - `\[General\]`: Acquisition tool, extraction type, timestamps
//! - `\[SHA256\]`: Hash values for extraction files
//!
//! ## Module Structure
//! ```text
//! ufed/
//! ├── mod.rs         - Main entry point, info() function
//! ├── types.rs       - UfedFormat, UfedInfo, CaseInfo, DeviceInfo, etc.
//! ├── detection.rs   - Format detection, is_ufed(), device hints
//! ├── parsing.rs     - UFD (INI) and UFDX (XML) parsers
//! ├── collection.rs  - Extraction sets, associated files
//! └── archive_scan.rs - UFED detection inside ZIP archives
//! ```

pub mod types;
pub mod detection;
pub mod parsing;
pub mod collection;

use crate::containers::ContainerError;
pub mod archive_scan;
pub mod vfs;

// Re-exports for convenience
pub use types::{
    UfedFormat, UfedInfo, CaseInfo, DeviceInfo, ExtractionInfo,
    StoredHash, AssociatedFile, CollectionInfo, UFED_EXTENSIONS,
};
pub use detection::{is_ufed, detect_format, is_ufed_file, find_sibling_ufd};
pub use archive_scan::detect_in_zip;
pub use vfs::UfedVfs;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::{debug, instrument};

use crate::common::hash::{StreamingHasher, HashAlgorithm};

/// Get UFED container information
#[instrument]
pub fn info(path: &str) -> Result<UfedInfo, ContainerError> {
    debug!(path = %path, "Getting UFED info");
    
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!("UFED file not found: {path}")));
    }
    
    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;
    
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Failed to read file metadata: {e}"))?;
    
    let size = metadata.len();
    
    // Get parent folder name (often contains device info)
    let parent_folder = path_obj
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());
    
    // Try to extract device hint from filename or path
    let device_hint = detection::extract_device_hint(path);
    
    // Extract evidence number from folder structure
    let evidence_number = detection::extract_evidence_number(path_obj);
    
    // Parse UFD file contents:
    // - If it's a .ufd file, parse it directly
    // - If it's a UFED ZIP, find and parse the sibling .ufd file
    let (case_info, device_info, extraction_info, stored_hashes) = match format {
        UfedFormat::Ufd => parsing::parse_ufd_file(path)?,
        UfedFormat::UfedZip => {
            // Find sibling UFD and parse it
            if let Some(ufd_path) = detection::find_sibling_ufd(path) {
                if ufd_path.exists() {
                    if let Some(ufd_str) = ufd_path.to_str() {
                        parsing::parse_ufd_file(ufd_str).unwrap_or((None, None, None, None))
                    } else {
                        (None, None, None, None)
                    }
                } else {
                    (None, None, None, None)
                }
            } else {
                (None, None, None, None)
            }
        }
        _ => (None, None, None, None),
    };
    
    // Find associated files in the same directory (with hash info)
    let associated_files = collection::find_associated_files(path_obj, stored_hashes.as_ref());
    
    // Find and parse EvidenceCollection.ufdx in parent directories
    let collection_info = collection::find_collection_ufdx(path_obj);
    
    // Check if this is part of a complete extraction set
    let is_extraction_set = collection::check_extraction_set(&associated_files, format);
    
    debug!(
        path = %path,
        format = %format,
        size = size,
        associated_files = associated_files.len(),
        is_extraction_set = is_extraction_set,
        has_case_info = case_info.is_some(),
        has_device_info = device_info.is_some(),
        has_collection_info = collection_info.is_some(),
        "UFED info loaded"
    );
    
    Ok(UfedInfo {
        format: format.to_string(),
        size,
        parent_folder,
        associated_files,
        is_extraction_set,
        device_hint,
        case_info,
        device_info,
        extraction_info,
        stored_hashes,
        evidence_number,
        collection_info,
    })
}

/// Fast UFED info - only reads basic metadata, skips associated file scanning
/// 
/// This is faster than `info()` because it:
/// - Skips scanning for associated files in the directory
/// - Skips parsing sibling UFD files for ZIP containers  
/// - Skips collection UFDX parsing
/// 
/// Use this for quick container detection/listing.
#[instrument]
pub fn info_fast(path: &str) -> Result<UfedInfo, ContainerError> {
    debug!(path = %path, "Getting fast UFED info");
    
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!("UFED file not found: {path}")));
    }
    
    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;
    
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Failed to read file metadata: {e}"))?;
    
    let size = metadata.len();
    
    // Get parent folder name (often contains device info) - cheap operation
    let parent_folder = path_obj
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());
    
    // Try to extract device hint from filename or path - cheap string operation
    let device_hint = detection::extract_device_hint(path);
    
    // Extract evidence number from folder structure - cheap path parsing
    let evidence_number = detection::extract_evidence_number(path_obj);
    
    // Only parse UFD file if it's actually a UFD file (not for ZIPs)
    let (case_info, device_info, extraction_info, stored_hashes) = match format {
        UfedFormat::Ufd => parsing::parse_ufd_file(path).unwrap_or((None, None, None, None)),
        _ => (None, None, None, None),
    };
    
    debug!(
        path = %path,
        format = %format,
        size = size,
        "Fast UFED info loaded"
    );
    
    Ok(UfedInfo {
        format: format.to_string(),
        size,
        parent_folder,
        associated_files: vec![],  // Skip expensive directory scan
        is_extraction_set: false,
        device_hint,
        case_info,
        device_info,
        extraction_info,
        stored_hashes,
        evidence_number,
        collection_info: None,  // Skip expensive UFDX parsing
    })
}

// =============================================================================
// Verification Functions
// =============================================================================

/// Verify a UFED file by computing its hash
#[instrument]
pub fn verify(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    verify_with_progress(path, algorithm, |_, _| {})
}

/// Verify a UFED file with progress callback
#[instrument(skip(progress_callback))]
pub fn verify_with_progress<F>(path: &str, algorithm: &str, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    debug!(path = %path, algorithm = %algorithm, "Verifying UFED file");
    
    let file = File::open(path)
        .map_err(|e| format!("Failed to open file: {e}"))?;
    
    let total_size = file.metadata()
        .map_err(|e| format!("Failed to get file size: {e}"))?
        .len();
    
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let algo = algorithm.parse::<HashAlgorithm>()
        .map_err(|e| format!("Unsupported algorithm: {e}"))?;
    let mut hasher = StreamingHasher::new(algo);
    
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut bytes_read = 0u64;
    
    loop {
        let n = reader.read(&mut buffer)
            .map_err(|e| format!("Read error: {e}"))?;
        
        if n == 0 {
            break;
        }
        
        hasher.update(&buffer[..n]);
        bytes_read += n as u64;
        progress_callback(bytes_read, total_size);
    }
    
    Ok(hasher.finalize())
}

/// Verify a specific file referenced in UFED metadata
/// 
/// This computes the hash of a file and returns it for comparison
/// with stored hashes in the UFD metadata.
#[instrument]
pub fn verify_file(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    verify(path, algorithm)
}

// =============================================================================
// Statistics Functions
// =============================================================================

/// UFED container statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedStats {
    /// Total size of the UFED file
    pub file_size: u64,
    /// Number of associated files found
    pub associated_file_count: usize,
    /// Total size of associated files
    pub associated_files_size: u64,
    /// Number of stored hashes
    pub stored_hash_count: usize,
    /// Whether case info is available
    pub has_case_info: bool,
    /// Whether device info is available
    pub has_device_info: bool,
    /// Whether extraction info is available
    pub has_extraction_info: bool,
    /// Whether collection info is available
    pub has_collection_info: bool,
    /// Format type
    pub format: String,
}

/// Get UFED container statistics
#[instrument]
pub fn get_stats(path: &str) -> Result<UfedStats, ContainerError> {
    debug!(path = %path, "Getting UFED stats");
    
    let info = info(path)?;
    
    let associated_files_size: u64 = info.associated_files
        .iter()
        .map(|f| f.size)
        .sum();
    
    let stored_hash_count = info.stored_hashes
        .as_ref()
        .map(|h| h.len())
        .unwrap_or(0);
    
    Ok(UfedStats {
        file_size: info.size,
        associated_file_count: info.associated_files.len(),
        associated_files_size,
        stored_hash_count,
        has_case_info: info.case_info.is_some(),
        has_device_info: info.device_info.is_some(),
        has_extraction_info: info.extraction_info.is_some(),
        has_collection_info: info.collection_info.is_some(),
        format: info.format,
    })
}

// =============================================================================
// Extraction Functions
// =============================================================================

/// Result from UFED extraction operations
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedExtractResult {
    /// Number of files extracted
    pub files_extracted: u64,
    /// Number of directories created
    pub directories_created: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// List of extraction errors
    pub errors: Vec<UfedExtractError>,
    /// Whether the extraction was successful
    pub success: bool,
}

/// Individual extraction error
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedExtractError {
    /// Path that failed to extract
    pub path: String,
    /// Error message
    pub error: String,
}

/// Extract UFED container contents to output directory
/// 
/// For ZIP containers, extracts the archive contents.
/// For UFD files, copies the file and any associated files.
/// For UFDR containers, copies the data to the output directory.
#[instrument]
pub fn extract(path: &str, output_dir: &str) -> Result<UfedExtractResult, ContainerError> {
    extract_with_progress(path, output_dir, |_, _| {})
}

/// Extract UFED container with progress callback
#[instrument(skip(progress_callback))]
pub fn extract_with_progress<F>(
    path: &str,
    output_dir: &str,
    mut progress_callback: F,
) -> Result<UfedExtractResult, ContainerError>
where
    F: FnMut(u64, u64),
{
    debug!(path = %path, output_dir = %output_dir, "Extracting UFED container");
    
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!("UFED file not found: {path}")));
    }
    
    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;
    
    let output_path = Path::new(output_dir);
    std::fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {e}"))?;
    
    match format {
        UfedFormat::UfedZip => {
            // Extract ZIP using archive module
            let result = crate::archive::extract(path, output_dir)?;
            Ok(UfedExtractResult {
                files_extracted: result.files_extracted,
                directories_created: result.directories_created,
                bytes_written: result.bytes_written,
                errors: result.failed_files.into_iter().map(|e| UfedExtractError {
                    path: e.path,
                    error: e.error,
                }).collect(),
                success: result.success,
            })
        }
        UfedFormat::Ufd | UfedFormat::Ufdr | UfedFormat::Ufdx => {
            // Copy the file and associated files
            let info = info(path)?;
            let total_size = info.size + info.associated_files.iter().map(|f| f.size).sum::<u64>();
            let mut bytes_written = 0u64;
            let mut files_extracted = 0u64;
            let mut errors = Vec::new();
            
            // Copy main file
            let main_name = path_obj.file_name()
                .ok_or_else(|| "Invalid file path".to_string())?;
            let dest_main = output_path.join(main_name);
            
            if let Err(e) = std::fs::copy(path_obj, &dest_main) {
                errors.push(UfedExtractError {
                    path: path.to_string(),
                    error: e.to_string(),
                });
            } else {
                bytes_written += info.size;
                files_extracted += 1;
                progress_callback(bytes_written, total_size);
            }
            
            // Copy associated files - construct full path from parent dir + filename
            let parent_dir = path_obj.parent().unwrap_or(path_obj);
            for assoc in &info.associated_files {
                let assoc_path = parent_dir.join(&assoc.filename);
                let dest = output_path.join(&assoc.filename);
                if let Err(e) = std::fs::copy(&assoc_path, &dest) {
                    errors.push(UfedExtractError {
                        path: assoc.filename.clone(),
                        error: e.to_string(),
                    });
                } else {
                    bytes_written += assoc.size;
                    files_extracted += 1;
                    progress_callback(bytes_written, total_size);
                }
            }
            
            Ok(UfedExtractResult {
                files_extracted,
                directories_created: 0,
                bytes_written,
                errors: errors.clone(),
                success: errors.is_empty(),
            })
        }
    }
}

// =============================================================================
// Segment Functions
// =============================================================================

/// Get paths of all segments in a UFED extraction set
/// 
/// For UFED containers, segments are the associated files that make up
/// the complete extraction set.
#[instrument]
pub fn get_segment_paths(path: &str) -> Result<Vec<std::path::PathBuf>, ContainerError> {
    debug!(path = %path, "Getting UFED segment paths");
    
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!("UFED file not found: {path}")));
    }
    
    let info = info(path)?;
    let mut segments = vec![path_obj.to_path_buf()];
    
    // Add associated files as "segments" - construct full path from parent dir + filename
    let parent_dir = path_obj.parent().unwrap_or(path_obj);
    for assoc in info.associated_files {
        segments.push(parent_dir.join(&assoc.filename));
    }
    
    Ok(segments)
}

// =============================================================================
// Tree Browsing Functions
// =============================================================================

/// UFED tree entry representing a file or directory in the extraction
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedTreeEntry {
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
    /// Associated hash from UFD file if available
    pub hash: Option<String>,
    /// Modified timestamp if available
    pub modified: Option<String>,
}

/// Get the file tree for a UFED container
/// 
/// For ZIP containers, lists the entries in the archive.
/// For UFD/UFDR/UFDX, lists the associated files.
#[instrument]
pub fn get_tree(path: &str) -> Result<Vec<UfedTreeEntry>, ContainerError> {
    debug!(path = %path, "Getting UFED tree");
    
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!("UFED file not found: {path}")));
    }
    
    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;
    
    match format {
        UfedFormat::UfedZip => {
            // List ZIP entries
            let entries = crate::archive::list_zip_entries(path)?;
            Ok(entries.into_iter().map(|e| UfedTreeEntry {
                path: e.path.clone(),
                name: Path::new(&e.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| e.path.clone()),
                is_dir: e.is_directory,
                size: e.size,
                entry_type: if e.is_directory { "folder".to_string() } else { "file".to_string() },
                hash: None,
                modified: Some(e.last_modified),
            }).collect())
        }
        _ => {
            // For UFD/UFDR/UFDX, return the file itself and associated files
            let info = info(path)?;
            let mut entries = Vec::new();
            
            // Add main file
            let name = path_obj.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            entries.push(UfedTreeEntry {
                path: path.to_string(),
                name,
                is_dir: false,
                size: info.size,
                entry_type: format.to_string(),
                hash: info.stored_hashes.as_ref()
                    .and_then(|h| h.first())
                    .map(|h| h.hash.clone()),
                modified: None,
            });
            
            // Add associated files
            let parent_dir = path_obj.parent().unwrap_or(path_obj);
            for assoc in info.associated_files {
                let assoc_full_path = parent_dir.join(&assoc.filename);
                entries.push(UfedTreeEntry {
                    path: assoc_full_path.to_string_lossy().to_string(),
                    name: assoc.filename,
                    is_dir: false,
                    size: assoc.size,
                    entry_type: assoc.file_type,
                    hash: assoc.stored_hash,
                    modified: None,
                });
            }
            
            Ok(entries)
        }
    }
}

/// Get children of a directory in a UFED container
/// 
/// For ZIP containers, lists entries at the specified path.
/// For other formats, returns associated files if parent_path is empty.
#[instrument]
pub fn get_children(path: &str, parent_path: &str) -> Result<Vec<UfedTreeEntry>, ContainerError> {
    debug!(path = %path, parent_path = %parent_path, "Getting UFED children");
    
    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;
    
    if let UfedFormat::UfedZip = format {
        // List ZIP entries under the parent path
        let entries = crate::archive::list_zip_entries(path)?;
        let normalized_parent = if parent_path.is_empty() || parent_path == "/" {
            "".to_string()
        } else {
            let p = parent_path.trim_start_matches('/').trim_end_matches('/');
            format!("{}/", p)
        };
        
        let children: Vec<UfedTreeEntry> = entries.into_iter()
            .filter(|e| {
                let entry_path = e.path.trim_start_matches('/');
                if normalized_parent.is_empty() {
                    // Root level: no slashes, or one trailing slash for dirs
                    !entry_path.contains('/') || 
                    (entry_path.ends_with('/') && entry_path.matches('/').count() == 1)
                } else {
                    // Direct children of parent
                    if entry_path.starts_with(&normalized_parent) {
                        let relative = &entry_path[normalized_parent.len()..];
                        !relative.is_empty() && 
                        (!relative.contains('/') || 
                         (relative.ends_with('/') && relative.matches('/').count() == 1))
                    } else {
                        false
                    }
                }
            })
            .map(|e| UfedTreeEntry {
                path: e.path.clone(),
                name: Path::new(&e.path.trim_end_matches('/'))
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| e.path.clone()),
                is_dir: e.is_directory,
                size: e.size,
                entry_type: if e.is_directory { "folder".to_string() } else { "file".to_string() },
                hash: None,
                modified: Some(e.last_modified),
            })
            .collect();
        
        Ok(children)
    } else {
        // For non-ZIP formats, return all at root level
        get_tree(path)
    }
}

// =============================================================================
// Export Metadata Functions
// =============================================================================

/// Export UFED container metadata as JSON
#[instrument]
pub fn export_metadata_json(path: &str) -> Result<String, ContainerError> {
    debug!(path = %path, "Exporting UFED metadata as JSON");
    
    let info = info(path)?;
    let stats = get_stats(path)?;
    
    #[derive(serde::Serialize)]
    struct UfedMetadata {
        format: String,
        file_size: u64,
        file_size_formatted: String,
        device_hint: Option<String>,
        statistics: UfedStatistics,
        case_info: Option<CaseInfoSummary>,
        device_info: Option<DeviceInfoSummary>,
        extraction_info: Option<ExtractionInfoSummary>,
        associated_files: Vec<AssociatedFileSummary>,
        stored_hashes: Vec<StoredHashSummary>,
    }
    
    #[derive(serde::Serialize)]
    struct UfedStatistics {
        associated_file_count: usize,
        associated_files_size: u64,
        stored_hash_count: usize,
        has_case_info: bool,
        has_device_info: bool,
        has_extraction_info: bool,
        has_collection_info: bool,
    }
    
    #[derive(serde::Serialize)]
    struct CaseInfoSummary {
        case_identifier: Option<String>,
        examiner_name: Option<String>,
        department: Option<String>,
        crime_type: Option<String>,
    }
    
    #[derive(serde::Serialize)]
    struct DeviceInfoSummary {
        full_name: Option<String>,
        model: Option<String>,
        os_version: Option<String>,
        vendor: Option<String>,
    }
    
    #[derive(serde::Serialize)]
    struct ExtractionInfoSummary {
        extraction_type: Option<String>,
        acquisition_tool: Option<String>,
        start_time: Option<String>,
        end_time: Option<String>,
    }
    
    #[derive(serde::Serialize)]
    struct AssociatedFileSummary {
        filename: String,
        file_type: String,
        size: u64,
        size_formatted: String,
        stored_hash: Option<String>,
    }
    
    #[derive(serde::Serialize)]
    struct StoredHashSummary {
        filename: String,
        hash: String,
        algorithm: String,
    }
    
    let metadata = UfedMetadata {
        format: info.format.clone(),
        file_size: info.size,
        file_size_formatted: crate::common::format_size(info.size),
        device_hint: info.device_hint.clone(),
        statistics: UfedStatistics {
            associated_file_count: stats.associated_file_count,
            associated_files_size: stats.associated_files_size,
            stored_hash_count: stats.stored_hash_count,
            has_case_info: stats.has_case_info,
            has_device_info: stats.has_device_info,
            has_extraction_info: stats.has_extraction_info,
            has_collection_info: stats.has_collection_info,
        },
        case_info: info.case_info.as_ref().map(|c| CaseInfoSummary {
            case_identifier: c.case_identifier.clone(),
            examiner_name: c.examiner_name.clone(),
            department: c.department.clone(),
            crime_type: c.crime_type.clone(),
        }),
        device_info: info.device_info.as_ref().map(|d| DeviceInfoSummary {
            full_name: d.full_name.clone(),
            model: d.model.clone(),
            os_version: d.os_version.clone(),
            vendor: d.vendor.clone(),
        }),
        extraction_info: info.extraction_info.as_ref().map(|e| ExtractionInfoSummary {
            extraction_type: e.extraction_type.clone(),
            acquisition_tool: e.acquisition_tool.clone(),
            start_time: e.start_time.clone(),
            end_time: e.end_time.clone(),
        }),
        associated_files: info.associated_files.iter().map(|f| AssociatedFileSummary {
            filename: f.filename.clone(),
            file_type: f.file_type.clone(),
            size: f.size,
            size_formatted: crate::common::format_size(f.size),
            stored_hash: f.stored_hash.clone(),
        }).collect(),
        stored_hashes: info.stored_hashes.as_ref().map(|hashes| {
            hashes.iter().map(|h| StoredHashSummary {
                filename: h.filename.clone(),
                hash: h.hash.clone(),
                algorithm: h.algorithm.clone(),
            }).collect()
        }).unwrap_or_default(),
    };
    
    serde_json::to_string_pretty(&metadata)
        .map_err(|e| ContainerError::SerializationError(format!("Failed to serialize UFED metadata to JSON: {e}")))
}

/// Export UFED container metadata as CSV
#[instrument]
pub fn export_metadata_csv(path: &str) -> Result<String, ContainerError> {
    debug!(path = %path, "Exporting UFED metadata as CSV");
    
    let info = info(path)?;
    let stats = get_stats(path)?;
    
    let mut csv = String::new();
    
    // Header section
    csv.push_str("# UFED Container Metadata\n");
    csv.push_str(&format!("Format,{}\n", info.format));
    csv.push_str(&format!("File Size,{}\n", info.size));
    csv.push_str(&format!("File Size (Formatted),\"{}\"\n", crate::common::format_size(info.size)));
    if let Some(ref hint) = info.device_hint {
        csv.push_str(&format!("Device Hint,\"{}\"\n", escape_csv(hint)));
    }
    csv.push('\n');
    
    // Statistics
    csv.push_str("# Statistics\n");
    csv.push_str(&format!("Associated File Count,{}\n", stats.associated_file_count));
    csv.push_str(&format!("Associated Files Size,{}\n", stats.associated_files_size));
    csv.push_str(&format!("Stored Hash Count,{}\n", stats.stored_hash_count));
    csv.push_str(&format!("Has Case Info,{}\n", stats.has_case_info));
    csv.push_str(&format!("Has Device Info,{}\n", stats.has_device_info));
    csv.push_str(&format!("Has Extraction Info,{}\n", stats.has_extraction_info));
    csv.push_str(&format!("Has Collection Info,{}\n", stats.has_collection_info));
    csv.push('\n');
    
    // Case info
    if let Some(ref case_info) = info.case_info {
        csv.push_str("# Case Information\n");
        if let Some(ref ci) = case_info.case_identifier {
            csv.push_str(&format!("Case Identifier,\"{}\"\n", escape_csv(ci)));
        }
        if let Some(ref examiner) = case_info.examiner_name {
            csv.push_str(&format!("Examiner Name,\"{}\"\n", escape_csv(examiner)));
        }
        if let Some(ref dept) = case_info.department {
            csv.push_str(&format!("Department,\"{}\"\n", escape_csv(dept)));
        }
        if let Some(ref crime) = case_info.crime_type {
            csv.push_str(&format!("Crime Type,\"{}\"\n", escape_csv(crime)));
        }
        csv.push('\n');
    }
    
    // Device info
    if let Some(ref device_info) = info.device_info {
        csv.push_str("# Device Information\n");
        if let Some(ref name) = device_info.full_name {
            csv.push_str(&format!("Full Name,\"{}\"\n", escape_csv(name)));
        }
        if let Some(ref model) = device_info.model {
            csv.push_str(&format!("Model,\"{}\"\n", escape_csv(model)));
        }
        if let Some(ref vendor) = device_info.vendor {
            csv.push_str(&format!("Vendor,\"{}\"\n", escape_csv(vendor)));
        }
        if let Some(ref os) = device_info.os_version {
            csv.push_str(&format!("OS Version,\"{}\"\n", escape_csv(os)));
        }
        csv.push('\n');
    }
    
    // Associated files
    csv.push_str("# Associated Files\n");
    csv.push_str("Filename,File Type,Size,Size (Formatted),Hash\n");
    for file in &info.associated_files {
        csv.push_str(&format!(
            "\"{}\",\"{}\",{},\"{}\",{}\n",
            escape_csv(&file.filename),
            escape_csv(&file.file_type),
            file.size,
            crate::common::format_size(file.size),
            file.stored_hash.as_deref().unwrap_or("")
        ));
    }
    csv.push('\n');
    
    // Stored hashes
    if let Some(ref hashes) = info.stored_hashes {
        csv.push_str("# Stored Hashes\n");
        csv.push_str("Filename,Hash,Algorithm\n");
        for hash in hashes {
            csv.push_str(&format!(
                "\"{}\",{},{}\n",
                escape_csv(&hash.filename),
                hash.hash,
                hash.algorithm
            ));
        }
    }
    
    Ok(csv)
}

/// Escape a value for CSV output
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

// =============================================================================
// Search Functions
// =============================================================================

/// Search result from UFED container
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedSearchResult {
    /// The matching entry
    pub entry: UfedTreeEntry,
    /// How the match was found (name, extension, etc.)
    pub match_type: String,
    /// Depth in the tree
    pub depth: u32,
}

/// Search for files by name pattern in a UFED container
#[instrument]
pub fn search_by_name(path: &str, pattern: &str) -> Result<Vec<UfedSearchResult>, ContainerError> {
    debug!(path = %path, pattern = %pattern, "Searching UFED by name");
    
    let tree = get_tree(path)?;
    let pattern_lower = pattern.to_lowercase();
    
    let results: Vec<UfedSearchResult> = tree.into_iter()
        .filter(|entry| entry.name.to_lowercase().contains(&pattern_lower))
        .map(|entry| {
            let depth = entry.path.matches('/').count() as u32;
            UfedSearchResult {
                entry,
                match_type: "name".to_string(),
                depth,
            }
        })
        .collect();
    
    Ok(results)
}

/// Search for files by extension in a UFED container
#[instrument]
pub fn search_by_extension(path: &str, extension: &str) -> Result<Vec<UfedSearchResult>, ContainerError> {
    debug!(path = %path, extension = %extension, "Searching UFED by extension");
    
    let tree = get_tree(path)?;
    let ext_lower = extension.to_lowercase().trim_start_matches('.').to_string();
    
    let results: Vec<UfedSearchResult> = tree.into_iter()
        .filter(|entry| {
            if entry.is_dir {
                return false;
            }
            if let Some(file_ext) = entry.name.rsplit('.').next() {
                file_ext.to_lowercase() == ext_lower
            } else {
                false
            }
        })
        .map(|entry| {
            let depth = entry.path.matches('/').count() as u32;
            UfedSearchResult {
                entry,
                match_type: "extension".to_string(),
                depth,
            }
        })
        .collect();
    
    Ok(results)
}

/// Hash a single file in a UFED container set
#[instrument(skip(progress_callback))]
pub fn hash_single_segment<F>(segment_path: &str, algorithm: &str, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    debug!(segment_path = %segment_path, algorithm = %algorithm, "Hashing single UFED segment");
    
    let path = Path::new(segment_path);
    if !path.exists() {
        return Err(ContainerError::FileNotFound(format!("Segment file not found: {}", segment_path)));
    }
    
    let algo: crate::common::hash::HashAlgorithm = algorithm.parse()?;
    let file_size = std::fs::metadata(path)
        .map_err(|e| format!("Failed to get file size: {e}"))?
        .len();
    
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open file: {e}"))?;
    let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
    
    let mut hasher = crate::common::hash::StreamingHasher::new(algo);
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut bytes_processed: u64 = 0;
    
    loop {
        use std::io::Read;
        let bytes_read = reader.read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        bytes_processed += bytes_read as u64;
        progress_callback(bytes_processed, file_size);
    }
    
    Ok(hasher.finalize())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ufed_stats_struct() {
        let stats = UfedStats {
            file_size: 1000,
            associated_file_count: 5,
            associated_files_size: 50000,
            stored_hash_count: 3,
            has_case_info: true,
            has_device_info: true,
            has_extraction_info: false,
            has_collection_info: false,
            format: "UFD".to_string(),
        };
        
        assert_eq!(stats.file_size, 1000);
        assert_eq!(stats.associated_file_count, 5);
        assert!(stats.has_case_info);
    }
    
    #[test]
    fn test_verify_nonexistent_file() {
        let result = verify("/nonexistent/path/file.ufd", "sha256");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_stats_nonexistent_file() {
        let result = get_stats("/nonexistent/path/file.ufd");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_extract_nonexistent_file() {
        let result = extract("/nonexistent/path/file.ufd", "/tmp/output");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_segment_paths_nonexistent() {
        let result = get_segment_paths("/nonexistent/path/file.ufd");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_tree_nonexistent() {
        let result = get_tree("/nonexistent/path/file.ufd");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_children_nonexistent() {
        let result = get_children("/nonexistent/path/file.ufd", "");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_ufed_tree_entry() {
        let entry = UfedTreeEntry {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
            size: 1024,
            entry_type: "file".to_string(),
            hash: Some("abc123".to_string()),
            modified: Some("2024-01-01".to_string()),
        };
        
        assert_eq!(entry.name, "file.txt");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 1024);
    }
    
    #[test]
    fn test_ufed_extract_result() {
        let result = UfedExtractResult {
            files_extracted: 5,
            directories_created: 2,
            bytes_written: 10000,
            errors: vec![],
            success: true,
        };
        
        assert_eq!(result.files_extracted, 5);
        assert!(result.success);
    }
}