// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # FAT Filesystem Driver
//!
//! Implements FAT12/16/32 filesystem access using the fatfs crate.
//! Provides read-only directory listing, file attribute retrieval, and file
//! content reading from FAT-formatted partitions within forensic disk images.

use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;

use super::traits::{FilesystemDriver, FilesystemInfo, FilesystemType, SeekableBlockDevice};
use crate::vfs::{normalize_path, DirEntry, FileAttr, VfsError};

const FAT_BPB_MIN_LEN: usize = 14;
const FAT12_16_SIGNATURE_END: usize = 62;
const FAT32_SIGNATURE_END: usize = 90;

// =============================================================================
// FAT Driver
// =============================================================================

/// FAT filesystem driver
pub struct FatDriver {
    /// Filesystem info
    info: FilesystemInfo,
    /// Block device
    device: Arc<dyn SeekableBlockDevice>,
    /// Partition offset
    offset: u64,
    /// Partition size
    #[allow(dead_code)]
    size: u64,
}

impl FatDriver {
    /// Create a new FAT driver
    pub fn new(
        device: Box<dyn SeekableBlockDevice>,
        offset: u64,
        size: u64,
    ) -> Result<Self, VfsError> {
        let device: Arc<dyn SeekableBlockDevice> = Arc::from(device);

        // Read boot sector to determine FAT type
        let mut boot_sector = vec![0u8; 512];
        let boot_sector_len = device
            .read_at(offset, &mut boot_sector)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        if boot_sector_len < FAT_BPB_MIN_LEN {
            return Err(VfsError::IoError(format!(
                "FAT boot sector too short: read {} bytes",
                boot_sector_len
            )));
        }

        let boot_sector = &boot_sector[..boot_sector_len];

        // Parse BPB (BIOS Parameter Block)
        let bytes_per_sector = u16::from_le_bytes([boot_sector[11], boot_sector[12]]) as u32;
        let sectors_per_cluster = boot_sector[13] as u32;
        let cluster_size = bytes_per_sector * sectors_per_cluster;

        // Determine FAT type from signature
        let fs_type = if boot_sector.len() >= FAT32_SIGNATURE_END
            && &boot_sector[82..90] == b"FAT32   "
        {
            FilesystemType::Fat32
        } else if boot_sector.len() >= FAT12_16_SIGNATURE_END && &boot_sector[54..62] == b"FAT16   "
        {
            FilesystemType::Fat16
        } else if boot_sector.len() >= FAT12_16_SIGNATURE_END && &boot_sector[54..62] == b"FAT12   "
        {
            FilesystemType::Fat12
        } else if boot_sector.len() < FAT12_16_SIGNATURE_END {
            return Err(VfsError::IoError(format!(
                "FAT boot sector too short to identify filesystem type: read {} bytes",
                boot_sector.len()
            )));
        } else if boot_sector.len() < FAT32_SIGNATURE_END {
            return Err(VfsError::IoError(format!(
                "FAT boot sector too short for FAT32 signature: read {} bytes",
                boot_sector.len()
            )));
        } else {
            FilesystemType::Fat32 // Default assumption
        };

        // Get volume label
        let label_offset = if fs_type == FilesystemType::Fat32 {
            71
        } else {
            43
        };
        let label = if boot_sector.len() >= label_offset + 11 {
            let label_bytes = &boot_sector[label_offset..label_offset + 11];
            let label = String::from_utf8_lossy(label_bytes).trim().to_string();
            if label.is_empty() || label == "NO NAME" {
                None
            } else {
                Some(label)
            }
        } else {
            None
        };

        let info = FilesystemInfo {
            fs_type,
            label,
            total_size: size,
            free_space: None, // Would need to scan FAT to determine
            cluster_size,
        };

        Ok(Self {
            info,
            device,
            offset,
            size,
        })
    }

    /// Create a wrapper that implements Read + Write + Seek for fatfs
    fn create_io_wrapper(&self) -> FatIoWrapper {
        FatIoWrapper {
            device: Arc::clone(&self.device),
            offset: self.offset,
            position: 0,
            size: self.size,
        }
    }
}

impl FilesystemDriver for FatDriver {
    fn info(&self) -> &FilesystemInfo {
        &self.info
    }

    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let normalized = normalize_path(path);
        let io = self.create_io_wrapper();

        let fs = fatfs::FileSystem::new(io, fatfs::FsOptions::new())
            .map_err(|e| VfsError::IoError(format!("Failed to open FAT filesystem: {}", e)))?;

        if normalized == "/" {
            return Ok(FileAttr {
                size: 0,
                is_directory: true,
                permissions: 0o555,
                nlink: 2,
                inode: 1,
                ..Default::default()
            });
        }

        // Navigate to the entry and extract info before fs is dropped
        let root_dir = fs.root_dir();
        let result = navigate_to_entry(&root_dir, &normalized);

        let attr = match result {
            Ok(EntryType::Dir(_)) => FileAttr {
                size: 0,
                is_directory: true,
                permissions: 0o555,
                nlink: 2,
                inode: 0,
                ..Default::default()
            },
            Ok(EntryType::File(file)) => {
                // Get file size by seeking to end
                let mut f = file;
                let size = f.seek(SeekFrom::End(0)).unwrap_or(0);
                FileAttr {
                    size,
                    is_directory: false,
                    permissions: 0o444,
                    nlink: 1,
                    inode: 0,
                    ..Default::default()
                }
            }
            Err(e) => return Err(e),
        };

        Ok(attr)
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let normalized = normalize_path(path);
        let io = self.create_io_wrapper();

        let fs = fatfs::FileSystem::new(io, fatfs::FsOptions::new())
            .map_err(|e| VfsError::IoError(format!("Failed to open FAT filesystem: {}", e)))?;

        let root_dir = fs.root_dir();

        // Get directory to list
        let dir = if normalized == "/" {
            Ok(root_dir)
        } else {
            match navigate_to_entry(&root_dir, &normalized) {
                Ok(EntryType::Dir(d)) => Ok(d),
                Ok(EntryType::File(_)) => Err(VfsError::NotADirectory(normalized.clone())),
                Err(e) => Err(e),
            }
        }?;

        let mut entries = Vec::new();
        for entry_result in dir.iter() {
            let entry = entry_result
                .map_err(|e| VfsError::IoError(format!("Failed to read directory entry: {}", e)))?;

            let name = entry.file_name();
            if name == "." || name == ".." {
                continue;
            }

            entries.push(DirEntry {
                name,
                is_directory: entry.is_dir(),
                inode: 0,
                file_type: if entry.is_dir() { 4 } else { 8 },
            });
        }

        Ok(entries)
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path);
        let io = self.create_io_wrapper();

        let fs = fatfs::FileSystem::new(io, fatfs::FsOptions::new())
            .map_err(|e| VfsError::IoError(format!("Failed to open FAT filesystem: {}", e)))?;

        let root_dir = fs.root_dir();

        let mut file = match navigate_to_entry(&root_dir, &normalized) {
            Ok(EntryType::File(f)) => f,
            Ok(EntryType::Dir(_)) => return Err(VfsError::NotAFile(normalized)),
            Err(e) => return Err(e),
        };

        let file_size = file
            .seek(SeekFrom::End(0))
            .map_err(|e| VfsError::IoError(format!("Failed to get file size: {}", e)))?;

        if offset >= file_size {
            return Ok(Vec::new());
        }

        file.seek(SeekFrom::Start(offset))
            .map_err(|e| VfsError::IoError(format!("Failed to seek: {}", e)))?;

        let actual_size = match bounded_fat_read_len(file_size, offset, size) {
            Some(actual_size) => actual_size,
            None => return Ok(Vec::new()),
        };
        let mut buf = vec![0u8; actual_size];

        let bytes_read = file
            .read(&mut buf)
            .map_err(|e| VfsError::IoError(format!("Failed to read file: {}", e)))?;

        buf.truncate(bytes_read);
        Ok(buf)
    }
}

fn bounded_fat_read_len(file_size: u64, offset: u64, requested_size: usize) -> Option<usize> {
    let remaining = file_size.checked_sub(offset)?;
    if remaining == 0 {
        return None;
    }

    let remaining_usize = usize::try_from(remaining).unwrap_or(usize::MAX);
    Some(requested_size.min(remaining_usize))
}

fn checked_fat_absolute_offset(base: u64, position: u64) -> Option<u64> {
    base.checked_add(position)
}

fn checked_fat_seek_position(base: u64, delta: i64) -> std::io::Result<u64> {
    if delta >= 0 {
        base.checked_add(delta as u64).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "FAT seek overflow")
        })
    } else {
        base.checked_sub(delta.unsigned_abs()).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "FAT seek before start")
        })
    }
}

// =============================================================================
// I/O Wrapper for fatfs
// =============================================================================

/// I/O wrapper that implements Read + Write + Seek for fatfs
struct FatIoWrapper {
    device: Arc<dyn SeekableBlockDevice>,
    offset: u64,
    position: u64,
    size: u64,
}

impl Read for FatIoWrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let absolute_offset =
            checked_fat_absolute_offset(self.offset, self.position).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "FAT read offset overflow")
            })?;
        let bytes_read = self
            .device
            .read_at(absolute_offset, buf)
            .map_err(std::io::Error::other)?;
        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl Write for FatIoWrapper {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        // Read-only filesystem
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Read-only filesystem",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for FatIoWrapper {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::Current(p) => checked_fat_seek_position(self.position, p)?,
            SeekFrom::End(p) => checked_fat_seek_position(self.size, p)?,
        };
        self.position = new_pos;
        Ok(new_pos)
    }
}

// =============================================================================
// Helper Types and Functions
// =============================================================================

/// Result of navigating to a path entry
enum EntryType<'a, IO: Read + Write + Seek> {
    Dir(fatfs::Dir<'a, IO>),
    File(fatfs::File<'a, IO>),
}

/// Navigate to a path and return the entry (file or directory)
fn navigate_to_entry<'a, IO: Read + Write + Seek>(
    root: &fatfs::Dir<'a, IO>,
    path: &str,
) -> Result<EntryType<'a, IO>, VfsError> {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        // For root, we can't return a proper Dir without ownership issues
        return Err(VfsError::Internal(
            "Use root_dir() directly for root".to_string(),
        ));
    }

    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current_dir = root.clone();

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        let mut found = false;

        for entry_result in current_dir.iter() {
            let entry = entry_result
                .map_err(|e| VfsError::IoError(format!("Failed to read directory: {}", e)))?;

            if entry.file_name().eq_ignore_ascii_case(part) {
                if is_last {
                    if entry.is_dir() {
                        return Ok(EntryType::Dir(entry.to_dir()));
                    } else {
                        return Ok(EntryType::File(entry.to_file()));
                    }
                } else if entry.is_dir() {
                    current_dir = entry.to_dir();
                    found = true;
                    break;
                } else {
                    return Err(VfsError::NotADirectory(path.to_string()));
                }
            }
        }

        if !found && !is_last {
            return Err(VfsError::NotFound(path.to_string()));
        }
    }

    Err(VfsError::NotFound(path.to_string()))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ffx_errors::ContainerError;
    use std::io::Cursor;

    struct TestDevice {
        data: Vec<u8>,
    }

    impl super::super::traits::BlockReader for Cursor<Vec<u8>> {}

    impl super::super::traits::BlockDevice for TestDevice {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError> {
            let start = usize::try_from(offset).unwrap_or(usize::MAX);
            if start >= self.data.len() {
                return Ok(0);
            }

            let available = self.data.len() - start;
            let to_copy = available.min(buf.len());
            buf[..to_copy].copy_from_slice(&self.data[start..start + to_copy]);
            Ok(to_copy)
        }

        fn size(&self) -> u64 {
            self.data.len() as u64
        }
    }

    impl SeekableBlockDevice for TestDevice {
        fn reader_at(&self, offset: u64) -> Box<dyn super::super::traits::BlockReader> {
            let start = usize::try_from(offset)
                .unwrap_or(usize::MAX)
                .min(self.data.len());
            Box::new(Cursor::new(self.data[start..].to_vec()))
        }
    }

    fn fat32_boot_sector() -> Vec<u8> {
        let mut boot_sector = vec![0u8; 512];
        boot_sector[11..13].copy_from_slice(&512u16.to_le_bytes());
        boot_sector[13] = 8;
        boot_sector[71..82].copy_from_slice(b"COREFAT32  ");
        boot_sector[82..90].copy_from_slice(b"FAT32   ");
        boot_sector
    }

    #[test]
    fn test_fat_filesystem_type() {
        assert_eq!(FilesystemType::Fat32.to_string(), "FAT32");
    }

    #[test]
    fn test_fat_new_rejects_short_boot_sector() {
        let result = FatDriver::new(
            Box::new(TestDevice {
                data: vec![0u8; 13],
            }),
            0,
            13,
        );
        assert!(matches!(result, Err(VfsError::IoError(message)) if message.contains("too short")));
    }

    #[test]
    fn test_fat_new_uses_bytes_actually_read() {
        let result = FatDriver::new(
            Box::new(TestDevice {
                data: fat32_boot_sector()[..64].to_vec(),
            }),
            0,
            64,
        );
        assert!(
            matches!(result, Err(VfsError::IoError(message)) if message.contains("FAT32 signature"))
        );
    }

    #[test]
    fn test_fat_new_parses_valid_fat32_boot_sector() {
        let driver = FatDriver::new(
            Box::new(TestDevice {
                data: fat32_boot_sector(),
            }),
            0,
            512,
        )
        .unwrap();

        assert_eq!(driver.info().fs_type, FilesystemType::Fat32);
        assert_eq!(driver.info().cluster_size, 4096);
        assert_eq!(driver.info().label.as_deref(), Some("COREFAT32"));
    }

    #[test]
    fn test_bounded_fat_read_len_rejects_eof() {
        assert_eq!(bounded_fat_read_len(128, 128, 16), None);
    }

    #[test]
    fn test_bounded_fat_read_len_clamps_to_remaining() {
        assert_eq!(bounded_fat_read_len(128, 120, 16), Some(8));
    }

    #[test]
    fn test_bounded_fat_read_len_handles_large_remaining() {
        assert_eq!(bounded_fat_read_len(u64::MAX, 0, 32), Some(32));
    }

    #[test]
    fn test_checked_fat_absolute_offset_rejects_overflow() {
        assert_eq!(checked_fat_absolute_offset(u64::MAX, 1), None);
    }

    #[test]
    fn test_checked_fat_seek_position_rejects_underflow() {
        let err = checked_fat_seek_position(0, -1).expect_err("seek before start should fail");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_checked_fat_seek_position_rejects_overflow() {
        let err = checked_fat_seek_position(u64::MAX, 1).expect_err("seek overflow should fail");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
}
