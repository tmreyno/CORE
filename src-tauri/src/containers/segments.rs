// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Segment handling utilities for multi-part forensic containers
//!
//! This module provides functions for detecting and handling segmented container files
//! like .E01/.E02, .ad1/.ad2, .001/.002, and various archive split formats.

use std::path::Path;
use tracing::debug;

/// Check if filename is a numbered segment (.001, .002, etc.)
pub fn is_numbered_segment(lower: &str) -> bool {
    if let Some(ext_start) = lower.rfind('.') {
        let ext = &lower[ext_start + 1..];
        if ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

/// Get the path to the first available segment given any segment path
/// Tries .001 first, then scans for the lowest numbered segment
/// NOTE: This does file I/O to check existence - use get_first_segment_path_fast for scanning
#[allow(dead_code)]
pub fn get_first_segment_path(path: &str) -> String {
    let path_obj = Path::new(path);
    if let Some(parent) = path_obj.parent() {
        if let Some(filename) = path_obj.file_name() {
            let filename_str = filename.to_string_lossy();
            if let Some(dot_pos) = filename_str.rfind('.') {
                let base = &filename_str[..dot_pos];

                // Try .001 first (most common)
                let first_seg = format!("{}.001", base);
                let first_path = parent.join(&first_seg);
                if first_path.exists() {
                    return first_path.to_string_lossy().to_string();
                }

                // If .001 doesn't exist, find the lowest numbered segment
                for num in 2..=999 {
                    let seg_name = format!("{}.{:03}", base, num);
                    let seg_path = parent.join(&seg_name);
                    if seg_path.exists() {
                        return seg_path.to_string_lossy().to_string();
                    }
                }
            }
        }
    }
    // Return original path if we can't find any lower segment
    path.to_string()
}

/// Fast version - just constructs .001 path without checking existence
/// Used during directory scan to avoid slow file I/O
pub fn get_first_segment_path_fast(path: &str) -> String {
    let path_obj = Path::new(path);
    if let Some(parent) = path_obj.parent() {
        if let Some(filename) = path_obj.file_name() {
            let filename_str = filename.to_string_lossy();
            if let Some(dot_pos) = filename_str.rfind('.') {
                let base = &filename_str[..dot_pos];
                let first_seg = format!("{}.001", base);
                return parent.join(&first_seg).to_string_lossy().to_string();
            }
        }
    }
    path.to_string()
}

/// Check if file is part of a segmented series
#[allow(dead_code)]
pub fn is_segmented_file(lower: &str) -> bool {
    lower.ends_with(".e01")
        || lower.ends_with(".e02")
        || lower.ends_with(".ex01")
        || is_numbered_segment(lower)
        || is_ad1_segment(lower)
}

/// Check if filename is an AD1 segment (.ad1, .ad2, .ad3, etc.)
pub fn is_ad1_segment(lower: &str) -> bool {
    if lower.len() < 4 {
        return false;
    }
    if let Some(dot_pos) = lower.rfind('.') {
        let ext = &lower[dot_pos + 1..];
        if ext.starts_with("ad") && ext.len() >= 3 {
            let num_part = &ext[2..];
            return num_part.chars().all(|c| c.is_ascii_digit()) && !num_part.is_empty();
        }
    }
    false
}

/// Check if this is the first segment of a multi-segment file
pub fn is_first_segment(lower: &str) -> bool {
    // AD1 files: .ad1 is first
    if lower.ends_with(".ad1") {
        return true;
    }
    // AD1 segments but not first: .ad2, .ad3, etc.
    if is_ad1_segment(lower) && !lower.ends_with(".ad1") {
        return false;
    }

    // E01 files: .E01 is first
    if lower.ends_with(".e01") {
        return true;
    }
    // E01 segments but not first
    if lower.ends_with(".e02") {
        return false;
    }
    for i in 3..=99 {
        if lower.ends_with(&format!(".e{:02}", i)) {
            return false;
        }
    }

    // Archive formats - first segments
    // 7z: .7z or .7z.001 is first
    if lower.ends_with(".7z") {
        return true;
    }
    if lower.ends_with(".7z.001") {
        return true;
    }
    // .7z.002, .7z.003, etc. are not first
    if is_7z_continuation(lower) {
        return false;
    }

    // ZIP: .zip or .zip.001 or .z01 is first
    if lower.ends_with(".zip") {
        return true;
    }
    if lower.ends_with(".zip.001") {
        return true;
    }
    if lower.ends_with(".z01") {
        return true;
    }
    // .zip.002, .z02, etc. are not first
    if is_zip_continuation(lower) {
        return false;
    }

    // RAR: .rar or .r00 is first
    if lower.ends_with(".rar") {
        return true;
    }
    if lower.ends_with(".r00") {
        return true;
    }
    // .r01, .r02, etc. are not first
    if is_rar_continuation(lower) {
        return false;
    }

    // TAR archives
    if lower.ends_with(".tar") || lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        return true;
    }

    // GZIP
    if lower.ends_with(".gz") {
        return true;
    }

    // Numbered segments: .001 is first (for non-archive files)
    if let Some(dot_pos) = lower.rfind('.') {
        let ext = &lower[dot_pos + 1..];
        if ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
            return ext == "001";
        }
    }

    // Not a segment file, treat as first
    true
}

/// Check if this is a 7z continuation segment (.7z.002, .7z.003, etc.)
pub fn is_7z_continuation(lower: &str) -> bool {
    if let Some(pos) = lower.rfind(".7z.") {
        let suffix = &lower[pos + 4..];
        if suffix.chars().all(|c| c.is_ascii_digit()) && suffix != "001" {
            return true;
        }
    }
    false
}

/// Check if this is a ZIP continuation segment (.zip.002, .z02, etc.)
pub fn is_zip_continuation(lower: &str) -> bool {
    // .zip.002, .zip.003, etc.
    if let Some(pos) = lower.rfind(".zip.") {
        let suffix = &lower[pos + 5..];
        if suffix.chars().all(|c| c.is_ascii_digit()) && suffix != "001" {
            return true;
        }
    }
    // .z02, .z03, etc.
    if lower.len() >= 4 {
        let ext = &lower[lower.len() - 4..];
        if ext.starts_with(".z") && ext[2..].chars().all(|c| c.is_ascii_digit()) && ext != ".z01" {
            return true;
        }
    }
    false
}

/// Check if this is a RAR continuation segment (.r01, .r02, etc.)
pub fn is_rar_continuation(lower: &str) -> bool {
    if lower.len() >= 4 {
        let ext = &lower[lower.len() - 4..];
        if ext.starts_with(".r") && ext[2..].chars().all(|c| c.is_ascii_digit()) && ext != ".r00" {
            return true;
        }
    }
    false
}

/// Check if this is an archive segment (any type)
pub fn is_archive_segment(lower: &str) -> bool {
    is_7z_continuation(lower)
        || is_zip_continuation(lower)
        || is_rar_continuation(lower)
        || lower.ends_with(".7z.001")
        || lower.ends_with(".zip.001")
}

/// Get the base name without segment number for grouping
pub fn get_segment_basename(filename: &str) -> String {
    let lower = filename.to_lowercase();

    // Handle .E01, .E02, etc.
    if lower.ends_with(".e01") {
        return filename[..filename.len() - 4].to_string();
    }

    // Handle .ad1, .ad2, .ad3, etc.
    if is_ad1_segment(&lower) {
        if let Some(dot_pos) = filename.rfind('.') {
            return filename[..dot_pos].to_string();
        }
    }

    // Handle .7z.001, .7z.002, etc.
    if let Some(pos) = lower.rfind(".7z.") {
        return filename[..pos + 3].to_string(); // Keep .7z
    }

    // Handle .zip.001, .zip.002, etc.
    if let Some(pos) = lower.rfind(".zip.") {
        return filename[..pos + 4].to_string(); // Keep .zip
    }

    // Handle .z01, .z02, etc. (ZIP split)
    if lower.len() >= 4 {
        let ext = &lower[lower.len() - 4..];
        if ext.starts_with(".z") && ext[2..].chars().all(|c| c.is_ascii_digit()) {
            return filename[..filename.len() - 4].to_string();
        }
    }

    // Handle .r00, .r01, etc. (RAR segments)
    if lower.len() >= 4 {
        let ext = &lower[lower.len() - 4..];
        if ext.starts_with(".r") && ext[2..].chars().all(|c| c.is_ascii_digit()) {
            return filename[..filename.len() - 4].to_string();
        }
    }

    // Handle .001, .002, etc.
    if let Some(dot_pos) = filename.rfind('.') {
        let ext = &filename[dot_pos + 1..];
        if ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
            return filename[..dot_pos].to_string();
        }
    }

    filename.to_string()
}

/// Detailed segment information
#[allow(dead_code)]
pub struct SegmentInfo {
    pub total_size: u64,
    pub count: u32,
    pub files: Vec<String>,
    pub sizes: Vec<u64>,
}

/// Calculate total size of all segments in a series
#[allow(dead_code)]
pub fn calculate_total_segment_info(dir: &Path, basename: &str) -> Option<SegmentInfo> {
    let mut total = 0u64;
    let mut count = 0u32;
    let mut files = Vec::new();
    let mut sizes = Vec::new();

    // Try AD1 segments first (.ad1, .ad2, .ad3, ...)
    for i in 1..=100 {
        let segment_name = format!("{}.ad{}", basename, i);
        let segment_path = dir.join(&segment_name);
        if let Ok(metadata) = segment_path.metadata() {
            let size = metadata.len();
            total += size;
            count += 1;
            files.push(segment_name);
            sizes.push(size);
        } else {
            break;
        }
    }

    if total > 0 {
        debug!("Found {} AD1 segments, total size: {}", count, total);
        return Some(SegmentInfo {
            total_size: total,
            count,
            files,
            sizes,
        });
    }

    // Try E01 segments (.E01, .E02, ...)
    for segment_num in 1..=100 {
        let segment_name = if segment_num == 1 {
            format!("{}.E01", basename)
        } else {
            format!("{}.E{:02}", basename, segment_num)
        };

        let segment_path = dir.join(&segment_name);
        if let Ok(metadata) = segment_path.metadata() {
            let size = metadata.len();
            total += size;
            count += 1;
            files.push(segment_name);
            sizes.push(size);
        } else {
            break;
        }
    }

    if total > 0 {
        debug!("Found {} E01 segments, total size: {}", count, total);
        return Some(SegmentInfo {
            total_size: total,
            count,
            files,
            sizes,
        });
    }

    // Try numbered segments (.001, .002, ...)
    for segment_num in 1..=999 {
        let segment_name = format!("{}.{:03}", basename, segment_num);
        let segment_path = dir.join(&segment_name);
        if let Ok(metadata) = segment_path.metadata() {
            let size = metadata.len();
            total += size;
            count += 1;
            files.push(segment_name);
            sizes.push(size);
        } else {
            break;
        }
    }

    if total > 0 {
        debug!("Found {} numbered segments, total size: {}", count, total);
        return Some(SegmentInfo {
            total_size: total,
            count,
            files,
            sizes,
        });
    }

    None
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_numbered_segment() {
        assert!(is_numbered_segment("image.001"));
        assert!(is_numbered_segment("test.042"));
        assert!(is_numbered_segment("file.999"));

        assert!(!is_numbered_segment("image.E01")); // Not all digits
        assert!(!is_numbered_segment("image.01")); // Too short
        assert!(!is_numbered_segment("image.0001")); // Too long
        assert!(!is_numbered_segment("image")); // No extension
    }

    #[test]
    fn test_is_ad1_segment() {
        assert!(is_ad1_segment("image.ad1"));
        assert!(is_ad1_segment("test.ad2"));
        assert!(is_ad1_segment("file.ad99"));

        assert!(!is_ad1_segment("image.ad")); // No number
        assert!(!is_ad1_segment("image.e01")); // Not AD1
        assert!(!is_ad1_segment("image.ad1x")); // Not just digits
        assert!(!is_ad1_segment("x.a")); // Too short
    }

    #[test]
    fn test_is_first_segment() {
        // AD1 first segments
        assert!(is_first_segment("image.ad1"));

        // E01 first segments
        assert!(is_first_segment("disk.e01"));

        // Numbered first segments
        assert!(is_first_segment("image.001"));

        // Non-first segments
        assert!(!is_first_segment("image.ad2"));
        assert!(!is_first_segment("disk.e02"));
        assert!(!is_first_segment("image.002"));
    }

    #[test]
    fn test_get_first_segment_path_fast() {
        // Should return .001 path
        let result = get_first_segment_path_fast("/test/image.005");
        assert!(result.ends_with("image.001"));

        // Should preserve directory structure
        assert!(result.starts_with("/test/"));
    }

    #[test]
    fn test_is_7z_continuation() {
        assert!(is_7z_continuation("archive.7z.002"));
        assert!(is_7z_continuation("archive.7z.003"));
        assert!(is_7z_continuation("test.7z.099"));

        assert!(!is_7z_continuation("archive.7z.001")); // First segment
        assert!(!is_7z_continuation("archive.7z")); // Main file
        assert!(!is_7z_continuation("archive.zip.002")); // Wrong format
    }

    #[test]
    fn test_is_zip_continuation() {
        assert!(is_zip_continuation("archive.zip.002"));
        assert!(is_zip_continuation("test.z02"));
        assert!(is_zip_continuation("file.z99"));

        assert!(!is_zip_continuation("archive.zip.001")); // First segment
        assert!(!is_zip_continuation("archive.z01")); // First segment
        assert!(!is_zip_continuation("archive.zip")); // Main file
    }

    #[test]
    fn test_is_rar_continuation() {
        assert!(is_rar_continuation("archive.r01"));
        assert!(is_rar_continuation("test.r02"));
        assert!(is_rar_continuation("file.r99"));

        assert!(!is_rar_continuation("archive.r00")); // First segment
        assert!(!is_rar_continuation("archive.rar")); // Main file
    }

    #[test]
    fn test_is_archive_segment() {
        assert!(is_archive_segment("archive.7z.002"));
        assert!(is_archive_segment("archive.zip.002"));
        assert!(is_archive_segment("archive.r01"));
        assert!(is_archive_segment("archive.7z.001"));
        assert!(is_archive_segment("archive.zip.001"));

        assert!(!is_archive_segment("image.001")); // Numbered, not archive
        assert!(!is_archive_segment("disk.e01")); // E01, not archive
    }

    #[test]
    fn test_get_segment_basename() {
        // E01 segments
        assert_eq!(get_segment_basename("disk.E01"), "disk");

        // AD1 segments
        assert_eq!(get_segment_basename("image.ad1"), "image");
        assert_eq!(get_segment_basename("test.ad5"), "test");

        // Numbered segments
        assert_eq!(get_segment_basename("raw.001"), "raw");
        assert_eq!(get_segment_basename("file.005"), "file");

        // 7z split
        assert_eq!(get_segment_basename("archive.7z.001"), "archive.7z");

        // ZIP split
        assert_eq!(get_segment_basename("archive.zip.002"), "archive.zip");

        // Non-segmented file
        assert_eq!(get_segment_basename("normal.txt"), "normal.txt");
    }

    #[test]
    fn test_segment_info_struct() {
        let info = SegmentInfo {
            total_size: 1024 * 1024 * 100, // 100 MB
            count: 5,
            files: vec!["image.001".to_string(), "image.002".to_string()],
            sizes: vec![50 * 1024 * 1024, 50 * 1024 * 1024],
        };

        assert_eq!(info.count, 5);
        assert_eq!(info.total_size, 104857600);
        assert_eq!(info.files.len(), 2);
    }

    #[test]
    fn test_is_segmented_file() {
        assert!(is_segmented_file("disk.e01"));
        assert!(is_segmented_file("disk.e02"));
        assert!(is_segmented_file("image.001"));
        assert!(is_segmented_file("file.ad1"));

        assert!(!is_segmented_file("document.txt"));
        assert!(!is_segmented_file("archive.zip"));
    }
}
