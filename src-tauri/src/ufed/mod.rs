// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED container shim — re-exports from the `ffx-ufed` crate and registers
//! the archive operations bridge that connects UFED ZIP handling to the main
//! app's `archive` module.

// Re-export everything from the standalone crate
pub use ffx_ufed::*;

use crate::archive;
use crate::containers::ContainerError;

// Alias to avoid shadowing the pub use glob re-export of `archive_ops` module
use bridge::ArchiveOps;
use ffx_ufed::archive_ops as bridge;

// =============================================================================
// Archive Bridge Implementation
// =============================================================================

/// Concrete implementation of [`ArchiveOps`] that delegates to the app's
/// `archive` module for ZIP operations needed by UFED containers.
struct RealArchiveOps;

impl ArchiveOps for RealArchiveOps {
    fn get_zip_index(&self, path: &str) -> Result<bridge::ZipTreeIndex, ContainerError> {
        let index = archive::ZipIndex::get_or_create(path)?;

        // Convert root_entries
        let root_entries: Vec<bridge::ZipIndexEntry> = index
            .root_entries
            .iter()
            .map(convert_zip_index_entry)
            .collect();

        // Convert children map
        let children: std::collections::HashMap<String, Vec<bridge::ZipIndexEntry>> = index
            .children
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().map(convert_zip_index_entry).collect()))
            .collect();

        Ok(bridge::ZipTreeIndex {
            entry_count: index.entry_count as usize,
            root_entries,
            children,
        })
    }

    fn list_zip_entries(
        &self,
        path: &str,
    ) -> Result<Vec<bridge::ArchiveListEntry>, ContainerError> {
        let entries = archive::list_zip_entries(path)?;
        Ok(entries
            .into_iter()
            .map(|e| bridge::ArchiveListEntry {
                index: e.index,
                path: e.path,
                is_directory: e.is_directory,
                size: e.size,
                compressed_size: e.compressed_size,
                crc32: e.crc32,
                compression_method: e.compression_method,
                last_modified: e.last_modified,
            })
            .collect())
    }

    fn get_zip_entry_count(&self, path: &str) -> Result<usize, ContainerError> {
        archive::get_zip_entry_count(path)
    }

    fn extract_archive(
        &self,
        path: &str,
        output_dir: &str,
    ) -> Result<bridge::ArchiveExtractResult, ContainerError> {
        let result = archive::extract(path, output_dir)?;
        Ok(bridge::ArchiveExtractResult {
            files_extracted: result.files_extracted,
            directories_created: result.directories_created,
            bytes_written: result.bytes_written,
            failed_files: result
                .failed_files
                .into_iter()
                .map(|f| bridge::ArchiveExtractFailure {
                    path: f.path,
                    error: f.error,
                })
                .collect(),
            success: result.success,
        })
    }

    fn read_archive_file(
        &self,
        archive_path: &str,
        entry_path: &str,
    ) -> Result<Vec<u8>, ContainerError> {
        archive::libarchive_read_file(archive_path, entry_path)
    }
}

/// Convert an `archive::ZipIndexEntry` to the bridge `ZipIndexEntry`.
fn convert_zip_index_entry(e: &archive::ZipIndexEntry) -> bridge::ZipIndexEntry {
    bridge::ZipIndexEntry {
        name: e.name.clone(),
        path: e.path.clone(),
        is_directory: e.is_directory,
        size: e.size,
        compressed_size: e.compressed_size,
        crc32: e.crc32,
        index: e.index,
    }
}

/// Register the archive operations bridge.
///
/// Must be called once at app startup before any UFED ZIP operations.
pub fn init_archive_bridge() {
    bridge::register(Box::new(RealArchiveOps));
}
