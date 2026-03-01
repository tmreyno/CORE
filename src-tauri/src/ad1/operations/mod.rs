// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Container Operations
//!
//! Public API for AD1 (AccessData Logical Image) container operations.
//!
//! ## Submodules
//! - `verify` — Verification, hashing, and companion log operations
//! - `tree` — Tree navigation and data reading operations
//! - `search` — Search by name, extension, or hash
//! - `export` — Export (JSON/CSV) and extraction operations

mod export;
mod search;
mod tree;
mod verify;

// Re-export all public APIs
pub use export::{
    export_metadata_csv, export_metadata_json, export_tree_csv, export_tree_json, extract,
    extract_with_progress,
};
pub use search::{find_by_extension, find_by_hash, find_by_name};
pub use tree::{
    get_children, get_children_at_addr, get_children_at_addr_lazy, get_entry_info, get_tree,
    read_entry_chunk, read_entry_data, read_entry_data_by_addr,
};
pub use verify::{
    hash_segments, hash_segments_with_progress, hash_single_segment, verify, verify_against_log,
    verify_chunks, verify_with_progress,
};

use std::fs::File;
use std::io::Read;
use tracing::{debug, instrument, trace};

use super::parser::Session;
use super::types::{
    Ad1Info, Ad1Stats, Item, AD1_FOLDER_SIGNATURE, AD1_SIGNATURE, HASH_INFO, MD5_HASH, SHA1_HASH,
};
use super::utils::*;
use crate::containers::ContainerError;

// =============================================================================
// Container Information
// =============================================================================

/// Fast info - only reads headers, doesn't parse full item tree
/// Use this for quick container detection/display
/// This uses lenient validation - will return info even with missing segments
#[must_use = "this returns the AD1 info, which should be used"]
#[instrument]
pub fn info_fast(path: &str) -> Result<Ad1Info, ContainerError> {
    debug!("Getting fast AD1 info (headers only)");
    validate_ad1(path, false)?; // Only validate format, not segments

    let mut file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file '{path}': {e}")))?;

    let segment_header = read_segment_header(&mut file)?;
    let logical_header = read_logical_header(&mut file)?;

    // Parse volume info from header
    let volume = parse_volume_info(&mut file);

    // Parse companion log file for case metadata
    let companion_log = parse_companion_log(path);

    // Get segment files with sizes (includes missing segments)
    let (segment_files, segment_sizes, total_size, missing_segments) =
        get_segment_files_with_sizes(path, segment_header.segment_number);

    // Get detailed segment summary
    let segment_summary = get_segment_summary(
        path,
        segment_header.segment_number,
        segment_header.fragments_size,
    );

    let missing = if missing_segments.is_empty() {
        None
    } else {
        Some(missing_segments)
    };

    Ok(Ad1Info {
        segment: segment_header_info(&segment_header),
        logical: logical_header_info(&logical_header),
        item_count: 0, // Not parsed in fast mode
        tree: None,
        segment_files: Some(segment_files),
        segment_sizes: Some(segment_sizes),
        total_size: Some(total_size),
        missing_segments: missing,
        segment_summary: Some(segment_summary),
        volume,
        companion_log,
    })
}

/// Get full AD1 container information
/// Note: This still requires all segments to be present (strict validation via Session::open)
#[must_use = "this returns the AD1 info, which should be used"]
#[instrument]
pub fn info(path: &str, include_tree: bool) -> Result<Ad1Info, ContainerError> {
    debug!("Getting AD1 info, include_tree={}", include_tree);
    let session = Session::open(path)?;

    let tree = if include_tree {
        let mut entries = Vec::new();
        collect_tree(&session.root_items, "", &mut entries);
        Some(entries)
    } else {
        None
    };

    // Get segment files with sizes
    let (segment_files, segment_sizes, total_size, missing_segments) =
        get_segment_files_with_sizes(path, session.segment_header.segment_number);

    // Get detailed segment summary
    let segment_summary = get_segment_summary(
        path,
        session.segment_header.segment_number,
        session.segment_header.fragments_size,
    );

    let missing = if missing_segments.is_empty() {
        None
    } else {
        Some(missing_segments)
    };

    // Parse volume info from the first segment file
    let volume = {
        let mut file = File::open(path).map_err(|e| {
            ContainerError::IoError(format!("Failed to open AD1 file for volume info: {e}"))
        })?;
        parse_volume_info(&mut file)
    };

    // Parse companion log file for case metadata
    let companion_log = parse_companion_log(path);

    Ok(Ad1Info {
        segment: segment_header_info(&session.segment_header),
        logical: logical_header_info(&session.logical_header),
        item_count: session.item_counter,
        tree,
        segment_files: Some(segment_files),
        segment_sizes: Some(segment_sizes),
        total_size: Some(total_size),
        missing_segments: missing,
        segment_summary: Some(segment_summary),
        volume,
        companion_log,
    })
}

// =============================================================================
// Format Detection
// =============================================================================

/// Check if file is an AD1 container
#[must_use = "this returns whether the file is AD1, which should be used"]
pub fn is_ad1(path: &str) -> Result<bool, ContainerError> {
    let mut file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open input file: {e}")))?;
    let mut signature = [0u8; 16];
    file.read_exact(&mut signature)
        .map_err(|e| ContainerError::IoError(format!("Failed to read file signature: {e}")))?;
    let is_ad1 = &signature[..15] == AD1_SIGNATURE;
    trace!(path, is_ad1, "AD1 signature check");
    Ok(is_ad1)
}

// =============================================================================
// Segment Path Functions
// =============================================================================

/// Get all segment file paths for an AD1 container
/// Returns paths for .ad1, .ad2, .ad3, etc.
#[must_use = "this returns the segment paths, which should be used"]
pub fn get_segment_paths(path: &str) -> Result<Vec<std::path::PathBuf>, ContainerError> {
    validate_ad1(path, false)?;

    let mut file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file: {e}")))?;
    let segment_header = read_segment_header(&mut file)?;

    let mut paths = Vec::with_capacity(segment_header.segment_number as usize);
    for i in 1..=segment_header.segment_number {
        let segment_path = build_segment_path(path, i);
        paths.push(std::path::PathBuf::from(segment_path));
    }

    Ok(paths)
}

// =============================================================================
// Statistics Functions
// =============================================================================

/// Get container statistics
#[must_use = "this returns the statistics, which should be used"]
pub fn get_stats(path: &str) -> Result<Ad1Stats, ContainerError> {
    let session = Session::open(path)?;
    let mut stats = Ad1Stats::default();

    // Get compressed size from segments
    let segment_paths = get_segment_paths(path)?;
    for seg_path in &segment_paths {
        if let Ok(meta) = std::fs::metadata(seg_path) {
            stats.compressed_size += meta.len();
        }
    }

    // Recursively gather stats from items
    fn gather_stats(items: &[Item], stats: &mut Ad1Stats, depth: u32) {
        stats.max_depth = stats.max_depth.max(depth);

        for item in items {
            stats.total_items += 1;

            if item.item_type == AD1_FOLDER_SIGNATURE {
                stats.total_folders += 1;
            } else {
                stats.total_files += 1;
                stats.total_size += item.decompressed_size;

                // Track largest file
                if item.decompressed_size > stats.largest_file_size {
                    stats.largest_file_size = item.decompressed_size;
                    stats.largest_file_path = Some(item.name.clone());
                }

                // Check for hashes
                let has_md5 = item
                    .metadata
                    .iter()
                    .any(|m| m.category == HASH_INFO && m.key == MD5_HASH);
                let has_sha1 = item
                    .metadata
                    .iter()
                    .any(|m| m.category == HASH_INFO && m.key == SHA1_HASH);
                if has_md5 {
                    stats.files_with_md5 += 1;
                }
                if has_sha1 {
                    stats.files_with_sha1 += 1;
                }
            }

            gather_stats(&item.children, stats, depth + 1);
        }
    }

    gather_stats(&session.root_items, &mut stats, 0);

    // Calculate compression ratio
    if stats.total_size > 0 {
        stats.compression_ratio = stats.compressed_size as f64 / stats.total_size as f64;
    }

    Ok(stats)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Create a minimal AD1 file that passes signature validation
    fn create_test_ad1(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(b"ADSEGMENTEDFILE\0").unwrap();
        file.write_all(&[0u8; 496]).unwrap();
        path
    }

    #[test]
    fn test_is_ad1_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let ad1_path = create_test_ad1(temp_dir.path(), "test.ad1");
        let result = is_ad1(ad1_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_ad1_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("not_ad1.bin");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"NOT AN AD1 FILE!").unwrap();
        let result = is_ad1(path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_ad1_nonexistent_file() {
        let result = is_ad1("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_ad1_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("empty.ad1");
        File::create(&path).unwrap();
        let result = is_ad1(path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_ad1_partial_signature() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("partial.ad1");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"ADSEGMENTED").unwrap();
        let result = is_ad1(path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_info_fast_nonexistent() {
        let result = info_fast("/nonexistent/path/file.ad1");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not found") || err.contains("No such file") || err.contains("Failed")
        );
    }

    #[test]
    fn test_info_nonexistent() {
        let result = info("/nonexistent/path/file.ad1", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_segment_paths_nonexistent() {
        let result = get_segment_paths("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_stats_nonexistent() {
        let result = get_stats("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_ad1_stats_default() {
        let stats = Ad1Stats::default();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_folders, 0);
        assert_eq!(stats.compression_ratio, 0.0);
    }
}
