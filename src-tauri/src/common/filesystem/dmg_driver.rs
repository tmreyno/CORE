// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # DMG (Apple Disk Image) Driver
//!
//! Provides support for reading macOS DMG files by:
//! 1. Parsing the DMG container format using `apple-dmg`
//! 2. Decompressing partition data (UDIF format with zlib/bzip2/LZFSE)
//! 3. Exposing the raw partition as a `SeekableBlockDevice` for HFS+ parsing
//!
//! ## DMG Structure
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    DMG Container (UDIF)                     │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Compressed Blocks (zlib, bzip2, LZFSE, or raw)             │
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │ Partition 0: Driver (usually)                           ││
//! │  │ Partition 1: HFS+ Filesystem                            ││
//! │  │ Partition 2: (optional)                                 ││
//! │  └─────────────────────────────────────────────────────────┘│
//! ├─────────────────────────────────────────────────────────────┤
//! │  XML Plist (partition table, block maps)                    │
//! │  Koly Trailer (512 bytes at EOF)                            │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//! ```rust,ignore
//! use crate::common::filesystem::dmg_driver::DmgDriver;
//!
//! let dmg = DmgDriver::open("/path/to/disk.dmg")?;
//! 
//! // List partitions
//! for (i, name) in dmg.partitions().iter().enumerate() {
//!     println!("Partition {}: {}", i, name);
//! }
//!
//! // Get a block device for a partition (for HFS+ parsing)
//! let device = dmg.partition_device(1)?;
//! let hfs_driver = HfsPlusDriver::new(device, 0, device.size())?;
//! ```

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, RwLock};

use apple_dmg::DmgReader;

use crate::common::vfs::VfsError;
use crate::containers::ContainerError;
use super::traits::{BlockDevice, SeekableBlockDevice, BlockReader};

// =============================================================================
// DMG Driver
// =============================================================================

/// DMG container reader
/// 
/// Wraps `apple-dmg` to provide partition access for forensic analysis.
pub struct DmgDriver {
    /// Path to the DMG file
    path: String,
    /// Partition names
    partition_names: Vec<String>,
    /// Partition data cache (decompressed on demand)
    /// Using RwLock for interior mutability since decompression is expensive
    partition_cache: RwLock<Vec<Option<Vec<u8>>>>,
    /// DMG reader (needs mutable access for reading)
    reader: RwLock<DmgReader<BufReader<File>>>,
}

impl DmgDriver {
    /// Open a DMG file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, VfsError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let reader = DmgReader::open(path.as_ref())
            .map_err(|e| VfsError::IoError(format!("Failed to open DMG: {}", e)))?;
        
        // Extract partition names from plist
        let plist = reader.plist();
        let partition_names: Vec<String> = plist.partitions()
            .iter()
            .enumerate()
            .map(|(i, p)| {
                // Use the partition name, or fall back to index
                if p.name.is_empty() {
                    format!("Partition_{}", i)
                } else {
                    p.name.clone()
                }
            })
            .collect();
        
        let num_partitions = partition_names.len();
        
        Ok(Self {
            path: path_str,
            partition_names,
            partition_cache: RwLock::new(vec![None; num_partitions]),
            reader: RwLock::new(reader),
        })
    }
    
    /// Get the path to the DMG file
    pub fn path(&self) -> &str {
        &self.path
    }
    
    /// Get the number of partitions
    pub fn partition_count(&self) -> usize {
        self.partition_names.len()
    }
    
    /// Get partition names
    pub fn partitions(&self) -> &[String] {
        &self.partition_names
    }
    
    /// Get partition info
    pub fn partition_info(&self, index: usize) -> Option<DmgPartitionInfo> {
        if index >= self.partition_names.len() {
            return None;
        }
        
        let reader = self.reader.read().ok()?;
        let plist = reader.plist();
        
        if index >= plist.partitions().len() {
            return None;
        }
        
        let partition = &plist.partitions()[index];
        
        // Get size from table if available
        let size = partition.table()
            .map(|t| t.sector_count * 512)
            .unwrap_or(0);
        
        Some(DmgPartitionInfo {
            index,
            name: self.partition_names[index].clone(),
            size,
        })
    }
    
    /// Find the main HFS+ partition (usually the largest one)
    pub fn find_hfs_partition(&self) -> Option<usize> {
        let reader = self.reader.read().ok()?;
        let plist = reader.plist();
        
        // Look for partition with "Apple_HFS" or similar in name
        for (i, name) in self.partition_names.iter().enumerate() {
            let name_lower = name.to_lowercase();
            if name_lower.contains("apple_hfs") || 
               name_lower.contains("hfs") ||
               name_lower.contains("apple hfs") {
                return Some(i);
            }
        }
        
        // Fallback: find the largest partition (usually the HFS+ one)
        let mut largest_idx = 0;
        let mut largest_size = 0u64;
        
        for (i, partition) in plist.partitions().iter().enumerate() {
            if let Ok(table) = partition.table() {
                let size = table.sector_count * 512;
                if size > largest_size {
                    largest_size = size;
                    largest_idx = i;
                }
            }
        }
        
        if largest_size > 0 {
            Some(largest_idx)
        } else {
            None
        }
    }
    
    /// Get decompressed partition data
    /// 
    /// This decompresses the entire partition into memory.
    /// For large DMGs, this may use significant memory.
    pub fn partition_data(&self, index: usize) -> Result<Vec<u8>, VfsError> {
        // Check cache first
        {
            let cache = self.partition_cache.read()
                .map_err(|_| VfsError::Internal("Cache lock poisoned".to_string()))?;
            
            if let Some(Some(data)) = cache.get(index) {
                return Ok(data.clone());
            }
        }
        
        // Decompress partition
        let data = {
            let mut reader = self.reader.write()
                .map_err(|_| VfsError::Internal("Reader lock poisoned".to_string()))?;
            
            reader.partition_data(index)
                .map_err(|e| VfsError::IoError(format!("Failed to decompress partition {}: {}", index, e)))?
        };
        
        // Cache the result
        {
            let mut cache = self.partition_cache.write()
                .map_err(|_| VfsError::Internal("Cache lock poisoned".to_string()))?;
            
            if index < cache.len() {
                cache[index] = Some(data.clone());
            }
        }
        
        Ok(data)
    }
    
    /// Get a block device for a partition
    /// 
    /// Returns a `SeekableBlockDevice` that can be used with filesystem drivers.
    pub fn partition_device(&self, index: usize) -> Result<Box<dyn SeekableBlockDevice>, VfsError> {
        let data = self.partition_data(index)?;
        Ok(Box::new(MemoryBlockDevice::new(data)))
    }
}

/// Information about a DMG partition
#[derive(Debug, Clone)]
pub struct DmgPartitionInfo {
    /// Partition index
    pub index: usize,
    /// Partition name
    pub name: String,
    /// Partition size in bytes
    pub size: u64,
}

// =============================================================================
// Memory Block Device
// =============================================================================

/// A block device backed by in-memory data
/// 
/// Used to wrap decompressed DMG partition data for filesystem drivers.
pub struct MemoryBlockDevice {
    data: Arc<Vec<u8>>,
}

impl MemoryBlockDevice {
    /// Create a new memory block device from data
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(data),
        }
    }
    
    /// Create from existing Arc'd data
    pub fn from_arc(data: Arc<Vec<u8>>) -> Self {
        Self { data }
    }
}

impl BlockDevice for MemoryBlockDevice {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError> {
        let offset = offset as usize;
        
        if offset >= self.data.len() {
            return Ok(0);
        }
        
        let available = self.data.len() - offset;
        let to_read = buf.len().min(available);
        
        buf[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
        
        Ok(to_read)
    }
    
    fn size(&self) -> u64 {
        self.data.len() as u64
    }
}

impl SeekableBlockDevice for MemoryBlockDevice {
    fn reader_at(&self, offset: u64) -> Box<dyn BlockReader> {
        Box::new(MemoryBlockReader {
            data: Arc::clone(&self.data),
            position: offset as usize,
        })
    }
}

/// Reader for memory block device
struct MemoryBlockReader {
    data: Arc<Vec<u8>>,
    position: usize,
}

impl Read for MemoryBlockReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.position >= self.data.len() {
            return Ok(0);
        }
        
        let available = self.data.len() - self.position;
        let to_read = buf.len().min(available);
        
        buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
        self.position += to_read;
        
        Ok(to_read)
    }
}

impl Seek for MemoryBlockReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.data.len() as i64 + offset,
            SeekFrom::Current(offset) => self.position as i64 + offset,
        };
        
        if new_pos < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Seek before start of data",
            ));
        }
        
        self.position = new_pos as usize;
        Ok(self.position as u64)
    }
}

impl BlockReader for MemoryBlockReader {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_block_device() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let device = MemoryBlockDevice::new(data);
        
        assert_eq!(device.size(), 10);
        
        let mut buf = [0u8; 4];
        let read = device.read_at(2, &mut buf).unwrap();
        assert_eq!(read, 4);
        assert_eq!(buf, [2, 3, 4, 5]);
    }
    
    #[test]
    fn test_memory_block_device_eof() {
        let data = vec![0, 1, 2, 3, 4];
        let device = MemoryBlockDevice::new(data);
        
        let mut buf = [0u8; 10];
        let read = device.read_at(3, &mut buf).unwrap();
        assert_eq!(read, 2); // Only 2 bytes available
        assert_eq!(&buf[..2], [3, 4]);
    }
    
    #[test]
    fn test_memory_block_device_past_eof() {
        let data = vec![0, 1, 2];
        let device = MemoryBlockDevice::new(data);
        
        let mut buf = [0u8; 4];
        let read = device.read_at(100, &mut buf).unwrap();
        assert_eq!(read, 0);
    }
    
    #[test]
    fn test_memory_block_reader() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let device = MemoryBlockDevice::new(data);
        
        let mut reader = device.reader_at(0);
        
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [0, 1, 2]);
        
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [3, 4, 5]);
        
        // Seek to position 8
        reader.seek(SeekFrom::Start(8)).unwrap();
        reader.read_exact(&mut buf[..2]).unwrap();
        assert_eq!(&buf[..2], [8, 9]);
    }
}
