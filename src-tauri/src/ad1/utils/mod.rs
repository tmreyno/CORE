// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Utility Functions
//!
//! ## Section Brief
//! Helper functions for AD1 container parsing and data extraction:
//!
//! ### Segment Management
//! - `get_segment_files_with_sizes()` - Discover segment files with sizes
//! - `build_segment_path()` - Build path for segment N (delegates to common)
//! - `segment_span()` - Calculate segment data span
//!
//! ### Path Utilities
//! - `join_path()` - Join path components for container paths
//!
//! ### String Utilities
//! - `bytes_to_string()` - Convert bytes to string (stops at null terminator)
//!
//! ### Header Info Conversion
//! - `segment_header_info()` - Convert internal to public header info
//! - `logical_header_info()` - Convert internal to public header info
//!
//! ### File Metadata
//! - `apply_metadata()` - Apply metadata timestamps to extracted file
//!
//! ### Sub-modules
//! - `parsing` - AD1 format parsing utilities (headers, validation, hashes, timestamps)
//! - `tree_builder` - Tree building logic (tree entry construction, collection, search)

pub mod parsing;
pub mod tree_builder;

// Re-export everything from sub-modules for backwards compatibility
pub use parsing::*;
pub use tree_builder::*;

use filetime::FileTime;
use std::path::Path;
use std::time::SystemTime;

use crate::ad1::types::*;
use crate::common::segments::{build_ad1_segment_path, discover_ad1_segments};
use crate::containers::ContainerError;

// =============================================================================
// Segment Management
// =============================================================================

/// Get segment files with their sizes and track missing segments
/// Returns (segment_names, segment_sizes, total_size, missing_segments)
///
/// Uses shared segment discovery from common/segments.rs
pub fn get_segment_files_with_sizes(
    path: &str,
    segment_count: u32,
) -> (Vec<String>, Vec<u64>, u64, Vec<String>) {
    let (paths, sizes, missing) = discover_ad1_segments(path, segment_count);

    let segment_names: Vec<String> = paths
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .collect();

    let total_size: u64 = sizes.iter().sum();

    (segment_names, sizes, total_size, missing)
}

/// Get detailed segment summary with offset ranges
pub fn get_segment_summary(path: &str, segment_count: u32, fragments_size: u32) -> SegmentSummary {
    let path_obj = Path::new(path);
    let parent = path_obj.parent();
    let stem = path_obj
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let seg_span = segment_span(fragments_size);
    let mut segments = Vec::with_capacity(segment_count as usize);
    let mut total_size = 0u64;
    let mut total_data_size = 0u64;
    let mut found_count = 0u32;
    let mut missing_count = 0u32;

    for i in 1..=segment_count {
        let segment_name = format!("{}.ad{}", stem, i);
        let segment_path = if let Some(parent_dir) = parent {
            parent_dir.join(&segment_name).to_string_lossy().to_string()
        } else {
            segment_name.clone()
        };

        let seg_path = Path::new(&segment_path);
        let exists = seg_path.exists();
        let size = if exists {
            std::fs::metadata(&segment_path)
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        let data_size = size.saturating_sub(AD1_LOGICAL_MARGIN);
        let offset_start = (i as u64 - 1) * seg_span;
        let offset_end = offset_start + data_size;

        if exists {
            found_count += 1;
            total_size += size;
            total_data_size += data_size;
        } else {
            missing_count += 1;
        }

        segments.push(SegmentFileInfo {
            number: i,
            path: segment_path,
            filename: segment_name,
            size,
            exists,
            data_size,
            offset_start,
            offset_end,
        });
    }

    SegmentSummary {
        expected_count: segment_count,
        found_count,
        missing_count,
        total_size,
        total_data_size,
        segments,
        is_complete: missing_count == 0,
    }
}

/// Build segment file path from base path and segment index
/// Wrapper around shared function for backwards compatibility
pub fn build_segment_path(base: &str, index: u32) -> String {
    build_ad1_segment_path(base, index)
}

/// Calculate segment span from fragments size
pub fn segment_span(fragments_size: u32) -> u64 {
    (fragments_size as u64 * SEGMENT_BLOCK_SIZE).saturating_sub(AD1_LOGICAL_MARGIN)
}

// =============================================================================
// String Utilities
// =============================================================================

/// Convert bytes to string (stops at null terminator)
///
/// # Arguments
/// * `bytes` - Raw byte slice to convert
/// * `trim` - If true, trim leading/trailing whitespace from result
///
/// # Returns
/// UTF-8 string up to first null terminator, optionally trimmed
pub fn bytes_to_string(bytes: &[u8], trim: bool) -> String {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    let s = String::from_utf8_lossy(&bytes[..end]);
    if trim {
        s.trim().to_string()
    } else {
        s.to_string()
    }
}

// =============================================================================
// Path Utilities
// =============================================================================

/// Join path components
pub fn join_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else if name.is_empty() {
        parent.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

// =============================================================================
// Header Info Conversion
// =============================================================================

/// Convert SegmentHeader to public SegmentHeaderInfo
pub fn segment_header_info(header: &SegmentHeader) -> SegmentHeaderInfo {
    SegmentHeaderInfo {
        signature: bytes_to_string(&header.signature, false),
        segment_index: header.segment_index,
        segment_number: header.segment_number,
        fragments_size: header.fragments_size,
        header_size: header.header_size,
    }
}

/// Convert LogicalHeader to public LogicalHeaderInfo  
pub fn logical_header_info(header: &LogicalHeader) -> LogicalHeaderInfo {
    LogicalHeaderInfo {
        signature: bytes_to_string(&header.signature, false),
        image_version: header.image_version,
        zlib_chunk_size: header.zlib_chunk_size,
        logical_metadata_addr: header.logical_metadata_addr,
        first_item_addr: header.first_item_addr,
        data_source_name_length: header.data_source_name_length,
        ad_signature: bytes_to_string(&header.ad_signature, false),
        data_source_name_addr: header.data_source_name_addr,
        attrguid_footer_addr: header.attrguid_footer_addr,
        locsguid_footer_addr: header.locsguid_footer_addr,
        data_source_name: header.data_source_name.clone(),
    }
}

// =============================================================================
// File Metadata Application
// =============================================================================

/// Apply metadata timestamps to extracted file
pub fn apply_metadata(path: &Path, metadata: &[Metadata]) -> Result<(), ContainerError> {
    let mut access_time = None;
    let mut modified_time = None;

    for meta in metadata {
        if meta.category != TIMESTAMP {
            continue;
        }
        let value = bytes_to_string(&meta.data, true);
        match meta.key {
            ACCESS => access_time = parse_timestamp(&value),
            MODIFIED => modified_time = parse_timestamp(&value),
            _ => {}
        }
    }

    if access_time.is_none() && modified_time.is_none() {
        return Ok(());
    }

    let now = FileTime::from_system_time(SystemTime::now());
    let atime = access_time.unwrap_or(now);
    let mtime = modified_time.unwrap_or(atime);
    filetime::set_file_times(path, atime, mtime)
        .map_err(|e| format!("Failed to set file times for {:?}: {e}", path))?;
    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Normalize path separators for cross-platform test assertions
    fn norm(s: &str) -> String {
        s.replace('\\', "/")
    }

    #[test]
    fn test_build_segment_path() {
        assert_eq!(
            norm(&build_segment_path("/path/to/file.ad1", 1)),
            "/path/to/file.ad1"
        );
        assert_eq!(
            norm(&build_segment_path("/path/to/file.ad1", 2)),
            "/path/to/file.ad2"
        );
        assert_eq!(
            norm(&build_segment_path("/path/to/file.ad1", 3)),
            "/path/to/file.ad3"
        );
        assert_eq!(
            norm(&build_segment_path("/path/to/file.ad1", 10)),
            "/path/to/file.ad10"
        );
        assert_eq!(build_segment_path("", 1), "");
    }

    #[test]
    fn test_join_path() {
        assert_eq!(join_path("", "file.txt"), "file.txt");
        assert_eq!(join_path("folder", ""), "folder");
        assert_eq!(join_path("folder", "file.txt"), "folder/file.txt");
        assert_eq!(join_path("a/b", "c.txt"), "a/b/c.txt");
    }

    #[test]
    fn test_segment_span() {
        assert_eq!(
            segment_span(0x10000),
            SEGMENT_BLOCK_SIZE * 0x10000 - AD1_LOGICAL_MARGIN
        );
        assert_eq!(segment_span(1), SEGMENT_BLOCK_SIZE - AD1_LOGICAL_MARGIN);
        assert_eq!(segment_span(0), 0);
    }

    #[test]
    fn test_segment_span_calculation() {
        // Test segment span with typical fragment sizes
        // 0x10000 fragments = 65536 * 65536 - 512 = 4294901248
        assert_eq!(
            segment_span(0x10000),
            SEGMENT_BLOCK_SIZE * 0x10000 - AD1_LOGICAL_MARGIN
        );

        // Single fragment
        assert_eq!(segment_span(1), SEGMENT_BLOCK_SIZE - AD1_LOGICAL_MARGIN);

        // 100 fragments
        assert_eq!(
            segment_span(100),
            100 * SEGMENT_BLOCK_SIZE - AD1_LOGICAL_MARGIN
        );

        // Zero fragments
        assert_eq!(segment_span(0), 0);
    }

    #[test]
    fn test_bytes_to_string() {
        // Normal string without trim
        assert_eq!(bytes_to_string(b"hello", false), "hello");

        // Null terminated without trim
        assert_eq!(bytes_to_string(b"hello\0world", false), "hello");

        // Empty
        assert_eq!(bytes_to_string(b"", false), "");

        // All nulls
        assert_eq!(bytes_to_string(&[0, 0, 0], false), "");
    }

    #[test]
    fn test_bytes_to_string_with_trim() {
        // Normal string with trim
        assert_eq!(bytes_to_string(b"value", true), "value");

        // With whitespace (trimmed)
        assert_eq!(bytes_to_string(b"  value  ", true), "value");

        // Null terminated with whitespace
        assert_eq!(bytes_to_string(b"value\0extra", true), "value");

        // Without trim - preserves whitespace
        assert_eq!(bytes_to_string(b"  value  ", false), "  value  ");
    }
}
