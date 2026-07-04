// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive extraction support
//!
//! Provides extraction capabilities for ZIP archives. Other formats (7z, RAR)
//! require external tools for extraction.
//!
//! ## Forensic Considerations
//!
//! - All extraction is read-only and doesn't modify source archives
//! - Timestamps are preserved where possible
//! - Path traversal attacks are prevented
//! - CRC verification is performed during extraction

use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tracing::debug;
use zip::ZipArchive;

use super::types::ArchiveFormat;
use ffx_common::path_security::{safe_join, sanitize_filename};
use ffx_errors::ContainerError;

fn add_extracted_bytes(total: u64, bytes: u64, path: &str) -> Result<u64, String> {
    total.checked_add(bytes).ok_or_else(|| {
        format!(
            "Archive extraction byte count overflow while adding {} bytes from {} to {} bytes",
            bytes, path, total
        )
    })
}

fn archive_entry_output_path(
    output_path: &Path,
    entry_name: &str,
    entry_path: &Path,
) -> Result<PathBuf, ExtractError> {
    let sanitized_name = sanitize_filename(&entry_path.to_string_lossy());
    if sanitized_name.is_empty() {
        return Err(ExtractError {
            path: entry_name.to_string(),
            error: "Invalid empty extraction path".to_string(),
        });
    }

    safe_join(output_path, &sanitized_name).map_err(|e| ExtractError {
        path: entry_name.to_string(),
        error: format!("Path traversal detected: {:?}", e),
    })
}

/// Extract result containing extraction statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractResult {
    /// Total files extracted
    pub files_extracted: u64,
    /// Total directories created
    pub directories_created: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Files that failed to extract
    pub failed_files: Vec<ExtractError>,
    /// Whether extraction completed successfully
    pub success: bool,
}

/// Individual file extraction error
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractError {
    /// Path within the archive
    pub path: String,
    /// Error message
    pub error: String,
}

/// Extract a ZIP archive to the specified output directory
pub fn extract_zip(archive_path: &str, output_dir: &str) -> Result<ExtractResult, ContainerError> {
    let archive_path = Path::new(archive_path);
    let output_path = Path::new(output_dir);

    if !archive_path.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive not found: {}",
            archive_path.display()
        )));
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut result = ExtractResult {
        files_extracted: 0,
        directories_created: 0,
        bytes_written: 0,
        failed_files: vec![],
        success: true,
    };

    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(e) => {
                result.failed_files.push(ExtractError {
                    path: format!("entry {}", i),
                    error: e.to_string(),
                });
                continue;
            }
        };

        let entry_path = match entry.enclosed_name() {
            Some(path) => path.to_owned(),
            None => {
                result.failed_files.push(ExtractError {
                    path: entry.name().to_string(),
                    error: "Invalid or unsafe path".to_string(),
                });
                continue;
            }
        };

        let dest_path = match archive_entry_output_path(output_path, entry.name(), &entry_path) {
            Ok(p) => p,
            Err(error) => {
                result.failed_files.push(error);
                continue;
            }
        };

        if entry.is_dir() {
            if let Err(e) = fs::create_dir_all(&dest_path) {
                result.failed_files.push(ExtractError {
                    path: entry.name().to_string(),
                    error: e.to_string(),
                });
                continue;
            }
            result.directories_created += 1;
            debug!(path = %dest_path.display(), "Created directory");
        } else {
            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        result.failed_files.push(ExtractError {
                            path: entry.name().to_string(),
                            error: format!("Failed to create parent directory: {}", e),
                        });
                        continue;
                    }
                }
            }

            match copy_entry_to_file(&mut entry, &dest_path, |_| {}) {
                Ok(bytes_written) => {
                    match add_extracted_bytes(result.bytes_written, bytes_written, entry.name()) {
                        Ok(total) => {
                            result.bytes_written = total;
                            result.files_extracted += 1;
                            debug!(path = %dest_path.display(), bytes = bytes_written, "Extracted file");
                        }
                        Err(error) => {
                            let _ = fs::remove_file(&dest_path);
                            result.failed_files.push(ExtractError {
                                path: entry.name().to_string(),
                                error,
                            });
                        }
                    }
                }
                Err(error) => {
                    result.failed_files.push(ExtractError {
                        path: entry.name().to_string(),
                        error,
                    });
                }
            }
        }
    }

    result.success = result.failed_files.is_empty();

    debug!(
        files = result.files_extracted,
        dirs = result.directories_created,
        bytes = result.bytes_written,
        failed = result.failed_files.len(),
        "ZIP extraction complete"
    );

    Ok(result)
}

/// Extract a ZIP archive with progress callback
pub fn extract_zip_with_progress<F>(
    archive_path: &str,
    output_dir: &str,
    mut progress_callback: F,
) -> Result<ExtractResult, ContainerError>
where
    F: FnMut(u64, u64),
{
    let archive_path = Path::new(archive_path);
    let output_path = Path::new(output_dir);

    if !archive_path.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive not found: {}",
            archive_path.display()
        )));
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Calculate total uncompressed size for progress
    let total_size: u64 = {
        let mut total = 0u64;
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                total = add_extracted_bytes(total, entry.size(), entry.name())?;
            }
        }
        total
    };

    let mut result = ExtractResult {
        files_extracted: 0,
        directories_created: 0,
        bytes_written: 0,
        failed_files: vec![],
        success: true,
    };

    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(e) => {
                result.failed_files.push(ExtractError {
                    path: format!("entry {}", i),
                    error: e.to_string(),
                });
                continue;
            }
        };

        let entry_path = match entry.enclosed_name() {
            Some(path) => path.to_owned(),
            None => {
                result.failed_files.push(ExtractError {
                    path: entry.name().to_string(),
                    error: "Invalid or unsafe path".to_string(),
                });
                continue;
            }
        };

        let dest_path = match archive_entry_output_path(output_path, entry.name(), &entry_path) {
            Ok(p) => p,
            Err(error) => {
                result.failed_files.push(error);
                continue;
            }
        };

        if entry.is_dir() {
            if let Err(e) = fs::create_dir_all(&dest_path) {
                result.failed_files.push(ExtractError {
                    path: entry.name().to_string(),
                    error: e.to_string(),
                });
                continue;
            }
            result.directories_created += 1;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        result.failed_files.push(ExtractError {
                            path: entry.name().to_string(),
                            error: format!("Failed to create parent directory: {}", e),
                        });
                        continue;
                    }
                }
            }

            let committed_bytes = result.bytes_written;
            let entry_name = entry.name().to_string();
            let mut progress_error = None;
            match copy_entry_to_file(&mut entry, &dest_path, |chunk_bytes| {
                if progress_error.is_some() {
                    return;
                }

                match add_extracted_bytes(committed_bytes, chunk_bytes, &entry_name) {
                    Ok(total) => progress_callback(total, total_size),
                    Err(error) => progress_error = Some(error),
                }
            }) {
                Ok(bytes_for_file) => {
                    if let Some(error) = progress_error {
                        let _ = fs::remove_file(&dest_path);
                        result.failed_files.push(ExtractError {
                            path: entry.name().to_string(),
                            error,
                        });
                        continue;
                    }

                    match add_extracted_bytes(result.bytes_written, bytes_for_file, entry.name()) {
                        Ok(total) => {
                            result.bytes_written = total;
                            result.files_extracted += 1;
                            debug!(path = %dest_path.display(), bytes = bytes_for_file, "Extracted file");
                        }
                        Err(error) => {
                            let _ = fs::remove_file(&dest_path);
                            result.failed_files.push(ExtractError {
                                path: entry.name().to_string(),
                                error,
                            });
                        }
                    }
                }
                Err(error) => {
                    result.failed_files.push(ExtractError {
                        path: entry.name().to_string(),
                        error,
                    });
                }
            }
        }
    }

    result.success = result.failed_files.is_empty();
    Ok(result)
}

/// Extract an archive with progress callback
pub fn extract_with_progress<F>(
    archive_path: &str,
    output_dir: &str,
    progress_callback: F,
) -> Result<ExtractResult, ContainerError>
where
    F: FnMut(u64, u64),
{
    let format = super::detection::detect_archive_format(archive_path)?
        .ok_or_else(|| format!("Unable to detect archive format: {}", archive_path))?;

    match format {
        ArchiveFormat::Zip | ArchiveFormat::Zip64 => {
            extract_zip_with_progress(archive_path, output_dir, progress_callback)
        }
        ArchiveFormat::SevenZip => Err(ContainerError::UnsupportedOperation(
            "7-Zip extraction requires external 7z tool. Use: 7z x archive.7z -ooutput_dir"
                .to_string(),
        )),
        ArchiveFormat::Rar4 | ArchiveFormat::Rar5 => Err(ContainerError::UnsupportedOperation(
            "RAR extraction requires external unrar tool. Use: unrar x archive.rar output_dir"
                .to_string(),
        )),
        ArchiveFormat::Gzip => extract_gzip(archive_path, output_dir),
        _ => Err(ContainerError::UnsupportedOperation(format!(
            "Extraction not supported for format: {}",
            format
        ))),
    }
}

/// Extract an archive based on format detection
pub fn extract(archive_path: &str, output_dir: &str) -> Result<ExtractResult, ContainerError> {
    extract_with_progress(archive_path, output_dir, |_, _| {})
}

/// Extract a GZIP file (single file compression)
pub fn extract_gzip(gzip_path: &str, output_dir: &str) -> Result<ExtractResult, ContainerError> {
    use flate2::read::GzDecoder;

    let gzip_path_obj = Path::new(gzip_path);
    let output_path = Path::new(output_dir);

    if !gzip_path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "File not found: {}",
            gzip_path
        )));
    }

    // Create output directory
    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Determine output filename (strip .gz extension)
    let output_name = gzip_path_obj
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());

    let dest_path = output_path.join(&output_name);

    let file = File::open(gzip_path_obj).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut decoder = GzDecoder::new(file);
    let bytes_written = copy_entry_to_file(&mut decoder, &dest_path, |_| {})
        .map_err(|e| format!("Failed to decompress: {}", e))?;

    debug!(
        src = %gzip_path,
        dest = %dest_path.display(),
        bytes = bytes_written,
        "GZIP extraction complete"
    );

    Ok(ExtractResult {
        files_extracted: 1,
        directories_created: 0,
        bytes_written,
        failed_files: vec![],
        success: true,
    })
}

fn copy_entry_to_file<R, F>(
    reader: &mut R,
    dest_path: &Path,
    mut progress_callback: F,
) -> Result<u64, String>
where
    R: Read,
    F: FnMut(u64),
{
    let mut created_output = false;
    let copy_result = (|| {
        let mut outfile = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dest_path)
            .map_err(|e| {
                if e.kind() == ErrorKind::AlreadyExists {
                    format!(
                        "Output file already exists and will not be overwritten: {}",
                        dest_path.display()
                    )
                } else {
                    format!("Failed to create output file: {}", e)
                }
            })?;
        created_output = true;
        let mut buffer = [0u8; 65536];
        let mut bytes_written = 0u64;

        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .map_err(|e| format!("Read error: {}", e))?;
            if bytes_read == 0 {
                break;
            }

            outfile
                .write_all(&buffer[..bytes_read])
                .map_err(|e| format!("Write error: {}", e))?;
            bytes_written = add_extracted_bytes(
                bytes_written,
                bytes_read as u64,
                &dest_path.display().to_string(),
            )?;
            progress_callback(bytes_written);
        }

        outfile.flush().map_err(|e| format!("Flush error: {}", e))?;
        Ok(bytes_written)
    })();

    if copy_result.is_err() && created_output {
        let _ = fs::remove_file(dest_path);
    }

    copy_result
}

/// Extract a single entry from a ZIP archive to a destination path
///
/// # Arguments
/// * `archive_path` - Path to the ZIP archive
/// * `entry_path` - Path of the entry within the archive (e.g., "folder/file.txt")
/// * `output_path` - Destination file path for the extracted entry
///
/// # Returns
/// Result with extracted bytes count, or error
pub fn extract_zip_entry(
    archive_path: &str,
    entry_path: &str,
    output_path: &str,
) -> Result<u64, ContainerError> {
    let archive_path_obj = Path::new(archive_path);
    let output_path_obj = Path::new(output_path);

    if !archive_path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive not found: {}",
            archive_path
        )));
    }

    let file =
        File::open(archive_path_obj).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Normalize entry path (remove leading slash, handle both / and \)
    let normalized_entry = entry_path.trim_start_matches('/').replace('\\', "/");
    let with_slash = format!("{}/", normalized_entry);

    // Find the entry index first (without holding a mutable borrow)
    let entry_index = (0..archive.len())
        .find(|&i| {
            archive
                .by_index(i)
                .map(|e| {
                    let name = e.name();
                    name == normalized_entry
                        || name == with_slash
                        || name.trim_end_matches('/') == normalized_entry
                })
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("Entry not found in archive: {}", entry_path))?;

    // Now get the entry by index
    let mut entry = archive
        .by_index(entry_index)
        .map_err(|e| format!("Failed to read entry: {}", e))?;

    if entry.is_dir() {
        return Err(ContainerError::UnsupportedOperation(format!(
            "Cannot extract directory as a single file: {}",
            entry_path
        )));
    }

    // Create parent directories if needed
    if let Some(parent) = output_path_obj.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    let bytes_written = copy_entry_to_file(&mut entry, output_path_obj, |_| {})
        .map_err(|e| format!("Failed to extract entry: {}", e))?;

    debug!(
        archive = %archive_path,
        entry = %entry_path,
        output = %output_path,
        bytes = bytes_written,
        "Extracted single entry from ZIP"
    );

    Ok(bytes_written)
}

/// List entries in a ZIP archive without extracting
///
/// Uses `by_index_raw()` to read metadata for encrypted entries without
/// requiring decryption. This allows listing the directory structure of
/// password-protected ZIP files.
pub fn list_zip_entries(archive_path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let path = Path::new(archive_path);
    if !path.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive not found: {}",
            archive_path
        )));
    }

    // Try the zip crate first (faster for valid archives with central directory)
    match list_zip_entries_native(archive_path) {
        Ok(entries) => Ok(entries),
        Err(e) => {
            // Fallback to libarchive for damaged/truncated ZIPs or those without EOCD
            // libarchive can read by scanning local file headers
            tracing::info!(
                path = %archive_path,
                error = %e,
                "zip crate failed, falling back to libarchive"
            );
            super::libarchive_backend::list_entries_as_archive_entry(archive_path, "ZIP")
        }
    }
}

/// Internal function using the zip crate (requires valid central directory)
fn list_zip_entries_native(archive_path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let path = Path::new(archive_path);
    let file = File::open(path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut entries = Vec::with_capacity(archive.len());

    for i in 0..archive.len() {
        // Use by_index_raw() which works for encrypted entries
        // It returns raw compressed data reader without decryption
        match archive.by_index_raw(i) {
            Ok(entry) => {
                entries.push(ArchiveEntry {
                    index: i,
                    path: entry.name().to_string(),
                    is_directory: entry.is_dir(),
                    size: entry.size(),
                    compressed_size: entry.compressed_size(),
                    crc32: entry.crc32(),
                    compression_method: if entry.encrypted() {
                        format!("{:?} (encrypted)", entry.compression())
                    } else {
                        format!("{:?}", entry.compression())
                    },
                    last_modified: entry
                        .last_modified()
                        .map(|dt| {
                            format!(
                                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                                dt.year(),
                                dt.month(),
                                dt.day(),
                                dt.hour(),
                                dt.minute(),
                                dt.second()
                            )
                        })
                        .unwrap_or_default(),
                });
            }
            Err(e) => {
                // Log error but continue with remaining entries
                tracing::warn!("Failed to read ZIP entry {}: {}", i, e);
            }
        }
    }

    Ok(entries)
}

/// Entry information from an archive
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveEntry {
    /// Entry index
    pub index: usize,
    /// Path within the archive
    pub path: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Uncompressed size
    pub size: u64,
    /// Compressed size
    pub compressed_size: u64,
    /// CRC32 checksum
    pub crc32: u32,
    /// Compression method
    pub compression_method: String,
    /// Last modified timestamp
    pub last_modified: String,
}

// =============================================================================
// Lazy Loading Support for Large ZIP Archives
// =============================================================================

/// Get the total entry count in a ZIP archive without reading all entries
///
/// This is a fast operation that only reads the central directory,
/// useful for showing progress before loading entries.
pub fn get_zip_entry_count(archive_path: &str) -> Result<usize, ContainerError> {
    let path = Path::new(archive_path);
    if !path.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive not found: {}",
            archive_path
        )));
    }

    // Try native method first
    let file = File::open(path).map_err(|e| format!("Failed to open archive: {}", e))?;

    match ZipArchive::new(file) {
        Ok(archive) => Ok(archive.len()),
        Err(e) => {
            // Fallback to libarchive - count entries from full list
            tracing::info!(
                path = %archive_path,
                error = %e,
                "zip crate failed for entry count, falling back to libarchive"
            );
            let entries =
                super::libarchive_backend::list_entries_as_archive_entry(archive_path, "ZIP")?;
            Ok(entries.len())
        }
    }
}

/// Get root-level entries in a ZIP archive (lazy loading)
///
/// Returns only entries at the root level (no parent directory),
/// suitable for initial tree display without loading the entire archive.
pub fn get_zip_root_entries(archive_path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    get_zip_children_at_path(archive_path, "")
}

/// Get children entries at a specific path in a ZIP archive (lazy loading)
///
/// Returns entries that are direct children of the given parent path.
/// This enables on-demand loading of ZIP contents as the user expands folders.
///
/// # Arguments
/// * `archive_path` - Path to the ZIP file
/// * `parent_path` - Path within the ZIP to list children of (empty string for root)
///
/// # Performance
/// This iterates through all entries but filters them efficiently.
/// For very large archives (>100k entries), consider using indexed access.
pub fn get_zip_children_at_path(
    archive_path: &str,
    parent_path: &str,
) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let path = Path::new(archive_path);
    if !path.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive not found: {}",
            archive_path
        )));
    }

    // Try native method first, fallback to libarchive-based filtering
    match get_zip_children_at_path_native(archive_path, parent_path) {
        Ok(entries) => Ok(entries),
        Err(e) => {
            // Fallback: Get all entries via libarchive and filter
            tracing::info!(
                path = %archive_path,
                error = %e,
                "zip crate failed for children, using list_zip_entries with filter"
            );
            let all_entries = list_zip_entries(archive_path)?;
            Ok(filter_children_entries(all_entries, parent_path))
        }
    }
}

/// Filter a list of entries to only include direct children of the parent path
fn filter_children_entries(entries: Vec<ArchiveEntry>, parent_path: &str) -> Vec<ArchiveEntry> {
    // Normalize parent path
    let normalized_parent = if parent_path.is_empty() || parent_path == "/" {
        String::new()
    } else {
        let p = parent_path.trim_start_matches('/').trim_end_matches('/');
        format!("{}/", p)
    };

    let mut result = Vec::new();
    let mut seen_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in entries {
        let normalized_entry = entry.path.trim_start_matches('/');

        if normalized_parent.is_empty() {
            // Root level
            if let Some(slash_pos) = normalized_entry.find('/') {
                // Has a subdirectory - add implicit top-level dir if not seen
                let top_dir = &normalized_entry[..slash_pos];
                if !seen_dirs.contains(top_dir) {
                    seen_dirs.insert(top_dir.to_string());
                    if entry.is_directory && normalized_entry.trim_end_matches('/') == top_dir {
                        result.push(entry);
                    } else {
                        result.push(ArchiveEntry {
                            index: entry.index,
                            path: format!("{}/", top_dir),
                            is_directory: true,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            compression_method: String::new(),
                            last_modified: String::new(),
                        });
                    }
                }
            } else {
                // File at root level
                result.push(entry);
            }
        } else {
            // Child of specific parent
            if !normalized_entry.starts_with(&normalized_parent) {
                continue;
            }

            let relative = &normalized_entry[normalized_parent.len()..];
            if relative.is_empty() {
                continue;
            }

            if let Some(slash_pos) = relative.find('/') {
                // Has subdirectory
                let subdir = &relative[..slash_pos];
                let full_subdir_path = format!("{}{}/", normalized_parent, subdir);
                if !seen_dirs.contains(&full_subdir_path) {
                    seen_dirs.insert(full_subdir_path.clone());
                    if entry.is_directory
                        && normalized_entry.trim_end_matches('/')
                            == full_subdir_path.trim_end_matches('/')
                    {
                        result.push(entry);
                    } else {
                        result.push(ArchiveEntry {
                            index: entry.index,
                            path: full_subdir_path,
                            is_directory: true,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            compression_method: String::new(),
                            last_modified: String::new(),
                        });
                    }
                }
            } else {
                // Direct child file
                result.push(entry);
            }
        }
    }

    // Sort entries: directories first, then by name
    result.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.path.to_lowercase().cmp(&b.path.to_lowercase()),
    });

    result
}

/// Native implementation using zip crate
fn get_zip_children_at_path_native(
    archive_path: &str,
    parent_path: &str,
) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let path = Path::new(archive_path);
    let file = File::open(path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Normalize parent path
    let normalized_parent = if parent_path.is_empty() || parent_path == "/" {
        String::new()
    } else {
        let p = parent_path.trim_start_matches('/').trim_end_matches('/');
        format!("{}/", p)
    };

    let mut entries = Vec::new();
    let mut seen_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 0..archive.len() {
        // Extract entry info using by_index_raw which works for encrypted entries
        let entry_info = match archive.by_index_raw(i) {
            Ok(e) => Some((
                e.name().to_string(),
                e.is_dir(),
                e.size(),
                e.compressed_size(),
                e.crc32(),
                e.encrypted(),
                format!("{:?}", e.compression()),
                e.last_modified()
                    .map(|dt| {
                        format!(
                            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                            dt.year(),
                            dt.month(),
                            dt.day(),
                            dt.hour(),
                            dt.minute(),
                            dt.second()
                        )
                    })
                    .unwrap_or_default(),
            )),
            Err(_) => None,
        };

        let (
            entry_path,
            is_dir,
            size,
            compressed_size,
            crc32,
            encrypted,
            compression,
            last_modified,
        ) = match entry_info {
            Some(info) => info,
            None => continue,
        };

        let normalized_entry = entry_path.trim_start_matches('/');

        let compression_method = if encrypted {
            format!("{} (encrypted)", compression)
        } else {
            compression
        };

        // Check if this entry is a direct child of parent_path
        if normalized_parent.is_empty() {
            // Root level: entry should have no directory component, or be a top-level dir
            let entry_depth = normalized_entry.trim_end_matches('/').matches('/').count();

            if entry_depth == 0 {
                // Direct file or directory at root
                let dir_name = if is_dir {
                    normalized_entry.trim_end_matches('/').to_string()
                } else {
                    // Check if there's an implicit directory
                    if let Some(slash_pos) = normalized_entry.find('/') {
                        normalized_entry[..slash_pos].to_string()
                    } else {
                        // File at root
                        entries.push(ArchiveEntry {
                            index: i,
                            path: entry_path.clone(),
                            is_directory: is_dir,
                            size,
                            compressed_size,
                            crc32,
                            compression_method: compression_method.clone(),
                            last_modified: last_modified.clone(),
                        });
                        continue;
                    }
                };

                // Add directory if not seen
                if is_dir {
                    entries.push(ArchiveEntry {
                        index: i,
                        path: entry_path.clone(),
                        is_directory: is_dir,
                        size,
                        compressed_size,
                        crc32,
                        compression_method: compression_method.clone(),
                        last_modified: last_modified.clone(),
                    });
                } else if !seen_dirs.contains(&dir_name) {
                    // Implicit directory from file path
                    if normalized_entry.contains('/') {
                        seen_dirs.insert(dir_name.clone());
                        entries.push(ArchiveEntry {
                            index: i,
                            path: format!("{}/", dir_name),
                            is_directory: true,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            compression_method: String::new(),
                            last_modified: String::new(),
                        });
                    } else {
                        entries.push(ArchiveEntry {
                            index: i,
                            path: entry_path.clone(),
                            is_directory: is_dir,
                            size,
                            compressed_size,
                            crc32,
                            compression_method: compression_method.clone(),
                            last_modified: last_modified.clone(),
                        });
                    }
                }
            } else if entry_depth == 0 && !normalized_entry.contains('/') {
                entries.push(ArchiveEntry {
                    index: i,
                    path: entry_path.clone(),
                    is_directory: is_dir,
                    size,
                    compressed_size,
                    crc32,
                    compression_method: compression_method.clone(),
                    last_modified: last_modified.clone(),
                });
            } else {
                // Check for implicit top-level directory
                if let Some(slash_pos) = normalized_entry.find('/') {
                    let top_dir = &normalized_entry[..slash_pos];
                    if !seen_dirs.contains(top_dir) {
                        seen_dirs.insert(top_dir.to_string());
                        entries.push(ArchiveEntry {
                            index: i,
                            path: format!("{}/", top_dir),
                            is_directory: true,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            compression_method: String::new(),
                            last_modified: String::new(),
                        });
                    }
                }
            }
        } else {
            // Child of specific parent
            if !normalized_entry.starts_with(&normalized_parent) {
                continue;
            }

            let relative = &normalized_entry[normalized_parent.len()..];
            if relative.is_empty() {
                continue;
            }

            let relative_depth = relative.trim_end_matches('/').matches('/').count();

            if relative_depth == 0 {
                // Direct child
                if is_dir || !relative.contains('/') {
                    entries.push(ArchiveEntry {
                        index: i,
                        path: entry_path.clone(),
                        is_directory: is_dir,
                        size,
                        compressed_size,
                        crc32,
                        compression_method: compression_method.clone(),
                        last_modified: last_modified.clone(),
                    });
                }
            } else {
                // Check for implicit subdirectory
                if let Some(slash_pos) = relative.find('/') {
                    let subdir = &relative[..slash_pos];
                    let full_subdir_path = format!("{}{}/", normalized_parent, subdir);
                    if !seen_dirs.contains(&full_subdir_path) {
                        seen_dirs.insert(full_subdir_path.clone());
                        entries.push(ArchiveEntry {
                            index: i,
                            path: full_subdir_path,
                            is_directory: true,
                            size: 0,
                            compressed_size: 0,
                            crc32: 0,
                            compression_method: String::new(),
                            last_modified: String::new(),
                        });
                    }
                }
            }
        }
    }

    // Sort entries: directories first, then by name
    entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.path.to_lowercase().cmp(&b.path.to_lowercase()),
    });

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_result_default() {
        let result = ExtractResult {
            files_extracted: 0,
            directories_created: 0,
            bytes_written: 0,
            failed_files: vec![],
            success: true,
        };
        assert!(result.success);
        assert_eq!(result.files_extracted, 0);
    }

    #[test]
    fn test_extract_error() {
        let error = ExtractError {
            path: "test/file.txt".to_string(),
            error: "Permission denied".to_string(),
        };
        assert_eq!(error.path, "test/file.txt");
        assert!(error.error.contains("Permission"));
    }

    #[test]
    fn add_extracted_bytes_rejects_overflow() {
        assert_eq!(add_extracted_bytes(40, 2, "entry.bin").unwrap(), 42);

        let err = add_extracted_bytes(u64::MAX - 5, 10, "entry.bin").unwrap_err();
        assert!(err.contains("byte count overflow"), "unexpected: {}", err);
    }

    #[test]
    fn extraction_progress_addition_rejects_overflow() {
        let committed_bytes = u64::MAX - 1;

        let err = add_extracted_bytes(committed_bytes, 64, "entry.bin").unwrap_err();

        assert!(err.contains("entry.bin"), "unexpected: {}", err);
    }

    #[test]
    fn archive_entry_output_path_sanitizes_archive_name() {
        let temp_dir = TempDir::new().unwrap();
        let dest_path = archive_entry_output_path(
            temp_dir.path(),
            "folder/file?.txt",
            Path::new("folder/file?.txt"),
        )
        .unwrap();

        let canonical_base = temp_dir.path().canonicalize().unwrap();
        assert!(dest_path.starts_with(canonical_base));
        assert_eq!(
            dest_path.file_name().and_then(|name| name.to_str()),
            Some("folder_file.txt")
        );
    }

    #[test]
    fn archive_entry_output_path_rejects_empty_sanitized_name() {
        let temp_dir = TempDir::new().unwrap();
        let err = archive_entry_output_path(temp_dir.path(), "...", Path::new("...")).unwrap_err();

        assert_eq!(err.path, "...");
        assert!(err.error.contains("empty extraction path"));
    }

    #[test]
    fn test_extract_zip_nonexistent() {
        let result = extract_zip("/nonexistent/archive.zip", "/tmp/output");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_extract_gzip_nonexistent() {
        let result = extract_gzip("/nonexistent/file.gz", "/tmp/output");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_gzip_removes_partial_output_on_decode_error() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let temp_dir = TempDir::new().unwrap();
        let gzip_path = temp_dir.path().join("corrupt.gz");
        let output_dir = temp_dir.path().join("out");
        let output_file = output_dir.join("corrupt");

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(b"decoded bytes before footer failure")
            .unwrap();
        let mut gzip_data = encoder.finish().unwrap();
        let last = gzip_data.last_mut().unwrap();
        *last ^= 0xff;
        std::fs::write(&gzip_path, gzip_data).unwrap();

        let err = extract_gzip(gzip_path.to_str().unwrap(), output_dir.to_str().unwrap())
            .expect_err("corrupt gzip footer should fail extraction");

        assert!(
            err.to_string().contains("Failed to decompress"),
            "unexpected error: {}",
            err
        );
        assert!(
            !output_file.exists(),
            "partial GZIP extraction output should be removed"
        );
    }

    #[test]
    fn test_extract_gzip_rejects_existing_output_without_removing_it() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let temp_dir = TempDir::new().unwrap();
        let gzip_path = temp_dir.path().join("evidence.gz");
        let output_dir = temp_dir.path().join("out");
        let output_file = output_dir.join("evidence");

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"new extracted bytes").unwrap();
        std::fs::write(&gzip_path, encoder.finish().unwrap()).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        std::fs::write(&output_file, b"original examiner data").unwrap();

        let err = extract_gzip(gzip_path.to_str().unwrap(), output_dir.to_str().unwrap())
            .expect_err("existing GZIP destination should be rejected");

        assert!(
            err.to_string().contains("will not be overwritten"),
            "unexpected error: {}",
            err
        );
        assert_eq!(
            std::fs::read(&output_file).unwrap(),
            b"original examiner data"
        );
    }

    #[test]
    fn test_list_zip_entries_nonexistent() {
        let result = list_zip_entries("/nonexistent/archive.zip");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_unsupported_format() {
        // Create a temp file with unknown format
        let temp_dir = TempDir::new().unwrap();
        let fake_archive = temp_dir.path().join("test.unknown");
        File::create(&fake_archive)
            .unwrap()
            .write_all(b"not an archive")
            .unwrap();

        let result = extract(
            fake_archive.to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_entry() {
        let entry = ArchiveEntry {
            index: 0,
            path: "test.txt".to_string(),
            is_directory: false,
            size: 1024,
            compressed_size: 512,
            crc32: 0x12345678,
            compression_method: "Deflated".to_string(),
            last_modified: "2024-01-01 12:00:00".to_string(),
        };

        assert_eq!(entry.index, 0);
        assert!(!entry.is_directory);
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.compressed_size, 512);
    }

    #[test]
    fn test_copy_entry_to_file_removes_partial_output_on_read_error() {
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("partial.bin");
        let mut reader = FailingReader::new(b"partial bytes", true);

        let err = copy_entry_to_file(&mut reader, &dest_path, |_| {}).unwrap_err();

        assert!(err.contains("Read error"), "unexpected error: {}", err);
        assert!(
            !dest_path.exists(),
            "partial extraction output should be removed"
        );
    }

    #[test]
    fn test_copy_entry_to_file_rejects_existing_output_without_removing_it() {
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("existing.bin");
        std::fs::write(&dest_path, b"existing bytes").unwrap();
        let mut reader = std::io::Cursor::new(b"replacement bytes".to_vec());

        let err = copy_entry_to_file(&mut reader, &dest_path, |_| {}).unwrap_err();

        assert!(
            err.contains("will not be overwritten"),
            "unexpected error: {}",
            err
        );
        assert_eq!(std::fs::read(&dest_path).unwrap(), b"existing bytes");
    }

    #[test]
    fn test_copy_entry_to_file_reports_success_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("complete.bin");
        let mut reader = std::io::Cursor::new(b"complete bytes".to_vec());
        let mut progress = Vec::new();

        let bytes = copy_entry_to_file(&mut reader, &dest_path, |written| {
            progress.push(written);
        })
        .unwrap();

        assert_eq!(bytes, 14);
        assert_eq!(std::fs::read(&dest_path).unwrap(), b"complete bytes");
        assert_eq!(progress.last().copied(), Some(14));
    }

    #[test]
    fn test_extract_zip_rejects_existing_output_without_removing_it() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("archive.zip");
        let output_dir = temp_dir.path().join("out");
        let output_file = output_dir.join("file.txt");
        write_test_zip(&zip_path, "file.txt", b"new archive bytes");
        std::fs::create_dir_all(&output_dir).unwrap();
        std::fs::write(&output_file, b"existing bytes").unwrap();

        let result = extract_zip(zip_path.to_str().unwrap(), output_dir.to_str().unwrap()).unwrap();

        assert!(!result.success);
        assert_eq!(result.files_extracted, 0);
        assert_eq!(result.failed_files.len(), 1);
        assert!(
            result.failed_files[0]
                .error
                .contains("will not be overwritten"),
            "unexpected error: {}",
            result.failed_files[0].error
        );
        assert_eq!(std::fs::read(&output_file).unwrap(), b"existing bytes");
    }

    #[test]
    fn test_extract_zip_entry_rejects_existing_output_without_removing_it() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("archive.zip");
        let output_file = temp_dir.path().join("single.bin");
        write_test_zip(&zip_path, "file.txt", b"new archive bytes");
        std::fs::write(&output_file, b"existing bytes").unwrap();

        let err = extract_zip_entry(
            zip_path.to_str().unwrap(),
            "file.txt",
            output_file.to_str().unwrap(),
        )
        .expect_err("existing single-entry destination should be rejected");

        assert!(
            err.to_string().contains("will not be overwritten"),
            "unexpected error: {}",
            err
        );
        assert_eq!(std::fs::read(&output_file).unwrap(), b"existing bytes");
    }

    fn write_test_zip(path: &Path, entry_path: &str, data: &[u8]) {
        use zip::write::SimpleFileOptions;
        use zip::{CompressionMethod, ZipWriter};

        let file = File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file(entry_path, options).unwrap();
        zip.write_all(data).unwrap();
        zip.finish().unwrap();
    }

    struct FailingReader {
        data: Vec<u8>,
        offset: usize,
        fail_after_data: bool,
    }

    impl FailingReader {
        fn new(data: &[u8], fail_after_data: bool) -> Self {
            Self {
                data: data.to_vec(),
                offset: 0,
                fail_after_data,
            }
        }
    }

    impl Read for FailingReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.offset < self.data.len() {
                let len = (self.data.len() - self.offset).min(buf.len());
                buf[..len].copy_from_slice(&self.data[self.offset..self.offset + len]);
                self.offset += len;
                return Ok(len);
            }
            if self.fail_after_data {
                self.fail_after_data = false;
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "forced EOF"));
            }
            Ok(0)
        }
    }
}
