// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Filesystem Parsing Module
//!
//! ## Section Brief
//! Provides filesystem detection and parsing for disk images.
//! Enables file tree access for EWF (E01) and Raw disk images.
//!
//! ### Submodules
//! - `traits` - Filesystem driver trait definitions
//! - `partition` - MBR/GPT partition table parsing
//! - `fat` - FAT12/16/32 filesystem driver
//! - `ntfs` - NTFS filesystem driver
//!
//! ### Architecture
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Disk Image (EWF/Raw)                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Partition Table (MBR or GPT)                               │
//! │  ┌─────────────┬─────────────┬─────────────┐                │
//! │  │ Partition 1 │ Partition 2 │ Partition 3 │                │
//! │  │   (NTFS)    │   (FAT32)   │   (ext4)    │                │
//! │  └─────────────┴─────────────┴─────────────┘                │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ### Usage
//! ```rust,ignore
//! use crate::common::filesystem::{detect_partitions, mount_filesystem};
//!
//! // Detect partitions
//! let partitions = detect_partitions(&disk_reader)?;
//!
//! // Mount a partition's filesystem
//! let fs = mount_filesystem(&disk_reader, &partitions[0])?;
//!
//! // Access files
//! let entries = fs.readdir("/")?;
//! ```

pub mod traits;
pub mod partition;
pub mod fat;
pub mod ntfs_driver;
// pub mod ext;
// NOTE: ext2/3/4 support pending - requires ext4 crate stabilization
// See: https://crates.io/crates/ext4 - implement ExtDriver when available

// Re-exports
pub use traits::{
    FilesystemDriver, FilesystemType, FilesystemInfo,
    BlockDevice, SeekableBlockDevice, BlockReader,
};
pub use partition::{
    PartitionTable, PartitionTableType, PartitionEntry,
    detect_partition_table, get_partition_filesystem_type,
};
pub use fat::FatDriver;
pub use ntfs_driver::NtfsDriver;

use crate::common::vfs::VfsError;

/// Detect and mount filesystem from a block device at given offset
pub fn mount_filesystem(
    device: Box<dyn SeekableBlockDevice>,
    offset: u64,
    size: u64,
) -> Result<Box<dyn FilesystemDriver>, VfsError> {
    // First, detect the filesystem type
    let fs_type = detect_filesystem_type(device.as_ref(), offset)?;
    
    match fs_type {
        FilesystemType::Fat12 | FilesystemType::Fat16 | FilesystemType::Fat32 => {
            let driver = FatDriver::new(device, offset, size)?;
            Ok(Box::new(driver))
        }
        FilesystemType::Ntfs => {
            let driver = NtfsDriver::new(device, offset, size)?;
            Ok(Box::new(driver))
        }
        FilesystemType::Ext2 | FilesystemType::Ext3 | FilesystemType::Ext4 => {
            // ext2/3/4 support requires the ext4 crate which is currently unavailable
            // See: https://crates.io/crates/ext4 (awaiting stable release)
            // When available, implement ExtDriver similar to NtfsDriver
            Err(VfsError::Internal(
                "ext2/3/4 filesystem detected but not yet supported. \
                 See GitHub issue for tracking: support will be added when ext4 crate stabilizes."
                    .to_string()
            ))
        }
        FilesystemType::Unknown => {
            Err(VfsError::Internal("Unknown or unsupported filesystem".to_string()))
        }
        _ => {
            Err(VfsError::Internal(format!("Filesystem {:?} not yet supported", fs_type)))
        }
    }
}

/// Detect filesystem type by reading boot sector
pub fn detect_filesystem_type(device: &dyn SeekableBlockDevice, offset: u64) -> Result<FilesystemType, VfsError> {
    // Read first 512 bytes (boot sector)
    let mut buf = vec![0u8; 512];
    let bytes_read = device.read_at(offset, &mut buf)
        .map_err(|e| VfsError::IoError(e.to_string()))?;
    
    if bytes_read < 512 {
        return Ok(FilesystemType::Unknown);
    }
    
    // Check for NTFS signature "NTFS    " at offset 3
    if &buf[3..11] == b"NTFS    " {
        return Ok(FilesystemType::Ntfs);
    }
    
    // Check for FAT signatures
    // FAT12/16: Look for "FAT12   " or "FAT16   " at offset 54
    // FAT32: Look for "FAT32   " at offset 82
    if &buf[54..62] == b"FAT12   " {
        return Ok(FilesystemType::Fat12);
    }
    if &buf[54..62] == b"FAT16   " {
        return Ok(FilesystemType::Fat16);
    }
    if &buf[82..90] == b"FAT32   " {
        return Ok(FilesystemType::Fat32);
    }
    
    // Check for ext2/3/4 magic at offset 0x438 (1080)
    // Need to read more data for this
    let mut ext_buf = vec![0u8; 2];
    if device.read_at(offset + 0x438, &mut ext_buf).is_ok() && ext_buf == [0x53, 0xEF] {
        return Ok(FilesystemType::Ext4);
    }
    
    // Check for exFAT signature "EXFAT   " at offset 3
    if &buf[3..11] == b"EXFAT   " {
        return Ok(FilesystemType::ExFat);
    }
    
    Ok(FilesystemType::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock block device for testing
    struct MockBlockDevice {
        data: Vec<u8>,
    }

    impl MockBlockDevice {
        fn new(size: usize) -> Self {
            Self { data: vec![0u8; size] }
        }

        fn with_data(data: Vec<u8>) -> Self {
            Self { data }
        }
    }

    impl BlockDevice for MockBlockDevice {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, crate::containers::ContainerError> {
            let offset = offset as usize;
            if offset >= self.data.len() {
                return Ok(0);
            }
            let end = std::cmp::min(offset + buf.len(), self.data.len());
            let len = end - offset;
            buf[..len].copy_from_slice(&self.data[offset..end]);
            Ok(len)
        }

        fn size(&self) -> u64 {
            self.data.len() as u64
        }
    }

    impl SeekableBlockDevice for MockBlockDevice {
        fn reader_at(&self, _offset: u64) -> Box<dyn BlockReader> {
            unimplemented!("Not needed for these tests")
        }
    }

    // ==================== detect_filesystem_type tests ====================

    #[test]
    fn test_detect_filesystem_type_unknown_empty() {
        let device = MockBlockDevice::new(1024);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Unknown);
    }

    #[test]
    fn test_detect_filesystem_type_too_small() {
        let device = MockBlockDevice::new(100); // Less than 512 bytes
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Unknown);
    }

    #[test]
    fn test_detect_filesystem_type_ntfs() {
        let mut data = vec![0u8; 1024];
        // NTFS signature at offset 3
        data[3..11].copy_from_slice(b"NTFS    ");
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Ntfs);
    }

    #[test]
    fn test_detect_filesystem_type_fat12() {
        let mut data = vec![0u8; 1024];
        // FAT12 signature at offset 54
        data[54..62].copy_from_slice(b"FAT12   ");
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Fat12);
    }

    #[test]
    fn test_detect_filesystem_type_fat16() {
        let mut data = vec![0u8; 1024];
        // FAT16 signature at offset 54
        data[54..62].copy_from_slice(b"FAT16   ");
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Fat16);
    }

    #[test]
    fn test_detect_filesystem_type_fat32() {
        let mut data = vec![0u8; 1024];
        // FAT32 signature at offset 82
        data[82..90].copy_from_slice(b"FAT32   ");
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Fat32);
    }

    #[test]
    fn test_detect_filesystem_type_exfat() {
        let mut data = vec![0u8; 1024];
        // exFAT signature at offset 3
        data[3..11].copy_from_slice(b"EXFAT   ");
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::ExFat);
    }

    #[test]
    fn test_detect_filesystem_type_ext() {
        // ext filesystems have magic at offset 0x438 (1080)
        let mut data = vec![0u8; 2048];
        data[0x438] = 0x53;
        data[0x439] = 0xEF;
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Ext4);
    }

    #[test]
    fn test_detect_filesystem_type_with_offset() {
        let mut data = vec![0u8; 2048];
        // Put NTFS signature at offset 512 + 3
        data[515..523].copy_from_slice(b"NTFS    ");
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 512);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Ntfs);
    }

    // ==================== mount_filesystem tests ====================

    #[test]
    fn test_mount_filesystem_unknown_returns_error() {
        let device: Box<dyn SeekableBlockDevice> = Box::new(MockBlockDevice::new(1024));
        let result = mount_filesystem(device, 0, 1024);
        assert!(result.is_err());
    }

    #[test]
    fn test_mount_filesystem_ext_not_supported() {
        let mut data = vec![0u8; 2048];
        // ext magic at offset 0x438
        data[0x438] = 0x53;
        data[0x439] = 0xEF;
        
        let device: Box<dyn SeekableBlockDevice> = Box::new(MockBlockDevice::with_data(data));
        let result = mount_filesystem(device, 0, 2048);
        assert!(result.is_err());
        
        if let Err(VfsError::Internal(msg)) = result {
            assert!(msg.contains("ext"));
        } else {
            panic!("Expected Internal error with ext message");
        }
    }
}
