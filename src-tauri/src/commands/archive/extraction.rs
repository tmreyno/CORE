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
        let temp_dir = std::env::temp_dir().join("core-ffx-nested");
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
            "7z" | "rar" | "r00" | "r01" | "tar" | "tgz" | "gz" | "bz2" | "xz" => {
                // Use libarchive backend for all non-ZIP formats
                let data = archive::libarchive_read_file(&containerPath, &entryPath)
                    .map_err(|e| format!("Failed to extract entry from archive: {}", e))?;
                std::fs::write(&output_path, &data)
                    .map_err(|e| format!("Failed to write extracted file: {}", e))?;
                Ok(output_str)
            }
            _ => {
                // Try ZIP as fallback
                match archive::extract_zip_entry(&containerPath, &entryPath, &output_str) {
                    Ok(_) => Ok(output_str),
                    Err(e) => Err(format!(
                        "Extraction not supported for this archive type: {}",
                        e
                    )),
                }
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
        let data = if entryPath.starts_with("(Compressed") {
            // Single-file compressed format (BZ2, GZ, XZ, etc.) with synthetic entry name.
            // Read the first (only) entry using its real name from libarchive.
            let entries = archive::libarchive_list_all(&containerPath)
                .map_err(|e| format!("Failed to list compressed file entries: {}", e))?;
            let real_path = entries.first()
                .map(|e| e.path.clone())
                .ok_or_else(|| "Compressed file contains no entries".to_string())?;
            archive::libarchive_read_file(&containerPath, &real_path)
                .map_err(|e| format!("Failed to decompress file: {}", e))?
        } else {
            archive::libarchive_read_file(&containerPath, &entryPath)
                .map_err(|e| {
                    // Log available entries for debugging
                    if let Ok(entries) = archive::libarchive_list_all(&containerPath) {
                        let paths: Vec<_> = entries.iter().take(10).map(|e| e.path.as_str()).collect();
                        debug!("archive_read_entry_chunk: Entry '{}' not found. First 10 entries in archive: {:?}",
                               entryPath, paths);
                    }
                    format!("Failed to read archive entry '{}': {}", entryPath, e)
                })?
        };

        let total_size = data.len() as u64;
        debug!("archive_read_entry_chunk: Read {} bytes from entry", total_size);

        // Bounds checking
        if offset >= total_size {
            return Ok(Vec::new());
        }

        let start = offset as usize;
        let end = std::cmp::min(start + size as usize, data.len());

        Ok(data[start..end].to_vec())
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
}
