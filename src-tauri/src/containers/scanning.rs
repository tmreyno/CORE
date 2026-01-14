// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Directory scanning for forensic containers
//!
//! This module provides functions for discovering forensic container files
//! in directories, with support for streaming results and recursive scanning.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tracing::debug;
use super::ContainerError;

use super::types::DiscoveredFile;
use super::segments::{
    is_first_segment, is_numbered_segment, is_archive_segment,
    get_segment_basename, get_first_segment_path_fast,
};
use crate::formats::detect_format_by_extension;

/// Scan a directory for forensic container files (non-recursive)
pub fn scan_directory(dir_path: &str) -> Result<Vec<DiscoveredFile>, ContainerError> {
    scan_directory_impl(dir_path, false)
}

/// Scan a directory recursively for forensic container files
pub fn scan_directory_recursive(dir_path: &str) -> Result<Vec<DiscoveredFile>, ContainerError> {
    scan_directory_impl(dir_path, true)
}

/// Streaming scan that calls callback for each file found (for real-time UI updates)
pub fn scan_directory_streaming<F>(dir_path: &str, recursive: bool, on_file_found: F) -> Result<usize, ContainerError>
where
    F: Fn(&DiscoveredFile),
{
    let path = Path::new(dir_path);
    if !path.exists() {
        return Err(ContainerError::from(format!("Directory not found: {dir_path}")));
    }
    if !path.is_dir() {
        return Err(ContainerError::from(format!("Path is not a directory: {dir_path}")));
    }

    let mut seen_basenames = HashSet::new();
    let mut count = 0;

    scan_dir_streaming_internal(path, &mut seen_basenames, recursive, &on_file_found, &mut count)?;

    Ok(count)
}

fn scan_dir_streaming_internal<F>(
    path: &Path,
    seen_basenames: &mut HashSet<String>,
    recursive: bool,
    on_file_found: &F,
    count: &mut usize,
) -> Result<(), ContainerError>
where
    F: Fn(&DiscoveredFile),
{
    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {e}"))?;

    // First pass: collect all entries and find UFD files (to identify UFED extraction sets)
    let mut file_entries = Vec::new();
    let mut ufd_basenames: HashSet<String> = HashSet::new();
    let mut ufd_paths: std::collections::HashMap<String, std::path::PathBuf> = std::collections::HashMap::new();
    let mut subdirs = Vec::new();
    
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let entry_path = entry.path();
        
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                tracing::warn!("Failed to get file type for {:?}: {}", entry_path, e);
                continue;
            }
        };
        
        if file_type.is_dir() {
            if recursive {
                subdirs.push(entry_path);
            }
            continue;
        }
        
        if !file_type.is_file() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();
        let lower = filename.to_lowercase();
        
        // Track UFD files to identify UFED extraction sets
        if lower.ends_with(".ufd") {
            // Extract basename without extension
            if let Some(stem) = Path::new(&filename).file_stem() {
                let stem_lower = stem.to_string_lossy().to_lowercase();
                ufd_basenames.insert(stem_lower.clone());
                ufd_paths.insert(stem_lower, entry_path.clone());
            }
        }
        
        file_entries.push((entry, filename, lower));
    }
    
    // Recurse into subdirectories
    for subdir in subdirs {
        let _ = scan_dir_streaming_internal(&subdir, seen_basenames, recursive, on_file_found, count);
    }
    
    // Second pass: process files
    // - UFD files are skipped (metadata only, not evidence containers)
    // - UFDX files are skipped (collection index)
    // - ZIP files with matching UFD are detected as "UFED" type containers
    for (entry, filename, lower) in file_entries {
        let entry_path = entry.path();
        
        let path_str = match entry_path.to_str() {
            Some(s) => s,
            None => continue,
        };

        // Skip macOS resource fork files (._filename)
        if filename.starts_with("._") {
            continue;
        }
        
        // Skip non-first segments entirely - we only want to show one entry per container
        if !is_first_segment(&lower) {
            continue;
        }
        
        // Skip UFDX files - these are collection indexes/pointers, not evidence containers
        // They point to actual evidence but contain no evidence data themselves
        if lower.ends_with(".ufdx") {
            debug!("Skipping UFED collection index: {} (metadata pointer, not evidence)", filename);
            continue;
        }
        
        // Skip UFD files when they exist alongside matching ZIP (metadata only)
        if !ufd_basenames.is_empty() && lower.ends_with(".ufd") {
            debug!("Skipping UFED metadata file: {} (metadata only)", filename);
            continue;
        }
        
        // Check for forensic container files by extension only (fast, no file I/O)
        // Special case: ZIP files with sibling UFD are UFED extraction containers
        // Note: UFD may have suffix like "_AdvancedLogical" that ZIP doesn't have,
        // so we check if any UFD basename STARTS WITH the ZIP stem.
        let container_type = if lower.ends_with(".zip") {
            if let Some(stem) = Path::new(&filename).file_stem() {
                let stem_lower = stem.to_string_lossy().to_lowercase();
                // Check if any UFD file starts with this ZIP's stem
                let has_matching_ufd = ufd_basenames.iter().any(|ufd_stem| {
                    ufd_stem.starts_with(&stem_lower)
                });
                if has_matching_ufd {
                    Some("UFED")
                } else {
                    detect_container_type_by_extension(&lower)
                }
            } else {
                detect_container_type_by_extension(&lower)
            }
        } else {
            detect_container_type_by_extension(&lower)
        };

        if let Some(ctype) = container_type {
            // For multi-segment files (like .E01, .001), only show the first segment
            let basename = get_segment_basename(&filename);
            if seen_basenames.insert(basename.clone()) {
                // For numbered segments, construct .001 path without checking existence (fast)
                let display_path = if is_numbered_segment(&lower) {
                    get_first_segment_path_fast(path_str)
                } else {
                    path_str.to_string()
                };
                
                let display_filename = Path::new(&display_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or(filename.clone());
                
                // Use DirEntry metadata (cached from readdir syscall) - fast
                let metadata = entry.metadata().ok();
                let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                
                // Extract timestamps from metadata
                let created = metadata.as_ref()
                    .and_then(|m| m.created().ok())
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    });
                let modified = metadata.as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    });
                
                // FAST PATH: Skip segment calculation during scan - it's slow on external drives
                // Segment details will be calculated on-demand when user selects a file
                
                let file = DiscoveredFile {
                    path: display_path,
                    filename: display_filename,
                    container_type: ctype.to_string(),
                    size: file_size, // Just first segment size - full size calculated on-demand
                    segment_count: None,
                    segment_files: None,
                    segment_sizes: None,
                    total_segment_size: None,
                    created,
                    modified,
                };
                
                on_file_found(&file);
                *count += 1;
            } else {
                debug!("Skipping duplicate basename: {}", filename);
            }
        } else {
            debug!("Skipping file with unrecognized container type: {}", filename);
        }
    }

    Ok(())
}

fn scan_directory_impl(dir_path: &str, recursive: bool) -> Result<Vec<DiscoveredFile>, ContainerError> {
    let path = Path::new(dir_path);
    if !path.exists() {
        return Err(ContainerError::from(format!("Directory not found: {dir_path}")));
    }
    if !path.is_dir() {
        return Err(ContainerError::from(format!("Path is not a directory: {dir_path}")));
    }

    let mut discovered = Vec::new();
    let mut seen_basenames = HashSet::new();

    scan_dir_internal(path, &mut discovered, &mut seen_basenames, recursive)?;

    Ok(discovered)
}

fn scan_dir_internal(
    path: &Path,
    discovered: &mut Vec<DiscoveredFile>,
    seen_basenames: &mut HashSet<String>,
    recursive: bool,
) -> Result<(), ContainerError> {
    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {e}"))?;

    // First pass: collect all entries and find UFD files (to identify UFED extraction sets)
    let mut file_entries = Vec::new();
    let mut ufd_basenames: HashSet<String> = HashSet::new();
    let mut subdirs = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let entry_path = entry.path();
        
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                tracing::warn!("Failed to get file type for {:?}: {}", entry_path, e);
                continue;
            }
        };
        
        if file_type.is_dir() {
            if recursive {
                subdirs.push(entry_path);
            }
            continue;
        }
        
        if !file_type.is_file() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();
        let lower = filename.to_lowercase();
        
        // Track UFD files to identify UFED extraction sets
        if lower.ends_with(".ufd") {
            if let Some(stem) = Path::new(&filename).file_stem() {
                let stem_lower = stem.to_string_lossy().to_lowercase();
                ufd_basenames.insert(stem_lower);
            }
        }
        
        file_entries.push((entry, filename, lower));
    }
    
    // Recurse into subdirectories
    for subdir in subdirs {
        let _ = scan_dir_internal(&subdir, discovered, seen_basenames, recursive);
    }

    // Second pass: process files
    // - UFD files are skipped (metadata only, not evidence containers)
    // - UFDX files are skipped (collection index)
    // - ZIP files with matching UFD are detected as "UFED" type containers
    for (entry, filename, lower) in file_entries {
        let entry_path = entry.path();
        
        let path_str = match entry_path.to_str() {
            Some(s) => s,
            None => {
                tracing::warn!("Failed to convert path to string: {:?}", entry_path);
                continue;
            }
        };

        // Skip macOS resource fork files (._filename)
        if filename.starts_with("._") {
            continue;
        }
        
        // Skip non-first segments entirely - we only want to show one entry per container
        if !is_first_segment(&lower) {
            continue;
        }
        
        // Skip UFDX files - these are collection indexes/pointers, not evidence containers
        // They point to actual evidence but contain no evidence data themselves
        if lower.ends_with(".ufdx") {
            debug!("Skipping UFED collection index: {} (metadata pointer, not evidence)", filename);
            continue;
        }
        
        // Skip UFD files when they exist alongside matching ZIP (metadata only)
        if !ufd_basenames.is_empty() && lower.ends_with(".ufd") {
            debug!("Skipping UFED metadata file: {} (metadata only)", filename);
            continue;
        }
        
        // Check for forensic container files by extension only (fast, no file I/O)
        // Special case: ZIP files with sibling UFD are UFED extraction containers
        // Note: UFD may have suffix like "_AdvancedLogical" that ZIP doesn't have,
        // so we check if any UFD basename STARTS WITH the ZIP stem.
        let container_type = if lower.ends_with(".zip") {
            if let Some(stem) = Path::new(&filename).file_stem() {
                let stem_lower = stem.to_string_lossy().to_lowercase();
                // Check if any UFD file starts with this ZIP's stem
                let has_matching_ufd = ufd_basenames.iter().any(|ufd_stem| {
                    ufd_stem.starts_with(&stem_lower)
                });
                if has_matching_ufd {
                    Some("UFED")
                } else {
                    detect_container_type_by_extension(&lower)
                }
            } else {
                detect_container_type_by_extension(&lower)
            }
        } else {
            detect_container_type_by_extension(&lower)
        };

        if let Some(ctype) = container_type {
            // For multi-segment files (like .E01, .001), only show the first segment
            let basename = get_segment_basename(&filename);
            if seen_basenames.insert(basename.clone()) {
                // For numbered segments, always use the first segment path (.001)
                let display_path = if is_numbered_segment(&lower) {
                    get_first_segment_path_fast(path_str)
                } else {
                    path_str.to_string()
                };
                
                let display_filename = Path::new(&display_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or(filename.clone());
                
                // Use DirEntry metadata (cached from readdir syscall) - fast
                let metadata = entry.metadata().ok();
                let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                
                // Extract timestamps from metadata
                let created = metadata.as_ref()
                    .and_then(|m| m.created().ok())
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    });
                let modified = metadata.as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    });
                
                discovered.push(DiscoveredFile {
                    path: display_path,
                    filename: display_filename,
                    container_type: ctype.to_string(),
                    size: file_size, // Just first segment size - full size calculated on-demand
                    segment_count: None,
                    segment_files: None,
                    segment_sizes: None,
                    total_segment_size: None,
                    created,
                    modified,
                });
            } else {
                debug!("Skipping duplicate basename: {}", filename);
            }
        } else {
            debug!("Skipping file with unrecognized container type: {}", filename);
        }
    }

    Ok(())
}

/// Detect container type by file extension only (fast, no file I/O)
/// Returns None for unrecognized extensions
/// 
/// This function uses the centralized format definitions from `crate::formats`
/// but returns display strings for backward compatibility with existing code.
fn detect_container_type_by_extension(lower: &str) -> Option<&'static str> {
    // First, try the centralized format detection
    if let Some(format) = detect_format_by_extension(lower) {
        return Some(match format.id {
            // Forensic containers - use display names matching existing code
            "e01" => "EnCase (E01)",
            "ex01" => "EnCase (Ex01)",
            "l01" => "L01",
            "lx01" => "Lx01",
            "ad1" => "AD1",
            "aff" => "AFF",
            "aff4" => "AFF4",
            // Raw images
            "raw" => "Raw Image",
            // Mobile forensics
            "ufed_ufd" => "UFED (UFD)",
            "ufed_ufdr" => "UFED (UFDR)",
            // Archives
            "7z" => "7-Zip",
            "zip" => "ZIP",
            // Virtual disks
            "vmdk" => "VMDK",
            "vhd" => "VHD",
            "vhdx" => "VHDX",
            "qcow2" => "QCOW2",
            // Optical discs
            "iso" => "ISO 9660",
            "dmg" => "DMG",
            // Fallback to type name
            _ => format.type_name,
        });
    }
    
    // Additional formats not in the centralized registry (for backward compatibility)
    // =========================================================================
    // Extended archive formats with compression indicators
    // =========================================================================
    if lower.ends_with(".rar") || lower.ends_with(".r00") {
        Some("RAR")
    } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Some("TAR.GZ")
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        Some("TAR.XZ")
    } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
        Some("TAR.BZ2")
    } else if lower.ends_with(".tar.zst") {
        Some("TAR.ZSTD")
    } else if lower.ends_with(".tar.lz4") {
        Some("TAR.LZ4")
    } else if lower.ends_with(".tar") {
        if lower.contains("logical") {
            Some("TAR (Logical)")
        } else {
            Some("TAR")
        }
    } else if lower.ends_with(".gz") && !lower.ends_with(".tar.gz") {
        Some("GZIP")
    } else if lower.ends_with(".xz") && !lower.ends_with(".tar.xz") {
        Some("XZ")
    } else if lower.ends_with(".bz2") && !lower.ends_with(".tar.bz2") {
        Some("BZIP2")
    } else if lower.ends_with(".zst") || lower.ends_with(".zstd") {
        Some("ZSTD")
    } else if lower.ends_with(".lz4") && !lower.ends_with(".tar.lz4") {
        Some("LZ4")
    // =========================================================================
    // Additional virtual disk formats
    // =========================================================================
    } else if lower.ends_with(".vdi") {
        Some("VDI")
    // =========================================================================
    // macOS specific formats
    // =========================================================================
    } else if lower.ends_with(".sparsebundle") || lower.ends_with(".sparseimage") {
        Some("Apple Sparse Image")
    // =========================================================================
    // Other forensic formats
    // =========================================================================
    } else if lower.ends_with(".s01") || lower.ends_with(".s02") {
        Some("SMART")
    } else if lower.ends_with(".bin") || lower.ends_with(".cue") {
        Some("BIN/CUE")
    } else if lower.ends_with(".ufdx") {
        // UFDX files are collection indexes/pointers, not evidence containers
        // They point to actual evidence but contain no evidence data themselves
        None
    } else if is_numbered_segment(lower) && !is_archive_segment(lower) {
        // Raw image segments (.001, .002, etc.) - but not archive segments
        Some("Raw Image")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    // ==================== detect_container_type_by_extension tests ====================

    #[test]
    fn test_detect_container_type_e01() {
        assert_eq!(detect_container_type_by_extension("evidence.e01"), Some("EnCase (E01)"));
        assert_eq!(detect_container_type_by_extension("evidence.E01"), Some("EnCase (E01)"));
    }

    #[test]
    fn test_detect_container_type_ex01() {
        assert_eq!(detect_container_type_by_extension("evidence.ex01"), Some("EnCase (Ex01)"));
    }

    #[test]
    fn test_detect_container_type_l01() {
        assert_eq!(detect_container_type_by_extension("logical.l01"), Some("L01"));
    }

    #[test]
    fn test_detect_container_type_lx01() {
        assert_eq!(detect_container_type_by_extension("logical.lx01"), Some("Lx01"));
    }

    #[test]
    fn test_detect_container_type_ad1() {
        assert_eq!(detect_container_type_by_extension("image.ad1"), Some("AD1"));
    }

    #[test]
    fn test_detect_container_type_raw() {
        assert_eq!(detect_container_type_by_extension("disk.dd"), Some("Raw Image"));
        assert_eq!(detect_container_type_by_extension("disk.raw"), Some("Raw Image"));
        assert_eq!(detect_container_type_by_extension("disk.img"), Some("Raw Image"));
    }

    #[test]
    fn test_detect_container_type_vmdk() {
        assert_eq!(detect_container_type_by_extension("virtual.vmdk"), Some("VMDK"));
    }

    #[test]
    fn test_detect_container_type_vhd() {
        assert_eq!(detect_container_type_by_extension("disk.vhd"), Some("VHD"));
        assert_eq!(detect_container_type_by_extension("disk.vhdx"), Some("VHDX"));
    }

    #[test]
    fn test_detect_container_type_qcow2() {
        assert_eq!(detect_container_type_by_extension("disk.qcow2"), Some("QCOW2"));
    }

    #[test]
    fn test_detect_container_type_iso() {
        assert_eq!(detect_container_type_by_extension("disc.iso"), Some("ISO 9660"));
    }

    #[test]
    fn test_detect_container_type_dmg() {
        assert_eq!(detect_container_type_by_extension("disk.dmg"), Some("DMG"));
    }

    #[test]
    fn test_detect_container_type_7z() {
        assert_eq!(detect_container_type_by_extension("archive.7z"), Some("7-Zip"));
    }

    #[test]
    fn test_detect_container_type_zip() {
        assert_eq!(detect_container_type_by_extension("archive.zip"), Some("ZIP"));
    }

    #[test]
    fn test_detect_container_type_ufed() {
        assert_eq!(detect_container_type_by_extension("mobile.ufd"), Some("UFED (UFD)"));
        assert_eq!(detect_container_type_by_extension("mobile.ufdr"), Some("UFED (UFDR)"));
    }

    #[test]
    fn test_detect_container_type_tar_variants() {
        assert_eq!(detect_container_type_by_extension("archive.tar"), Some("TAR"));
        assert_eq!(detect_container_type_by_extension("archive.tar.gz"), Some("TAR.GZ"));
        assert_eq!(detect_container_type_by_extension("archive.tgz"), Some("TAR.GZ"));
        assert_eq!(detect_container_type_by_extension("archive.tar.xz"), Some("TAR.XZ"));
        assert_eq!(detect_container_type_by_extension("archive.txz"), Some("TAR.XZ"));
        assert_eq!(detect_container_type_by_extension("archive.tar.bz2"), Some("TAR.BZ2"));
    }

    #[test]
    fn test_detect_container_type_compression() {
        assert_eq!(detect_container_type_by_extension("file.gz"), Some("GZIP"));
        assert_eq!(detect_container_type_by_extension("file.xz"), Some("XZ"));
        assert_eq!(detect_container_type_by_extension("file.bz2"), Some("BZIP2"));
        assert_eq!(detect_container_type_by_extension("file.zst"), Some("ZSTD"));
        assert_eq!(detect_container_type_by_extension("file.lz4"), Some("LZ4"));
    }

    #[test]
    fn test_detect_container_type_rar() {
        assert_eq!(detect_container_type_by_extension("archive.rar"), Some("RAR"));
        assert_eq!(detect_container_type_by_extension("archive.r00"), Some("RAR"));
    }

    #[test]
    fn test_detect_container_type_vdi() {
        assert_eq!(detect_container_type_by_extension("disk.vdi"), Some("VDI"));
    }

    #[test]
    fn test_detect_container_type_sparse() {
        assert_eq!(detect_container_type_by_extension("disk.sparsebundle"), Some("Apple Sparse Image"));
        assert_eq!(detect_container_type_by_extension("disk.sparseimage"), Some("Apple Sparse Image"));
    }

    #[test]
    fn test_detect_container_type_smart() {
        assert_eq!(detect_container_type_by_extension("image.s01"), Some("SMART"));
        assert_eq!(detect_container_type_by_extension("image.s02"), Some("SMART"));
    }

    #[test]
    fn test_detect_container_type_bin_cue() {
        // .bin is detected as Raw Image by centralized format detection
        // .cue is detected as BIN/CUE
        assert_eq!(detect_container_type_by_extension("disc.bin"), Some("Raw Image"));
        assert_eq!(detect_container_type_by_extension("disc.cue"), Some("BIN/CUE"));
    }

    #[test]
    fn test_detect_container_type_ufdx_returns_none() {
        // UFDX files are collection indexes, not evidence containers
        assert_eq!(detect_container_type_by_extension("collection.ufdx"), None);
    }

    #[test]
    fn test_detect_container_type_unknown() {
        assert_eq!(detect_container_type_by_extension("document.pdf"), None);
        assert_eq!(detect_container_type_by_extension("image.png"), None);
        assert_eq!(detect_container_type_by_extension("data.txt"), None);
    }

    #[test]
    fn test_detect_container_type_numbered_segment() {
        // Numbered segments like .001, .002 should be detected as raw images
        assert_eq!(detect_container_type_by_extension("disk.001"), Some("Raw Image"));
    }

    #[test]
    fn test_detect_container_type_logical_tar() {
        // TAR files with "logical" in name get special label
        assert_eq!(detect_container_type_by_extension("evidence-logical.tar"), Some("TAR (Logical)"));
    }

    // ==================== scan_directory tests ====================

    #[test]
    fn test_scan_directory_nonexistent() {
        let result = scan_directory("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_directory_not_a_directory() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("somefile.txt");
        File::create(&file_path).unwrap();
        
        let result = scan_directory(file_path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_directory_empty() {
        let temp = TempDir::new().unwrap();
        let result = scan_directory(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_scan_directory_finds_e01() {
        let temp = TempDir::new().unwrap();
        let e01_path = temp.path().join("evidence.E01");
        let mut file = File::create(&e01_path).unwrap();
        file.write_all(b"dummy content").unwrap();
        
        let result = scan_directory(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].container_type, "EnCase (E01)");
    }

    #[test]
    fn test_scan_directory_skips_dot_underscore_files() {
        let temp = TempDir::new().unwrap();
        let resource_fork = temp.path().join("._evidence.E01");
        File::create(&resource_fork).unwrap();
        
        let result = scan_directory(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_scan_directory_multiple_files() {
        let temp = TempDir::new().unwrap();
        
        let e01 = temp.path().join("evidence.E01");
        let ad1 = temp.path().join("image.ad1");
        let txt = temp.path().join("readme.txt");
        
        File::create(&e01).unwrap().write_all(b"E01 data").unwrap();
        File::create(&ad1).unwrap().write_all(b"AD1 data").unwrap();
        File::create(&txt).unwrap().write_all(b"readme").unwrap();
        
        let result = scan_directory(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let files = result.unwrap();
        
        // Should find E01 and AD1, not the txt file
        assert_eq!(files.len(), 2);
        let types: Vec<&str> = files.iter().map(|f| f.container_type.as_str()).collect();
        assert!(types.contains(&"EnCase (E01)"));
        assert!(types.contains(&"AD1"));
    }

    // ==================== scan_directory_recursive tests ====================

    #[test]
    fn test_scan_directory_recursive_finds_nested() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        
        let root_file = temp.path().join("root.E01");
        let nested_file = subdir.join("nested.ad1");
        
        File::create(&root_file).unwrap().write_all(b"E01").unwrap();
        File::create(&nested_file).unwrap().write_all(b"AD1").unwrap();
        
        let result = scan_directory_recursive(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let files = result.unwrap();
        
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scan_directory_non_recursive_ignores_nested() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        
        let root_file = temp.path().join("root.E01");
        let nested_file = subdir.join("nested.ad1");
        
        File::create(&root_file).unwrap().write_all(b"E01").unwrap();
        File::create(&nested_file).unwrap().write_all(b"AD1").unwrap();
        
        let result = scan_directory(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let files = result.unwrap();
        
        // Non-recursive should only find root file
        assert_eq!(files.len(), 1);
    }

    // ==================== scan_directory_streaming tests ====================

    #[test]
    fn test_scan_directory_streaming_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        
        let temp = TempDir::new().unwrap();
        let e01 = temp.path().join("test.E01");
        File::create(&e01).unwrap().write_all(b"data").unwrap();
        
        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_clone = callback_count.clone();
        
        let result = scan_directory_streaming(
            temp.path().to_str().unwrap(),
            false,
            move |_file| {
                callback_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(callback_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_scan_directory_streaming_nonexistent() {
        let result = scan_directory_streaming("/nonexistent/path", false, |_| {});
        assert!(result.is_err());
    }
}
