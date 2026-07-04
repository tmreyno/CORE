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
//! use ffx_common::vfs::VirtualFileSystem;
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

use super::detection::detect_archive_format;
use super::types::ArchiveFormat;
use ffx_common::vfs::{join_path, normalize_path, DirEntry, FileAttr, VfsError, VirtualFileSystem};

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

const ZIP_CENTRAL_DIRECTORY_ENTRY_HEADER_SIZE: usize = 46;
const ZIP_EOCD_RECORD_SIZE: usize = 22;
const ZIP_LOCAL_FILE_HEADER_SIZE: u64 = 30;
const ZIP_STREAM_BUFFER_BYTES: usize = 64 * 1024;
const ZIP32_SIZE_SENTINEL: u64 = u32::MAX as u64;

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

        // Only support ZIP natively (7z/RAR/TAR use libarchive backend)
        match format {
            ArchiveFormat::Zip | ArchiveFormat::Zip64 => {}
            ArchiveFormat::SevenZip
            | ArchiveFormat::Rar4
            | ArchiveFormat::Rar5
            | ArchiveFormat::Tar
            | ArchiveFormat::TarGz
            | ArchiveFormat::Iso => {
                // Supported via libarchive - will use load_libarchive_entries
            }
            ArchiveFormat::Gzip
            | ArchiveFormat::Xz
            | ArchiveFormat::Bzip2
            | ArchiveFormat::Lz4
            | ArchiveFormat::Zstd => {
                return Err(VfsError::Internal(format!(
                    "{} is a single-stream compression format without directory structure. \
                     Use decompression tools instead of VFS browsing.",
                    format
                )));
            }
            ArchiveFormat::Vmdk
            | ArchiveFormat::Vhd
            | ArchiveFormat::Vhdx
            | ArchiveFormat::Qcow2
            | ArchiveFormat::Vdi
            | ArchiveFormat::Dmg => {
                return Err(VfsError::Internal(format!(
                    "{} is a disk image format. Use the disk image VFS (EWF/raw) for browsing.",
                    format
                )));
            }
            _ => {
                return Err(VfsError::Internal(format!(
                    "VFS browsing is not applicable for {} format. \
                     This format does not contain a browseable directory structure.",
                    format
                )));
            }
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

        match self.format {
            ArchiveFormat::Zip | ArchiveFormat::Zip64 => {
                self.load_zip_entries()?;
            }
            _ => {
                self.load_libarchive_entries()?;
            }
        }

        self.loaded.store(true, Ordering::Release);

        Ok(())
    }

    /// Load entries from ZIP archive using central directory
    fn load_zip_entries(&self) -> Result<(), VfsError> {
        let mut file = File::open(&self.path).map_err(|e| VfsError::IoError(e.to_string()))?;

        let file_size = file
            .metadata()
            .map_err(|e| VfsError::IoError(e.to_string()))?
            .len();

        // Find EOCD (End of Central Directory)
        let search_size = file_size.min(65557) as usize;
        let mut buf = vec![0u8; search_size];

        file.seek(SeekFrom::End(-(search_size as i64)))
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        file.read_exact(&mut buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        let eocd_offset = find_zip_eocd_offset(&buf)
            .ok_or_else(|| VfsError::InvalidPath("ZIP EOCD not found".to_string()))?;

        // Parse EOCD
        // Offset 8: Total entries (2 bytes)
        // Offset 12: Central dir size (4 bytes)
        // Offset 16: Central dir offset (4 bytes)
        let entry_count = u16::from_le_bytes([buf[eocd_offset + 8], buf[eocd_offset + 9]]) as usize;
        let cd_size = u32::from_le_bytes([
            buf[eocd_offset + 12],
            buf[eocd_offset + 13],
            buf[eocd_offset + 14],
            buf[eocd_offset + 15],
        ]) as u64;
        let cd_offset = u32::from_le_bytes([
            buf[eocd_offset + 16],
            buf[eocd_offset + 17],
            buf[eocd_offset + 18],
            buf[eocd_offset + 19],
        ]) as u64;
        ensure_zip32_entry_count(entry_count)?;
        ensure_zip32_central_directory_fields(cd_size, cd_offset)?;

        // Read Central Directory
        checked_central_directory_bounds(cd_offset, cd_size, file_size)?;
        let mut cd_buf = vec![0u8; checked_central_directory_size(cd_size)?];
        file.seek(SeekFrom::Start(cd_offset))
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        file.read_exact(&mut cd_buf)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        // Parse Central Directory entries
        // Using DashMap - we can insert directly without acquiring a write lock

        let mut pos = 0usize;
        for _ in 0..entry_count {
            if checked_central_directory_header_end(pos, cd_buf.len()).is_none() {
                break;
            }

            // Verify signature (PK\x01\x02)
            if cd_buf[pos..pos + 4] != [0x50, 0x4B, 0x01, 0x02] {
                break;
            }

            // Parse entry header
            let compressed_size = u32::from_le_bytes([
                cd_buf[pos + 20],
                cd_buf[pos + 21],
                cd_buf[pos + 22],
                cd_buf[pos + 23],
            ]) as u64;
            let uncompressed_size = u32::from_le_bytes([
                cd_buf[pos + 24],
                cd_buf[pos + 25],
                cd_buf[pos + 26],
                cd_buf[pos + 27],
            ]) as u64;
            ensure_zip32_entry_sizes(compressed_size, uncompressed_size)?;
            let filename_len = u16::from_le_bytes([cd_buf[pos + 28], cd_buf[pos + 29]]) as usize;
            let extra_len = u16::from_le_bytes([cd_buf[pos + 30], cd_buf[pos + 31]]) as usize;
            let comment_len = u16::from_le_bytes([cd_buf[pos + 32], cd_buf[pos + 33]]) as usize;
            let external_attrs = u32::from_le_bytes([
                cd_buf[pos + 38],
                cd_buf[pos + 39],
                cd_buf[pos + 40],
                cd_buf[pos + 41],
            ]);
            let crc32 = u32::from_le_bytes([
                cd_buf[pos + 16],
                cd_buf[pos + 17],
                cd_buf[pos + 18],
                cd_buf[pos + 19],
            ]);

            // DOS time/date (not converting for now)
            let _mod_time = u16::from_le_bytes([cd_buf[pos + 12], cd_buf[pos + 13]]);
            let _mod_date = u16::from_le_bytes([cd_buf[pos + 14], cd_buf[pos + 15]]);

            // Get filename
            let Some((filename_start, filename_end)) =
                checked_central_directory_filename_range(pos, filename_len, cd_buf.len())
            else {
                break;
            };
            let filename =
                String::from_utf8_lossy(&cd_buf[filename_start..filename_end]).to_string();

            let is_dir = filename.ends_with('/') || (external_attrs >> 16) & 0x4000 != 0;
            let Some(normalized) = normalize_archive_entry_path(&filename, is_dir) else {
                let Some(next_pos) = checked_central_directory_next_pos(
                    pos,
                    filename_len,
                    extra_len,
                    comment_len,
                    cd_buf.len(),
                ) else {
                    break;
                };
                pos = next_pos;
                continue;
            };

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
            let parent = ffx_common::vfs::parent_path(&normalized).unwrap_or("/".to_string());

            // Ensure parent directories exist
            let mut current = parent.clone();
            while current != "/" && !self.entries.contains_key(&current) {
                let parent_inode = self.alloc_inode();
                self.entries.insert(
                    current.clone(),
                    ArchiveEntry {
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
                    },
                );
                self.dir_children.entry(current.clone()).or_default();

                let grandparent = ffx_common::vfs::parent_path(&current).unwrap_or("/".to_string());
                let name = ffx_common::vfs::filename(&current).to_string();
                self.dir_children.entry(grandparent.clone()).or_default();
                if let Some(mut children) = self.dir_children.get_mut(&grandparent) {
                    if !children.contains(&name) {
                        children.push(name);
                    }
                }
                current = grandparent;
            }

            // Add to parent's children
            let child_name = ffx_common::vfs::filename(&normalized).to_string();
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
            let Some(next_pos) = checked_central_directory_next_pos(
                pos,
                filename_len,
                extra_len,
                comment_len,
                cd_buf.len(),
            ) else {
                break;
            };
            pos = next_pos;
        }

        Ok(())
    }

    /// Load entries from non-ZIP archives using libarchive backend
    fn load_libarchive_entries(&self) -> Result<(), VfsError> {
        let entries = super::libarchive_list_all(&self.path)
            .map_err(|e| VfsError::IoError(format!("Failed to list archive entries: {}", e)))?;

        for entry_info in entries {
            let is_dir = entry_info.is_dir;
            let Some(normalized) = normalize_archive_entry_path(&entry_info.path, is_dir) else {
                continue;
            };

            let inode = self.alloc_inode();
            let entry = ArchiveEntry {
                attr: FileAttr {
                    size: if is_dir { 0 } else { entry_info.size },
                    is_directory: is_dir,
                    permissions: if is_dir { 0o555 } else { 0o444 },
                    nlink: if is_dir { 2 } else { 1 },
                    inode,
                    ..Default::default()
                },
                index: entry_info.index,
                compressed_size: 0,
                crc32: None,
            };

            self.entries.insert(normalized.clone(), entry);

            // Update directory children
            let parent = ffx_common::vfs::parent_path(&normalized).unwrap_or("/".to_string());

            // Ensure parent directories exist
            let mut current = parent.clone();
            while current != "/" && !self.entries.contains_key(&current) {
                let parent_inode = self.alloc_inode();
                self.entries.insert(
                    current.clone(),
                    ArchiveEntry {
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
                    },
                );
                self.dir_children.entry(current.clone()).or_default();

                let grandparent = ffx_common::vfs::parent_path(&current).unwrap_or("/".to_string());
                let name = ffx_common::vfs::filename(&current).to_string();
                self.dir_children.entry(grandparent.clone()).or_default();
                if let Some(mut children) = self.dir_children.get_mut(&grandparent) {
                    if !children.contains(&name) {
                        children.push(name);
                    }
                }
                current = grandparent;
            }

            // Add to parent's children
            let child_name = ffx_common::vfs::filename(&normalized).to_string();
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
        }

        Ok(())
    }

    /// Read file data from a non-ZIP archive using libarchive
    fn read_libarchive_file(
        &self,
        path: &str,
        offset: u64,
        size: usize,
    ) -> Result<Vec<u8>, VfsError> {
        let entry = self
            .entries
            .get(path)
            .ok_or_else(|| VfsError::NotFound(path.to_string()))?;

        if entry.attr.is_directory {
            return Err(VfsError::NotAFile(path.to_string()));
        }

        let entry_path = path.trim_start_matches('/');
        let data = super::libarchive_read_file(&self.path, entry_path)
            .map_err(|e| VfsError::IoError(format!("Failed to read archive entry: {}", e)))?;

        let Some((start, end)) = bounded_read_range(offset, size, data.len())? else {
            return Ok(Vec::new());
        };
        Ok(data[start..end].to_vec())
    }

    /// Read file data from archive
    fn read_zip_file(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        // For now, use the zip crate if available, or implement manual extraction
        // This is a simplified implementation that reads the local file header

        let mut file = File::open(&self.path).map_err(|e| VfsError::IoError(e.to_string()))?;

        // Get the entry to find its location
        let entry = self
            .entries
            .get(path)
            .ok_or_else(|| VfsError::NotFound(path.to_string()))?;

        if entry.attr.is_directory {
            return Err(VfsError::NotAFile(path.to_string()));
        }

        // We need to search for this file in the archive
        // For a proper implementation, we'd cache local header offsets
        // For now, return an error suggesting use of extract command

        // Search for the file by scanning local headers (expensive but works)
        let file_size = file
            .metadata()
            .map_err(|e| VfsError::IoError(e.to_string()))?
            .len();

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
            let compressed_size =
                u32::from_le_bytes([header[14], header[15], header[16], header[17]]) as u64;
            let uncompressed_size =
                u32::from_le_bytes([header[18], header[19], header[20], header[21]]) as u64;
            ensure_zip32_entry_sizes(compressed_size, uncompressed_size)?;
            let filename_len = usize::from(u16::from_le_bytes([header[22], header[23]]));
            let extra_len = usize::from(u16::from_le_bytes([header[24], header[25]]));
            let next_pos = checked_local_file_entry_end(
                pos,
                filename_len,
                extra_len,
                compressed_size,
                file_size,
            )?;

            // Read filename
            let mut filename_buf = vec![0u8; filename_len];
            file.read_exact(&mut filename_buf)
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            let filename = String::from_utf8_lossy(&filename_buf);

            // Skip extra field
            file.seek(SeekFrom::Current(i64::try_from(extra_len).map_err(
                |_| VfsError::InvalidPath("ZIP extra field too large".to_string()),
            )?))
            .map_err(|e| VfsError::IoError(e.to_string()))?;

            if normalize_archive_entry_path(&filename, false).as_deref() == Some(path) {
                if compression == 0 {
                    let Some((start, end)) =
                        bounded_read_range_u64(offset, size, uncompressed_size)?
                    else {
                        return Ok(Vec::new());
                    };
                    let data_offset = pos
                        .checked_add(ZIP_LOCAL_FILE_HEADER_SIZE)
                        .and_then(|offset| offset.checked_add(u64::try_from(filename_len).ok()?))
                        .and_then(|offset| offset.checked_add(u64::try_from(extra_len).ok()?))
                        .and_then(|offset| offset.checked_add(start))
                        .ok_or_else(|| {
                            VfsError::InvalidPath("ZIP stored entry range overflow".to_string())
                        })?;
                    file.seek(SeekFrom::Start(data_offset))
                        .map_err(|e| VfsError::IoError(e.to_string()))?;
                    let read_size = usize::try_from(end - start).map_err(|_| {
                        VfsError::InvalidPath("ZIP stored range too large".to_string())
                    })?;
                    let mut data = vec![0u8; read_size];
                    file.read_exact(&mut data)
                        .map_err(|e| VfsError::IoError(e.to_string()))?;
                    return Ok(data);
                }

                if compression == 8 {
                    return read_deflated_zip_range(
                        &mut file,
                        compressed_size,
                        uncompressed_size,
                        offset,
                        size,
                    );
                } else {
                    return Err(VfsError::Internal(format!(
                        "Unsupported compression method: {}",
                        compression
                    )));
                }
            }

            // Move to next entry
            pos = next_pos;
        }

        Err(VfsError::NotFound(path.to_string()))
    }
}

impl VirtualFileSystem for ArchiveVfs {
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        self.ensure_loaded()?;

        let normalized = normalize_path(path);

        self.entries
            .get(&normalized)
            .map(|e| e.attr.clone())
            .ok_or(VfsError::NotFound(normalized))
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        self.ensure_loaded()?;

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
        self.ensure_loaded()?;

        let normalized = normalize_path(path);
        match self.format {
            ArchiveFormat::Zip | ArchiveFormat::Zip64 => {
                self.read_zip_file(&normalized, offset, size)
            }
            _ => self.read_libarchive_file(&normalized, offset, size),
        }
    }
}

fn bounded_read_range(
    offset: u64,
    size: usize,
    len: usize,
) -> Result<Option<(usize, usize)>, VfsError> {
    let len_u64 = u64::try_from(len)
        .map_err(|_| VfsError::InvalidPath("Archive entry length too large".to_string()))?;
    if offset > len_u64 {
        return Err(VfsError::OutOfBounds { offset, size });
    }
    if offset == len_u64 || size == 0 {
        return Ok(None);
    }

    let start = usize::try_from(offset)
        .map_err(|_| VfsError::InvalidPath("Archive range offset too large".to_string()))?;
    let end = start.saturating_add(size).min(len);
    Ok(Some((start, end)))
}

fn bounded_read_range_u64(
    offset: u64,
    size: usize,
    len: u64,
) -> Result<Option<(u64, u64)>, VfsError> {
    if offset > len {
        return Err(VfsError::OutOfBounds { offset, size });
    }
    if offset == len || size == 0 {
        return Ok(None);
    }

    let end = offset.saturating_add(size as u64).min(len);
    Ok(Some((offset, end)))
}

fn normalize_archive_entry_path(entry_path: &str, is_dir: bool) -> Option<String> {
    if entry_path.contains('\0') {
        return None;
    }

    let entry_path = entry_path.replace('\\', "/");
    if entry_path.starts_with('/') || entry_path.starts_with("~/") {
        return None;
    }

    let trimmed = if is_dir {
        entry_path.trim_end_matches('/')
    } else {
        entry_path.as_str()
    };

    let mut parts = Vec::new();
    for part in trimmed.split('/') {
        match part {
            "" | "." => continue,
            ".." => return None,
            _ if part.ends_with(':') => return None,
            _ => parts.push(part),
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(format!("/{}", parts.join("/")))
}

fn read_deflated_zip_range(
    file: &mut File,
    compressed_size: u64,
    uncompressed_size: u64,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, VfsError> {
    let Some((start, end)) = bounded_read_range_u64(offset, size, uncompressed_size)? else {
        return Ok(Vec::new());
    };
    let reader = file.by_ref().take(compressed_size);
    let decoder = flate2::read::DeflateDecoder::new(reader);
    read_uncompressed_range(decoder, start, end)
}

fn read_uncompressed_range(
    mut reader: impl Read,
    start: u64,
    end: u64,
) -> Result<Vec<u8>, VfsError> {
    let output_size = usize::try_from(end.saturating_sub(start))
        .map_err(|_| VfsError::InvalidPath("ZIP decompressed range too large".to_string()))?;
    if output_size == 0 {
        return Ok(Vec::new());
    }

    let mut output = Vec::with_capacity(output_size);
    let mut position = 0u64;
    let mut buffer = [0u8; ZIP_STREAM_BUFFER_BYTES];

    while position < end {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| VfsError::IoError(format!("Decompression failed: {}", e)))?;
        if bytes_read == 0 {
            return Err(VfsError::IoError(
                "Decompressed ZIP entry ended before requested range".to_string(),
            ));
        }

        let chunk_start = position;
        let chunk_end = position.saturating_add(bytes_read as u64);
        if chunk_end > start {
            let copy_start = start.saturating_sub(chunk_start) as usize;
            let copy_end = end.min(chunk_end).saturating_sub(chunk_start) as usize;
            output.extend_from_slice(&buffer[copy_start..copy_end]);
        }
        position = chunk_end;
    }

    Ok(output)
}

fn checked_central_directory_size(cd_size: u64) -> Result<usize, VfsError> {
    usize::try_from(cd_size)
        .map_err(|_| VfsError::InvalidPath("ZIP central directory too large".to_string()))
}

fn find_zip_eocd_offset(buf: &[u8]) -> Option<usize> {
    if buf.len() < ZIP_EOCD_RECORD_SIZE {
        return None;
    }

    let eocd_sig = [0x50, 0x4B, 0x05, 0x06];
    for offset in (0..=buf.len() - ZIP_EOCD_RECORD_SIZE).rev() {
        if buf[offset..offset + 4] != eocd_sig {
            continue;
        }

        let comment_len = usize::from(u16::from_le_bytes([buf[offset + 20], buf[offset + 21]]));
        let Some(record_end) = offset
            .checked_add(ZIP_EOCD_RECORD_SIZE)
            .and_then(|end| end.checked_add(comment_len))
        else {
            continue;
        };

        if record_end == buf.len() {
            return Some(offset);
        }
    }

    None
}

fn ensure_zip32_central_directory_fields(cd_size: u64, cd_offset: u64) -> Result<(), VfsError> {
    if cd_size == ZIP32_SIZE_SENTINEL || cd_offset == ZIP32_SIZE_SENTINEL {
        return Err(VfsError::InvalidPath(
            "ZIP64 central directory fields are not supported by archive VFS".to_string(),
        ));
    }

    Ok(())
}

fn ensure_zip32_entry_count(entry_count: usize) -> Result<(), VfsError> {
    if entry_count == u16::MAX as usize {
        return Err(VfsError::InvalidPath(
            "ZIP64 entry count fields are not supported by archive VFS".to_string(),
        ));
    }

    Ok(())
}

fn ensure_zip32_entry_sizes(compressed_size: u64, uncompressed_size: u64) -> Result<(), VfsError> {
    if compressed_size == ZIP32_SIZE_SENTINEL || uncompressed_size == ZIP32_SIZE_SENTINEL {
        return Err(VfsError::InvalidPath(
            "ZIP64 entry size fields are not supported by archive VFS".to_string(),
        ));
    }

    Ok(())
}

fn checked_central_directory_bounds(
    cd_offset: u64,
    cd_size: u64,
    file_size: u64,
) -> Result<(), VfsError> {
    let cd_end = cd_offset.checked_add(cd_size).ok_or_else(|| {
        VfsError::InvalidPath("ZIP central directory bounds overflow".to_string())
    })?;

    if cd_end > file_size {
        return Err(VfsError::InvalidPath(
            "ZIP central directory exceeds archive bounds".to_string(),
        ));
    }

    Ok(())
}

fn checked_central_directory_header_end(pos: usize, buf_len: usize) -> Option<usize> {
    pos.checked_add(ZIP_CENTRAL_DIRECTORY_ENTRY_HEADER_SIZE)
        .filter(|end| *end <= buf_len)
}

fn checked_central_directory_filename_range(
    pos: usize,
    filename_len: usize,
    buf_len: usize,
) -> Option<(usize, usize)> {
    let start = checked_central_directory_header_end(pos, buf_len)?;
    let end = start.checked_add(filename_len)?;
    (end <= buf_len).then_some((start, end))
}

fn checked_central_directory_next_pos(
    pos: usize,
    filename_len: usize,
    extra_len: usize,
    comment_len: usize,
    buf_len: usize,
) -> Option<usize> {
    let next = pos
        .checked_add(ZIP_CENTRAL_DIRECTORY_ENTRY_HEADER_SIZE)?
        .checked_add(filename_len)?
        .checked_add(extra_len)?
        .checked_add(comment_len)?;
    (next <= buf_len).then_some(next)
}

fn checked_local_file_entry_end(
    pos: u64,
    filename_len: usize,
    extra_len: usize,
    compressed_size: u64,
    file_size: u64,
) -> Result<u64, VfsError> {
    let filename_len = u64::try_from(filename_len)
        .map_err(|_| VfsError::InvalidPath("ZIP filename too large".to_string()))?;
    let extra_len = u64::try_from(extra_len)
        .map_err(|_| VfsError::InvalidPath("ZIP extra field too large".to_string()))?;

    let end = pos
        .checked_add(ZIP_LOCAL_FILE_HEADER_SIZE)
        .and_then(|offset| offset.checked_add(filename_len))
        .and_then(|offset| offset.checked_add(extra_len))
        .and_then(|offset| offset.checked_add(compressed_size))
        .ok_or_else(|| VfsError::InvalidPath("ZIP local entry bounds overflow".to_string()))?;

    if end > file_size {
        return Err(VfsError::InvalidPath(
            "ZIP local entry exceeds archive bounds".to_string(),
        ));
    }

    Ok(end)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    fn write_u16(file: &mut File, value: u16) {
        file.write_all(&value.to_le_bytes()).unwrap();
    }

    fn write_u32(file: &mut File, value: u32) {
        file.write_all(&value.to_le_bytes()).unwrap();
    }

    fn create_sparse_stored_zip(path: &Path, entry_name: &str, entry_size: u32) {
        let mut file = File::create(path).unwrap();
        let name = entry_name.as_bytes();
        let name_len = u16::try_from(name.len()).unwrap();

        write_u32(&mut file, 0x0403_4b50);
        write_u16(&mut file, 20);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u32(&mut file, 0);
        write_u32(&mut file, entry_size);
        write_u32(&mut file, entry_size);
        write_u16(&mut file, name_len);
        write_u16(&mut file, 0);
        file.write_all(name).unwrap();

        let data_start = ZIP_LOCAL_FILE_HEADER_SIZE + u64::from(name_len);
        file.write_all(b"head").unwrap();
        file.set_len(data_start + u64::from(entry_size)).unwrap();
        file.seek(SeekFrom::Start(data_start + u64::from(entry_size)))
            .unwrap();

        let cd_offset = file.stream_position().unwrap();
        write_u32(&mut file, 0x0201_4b50);
        write_u16(&mut file, 20);
        write_u16(&mut file, 20);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u32(&mut file, 0);
        write_u32(&mut file, entry_size);
        write_u32(&mut file, entry_size);
        write_u16(&mut file, name_len);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u32(&mut file, 0);
        write_u32(&mut file, 0);
        file.write_all(name).unwrap();

        let cd_size = file.stream_position().unwrap() - cd_offset;
        write_u32(&mut file, 0x0605_4b50);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 1);
        write_u16(&mut file, 1);
        write_u32(&mut file, u32::try_from(cd_size).unwrap());
        write_u32(&mut file, u32::try_from(cd_offset).unwrap());
        write_u16(&mut file, 0);
    }

    fn create_single_file_zip(
        path: &Path,
        entry_name: &str,
        data: &[u8],
        compression: u16,
        compressed_data: &[u8],
    ) {
        let mut file = File::create(path).unwrap();
        let name = entry_name.as_bytes();
        let name_len = u16::try_from(name.len()).unwrap();
        let data_len = u32::try_from(data.len()).unwrap();
        let compressed_len = u32::try_from(compressed_data.len()).unwrap();

        write_u32(&mut file, 0x0403_4b50);
        write_u16(&mut file, 20);
        write_u16(&mut file, 0);
        write_u16(&mut file, compression);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u32(&mut file, 0);
        write_u32(&mut file, compressed_len);
        write_u32(&mut file, data_len);
        write_u16(&mut file, name_len);
        write_u16(&mut file, 0);
        file.write_all(name).unwrap();
        file.write_all(compressed_data).unwrap();

        let cd_offset = file.stream_position().unwrap();
        write_u32(&mut file, 0x0201_4b50);
        write_u16(&mut file, 20);
        write_u16(&mut file, 20);
        write_u16(&mut file, 0);
        write_u16(&mut file, compression);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u32(&mut file, 0);
        write_u32(&mut file, compressed_len);
        write_u32(&mut file, data_len);
        write_u16(&mut file, name_len);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u32(&mut file, 0);
        write_u32(&mut file, 0);
        file.write_all(name).unwrap();

        let cd_size = file.stream_position().unwrap() - cd_offset;
        write_u32(&mut file, 0x0605_4b50);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, 1);
        write_u16(&mut file, 1);
        write_u32(&mut file, u32::try_from(cd_size).unwrap());
        write_u32(&mut file, u32::try_from(cd_offset).unwrap());
        write_u16(&mut file, 0);
    }

    fn create_stored_zip_entries(path: &Path, entries: &[(&str, &[u8])]) {
        let mut file = File::create(path).unwrap();
        let mut central_entries = Vec::new();

        for (name, data) in entries {
            let header_offset = file.stream_position().unwrap();
            let name = name.as_bytes();
            let name_len = u16::try_from(name.len()).unwrap();
            let data_len = u32::try_from(data.len()).unwrap();

            write_u32(&mut file, 0x0403_4b50);
            write_u16(&mut file, 20);
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u32(&mut file, 0);
            write_u32(&mut file, data_len);
            write_u32(&mut file, data_len);
            write_u16(&mut file, name_len);
            write_u16(&mut file, 0);
            file.write_all(name).unwrap();
            file.write_all(data).unwrap();

            central_entries.push((header_offset, name.to_vec(), data_len));
        }

        let cd_offset = file.stream_position().unwrap();
        for (header_offset, name, data_len) in &central_entries {
            write_u32(&mut file, 0x0201_4b50);
            write_u16(&mut file, 20);
            write_u16(&mut file, 20);
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u32(&mut file, 0);
            write_u32(&mut file, *data_len);
            write_u32(&mut file, *data_len);
            write_u16(&mut file, u16::try_from(name.len()).unwrap());
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u16(&mut file, 0);
            write_u32(&mut file, 0);
            write_u32(&mut file, u32::try_from(*header_offset).unwrap());
            file.write_all(name).unwrap();
        }

        let cd_size = file.stream_position().unwrap() - cd_offset;
        write_u32(&mut file, 0x0605_4b50);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, u16::try_from(entries.len()).unwrap());
        write_u16(&mut file, u16::try_from(entries.len()).unwrap());
        write_u32(&mut file, u32::try_from(cd_size).unwrap());
        write_u32(&mut file, u32::try_from(cd_offset).unwrap());
        write_u16(&mut file, 0);
    }

    fn deflate_bytes(data: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn test_archive_vfs_path_normalization() {
        // Just test that normalize_path works as expected
        assert_eq!(normalize_path("/foo/bar"), "/foo/bar");
        assert_eq!(normalize_path("foo/bar"), "/foo/bar");
        assert_eq!(normalize_path("/foo/bar/"), "/foo/bar");
    }

    #[test]
    fn test_bounded_read_range_allows_exact_eof() {
        assert_eq!(bounded_read_range(8, 4, 8).unwrap(), None);
    }

    #[test]
    fn test_bounded_read_range_rejects_offset_past_end() {
        let err = bounded_read_range(10, 4, 8).unwrap_err();
        assert!(matches!(
            err,
            VfsError::OutOfBounds {
                offset: 10,
                size: 4
            }
        ));
    }

    #[test]
    fn test_bounded_read_range_saturates_large_size() {
        assert_eq!(bounded_read_range(6, usize::MAX, 8).unwrap(), Some((6, 8)));
    }

    #[test]
    fn test_bounded_read_range_u64_saturates_large_size() {
        assert_eq!(
            bounded_read_range_u64(2 * 1024 * 1024 * 1024, usize::MAX, 3 * 1024 * 1024 * 1024)
                .unwrap(),
            Some((2 * 1024 * 1024 * 1024, 3 * 1024 * 1024 * 1024))
        );
    }

    #[test]
    fn archive_entry_path_normalization_rejects_traversal() {
        assert_eq!(
            normalize_archive_entry_path("dir\\file.txt", false),
            Some("/dir/file.txt".to_string())
        );
        assert_eq!(normalize_archive_entry_path("../evil.txt", false), None);
        assert_eq!(
            normalize_archive_entry_path("safe/../../evil.txt", false),
            None
        );
        assert_eq!(normalize_archive_entry_path("/absolute.txt", false), None);
        assert_eq!(normalize_archive_entry_path("C:/absolute.txt", false), None);
    }

    #[test]
    fn archive_vfs_reads_sparse_stored_zip_range_without_full_materialization() {
        let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        let entry_size = 2_u32 * 1024 * 1024 * 1024;
        create_sparse_stored_zip(tmp.path(), "files/large.bin", entry_size);

        let vfs = ArchiveVfs::open(tmp.path().to_str().unwrap()).unwrap();

        assert_eq!(vfs.read("/files/large.bin", 0, 4).unwrap(), b"head");
        assert_eq!(
            vfs.read("/files/large.bin", u64::from(entry_size) - 2, 8)
                .unwrap(),
            vec![0, 0]
        );
    }

    #[test]
    fn archive_vfs_streams_deflated_zip_range() {
        let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        let mut data = Vec::with_capacity(8 * 1024 * 1024);
        for i in 0..(8 * 1024 * 1024) {
            data.push((i % 251) as u8);
        }
        let compressed = deflate_bytes(&data);
        create_single_file_zip(tmp.path(), "files/deflated.bin", &data, 8, &compressed);

        let vfs = ArchiveVfs::open(tmp.path().to_str().unwrap()).unwrap();
        let offset = 3 * 1024 * 1024 + 17;

        assert_eq!(
            vfs.read("/files/deflated.bin", offset as u64, 64).unwrap(),
            data[offset..offset + 64]
        );
        assert_eq!(
            vfs.read("/files/deflated.bin", data.len() as u64, 8)
                .unwrap(),
            b""
        );
    }

    #[test]
    fn archive_vfs_skips_traversal_entries_and_reads_normalized_backslashes() {
        let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        create_stored_zip_entries(
            tmp.path(),
            &[
                ("../evil.txt", b"evil"),
                ("dir\\safe.txt", b"safe"),
                ("notes/readme..txt", b"dots"),
            ],
        );

        let vfs = ArchiveVfs::open(tmp.path().to_str().unwrap()).unwrap();

        assert!(vfs.getattr("/evil.txt").is_err());
        assert_eq!(vfs.read("/dir/safe.txt", 0, 4).unwrap(), b"safe");
        assert_eq!(vfs.read("/notes/readme..txt", 0, 4).unwrap(), b"dots");
    }

    #[test]
    fn test_checked_central_directory_bounds_rejects_overflow() {
        assert!(checked_central_directory_bounds(u64::MAX, 1, u64::MAX).is_err());
    }

    #[test]
    fn find_zip_eocd_offset_rejects_short_trailing_signature() {
        let buf = b"not a zip PK\x05\x06";

        assert_eq!(find_zip_eocd_offset(buf), None);
    }

    #[test]
    fn find_zip_eocd_offset_accepts_record_with_matching_comment() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"prefix");
        let offset = buf.len();
        buf.extend_from_slice(&[
            0x50, 0x4B, 0x05, 0x06, // signature
            0, 0, // disk number
            0, 0, // central directory disk
            0, 0, // entries on disk
            0, 0, // total entries
            0, 0, 0, 0, // central directory size
            0, 0, 0, 0, // central directory offset
            4, 0, // comment length
        ]);
        buf.extend_from_slice(b"note");

        assert_eq!(find_zip_eocd_offset(&buf), Some(offset));
    }

    #[test]
    fn find_zip_eocd_offset_ignores_comment_signature_suffix() {
        let mut buf = Vec::new();
        let offset = buf.len();
        buf.extend_from_slice(&[
            0x50, 0x4B, 0x05, 0x06, // signature
            0, 0, // disk number
            0, 0, // central directory disk
            0, 0, // entries on disk
            0, 0, // total entries
            0, 0, 0, 0, // central directory size
            0, 0, 0, 0, // central directory offset
            5, 0, // comment length
        ]);
        buf.extend_from_slice(b"aPK\x05\x06");

        assert_eq!(find_zip_eocd_offset(&buf), Some(offset));
    }

    #[test]
    fn zip32_central_directory_fields_reject_zip64_sentinels() {
        assert!(ensure_zip32_central_directory_fields(ZIP32_SIZE_SENTINEL, 0).is_err());
        assert!(ensure_zip32_central_directory_fields(0, ZIP32_SIZE_SENTINEL).is_err());
        assert!(ensure_zip32_central_directory_fields(1024, 2048).is_ok());
    }

    #[test]
    fn zip32_entry_count_rejects_zip64_sentinel() {
        assert!(ensure_zip32_entry_count(u16::MAX as usize).is_err());
        assert!(ensure_zip32_entry_count(42).is_ok());
    }

    #[test]
    fn archive_vfs_rejects_zip64_entry_count_sentinel() {
        let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        let mut file = File::create(tmp.path()).unwrap();
        write_u32(&mut file, 0x0605_4b50);
        write_u16(&mut file, 0);
        write_u16(&mut file, 0);
        write_u16(&mut file, u16::MAX);
        write_u16(&mut file, u16::MAX);
        write_u32(&mut file, 0);
        write_u32(&mut file, 0);
        write_u16(&mut file, 0);
        drop(file);

        let vfs = ArchiveVfs {
            path: tmp.path().to_string_lossy().to_string(),
            format: ArchiveFormat::Zip,
            entries: DashMap::new(),
            dir_children: DashMap::new(),
            next_inode: AtomicU64::new(2),
            loaded: AtomicBool::new(false),
        };
        vfs.init_root().unwrap();

        let err = vfs
            .load_zip_entries()
            .expect_err("ZIP64 entry count sentinel should be rejected");

        assert!(matches!(err, VfsError::InvalidPath(message) if message.contains("entry count")));
    }

    #[test]
    fn zip32_entry_sizes_reject_zip64_sentinels() {
        assert!(ensure_zip32_entry_sizes(ZIP32_SIZE_SENTINEL, 0).is_err());
        assert!(ensure_zip32_entry_sizes(0, ZIP32_SIZE_SENTINEL).is_err());
        assert!(ensure_zip32_entry_sizes(1024, 2048).is_ok());
    }

    #[test]
    fn test_checked_central_directory_filename_range_rejects_overflow() {
        assert_eq!(
            checked_central_directory_filename_range(usize::MAX, 1, usize::MAX),
            None
        );
    }

    #[test]
    fn test_checked_central_directory_next_pos_rejects_overflow() {
        assert_eq!(
            checked_central_directory_next_pos(usize::MAX, 1, 0, 0, usize::MAX),
            None
        );
    }

    #[test]
    fn test_checked_local_file_entry_end_rejects_overflow() {
        assert!(checked_local_file_entry_end(u64::MAX, 1, 0, 0, u64::MAX).is_err());
    }

    #[test]
    fn test_checked_local_file_entry_end_rejects_out_of_bounds() {
        assert!(checked_local_file_entry_end(10, 4, 4, 4, 20).is_err());
    }
}
