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
use crate::common::vfs::VirtualFileSystem;

const ARCHIVE_RANGE_MAX_BYTES: usize = 16 * 1024 * 1024;
const ARCHIVE_FULL_READ_MAX_BYTES: u64 = 256 * 1024 * 1024;

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

fn normalize_archive_entry_path(entry_path: &str) -> String {
    entry_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string()
}

fn archive_entry_path_matches(candidate: &str, requested: &str) -> bool {
    let candidate = normalize_archive_entry_path(candidate);
    let requested = normalize_archive_entry_path(requested);
    candidate == requested || candidate.trim_end_matches('/') == requested
}

fn checked_archive_full_read_size(size: u64, context: &str) -> Result<usize, String> {
    if size > ARCHIVE_FULL_READ_MAX_BYTES {
        return Err(format!(
            "{context} exceeds full archive entry read limit: {size} bytes > {ARCHIVE_FULL_READ_MAX_BYTES} bytes"
        ));
    }

    usize::try_from(size).map_err(|_| format!("{context} is too large to read"))
}

fn ensure_archive_full_read_len(len: usize, context: &str) -> Result<(), String> {
    if len as u64 > ARCHIVE_FULL_READ_MAX_BYTES {
        return Err(format!(
            "{context} exceeded full archive entry read limit after read: {len} bytes > {ARCHIVE_FULL_READ_MAX_BYTES} bytes"
        ));
    }
    Ok(())
}

fn archive_entry_size_from_list(
    entries: &[archive::ArchiveEntry],
    entry_path: &str,
) -> Result<u64, String> {
    entries
        .iter()
        .find(|entry| archive_entry_path_matches(&entry.path, entry_path))
        .map(|entry| entry.size)
        .ok_or_else(|| format!("Entry not found in archive: {entry_path}"))
}

fn libarchive_entry_size_from_list(
    entries: &[archive::ArchiveEntryInfo],
    entry_path: &str,
) -> Result<u64, String> {
    entries
        .iter()
        .find(|entry| archive_entry_path_matches(&entry.path, entry_path))
        .map(|entry| entry.size)
        .ok_or_else(|| format!("Entry not found in archive: {entry_path}"))
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

    let read_size = checked_archive_full_read_size(attr.size, "DMG entry")?;

    let data = hfs
        .read(&normalized_path, 0, read_size)
        .map_err(|e| format!("Failed to read DMG entry '{}': {}", normalized_path, e))?;
    ensure_archive_full_read_len(data.len(), "DMG entry")?;
    Ok(data)
}

fn read_dmg_entry_range(
    container_path: &str,
    entry_path: &str,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, String> {
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

    let read_size = bounded_archive_read_size(attr.size, offset, size)?;
    if read_size == 0 {
        return Ok(Vec::new());
    }

    let data = hfs
        .read(&normalized_path, offset, read_size)
        .map_err(|e| format!("Failed to read DMG entry '{}': {}", normalized_path, e))?;
    if data.len() > read_size {
        return Err(format!(
            "DMG filesystem returned too many bytes for '{}': requested {read_size}, received {}",
            normalized_path,
            data.len()
        ));
    }
    ensure_archive_range_read_len(data.len(), read_size, "DMG filesystem", &normalized_path)?;
    Ok(data)
}

fn checked_archive_range_request_size(size: usize) -> Result<usize, String> {
    if size > ARCHIVE_RANGE_MAX_BYTES {
        return Err(format!(
            "Archive entry chunk request is too large: {size} bytes > {ARCHIVE_RANGE_MAX_BYTES} bytes"
        ));
    }
    Ok(size)
}

fn bounded_archive_read_size(total_size: u64, offset: u64, size: usize) -> Result<usize, String> {
    let size = checked_archive_range_request_size(size)?;
    if size == 0 || offset == total_size {
        return Ok(0);
    }
    if offset > total_size {
        return Err(format!(
            "Archive entry chunk offset is beyond EOF: offset {offset} > size {total_size}"
        ));
    }
    let available = total_size - offset;
    usize::try_from(available.min(size as u64))
        .map_err(|_| "Archive entry range is too large to read".to_string())
}

fn ensure_archive_range_read_len(
    actual: usize,
    expected: usize,
    context: &str,
    entry_path: &str,
) -> Result<(), String> {
    if actual == expected {
        return Ok(());
    }

    if actual < expected {
        Err(format!(
            "{context} returned incomplete data for '{entry_path}': expected {expected} bytes, received {actual}"
        ))
    } else {
        Err(format!(
            "{context} returned too many bytes for '{entry_path}': expected {expected} bytes, received {actual}"
        ))
    }
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
        let entry_size = libarchive_entry_size_from_list(&entries, &real_path)?;
        checked_archive_full_read_size(entry_size, "Compressed archive entry")?;

        let data = archive::libarchive_read_file(container_path, &real_path)
            .or_else(|e| {
                debug!(
                    "libarchive failed for compressed entry, trying native: {}",
                    e
                );
                archive::read_entry_native(container_path, &real_path)
            })
            .map_err(|e| format!("Failed to decompress file: {}", e))?;
        ensure_archive_full_read_len(data.len(), "Compressed archive entry")?;
        return Ok(data);
    }

    if extension == "dmg" {
        return read_dmg_entry_bytes(container_path, entry_path);
    }

    let data = archive::libarchive_read_file(container_path, entry_path)
        .or_else(|e| {
            debug!(
                "libarchive failed for archive entry, trying native fallback: {}",
                e
            );
            archive::read_entry_native(container_path, entry_path)
        })
        .map_err(|e| format!("Failed to read archive entry '{}': {}", entry_path, e))?;
    ensure_archive_full_read_len(data.len(), "Archive entry")?;
    Ok(data)
}

fn checked_zip_extract_entry_size(container_path: &str, entry_path: &str) -> Result<(), String> {
    let entries = archive::list_zip_entries(container_path)
        .map_err(|e| format!("Failed to list ZIP entries before extraction: {e}"))?;
    let size = archive_entry_size_from_list(&entries, entry_path)?;
    checked_archive_full_read_size(size, "ZIP entry")?;
    Ok(())
}

pub(crate) fn read_archive_entry_range(
    container_path: &str,
    entry_path: &str,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, String> {
    checked_archive_range_request_size(size)?;
    let extension = std::path::Path::new(container_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if entry_path.starts_with("(Compressed") {
        let data = read_archive_entry_bytes(container_path, entry_path)?;
        return slice_chunk(&data, offset, size as u64);
    }

    if extension == "dmg" {
        return read_dmg_entry_range(container_path, entry_path, offset, size);
    }

    if matches!(extension.as_str(), "zip" | "zip64") {
        let vfs = archive::ArchiveVfs::open(container_path)
            .map_err(|e| format!("Failed to open archive VFS: {}", e))?;
        let total_size = vfs
            .file_size(entry_path)
            .map_err(|e| format!("Failed to inspect archive entry '{}': {}", entry_path, e))?;
        let read_size = bounded_archive_read_size(total_size, offset, size)?;
        if read_size == 0 {
            return Ok(Vec::new());
        }
        let data = vfs
            .read(entry_path, offset, read_size)
            .map_err(|e| format!("Failed to read archive entry '{}': {}", entry_path, e))?;
        if data.len() > read_size {
            return Err(format!(
                "Archive VFS returned too many bytes for '{}': requested {read_size}, received {}",
                entry_path,
                data.len()
            ));
        }
        ensure_archive_range_read_len(data.len(), read_size, "Archive VFS", entry_path)?;
        return Ok(data);
    }

    let data = read_archive_entry_bytes(container_path, entry_path)?;
    slice_chunk(&data, offset, size as u64)
}

fn slice_chunk(data: &[u8], offset: u64, size: u64) -> Result<Vec<u8>, String> {
    let total_size = data.len() as u64;
    if size == 0 || offset == total_size {
        return Ok(Vec::new());
    }
    if offset > total_size {
        return Err(format!(
            "Archive entry chunk offset is beyond EOF: offset {offset} > size {total_size}"
        ));
    }
    let Some(start) = usize::try_from(offset).ok() else {
        return Err("Archive entry chunk offset is too large".to_string());
    };

    let requested = usize::try_from(size).unwrap_or(usize::MAX);
    let end = start.saturating_add(requested).min(data.len());
    Ok(data[start..end].to_vec())
}

fn archive_entry_temp_filename(entry_path: &str) -> String {
    let leaf = entry_path
        .rsplit(['/', '\\'])
        .find(|component| !component.is_empty())
        .unwrap_or("extracted");
    let sanitized = crate::common::sanitize_filename(leaf);
    if sanitized.is_empty() {
        "extracted".to_string()
    } else {
        sanitized
    }
}

fn archive_entry_unique_temp_name(timestamp_ms: u128, entry_path: &str) -> String {
    format!(
        "{}_{}",
        timestamp_ms,
        archive_entry_temp_filename(entry_path)
    )
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

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let unique_name = archive_entry_unique_temp_name(timestamp_ms, &entryPath);

        let output_path = temp_dir.join(&unique_name);
        let output_str = output_path.to_string_lossy().to_string();

        match extension.as_str() {
            "zip" => {
                checked_zip_extract_entry_size(&containerPath, &entryPath)?;
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
        let size = usize::try_from(size)
            .map_err(|_| "Archive entry chunk size is too large to read".to_string())?;
        let data = read_archive_entry_range(&containerPath, &entryPath, offset, size).inspect_err(|_| {
            if let Ok(entries) = archive::libarchive_list_all(&containerPath) {
                let paths: Vec<_> = entries.iter().take(10).map(|entry| entry.path.as_str()).collect();
                debug!(
                    "archive_read_entry_chunk: Entry '{}' not found. First 10 entries in archive: {:?}",
                    entryPath,
                    paths
                );
            }
        })?;

        debug!("archive_read_entry_chunk: Read {} bytes from entry", data.len());

        Ok(data)
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
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    fn write_zip_file(path: &std::path::Path, entry_path: &str, bytes: &[u8]) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file(entry_path, options).unwrap();
        zip.write_all(bytes).unwrap();
        zip.finish().unwrap();
    }

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
    fn archive_entry_path_matches_normalizes_slashes_and_edges() {
        assert!(archive_entry_path_matches(
            r"folder\subfolder\file.txt",
            "/folder/subfolder/file.txt"
        ));
        assert!(archive_entry_path_matches(
            "folder/file.txt/",
            "folder/file.txt"
        ));
        assert!(!archive_entry_path_matches(
            "folder/file-a.txt",
            "folder/file-b.txt"
        ));
    }

    #[test]
    fn checked_archive_full_read_size_allows_limit() {
        assert_eq!(
            checked_archive_full_read_size(ARCHIVE_FULL_READ_MAX_BYTES, "archive entry").unwrap(),
            ARCHIVE_FULL_READ_MAX_BYTES as usize
        );
    }

    #[test]
    fn checked_archive_full_read_size_rejects_oversized_entry() {
        let err = checked_archive_full_read_size(ARCHIVE_FULL_READ_MAX_BYTES + 1, "archive entry")
            .unwrap_err();

        assert!(err.contains("archive entry exceeds full archive entry read limit"));
    }

    #[test]
    fn ensure_archive_full_read_len_rejects_oversized_result() {
        let oversized = usize::try_from(ARCHIVE_FULL_READ_MAX_BYTES)
            .unwrap()
            .saturating_add(1);
        let err = ensure_archive_full_read_len(oversized, "archive entry").unwrap_err();

        assert!(err.contains("archive entry exceeded full archive entry read limit after read"));
    }

    #[test]
    fn archive_entry_size_from_list_normalizes_requested_path() {
        let entries = vec![archive::ArchiveEntry {
            index: 0,
            path: "folder/file.txt".to_string(),
            is_directory: false,
            size: 123,
            compressed_size: 45,
            crc32: 0,
            compression_method: "Deflated".to_string(),
            last_modified: String::new(),
        }];

        assert_eq!(
            archive_entry_size_from_list(&entries, r"\folder\file.txt").unwrap(),
            123
        );
    }

    #[test]
    fn libarchive_entry_size_from_list_normalizes_requested_path() {
        let entries = vec![archive::ArchiveEntryInfo {
            path: "folder/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
            size: 456,
            parent: "folder".to_string(),
            index: 0,
            is_encrypted: false,
            mtime: None,
        }];

        assert_eq!(
            libarchive_entry_size_from_list(&entries, "/folder/file.txt").unwrap(),
            456
        );
    }

    #[test]
    fn test_slice_chunk_valid_range() {
        assert_eq!(slice_chunk(b"abcdef", 2, 3).unwrap(), b"cde");
    }

    #[test]
    fn test_slice_chunk_allows_offset_at_eof() {
        assert!(slice_chunk(b"abcdef", 6, 10).unwrap().is_empty());
    }

    #[test]
    fn test_slice_chunk_rejects_offset_past_eof() {
        let err = slice_chunk(b"abcdef", 99, 10).unwrap_err();

        assert!(err.contains("offset 99 > size 6"), "unexpected: {err}");
    }

    #[test]
    fn test_slice_chunk_huge_size_saturates_to_end() {
        assert_eq!(slice_chunk(b"abcdef", 4, u64::MAX).unwrap(), b"ef");
    }

    #[test]
    fn checked_archive_range_request_size_allows_limit() {
        assert_eq!(
            checked_archive_range_request_size(ARCHIVE_RANGE_MAX_BYTES).unwrap(),
            ARCHIVE_RANGE_MAX_BYTES
        );
    }

    #[test]
    fn checked_archive_range_request_size_rejects_oversized_request() {
        let err = checked_archive_range_request_size(ARCHIVE_RANGE_MAX_BYTES + 1).unwrap_err();

        assert!(err.contains("Archive entry chunk request is too large"));
    }

    #[test]
    fn bounded_archive_read_size_clamps_to_remaining_entry_bytes() {
        assert_eq!(
            bounded_archive_read_size(100, 90, ARCHIVE_RANGE_MAX_BYTES).unwrap(),
            10
        );
    }

    #[test]
    fn bounded_archive_read_size_returns_zero_at_eof() {
        assert_eq!(bounded_archive_read_size(100, 100, 64).unwrap(), 0);
    }

    #[test]
    fn bounded_archive_read_size_rejects_offset_past_eof() {
        let err = bounded_archive_read_size(100, 150, 64).unwrap_err();

        assert!(err.contains("offset 150 > size 100"), "unexpected: {err}");
    }

    #[test]
    fn ensure_archive_range_read_len_rejects_short_read() {
        let err = ensure_archive_range_read_len(2, 3, "Archive VFS", "file.bin").unwrap_err();

        assert!(err.contains("incomplete data"), "unexpected: {err}");
        assert!(err.contains("expected 3 bytes"), "unexpected: {err}");
        assert!(err.contains("received 2"), "unexpected: {err}");
    }

    #[test]
    fn archive_entry_temp_filename_uses_windows_leaf_name() {
        assert_eq!(
            archive_entry_temp_filename(r"folder\subfolder\evidence.E01"),
            "evidence.E01"
        );
    }

    #[test]
    fn archive_entry_temp_filename_sanitizes_unsafe_leaf() {
        assert_eq!(
            archive_entry_temp_filename(r#"folder/evil<>:"|?*.zip"#),
            "evil.zip"
        );
    }

    #[test]
    fn archive_entry_unique_temp_name_falls_back_for_empty_leaf() {
        assert_eq!(archive_entry_unique_temp_name(42, "..."), "42_extracted");
    }

    #[test]
    fn read_archive_entry_range_reads_zip_entry_range() {
        let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        write_zip_file(tmp.path(), "nested/file.txt", b"abcdef");

        let path = tmp.path().to_string_lossy();

        assert_eq!(
            read_archive_entry_range(&path, "nested/file.txt", 2, 3).unwrap(),
            b"cde"
        );
        assert!(read_archive_entry_range(&path, "nested/file.txt", 6, 3)
            .unwrap()
            .is_empty());
        let err = read_archive_entry_range(&path, "nested/file.txt", 7, 3).unwrap_err();
        assert!(err.contains("offset 7 > size 6"), "unexpected: {err}");
    }

    #[test]
    fn checked_zip_extract_entry_size_accepts_small_zip_entry() {
        let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        write_zip_file(tmp.path(), "nested/file.txt", b"abcdef");
        let path = tmp.path().to_string_lossy();

        checked_zip_extract_entry_size(&path, "/nested/file.txt").unwrap();
    }

    #[test]
    fn read_archive_entry_range_rejects_oversized_request_before_open() {
        let err = read_archive_entry_range(
            "/tmp/missing.zip",
            "nested/file.txt",
            0,
            ARCHIVE_RANGE_MAX_BYTES + 1,
        )
        .unwrap_err();

        assert!(err.contains("Archive entry chunk request is too large"));
    }
}
