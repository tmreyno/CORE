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

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use tracing::debug;
use zip::ZipArchive;

use super::types::ArchiveFormat;
use crate::common::path_security::{safe_join, sanitize_filename};
use crate::containers::ContainerError;

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
        return Err(ContainerError::FileNotFound(format!("Archive not found: {}", archive_path.display())));
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let file = File::open(archive_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

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

        // Sanitize and join path safely
        let sanitized_name = sanitize_filename(&entry_path.to_string_lossy());
        let dest_path = match safe_join(output_path, &sanitized_name) {
            Ok(p) => p,
            Err(e) => {
                result.failed_files.push(ExtractError {
                    path: entry.name().to_string(),
                    error: format!("Path traversal detected: {:?}", e),
                });
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

            // Extract file
            let mut outfile = match File::create(&dest_path) {
                Ok(f) => f,
                Err(e) => {
                    result.failed_files.push(ExtractError {
                        path: entry.name().to_string(),
                        error: e.to_string(),
                    });
                    continue;
                }
            };

            let mut buffer = [0u8; 65536];
            let mut bytes_written = 0u64;

            loop {
                let bytes_read = match entry.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        result.failed_files.push(ExtractError {
                            path: entry.name().to_string(),
                            error: format!("Read error: {}", e),
                        });
                        break;
                    }
                };

                if let Err(e) = outfile.write_all(&buffer[..bytes_read]) {
                    result.failed_files.push(ExtractError {
                        path: entry.name().to_string(),
                        error: format!("Write error: {}", e),
                    });
                    break;
                }

                bytes_written += bytes_read as u64;
            }

            result.bytes_written += bytes_written;
            result.files_extracted += 1;
            debug!(path = %dest_path.display(), bytes = bytes_written, "Extracted file");
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
        return Err(ContainerError::FileNotFound(format!("Archive not found: {}", archive_path.display())));
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let file = File::open(archive_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Calculate total uncompressed size for progress
    let total_size: u64 = {
        let mut total = 0u64;
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                total += entry.size();
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

        // Sanitize and join path safely
        let sanitized_name = sanitize_filename(&entry_path.to_string_lossy());
        let dest_path = match safe_join(output_path, &sanitized_name) {
            Ok(p) => p,
            Err(e) => {
                result.failed_files.push(ExtractError {
                    path: entry.name().to_string(),
                    error: format!("Path traversal detected: {:?}", e),
                });
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

            // Extract file
            let mut outfile = match File::create(&dest_path) {
                Ok(f) => f,
                Err(e) => {
                    result.failed_files.push(ExtractError {
                        path: entry.name().to_string(),
                        error: e.to_string(),
                    });
                    continue;
                }
            };

            let mut buffer = [0u8; 65536];
            let mut bytes_for_file = 0u64;

            loop {
                let bytes_read = match entry.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        result.failed_files.push(ExtractError {
                            path: entry.name().to_string(),
                            error: format!("Read error: {}", e),
                        });
                        break;
                    }
                };

                if let Err(e) = outfile.write_all(&buffer[..bytes_read]) {
                    result.failed_files.push(ExtractError {
                        path: entry.name().to_string(),
                        error: format!("Write error: {}", e),
                    });
                    break;
                }

                bytes_for_file += bytes_read as u64;
                result.bytes_written += bytes_read as u64;
                progress_callback(result.bytes_written, total_size);
            }

            result.files_extracted += 1;
            debug!(path = %dest_path.display(), bytes = bytes_for_file, "Extracted file");
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
        ArchiveFormat::SevenZip => {
            Err(ContainerError::UnsupportedOperation("7-Zip extraction requires external 7z tool. Use: 7z x archive.7z -ooutput_dir".to_string()))
        }
        ArchiveFormat::Rar4 | ArchiveFormat::Rar5 => {
            Err(ContainerError::UnsupportedOperation("RAR extraction requires external unrar tool. Use: unrar x archive.rar output_dir".to_string()))
        }
        ArchiveFormat::Gzip => {
            extract_gzip(archive_path, output_dir)
        }
        _ => {
            Err(ContainerError::UnsupportedOperation(format!("Extraction not supported for format: {}", format)))
        }
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
        return Err(ContainerError::FileNotFound(format!("File not found: {}", gzip_path)));
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

    let file = File::open(gzip_path_obj)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut decoder = GzDecoder::new(file);
    let mut outfile = File::create(&dest_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    let bytes_written = std::io::copy(&mut decoder, &mut outfile)
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
        return Err(ContainerError::FileNotFound(format!("Archive not found: {}", archive_path)));
    }

    let file = File::open(archive_path_obj)
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Normalize entry path (remove leading slash, handle both / and \)
    let normalized_entry = entry_path.trim_start_matches('/').replace('\\', "/");
    let with_slash = format!("{}/", normalized_entry);
    
    // Find the entry index first (without holding a mutable borrow)
    let entry_index = (0..archive.len())
        .find(|&i| {
            archive.by_index(i)
                .map(|e| {
                    let name = e.name();
                    name == normalized_entry || name == with_slash || 
                    name.trim_end_matches('/') == normalized_entry
                })
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("Entry not found in archive: {}", entry_path))?;
    
    // Now get the entry by index
    let mut entry = archive.by_index(entry_index)
        .map_err(|e| format!("Failed to read entry: {}", e))?;

    if entry.is_dir() {
        return Err(ContainerError::UnsupportedOperation(format!(
            "Cannot extract directory as a single file: {}", entry_path
        )));
    }

    // Create parent directories if needed
    if let Some(parent) = output_path_obj.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Create output file and extract
    let mut outfile = File::create(output_path_obj)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    let bytes_written = std::io::copy(&mut entry, &mut outfile)
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
pub fn list_zip_entries(archive_path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let path = Path::new(archive_path);
    if !path.exists() {
        return Err(ContainerError::FileNotFound(format!("Archive not found: {}", archive_path)));
    }

    let file = File::open(path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut entries = Vec::with_capacity(archive.len());

    for i in 0..archive.len() {
        let entry = archive.by_index(i)
            .map_err(|e| format!("Failed to read entry {}: {}", i, e))?;

        entries.push(ArchiveEntry {
            index: i,
            path: entry.name().to_string(),
            is_directory: entry.is_dir(),
            size: entry.size(),
            compressed_size: entry.compressed_size(),
            crc32: entry.crc32(),
            compression_method: format!("{:?}", entry.compression()),
            last_modified: entry.last_modified()
                .map(|dt| format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    dt.year(), dt.month(), dt.day(),
                    dt.hour(), dt.minute(), dt.second()))
                .unwrap_or_default(),
        });
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

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_list_zip_entries_nonexistent() {
        let result = list_zip_entries("/nonexistent/archive.zip");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_unsupported_format() {
        // Create a temp file with unknown format
        let temp_dir = TempDir::new().unwrap();
        let fake_archive = temp_dir.path().join("test.unknown");
        File::create(&fake_archive).unwrap().write_all(b"not an archive").unwrap();

        let result = extract(fake_archive.to_str().unwrap(), temp_dir.path().to_str().unwrap());
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
}
