// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive operations provider for UFED ZIP containers.
//!
//! UFED extraction sets can be packaged as ZIP archives. This module defines
//! the interface for ZIP archive operations that the host application injects
//! at startup. This keeps the UFED crate decoupled from the archive module.

use std::sync::OnceLock;

use ffx_errors::ContainerError;

// =============================================================================
// Bridge Types
// =============================================================================

/// Entry in a ZIP tree index (mirrors archive::ZipIndexEntry)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ZipIndexEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub compressed_size: u64,
    pub crc32: u32,
    pub index: u32,
}

/// Entry from archive listing (mirrors archive::ArchiveEntry)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveListEntry {
    pub index: usize,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub compressed_size: u64,
    pub crc32: u32,
    pub compression_method: String,
    pub last_modified: String,
}

/// Result from archive extraction (mirrors archive::ExtractResult)
#[derive(Debug, Clone)]
pub struct ArchiveExtractResult {
    pub files_extracted: u64,
    pub directories_created: u64,
    pub bytes_written: u64,
    pub failed_files: Vec<ArchiveExtractFailure>,
    pub success: bool,
}

/// Individual file extraction failure
#[derive(Debug, Clone)]
pub struct ArchiveExtractFailure {
    pub path: String,
    pub error: String,
}

/// Cached ZIP tree index (mirrors archive::ZipIndex)
pub struct ZipTreeIndex {
    pub entry_count: usize,
    pub root_entries: Vec<ZipIndexEntry>,
    pub children: std::collections::HashMap<String, Vec<ZipIndexEntry>>,
}

impl ZipTreeIndex {
    pub fn len(&self) -> usize {
        self.entry_count
    }

    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }

    pub fn get_root_entries(&self) -> &[ZipIndexEntry] {
        &self.root_entries
    }

    pub fn get_children(&self, path: &str) -> Option<&Vec<ZipIndexEntry>> {
        self.children.get(path)
    }
}

// =============================================================================
// Provider Trait
// =============================================================================

/// Trait for ZIP archive operations injected by the host application.
///
/// The host app registers an implementation at startup via [`register_archive_ops`].
/// UFED ZIP operations delegate to these methods.
pub trait ArchiveOps: Send + Sync {
    /// Get or create a cached ZIP tree index for the archive
    fn get_zip_index(&self, path: &str) -> Result<ZipTreeIndex, ContainerError>;

    /// List all entries in a ZIP archive
    fn list_zip_entries(&self, path: &str) -> Result<Vec<ArchiveListEntry>, ContainerError>;

    /// Get total entry count in a ZIP archive
    fn get_zip_entry_count(&self, path: &str) -> Result<usize, ContainerError>;

    /// Extract a ZIP archive to an output directory
    fn extract_archive(
        &self,
        path: &str,
        output_dir: &str,
    ) -> Result<ArchiveExtractResult, ContainerError>;

    /// Read a single file from an archive
    fn read_archive_file(
        &self,
        archive_path: &str,
        entry_path: &str,
    ) -> Result<Vec<u8>, ContainerError>;

    /// Read a byte range from a single file in an archive.
    fn read_archive_file_range(
        &self,
        archive_path: &str,
        entry_path: &str,
        offset: u64,
        size: usize,
    ) -> Result<Vec<u8>, ContainerError> {
        let data = self.read_archive_file(archive_path, entry_path)?;
        let start = usize::try_from(offset)
            .map_err(|_| ContainerError::InvalidFormat("Archive range offset too large".into()))?;
        if start >= data.len() {
            return Ok(Vec::new());
        }
        let end = start.saturating_add(size).min(data.len());
        Ok(data[start..end].to_vec())
    }
}

// =============================================================================
// Global Registry
// =============================================================================

static ARCHIVE_OPS: OnceLock<Box<dyn ArchiveOps>> = OnceLock::new();

/// Register the archive operations provider.
///
/// Must be called once at application startup before any UFED ZIP operations.
/// Subsequent calls are silently ignored.
pub fn register(ops: Box<dyn ArchiveOps>) {
    ARCHIVE_OPS.set(ops).ok();
}

/// Get the registered archive operations provider.
///
/// # Panics
/// Panics if [`register`] has not been called. In standalone/test mode,
/// call [`register`] with a stub implementation.
fn ops() -> &'static dyn ArchiveOps {
    ARCHIVE_OPS
        .get()
        .map(|b| b.as_ref())
        .expect("Archive operations not registered. Call ffx_ufed::archive_ops::register() first.")
}

/// Check if archive operations have been registered.
pub fn is_registered() -> bool {
    ARCHIVE_OPS.get().is_some()
}

// =============================================================================
// Public API (delegates to registered provider)
// =============================================================================

/// Get or create a cached ZIP tree index
pub fn get_zip_index(path: &str) -> Result<ZipTreeIndex, ContainerError> {
    ops().get_zip_index(path)
}

/// List all entries in a ZIP archive
pub fn list_zip_entries(path: &str) -> Result<Vec<ArchiveListEntry>, ContainerError> {
    ops().list_zip_entries(path)
}

/// Get total entry count in a ZIP archive
pub fn get_zip_entry_count(path: &str) -> Result<usize, ContainerError> {
    ops().get_zip_entry_count(path)
}

/// Extract a ZIP archive to an output directory
pub fn extract_archive(
    path: &str,
    output_dir: &str,
) -> Result<ArchiveExtractResult, ContainerError> {
    ops().extract_archive(path, output_dir)
}

/// Read a single file from an archive
pub fn read_archive_file(archive_path: &str, entry_path: &str) -> Result<Vec<u8>, ContainerError> {
    ops().read_archive_file(archive_path, entry_path)
}

/// Read a byte range from a single file in an archive
pub fn read_archive_file_range(
    archive_path: &str,
    entry_path: &str,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, ContainerError> {
    ops().read_archive_file_range(archive_path, entry_path, offset, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubArchiveOps;

    impl ArchiveOps for StubArchiveOps {
        fn get_zip_index(&self, _path: &str) -> Result<ZipTreeIndex, ContainerError> {
            Err(ContainerError::UnsupportedOperation("unused".to_string()))
        }

        fn list_zip_entries(&self, _path: &str) -> Result<Vec<ArchiveListEntry>, ContainerError> {
            Err(ContainerError::UnsupportedOperation("unused".to_string()))
        }

        fn get_zip_entry_count(&self, _path: &str) -> Result<usize, ContainerError> {
            Err(ContainerError::UnsupportedOperation("unused".to_string()))
        }

        fn extract_archive(
            &self,
            _path: &str,
            _output_dir: &str,
        ) -> Result<ArchiveExtractResult, ContainerError> {
            Err(ContainerError::UnsupportedOperation("unused".to_string()))
        }

        fn read_archive_file(
            &self,
            _archive_path: &str,
            _entry_path: &str,
        ) -> Result<Vec<u8>, ContainerError> {
            Ok(b"abcdef".to_vec())
        }
    }

    #[test]
    fn default_archive_range_reader_slices_full_entry() {
        if !is_registered() {
            register(Box::new(StubArchiveOps));
        }

        assert_eq!(
            read_archive_file_range("case.ufdr", "files/report.txt", 2, 3).unwrap(),
            b"cde"
        );
        assert_eq!(
            read_archive_file_range("case.ufdr", "files/report.txt", 99, 3).unwrap(),
            b""
        );
    }
}
