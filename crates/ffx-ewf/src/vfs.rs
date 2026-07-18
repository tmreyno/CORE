// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # EWF Virtual Filesystem Implementation
//!
//! ## Section Brief
//! Read-only virtual filesystem implementation for EWF containers (E01/L01/Ex01/Lx01).
//! Provides safe, corruption-proof access to disk image contents.
//!
//! ### Key Types
//! - `EwfVfs` - Virtual filesystem for EWF containers
//! - `EwfVfsMode` - Physical (raw sectors) vs Logical (file tree) vs Filesystem (parsed)
//!
//! ### Features
//! - Read-only access prevents container corruption
//! - Physical mode: Access raw disk sectors as single file
//! - Filesystem mode: Auto-detect and mount filesystems (NTFS, FAT, ext4)
//! - Partition support: MBR/GPT partition table parsing
//! - Logical mode: Access L01 file tree (when available)
//!
//! ### Usage
//! ```rust,ignore
//! use crate::ewf::vfs::EwfVfs;
//! use ffx_common::vfs::VirtualFileSystem;
//!
//! // Filesystem mode - access parsed file tree
//! let vfs = EwfVfs::open("/path/to/disk.E01")?;
//! let entries = vfs.readdir("/")?;  // Lists partitions or root files
//!
//! // Physical mode - access raw sectors only
//! let vfs = EwfVfs::open_physical("/path/to/disk.E01")?;
//! let mbr = vfs.read("/disk.raw", 0, 512)?;
//! ```

use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, RwLock};

use super::handle::EwfHandle;
use super::types::{checked_volume_media_size, VolumeSection};
use ffx_common::filesystem::{
    detect_partition_table, mount_filesystem, BlockDevice, BlockReader, PartitionEntry,
    PartitionTable, SeekableBlockDevice,
};
use ffx_common::vfs::{
    find_partition, normalize_path, DirEntry, FileAttr, MountedPartition, VfsError,
    VirtualFileSystem,
};

// =============================================================================
// EWF Virtual Filesystem
// =============================================================================

/// Mode for EWF virtual filesystem access
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EwfVfsMode {
    /// Physical mode - expose raw disk sectors as a single file
    Physical,
    /// Filesystem mode - parse and expose filesystem contents
    Filesystem,
    /// Logical mode - expose L01 logical evidence data as a single virtual file
    Logical,
}

/// Virtual filesystem implementation for EWF containers
///
/// For E01 (physical images):
/// - Filesystem mode (default): Parses partitions and mounts filesystems
/// - Physical mode: Exposes raw disk as single virtual file
///
/// For L01 (logical images): Exposes the embedded data stream as a single virtual file.
pub struct EwfVfs {
    /// Container path
    #[allow(dead_code)]
    path: String,
    /// EWF handle for data access (shared for block device)
    handle: Arc<RwLock<EwfHandle>>,
    /// Access mode
    mode: EwfVfsMode,
    /// Virtual file name for physical mode
    disk_filename: String,
    /// Mounted partitions (for filesystem mode)
    partitions: Vec<MountedPartition>,
    /// Partition table type (stored for future use in filesystem mode)
    #[allow(dead_code)]
    partition_table: Option<PartitionTable>,
}

impl EwfVfs {
    /// Open an EWF container in physical mode (raw disk access)
    pub fn open_physical(path: &str) -> Result<Self, VfsError> {
        // Verify file exists
        if !std::path::Path::new(path).exists() {
            return Err(VfsError::NotFound(path.to_string()));
        }

        let handle = EwfHandle::open(path).map_err(|e| VfsError::IoError(e.to_string()))?;

        // Generate filename from path
        let filename = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("disk");

        Ok(Self {
            path: path.to_string(),
            handle: Arc::new(RwLock::new(handle)),
            mode: EwfVfsMode::Physical,
            disk_filename: format!("{}.raw", filename),
            partitions: Vec::new(),
            partition_table: None,
        })
    }

    /// Open an EWF container in logical mode (L01/Lx01 data stream)
    ///
    /// Exposes the L01 logical evidence data as a single virtual file.
    /// Use this for L01/Lx01 logical evidence containers.
    pub fn open_logical(path: &str) -> Result<Self, VfsError> {
        if !std::path::Path::new(path).exists() {
            return Err(VfsError::NotFound(path.to_string()));
        }

        let handle = EwfHandle::open(path).map_err(|e| VfsError::IoError(e.to_string()))?;

        let filename = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("logical");

        Ok(Self {
            path: path.to_string(),
            handle: Arc::new(RwLock::new(handle)),
            mode: EwfVfsMode::Logical,
            disk_filename: format!("{}.raw", filename),
            partitions: Vec::new(),
            partition_table: None,
        })
    }

    /// Open an EWF container with filesystem parsing (auto-mount partitions)
    pub fn open_filesystem(path: &str) -> Result<Self, VfsError> {
        if !std::path::Path::new(path).exists() {
            return Err(VfsError::NotFound(path.to_string()));
        }

        let handle = EwfHandle::open(path).map_err(|e| VfsError::IoError(e.to_string()))?;

        let filename = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("disk");

        let handle = Arc::new(RwLock::new(handle));

        // Create block device wrapper
        let block_device = EwfBlockDevice {
            handle: Arc::clone(&handle),
        };

        // Detect partition table
        let partition_table = detect_partition_table(&block_device)?;

        tracing::debug!(
            "Detected partition table: {:?}, {} partitions",
            partition_table.table_type,
            partition_table.partitions.len()
        );

        // Mount filesystems on detected partitions
        let mut partitions = Vec::new();

        for entry in &partition_table.partitions {
            // Skip empty partitions
            if entry.size == 0 {
                continue;
            }

            tracing::debug!(
                "Attempting to mount partition {}: offset={}, size={}, type={:?}",
                entry.number,
                entry.start_offset,
                entry.size,
                entry.filesystem_type
            );

            // Try to mount the filesystem
            let device = Box::new(EwfBlockDevice {
                handle: Arc::clone(&handle),
            });

            match mount_filesystem(device, entry.start_offset, entry.size) {
                Ok(fs) => {
                    let fs_type = fs.info().fs_type.to_string();
                    let mount_name = format!("Partition{}_{}", entry.number, fs_type);
                    tracing::info!("Successfully mounted {} as {}", mount_name, fs_type);
                    partitions.push(MountedPartition {
                        entry: entry.clone(),
                        fs,
                        mount_name,
                    });
                }
                Err(e) => {
                    // Filesystem not supported or corrupt, log and skip
                    tracing::warn!("Failed to mount partition {}: {:?}", entry.number, e);
                    continue;
                }
            }
        }

        // If no partitions found, try mounting whole disk as filesystem
        if partitions.is_empty() {
            tracing::debug!("No partitions mounted, attempting whole-disk filesystem mount");

            let device = Box::new(EwfBlockDevice {
                handle: Arc::clone(&handle),
            });

            let disk_size = {
                let h = handle
                    .read()
                    .map_err(|e| VfsError::Internal(e.to_string()))?;
                ewf_media_size_for_vfs(&h.volume)?
            };

            tracing::debug!("Trying whole-disk mount: size={}", disk_size);

            match mount_filesystem(device, 0, disk_size) {
                Ok(fs) => {
                    let fs_type = fs.info().fs_type.to_string();
                    tracing::info!("Successfully mounted whole disk as {}", fs_type);
                    partitions.push(MountedPartition {
                        entry: PartitionEntry {
                            number: 0,
                            start_offset: 0,
                            size: disk_size,
                            partition_type: "Whole Disk".to_string(),
                            bootable: false,
                            name: Some("Whole Disk".to_string()),
                            filesystem_type: Some(fs.info().fs_type),
                        },
                        fs,
                        mount_name: format!("Volume_{}", fs_type),
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to mount whole disk as filesystem: {:?}", e);
                }
            }
        }

        Ok(Self {
            path: path.to_string(),
            handle,
            mode: EwfVfsMode::Filesystem,
            disk_filename: format!("{}.raw", filename),
            partitions,
            partition_table: Some(partition_table),
        })
    }

    /// Open an EWF container (auto-detect mode)
    /// - L01/Lx01 files → Logical mode (single data stream)
    /// - E01/Ex01 files → Filesystem mode first, falls back to physical
    pub fn open(path: &str) -> Result<Self, VfsError> {
        let lower = path.to_lowercase();
        // L01/Lx01 are logical evidence — use logical mode
        if lower.ends_with(".l01") || lower.ends_with(".lx01") {
            return Self::open_logical(path);
        }
        // E01/Ex01 — try filesystem mode first, fall back to physical
        match Self::open_filesystem(path) {
            Ok(vfs) if !vfs.partitions.is_empty() => Ok(vfs),
            _ => Self::open_physical(path),
        }
    }

    /// Get the number of mounted partitions
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Get the total disk size
    pub fn disk_size(&self) -> Result<u64, VfsError> {
        let handle = self
            .handle
            .read()
            .map_err(|e| VfsError::Internal(e.to_string()))?;
        ewf_media_size_for_vfs(&handle.volume)
    }

    /// Get the size of a specific partition by mount name
    pub fn get_partition_size(&self, mount_name: &str) -> Option<u64> {
        self.partitions
            .iter()
            .find(|p| p.mount_name == mount_name)
            .map(|p| p.entry.size)
    }
}

impl VirtualFileSystem for EwfVfs {
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let normalized = normalize_path(path);

        match self.mode {
            EwfVfsMode::Physical => {
                if normalized == "/" {
                    // Root directory
                    Ok(FileAttr {
                        size: 0,
                        is_directory: true,
                        permissions: 0o555,
                        nlink: 2,
                        inode: 1,
                        ..Default::default()
                    })
                } else if normalized == format!("/{}", self.disk_filename) {
                    // The virtual disk file
                    Ok(FileAttr {
                        size: self.disk_size()?,
                        is_directory: false,
                        permissions: 0o444,
                        nlink: 1,
                        inode: 2,
                        ..Default::default()
                    })
                } else {
                    Err(VfsError::NotFound(normalized))
                }
            }
            EwfVfsMode::Filesystem => {
                if normalized == "/" {
                    // Root directory listing partitions
                    Ok(FileAttr {
                        size: 0,
                        is_directory: true,
                        permissions: 0o555,
                        nlink: 2 + self.partitions.len() as u32,
                        inode: 1,
                        ..Default::default()
                    })
                } else if let Some((partition, remaining_path)) =
                    find_partition(&self.partitions, &normalized)
                {
                    // Delegate to filesystem driver
                    partition.fs.getattr(&remaining_path)
                } else {
                    // Check if it's a partition mount point
                    let name = normalized.trim_start_matches('/');
                    if self.partitions.iter().any(|p| p.mount_name == name) {
                        Ok(FileAttr {
                            size: 0,
                            is_directory: true,
                            permissions: 0o555,
                            nlink: 2,
                            inode: 0,
                            ..Default::default()
                        })
                    } else {
                        Err(VfsError::NotFound(normalized))
                    }
                }
            }
            EwfVfsMode::Logical => {
                // L01 logical evidence: expose data as a single virtual file
                if normalized == "/" {
                    Ok(FileAttr {
                        size: 0,
                        is_directory: true,
                        permissions: 0o555,
                        nlink: 2,
                        inode: 1,
                        ..Default::default()
                    })
                } else if normalized == format!("/{}", self.disk_filename) {
                    Ok(FileAttr {
                        size: self.disk_size()?,
                        is_directory: false,
                        permissions: 0o444,
                        nlink: 1,
                        inode: 2,
                        ..Default::default()
                    })
                } else {
                    Err(VfsError::NotFound(normalized))
                }
            }
        }
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let normalized = normalize_path(path);

        match self.mode {
            EwfVfsMode::Physical => {
                if normalized == "/" {
                    // Root directory contains the virtual disk file
                    Ok(vec![DirEntry {
                        name: self.disk_filename.clone(),
                        is_directory: false,
                        inode: 2,
                        file_type: 8, // Regular file
                    }])
                } else {
                    Err(VfsError::NotADirectory(normalized))
                }
            }
            EwfVfsMode::Filesystem => {
                if normalized == "/" {
                    // List all mounted partitions
                    Ok(self
                        .partitions
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            DirEntry {
                                name: p.mount_name.clone(),
                                is_directory: true,
                                inode: (i + 100) as u64, // Offset to avoid conflicts
                                file_type: 4,            // Directory
                            }
                        })
                        .collect())
                } else if let Some((partition, remaining_path)) =
                    find_partition(&self.partitions, &normalized)
                {
                    // Delegate to filesystem driver
                    partition.fs.readdir(&remaining_path)
                } else {
                    Err(VfsError::NotADirectory(normalized))
                }
            }
            EwfVfsMode::Logical => {
                // L01 logical evidence: root contains a single data file
                if normalized == "/" {
                    Ok(vec![DirEntry {
                        name: self.disk_filename.clone(),
                        is_directory: false,
                        inode: 2,
                        file_type: 8,
                    }])
                } else {
                    Err(VfsError::NotADirectory(normalized))
                }
            }
        }
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path);

        match self.mode {
            EwfVfsMode::Physical => {
                if normalized == format!("/{}", self.disk_filename) {
                    // Read from the virtual disk file
                    let mut handle = self
                        .handle
                        .write()
                        .map_err(|e| VfsError::Internal(e.to_string()))?;

                    let total_size = ewf_media_size_for_vfs(&handle.volume)?;

                    let actual_size = bounded_ewf_read_len(total_size, offset, size)?;
                    if actual_size == 0 {
                        return Ok(Vec::new());
                    }

                    handle
                        .read_at(offset, actual_size)
                        .map_err(|e| VfsError::IoError(e.to_string()))
                } else if normalized == "/" {
                    Err(VfsError::NotAFile(normalized))
                } else {
                    Err(VfsError::NotFound(normalized))
                }
            }
            EwfVfsMode::Filesystem => {
                if normalized == "/" {
                    Err(VfsError::NotAFile(normalized))
                } else if let Some((partition, remaining_path)) =
                    find_partition(&self.partitions, &normalized)
                {
                    // Delegate to filesystem driver
                    partition.fs.read(&remaining_path, offset, size)
                } else {
                    Err(VfsError::NotFound(normalized))
                }
            }
            EwfVfsMode::Logical => {
                // L01 logical evidence: read from the data stream
                if normalized == format!("/{}", self.disk_filename) {
                    let mut handle = self
                        .handle
                        .write()
                        .map_err(|e| VfsError::Internal(e.to_string()))?;

                    let total_size = ewf_media_size_for_vfs(&handle.volume)?;

                    let actual_size = bounded_ewf_read_len(total_size, offset, size)?;
                    if actual_size == 0 {
                        return Ok(Vec::new());
                    }

                    handle
                        .read_at(offset, actual_size)
                        .map_err(|e| VfsError::IoError(e.to_string()))
                } else if normalized == "/" {
                    Err(VfsError::NotAFile(normalized))
                } else {
                    Err(VfsError::NotFound(normalized))
                }
            }
        }
    }
}

// =============================================================================
// Block Device Wrapper for EWF
// =============================================================================

/// Block device wrapper for EWF handle
/// Implements SeekableBlockDevice trait for filesystem drivers
struct EwfBlockDevice {
    handle: Arc<RwLock<EwfHandle>>,
}

impl BlockDevice for EwfBlockDevice {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ffx_errors::ContainerError> {
        let mut handle = self
            .handle
            .write()
            .map_err(|e| ffx_errors::ContainerError::InternalError(e.to_string()))?;

        let data = handle.read_at(offset, buf.len())?;

        let len = data.len().min(buf.len());
        buf[..len].copy_from_slice(&data[..len]);
        Ok(len)
    }

    fn size(&self) -> u64 {
        let handle = self.handle.read().ok();
        match handle {
            Some(h) => checked_volume_media_size(&h.volume).unwrap_or(0),
            None => 0,
        }
    }
}

impl SeekableBlockDevice for EwfBlockDevice {
    fn reader_at(&self, offset: u64) -> Box<dyn BlockReader> {
        Box::new(EwfBlockReader {
            handle: Arc::clone(&self.handle),
            position: offset,
        })
    }
}

/// Block reader for EWF handle
struct EwfBlockReader {
    handle: Arc<RwLock<EwfHandle>>,
    position: u64,
}

impl Read for EwfBlockReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut handle = self
            .handle
            .write()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let data = handle
            .read_at(self.position, buf.len())
            .map_err(std::io::Error::other)?;

        let len = data.len().min(buf.len());
        buf[..len].copy_from_slice(&data[..len]);
        self.position = checked_ewf_read_advance(self.position, len)?;
        Ok(len)
    }
}

impl Seek for EwfBlockReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let size = {
            let handle = self
                .handle
                .read()
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            ewf_media_size_for_io(&handle.volume)?
        };

        let new_pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::Current(p) => checked_ewf_seek_position(self.position, p)?,
            SeekFrom::End(p) => checked_ewf_seek_position(size, p)?,
        };
        self.position = new_pos;
        Ok(new_pos)
    }
}

impl BlockReader for EwfBlockReader {}

fn ewf_media_size_for_vfs(volume: &VolumeSection) -> Result<u64, VfsError> {
    checked_volume_media_size(volume).map_err(|e| VfsError::IoError(e.to_string()))
}

fn ewf_media_size_for_io(volume: &VolumeSection) -> std::io::Result<u64> {
    checked_volume_media_size(volume).map_err(std::io::Error::other)
}

fn checked_ewf_seek_position(base: u64, delta: i64) -> std::io::Result<u64> {
    if delta >= 0 {
        base.checked_add(delta as u64).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "EWF seek overflow")
        })
    } else {
        base.checked_sub(delta.unsigned_abs()).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "EWF seek before start")
        })
    }
}

fn checked_ewf_read_advance(base: u64, len: usize) -> std::io::Result<u64> {
    let len = u64::try_from(len).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "EWF read length exceeds 64-bit position",
        )
    })?;
    base.checked_add(len).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "EWF read position overflow",
        )
    })
}

fn bounded_ewf_read_len(
    total_size: u64,
    offset: u64,
    requested_size: usize,
) -> Result<usize, VfsError> {
    if offset > total_size {
        return Err(VfsError::OutOfBounds {
            offset,
            size: requested_size,
        });
    }

    let remaining = total_size - offset;
    let remaining = usize::try_from(remaining).unwrap_or(usize::MAX);
    Ok(requested_size.min(remaining))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ewf_vfs_mode() {
        assert_eq!(EwfVfsMode::Physical, EwfVfsMode::Physical);
        assert_ne!(EwfVfsMode::Physical, EwfVfsMode::Logical);
    }

    #[test]
    fn test_ewf_vfs_mode_logical() {
        assert_eq!(EwfVfsMode::Logical, EwfVfsMode::Logical);
        assert_ne!(EwfVfsMode::Logical, EwfVfsMode::Filesystem);
        assert_ne!(EwfVfsMode::Logical, EwfVfsMode::Physical);
    }

    #[test]
    fn test_ewf_vfs_mode_all_variants() {
        // Ensure all three modes are distinct
        let modes = [
            EwfVfsMode::Physical,
            EwfVfsMode::Filesystem,
            EwfVfsMode::Logical,
        ];
        for i in 0..modes.len() {
            for j in 0..modes.len() {
                if i == j {
                    assert_eq!(modes[i], modes[j]);
                } else {
                    assert_ne!(modes[i], modes[j]);
                }
            }
        }
    }

    #[test]
    fn test_open_physical_nonexistent() {
        let result = EwfVfs::open_physical("/nonexistent/path/test.E01");
        match result {
            Err(VfsError::NotFound(path)) => assert!(path.contains("nonexistent")),
            Err(other) => panic!("Expected NotFound, got {:?}", other),
            Ok(_) => panic!("Expected error for nonexistent file"),
        }
    }

    #[test]
    fn test_open_logical_nonexistent() {
        let result = EwfVfs::open_logical("/nonexistent/path/test.L01");
        match result {
            Err(VfsError::NotFound(path)) => assert!(path.contains("nonexistent")),
            Err(other) => panic!("Expected NotFound, got {:?}", other),
            Ok(_) => panic!("Expected error for nonexistent file"),
        }
    }

    #[test]
    fn test_open_filesystem_nonexistent() {
        let result = EwfVfs::open_filesystem("/nonexistent/path/test.E01");
        match result {
            Err(VfsError::NotFound(path)) => assert!(path.contains("nonexistent")),
            Err(other) => panic!("Expected NotFound, got {:?}", other),
            Ok(_) => panic!("Expected error for nonexistent file"),
        }
    }

    #[test]
    fn test_open_auto_detect_l01_extension() {
        // L01 extension should attempt logical mode (which will fail on nonexistent file)
        let result = EwfVfs::open("/nonexistent/path/evidence.L01");
        match result {
            Err(VfsError::NotFound(_)) => {} // Expected
            Err(other) => panic!("Expected NotFound for L01, got {:?}", other),
            Ok(_) => panic!("Expected error for nonexistent L01"),
        }
    }

    #[test]
    fn test_open_auto_detect_lx01_extension() {
        let result = EwfVfs::open("/nonexistent/path/evidence.Lx01");
        match result {
            Err(VfsError::NotFound(_)) => {} // Expected - open_logical checks existence
            Err(other) => panic!("Expected NotFound for Lx01, got {:?}", other),
            Ok(_) => panic!("Expected error for nonexistent Lx01"),
        }
    }

    #[test]
    fn test_open_auto_detect_e01_extension() {
        // E01 should try filesystem mode first, then fall back to physical
        let result = EwfVfs::open("/nonexistent/path/evidence.E01");
        assert!(result.is_err());
    }

    #[test]
    fn test_l01_with_invalid_file() {
        // Create a temp file with invalid (non-LVF) content
        let dir = tempfile::TempDir::new().unwrap();
        let l01_path = dir.path().join("test.L01");
        std::fs::write(&l01_path, b"NOT_A_VALID_LVF_FILE_HEADER_DATA").unwrap();

        let result = EwfVfs::open_logical(l01_path.to_str().unwrap());
        // Should fail because the EwfHandle::open will reject the invalid signature
        assert!(result.is_err());
    }

    #[test]
    fn test_vfs_mode_copy_clone() {
        let mode = EwfVfsMode::Logical;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_checked_ewf_seek_position_rejects_underflow() {
        assert!(checked_ewf_seek_position(0, -1).is_err());
        assert!(checked_ewf_seek_position(10, -11).is_err());
    }

    #[test]
    fn test_checked_ewf_seek_position_rejects_overflow() {
        assert!(checked_ewf_seek_position(u64::MAX, 1).is_err());
    }

    #[test]
    fn test_checked_ewf_seek_position_allows_valid_offsets() {
        assert_eq!(checked_ewf_seek_position(10, -5).unwrap(), 5);
        assert_eq!(checked_ewf_seek_position(10, 5).unwrap(), 15);
    }

    #[test]
    fn test_checked_ewf_read_advance_rejects_overflow() {
        assert!(checked_ewf_read_advance(u64::MAX, 1).is_err());
    }

    #[test]
    fn test_checked_ewf_read_advance_allows_valid_offsets() {
        assert_eq!(checked_ewf_read_advance(10, 5).unwrap(), 15);
    }

    #[test]
    fn test_bounded_ewf_read_len_allows_exact_eof() {
        assert_eq!(bounded_ewf_read_len(10, 10, 4).unwrap(), 0);
    }

    #[test]
    fn test_bounded_ewf_read_len_rejects_offset_past_eof() {
        let err = bounded_ewf_read_len(10, 11, 4).unwrap_err();
        assert!(matches!(
            err,
            VfsError::OutOfBounds {
                offset: 11,
                size: 4
            }
        ));
    }

    #[test]
    fn test_bounded_ewf_read_len_clamps_to_remaining() {
        assert_eq!(bounded_ewf_read_len(10, 8, 5).unwrap(), 2);
        assert_eq!(bounded_ewf_read_len(u64::MAX, 0, 8).unwrap(), 8);
    }
}
