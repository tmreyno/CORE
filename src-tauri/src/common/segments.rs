// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Shared segment discovery utilities for forensic container parsers
//
// Handles multi-segment forensic images in various formats:
// - Numbered segments: .001, .002, .003, etc.
// - E01 segments: .E01, .E02, ..., .E99, then .Ex00, .Ex01, etc.
// - AD1 segments: .ad1, .ad2, .ad3, etc.

use std::path::{Path, PathBuf};
use std::fs;
use tracing::{debug, trace};

use crate::containers::ContainerError;

// =============================================================================
// Numbered Segment Discovery (.001, .002, etc.)
// =============================================================================

/// Discover numbered segments (.001, .002, etc.) starting from any segment
/// 
/// Returns (paths, sizes) sorted by segment number
pub fn discover_numbered_segments(path: &str) -> Result<(Vec<PathBuf>, Vec<u64>), String> {
    debug!(path, "Discovering numbered segments");
    let path_obj = Path::new(path);
    let parent = path_obj.parent().unwrap_or(Path::new("."));
    let filename = path_obj.file_name()
        .ok_or("Invalid filename")?
        .to_string_lossy();

    let lower = filename.to_lowercase();
    
    // Check if this is a numbered segment (.001, .002, etc.)
    if let Some(ext_start) = lower.rfind('.') {
        let ext = &lower[ext_start + 1..];
        if ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
            // Multi-segment numbered format
            let base = &filename[..filename.len() - 4]; // Remove .XXX
            trace!(base, "Detected numbered segment format");
            return discover_numbered_segments_by_base(parent, base);
        }
    }

    // Single file or other format - just use the one file
    let size = fs::metadata(path)
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();
    
    debug!(path, size, "Single file (non-segmented)");
    Ok((vec![path_obj.to_path_buf()], vec![size]))
}

/// Discover numbered segments by base name
fn discover_numbered_segments_by_base(dir: &Path, base: &str) -> Result<(Vec<PathBuf>, Vec<u64>), String> {
    trace!(?dir, base, "Discovering segments by base name");
    // First try direct path construction
    let result = discover_numbered_segments_direct(dir, base);
    if let Ok((segs, _)) = &result {
        if !segs.is_empty() {
            debug!(segment_count = segs.len(), "Found segments via direct path");
            return result;
        }
    }
    
    // Fall back to directory scan for case-insensitive matching
    trace!("Falling back to directory scan for case-insensitive matching");
    discover_numbered_segments_scan(dir, base)
}

/// Try to find segments by constructing paths directly
fn discover_numbered_segments_direct(dir: &Path, base: &str) -> Result<(Vec<PathBuf>, Vec<u64>), String> {
    let mut segments = Vec::new();
    let mut sizes = Vec::new();
    let mut found_any = false;
    let mut consecutive_missing = 0;

    for num in 1..=999 {
        let segment_name = format!("{}.{:03}", base, num);
        let segment_path = dir.join(&segment_name);
        
        // Try original case first
        if segment_path.exists() {
            let size = fs::metadata(&segment_path)
                .map_err(|e| format!("Failed to get segment size: {}", e))?
                .len();
            segments.push(segment_path);
            sizes.push(size);
            found_any = true;
            consecutive_missing = 0;
            continue;
        }
        
        // Try lowercase
        let segment_name_lower = segment_name.to_lowercase();
        let segment_path_lower = dir.join(&segment_name_lower);
        if segment_path_lower.exists() {
            let size = fs::metadata(&segment_path_lower)
                .map_err(|e| format!("Failed to get segment size: {}", e))?
                .len();
            segments.push(segment_path_lower);
            sizes.push(size);
            found_any = true;
            consecutive_missing = 0;
            continue;
        }
        
        // Try uppercase
        let segment_name_upper = segment_name.to_uppercase();
        let segment_path_upper = dir.join(&segment_name_upper);
        if segment_path_upper.exists() {
            let size = fs::metadata(&segment_path_upper)
                .map_err(|e| format!("Failed to get segment size: {}", e))?
                .len();
            segments.push(segment_path_upper);
            sizes.push(size);
            found_any = true;
            consecutive_missing = 0;
            continue;
        }
        
        // Segment not found
        consecutive_missing += 1;
        
        // If we've found segments before and now have a gap, stop
        if found_any {
            break; // End of sequence
        } else if consecutive_missing > 10 {
            // Haven't found any and checked first 10 numbers - give up
            break;
        }
    }

    Ok((segments, sizes))
}

/// Scan directory to find segments with case-insensitive matching
fn discover_numbered_segments_scan(dir: &Path, base: &str) -> Result<(Vec<PathBuf>, Vec<u64>), String> {
    let base_lower = base.to_lowercase();
    let mut found_segments: Vec<(u32, PathBuf, u64)> = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            let filename_lower = filename.to_lowercase();
            
            // Check if filename matches our pattern (base.XXX where XXX is numeric)
            if let Some(dot_pos) = filename_lower.rfind('.') {
                let file_base = &filename_lower[..dot_pos];
                let ext = &filename_lower[dot_pos + 1..];
                
                if file_base == base_lower && ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
                    if let Ok(num) = ext.parse::<u32>() {
                        if let Ok(meta) = entry.metadata() {
                            found_segments.push((num, entry.path(), meta.len()));
                        }
                    }
                }
            }
        }
    }
    
    if found_segments.is_empty() {
        return Err("No segments found".to_string());
    }
    
    // Sort by segment number
    found_segments.sort_by_key(|(num, _, _)| *num);
    
    let segments: Vec<PathBuf> = found_segments.iter().map(|(_, p, _)| p.clone()).collect();
    let sizes: Vec<u64> = found_segments.iter().map(|(_, _, s)| *s).collect();
    
    Ok((segments, sizes))
}

// =============================================================================
// AD1 Segment Discovery (.ad1, .ad2, .ad3, etc.)
// =============================================================================

/// Check if filename is an AD1 segment (.ad1, .ad2, etc.)
pub fn is_ad1_segment(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    if let Some(ext_start) = lower.rfind('.') {
        let ext = &lower[ext_start + 1..];
        if ext.starts_with("ad") && ext.len() >= 3 {
            let num_part = &ext[2..];
            return !num_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit());
        }
    }
    false
}

/// Check if this is the first AD1 segment (.ad1)
pub fn is_first_ad1_segment(filename: &str) -> bool {
    filename.to_lowercase().ends_with(".ad1")
}

/// Extract segment number from AD1 filename (.ad1 -> 1, .ad2 -> 2, etc.)
pub fn extract_ad1_segment_number(filename: &str) -> Option<u32> {
    let lower = filename.to_lowercase();
    if let Some(ext_start) = lower.rfind('.') {
        let ext = &lower[ext_start + 1..];
        if ext.starts_with("ad") && ext.len() >= 3 {
            return ext[2..].parse::<u32>().ok();
        }
    }
    None
}

/// Build the path for an AD1 segment given base path and segment number
pub fn build_ad1_segment_path(base_path: &str, segment_num: u32) -> String {
    // Handle empty path case
    if base_path.is_empty() {
        return String::new();
    }
    
    let path = Path::new(base_path);
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    
    parent.join(format!("{}.ad{}", stem, segment_num))
        .to_string_lossy()
        .to_string()
}

/// Discover AD1 segments starting from the first segment
/// Returns (paths, sizes, missing_segments)
pub fn discover_ad1_segments(base_path: &str, expected_count: u32) -> (Vec<PathBuf>, Vec<u64>, Vec<String>) {
    debug!(base_path, expected_count, "Discovering AD1 segments");
    let path = Path::new(base_path);
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let mut paths = Vec::with_capacity(expected_count as usize);
    let mut sizes = Vec::with_capacity(expected_count as usize);
    let mut missing = Vec::new();
    
    for i in 1..=expected_count {
        let segment_name = format!("{}.ad{}", stem, i);
        let segment_path = parent.join(&segment_name);
        
        if segment_path.exists() {
            let size = fs::metadata(&segment_path)
                .map(|m| m.len())
                .unwrap_or(0);
            trace!(segment = i, size, "Found AD1 segment");
            paths.push(segment_path);
            sizes.push(size);
        } else {
            // Try lowercase
            let segment_name_lower = segment_name.to_lowercase();
            let segment_path_lower = parent.join(&segment_name_lower);
            if segment_path_lower.exists() {
                let size = fs::metadata(&segment_path_lower)
                    .map(|m| m.len())
                    .unwrap_or(0);
                trace!(segment = i, size, "Found AD1 segment (lowercase)");
                paths.push(segment_path_lower);
                sizes.push(size);
            } else {
                debug!(segment = i, segment_name, "Missing AD1 segment");
                missing.push(segment_name);
            }
        }
    }
    
    debug!(
        found = paths.len(),
        missing = missing.len(),
        "AD1 segment discovery complete"
    );
    (paths, sizes, missing)
}

// =============================================================================
// E01 Segment Discovery (.E01, .E02, ..., .Ex01, etc.)
// =============================================================================


/// Discover E01 segments (.E01, .E02, ..., .E99, .Ex00, .Ex01, etc.)
pub fn discover_e01_segments(base_path: &str) -> Result<Vec<PathBuf>, ContainerError> {
    debug!(base_path, "Discovering E01 segments");
    let path = Path::new(base_path);
    let parent = path.parent().ok_or("Invalid path")?;
    let stem = path.file_stem().ok_or("No filename")?.to_string_lossy();
    
    let mut paths = vec![path.to_path_buf()];
    
    for i in 2..=999 {
        let segment_name = if i <= 99 {
            format!("{}.E{:02}", stem, i)
        } else {
            // After E99 comes Ex00, Ex01, etc.
            format!("{}.Ex{:02}", stem, i - 99)
        };
        
        let segment_path = parent.join(&segment_name);
        if segment_path.exists() {
            trace!(segment = i, ?segment_path, "Found E01 segment");
            paths.push(segment_path);
        } else {
            // Also try lowercase
            let segment_name_lower = segment_name.to_lowercase();
            let segment_path_lower = parent.join(&segment_name_lower);
            if segment_path_lower.exists() {
                trace!(segment = i, ?segment_path_lower, "Found E01 segment (lowercase)");
                paths.push(segment_path_lower);
            } else {
                break;
            }
        }
    }
    
    debug!(segment_count = paths.len(), "E01 segments discovered");
    Ok(paths)
}

// =============================================================================
// L01 Segment Discovery (.L01, .L02, ..., .L99, .LAA, ..., .LZZ)
// =============================================================================

/// Generate the expected L01 segment extension for a given segment number.
///
/// Follows the same scheme as the L01 writer:
/// - Segments 1–99: `.L01` ... `.L99`
/// - Segments 100+: `.LAA`, `.LAB`, ... `.LZZ` (676 additional)
fn l01_segment_extension(segment_number: u16) -> String {
    if segment_number <= 99 {
        format!("L{:02}", segment_number.max(1))
    } else {
        let idx = (segment_number - 100) as u32;
        let first = (b'A' + (idx / 26) as u8) as char;
        let second = (b'A' + (idx % 26) as u8) as char;
        format!("L{}{}", first, second)
    }
}

/// Discover L01 segments (.L01, .L02, ..., .L99, .LAA, ..., .LZZ)
///
/// Starting from the base `.L01` path, scans for consecutive segment files.
/// Maximum 775 segments (99 numeric + 676 letter pairs).
pub fn discover_l01_segments(base_path: &str) -> Result<Vec<PathBuf>, ContainerError> {
    debug!(base_path, "Discovering L01 segments");
    let path = Path::new(base_path);
    let parent = path.parent().ok_or("Invalid path")?;
    let stem = path.file_stem().ok_or("No filename")?.to_string_lossy();

    let mut paths = vec![path.to_path_buf()];

    // Max segments: 99 numeric + 676 letter pairs = 775
    for i in 2..=775u16 {
        let ext = l01_segment_extension(i);
        let segment_name = format!("{}.{}", stem, ext);

        let segment_path = parent.join(&segment_name);
        if segment_path.exists() {
            trace!(segment = i, ?segment_path, "Found L01 segment");
            paths.push(segment_path);
        } else {
            // Also try lowercase
            let segment_name_lower = format!("{}.{}", stem, ext.to_lowercase());
            let segment_path_lower = parent.join(&segment_name_lower);
            if segment_path_lower.exists() {
                trace!(segment = i, ?segment_path_lower, "Found L01 segment (lowercase)");
                paths.push(segment_path_lower);
            } else {
                break;
            }
        }
    }

    debug!(segment_count = paths.len(), "L01 segments discovered");
    Ok(paths)
}

/// Check if a filename is an L01 segment file (L01, L02, ..., L99, LAA, ..., LZZ)
pub fn is_l01_segment(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    if let Some(dot_pos) = lower.rfind('.') {
        let ext = &lower[dot_pos + 1..];
        if ext.len() == 3 && ext.starts_with('l') {
            let rest = &ext[1..];
            // L01–L99 (numeric)
            if rest.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
            // LAA–LZZ (letter pairs)
            if rest.len() == 2 && rest.chars().all(|c| c.is_ascii_lowercase()) {
                return true;
            }
        }
    }
    false
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Check if filename is a numbered segment (.001, .002, etc.)
pub fn is_numbered_segment(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    if let Some(ext_start) = lower.rfind('.') {
        let ext = &lower[ext_start + 1..];
        if ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

/// Check if file is part of a segmented series (E01, L01, AD1, or numbered)
pub fn is_segmented_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".e01") 
        || lower.ends_with(".e02") 
        || lower.ends_with(".ex01") 
        || is_numbered_segment(filename)
        || is_ad1_segment(filename)
        || is_l01_segment(filename)
}

/// Get the base name without segment number for grouping
/// Example: "image.001" -> "image", "image.E01" -> "image"
pub fn get_segment_basename(filename: &str) -> String {
    let lower = filename.to_lowercase();
    
    // Handle .E01, .E02, etc.
    if lower.ends_with(".e01") {
        return filename[..filename.len() - 4].to_string();
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

/// Get the path to the first available segment given any segment path
/// Tries .001 first, then scans for the lowest numbered segment
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

/// Calculate total size of all segments in a series
pub fn calculate_total_segment_size(dir: &Path, basename: &str) -> Option<(u64, Option<u32>)> {
    let mut total = 0u64;
    let mut count = 0u32;
    
    // Try common segment patterns
    let patterns = [
        format!("{}.E01", basename),
        format!("{}.001", basename),
    ];
    
    for pattern in &patterns {
        let mut segment_num = 1;
        // Limit to 100 segments max to prevent infinite loops
        while segment_num <= 100 {
            let segment_name = if pattern.contains(".E01") {
                if segment_num == 1 {
                    format!("{}.E01", basename)
                } else {
                    format!("{}.E{:02}", basename, segment_num)
                }
            } else {
                format!("{}.{:03}", basename, segment_num)
            };
            
            let segment_path = dir.join(&segment_name);
            if let Ok(metadata) = segment_path.metadata() {
                total += metadata.len();
                count += 1;
                segment_num += 1;
            } else {
                // Stop at first missing segment
                break;
            }
        }
        
        if total > 0 {
            return Some((total, if count > 1 { Some(count) } else { None }));
        }
    }
    
    None
}

/// Extract segment number from segment name (e.g., "SCHARDT.001" -> 1)
pub fn extract_segment_number(name: &str) -> Option<u32> {
    // Try to find numeric extension
    if let Some(dot_pos) = name.rfind('.') {
        let ext = &name[dot_pos + 1..];
        if let Ok(num) = ext.parse::<u32>() {
            return Some(num);
        }
        // Also handle E01 format
        let ext_lower = ext.to_lowercase();
        if ext_lower.starts_with('e') && ext_lower.len() >= 2 {
            if let Ok(num) = ext_lower[1..].parse::<u32>() {
                return Some(num);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_numbered_segment() {
        assert!(is_numbered_segment("image.001"));
        assert!(is_numbered_segment("image.002"));
        assert!(is_numbered_segment("IMAGE.001"));
        assert!(!is_numbered_segment("image.dd"));
        assert!(!is_numbered_segment("image.e01"));
        assert!(!is_numbered_segment("image.01")); // Only 2 digits
    }

    #[test]
    fn test_is_ad1_segment() {
        assert!(is_ad1_segment("evidence.ad1"));
        assert!(is_ad1_segment("evidence.ad2"));
        assert!(is_ad1_segment("evidence.AD1"));
        assert!(is_ad1_segment("evidence.ad10"));
        assert!(!is_ad1_segment("evidence.add")); // Not numeric
        assert!(!is_ad1_segment("evidence.a1"));  // Missing 'd'
        assert!(!is_ad1_segment("evidence.txt"));
    }

    #[test]
    fn test_extract_ad1_segment_number() {
        assert_eq!(extract_ad1_segment_number("evidence.ad1"), Some(1));
        assert_eq!(extract_ad1_segment_number("evidence.ad2"), Some(2));
        assert_eq!(extract_ad1_segment_number("evidence.ad10"), Some(10));
        assert_eq!(extract_ad1_segment_number("evidence.AD1"), Some(1));
        assert_eq!(extract_ad1_segment_number("evidence.txt"), None);
    }

    #[test]
    fn test_get_segment_basename() {
        assert_eq!(get_segment_basename("image.001"), "image");
        assert_eq!(get_segment_basename("image.E01"), "image");
        assert_eq!(get_segment_basename("image.dd"), "image.dd");
        assert_eq!(get_segment_basename("SCHARDT.001"), "SCHARDT");
    }

    #[test]
    fn test_is_segmented_file() {
        assert!(is_segmented_file("image.001"));
        assert!(is_segmented_file("image.E01"));
        assert!(is_segmented_file("image.e01"));
        assert!(is_segmented_file("evidence.ad1"));
        assert!(is_segmented_file("evidence.ad2"));
        assert!(is_segmented_file("evidence.L01"));
        assert!(is_segmented_file("evidence.l01"));
        assert!(is_segmented_file("evidence.L02"));
        assert!(is_segmented_file("evidence.LAA"));
        assert!(!is_segmented_file("image.dd"));
    }

    #[test]
    fn test_extract_segment_number() {
        assert_eq!(extract_segment_number("SCHARDT.001"), Some(1));
        assert_eq!(extract_segment_number("image.E01"), Some(1));
        assert_eq!(extract_segment_number("image.E99"), Some(99));
        assert_eq!(extract_segment_number("image.dd"), None);
    }

    #[test]
    fn test_is_l01_segment() {
        assert!(is_l01_segment("evidence.L01"));
        assert!(is_l01_segment("evidence.l01"));
        assert!(is_l01_segment("evidence.L99"));
        assert!(is_l01_segment("evidence.LAA"));
        assert!(is_l01_segment("evidence.laa"));
        assert!(is_l01_segment("evidence.LZZ"));
        assert!(!is_l01_segment("evidence.E01"));
        assert!(!is_l01_segment("evidence.txt"));
        assert!(!is_l01_segment("evidence.L0")); // Only 2 chars after L
    }

    #[test]
    fn test_l01_segment_extension() {
        assert_eq!(l01_segment_extension(1), "L01");
        assert_eq!(l01_segment_extension(2), "L02");
        assert_eq!(l01_segment_extension(50), "L50");
        assert_eq!(l01_segment_extension(99), "L99");
        assert_eq!(l01_segment_extension(100), "LAA");
        assert_eq!(l01_segment_extension(101), "LAB");
        assert_eq!(l01_segment_extension(125), "LAZ");
        assert_eq!(l01_segment_extension(126), "LBA");
    }

    #[test]
    fn test_discover_l01_segments_single() {
        let tmp = tempfile::TempDir::new().unwrap();
        let l01_path = tmp.path().join("evidence.L01");
        std::fs::write(&l01_path, b"LVF\x09\x0a\x0d\xff\x00").unwrap();

        let result = discover_l01_segments(l01_path.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], l01_path);
    }

    #[test]
    fn test_discover_l01_segments_multiple() {
        let tmp = tempfile::TempDir::new().unwrap();
        let l01_path = tmp.path().join("case.L01");
        let l02_path = tmp.path().join("case.L02");
        let l03_path = tmp.path().join("case.L03");
        std::fs::write(&l01_path, b"LVF\x09\x0a\x0d\xff\x00").unwrap();
        std::fs::write(&l02_path, b"LVF\x09\x0a\x0d\xff\x00").unwrap();
        std::fs::write(&l03_path, b"LVF\x09\x0a\x0d\xff\x00").unwrap();

        let result = discover_l01_segments(l01_path.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 3);
    }
}
