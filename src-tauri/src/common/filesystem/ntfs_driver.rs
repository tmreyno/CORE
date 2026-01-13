// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # NTFS Filesystem Driver
//!
//! Implements NTFS filesystem access using the ntfs crate.

use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;

use ntfs::NtfsReadSeek;
use crate::common::vfs::{VfsError, FileAttr, DirEntry, normalize_path};
use super::traits::{FilesystemDriver, FilesystemInfo, FilesystemType, SeekableBlockDevice};

// =============================================================================
// NTFS Driver
// =============================================================================

/// NTFS filesystem driver
pub struct NtfsDriver {
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

impl NtfsDriver {
    /// Create a new NTFS driver
    pub fn new(
        device: Box<dyn SeekableBlockDevice>,
        offset: u64,
        size: u64,
    ) -> Result<Self, VfsError> {
        let device: Arc<dyn SeekableBlockDevice> = Arc::from(device);
        
        // Read boot sector to get volume info
        let mut boot_sector = vec![0u8; 512];
        device.read_at(offset, &mut boot_sector)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        // Verify NTFS signature
        if &boot_sector[3..11] != b"NTFS    " {
            return Err(VfsError::Internal(
                "Invalid NTFS signature".to_string()
            ));
        }
        
        // Parse BPB
        let bytes_per_sector = u16::from_le_bytes([boot_sector[11], boot_sector[12]]) as u32;
        let sectors_per_cluster = boot_sector[13] as u32;
        let cluster_size = bytes_per_sector * sectors_per_cluster;
        
        // Get total sectors
        let total_sectors = u64::from_le_bytes([
            boot_sector[40], boot_sector[41], boot_sector[42], boot_sector[43],
            boot_sector[44], boot_sector[45], boot_sector[46], boot_sector[47],
        ]);
        let total_size = total_sectors * bytes_per_sector as u64;
        
        let info = FilesystemInfo {
            fs_type: FilesystemType::Ntfs,
            label: None, // Will be read from $Volume later
            total_size,
            free_space: None, // Would need to scan $Bitmap
            cluster_size,
        };
        
        Ok(Self {
            info,
            device,
            offset,
            size,
        })
    }
    
    /// Create I/O wrapper for ntfs crate
    fn create_io_wrapper(&self) -> NtfsIoWrapper {
        NtfsIoWrapper {
            device: Arc::clone(&self.device),
            offset: self.offset,
            position: 0,
            size: self.size,
        }
    }
}

impl FilesystemDriver for NtfsDriver {
    fn info(&self) -> &FilesystemInfo {
        &self.info
    }
    
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let normalized = normalize_path(path);
        let mut io = self.create_io_wrapper();
        
        let fs = ntfs::Ntfs::new(&mut io)
            .map_err(|e| VfsError::IoError(format!("Failed to open NTFS: {:?}", e)))?;
        
        if normalized == "/" {
            return Ok(FileAttr {
                size: 0,
                is_directory: true,
                permissions: 0o555,
                nlink: 2,
                inode: 5, // Root directory record number in NTFS
                ..Default::default()
            });
        }
        
        let root = fs.root_directory(&mut io)
            .map_err(|e| VfsError::IoError(format!("Failed to get root: {:?}", e)))?;
        
        let entry = find_entry(&fs, &mut io, &root, &normalized)?;
        
        let is_dir = entry.is_directory();
        let size = if is_dir { 0u64 } else { entry.allocated_size() as u64 };
        
        Ok(FileAttr {
            size,
            is_directory: is_dir,
            permissions: if is_dir { 0o555 } else { 0o444 },
            nlink: 1,
            inode: entry.file_record_number(),
            ..Default::default()
        })
    }
    
    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let normalized = normalize_path(path);
        let mut io = self.create_io_wrapper();
        
        let fs = ntfs::Ntfs::new(&mut io)
            .map_err(|e| VfsError::IoError(format!("Failed to open NTFS: {:?}", e)))?;
        
        let root = fs.root_directory(&mut io)
            .map_err(|e| VfsError::IoError(format!("Failed to get root: {:?}", e)))?;
        
        let dir = if normalized == "/" {
            root
        } else {
            let entry = find_entry(&fs, &mut io, &root, &normalized)?;
            if !entry.is_directory() {
                return Err(VfsError::NotADirectory(normalized));
            }
            entry
        };
        
        let index = dir.directory_index(&mut io)
            .map_err(|e| VfsError::IoError(format!("Failed to get directory index: {:?}", e)))?;
        
        let mut entries = Vec::new();
        let mut iter = index.entries();
        
        while let Some(result) = iter.next(&mut io) {
            let entry = result
                .map_err(|e| VfsError::IoError(format!("Failed to read entry: {:?}", e)))?;
            
            let key = match entry.key() {
                Some(Ok(k)) => k,
                _ => continue,
            };
            
            let name = key.name();
            let name_str = name.to_string_lossy();
            
            // Skip system entries and . / ..
            if name_str == "." || name_str == ".." || name_str.starts_with('$') {
                continue;
            }
            
            let is_dir = key.is_directory();
            
            entries.push(DirEntry {
                name: name_str.to_string(),
                is_directory: is_dir,
                inode: entry.file_reference().file_record_number(),
                file_type: if is_dir { 4 } else { 8 },
            });
        }
        
        Ok(entries)
    }
    
    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path);
        let mut io = self.create_io_wrapper();
        
        let fs = ntfs::Ntfs::new(&mut io)
            .map_err(|e| VfsError::IoError(format!("Failed to open NTFS: {:?}", e)))?;
        
        let root = fs.root_directory(&mut io)
            .map_err(|e| VfsError::IoError(format!("Failed to get root: {:?}", e)))?;
        
        let entry = find_entry(&fs, &mut io, &root, &normalized)?;
        
        if entry.is_directory() {
            return Err(VfsError::NotAFile(normalized));
        }
        
        // Get $DATA attribute
        let data_item = entry.data(&mut io, "")
            .ok_or_else(|| VfsError::IoError("No data attribute found".to_string()))?
            .map_err(|e| VfsError::IoError(format!("Failed to get data attribute: {:?}", e)))?;
        
        let data_attr = data_item.to_attribute()
            .map_err(|e| VfsError::IoError(format!("Failed to get attribute: {:?}", e)))?;
        
        let file_size = data_attr.value_length();
        if offset >= file_size {
            return Ok(Vec::new());
        }
        
        let actual_size = size.min((file_size - offset) as usize);
        let mut buf = vec![0u8; actual_size];
        
        // Create a reader and read the data
        let mut reader = data_attr.value(&mut io)
            .map_err(|e| VfsError::IoError(format!("Failed to create reader: {:?}", e)))?;
        
        // Seek to offset
        reader.seek(&mut io, SeekFrom::Start(offset))
            .map_err(|e| VfsError::IoError(format!("Failed to seek: {:?}", e)))?;
        
        // Read data
        let bytes_read = reader.read(&mut io, &mut buf)
            .map_err(|e| VfsError::IoError(format!("Failed to read: {:?}", e)))?;
        
        buf.truncate(bytes_read);
        Ok(buf)
    }
}

// =============================================================================
// I/O Wrapper for NTFS
// =============================================================================

/// I/O wrapper that implements Read + Seek for ntfs crate
pub struct NtfsIoWrapper {
    device: Arc<dyn SeekableBlockDevice>,
    offset: u64,
    position: u64,
    size: u64,
}

impl Read for NtfsIoWrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.device.read_at(self.offset + self.position, buf)
            .map_err(std::io::Error::other)?;
        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl Seek for NtfsIoWrapper {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::Current(p) => {
                if p >= 0 {
                    self.position.saturating_add(p as u64)
                } else {
                    self.position.saturating_sub((-p) as u64)
                }
            }
            SeekFrom::End(p) => {
                if p >= 0 {
                    self.size.saturating_add(p as u64)
                } else {
                    self.size.saturating_sub((-p) as u64)
                }
            }
        };
        self.position = new_pos;
        Ok(new_pos)
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Find an entry by path
fn find_entry<'n, T: Read + Seek>(
    fs: &'n ntfs::Ntfs,
    io: &mut T,
    root: &ntfs::NtfsFile<'n>,
    path: &str,
) -> Result<ntfs::NtfsFile<'n>, VfsError> {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return Err(VfsError::Internal("Cannot get entry for root".to_string()));
    }
    
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current = root.clone();
    
    for part in parts.iter() {
        if !current.is_directory() {
            return Err(VfsError::NotADirectory(path.to_string()));
        }
        
        let index = current.directory_index(io)
            .map_err(|e| VfsError::IoError(format!("Failed to get directory index: {:?}", e)))?;
        
        let mut found = None;
        let mut iter = index.entries();
        
        while let Some(result) = iter.next(io) {
            let entry = result
                .map_err(|e| VfsError::IoError(format!("Failed to read entry: {:?}", e)))?;
            
            if let Some(Ok(key)) = entry.key() {
                let name = key.name().to_string_lossy();
                if name.eq_ignore_ascii_case(part) {
                    let file_ref = entry.file_reference();
                    let file = file_ref.to_file(fs, io)
                        .map_err(|e| VfsError::IoError(format!("Failed to open file: {:?}", e)))?;
                    found = Some(file);
                    break;
                }
            }
        }
        
        match found {
            Some(file) => current = file,
            None => return Err(VfsError::NotFound(path.to_string())),
        }
    }
    
    Ok(current)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ntfs_filesystem_type() {
        assert_eq!(FilesystemType::Ntfs.to_string(), "NTFS");
    }
}
