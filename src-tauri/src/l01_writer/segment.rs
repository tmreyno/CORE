// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Multi-segment file support for L01 writer.
//!
//! When the output exceeds the configured segment size, the writer splits
//! the L01 into multiple segment files:
//!
//! - `.L01` (segment 1)
//! - `.L02` ... `.L99` (segments 2–99)
//! - `.LAA` ... `.LZZ` (segments 100+, if needed)
//!
//! Each segment file has its own LVF file header with the segment number,
//! and the last section of a non-final segment is "next" (indicating
//! continuation). The final segment ends with "done".

use std::path::{Path, PathBuf};

/// Generate the segment file extension for a given segment number.
///
/// Segment 1: `.L01`
/// Segments 2-99: `.L02` ... `.L99`
/// Segments 100+: `.LAA`, `.LAB`, ... `.LZZ`
pub fn segment_extension(segment_number: u16) -> String {
    if segment_number == 0 {
        return "L01".to_string();
    }

    if segment_number <= 99 {
        format!("L{:02}", segment_number)
    } else {
        // Segments 100+: use letter pairs AA-ZZ (676 additional segments)
        let idx = (segment_number - 100) as u32;
        let first = (b'A' + (idx / 26) as u8) as char;
        let second = (b'A' + (idx % 26) as u8) as char;
        format!("L{}{}", first, second)
    }
}

/// Generate the full path for a segment file.
///
/// Takes the base output path (e.g., `/path/to/output.L01`) and returns
/// the path for the given segment number.
pub fn segment_path(base_path: &Path, segment_number: u16) -> PathBuf {
    let ext = segment_extension(segment_number);
    base_path.with_extension(&ext)
}

/// Maximum number of segments supported.
///
/// 99 numeric (L01-L99) + 676 letter pairs (LAA-LZZ) = 775
pub const MAX_SEGMENTS: u16 = 775;

/// Check if we need to start a new segment based on the current file
/// position and the configured segment size.
///
/// Returns `true` if the current position plus `additional_bytes`
/// would exceed the segment size limit.
pub fn should_split(current_position: u64, additional_bytes: u64, segment_size: u64) -> bool {
    if segment_size == 0 {
        return false; // No splitting
    }
    current_position + additional_bytes > segment_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_extension_first() {
        assert_eq!(segment_extension(0), "L01");
        assert_eq!(segment_extension(1), "L01");
    }

    #[test]
    fn test_segment_extension_numeric() {
        assert_eq!(segment_extension(2), "L02");
        assert_eq!(segment_extension(10), "L10");
        assert_eq!(segment_extension(99), "L99");
    }

    #[test]
    fn test_segment_extension_letters() {
        assert_eq!(segment_extension(100), "LAA");
        assert_eq!(segment_extension(101), "LAB");
        assert_eq!(segment_extension(125), "LAZ");
        assert_eq!(segment_extension(126), "LBA");
    }

    #[test]
    fn test_segment_path() {
        let base = Path::new("/evidence/case.L01");
        assert_eq!(segment_path(base, 1), PathBuf::from("/evidence/case.L01"));
        assert_eq!(segment_path(base, 2), PathBuf::from("/evidence/case.L02"));
        assert_eq!(segment_path(base, 100), PathBuf::from("/evidence/case.LAA"));
    }

    #[test]
    fn test_should_split() {
        // No splitting when segment_size is 0
        assert!(!should_split(1_000_000, 1_000_000, 0));

        // Under limit
        assert!(!should_split(500, 400, 1000));

        // At limit
        assert!(!should_split(500, 500, 1000));

        // Over limit
        assert!(should_split(500, 501, 1000));
    }
}
