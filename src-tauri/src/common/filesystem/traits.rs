// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Filesystem Driver Traits
//!
//! Core trait definitions for filesystem drivers.

use std::io::{Read, Seek};
use crate::common::vfs::{VfsError, FileAttr, DirEntry};
use crate::containers::ContainerError;

// =============================================================================
// Block Device Traits
// =============================================================================

/// A block device that can be read from
pub trait BlockDevice: Send + Sync {
    /// Read data at the given offset
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError>;
    
    /// Get the total size of the device
    fn size(&self) -> u64;
}

/// A seekable block device for filesystem drivers
pub trait SeekableBlockDevice: BlockDevice {
    /// Create a reader at a specific offset
    fn reader_at(&self, offset: u64) -> Box<dyn BlockReader>;
}

/// A reader for a block device section
pub trait BlockReader: Read + Seek + Send {}

// =============================================================================
// Filesystem Types
// =============================================================================

/// Detected filesystem type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemType {
    /// FAT12 filesystem
    Fat12,
    /// FAT16 filesystem
    Fat16,
    /// FAT32 filesystem
    Fat32,
    /// exFAT filesystem
    ExFat,
    /// NTFS filesystem
    Ntfs,
    /// ext2 filesystem
    Ext2,
    /// ext3 filesystem
    Ext3,
    /// ext4 filesystem
    Ext4,
    /// HFS+ filesystem (macOS)
    HfsPlus,
    /// APFS filesystem (macOS)
    Apfs,
    /// Unknown filesystem
    Unknown,
}

impl std::fmt::Display for FilesystemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fat12 => write!(f, "FAT12"),
            Self::Fat16 => write!(f, "FAT16"),
            Self::Fat32 => write!(f, "FAT32"),
            Self::ExFat => write!(f, "exFAT"),
            Self::Ntfs => write!(f, "NTFS"),
            Self::Ext2 => write!(f, "ext2"),
            Self::Ext3 => write!(f, "ext3"),
            Self::Ext4 => write!(f, "ext4"),
            Self::HfsPlus => write!(f, "HFS+"),
            Self::Apfs => write!(f, "APFS"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Filesystem information
#[derive(Debug, Clone)]
pub struct FilesystemInfo {
    /// Filesystem type
    pub fs_type: FilesystemType,
    /// Volume label (if available)
    pub label: Option<String>,
    /// Total size in bytes
    pub total_size: u64,
    /// Free space in bytes (if available)
    pub free_space: Option<u64>,
    /// Cluster/block size
    pub cluster_size: u32,
}

// =============================================================================
// Filesystem Driver Trait
// =============================================================================

/// Filesystem driver trait for accessing files on a mounted filesystem
pub trait FilesystemDriver: Send + Sync {
    /// Get filesystem information
    fn info(&self) -> &FilesystemInfo;
    
    /// Get file/directory attributes
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError>;
    
    /// List directory contents
    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError>;
    
    /// Read file data
    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError>;
    
    /// Check if a path exists
    fn exists(&self, path: &str) -> bool {
        self.getattr(path).is_ok()
    }
    
    /// Check if path is a directory
    fn is_dir(&self, path: &str) -> bool {
        self.getattr(path).map(|a| a.is_directory).unwrap_or(false)
    }
    
    /// Check if path is a file
    fn is_file(&self, path: &str) -> bool {
        self.getattr(path).map(|a| !a.is_directory).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== FilesystemType tests ====================

    #[test]
    fn test_filesystem_type_display_fat() {
        assert_eq!(format!("{}", FilesystemType::Fat12), "FAT12");
        assert_eq!(format!("{}", FilesystemType::Fat16), "FAT16");
        assert_eq!(format!("{}", FilesystemType::Fat32), "FAT32");
    }

    #[test]
    fn test_filesystem_type_display_exfat() {
        assert_eq!(format!("{}", FilesystemType::ExFat), "exFAT");
    }

    #[test]
    fn test_filesystem_type_display_ntfs() {
        assert_eq!(format!("{}", FilesystemType::Ntfs), "NTFS");
    }

    #[test]
    fn test_filesystem_type_display_ext() {
        assert_eq!(format!("{}", FilesystemType::Ext2), "ext2");
        assert_eq!(format!("{}", FilesystemType::Ext3), "ext3");
        assert_eq!(format!("{}", FilesystemType::Ext4), "ext4");
    }

    #[test]
    fn test_filesystem_type_display_macos() {
        assert_eq!(format!("{}", FilesystemType::HfsPlus), "HFS+");
        assert_eq!(format!("{}", FilesystemType::Apfs), "APFS");
    }

    #[test]
    fn test_filesystem_type_display_unknown() {
        assert_eq!(format!("{}", FilesystemType::Unknown), "Unknown");
    }

    #[test]
    fn test_filesystem_type_equality() {
        assert_eq!(FilesystemType::Fat32, FilesystemType::Fat32);
        assert_ne!(FilesystemType::Fat32, FilesystemType::Ntfs);
    }

    #[test]
    fn test_filesystem_type_copy() {
        let fs = FilesystemType::Ntfs;
        let fs_copy = fs;
        assert_eq!(fs, fs_copy);
    }

    // ==================== FilesystemInfo tests ====================

    #[test]
    fn test_filesystem_info_construction() {
        let info = FilesystemInfo {
            fs_type: FilesystemType::Fat32,
            label: Some("MYVOLUME".to_string()),
            total_size: 1024 * 1024 * 1024, // 1 GB
            free_space: Some(512 * 1024 * 1024), // 512 MB
            cluster_size: 4096,
        };
        
        assert_eq!(info.fs_type, FilesystemType::Fat32);
        assert_eq!(info.label, Some("MYVOLUME".to_string()));
        assert_eq!(info.total_size, 1024 * 1024 * 1024);
        assert_eq!(info.free_space, Some(512 * 1024 * 1024));
        assert_eq!(info.cluster_size, 4096);
    }

    #[test]
    fn test_filesystem_info_no_label() {
        let info = FilesystemInfo {
            fs_type: FilesystemType::Ntfs,
            label: None,
            total_size: 500_000_000_000, // 500 GB
            free_space: None,
            cluster_size: 4096,
        };
        
        assert_eq!(info.label, None);
        assert_eq!(info.free_space, None);
    }

    #[test]
    fn test_filesystem_info_clone() {
        let info = FilesystemInfo {
            fs_type: FilesystemType::Ext4,
            label: Some("root".to_string()),
            total_size: 100_000_000_000,
            free_space: Some(50_000_000_000),
            cluster_size: 4096,
        };
        
        let cloned = info.clone();
        assert_eq!(cloned.fs_type, FilesystemType::Ext4);
        assert_eq!(cloned.label, Some("root".to_string()));
    }

    #[test]
    fn test_filesystem_info_debug() {
        let info = FilesystemInfo {
            fs_type: FilesystemType::HfsPlus,
            label: Some("Macintosh HD".to_string()),
            total_size: 256_000_000_000,
            free_space: Some(100_000_000_000),
            cluster_size: 4096,
        };
        
        let debug = format!("{:?}", info);
        assert!(debug.contains("HfsPlus"));
        assert!(debug.contains("Macintosh HD"));
    }
}
