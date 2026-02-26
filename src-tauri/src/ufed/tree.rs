// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED tree browsing and lazy loading operations
//!
//! Provides file tree listing for UFED containers with lazy loading support
//! for large ZIP-based extractions (100k+ entries).

use std::path::Path;

use tracing::{debug, instrument};

use crate::containers::ContainerError;
use super::detection;
use super::types::UfedFormat;

/// UFED tree entry representing a file or directory in the extraction
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedTreeEntry {
    /// Path within the UFED container
    pub path: String,
    /// Filename
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// File size (0 for directories)
    pub size: u64,
    /// UFED-specific type (file, folder, extraction, etc.)
    pub entry_type: String,
    /// Associated hash from UFD file if available
    pub hash: Option<String>,
    /// Modified timestamp if available
    pub modified: Option<String>,
}

/// Get the file tree for a UFED container (ROOT LEVEL ONLY for large ZIPs)
///
/// For large UFED ZIP containers (>50k entries), this returns only root-level
/// entries to prevent long load times. Use get_children() for lazy expansion.
///
/// For smaller ZIPs and UFD/UFDR/UFDX, lists all entries.
///
/// # Performance Notes
/// Large UFED extractions can have 100k+ files. This function uses a cached
/// tree index for fast lookups. First access builds the index (O(n)),
/// subsequent accesses are O(1).
#[instrument]
pub fn get_tree(path: &str) -> Result<Vec<UfedTreeEntry>, ContainerError> {
    debug!(path = %path, "Getting UFED tree");

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "UFED file not found: {path}"
        )));
    }

    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;

    match format {
        UfedFormat::UfedZip => {
            // Use fast ZipIndex - builds on first access, cached after
            let index = crate::archive::ZipIndex::get_or_create(path)?;

            debug!(
                path = %path,
                entry_count = index.len(),
                "UFED ZIP index loaded"
            );

            // Large archive threshold - use lazy loading for >10k entries
            const LAZY_THRESHOLD: usize = 10_000;

            if index.len() > LAZY_THRESHOLD {
                debug!(
                    path = %path,
                    entry_count = index.len(),
                    threshold = LAZY_THRESHOLD,
                    "Large UFED ZIP - returning root level only"
                );
                // Return only root-level entries for large archives
                return Ok(index
                    .get_root_entries()
                    .iter()
                    .map(|e| UfedTreeEntry {
                        path: e.path.clone(),
                        name: e.name.clone(),
                        is_dir: e.is_directory,
                        size: e.size,
                        entry_type: if e.is_directory {
                            "folder".to_string()
                        } else {
                            "file".to_string()
                        },
                        hash: None,
                        modified: None,
                    })
                    .collect());
            }

            // For smaller archives, load all entries
            let entries = crate::archive::list_zip_entries(path)?;
            Ok(entries
                .into_iter()
                .map(|e| UfedTreeEntry {
                    path: e.path.clone(),
                    name: Path::new(&e.path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| e.path.clone()),
                    is_dir: e.is_directory,
                    size: e.size,
                    entry_type: if e.is_directory {
                        "folder".to_string()
                    } else {
                        "file".to_string()
                    },
                    hash: None,
                    modified: Some(e.last_modified),
                })
                .collect())
        }
        _ => {
            // For UFD/UFDR/UFDX, return the file itself and associated files
            let info = super::info(path)?;
            let mut entries = Vec::new();

            // Add main file
            let name = path_obj
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            entries.push(UfedTreeEntry {
                path: path.to_string(),
                name,
                is_dir: false,
                size: info.size,
                entry_type: format.to_string(),
                hash: info
                    .stored_hashes
                    .as_ref()
                    .and_then(|h| h.first())
                    .map(|h| h.hash.clone()),
                modified: None,
            });

            // Add associated files
            let parent_dir = path_obj.parent().unwrap_or(path_obj);
            for assoc in info.associated_files {
                let assoc_full_path = parent_dir.join(&assoc.filename);
                entries.push(UfedTreeEntry {
                    path: assoc_full_path.to_string_lossy().to_string(),
                    name: assoc.filename,
                    is_dir: false,
                    size: assoc.size,
                    entry_type: assoc.file_type,
                    hash: assoc.stored_hash,
                    modified: None,
                });
            }

            Ok(entries)
        }
    }
}

/// Get children of a directory in a UFED container (LAZY LOADING)
///
/// For ZIP containers, uses the fast ZipIndex for O(1) directory lookups.
/// This is optimized for large archives (like 18GB UFED extractions) by using
/// a cached tree index instead of iterating through all entries.
///
/// For other formats, returns associated files if parent_path is empty.
#[instrument]
pub fn get_children(
    path: &str,
    parent_path: &str,
) -> Result<Vec<UfedTreeEntry>, ContainerError> {
    debug!(path = %path, parent_path = %parent_path, "Getting UFED children (lazy)");

    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;

    if let UfedFormat::UfedZip = format {
        // Use fast ZipIndex for O(1) lookups
        let index = crate::archive::ZipIndex::get_or_create(path)?;

        let entries = if parent_path.is_empty() || parent_path == "/" {
            index.get_root_entries().to_vec()
        } else {
            index
                .get_children(parent_path)
                .cloned()
                .unwrap_or_default()
        };

        Ok(entries
            .into_iter()
            .map(|e| UfedTreeEntry {
                path: e.path.clone(),
                name: e.name.clone(),
                is_dir: e.is_directory,
                size: e.size,
                entry_type: if e.is_directory {
                    "folder".to_string()
                } else {
                    "file".to_string()
                },
                hash: None,
                modified: None,
            })
            .collect())
    } else {
        // For non-ZIP formats, return all at root level
        get_tree(path)
    }
}

/// Get root children of a UFED container (LAZY LOADING)
///
/// For large UFED ZIP files, this only loads the root-level entries
/// instead of iterating through all 100k+ entries.
#[instrument]
pub fn get_root_children(path: &str) -> Result<Vec<UfedTreeEntry>, ContainerError> {
    get_children(path, "")
}

/// Get entry count for a UFED ZIP without loading all entries
///
/// Useful for showing loading progress or deciding on loading strategy.
#[instrument]
pub fn get_entry_count(path: &str) -> Result<usize, ContainerError> {
    debug!(path = %path, "Getting UFED entry count");

    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;

    match format {
        UfedFormat::UfedZip => crate::archive::get_zip_entry_count(path),
        _ => {
            // For non-ZIP, count associated files
            let info = super::info(path)?;
            Ok(info.associated_files.len() + 1) // +1 for main file
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tree_nonexistent() {
        let result = get_tree("/nonexistent/path/file.ufd");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_children_nonexistent() {
        let result = get_children("/nonexistent/path/file.ufd", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_ufed_tree_entry() {
        let entry = UfedTreeEntry {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            is_dir: false,
            size: 1024,
            entry_type: "file".to_string(),
            hash: Some("abc123".to_string()),
            modified: Some("2024-01-01".to_string()),
        };

        assert_eq!(entry.name, "file.txt");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 1024);
    }
}
