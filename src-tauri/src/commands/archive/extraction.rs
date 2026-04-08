// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive entry extraction and chunk reading operations.
//!
//! Provides single-entry extraction for nested container support and
//! chunk-based reading for hex viewer integration without full extraction.

use tracing::debug;

use crate::archive;
use crate::common::filesystem::FilesystemDriver;

// =============================================================================
// Helper Functions (testable without Tauri runtime)
// =============================================================================

/// Determine the extraction method for an archive based on file extension.
/// Returns a routing label for extraction dispatch.
#[cfg(test)]
pub(crate) fn classify_archive_extraction(extension: &str) -> &'static str {
    match extension {
        "zip" => "zip-native",
        "7z" | "rar" | "r00" | "r01" | "tar" | "tgz" | "gz" | "bz2" | "xz" => "libarchive",
        _ => "fallback",
    }
}

/// Check if an entry path is a compressed synthetic name that requires
/// listing the archive to find the real entry name.
#[cfg(test)]
pub(crate) fn needs_synthetic_resolution(entry_path: &str) -> bool {
    entry_path.starts_with("(Compressed")
}

fn normalize_filesystem_entry_path(entry_path: &str) -> String {
    if entry_path.starts_with('/') {
        entry_path.to_string()
    } else {
        format!("/{}", entry_path)
    }
}

fn read_dmg_entry_bytes(container_path: &str, entry_path: &str) -> Result<Vec<u8>, String> {
    let dmg = crate::common::filesystem::DmgDriver::open(container_path)
        .map_err(|e| format!("Failed to open DMG: {}", e))?;

    let hfs_idx = dmg
        .find_hfs_partition()
        .ok_or_else(|| "No HFS+ partition found in DMG".to_string())?;
    let device = dmg
        .partition_device(hfs_idx)
        .map_err(|e| format!("Failed to access DMG partition: {}", e))?;
    let partition_size = device.size();
    let hfs = crate::common::filesystem::HfsPlusDriver::new(device, 0, partition_size)
        .map_err(|e| format!("Failed to mount HFS+ filesystem: {}", e))?;

    let normalized_path = normalize_filesystem_entry_path(entry_path);
    let attr = hfs
        .getattr(&normalized_path)
        .map_err(|e| format!("Failed to stat DMG entry '{}': {}", normalized_path, e))?;

    if attr.is_directory {
        return Err(format!(
            "Cannot read directory entry from DMG: {}",
            normalized_path
        ));
    }

    let read_size = usize::try_from(attr.size)
        .map_err(|_| format!("DMG entry is too large to read: {}", normalized_path))?;

    hfs.read(&normalized_path, 0, read_size)
        .map_err(|e| format!("Failed to read DMG entry '{}': {}", normalized_path, e))
}

pub(crate) fn read_archive_entry_bytes(
    container_path: &str,
    entry_path: &str,
) -> Result<Vec<u8>, String> {
    let extension = std::path::Path::new(container_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if entry_path.starts_with("(Compressed") {
        let entries = archive::libarchive_list_all(container_path)
            .map_err(|e| format!("Failed to list compressed file entries: {}", e))?;

        let real_path = entries
            .first()
            .map(|entry| entry.path.clone())
            .ok_or_else(|| "Compressed file contains no entries".to_string())?;

        return archive::libarchive_read_file(container_path, &real_path)
            .or_else(|e| {
                debug!(
                    "libarchive failed for compressed entry, trying native: {}",
                    e
                );
                archive::read_entry_native(container_path, &real_path)
            })
            .map_err(|e| format!("Failed to decompress file: {}", e));
    }

    if extension == "dmg" {
        return read_dmg_entry_bytes(container_path, entry_path);
    }

    archive::libarchive_read_file(container_path, entry_path)
        .or_else(|e| {
            debug!(
                "libarchive failed for archive entry, trying native fallback: {}",
                e
            );
            archive::read_entry_native(container_path, entry_path)
        })
        .map_err(|e| format!("Failed to read archive entry '{}': {}", entry_path, e))
}

fn slice_chunk(data: &[u8], offset: u64, size: u64) -> Vec<u8> {
    let Some(start) = usize::try_from(offset).ok() else {
        return Vec::new();
    };
    if start >= data.len() {
        return Vec::new();
    }

    let requested = usize::try_from(size).unwrap_or(usize::MAX);
    let end = start.saturating_add(requested).min(data.len());
    data[start..end].to_vec()
}

/// Extract a single entry from an archive to a temp file
///
/// Used for opening nested containers (containers inside archives)
/// Returns the path to the extracted temp file
#[tauri::command]
pub async fn archive_extract_entry(
    #[allow(non_snake_case)] containerPath: String,
    #[allow(non_snake_case)] entryPath: String,
) -> Result<String, String> {
    debug!(
        "archive_extract_entry: container={}, entry={}",
        containerPath, entryPath
    );

    tauri::async_runtime::spawn_blocking(move || {
        let container_path = std::path::Path::new(&containerPath);
        let extension = container_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        // Create temp directory for extracted file
        let temp_dir = crate::commands::portable::portable_temp_dir()
            .join(crate::app_paths::NESTED_TEMP_DIR_NAME);
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        // Generate output filename from entry path (preserving extension)
        let entry_filename = std::path::Path::new(&entryPath)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("extracted");

        // Add unique prefix to avoid collisions
        let unique_name = format!(
            "{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            entry_filename
        );

        let output_path = temp_dir.join(&unique_name);
        let output_str = output_path.to_string_lossy().to_string();

        match extension.as_str() {
            "zip" => {
                archive::extract_zip_entry(&containerPath, &entryPath, &output_str)
                    .map_err(|e| e.to_string())?;
                Ok(output_str)
            }
            _ => {
                let data = read_archive_entry_bytes(&containerPath, &entryPath)?;
                std::fs::write(&output_path, &data)
                    .map_err(|e| format!("Failed to write extracted file: {}", e))?;
                Ok(output_str)
            }
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Read a chunk of bytes from an entry within an archive
///
/// Used for HexViewer to show archive file contents without extracting the entire file.
/// For most archive formats, we need to decompress the entire entry first, then slice it.
/// The data is cached in memory to allow efficient scrolling through large files.
#[tauri::command]
pub async fn archive_read_entry_chunk(
    #[allow(non_snake_case)] containerPath: String,
    #[allow(non_snake_case)] entryPath: String,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, String> {
    debug!(
        "archive_read_entry_chunk: container={}, entry='{}', offset={}, size={}",
        containerPath, entryPath, offset, size
    );

    tauri::async_runtime::spawn_blocking(move || {
        // Read the full entry content using libarchive
        // Note: Most archive formats require sequential decompression,
        // so we read the whole file and slice it. For very large files,
        // consider extracting to temp and memory-mapping.
        let data = read_archive_entry_bytes(&containerPath, &entryPath).inspect_err(|_| {
            if let Ok(entries) = archive::libarchive_list_all(&containerPath) {
                let paths: Vec<_> = entries.iter().take(10).map(|entry| entry.path.as_str()).collect();
                debug!(
                    "archive_read_entry_chunk: Entry '{}' not found. First 10 entries in archive: {:?}",
                    entryPath,
                    paths
                );
            }
        })?;

        let total_size = data.len() as u64;
        debug!("archive_read_entry_chunk: Read {} bytes from entry", total_size);

        Ok(slice_chunk(&data, offset, size))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== classify_archive_extraction tests ====================

    #[test]
    fn test_classify_zip() {
        assert_eq!(classify_archive_extraction("zip"), "zip-native");
    }

    #[test]
    fn test_classify_7z() {
        assert_eq!(classify_archive_extraction("7z"), "libarchive");
    }

    #[test]
    fn test_classify_rar() {
        assert_eq!(classify_archive_extraction("rar"), "libarchive");
    }

    #[test]
    fn test_classify_tar_variants() {
        assert_eq!(classify_archive_extraction("tar"), "libarchive");
        assert_eq!(classify_archive_extraction("tgz"), "libarchive");
        assert_eq!(classify_archive_extraction("gz"), "libarchive");
        assert_eq!(classify_archive_extraction("bz2"), "libarchive");
        assert_eq!(classify_archive_extraction("xz"), "libarchive");
    }

    #[test]
    fn test_classify_unknown() {
        assert_eq!(classify_archive_extraction("iso"), "fallback");
        assert_eq!(classify_archive_extraction(""), "fallback");
    }

    // ==================== needs_synthetic_resolution tests ====================

    #[test]
    fn test_synthetic_compressed_formats() {
        assert!(needs_synthetic_resolution("(Compressed BZ2 file)"));
        assert!(needs_synthetic_resolution("(Compressed GZ file)"));
        assert!(needs_synthetic_resolution("(Compressed XZ file)"));
    }

    #[test]
    fn test_synthetic_regular_path() {
        assert!(!needs_synthetic_resolution("some/path/file.txt"));
        assert!(!needs_synthetic_resolution(""));
        assert!(!needs_synthetic_resolution("Compressed file.bz2"));
    }

    #[test]
    fn test_slice_chunk_valid_range() {
        assert_eq!(slice_chunk(b"abcdef", 2, 3), b"cde");
    }

    #[test]
    fn test_slice_chunk_out_of_bounds_returns_empty() {
        assert!(slice_chunk(b"abcdef", 99, 10).is_empty());
    }

    #[test]
    fn test_slice_chunk_huge_size_saturates_to_end() {
        assert_eq!(slice_chunk(b"abcdef", 4, u64::MAX), b"ef");
    }
}
