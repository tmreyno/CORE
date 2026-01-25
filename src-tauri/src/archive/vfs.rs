// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Archive Virtual Filesystem Implementation
//!
//! ## Section Brief
//! Read-only virtual filesystem implementation for archive containers (ZIP, 7z).
//! Provides safe, corruption-proof access to archive contents.
//!
//! ### Key Types
//! - `ArchiveVfs` - Virtual filesystem for ZIP/7z archives
//! - `ArchiveEntry` - Cached entry metadata
//!
//! ### Features
//! - Read-only access prevents archive corruption
//! - Lazy extraction of file contents
//! - Directory structure from archive manifest
//! - Path traversal prevention
//!
//! ### Usage
//! ```rust,ignore
//! use crate::archive::vfs::ArchiveVfs;
//! use crate::common::vfs::VirtualFileSystem;
//!
//! let vfs = ArchiveVfs::open("/path/to/archive.zip")?;
//!
//! // List root directory
//! let entries = vfs.readdir("/")?;
//!
//! // Read a file
//! let data = vfs.read("/Documents/file.txt", 0, 1024)?;
//! ```

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use dashmap::DashMap;

use crate::common::vfs::{VirtualFileSystem, VfsError, FileAttr, DirEntry, normalize_path, join_path};
use super::types::ArchiveFormat;
use super::detection::detect_archive_format;

// =============================================================================
// Archive Virtual Filesystem
// =============================================================================

/// Virtual filesystem implementation for archive containers
/// 
/// Provides read-only access to ZIP and 7z archive contents through a 
/// filesystem-like interface. All operations are safe and cannot
/// modify the underlying archive.
/// 
/// Uses `DashMap` for lock-free concurrent access to cached entries.
pub struct ArchiveVfs {
    /// Archive path
    #[allow(dead_code)]
    path: String,
    /// Archive format
    #[allow(dead_code)]
    format: ArchiveFormat,
    /// Entry tree (path -> entry info) - lock-free concurrent map
    entries: DashMap<String, ArchiveEntry>,
    /// Directory children map (dir_path -> child names) - lock-free concurrent map
    dir_children: DashMap<String, Vec<String>>,
    /// Next synthetic inode number - atomic for lock-free increment
    next_inode: AtomicU64,
    /// Whether entries have been loaded - atomic for lock-free check
    loaded: AtomicBool,
}

/// Cached archive entry information
#[derive(Clone)]
struct ArchiveEntry {
    /// File attributes
    attr: FileAttr,
    /// Index in archive (for extraction)
    #[allow(dead_code)]
    index: usize,
    /// Compressed size
    #[allow(dead_code)]
    compressed_size: u64,
    /// CRC32 if available
    #[allow(dead_code)]
    crc32: Option<u32>,
}

impl ArchiveVfs {
    /// Open an archive for virtual filesystem access
    pub fn open(path: &str) -> Result<Self, VfsError> {
        // Verify file exists
        if !std::path::Path::new(path).exists() {
            return Err(VfsError::NotFound(path.to_string()));
        }
        
        // Detect format
        let format = detect_archive_format(path)
            .map_err(|e| VfsError::IoError(e.to_string()))?
            .ok_or_else(|| VfsError::InvalidPath(format!("Not a supported archive: {}", path)))?;
        
        // Only support ZIP for now (7z requires external library)
        match format {
            ArchiveFormat::Zip | ArchiveFormat::Zip64 => {}
            _ => return Err(VfsError::Internal(format!("VFS not yet supported for format: {}", format))),
        }
        
        let vfs = Self {
            path: path.to_string(),
            format,
            entries: DashMap::new(),
            dir_children: DashMap::new(),
            next_inode: AtomicU64::new(2), // 1 is reserved for root
            loaded: AtomicBool::new(false),
        };
        
        // Initialize root entry
        vfs.init_root()?;
        
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
        
        let root_entry = ArchiveEntry {
            attr: root_attr,
            index: 0,
            compressed_size: 0,
            crc32: None,
        };
        
        self.entries.insert("/".to_string(), root_entry);
        self.dir_children.insert("/".to_string(), Vec::new());
        
        Ok(())
    }

    /// Allocate a new inode number
    fn alloc_inode(&self) -> u64 {
        self.next_inode.fetch_add(1, Ordering::Relaxed)
    }

    /// Load all entries from the archive (lazy loading)
    fn ensure_loaded(&self) -> Result<(), VfsError> {
        // Fast check without lock
        if self.loaded.load(Ordering::Acquire) {
            return Ok(());
        }
        
        self.load_zip_entries()?;
        
        self.loaded.store(true, Ordering::Release);
        
        Ok(())
    }

    /// Load entries from ZIP archive using central directory
    fn load_zip_entries(&self) -> Result<(), VfsError> {
        let mut file = File::open(&self.path)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        let file_size = file.metadata()
            .map_err(|e| VfsError::IoError(e.to_string()))?
            .len();
        
        // Find EOCD (End of Central Directory)
        let search_size = file_size.min(65557) as usize;
        let mut buf = vec![0u8; search_size];
        
        file.seek(SeekFrom::End(-(search_size as i64)))
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        file.read_exact(&mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        // Find EOCD signature (PK\x05\x06)
        let eocd_sig = [0x50, 0x4B, 0x05, 0x06];
        let eocd_offset = (0..buf.len().saturating_sub(4))
            .rev()
            .find(|&i| buf[i..i + 4] == eocd_sig)
            .ok_or_else(|| VfsError::InvalidPath("ZIP EOCD not found".to_string()))?;
        
        // Parse EOCD
        // Offset 8: Total entries (2 bytes)
        // Offset 12: Central dir size (4 bytes)
        // Offset 16: Central dir offset (4 bytes)
        let entry_count = u16::from_le_bytes([buf[eocd_offset + 8], buf[eocd_offset + 9]]) as usize;
        let cd_size = u32::from_le_bytes([
            buf[eocd_offset + 12], buf[eocd_offset + 13],
            buf[eocd_offset + 14], buf[eocd_offset + 15],
        ]) as u64;
        let cd_offset = u32::from_le_bytes([
            buf[eocd_offset + 16], buf[eocd_offset + 17],
            buf[eocd_offset + 18], buf[eocd_offset + 19],
        ]) as u64;
        
        // Read Central Directory
        let mut cd_buf = vec![0u8; cd_size as usize];
        file.seek(SeekFrom::Start(cd_offset))
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        file.read_exact(&mut cd_buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        // Parse Central Directory entries
        // Using DashMap - we can insert directly without acquiring a write lock
        
        let mut pos = 0usize;
        for _ in 0..entry_count {
            if pos + 46 > cd_buf.len() {
                break;
            }
            
            // Verify signature (PK\x01\x02)
            if cd_buf[pos..pos + 4] != [0x50, 0x4B, 0x01, 0x02] {
                break;
            }
            
            // Parse entry header
            let compressed_size = u32::from_le_bytes([
                cd_buf[pos + 20], cd_buf[pos + 21],
                cd_buf[pos + 22], cd_buf[pos + 23],
            ]) as u64;
            let uncompressed_size = u32::from_le_bytes([
                cd_buf[pos + 24], cd_buf[pos + 25],
                cd_buf[pos + 26], cd_buf[pos + 27],
            ]) as u64;
            let filename_len = u16::from_le_bytes([cd_buf[pos + 28], cd_buf[pos + 29]]) as usize;
            let extra_len = u16::from_le_bytes([cd_buf[pos + 30], cd_buf[pos + 31]]) as usize;
            let comment_len = u16::from_le_bytes([cd_buf[pos + 32], cd_buf[pos + 33]]) as usize;
            let external_attrs = u32::from_le_bytes([
                cd_buf[pos + 38], cd_buf[pos + 39],
                cd_buf[pos + 40], cd_buf[pos + 41],
            ]);
            let crc32 = u32::from_le_bytes([
                cd_buf[pos + 16], cd_buf[pos + 17],
                cd_buf[pos + 18], cd_buf[pos + 19],
            ]);
            
            // DOS time/date (not converting for now)
            let _mod_time = u16::from_le_bytes([cd_buf[pos + 12], cd_buf[pos + 13]]);
            let _mod_date = u16::from_le_bytes([cd_buf[pos + 14], cd_buf[pos + 15]]);
            
            // Get filename
            if pos + 46 + filename_len > cd_buf.len() {
                break;
            }
            let filename = String::from_utf8_lossy(&cd_buf[pos + 46..pos + 46 + filename_len]).to_string();
            
            // Normalize path
            let is_dir = filename.ends_with('/') || (external_attrs >> 16) & 0x4000 != 0;
            let normalized = normalize_path(&format!("/{}", filename.trim_end_matches('/')));
            
            // Create entry
            let inode = self.alloc_inode();
            let entry = ArchiveEntry {
                attr: FileAttr {
                    size: if is_dir { 0 } else { uncompressed_size },
                    is_directory: is_dir,
                    permissions: if is_dir { 0o555 } else { 0o444 },
                    nlink: if is_dir { 2 } else { 1 },
                    inode,
                    ..Default::default()
                },
                index: self.entries.len(),
                compressed_size,
                crc32: Some(crc32),
            };
            
            self.entries.insert(normalized.clone(), entry);
            
            // Update directory children
            let parent = crate::common::vfs::parent_path(&normalized).unwrap_or("/".to_string());
            
            // Ensure parent directories exist
            let mut current = parent.clone();
            while current != "/" && !self.entries.contains_key(&current) {
                let parent_inode = self.alloc_inode();
                self.entries.insert(current.clone(), ArchiveEntry {
                    attr: FileAttr {
                        size: 0,
                        is_directory: true,
                        permissions: 0o555,
                        nlink: 2,
                        inode: parent_inode,
                        ..Default::default()
                    },
                    index: 0,
                    compressed_size: 0,
                    crc32: None,
                });
                self.dir_children.entry(current.clone()).or_default();
                
                let grandparent = crate::common::vfs::parent_path(&current).unwrap_or("/".to_string());
                let name = crate::common::vfs::filename(&current).to_string();
                self.dir_children.entry(grandparent.clone()).or_default();
                if let Some(mut children) = self.dir_children.get_mut(&grandparent) {
                    if !children.contains(&name) {
                        children.push(name);
                    }
                }
                current = grandparent;
            }
            
            // Add to parent's children
            let child_name = crate::common::vfs::filename(&normalized).to_string();
            self.dir_children.entry(parent.clone()).or_default();
            if let Some(mut children) = self.dir_children.get_mut(&parent) {
                if !children.contains(&child_name) {
                    children.push(child_name);
                }
            }
            
            // If this is a directory, ensure it has a children entry
            if is_dir {
                self.dir_children.entry(normalized).or_default();
            }
            
            // Move to next entry
            pos += 46 + filename_len + extra_len + comment_len;
        }
        
        Ok(())
    }

    /// Read file data from archive
    fn read_zip_file(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        // For now, use the zip crate if available, or implement manual extraction
        // This is a simplified implementation that reads the local file header
        
        let mut file = File::open(&self.path)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        // Get the entry to find its location
        let entry = self.entries.get(path)
            .ok_or_else(|| VfsError::NotFound(path.to_string()))?;
        
        if entry.attr.is_directory {
            return Err(VfsError::NotAFile(path.to_string()));
        }
        
        // We need to search for this file in the archive
        // For a proper implementation, we'd cache local header offsets
        // For now, return an error suggesting use of extract command
        
        // Search for the file by scanning local headers (expensive but works)
        let file_size = file.metadata()
            .map_err(|e| VfsError::IoError(e.to_string()))?
            .len();
        
        let target_name = path.trim_start_matches('/');
        let mut pos = 0u64;
        
        while pos < file_size {
            file.seek(SeekFrom::Start(pos))
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            
            let mut sig = [0u8; 4];
            if file.read_exact(&mut sig).is_err() {
                break;
            }
            
            // Check for local file header (PK\x03\x04)
            if sig != [0x50, 0x4B, 0x03, 0x04] {
                break;
            }
            
            // Read local file header
            let mut header = [0u8; 26];
            file.read_exact(&mut header)
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            
            let compression = u16::from_le_bytes([header[4], header[5]]);
            let compressed_size = u32::from_le_bytes([header[14], header[15], header[16], header[17]]) as u64;
            let uncompressed_size = u32::from_le_bytes([header[18], header[19], header[20], header[21]]) as u64;
            let filename_len = u16::from_le_bytes([header[22], header[23]]) as usize;
            let extra_len = u16::from_le_bytes([header[24], header[25]]) as usize;
            
            // Read filename
            let mut filename_buf = vec![0u8; filename_len];
            file.read_exact(&mut filename_buf)
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            let filename = String::from_utf8_lossy(&filename_buf);
            
            // Skip extra field
            file.seek(SeekFrom::Current(extra_len as i64))
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            
            if filename.trim_end_matches('/') == target_name {
                // Found the file! Read and decompress
                let mut compressed_data = vec![0u8; compressed_size as usize];
                file.read_exact(&mut compressed_data)
                    .map_err(|e| VfsError::IoError(e.to_string()))?;
                
                let data = if compression == 0 {
                    // Stored (no compression)
                    compressed_data
                } else if compression == 8 {
                    // Deflate
                    use flate2::read::DeflateDecoder;
                    let mut decoder = DeflateDecoder::new(&compressed_data[..]);
                    let mut decompressed = Vec::with_capacity(uncompressed_size as usize);
                    decoder.read_to_end(&mut decompressed)
                        .map_err(|e| VfsError::IoError(format!("Decompression failed: {}", e)))?;
                    decompressed
                } else {
                    return Err(VfsError::Internal(format!("Unsupported compression method: {}", compression)));
                };
                
                // Apply offset and size
                let start = offset as usize;
                if start >= data.len() {
                    return Ok(Vec::new());
                }
                let end = (start + size).min(data.len());
                return Ok(data[start..end].to_vec());
            }
            
            // Move to next entry
            pos += 30 + filename_len as u64 + extra_len as u64 + compressed_size;
        }
        
        Err(VfsError::NotFound(path.to_string()))
    }
}

impl VirtualFileSystem for ArchiveVfs {
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        self.ensure_loaded()?;
        
        let normalized = normalize_path(path);
        
        self.entries.get(&normalized)
            .map(|e| e.attr.clone())
            .ok_or(VfsError::NotFound(normalized))
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        self.ensure_loaded()?;
        
        let normalized = normalize_path(path);
        
        // Verify it's a directory
        let entry = self.entries.get(&normalized)
            .ok_or_else(|| VfsError::NotFound(normalized.clone()))?;
        
        if !entry.attr.is_directory {
            return Err(VfsError::NotADirectory(normalized));
        }
        
        // Get children
        let children = self.dir_children.get(&normalized)
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
        self.ensure_loaded()?;
        
        let normalized = normalize_path(path);
        self.read_zip_file(&normalized, offset, size)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_vfs_path_normalization() {
        // Just test that normalize_path works as expected
        assert_eq!(normalize_path("/foo/bar"), "/foo/bar");
        assert_eq!(normalize_path("foo/bar"), "/foo/bar");
        assert_eq!(normalize_path("/foo/bar/"), "/foo/bar");
    }
}
