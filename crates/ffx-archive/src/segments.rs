// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive segment discovery
//!
//! Provides multi-part archive segment detection and discovery for:
//! - 7-Zip: .7z.001, .7z.002, etc.
//! - ZIP: .zip.001, .z01, .z02, etc.
//! - RAR: .rar, .r00, .r01, etc.
//!
//! Note: For unified segment classification (is_first_segment, is_continuation_segment),
//! use `containers::segments` which handles all format types.

use std::path::Path;

use super::types::ArchiveFormat;

// =============================================================================
// Segment Pattern Types
// =============================================================================

#[derive(Clone, Copy)]
pub enum SegmentPatternType {
    /// .7z.001, .7z.002, etc.
    DotNumeric,
    /// .zip.001, .zip.002, etc.
    ZipNumeric,
    /// .r00, .r01, .rar
    RarStyle,
    /// .z01, .z02, .zip
    ZipSplit,
    /// Single file (no pattern)
    Single,
}

// =============================================================================
// Segment Discovery
// =============================================================================

/// Discover all segments of a multi-part archive
pub fn discover_segments(
    path: &str,
    format: ArchiveFormat,
) -> Result<(Vec<String>, Vec<u64>), String> {
    let path_obj = Path::new(path);
    let dir = path_obj
        .parent()
        .ok_or_else(|| "Cannot determine parent directory".to_string())?;
    let filename = path_obj
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?;

    let lower = filename.to_lowercase();

    // Get the base name for segment matching
    let (base_name, pattern_type) = get_segment_pattern(&lower, format);

    let mut segments: Vec<(String, u64, u32)> = Vec::new();

    // Read directory and find matching segments
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let entry_name = entry.file_name().to_string_lossy().to_string();
            let entry_lower = entry_name.to_lowercase();

            if let Some(seg_num) = match_segment(&entry_lower, &base_name, pattern_type) {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        let full_path = entry.path().to_string_lossy().to_string();
                        segments.push((full_path, meta.len(), seg_num));
                    }
                }
            }
        }
    }

    // Sort by segment number
    segments.sort_by_key(|(_, _, num)| *num);

    // If no segments found, just use the original file
    if segments.is_empty() {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        return Ok((vec![path.to_string()], vec![size]));
    }

    let names: Vec<String> = segments.iter().map(|(n, _, _)| n.clone()).collect();
    let sizes: Vec<u64> = segments.iter().map(|(_, s, _)| *s).collect();

    Ok((names, sizes))
}

/// Get segment pattern for an archive format
pub fn get_segment_pattern(lower: &str, format: ArchiveFormat) -> (String, SegmentPatternType) {
    match format {
        ArchiveFormat::SevenZip => {
            // Check for .7z.001 pattern
            if let Some(pos) = lower.rfind(".7z.") {
                let base = &lower[..pos + 3]; // Include .7z
                return (base.to_string(), SegmentPatternType::DotNumeric);
            }
            // Single .7z file
            if let Some(base) = lower.strip_suffix(".7z") {
                return (base.to_string(), SegmentPatternType::Single);
            }
        }
        ArchiveFormat::Zip => {
            // Check for .zip.001 pattern
            if let Some(pos) = lower.rfind(".zip.") {
                let base = &lower[..pos + 4]; // Include .zip
                return (base.to_string(), SegmentPatternType::ZipNumeric);
            }
            // Check for .z01, .z02 pattern (pkzip split)
            if lower.ends_with(".z01") || lower.ends_with(".z02") {
                let base = &lower[..lower.len() - 4];
                return (base.to_string(), SegmentPatternType::ZipSplit);
            }
            if let Some(base) = lower.strip_suffix(".zip") {
                return (base.to_string(), SegmentPatternType::Single);
            }
        }
        ArchiveFormat::Zip64 => {
            // Same patterns as ZIP
            if let Some(pos) = lower.rfind(".zip.") {
                let base = &lower[..pos + 4];
                return (base.to_string(), SegmentPatternType::ZipNumeric);
            }
            if lower.ends_with(".z01") || lower.ends_with(".z02") {
                let base = &lower[..lower.len() - 4];
                return (base.to_string(), SegmentPatternType::ZipSplit);
            }
            if let Some(base) = lower.strip_suffix(".zip") {
                return (base.to_string(), SegmentPatternType::Single);
            }
        }
        ArchiveFormat::Rar4 | ArchiveFormat::Rar5 => {
            // RAR segments: .rar, .r00, .r01, etc.
            if let Some(base) = lower.strip_suffix(".rar") {
                return (base.to_string(), SegmentPatternType::RarStyle);
            }
            if lower.len() > 4
                && lower
                    .chars()
                    .rev()
                    .take(3)
                    .all(|c| c.is_ascii_digit() || c == 'r' || c == '.')
            {
                // .r00, .r01, etc.
                let base = &lower[..lower.len() - 4];
                return (base.to_string(), SegmentPatternType::RarStyle);
            }
        }
        _ => {}
    }

    // Fallback: single file
    (lower.to_string(), SegmentPatternType::Single)
}

/// Match a filename against a segment pattern
pub fn match_segment(
    entry_lower: &str,
    base_name: &str,
    pattern: SegmentPatternType,
) -> Option<u32> {
    match pattern {
        SegmentPatternType::DotNumeric => {
            // Match base.7z.NNN
            if entry_lower.starts_with(base_name) && entry_lower.len() > base_name.len() + 1 {
                let suffix = &entry_lower[base_name.len() + 1..]; // Skip the dot after .7z
                if suffix.chars().all(|c| c.is_ascii_digit()) {
                    return suffix.parse().ok();
                }
            }
            None
        }
        SegmentPatternType::ZipNumeric => {
            // Match base.zip.NNN
            if entry_lower.starts_with(base_name) && entry_lower.len() > base_name.len() + 1 {
                let suffix = &entry_lower[base_name.len() + 1..];
                if suffix.chars().all(|c| c.is_ascii_digit()) {
                    return suffix.parse().ok();
                }
            }
            None
        }
        SegmentPatternType::RarStyle => {
            // Match base.rar (segment 0) or base.rNN
            if entry_lower == format!("{}.rar", base_name) {
                return Some(0);
            }
            let prefix = format!("{}.", base_name);
            if entry_lower.starts_with(&prefix) {
                let ext = &entry_lower[prefix.len()..];
                if ext.starts_with('r') && ext.len() == 3 {
                    let num_part = &ext[1..];
                    if num_part.chars().all(|c| c.is_ascii_digit()) {
                        return num_part.parse::<u32>().ok().map(|n| n + 1);
                    }
                }
            }
            None
        }
        SegmentPatternType::ZipSplit => {
            // Match base.zip (last segment) or base.zNN
            if entry_lower == format!("{}.zip", base_name) {
                return Some(999); // ZIP file is always last in split archives
            }
            let prefix = format!("{}.", base_name);
            if entry_lower.starts_with(&prefix) {
                let ext = &entry_lower[prefix.len()..];
                if ext.starts_with('z') && ext.len() == 3 {
                    let num_part = &ext[1..];
                    if num_part.chars().all(|c| c.is_ascii_digit()) {
                        return num_part.parse().ok();
                    }
                }
            }
            None
        }
        SegmentPatternType::Single => {
            if entry_lower == base_name {
                return Some(1);
            }
            None
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_segment_pattern_7z() {
        let (base, pattern) = get_segment_pattern("archive.7z.001", ArchiveFormat::SevenZip);
        assert_eq!(base, "archive.7z");
        assert!(matches!(pattern, SegmentPatternType::DotNumeric));

        let (base, pattern) = get_segment_pattern("archive.7z", ArchiveFormat::SevenZip);
        assert_eq!(base, "archive");
        assert!(matches!(pattern, SegmentPatternType::Single));
    }

    #[test]
    fn test_get_segment_pattern_zip() {
        let (base, pattern) = get_segment_pattern("archive.zip.001", ArchiveFormat::Zip);
        assert_eq!(base, "archive.zip");
        assert!(matches!(pattern, SegmentPatternType::ZipNumeric));

        let (base, pattern) = get_segment_pattern("archive.z01", ArchiveFormat::Zip);
        assert_eq!(base, "archive");
        assert!(matches!(pattern, SegmentPatternType::ZipSplit));

        let (base, pattern) = get_segment_pattern("archive.zip", ArchiveFormat::Zip);
        assert_eq!(base, "archive");
        assert!(matches!(pattern, SegmentPatternType::Single));
    }

    #[test]
    fn test_get_segment_pattern_rar() {
        let (base, pattern) = get_segment_pattern("archive.rar", ArchiveFormat::Rar5);
        assert_eq!(base, "archive");
        assert!(matches!(pattern, SegmentPatternType::RarStyle));

        let (base, pattern) = get_segment_pattern("archive.r00", ArchiveFormat::Rar5);
        assert_eq!(base, "archive");
        assert!(matches!(pattern, SegmentPatternType::RarStyle));
    }

    #[test]
    fn test_match_segment_7z_numeric() {
        let pattern = SegmentPatternType::DotNumeric;
        assert_eq!(
            match_segment("archive.7z.001", "archive.7z", pattern),
            Some(1)
        );
        assert_eq!(
            match_segment("archive.7z.002", "archive.7z", pattern),
            Some(2)
        );
        assert_eq!(
            match_segment("archive.7z.100", "archive.7z", pattern),
            Some(100)
        );
        assert_eq!(match_segment("other.7z.001", "archive.7z", pattern), None);
    }

    #[test]
    fn test_match_segment_zip_numeric() {
        let pattern = SegmentPatternType::ZipNumeric;
        assert_eq!(
            match_segment("archive.zip.001", "archive.zip", pattern),
            Some(1)
        );
        assert_eq!(
            match_segment("archive.zip.002", "archive.zip", pattern),
            Some(2)
        );
    }

    #[test]
    fn test_match_segment_rar_style() {
        let pattern = SegmentPatternType::RarStyle;
        // .rar is segment 0
        assert_eq!(match_segment("archive.rar", "archive", pattern), Some(0));
        // .r00 is segment 1, .r01 is segment 2, etc.
        assert_eq!(match_segment("archive.r00", "archive", pattern), Some(1));
        assert_eq!(match_segment("archive.r01", "archive", pattern), Some(2));
        assert_eq!(match_segment("archive.r99", "archive", pattern), Some(100));
    }

    #[test]
    fn test_match_segment_zip_split() {
        let pattern = SegmentPatternType::ZipSplit;
        // .zip is the final segment (999)
        assert_eq!(match_segment("archive.zip", "archive", pattern), Some(999));
        // .z01, .z02, etc.
        assert_eq!(match_segment("archive.z01", "archive", pattern), Some(1));
        assert_eq!(match_segment("archive.z02", "archive", pattern), Some(2));
    }

    #[test]
    fn test_match_segment_single() {
        let pattern = SegmentPatternType::Single;
        assert_eq!(match_segment("archive.7z", "archive.7z", pattern), Some(1));
        assert_eq!(match_segment("other.7z", "archive.7z", pattern), None);
    }
}
