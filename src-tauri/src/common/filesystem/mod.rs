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
pub mod hfsplus_driver;
pub mod apfs_driver;
pub mod ext_driver;
pub mod dmg_driver;
pub mod exfat_driver;

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
pub use hfsplus_driver::HfsPlusDriver;
pub use apfs_driver::ApfsDriver;
pub use ext_driver::ExtDriver;
pub use dmg_driver::{DmgDriver, DmgPartitionInfo, MemoryBlockDevice};
pub use exfat_driver::ExFatDriver;

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
        FilesystemType::HfsPlus => {
            let driver = HfsPlusDriver::new(device, offset, size)?;
            Ok(Box::new(driver))
        }
        FilesystemType::Apfs => {
            let driver = ApfsDriver::new(device, offset, size)?;
            Ok(Box::new(driver))
        }
        FilesystemType::Ext2 | FilesystemType::Ext3 | FilesystemType::Ext4 => {
            let driver = ExtDriver::new(device, offset, size)?;
            Ok(Box::new(driver))
        }
        FilesystemType::ExFat => {
            let driver = ExFatDriver::new(device, offset, size)?;
            Ok(Box::new(driver))
        }
        FilesystemType::Unknown => {
            Err(VfsError::Internal(
                "Unknown or unsupported filesystem. Could not detect a recognized filesystem signature. \
                 Supported filesystems: NTFS, FAT12/FAT16/FAT32, exFAT, HFS+, APFS, ext2/ext3/ext4. \
                 The partition may be encrypted, damaged, or use an unsupported filesystem (e.g., XFS, Btrfs, ReFS, ZFS)."
                    .to_string(),
            ))
        }
    }
}

/// Detect filesystem type by reading boot sector
pub fn detect_filesystem_type(device: &dyn SeekableBlockDevice, offset: u64) -> Result<FilesystemType, VfsError> {
    tracing::debug!("detect_filesystem_type: checking offset={}", offset);
    
    // Read first 512 bytes (boot sector)
    let mut buf = vec![0u8; 512];
    let bytes_read = device.read_at(offset, &mut buf)
        .map_err(|e| VfsError::IoError(e.to_string()))?;
    
    if bytes_read < 512 {
        tracing::debug!("detect_filesystem_type: only read {} bytes, need 512", bytes_read);
        return Ok(FilesystemType::Unknown);
    }
    
    // Log first 16 bytes for debugging
    tracing::debug!("detect_filesystem_type: first 16 bytes: {:02x?}", &buf[0..16]);
    
    // Check for NTFS signature "NTFS    " at offset 3
    if &buf[3..11] == b"NTFS    " {
        tracing::debug!("detect_filesystem_type: detected NTFS");
        return Ok(FilesystemType::Ntfs);
    }
    
    // Check for FAT signatures
    // FAT12/16: Look for "FAT12   " or "FAT16   " at offset 54
    // FAT32: Look for "FAT32   " at offset 82
    if &buf[54..62] == b"FAT12   " {
        tracing::debug!("detect_filesystem_type: detected FAT12");
        return Ok(FilesystemType::Fat12);
    }
    if &buf[54..62] == b"FAT16   " {
        tracing::debug!("detect_filesystem_type: detected FAT16");
        return Ok(FilesystemType::Fat16);
    }
    if &buf[82..90] == b"FAT32   " {
        tracing::debug!("detect_filesystem_type: detected FAT32");
        return Ok(FilesystemType::Fat32);
    }
    
    // Check for exFAT signature "EXFAT   " at offset 3
    if &buf[3..11] == b"EXFAT   " {
        tracing::debug!("detect_filesystem_type: detected exFAT");
        return Ok(FilesystemType::ExFat);
    }
    
    // Check for APFS container superblock at block 0
    // APFS magic 'NXSB' at offset 32 (after 32-byte object header)
    let apfs_magic = u32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]);
    tracing::debug!("detect_filesystem_type: APFS magic check: 0x{:08X} (want 0x4E585342)", apfs_magic);
    if apfs_magic == 0x4E585342 {
        tracing::debug!("detect_filesystem_type: detected APFS");
        return Ok(FilesystemType::Apfs);
    }
    
    // Check for HFS+/HFSX at offset 1024 (volume header location)
    let mut hfs_buf = vec![0u8; 512];
    if device.read_at(offset + 1024, &mut hfs_buf).is_ok() {
        // HFS+ signature 'H+' (0x482B) or HFSX 'HX' (0x4858) at offset 0
        let hfs_sig = u16::from_be_bytes([hfs_buf[0], hfs_buf[1]]);
        tracing::debug!("detect_filesystem_type: HFS+ signature check at offset {}: 0x{:04X} (want 0x482B or 0x4858)", offset + 1024, hfs_sig);
        if hfs_sig == 0x482B || hfs_sig == 0x4858 {
            tracing::debug!("detect_filesystem_type: detected HFS+");
            return Ok(FilesystemType::HfsPlus);
        }
    }
    
    // Check for ext2/3/4 magic at offset 0x438 (1080) from superblock
    // Superblock is at offset 1024 from partition start
    // Magic is at offset 56 (0x38) within superblock = 1024 + 56 = 1080
    let mut ext_buf = vec![0u8; 128];
    if device.read_at(offset + 1024, &mut ext_buf).is_ok() {
        let magic = u16::from_le_bytes([ext_buf[56], ext_buf[57]]);
        if magic == 0xEF53 {
            // Read feature flags to determine ext2/3/4
            let feature_compat = u32::from_le_bytes([ext_buf[92], ext_buf[93], ext_buf[94], ext_buf[95]]);
            let feature_incompat = u32::from_le_bytes([ext_buf[96], ext_buf[97], ext_buf[98], ext_buf[99]]);
            
            // ext4 has extents or flex_bg features
            if (feature_incompat & 0x0040) != 0 || (feature_incompat & 0x0200) != 0 {
                tracing::debug!("detect_filesystem_type: detected ext4");
                return Ok(FilesystemType::Ext4);
            }
            // ext3 has journal feature
            if (feature_compat & 0x0004) != 0 {
                tracing::debug!("detect_filesystem_type: detected ext3");
                return Ok(FilesystemType::Ext3);
            }
            // Otherwise it's ext2
            tracing::debug!("detect_filesystem_type: detected ext2");
            return Ok(FilesystemType::Ext2);
        }
    }
    
    tracing::debug!("detect_filesystem_type: unknown filesystem");
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
        // ext filesystems have magic at offset 1024 + 56 = 1080 (0x438)
        // Magic is at offset 56 within the superblock (which starts at 1024)
        let mut data = vec![0u8; 2048];
        // Set ext magic (0xEF53) at superblock offset 56 = data offset 1024+56=1080
        data[1024 + 56] = 0x53; // Magic low byte
        data[1024 + 57] = 0xEF; // Magic high byte
        // Set ext4 feature flag (extents = 0x0040) at offset 96 (feature_incompat)
        data[1024 + 96] = 0x40; // extents feature flag for ext4
        
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
    fn test_mount_filesystem_ext_invalid_superblock() {
        // Test that mounting ext with invalid superblock data fails gracefully
        let mut data = vec![0u8; 4096];
        // Set ext magic at correct position (superblock at 1024, magic at offset 56)
        data[1024 + 56] = 0x53;
        data[1024 + 57] = 0xEF;
        // Leave all other superblock fields as zeros (invalid)
        
        let device: Box<dyn SeekableBlockDevice> = Box::new(MockBlockDevice::with_data(data));
        let result = mount_filesystem(device, 0, 4096);
        // Should fail due to invalid superblock (zeros for block_size, blocks_per_group, etc)
        assert!(result.is_err());
    }

    // ==================== HFS+ detection tests ====================

    #[test]
    fn test_detect_filesystem_type_hfsplus() {
        // HFS+ volume header is at offset 1024
        let mut data = vec![0u8; 2048];
        // HFS+ signature 'H+' (0x482B) at offset 1024 (big-endian)
        data[1024] = 0x48; // 'H'
        data[1025] = 0x2B; // '+'
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::HfsPlus);
    }

    #[test]
    fn test_detect_filesystem_type_hfsx() {
        // HFSX (case-sensitive HFS+) volume header is at offset 1024
        let mut data = vec![0u8; 2048];
        // HFSX signature 'HX' (0x4858) at offset 1024 (big-endian)
        data[1024] = 0x48; // 'H'
        data[1025] = 0x58; // 'X'
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::HfsPlus);
    }

    // ==================== APFS detection tests ====================

    #[test]
    fn test_detect_filesystem_type_apfs() {
        let mut data = vec![0u8; 1024];
        // APFS container magic 'NXSB' at offset 32 (little-endian: 0x4E585342)
        data[32] = 0x42; // 'B'
        data[33] = 0x53; // 'S'
        data[34] = 0x58; // 'X'
        data[35] = 0x4E; // 'N'
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::Apfs);
    }

    #[test]
    fn test_detect_filesystem_type_hfsplus_with_offset() {
        // HFS+ at partition offset 512
        let mut data = vec![0u8; 2560]; // 512 + 2048
        // HFS+ signature at offset 512 + 1024 = 1536
        data[1536] = 0x48; // 'H'
        data[1537] = 0x2B; // '+'
        
        let device = MockBlockDevice::with_data(data);
        let result = detect_filesystem_type(&device, 512);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilesystemType::HfsPlus);
    }
}
