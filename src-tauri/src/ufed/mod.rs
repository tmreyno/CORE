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
//! ## Module Structure
//! ```text
//! ufed/
//! ├── mod.rs          - Entry point, info/info_fast/stats, re-exports
//! ├── types.rs        - UfedFormat, UfedInfo, CaseInfo, DeviceInfo, etc.
//! ├── detection.rs    - Format detection, is_ufed(), device hints
//! ├── parsing.rs      - UFD (INI) and UFDX (XML) parsers
//! ├── collection.rs   - Extraction sets, associated files
//! ├── archive_scan.rs - UFED detection inside ZIP archives
//! ├── vfs.rs          - Virtual filesystem for UFED containers
//! ├── verify.rs       - Verification and hashing operations
//! ├── tree.rs         - Tree browsing and lazy loading
//! ├── extract.rs      - Extraction operations
//! ├── export.rs       - Metadata export (JSON, CSV)
//! └── search.rs       - File search operations
//! ```

pub mod archive_scan;
pub mod collection;
pub mod detection;
pub mod export;
pub mod extract;
pub mod parsing;
pub mod search;
pub mod tree;
pub mod types;
pub mod verify;
pub mod vfs;

use crate::containers::ContainerError;

// Re-exports from types module
pub use types::{
    AssociatedFile, CaseInfo, CollectionInfo, DeviceInfo, ExtractionInfo, StoredHash, UfedFormat,
    UfedInfo, UFED_EXTENSIONS,
};

// Re-exports from detection/archive_scan/vfs modules
pub use archive_scan::detect_in_zip;
pub use detection::{detect_format, find_sibling_ufd, is_ufed, is_ufed_file};
pub use vfs::UfedVfs;

// Re-exports from new child modules (preserves ufed::function_name() API)
pub use export::{export_metadata_csv, export_metadata_json};
pub use extract::{
    extract, extract_with_progress, get_segment_paths, UfedExtractError, UfedExtractResult,
};
pub use search::{search_by_extension, search_by_name, UfedSearchResult};
pub use tree::{get_children, get_entry_count, get_root_children, get_tree, UfedTreeEntry};
pub use verify::{hash_single_segment, verify, verify_file, verify_with_progress};

use std::path::Path;
use tracing::{debug, instrument};

/// Get UFED container information
#[instrument]
pub fn info(path: &str) -> Result<UfedInfo, ContainerError> {
    debug!(path = %path, "Getting UFED info");

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "UFED file not found: {path}"
        )));
    }

    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;

    let metadata =
        std::fs::metadata(path).map_err(|e| format!("Failed to read file metadata: {e}"))?;

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
        return Err(ContainerError::FileNotFound(format!(
            "UFED file not found: {path}"
        )));
    }

    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;

    let metadata =
        std::fs::metadata(path).map_err(|e| format!("Failed to read file metadata: {e}"))?;

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
        associated_files: vec![], // Skip expensive directory scan
        is_extraction_set: false,
        device_hint,
        case_info,
        device_info,
        extraction_info,
        stored_hashes,
        evidence_number,
        collection_info: None, // Skip expensive UFDX parsing
    })
}

// =============================================================================
// Statistics
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

    let associated_files_size: u64 = info.associated_files.iter().map(|f| f.size).sum();

    let stored_hash_count = info.stored_hashes.as_ref().map(|h| h.len()).unwrap_or(0);

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
    fn test_get_stats_nonexistent_file() {
        let result = get_stats("/nonexistent/path/file.ufd");
        assert!(result.is_err());
    }
}
