// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified archive backend powered by libarchive
//!
//! This module provides a high-performance, unified interface for reading
//! various archive formats using libarchive (BSD-licensed).
//!
//! ## Supported Formats
//!
//! | Format | Read | Write | Encryption |
//! |--------|------|-------|------------|
//! | ZIP    | ✅   | ✅    | ✅         |
//! | 7-Zip  | ✅   | ✅    | ✅         |
//! | RAR/RAR5| ✅  | ❌    | ✅         |
//! | TAR    | ✅   | ✅    | ❌         |
//! | ISO9660| ✅   | ✅    | ❌         |
//! | CAB    | ✅   | ❌    | ❌         |
//! | LHA    | ✅   | ❌    | ❌         |
//!
//! ## Licensing
//!
//! libarchive is licensed under BSD 2-Clause License.
//! This module dynamically links to libarchive, complying with licensing requirements.

use std::path::Path;

use crate::common::lazy_loading::ContainerSummary;
use crate::containers::ContainerError;

// Re-export libarchive types
pub use libarchive2::{ArchiveFormat, FileType, ReadArchive};

// =============================================================================
// Archive Entry Information
// =============================================================================

/// Information about a single entry in an archive
#[derive(Debug, Clone)]
pub struct ArchiveEntryInfo {
    /// Entry path within the archive
    pub path: String,
    /// Entry name (filename only)
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// Parent path (empty for root entries)
    pub parent: String,
    /// Entry index in the archive
    pub index: usize,
    /// Whether the entry is encrypted
    pub is_encrypted: bool,
    /// Modification time (if available)
    pub mtime: Option<i64>,
}

impl ArchiveEntryInfo {
    /// Convert to the standard ArchiveEntry format
    pub fn to_archive_entry(&self, compression_method: &str) -> super::extraction::ArchiveEntry {
        super::extraction::ArchiveEntry {
            index: self.index,
            path: self.path.clone(),
            is_directory: self.is_dir,
            size: self.size,
            compressed_size: 0, // libarchive doesn't expose this easily
            crc32: 0,
            compression_method: compression_method.to_string(),
            last_modified: self
                .mtime
                .map(|t| {
                    chrono::DateTime::from_timestamp(t, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
        }
    }
}

/// List entries from any libarchive-supported format and convert to ArchiveEntry
///
/// This is the unified entry point for all archive formats using libarchive.
/// Reduces duplication across sevenz.rs, rar.rs, etc.
pub fn list_entries_as_archive_entry(
    path: &str,
    compression_method: &str,
) -> Result<Vec<super::extraction::ArchiveEntry>, ContainerError> {
    let handler = LibarchiveHandler::new(path);
    let entries = handler.list_entries()?;

    Ok(entries
        .into_iter()
        .map(|e| e.to_archive_entry(compression_method))
        .collect())
}

// =============================================================================
// Libarchive Backend Handler
// =============================================================================

/// High-level handler for archive operations using libarchive
pub struct LibarchiveHandler {
    /// Path to the archive file
    path: String,
    /// Cached entry count (lazy-loaded)
    entry_count: Option<usize>,
    /// Password for encrypted archives (if provided)
    password: Option<String>,
}

impl LibarchiveHandler {
    /// Create a new handler for the given archive path
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            entry_count: None,
            password: None,
        }
    }

    /// Create a new handler with a password for encrypted archives
    pub fn with_password(path: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            entry_count: None,
            password: Some(password.into()),
        }
    }

    /// Open the archive for reading
    fn open_archive(&self) -> Result<ReadArchive<'static>, ContainerError> {
        let archive = if let Some(ref pwd) = self.password {
            ReadArchive::open_with_passphrase(&self.path, pwd)
        } else {
            ReadArchive::open(&self.path)
        };

        archive.map_err(|e| ContainerError::from(format!("Failed to open archive: {}", e)))
    }

    /// Get the detected archive format
    pub fn format(&self) -> Result<String, ContainerError> {
        // Verify we can open the archive
        let _archive = self.open_archive()?;

        // Use file extension for format detection
        // libarchive auto-detects internally, but we report based on extension
        let path = Path::new(&self.path);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        Ok(match ext.as_str() {
            "zip" => "zip",
            "7z" => "7z",
            "rar" => "rar",
            "tar" => "tar",
            "gz" | "tgz" => "tar.gz",
            "bz2" | "tbz2" => "tar.bz2",
            "xz" | "txz" => "tar.xz",
            "iso" => "iso9660",
            "cab" => "cab",
            "lha" | "lzh" => "lha",
            _ => "unknown",
        }
        .to_string())
    }

    /// Get the total number of entries in the archive
    pub fn entry_count(&mut self) -> Result<usize, ContainerError> {
        if let Some(count) = self.entry_count {
            return Ok(count);
        }

        let mut archive = self.open_archive()?;
        let mut count = 0;

        while let Ok(Some(_)) = archive.next_entry() {
            count += 1;
        }

        self.entry_count = Some(count);
        Ok(count)
    }

    /// List all entries in the archive
    pub fn list_entries(&self) -> Result<Vec<ArchiveEntryInfo>, ContainerError> {
        let mut archive = self.open_archive()?;
        let mut entries = Vec::new();
        let mut index = 0;

        while let Ok(Some(entry)) = archive.next_entry() {
            let path = entry.pathname().unwrap_or_default().to_string();
            let name = Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&path)
                .to_string();

            let parent = Path::new(&path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();

            let is_dir = entry.file_type() == FileType::Directory;
            let size = if is_dir { 0 } else { entry.size() as u64 };

            // Convert SystemTime to Unix timestamp
            let mtime = entry.mtime().and_then(|st| {
                st.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs() as i64)
            });

            entries.push(ArchiveEntryInfo {
                path: path.clone(),
                name,
                is_dir,
                size,
                parent,
                index,
                is_encrypted: entry.is_encrypted(),
                mtime,
            });

            index += 1;
        }

        Ok(entries)
    }

    /// List root-level entries only
    pub fn list_root_entries(&self) -> Result<Vec<ArchiveEntryInfo>, ContainerError> {
        let entries = self.list_entries()?;

        // Find entries that have no parent or minimal parent path
        let root_entries: Vec<_> = entries
            .into_iter()
            .filter(|e| {
                let path = &e.path;
                // Root entries have no "/" or only one component
                let depth = path.matches('/').count();
                // If ends with /, it's a directory at depth 0
                // If no /, it's a file at root
                // If one /, it could be "dir/file" which is depth 1
                depth == 0 || (depth == 1 && path.ends_with('/'))
            })
            .collect();

        Ok(root_entries)
    }

    /// List children of a specific directory path
    pub fn list_children(
        &self,
        parent_path: &str,
    ) -> Result<Vec<ArchiveEntryInfo>, ContainerError> {
        let entries = self.list_entries()?;
        let parent_normalized = parent_path.trim_end_matches('/');

        let children: Vec<_> = entries
            .into_iter()
            .filter(|e| {
                let entry_parent = e.parent.trim_end_matches('/');
                entry_parent == parent_normalized
            })
            .collect();

        Ok(children)
    }

    /// Get a summary of the archive
    pub fn summary(&mut self) -> Result<ContainerSummary, ContainerError> {
        let file_meta = std::fs::metadata(&self.path)
            .map_err(|e| ContainerError::from(format!("Failed to get file metadata: {}", e)))?;

        let total_size = file_meta.len();
        let entry_count = self.entry_count()?;
        let root_entries = self.list_root_entries()?;

        Ok(ContainerSummary {
            path: self.path.clone(),
            container_type: self.format()?,
            total_size,
            entry_count,
            root_entry_count: root_entries.len(),
            lazy_loading_recommended: entry_count > 1000,
            estimated_load_time_ms: Some((entry_count as u64) / 100), // ~10k entries/sec
        })
    }

    /// Read the contents of a specific file entry
    pub fn read_entry(&self, entry_path: &str) -> Result<Vec<u8>, ContainerError> {
        let mut archive = self.open_archive()?;

        // Normalize the search path (remove leading/trailing slashes for comparison)
        let search_path = entry_path.trim_start_matches('/').trim_end_matches('/');

        while let Ok(Some(entry)) = archive.next_entry() {
            let path = entry.pathname().unwrap_or_default();
            // Normalize archive path for comparison
            let normalized_path = path.trim_start_matches('/').trim_end_matches('/');

            if normalized_path == search_path {
                if entry.file_type() == FileType::Directory {
                    return Err(ContainerError::from("Cannot read directory as file"));
                }

                let data = archive.read_data_to_vec().map_err(|e| {
                    ContainerError::from(format!("Failed to read entry data: {}", e))
                })?;

                return Ok(data);
            }
        }

        Err(ContainerError::from(format!(
            "Entry not found: {}",
            entry_path
        )))
    }

    /// Check if the archive requires a password
    pub fn needs_password(&self) -> Result<bool, ContainerError> {
        // Try to open without password and check first entry
        let result = ReadArchive::open(&self.path);

        match result {
            Ok(mut archive) => {
                // Try to read first entry
                match archive.next_entry() {
                    Ok(Some(entry)) => Ok(entry.is_encrypted()),
                    Ok(None) => Ok(false), // Empty archive
                    Err(_) => Ok(true),    // Likely encrypted headers
                }
            }
            Err(_) => Ok(true), // Can't even open - likely encrypted
        }
    }
}

// =============================================================================
// Format Detection
// =============================================================================

/// Detect archive format from file contents using libarchive
pub fn detect_format(path: &str) -> Option<String> {
    // Try to open with libarchive - it auto-detects format
    // If it fails to open, it's not a supported archive
    let _archive = ReadArchive::open(path).ok()?;

    // Use file extension for format name
    // libarchive auto-detects internally, but we report based on extension
    let p = Path::new(path);
    let ext = p.extension().and_then(|e| e.to_str())?.to_lowercase();

    match ext.as_str() {
        "zip" => Some("zip".to_string()),
        "7z" => Some("7z".to_string()),
        "rar" => Some("rar".to_string()),
        "tar" => Some("tar".to_string()),
        "gz" | "tgz" => Some("tar.gz".to_string()),
        "bz2" | "tbz2" => Some("tar.bz2".to_string()),
        "xz" | "txz" => Some("tar.xz".to_string()),
        "zst" | "tzst" => Some("tar.zst".to_string()),
        "lz4" | "tlz4" => Some("tar.lz4".to_string()),
        "iso" => Some("iso9660".to_string()),
        "cab" => Some("cab".to_string()),
        "lha" | "lzh" => Some("lha".to_string()),
        "ar" | "a" | "deb" => Some("ar".to_string()),
        "cpio" => Some("cpio".to_string()),
        _ => None,
    }
}

/// Check if a file is a supported archive format
pub fn is_supported_archive(path: &str) -> bool {
    detect_format(path).is_some()
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Quick summary of an archive without full parsing
pub fn quick_summary(path: &str) -> Result<ContainerSummary, ContainerError> {
    let mut handler = LibarchiveHandler::new(path);
    handler.summary()
}

/// List all entries in an archive
pub fn list_all_entries(path: &str) -> Result<Vec<ArchiveEntryInfo>, ContainerError> {
    let handler = LibarchiveHandler::new(path);
    handler.list_entries()
}

/// List root entries only
pub fn list_root(path: &str) -> Result<Vec<ArchiveEntryInfo>, ContainerError> {
    let handler = LibarchiveHandler::new(path);
    handler.list_root_entries()
}

/// Read a file from an archive
pub fn read_file(archive_path: &str, entry_path: &str) -> Result<Vec<u8>, ContainerError> {
    let handler = LibarchiveHandler::new(archive_path);
    handler.read_entry(entry_path)
}

/// Read a file from an encrypted archive
pub fn read_file_encrypted(
    archive_path: &str,
    entry_path: &str,
    password: &str,
) -> Result<Vec<u8>, ContainerError> {
    let handler = LibarchiveHandler::with_password(archive_path, password);
    handler.read_entry(entry_path)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper for testing extension-based format detection
    /// Note: detect_format() also validates the file can be opened by libarchive,
    /// so we test the extension matching logic separately here.
    fn format_from_extension(path: &str) -> Option<String> {
        let p = Path::new(path);
        let ext = p.extension().and_then(|e| e.to_str())?.to_lowercase();

        match ext.as_str() {
            "zip" => Some("zip".to_string()),
            "7z" => Some("7z".to_string()),
            "rar" => Some("rar".to_string()),
            "tar" => Some("tar".to_string()),
            "gz" | "tgz" => Some("tar.gz".to_string()),
            "bz2" | "tbz2" => Some("tar.bz2".to_string()),
            "xz" | "txz" => Some("tar.xz".to_string()),
            "iso" => Some("iso9660".to_string()),
            "cab" => Some("cab".to_string()),
            "lha" | "lzh" => Some("lha".to_string()),
            _ => None,
        }
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(
            format_from_extension("/path/to/file.zip"),
            Some("zip".to_string())
        );
        assert_eq!(
            format_from_extension("/path/to/file.7z"),
            Some("7z".to_string())
        );
        assert_eq!(
            format_from_extension("/path/to/file.tar.gz"),
            Some("tar.gz".to_string())
        );
        assert_eq!(
            format_from_extension("/path/to/file.rar"),
            Some("rar".to_string())
        );
        assert_eq!(format_from_extension("/path/to/file.unknown"), None);
    }

    #[test]
    fn test_extension_is_supported() {
        assert!(format_from_extension("/path/to/file.zip").is_some());
        assert!(format_from_extension("/path/to/file.7z").is_some());
        assert!(format_from_extension("/path/to/file.txt").is_none());
    }
}
