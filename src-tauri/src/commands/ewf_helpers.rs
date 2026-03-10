// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF export helper functions — format parsing, validation, and filesystem utilities.

use libewf_ffi::{EwfCompression, EwfCompressionMethod, EwfFormat};
use std::path::Path;

// =============================================================================
// Format & Compression Parsing
// =============================================================================

pub(super) fn parse_format(format: &str) -> Result<EwfFormat, String> {
    match format.to_lowercase().as_str() {
        "e01" | "encase5" => Ok(EwfFormat::Encase5),
        "encase6" => Ok(EwfFormat::Encase6),
        // Encase7 (0x07) produces .E01 (EWF1 segment type)
        "encase7" => Ok(EwfFormat::Encase7),
        // V2Encase7 (0x37) produces .Ex01 (EWF2 segment type) — supports BZIP2
        "v2encase7" | "ex01" => Ok(EwfFormat::V2Encase7),
        "ftk" => Ok(EwfFormat::FtkImager),
        // Logical formats (L01, Lx01) are NOT supported for writing by libewf 20251220
        "l01" | "logical" | "logical_encase5" | "l01v6" | "logical_encase6"
        | "l01v7" | "logical_encase7" | "lx01" | "v2logical_encase7" => {
            Err("Logical EWF formats (L01/Lx01) are not supported for writing by libewf. Use a physical image format (E01/Ex01) instead.".to_string())
        }
        _ => Err(format!("Unknown EWF format: {}", format)),
    }
}

pub(super) fn parse_compression(compression: &str) -> Result<EwfCompression, String> {
    match compression.to_lowercase().as_str() {
        "none" | "store" => Ok(EwfCompression::None),
        "fast" => Ok(EwfCompression::Fast),
        "best" | "maximum" => Ok(EwfCompression::Best),
        _ => Err(format!("Unknown compression level: {}", compression)),
    }
}

pub(super) fn parse_compression_method(method: &str) -> Result<EwfCompressionMethod, String> {
    match method.to_lowercase().as_str() {
        "deflate" | "zlib" => Ok(EwfCompressionMethod::Deflate),
        "bzip2" | "bz2" => Ok(EwfCompressionMethod::Bzip2),
        "none" => Ok(EwfCompressionMethod::None),
        _ => Err(format!("Unknown compression method: {}", method)),
    }
}

// =============================================================================
// Filesystem & Validation Utilities
// =============================================================================

/// Check if a canonicalized path is the system boot volume.
/// Cross-platform: detects macOS root, Windows C:\ drive, and Linux root.
pub(super) fn is_system_boot_volume(canon: &Path) -> bool {
    let canon_str = canon.to_string_lossy();
    #[cfg(target_os = "macos")]
    {
        if canon_str == "/" || canon_str == "/System/Volumes/Data" {
            return true;
        }
    }
    #[cfg(target_os = "windows")]
    {
        let upper = canon_str.to_uppercase();
        if upper == "C:\\" || upper == "C:" || upper.starts_with("C:\\") && canon.parent().is_none()
        {
            return true;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if canon_str == "/" {
            return true;
        }
    }
    false
}

/// Result of a quick statvfs / disk-space query.
pub(super) struct DiskSpaceInfo {
    pub available_space: u64,
}

/// Query the available space on the filesystem containing `path`.
pub(super) fn nix_stat(path: &Path) -> Result<DiskSpaceInfo, String> {
    // Use std::fs metadata approach — works cross-platform.
    // On Unix we can use statvfs for accuracy.
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|e| format!("Invalid path: {e}"))?;
        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                #[allow(clippy::unnecessary_cast)]
                let avail = stat.f_bavail as u64 * stat.f_frsize as u64;
                return Ok(DiskSpaceInfo {
                    available_space: avail,
                });
            }
        }
        Err("statvfs failed".into())
    }
    #[cfg(not(unix))]
    {
        // Fallback: sysinfo crate disk info (less precise but works)
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for d in disks.iter() {
            if path.starts_with(d.mount_point()) {
                return Ok(DiskSpaceInfo {
                    available_space: d.available_space(),
                });
            }
        }
        Err("Could not determine available space".into())
    }
}

/// Format a byte count as a human-readable string (e.g. "12.3 GB").
pub(super) fn format_byte_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let exp = (bytes as f64).log(1024.0).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);
    let value = bytes as f64 / 1024_f64.powi(exp as i32);
    if exp == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", value, UNITS[exp])
    }
}

/// Recursively walk a directory and collect all files with their sizes.
/// Returned paths are absolute. Skips symlinks and unreadable entries.
pub(super) fn walk_dir_files(dir: &Path) -> Result<Vec<(String, u64)>, String> {
    let mut results = Vec::new();
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry =
            entry.map_err(|e| format!("Failed to read entry in {}: {}", dir.display(), e))?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .map_err(|e| format!("Failed to get file type for {}: {}", path.display(), e))?;
        if ft.is_file() {
            let size = entry
                .metadata()
                .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?
                .len();
            results.push((path.to_string_lossy().into_owned(), size));
        } else if ft.is_dir() {
            let sub = walk_dir_files(&path)?;
            results.extend(sub);
        }
        // Skip symlinks and other special entries
    }
    Ok(results)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse_format ====================

    #[test]
    fn test_parse_format_e01() {
        let result = parse_format("e01").unwrap();
        assert!(matches!(result, EwfFormat::Encase5));
    }

    #[test]
    fn test_parse_format_encase5() {
        let result = parse_format("encase5").unwrap();
        assert!(matches!(result, EwfFormat::Encase5));
    }

    #[test]
    fn test_parse_format_encase6() {
        let result = parse_format("encase6").unwrap();
        assert!(matches!(result, EwfFormat::Encase6));
    }

    #[test]
    fn test_parse_format_encase7() {
        let result = parse_format("encase7").unwrap();
        assert!(matches!(result, EwfFormat::Encase7));
    }

    #[test]
    fn test_parse_format_v2encase7() {
        let result = parse_format("v2encase7").unwrap();
        assert!(matches!(result, EwfFormat::V2Encase7));
    }

    #[test]
    fn test_parse_format_ex01() {
        let result = parse_format("ex01").unwrap();
        assert!(matches!(result, EwfFormat::V2Encase7));
    }

    #[test]
    fn test_parse_format_l01_rejected() {
        let result = parse_format("l01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not supported"));
    }

    #[test]
    fn test_parse_format_logical_rejected() {
        assert!(parse_format("logical").is_err());
    }

    #[test]
    fn test_parse_format_logical_encase5_rejected() {
        assert!(parse_format("logical_encase5").is_err());
    }

    #[test]
    fn test_parse_format_l01v6_rejected() {
        assert!(parse_format("l01v6").is_err());
    }

    #[test]
    fn test_parse_format_logical_encase6_rejected() {
        assert!(parse_format("logical_encase6").is_err());
    }

    #[test]
    fn test_parse_format_l01v7_rejected() {
        assert!(parse_format("l01v7").is_err());
    }

    #[test]
    fn test_parse_format_logical_encase7_rejected() {
        assert!(parse_format("logical_encase7").is_err());
    }

    #[test]
    fn test_parse_format_lx01_rejected() {
        assert!(parse_format("lx01").is_err());
    }

    #[test]
    fn test_parse_format_v2logical_encase7_rejected() {
        assert!(parse_format("v2logical_encase7").is_err());
    }

    #[test]
    fn test_parse_format_ftk() {
        let result = parse_format("ftk").unwrap();
        assert!(matches!(result, EwfFormat::FtkImager));
    }

    #[test]
    fn test_parse_format_case_insensitive() {
        assert!(matches!(parse_format("E01").unwrap(), EwfFormat::Encase5));
        assert!(matches!(
            parse_format("ENCASE7").unwrap(),
            EwfFormat::Encase7
        ));
        assert!(matches!(
            parse_format("V2Encase7").unwrap(),
            EwfFormat::V2Encase7
        ));
        assert!(matches!(parse_format("FTK").unwrap(), EwfFormat::FtkImager));
    }

    #[test]
    fn test_parse_format_unknown() {
        let result = parse_format("unknown");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown EWF format"));
    }

    #[test]
    fn test_parse_format_empty() {
        let result = parse_format("");
        assert!(result.is_err());
    }

    // ==================== parse_compression ====================

    #[test]
    fn test_parse_compression_none() {
        let result = parse_compression("none").unwrap();
        assert!(matches!(result, EwfCompression::None));
    }

    #[test]
    fn test_parse_compression_store() {
        let result = parse_compression("store").unwrap();
        assert!(matches!(result, EwfCompression::None));
    }

    #[test]
    fn test_parse_compression_fast() {
        let result = parse_compression("fast").unwrap();
        assert!(matches!(result, EwfCompression::Fast));
    }

    #[test]
    fn test_parse_compression_best() {
        let result = parse_compression("best").unwrap();
        assert!(matches!(result, EwfCompression::Best));
    }

    #[test]
    fn test_parse_compression_maximum() {
        let result = parse_compression("maximum").unwrap();
        assert!(matches!(result, EwfCompression::Best));
    }

    #[test]
    fn test_parse_compression_case_insensitive() {
        assert!(matches!(
            parse_compression("NONE").unwrap(),
            EwfCompression::None
        ));
        assert!(matches!(
            parse_compression("Fast").unwrap(),
            EwfCompression::Fast
        ));
        assert!(matches!(
            parse_compression("BEST").unwrap(),
            EwfCompression::Best
        ));
    }

    #[test]
    fn test_parse_compression_unknown() {
        let result = parse_compression("turbo");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown compression level"));
    }

    // ==================== parse_compression_method ====================

    #[test]
    fn test_parse_compression_method_deflate() {
        let result = parse_compression_method("deflate").unwrap();
        assert!(matches!(result, EwfCompressionMethod::Deflate));
    }

    #[test]
    fn test_parse_compression_method_zlib() {
        let result = parse_compression_method("zlib").unwrap();
        assert!(matches!(result, EwfCompressionMethod::Deflate));
    }

    #[test]
    fn test_parse_compression_method_bzip2() {
        let result = parse_compression_method("bzip2").unwrap();
        assert!(matches!(result, EwfCompressionMethod::Bzip2));
    }

    #[test]
    fn test_parse_compression_method_bz2() {
        let result = parse_compression_method("bz2").unwrap();
        assert!(matches!(result, EwfCompressionMethod::Bzip2));
    }

    #[test]
    fn test_parse_compression_method_none() {
        let result = parse_compression_method("none").unwrap();
        assert!(matches!(result, EwfCompressionMethod::None));
    }

    #[test]
    fn test_parse_compression_method_case_insensitive() {
        assert!(matches!(
            parse_compression_method("DEFLATE").unwrap(),
            EwfCompressionMethod::Deflate
        ));
        assert!(matches!(
            parse_compression_method("BZIP2").unwrap(),
            EwfCompressionMethod::Bzip2
        ));
        assert!(matches!(
            parse_compression_method("None").unwrap(),
            EwfCompressionMethod::None
        ));
    }

    #[test]
    fn test_parse_compression_method_unknown() {
        let result = parse_compression_method("lzma");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown compression method"));
    }

    // ==================== Format extensions (critical forensic invariant) ====================

    #[test]
    fn test_encase5_produces_e01_extension() {
        let format = parse_format("e01").unwrap();
        assert_eq!(format.extension(), ".E01");
    }

    #[test]
    fn test_encase7_produces_e01_extension() {
        // Encase7 (0x07) uses EWF1 segment type → .E01, NOT .Ex01
        let format = parse_format("encase7").unwrap();
        assert_eq!(format.extension(), ".E01");
    }

    #[test]
    fn test_v2encase7_produces_ex01_extension() {
        // V2Encase7 (0x37) uses EWF2 segment type → .Ex01
        let format = parse_format("v2encase7").unwrap();
        assert_eq!(format.extension(), ".Ex01");
    }

    #[test]
    fn test_ex01_alias_produces_ex01_extension() {
        let format = parse_format("ex01").unwrap();
        assert_eq!(format.extension(), ".Ex01");
    }

    #[test]
    fn test_l01_rejected() {
        assert!(parse_format("l01").is_err());
    }

    #[test]
    fn test_lx01_rejected() {
        assert!(parse_format("lx01").is_err());
    }

    // ==================== Format-specific invariants ====================

    #[test]
    fn test_ex01_maps_to_v2encase7_not_encase7() {
        // Critical: "ex01" MUST map to V2Encase7, NOT Encase7
        // Encase7 produces .E01, V2Encase7 produces .Ex01
        let format = parse_format("ex01").unwrap();
        assert!(matches!(format, EwfFormat::V2Encase7));
        assert!(!matches!(format, EwfFormat::Encase7));
    }

    #[test]
    fn test_bzip2_requires_v2_format() {
        // bzip2 compression method is only valid with V2 formats
        let method = parse_compression_method("bzip2").unwrap();
        assert!(matches!(method, EwfCompressionMethod::Bzip2));

        // V2 formats that support bzip2
        let v2 = parse_format("v2encase7").unwrap();
        assert!(v2.is_v2());

        // Logical V2 (lx01) no longer supported for writing
        assert!(parse_format("lx01").is_err());

        // Non-V2 formats (bzip2 would be invalid with these)
        let e5 = parse_format("e01").unwrap();
        assert!(!e5.is_v2());
    }
}
