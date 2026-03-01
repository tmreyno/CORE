// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # UFED Virtual Filesystem Implementation
//!
//! ## Section Brief
//! Read-only virtual filesystem implementation for UFED mobile extractions.
//! Provides safe, corruption-proof access to extraction contents.
//!
//! ### Key Types
//! - `UfedVfs` - Virtual filesystem for UFED extractions
//!
//! ### Features
//! - Read-only access prevents extraction corruption
//! - Access extracted file system contents
//! - Navigate UFED folder structure
//! - Support for UFD, UFDR, and UFED ZIP formats
//!
//! ### Usage
//! ```rust,ignore
//! use crate::ufed::vfs::UfedVfs;
//! use crate::common::vfs::VirtualFileSystem;
//!
//! let vfs = UfedVfs::open("/path/to/extraction.zip")?;
//!
//! // List root directory
//! let entries = vfs.readdir("/")?;
//!
//! // Read a file
//! let data = vfs.read("/Files/DCIM/image.jpg", 0, 1024)?;
//! ```

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;

use super::detection::detect_format;
use super::types::UfedFormat;
use crate::common::vfs::{
    join_path, normalize_path, DirEntry, FileAttr, VfsError, VirtualFileSystem,
};

// =============================================================================
// UFED Virtual Filesystem
// =============================================================================

/// Virtual filesystem implementation for UFED extractions
///
/// Provides read-only access to UFED mobile extraction contents.
/// Supports both extracted folders and ZIP archives.
///
/// Uses `DashMap` for lock-free concurrent access to cached entries.
pub struct UfedVfs {
    /// Base path (extraction folder or ZIP file)
    #[allow(dead_code)]
    path: String,
    /// UFED format type
    #[allow(dead_code)]
    format: UfedFormat,
    /// Entry cache (path -> attributes) - lock-free concurrent map
    entries: DashMap<String, UfedEntry>,
    /// Directory children cache - lock-free concurrent map
    dir_children: DashMap<String, Vec<String>>,
    /// Next inode number - atomic for lock-free increment
    next_inode: AtomicU64,
    /// Whether this is a folder-based extraction
    is_folder: bool,
    /// Root folder path for folder-based extractions
    root_folder: Option<PathBuf>,
}

/// Cached UFED entry information
#[derive(Clone)]
struct UfedEntry {
    /// File attributes
    attr: FileAttr,
    /// Real path on disk (for folder mode)
    real_path: Option<PathBuf>,
}

impl UfedVfs {
    /// Open a UFED extraction for virtual filesystem access
    pub fn open(path: &str) -> Result<Self, VfsError> {
        let path_obj = std::path::Path::new(path);

        if !path_obj.exists() {
            return Err(VfsError::NotFound(path.to_string()));
        }

        let format = detect_format(path).ok_or_else(|| {
            VfsError::InvalidPath(format!("Not a recognized UFED format: {}", path))
        })?;

        let is_folder = path_obj.is_dir();
        let root_folder = if is_folder {
            Some(path_obj.to_path_buf())
        } else {
            // For ZIP files, check if there's an extracted folder
            path_obj.parent().map(|p| p.to_path_buf())
        };

        let vfs = Self {
            path: path.to_string(),
            format,
            entries: DashMap::new(),
            dir_children: DashMap::new(),
            next_inode: AtomicU64::new(2),
            is_folder,
            root_folder,
        };

        vfs.init_root()?;

        // If it's a folder, scan the contents
        if is_folder {
            vfs.scan_folder()?;
        }

        Ok(vfs)
    }

    /// Initialize the root directory entry
    fn init_root(&self) -> Result<(), VfsError> {
        let root_attr = FileAttr {
            size: 0,
            is_directory: true,
            permissions: 0o555,
            nlink: 2,
            inode: 1,
            ..Default::default()
        };

        let root_entry = UfedEntry {
            attr: root_attr,
            real_path: self.root_folder.clone(),
        };

        self.entries.insert("/".to_string(), root_entry);
        self.dir_children.insert("/".to_string(), Vec::new());

        Ok(())
    }

    /// Allocate a new inode number
    fn alloc_inode(&self) -> u64 {
        self.next_inode.fetch_add(1, Ordering::Relaxed)
    }

    /// Scan a folder-based extraction
    fn scan_folder(&self) -> Result<(), VfsError> {
        let root = self
            .root_folder
            .as_ref()
            .ok_or_else(|| VfsError::Internal("No root folder".to_string()))?;

        self.scan_dir_recursive(root, "/")?;

        Ok(())
    }

    /// Recursively scan a directory
    fn scan_dir_recursive(&self, dir: &std::path::Path, vfs_path: &str) -> Result<(), VfsError> {
        let read_dir = fs::read_dir(dir).map_err(|e| VfsError::IoError(e.to_string()))?;

        let mut children = Vec::new();

        for entry in read_dir {
            let entry = entry.map_err(|e| VfsError::IoError(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let child_path = join_path(vfs_path, &name);
            let real_path = entry.path();

            let metadata = entry
                .metadata()
                .map_err(|e| VfsError::IoError(e.to_string()))?;

            let is_dir = metadata.is_dir();
            let inode = self.alloc_inode();

            let attr = FileAttr {
                size: if is_dir { 0 } else { metadata.len() },
                is_directory: is_dir,
                permissions: if is_dir { 0o555 } else { 0o444 },
                nlink: if is_dir { 2 } else { 1 },
                inode,
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i64),
                ..Default::default()
            };

            self.entries.insert(
                child_path.clone(),
                UfedEntry {
                    attr,
                    real_path: Some(real_path.clone()),
                },
            );

            children.push(name);

            if is_dir {
                self.dir_children.insert(child_path.clone(), Vec::new());
            }
        }

        self.dir_children
            .insert(vfs_path.to_string(), children.clone());

        // Recurse into subdirectories
        for child_name in &children {
            let child_path = join_path(vfs_path, child_name);
            let real_path = dir.join(child_name);
            if real_path.is_dir() {
                self.scan_dir_recursive(&real_path, &child_path)?;
            }
        }

        Ok(())
    }

    /// Read file from folder-based extraction
    fn read_from_folder(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let entry = self
            .entries
            .get(path)
            .ok_or_else(|| VfsError::NotFound(path.to_string()))?;

        if entry.attr.is_directory {
            return Err(VfsError::NotAFile(path.to_string()));
        }

        let real_path = entry
            .real_path
            .as_ref()
            .ok_or_else(|| VfsError::Internal("No real path for entry".to_string()))?;

        let data = fs::read(real_path).map_err(|e| VfsError::IoError(e.to_string()))?;

        let start = offset as usize;
        if start >= data.len() {
            return Ok(Vec::new());
        }

        let end = (start + size).min(data.len());
        Ok(data[start..end].to_vec())
    }
}

impl VirtualFileSystem for UfedVfs {
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let normalized = normalize_path(path);

        self.entries
            .get(&normalized)
            .map(|e| e.attr.clone())
            .ok_or(VfsError::NotFound(normalized))
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let normalized = normalize_path(path);

        // Verify it's a directory
        let entry = self
            .entries
            .get(&normalized)
            .ok_or_else(|| VfsError::NotFound(normalized.clone()))?;

        if !entry.attr.is_directory {
            return Err(VfsError::NotADirectory(normalized));
        }

        // Get children
        let children = self
            .dir_children
            .get(&normalized)
            .ok_or_else(|| VfsError::NotFound(normalized.clone()))?;

        let mut result = Vec::new();
        for child_name in children.iter() {
            let child_path = join_path(&normalized, child_name);
            if let Some(child_entry) = self.entries.get(&child_path) {
                result.push(DirEntry {
                    name: child_name.to_string(),
                    is_directory: child_entry.attr.is_directory,
                    inode: child_entry.attr.inode,
                    file_type: if child_entry.attr.is_directory { 4 } else { 8 },
                });
            }
        }

        Ok(result)
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path);

        if self.is_folder {
            self.read_from_folder(&normalized, offset, size)
        } else {
            // For ZIP-based UFED, extract file data from the ZIP archive
            let entry_path = normalized.trim_start_matches('/');
            let data = crate::archive::libarchive_read_file(&self.path, entry_path)
                .map_err(|e| VfsError::IoError(format!("Failed to read from UFED ZIP: {}", e)))?;

            let start = offset as usize;
            if start >= data.len() {
                return Ok(Vec::new());
            }
            let end = (start + size).min(data.len());
            Ok(data[start..end].to_vec())
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
    fn test_ufed_vfs_format() {
        // Test that format enum works
        assert_eq!(UfedFormat::Ufd.to_string(), "UFD");
    }
}
