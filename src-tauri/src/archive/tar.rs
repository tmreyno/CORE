// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! TAR archive format parsing
//!
//! Provides file listing for TAR archives and compressed variants:
//! - .tar (uncompressed)
//! - .tar.gz / .tgz (gzip compressed)
//! - .tar.bz2 / .tbz2 (bzip2 compressed)
//! - .tar.xz / .txz (xz/lzma compressed)
//! - .tar.zst / .tzst (zstandard compressed)
//!
//! ## TAR Header Layout (512 bytes per entry)
//!
//! | Offset | Size | Field           |
//! |--------|------|-----------------|
//! | 0      | 100  | File name       |
//! | 100    | 8    | File mode       |
//! | 108    | 8    | Owner UID       |
//! | 116    | 8    | Owner GID       |
//! | 124    | 12   | File size       |
//! | 136    | 12   | Modified time   |
//! | 148    | 8    | Checksum        |
//! | 156    | 1    | Type flag       |
//! | 157    | 100  | Link name       |
//!
//! ## Type Flags
//! - '0' or '\0': Regular file
//! - '5': Directory
//! - '1': Hard link
//! - '2': Symbolic link

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use tracing::debug;

use super::extraction::ArchiveEntry;
use crate::containers::ContainerError;

// =============================================================================
// TAR Archive Types
// =============================================================================

/// TAR compression type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TarCompression {
    /// Uncompressed .tar
    None,
    /// Gzip compressed .tar.gz or .tgz
    Gzip,
    /// Bzip2 compressed .tar.bz2 or .tbz2
    Bzip2,
    /// XZ/LZMA compressed .tar.xz or .txz
    Xz,
    /// Zstandard compressed .tar.zst or .tzst
    Zstd,
}

impl TarCompression {
    /// Detect compression type from file extension
    pub fn from_extension(path: &str) -> Self {
        let path_lower = path.to_lowercase();
        
        if path_lower.ends_with(".tar.gz") || path_lower.ends_with(".tgz") {
            TarCompression::Gzip
        } else if path_lower.ends_with(".tar.bz2") || path_lower.ends_with(".tbz2") {
            TarCompression::Bzip2
        } else if path_lower.ends_with(".tar.xz") || path_lower.ends_with(".txz") {
            TarCompression::Xz
        } else if path_lower.ends_with(".tar.zst") || path_lower.ends_with(".tzst") {
            TarCompression::Zstd
        } else {
            TarCompression::None
        }
    }
}

// =============================================================================
// File Listing
// =============================================================================

/// List all entries in a TAR archive (compressed or uncompressed)
pub fn list_entries(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let compression = TarCompression::from_extension(path);
    
    debug!(path = %path, compression = ?compression, "Listing TAR archive entries");
    
    match compression {
        TarCompression::None => list_tar_entries(path),
        TarCompression::Gzip => list_tar_gz_entries(path),
        TarCompression::Bzip2 => list_tar_bz2_entries(path),
        TarCompression::Xz => list_tar_xz_entries(path),
        TarCompression::Zstd => list_tar_zstd_entries(path),
    }
}

/// List entries in uncompressed TAR archive
fn list_tar_entries(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open TAR archive: {}", e))?;
    
    let reader = BufReader::new(file);
    read_tar_archive(reader, path)
}

/// List entries in gzip-compressed TAR archive
fn list_tar_gz_entries(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    use flate2::read::GzDecoder;
    
    let file = File::open(path)
        .map_err(|e| format!("Failed to open tar.gz archive: {}", e))?;
    
    let decoder = GzDecoder::new(BufReader::new(file));
    read_tar_archive(decoder, path)
}

/// List entries in bzip2-compressed TAR archive
fn list_tar_bz2_entries(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    use bzip2::read::BzDecoder;
    
    let file = File::open(path)
        .map_err(|e| format!("Failed to open tar.bz2 archive: {}", e))?;
    
    let decoder = BzDecoder::new(BufReader::new(file));
    read_tar_archive(decoder, path)
}

/// List entries in XZ-compressed TAR archive
fn list_tar_xz_entries(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    use xz2::read::XzDecoder;
    
    let file = File::open(path)
        .map_err(|e| format!("Failed to open tar.xz archive: {}", e))?;
    
    let decoder = XzDecoder::new(BufReader::new(file));
    read_tar_archive(decoder, path)
}

/// List entries in Zstandard-compressed TAR archive
fn list_tar_zstd_entries(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open tar.zst archive: {}", e))?;
    
    let decoder = zstd::stream::read::Decoder::new(BufReader::new(file))
        .map_err(|e| format!("Failed to create zstd decoder: {}", e))?;
    
    read_tar_archive(decoder, path)
}

/// Read TAR archive entries from any reader
fn read_tar_archive<R: Read>(reader: R, archive_path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let mut archive = tar::Archive::new(reader);
    let mut entries = Vec::new();
    
    for (index, entry_result) in archive.entries()
        .map_err(|e| format!("Failed to read TAR entries: {}", e))?
        .enumerate()
    {
        let entry = entry_result
            .map_err(|e| format!("Failed to read TAR entry {}: {}", index, e))?;
        
        let header = entry.header();
        
        // Get entry path
        let entry_path = entry.path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| format!("entry_{}", index));
        
        // Determine if directory
        let is_directory = header.entry_type().is_dir();
        
        // Get size
        let size = header.size().unwrap_or(0);
        
        // Get modification time
        let last_modified = header.mtime()
            .ok()
            .map(|ts| {
                // Convert Unix timestamp to datetime string
                use chrono::{DateTime, Utc};
                DateTime::from_timestamp(ts as i64, 0)
                    .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        
        // Get checksum
        let cksum = header.cksum().unwrap_or(0) as u32;
        
        // Determine compression method description
        let compression = TarCompression::from_extension(archive_path);
        let compression_method = match compression {
            TarCompression::None => "None (TAR)",
            TarCompression::Gzip => "Gzip",
            TarCompression::Bzip2 => "Bzip2",
            TarCompression::Xz => "XZ/LZMA",
            TarCompression::Zstd => "Zstandard",
        }.to_string();
        
        entries.push(ArchiveEntry {
            index,
            path: entry_path,
            is_directory,
            size,
            compressed_size: size, // TAR doesn't track individual compressed sizes
            crc32: cksum,
            compression_method,
            last_modified,
        });
        
        // Safety limit - don't load more than 100,000 entries
        if entries.len() >= 100_000 {
            debug!(path = %archive_path, "TAR entry limit reached (100,000)");
            break;
        }
    }
    
    debug!(path = %archive_path, entries = entries.len(), "TAR listing complete");
    Ok(entries)
}

/// Check if file is a standalone gzip file (not tar.gz)
pub fn is_standalone_gzip(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.ends_with(".gz") && !path_lower.ends_with(".tar.gz") && !path_lower.ends_with(".tgz")
}

/// List a standalone gzip file (single compressed file)
pub fn list_gzip_entry(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    use flate2::read::GzDecoder;
    
    let file = File::open(path)
        .map_err(|e| format!("Failed to open gzip file: {}", e))?;
    
    let file_size = file.metadata()
        .map(|m| m.len())
        .unwrap_or(0);
    
    let mut decoder = GzDecoder::new(BufReader::new(file));
    
    // Get the original filename from gzip header if available
    let original_name = decoder.header()
        .and_then(|h| h.filename())
        .map(|f| String::from_utf8_lossy(f).to_string())
        .unwrap_or_else(|| {
            // Derive name by stripping .gz extension
            Path::new(path)
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "decompressed".to_string())
        });
    
    // Read a small amount to verify it's valid and estimate uncompressed size
    let mut buf = [0u8; 4096];
    let mut uncompressed_size = 0u64;
    
    loop {
        match decoder.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => uncompressed_size += n as u64,
            Err(e) => {
                debug!(path = %path, error = %e, "Error reading gzip for size estimation");
                break;
            }
        }
    }
    
    Ok(vec![ArchiveEntry {
        index: 0,
        path: original_name.clone(),
        is_directory: false,
        size: uncompressed_size,
        compressed_size: file_size,
        crc32: 0,
        compression_method: "Gzip".to_string(),
        last_modified: String::new(),
    }])
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tar_compression_detection() {
        assert_eq!(TarCompression::from_extension("file.tar"), TarCompression::None);
        assert_eq!(TarCompression::from_extension("file.tar.gz"), TarCompression::Gzip);
        assert_eq!(TarCompression::from_extension("file.tgz"), TarCompression::Gzip);
        assert_eq!(TarCompression::from_extension("file.tar.bz2"), TarCompression::Bzip2);
        assert_eq!(TarCompression::from_extension("file.tbz2"), TarCompression::Bzip2);
        assert_eq!(TarCompression::from_extension("file.tar.xz"), TarCompression::Xz);
        assert_eq!(TarCompression::from_extension("file.txz"), TarCompression::Xz);
        assert_eq!(TarCompression::from_extension("file.tar.zst"), TarCompression::Zstd);
        assert_eq!(TarCompression::from_extension("file.tzst"), TarCompression::Zstd);
    }

    #[test]
    fn test_is_standalone_gzip() {
        assert!(is_standalone_gzip("file.gz"));
        assert!(!is_standalone_gzip("file.tar.gz"));
        assert!(!is_standalone_gzip("file.tgz"));
    }

    #[test]
    fn test_list_tar_nonexistent() {
        let result = list_entries("/nonexistent/archive.tar");
        assert!(result.is_err());
    }
}
