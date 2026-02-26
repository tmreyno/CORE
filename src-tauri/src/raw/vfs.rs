// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Raw Image Virtual Filesystem Implementation
//!
//! Read-only virtual filesystem for raw disk images.
//! Supports two modes:
//! - Physical: Exposes the raw image as a single virtual file
//! - Filesystem: Auto-detects partitions and mounts filesystems

use std::sync::{Arc, RwLock};
use crate::common::vfs::{
    VirtualFileSystem, VfsError, FileAttr, DirEntry, normalize_path,
    MountedPartition, find_partition,
};
use crate::common::filesystem::{
    SeekableBlockDevice, BlockReader, BlockDevice,
    detect_partition_table, mount_filesystem,
};
use crate::containers::ContainerError;
use super::RawHandle;

/// Operating mode for the Raw VFS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawVfsMode {
    /// Physical mode: expose disk image as a single file
    Physical,
    /// Filesystem mode: auto-mount filesystems from partitions
    Filesystem,
}

/// Block device adapter for RawHandle to work with filesystem drivers
struct RawBlockDevice {
    handle: Arc<RwLock<RawHandle>>,
}

impl RawBlockDevice {
    fn new(handle: RawHandle) -> Self {
        Self {
            handle: Arc::new(RwLock::new(handle)),
        }
    }
}

impl crate::common::filesystem::BlockDevice for RawBlockDevice {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError> {
        let mut handle = self.handle.write()
            .map_err(|e| ContainerError::InternalError(format!("Lock error: {}", e)))?;
        handle.position = offset;
        handle.read(buf)
    }

    fn size(&self) -> u64 {
        self.handle.read()
            .map(|h| h.total_size())
            .unwrap_or(0)
    }
}

impl SeekableBlockDevice for RawBlockDevice {
    fn reader_at(&self, offset: u64) -> Box<dyn BlockReader> {
        Box::new(RawBlockReader {
            handle: Arc::clone(&self.handle),
            position: offset,
        })
    }
}

/// Block reader for specific offset in raw image
struct RawBlockReader {
    handle: Arc<RwLock<RawHandle>>,
    position: u64,
}

impl std::io::Read for RawBlockReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut handle = self.handle.write()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        handle.position = self.position;
        let bytes = handle.read(buf)
            .map_err(std::io::Error::other)?;
        self.position += bytes as u64;
        Ok(bytes)
    }
}

impl std::io::Seek for RawBlockReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let size = self.handle.read()
            .map(|h| h.total_size())
            .unwrap_or(0);
        
        let new_offset = match pos {
            std::io::SeekFrom::Start(o) => o,
            std::io::SeekFrom::End(o) => {
                if o >= 0 {
                    size.saturating_add(o as u64)
                } else {
                    size.saturating_sub((-o) as u64)
                }
            }
            std::io::SeekFrom::Current(o) => {
                if o >= 0 {
                    self.position.saturating_add(o as u64)
                } else {
                    self.position.saturating_sub((-o) as u64)
                }
            }
        };
        self.position = new_offset.min(size);
        Ok(self.position)
    }
}

impl BlockReader for RawBlockReader {}

/// Virtual filesystem implementation for raw disk images
pub struct RawVfs {
    /// Operating mode
    mode: RawVfsMode,
    /// Image path
    #[allow(dead_code)]
    path: String,
    /// Block device for filesystem access
    device: Option<Arc<RawBlockDevice>>,
    /// Raw handle for Physical mode
    handle: Option<RwLock<RawHandle>>,
    /// Virtual file name (Physical mode)
    filename: String,
    /// Mounted partitions (Filesystem mode)
    partitions: Vec<MountedPartition>,
    /// Partition table info for display
    #[allow(dead_code)]
    partition_table: Option<crate::common::filesystem::PartitionTable>,
}

impl RawVfs {
    /// Open a raw image in Physical mode (single file view)
    pub fn open(path: &str) -> Result<Self, VfsError> {
        if !std::path::Path::new(path).exists() {
            return Err(VfsError::NotFound(path.to_string()));
        }
        
        let handle = RawHandle::open(path)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        // Generate filename from path
        let filename = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("disk");
        
        Ok(Self {
            mode: RawVfsMode::Physical,
            path: path.to_string(),
            device: None,
            handle: Some(RwLock::new(handle)),
            filename: format!("{}.raw", filename),
            partitions: Vec::new(),
            partition_table: None,
        })
    }

    /// Open a raw image in Filesystem mode (auto-mount partitions)
    pub fn open_filesystem(path: &str) -> Result<Self, VfsError> {
        if !std::path::Path::new(path).exists() {
            return Err(VfsError::NotFound(path.to_string()));
        }
        
        let handle = RawHandle::open(path)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        // Create block device adapter
        let device = Arc::new(RawBlockDevice::new(handle));
        
        // Detect partition table
        let partition_table = detect_partition_table(device.as_ref())
            .map_err(|e| VfsError::Internal(format!("Partition detection failed: {}", e)))?;
        
        tracing::info!(
            table_type = ?partition_table.table_type,
            partition_count = partition_table.partitions.len(),
            "Raw image partition table detected"
        );
        
        // Try to mount filesystems on each partition
        let mut partitions = Vec::new();
        for (idx, entry) in partition_table.partitions.iter().enumerate() {
            // Clone device for each partition (Arc handles shared access)
            let fs_device: Box<dyn SeekableBlockDevice> = Box::new(RawBlockDeviceWrapper {
                inner: Arc::clone(&device),
            });
            
            match mount_filesystem(fs_device, entry.start_offset, entry.size) {
                Ok(fs) => {
                    let fs_info = fs.info();
                    let mount_name = format!(
                        "Partition_{}_{:?}",
                        idx + 1,
                        fs_info.fs_type
                    );
                    tracing::info!(
                        partition = idx + 1,
                        fs_type = ?fs_info.fs_type,
                        label = ?fs_info.label,
                        "Mounted filesystem"
                    );
                    partitions.push(MountedPartition {
                        entry: entry.clone(),
                        fs,
                        mount_name,
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        partition = idx + 1,
                        error = %e,
                        "Failed to mount partition filesystem"
                    );
                }
            }
        }
        
        let filename = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("disk");
        
        Ok(Self {
            mode: RawVfsMode::Filesystem,
            path: path.to_string(),
            device: Some(device),
            handle: None,
            filename: format!("{}.raw", filename),
            partitions,
            partition_table: Some(partition_table),
        })
    }

    /// Get the current mode
    pub fn mode(&self) -> RawVfsMode {
        self.mode
    }

    /// Get number of mounted partitions
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Get the total disk size
    fn disk_size(&self) -> Result<u64, VfsError> {
        if let Some(ref device) = self.device {
            Ok(device.size())
        } else if let Some(ref handle) = self.handle {
            let h = handle.read()
                .map_err(|e| VfsError::Internal(e.to_string()))?;
            Ok(h.total_size())
        } else {
            Err(VfsError::Internal("No device or handle available".to_string()))
        }
    }
}

/// Wrapper to make Arc<RawBlockDevice> implement SeekableBlockDevice
struct RawBlockDeviceWrapper {
    inner: Arc<RawBlockDevice>,
}

impl crate::common::filesystem::BlockDevice for RawBlockDeviceWrapper {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError> {
        self.inner.read_at(offset, buf)
    }

    fn size(&self) -> u64 {
        self.inner.size()
    }
}

impl SeekableBlockDevice for RawBlockDeviceWrapper {
    fn reader_at(&self, offset: u64) -> Box<dyn BlockReader> {
        self.inner.reader_at(offset)
    }
}

impl VirtualFileSystem for RawVfs {
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let normalized = normalize_path(path);
        
        match self.mode {
            RawVfsMode::Physical => {
                // Physical mode: single file view
                if normalized == "/" {
                    Ok(FileAttr {
                        size: 0,
                        is_directory: true,
                        permissions: 0o555,
                        nlink: 2,
                        inode: 1,
                        ..Default::default()
                    })
                } else if normalized == format!("/{}", self.filename) {
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
            RawVfsMode::Filesystem => {
                // Filesystem mode: mounted partitions
                if normalized == "/" {
                    Ok(FileAttr {
                        size: 0,
                        is_directory: true,
                        permissions: 0o555,
                        nlink: 2 + self.partitions.len() as u32,
                        inode: 1,
                        ..Default::default()
                    })
                } else if let Some((partition, sub_path)) = find_partition(&self.partitions, &normalized) {
                    partition.fs.getattr(&sub_path)
                } else {
                    // Check if it's a partition mount point
                    for (idx, partition) in self.partitions.iter().enumerate() {
                        if normalized == format!("/{}", partition.mount_name) {
                            return Ok(FileAttr {
                                size: 0,
                                is_directory: true,
                                permissions: 0o555,
                                nlink: 2,
                                inode: 100 + idx as u64,
                                ..Default::default()
                            });
                        }
                    }
                    Err(VfsError::NotFound(normalized))
                }
            }
        }
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let normalized = normalize_path(path);
        
        match self.mode {
            RawVfsMode::Physical => {
                if normalized == "/" {
                    Ok(vec![DirEntry {
                        name: self.filename.clone(),
                        is_directory: false,
                        inode: 2,
                        file_type: 8,
                    }])
                } else {
                    Err(VfsError::NotADirectory(normalized))
                }
            }
            RawVfsMode::Filesystem => {
                if normalized == "/" {
                    // List mounted partitions
                    let entries: Vec<DirEntry> = self.partitions.iter()
                        .enumerate()
                        .map(|(idx, p)| DirEntry {
                            name: p.mount_name.clone(),
                            is_directory: true,
                            inode: 100 + idx as u64,
                            file_type: 4, // Directory
                        })
                        .collect();
                    Ok(entries)
                } else if let Some((partition, sub_path)) = find_partition(&self.partitions, &normalized) {
                    partition.fs.readdir(&sub_path)
                } else {
                    Err(VfsError::NotADirectory(normalized))
                }
            }
        }
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path);
        
        match self.mode {
            RawVfsMode::Physical => {
                if normalized == format!("/{}", self.filename) {
                    let handle = self.handle.as_ref()
                        .ok_or_else(|| VfsError::Internal("No handle in Physical mode".to_string()))?;
                    
                    let mut h = handle.write()
                        .map_err(|e| VfsError::Internal(e.to_string()))?;
                    
                    let total_size = h.total_size();
                    
                    if offset >= total_size {
                        return Ok(Vec::new());
                    }
                    
                    h.position = offset;
                    
                    let actual_size = size.min((total_size - offset) as usize);
                    let mut buf = vec![0u8; actual_size];
                    
                    let bytes_read = h.read(&mut buf)
                        .map_err(|e| VfsError::IoError(e.to_string()))?;
                    
                    buf.truncate(bytes_read);
                    Ok(buf)
                } else if normalized == "/" {
                    Err(VfsError::NotAFile(normalized))
                } else {
                    Err(VfsError::NotFound(normalized))
                }
            }
            RawVfsMode::Filesystem => {
                if let Some((partition, sub_path)) = find_partition(&self.partitions, &normalized) {
                    partition.fs.read(&sub_path, offset, size)
                } else {
                    Err(VfsError::NotFound(normalized))
                }
            }
        }
    }
}
