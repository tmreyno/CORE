// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # exFAT Filesystem Driver
//!
//! Read-only exFAT filesystem driver for forensic disk images.
//! Implements the FilesystemDriver trait by manually parsing on-disk structures.
//!
//! ## exFAT On-Disk Layout
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │  Boot Sector (sector 0)  — VBR with exFAT params     │
//! │  Backup Boot Sector (sector 12)                      │
//! ├──────────────────────────────────────────────────────┤
//! │  FAT Region  — Cluster allocation table              │
//! ├──────────────────────────────────────────────────────┤
//! │  Cluster Heap  — File/directory data starting at     │
//! │                  cluster 2                           │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Directory Entry Types
//! - 0x85: File Directory Entry (primary)
//! - 0xC0: Stream Extension Entry (file size, start cluster, flags)
//! - 0xC1: File Name Extension Entry (up to 15 UTF-16 chars per entry)
//! - 0x83: Volume Label Entry
//! - 0x81: Allocation Bitmap Entry
//! - 0x82: Up-Case Table Entry

use std::sync::Arc;
use tracing::debug;

use super::traits::{FilesystemDriver, FilesystemInfo, FilesystemType, SeekableBlockDevice};
use crate::common::vfs::{VfsError, FileAttr, DirEntry};

// =============================================================================
// Constants
// =============================================================================

/// exFAT boot sector signature
const EXFAT_SIGNATURE: &[u8; 8] = b"EXFAT   ";
/// Boot signature at offset 510
const BOOT_SIGNATURE: u16 = 0xAA55;

// Directory entry type codes
const ENTRY_TYPE_END: u8 = 0x00;
const ENTRY_TYPE_FILE: u8 = 0x85;
const ENTRY_TYPE_STREAM: u8 = 0xC0;
const ENTRY_TYPE_FILENAME: u8 = 0xC1;
const ENTRY_TYPE_VOLUME_LABEL: u8 = 0x83;
#[allow(dead_code)] // Forensic format constant — retained for documentation
const ENTRY_TYPE_ALLOC_BITMAP: u8 = 0x81;
#[allow(dead_code)] // Forensic format constant — retained for documentation
const ENTRY_TYPE_UPCASE: u8 = 0x82;

// File attribute flags (in File Directory Entry)
const ATTR_READ_ONLY: u16 = 0x01;
#[allow(dead_code)] // Forensic format constant — retained for documentation
const ATTR_HIDDEN: u16 = 0x02;
#[allow(dead_code)] // Forensic format constant — retained for documentation
const ATTR_SYSTEM: u16 = 0x04;
const ATTR_DIRECTORY: u16 = 0x10;
#[allow(dead_code)] // Forensic format constant — retained for documentation
const ATTR_ARCHIVE: u16 = 0x20;

// Stream extension flags
const STREAM_FLAG_NO_FAT_CHAIN: u8 = 0x02;

// =============================================================================
// Boot Sector
// =============================================================================

/// Parsed exFAT boot sector
///
/// All fields are retained for forensic completeness, even if not all are
/// actively used by the read-only driver. The full boot sector structure
/// is important for format documentation and potential future use.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ExFatBootSector {
    /// Bytes per sector (power of 2, typically 512)
    bytes_per_sector_shift: u8,
    /// Sectors per cluster (power of 2)
    sectors_per_cluster_shift: u8,
    /// Number of FATs (1 or 2)
    number_of_fats: u8,
    /// Offset of first FAT in sectors
    fat_offset: u32,
    /// Length of FAT in sectors  
    fat_length: u32,
    /// Offset of cluster heap in sectors
    cluster_heap_offset: u32,
    /// Total clusters in cluster heap
    cluster_count: u32,
    /// First cluster of root directory
    root_directory_cluster: u32,
    /// Volume serial number
    volume_serial: u32,
    /// Volume label (from boot sector — may be overridden by dir entry)
    volume_label: Option<String>,
    /// Computed: bytes per sector
    bytes_per_sector: u32,
    /// Computed: bytes per cluster
    bytes_per_cluster: u32,
    /// Computed: total volume size
    volume_size: u64,
}

impl ExFatBootSector {
    /// Parse boot sector from raw bytes (must be at least 512 bytes)
    fn parse(data: &[u8]) -> Result<Self, VfsError> {
        if data.len() < 512 {
            return Err(VfsError::Internal("Boot sector too small".to_string()));
        }

        // Verify signature at offset 3
        if &data[3..11] != EXFAT_SIGNATURE {
            return Err(VfsError::Internal("Not an exFAT filesystem".to_string()));
        }

        // Verify boot signature at offset 510
        let sig = u16::from_le_bytes([data[510], data[511]]);
        if sig != BOOT_SIGNATURE {
            return Err(VfsError::Internal(format!(
                "Invalid boot signature: 0x{:04X} (expected 0xAA55)", sig
            )));
        }

        let bytes_per_sector_shift = data[108];
        let sectors_per_cluster_shift = data[109];
        let number_of_fats = data[110];
        let fat_offset = u32::from_le_bytes([data[80], data[81], data[82], data[83]]);
        let fat_length = u32::from_le_bytes([data[84], data[85], data[86], data[87]]);
        let cluster_heap_offset = u32::from_le_bytes([data[88], data[89], data[90], data[91]]);
        let cluster_count = u32::from_le_bytes([data[92], data[93], data[94], data[95]]);
        let root_directory_cluster = u32::from_le_bytes([data[96], data[97], data[98], data[99]]);
        let volume_serial = u32::from_le_bytes([data[100], data[101], data[102], data[103]]);

        // Validate ranges
        if bytes_per_sector_shift < 9 || bytes_per_sector_shift > 12 {
            return Err(VfsError::Internal(format!(
                "Invalid bytes_per_sector_shift: {} (must be 9-12)", bytes_per_sector_shift
            )));
        }
        if sectors_per_cluster_shift > 25u8.saturating_sub(bytes_per_sector_shift) {
            return Err(VfsError::Internal(format!(
                "Invalid sectors_per_cluster_shift: {}", sectors_per_cluster_shift
            )));
        }

        let bytes_per_sector = 1u32 << bytes_per_sector_shift;
        let bytes_per_cluster = bytes_per_sector << sectors_per_cluster_shift;
        let volume_size = (cluster_count as u64) * (bytes_per_cluster as u64);

        Ok(ExFatBootSector {
            bytes_per_sector_shift,
            sectors_per_cluster_shift,
            number_of_fats,
            fat_offset,
            fat_length,
            cluster_heap_offset,
            cluster_count,
            root_directory_cluster,
            volume_serial,
            volume_label: None,
            bytes_per_sector,
            bytes_per_cluster,
            volume_size,
        })
    }

    /// Calculate byte offset for a given cluster number
    fn cluster_offset(&self, cluster: u32) -> u64 {
        let heap_start = (self.cluster_heap_offset as u64) * (self.bytes_per_sector as u64);
        heap_start + ((cluster as u64 - 2) * (self.bytes_per_cluster as u64))
    }
}

// =============================================================================
// Parsed Directory Entry
// =============================================================================

/// A fully parsed file/directory entry combining File + Stream + FileName entries
#[derive(Debug, Clone)]
struct ExFatFileEntry {
    /// File name (UTF-8)
    name: String,
    /// Whether this is a directory
    is_directory: bool,
    /// File size in bytes (0 for directories)
    size: u64,
    /// First cluster of data
    start_cluster: u32,
    /// Whether the allocation is contiguous (no FAT chain needed)
    no_fat_chain: bool,
    /// File attributes
    attributes: u16,
    /// Created timestamp (DOS format)
    created: Option<u64>,
    /// Modified timestamp (DOS format)
    modified: Option<u64>,
    /// Accessed timestamp (DOS format)
    accessed: Option<u64>,
}

// =============================================================================
// exFAT Driver
// =============================================================================

/// Read-only exFAT filesystem driver
pub struct ExFatDriver {
    /// Underlying block device
    device: Arc<dyn SeekableBlockDevice>,
    /// Partition offset in bytes
    offset: u64,
    /// Parsed boot sector
    boot: ExFatBootSector,
    /// Filesystem info
    info: FilesystemInfo,
}

impl ExFatDriver {
    /// Create a new exFAT driver for the given block device and partition
    pub fn new(
        device: Box<dyn SeekableBlockDevice>,
        offset: u64,
        size: u64,
    ) -> Result<Self, VfsError> {
        let device: Arc<dyn SeekableBlockDevice> = Arc::from(device);

        // Read boot sector
        let mut buf = vec![0u8; 512];
        device.read_at(offset, &mut buf)
            .map_err(|e| VfsError::IoError(format!("Failed to read exFAT boot sector: {}", e)))?;

        let boot = ExFatBootSector::parse(&buf)?;

        debug!(
            "exFAT: bytes_per_sector={}, bytes_per_cluster={}, cluster_count={}, root_cluster={}, heap_offset={}",
            boot.bytes_per_sector, boot.bytes_per_cluster,
            boot.cluster_count, boot.root_directory_cluster, boot.cluster_heap_offset
        );

        // Try to read volume label from root directory
        let mut label = None;
        if let Ok(root_data) = Self::read_cluster_chain_static(
            &device, offset, &boot, boot.root_directory_cluster, boot.bytes_per_cluster as usize,
        ) {
            label = Self::find_volume_label(&root_data);
        }

        let total_size = if size > 0 { size } else { boot.volume_size };

        let info = FilesystemInfo {
            fs_type: FilesystemType::ExFat,
            label: label.clone(),
            total_size,
            free_space: None, // Would require reading allocation bitmap
            cluster_size: boot.bytes_per_cluster,
        };

        Ok(ExFatDriver {
            device,
            offset,
            boot,
            info,
        })
    }

    /// Read a single cluster's data
    fn read_cluster(&self, cluster: u32) -> Result<Vec<u8>, VfsError> {
        let cluster_offset = self.offset + self.boot.cluster_offset(cluster);
        let mut buf = vec![0u8; self.boot.bytes_per_cluster as usize];
        self.device.read_at(cluster_offset, &mut buf)
            .map_err(|e| VfsError::IoError(format!("Failed to read cluster {}: {}", cluster, e)))?;
        Ok(buf)
    }

    /// Read a FAT entry to find the next cluster in a chain
    fn read_fat_entry(&self, cluster: u32) -> Result<u32, VfsError> {
        let fat_byte_offset = self.offset
            + (self.boot.fat_offset as u64) * (self.boot.bytes_per_sector as u64)
            + (cluster as u64) * 4;
        let mut buf = [0u8; 4];
        self.device.read_at(fat_byte_offset, &mut buf)
            .map_err(|e| VfsError::IoError(format!("Failed to read FAT entry: {}", e)))?;
        Ok(u32::from_le_bytes(buf))
    }

    /// Read cluster chain data (static version for use during construction)
    fn read_cluster_chain_static(
        device: &Arc<dyn SeekableBlockDevice>,
        partition_offset: u64,
        boot: &ExFatBootSector,
        start_cluster: u32,
        max_bytes: usize,
    ) -> Result<Vec<u8>, VfsError> {
        let cluster_offset = partition_offset + boot.cluster_offset(start_cluster);
        let read_size = max_bytes.min(boot.bytes_per_cluster as usize);
        let mut buf = vec![0u8; read_size];
        device.read_at(cluster_offset, &mut buf)
            .map_err(|e| VfsError::IoError(format!("Failed to read cluster: {}", e)))?;
        Ok(buf)
    }

    /// Read all data for a file/directory, following the cluster chain if necessary
    fn read_data(&self, entry: &ExFatFileEntry) -> Result<Vec<u8>, VfsError> {
        if entry.start_cluster < 2 {
            return Ok(Vec::new());
        }

        let data_size = if entry.is_directory {
            // For directories, read at least one cluster
            self.boot.bytes_per_cluster as usize
        } else {
            entry.size as usize
        };

        if entry.no_fat_chain {
            // Contiguous allocation — read directly
            let clusters_needed = (data_size + self.boot.bytes_per_cluster as usize - 1)
                / self.boot.bytes_per_cluster as usize;
            let mut result = Vec::with_capacity(data_size);

            for i in 0..clusters_needed {
                let cluster = entry.start_cluster + i as u32;
                let cluster_data = self.read_cluster(cluster)?;
                result.extend_from_slice(&cluster_data);
            }

            result.truncate(data_size);
            Ok(result)
        } else {
            // Follow FAT chain
            let mut result = Vec::with_capacity(data_size);
            let mut current_cluster = entry.start_cluster;
            let max_clusters = (data_size + self.boot.bytes_per_cluster as usize - 1)
                / self.boot.bytes_per_cluster as usize;

            for _ in 0..max_clusters {
                if current_cluster < 2 || current_cluster >= 0xFFFFFFF7 {
                    break;
                }
                let cluster_data = self.read_cluster(current_cluster)?;
                result.extend_from_slice(&cluster_data);

                // Read next cluster from FAT
                current_cluster = self.read_fat_entry(current_cluster)?;
            }

            result.truncate(data_size);
            Ok(result)
        }
    }

    /// Find volume label from root directory entries
    fn find_volume_label(dir_data: &[u8]) -> Option<String> {
        let mut offset = 0;
        while offset + 32 <= dir_data.len() {
            let entry_type = dir_data[offset];
            if entry_type == ENTRY_TYPE_END {
                break;
            }
            if entry_type == ENTRY_TYPE_VOLUME_LABEL {
                let char_count = dir_data[offset + 1] as usize;
                if char_count > 0 && char_count <= 11 {
                    let label_bytes = &dir_data[offset + 2..offset + 2 + char_count * 2];
                    let chars: Vec<u16> = label_bytes
                        .chunks(2)
                        .map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
                        .collect();
                    return Some(String::from_utf16_lossy(&chars).trim().to_string());
                }
            }
            offset += 32;
        }
        None
    }

    /// Parse directory entries from raw directory data
    fn parse_directory_entries(&self, dir_data: &[u8]) -> Vec<ExFatFileEntry> {
        let mut entries = Vec::new();
        let mut offset = 0;

        while offset + 32 <= dir_data.len() {
            let entry_type = dir_data[offset];

            // End of directory
            if entry_type == ENTRY_TYPE_END {
                break;
            }

            // Skip unused/deleted entries (high bit clear = inactive)
            if entry_type & 0x80 == 0 && entry_type != ENTRY_TYPE_END {
                offset += 32;
                continue;
            }

            // File Directory Entry (0x85)
            if entry_type == ENTRY_TYPE_FILE {
                if let Some(entry) = self.parse_file_entry_set(dir_data, offset) {
                    entries.push(entry);
                }
            }

            offset += 32;
        }

        entries
    }

    /// Parse a complete File entry set (File + Stream + FileName entries)
    fn parse_file_entry_set(&self, data: &[u8], file_offset: usize) -> Option<ExFatFileEntry> {
        if file_offset + 32 > data.len() {
            return None;
        }

        let entry = &data[file_offset..file_offset + 32];
        if entry[0] != ENTRY_TYPE_FILE {
            return None;
        }

        let secondary_count = entry[1] as usize;
        let attributes = u16::from_le_bytes([entry[4], entry[5]]);
        let is_directory = (attributes & ATTR_DIRECTORY) != 0;

        // Parse timestamps (DOS format at offsets 8, 12, 16)
        let created = Self::parse_dos_timestamp(&entry[8..12]);
        let modified = Self::parse_dos_timestamp(&entry[12..16]);
        let accessed = Self::parse_dos_timestamp(&entry[20..24]);

        // Next entry should be Stream Extension (0xC0)
        let stream_offset = file_offset + 32;
        if stream_offset + 32 > data.len() || data[stream_offset] != ENTRY_TYPE_STREAM {
            return None;
        }

        let stream = &data[stream_offset..stream_offset + 32];
        let general_flags = stream[1];
        let no_fat_chain = (general_flags & STREAM_FLAG_NO_FAT_CHAIN) != 0;
        let name_length = stream[3] as usize;
        let start_cluster = u32::from_le_bytes([stream[20], stream[21], stream[22], stream[23]]);
        let data_length = u64::from_le_bytes([
            stream[24], stream[25], stream[26], stream[27],
            stream[28], stream[29], stream[30], stream[31],
        ]);

        // Collect file name from subsequent FileName entries (0xC1)
        let mut name_chars: Vec<u16> = Vec::with_capacity(name_length);
        let mut fname_offset = stream_offset + 32;

        // secondary_count includes stream entry, so remaining = secondary_count - 1
        let name_entries = secondary_count.saturating_sub(1);
        for _ in 0..name_entries {
            if fname_offset + 32 > data.len() || data[fname_offset] != ENTRY_TYPE_FILENAME {
                break;
            }
            // Each FileName entry has 15 UTF-16 characters at offset 2
            let fname_data = &data[fname_offset + 2..fname_offset + 32];
            for chunk in fname_data.chunks(2) {
                if name_chars.len() >= name_length {
                    break;
                }
                let ch = u16::from_le_bytes([chunk[0], chunk.get(1).copied().unwrap_or(0)]);
                if ch == 0 {
                    break;
                }
                name_chars.push(ch);
            }
            fname_offset += 32;
        }

        let name = String::from_utf16_lossy(&name_chars);
        if name.is_empty() {
            return None;
        }

        Some(ExFatFileEntry {
            name,
            is_directory,
            size: if is_directory { 0 } else { data_length },
            start_cluster,
            no_fat_chain,
            attributes,
            created,
            modified,
            accessed,
        })
    }

    /// Parse a DOS timestamp (4 bytes) to Unix timestamp (seconds since epoch)
    fn parse_dos_timestamp(data: &[u8]) -> Option<u64> {
        if data.len() < 4 {
            return None;
        }
        let time = u16::from_le_bytes([data[0], data[1]]);
        let date = u16::from_le_bytes([data[2], data[3]]);

        if date == 0 && time == 0 {
            return None;
        }

        let seconds = ((time & 0x1F) * 2) as u64;
        let minutes = ((time >> 5) & 0x3F) as u64;
        let hours = ((time >> 11) & 0x1F) as u64;
        let day = (date & 0x1F) as u64;
        let month = ((date >> 5) & 0x0F) as u64;
        let year = ((date >> 9) & 0x7F) as u64 + 1980;

        // Approximate Unix timestamp (not accounting for leap seconds)
        let days_since_epoch = (year - 1970) * 365
            + (year - 1969) / 4  // leap years
            + match month {
                1 => 0, 2 => 31, 3 => 59, 4 => 90, 5 => 120, 6 => 151,
                7 => 181, 8 => 212, 9 => 243, 10 => 273, 11 => 304, 12 => 334,
                _ => 0,
            }
            + day - 1;

        Some(days_since_epoch * 86400 + hours * 3600 + minutes * 60 + seconds)
    }

    /// Navigate to a directory and list its entries
    fn list_directory(&self, path: &str) -> Result<Vec<ExFatFileEntry>, VfsError> {
        let normalized = normalize_path(path);
        
        if normalized == "/" {
            // Read root directory
            let root_entry = ExFatFileEntry {
                name: String::new(),
                is_directory: true,
                size: 0,
                start_cluster: self.boot.root_directory_cluster,
                no_fat_chain: false, // Root dir typically uses FAT chain
                attributes: ATTR_DIRECTORY,
                created: None,
                modified: None,
                accessed: None,
            };
            let data = self.read_data(&root_entry)?;
            return Ok(self.parse_directory_entries(&data));
        }

        // Navigate path components
        let components: Vec<&str> = normalized
            .trim_start_matches('/')
            .split('/')
            .filter(|c| !c.is_empty())
            .collect();

        let mut current_cluster = self.boot.root_directory_cluster;
        let mut current_no_fat_chain = false;

        for (i, component) in components.iter().enumerate() {
            let dir_entry = ExFatFileEntry {
                name: String::new(),
                is_directory: true,
                size: 0,
                start_cluster: current_cluster,
                no_fat_chain: current_no_fat_chain,
                attributes: ATTR_DIRECTORY,
                created: None,
                modified: None,
                accessed: None,
            };
            let data = self.read_data(&dir_entry)?;
            let entries = self.parse_directory_entries(&data);

            let found = entries.iter().find(|e| {
                e.name.eq_ignore_ascii_case(component)
            });

            match found {
                Some(entry) => {
                    if i == components.len() - 1 {
                        // This is the target — list its contents
                        if !entry.is_directory {
                            return Err(VfsError::NotADirectory(normalized));
                        }
                        let dir_data = self.read_data(entry)?;
                        return Ok(self.parse_directory_entries(&dir_data));
                    } else {
                        // Intermediate directory — descend
                        if !entry.is_directory {
                            return Err(VfsError::NotADirectory(format!(
                                "/{}", components[..=i].join("/")
                            )));
                        }
                        current_cluster = entry.start_cluster;
                        current_no_fat_chain = entry.no_fat_chain;
                    }
                }
                None => {
                    return Err(VfsError::NotFound(format!(
                        "/{}", components[..=i].join("/")
                    )));
                }
            }
        }

        Err(VfsError::Internal("Unexpected path navigation state".to_string()))
    }

    /// Find a specific file entry by path
    fn find_entry(&self, path: &str) -> Result<ExFatFileEntry, VfsError> {
        let normalized = normalize_path(path);

        if normalized == "/" {
            return Ok(ExFatFileEntry {
                name: String::new(),
                is_directory: true,
                size: 0,
                start_cluster: self.boot.root_directory_cluster,
                no_fat_chain: false,
                attributes: ATTR_DIRECTORY,
                created: None,
                modified: None,
                accessed: None,
            });
        }

        let components: Vec<&str> = normalized
            .trim_start_matches('/')
            .split('/')
            .filter(|c| !c.is_empty())
            .collect();

        let mut current_cluster = self.boot.root_directory_cluster;
        let mut current_no_fat_chain = false;

        for (i, component) in components.iter().enumerate() {
            let dir_entry = ExFatFileEntry {
                name: String::new(),
                is_directory: true,
                size: 0,
                start_cluster: current_cluster,
                no_fat_chain: current_no_fat_chain,
                attributes: ATTR_DIRECTORY,
                created: None,
                modified: None,
                accessed: None,
            };
            let data = self.read_data(&dir_entry)?;
            let entries = self.parse_directory_entries(&data);

            let found = entries.iter().find(|e| {
                e.name.eq_ignore_ascii_case(component)
            });

            match found {
                Some(entry) => {
                    if i == components.len() - 1 {
                        return Ok(entry.clone());
                    } else if !entry.is_directory {
                        return Err(VfsError::NotADirectory(format!(
                            "/{}", components[..=i].join("/")
                        )));
                    }
                    current_cluster = entry.start_cluster;
                    current_no_fat_chain = entry.no_fat_chain;
                }
                None => {
                    return Err(VfsError::NotFound(format!(
                        "/{}", components[..=i].join("/")
                    )));
                }
            }
        }

        Err(VfsError::NotFound(normalized))
    }
}

/// Normalize a VFS path
fn normalize_path(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }
    let mut result = String::with_capacity(path.len() + 1);
    if !path.starts_with('/') {
        result.push('/');
    }
    result.push_str(path);
    // Remove trailing slash
    while result.len() > 1 && result.ends_with('/') {
        result.pop();
    }
    result
}

// =============================================================================
// FilesystemDriver Implementation
// =============================================================================

impl FilesystemDriver for ExFatDriver {
    fn info(&self) -> &FilesystemInfo {
        &self.info
    }

    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let entry = self.find_entry(path)?;
        Ok(FileAttr {
            size: entry.size,
            is_directory: entry.is_directory,
            permissions: if entry.is_directory { 0o555 } else {
                if entry.attributes & ATTR_READ_ONLY != 0 { 0o444 } else { 0o644 }
            },
            nlink: if entry.is_directory { 2 } else { 1 },
            inode: entry.start_cluster as u64,
            modified: entry.modified.map(|t| (t * 1_000_000_000) as i64),
            created: entry.created.map(|t| (t * 1_000_000_000) as i64),
            accessed: entry.accessed.map(|t| (t * 1_000_000_000) as i64),
            ..Default::default()
        })
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let entries = self.list_directory(path)?;
        Ok(entries.iter().map(|e| DirEntry {
            name: e.name.clone(),
            is_directory: e.is_directory,
            inode: e.start_cluster as u64,
            file_type: if e.is_directory { 4 } else { 8 },
        }).collect())
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let entry = self.find_entry(path)?;
        if entry.is_directory {
            return Err(VfsError::NotAFile(path.to_string()));
        }

        let data = self.read_data(&entry)?;
        let start = offset as usize;
        if start >= data.len() {
            return Ok(Vec::new());
        }
        let end = (start + size).min(data.len());
        Ok(data[start..end].to_vec())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Boot Sector Parsing ====================

    fn create_exfat_boot_sector() -> Vec<u8> {
        let mut data = vec![0u8; 512];
        // Jump boot code
        data[0] = 0xEB;
        data[1] = 0x76;
        data[2] = 0x90;
        // "EXFAT   " signature at offset 3
        data[3..11].copy_from_slice(EXFAT_SIGNATURE);
        // BytesPerSectorShift = 9 (512 bytes)
        data[108] = 9;
        // SectorsPerClusterShift = 3 (8 sectors = 4096 bytes per cluster)
        data[109] = 3;
        // NumberOfFats = 1
        data[110] = 1;
        // FatOffset = 24 sectors
        data[80..84].copy_from_slice(&24u32.to_le_bytes());
        // FatLength = 8 sectors
        data[84..88].copy_from_slice(&8u32.to_le_bytes());
        // ClusterHeapOffset = 32 sectors
        data[88..92].copy_from_slice(&32u32.to_le_bytes());
        // ClusterCount = 1000
        data[92..96].copy_from_slice(&1000u32.to_le_bytes());
        // RootDirectoryCluster = 2
        data[96..100].copy_from_slice(&2u32.to_le_bytes());
        // VolumeSerialNumber
        data[100..104].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
        // Boot signature
        data[510] = 0x55;
        data[511] = 0xAA;
        data
    }

    #[test]
    fn test_boot_sector_parse_valid() {
        let data = create_exfat_boot_sector();
        let boot = ExFatBootSector::parse(&data).unwrap();
        assert_eq!(boot.bytes_per_sector, 512);
        assert_eq!(boot.bytes_per_cluster, 4096);
        assert_eq!(boot.cluster_count, 1000);
        assert_eq!(boot.root_directory_cluster, 2);
        assert_eq!(boot.fat_offset, 24);
        assert_eq!(boot.cluster_heap_offset, 32);
        assert_eq!(boot.number_of_fats, 1);
        assert_eq!(boot.volume_serial, 0xDEADBEEF);
    }

    #[test]
    fn test_boot_sector_parse_too_small() {
        let data = vec![0u8; 100];
        let result = ExFatBootSector::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_boot_sector_parse_wrong_signature() {
        let mut data = create_exfat_boot_sector();
        data[3..11].copy_from_slice(b"NTFS    ");
        let result = ExFatBootSector::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_boot_sector_parse_bad_boot_sig() {
        let mut data = create_exfat_boot_sector();
        data[510] = 0x00;
        data[511] = 0x00;
        let result = ExFatBootSector::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_boot_sector_parse_invalid_sector_shift() {
        let mut data = create_exfat_boot_sector();
        data[108] = 8; // Invalid: must be 9-12
        let result = ExFatBootSector::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cluster_offset_calculation() {
        let data = create_exfat_boot_sector();
        let boot = ExFatBootSector::parse(&data).unwrap();
        // Cluster 2 is the first data cluster
        // heap_offset = 32 sectors * 512 bytes = 16384
        // cluster 2 offset = 16384 + (2-2) * 4096 = 16384
        assert_eq!(boot.cluster_offset(2), 16384);
        // cluster 3 offset = 16384 + (3-2) * 4096 = 20480
        assert_eq!(boot.cluster_offset(3), 20480);
    }

    // ==================== Volume Label ====================

    #[test]
    fn test_find_volume_label_present() {
        let mut data = vec![0u8; 512];
        // Volume Label entry (0x83)
        data[0] = ENTRY_TYPE_VOLUME_LABEL;
        data[1] = 4; // 4 characters
        // "TEST" in UTF-16LE
        data[2] = b'T';
        data[3] = 0;
        data[4] = b'E';
        data[5] = 0;
        data[6] = b'S';
        data[7] = 0;
        data[8] = b'T';
        data[9] = 0;

        let label = ExFatDriver::find_volume_label(&data);
        assert_eq!(label, Some("TEST".to_string()));
    }

    #[test]
    fn test_find_volume_label_absent() {
        let data = vec![0u8; 512]; // All zeros = end of entries
        let label = ExFatDriver::find_volume_label(&data);
        assert_eq!(label, None);
    }

    // ==================== DOS Timestamp Parsing ====================

    #[test]
    fn test_parse_dos_timestamp_zero() {
        let data = [0u8; 4];
        assert_eq!(ExFatDriver::parse_dos_timestamp(&data), None);
    }

    #[test]
    fn test_parse_dos_timestamp_valid() {
        // 2024-01-15 10:30:00
        // Date: day=15, month=1, year=2024-1980=44 → (44<<9)|(1<<5)|15 = 0x5835
        // Time: seconds=0/2=0, minutes=30, hours=10 → (10<<11)|(30<<5)|0 = 0x53C0
        let time_bytes = 0x53C0u16.to_le_bytes();
        let date_bytes = 0x5835u16.to_le_bytes();
        let data = [time_bytes[0], time_bytes[1], date_bytes[0], date_bytes[1]];
        let result = ExFatDriver::parse_dos_timestamp(&data);
        assert!(result.is_some());
        // Just verify it produces a positive number
        assert!(result.unwrap() > 0);
    }

    // ==================== Path Normalization ====================

    #[test]
    fn test_normalize_path_root() {
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path(""), "/");
    }

    #[test]
    fn test_normalize_path_no_leading_slash() {
        assert_eq!(normalize_path("dir/file.txt"), "/dir/file.txt");
    }

    #[test]
    fn test_normalize_path_trailing_slash() {
        assert_eq!(normalize_path("/dir/"), "/dir");
    }

    #[test]
    fn test_normalize_path_already_normalized() {
        assert_eq!(normalize_path("/dir/file.txt"), "/dir/file.txt");
    }

    // ==================== Directory Entry Constants ====================

    #[test]
    fn test_entry_type_constants() {
        assert_eq!(ENTRY_TYPE_END, 0x00);
        assert_eq!(ENTRY_TYPE_FILE, 0x85);
        assert_eq!(ENTRY_TYPE_STREAM, 0xC0);
        assert_eq!(ENTRY_TYPE_FILENAME, 0xC1);
        assert_eq!(ENTRY_TYPE_VOLUME_LABEL, 0x83);
    }

    #[test]
    fn test_attribute_flags() {
        assert_eq!(ATTR_DIRECTORY, 0x10);
        assert_eq!(ATTR_READ_ONLY, 0x01);
        assert_eq!(ATTR_HIDDEN, 0x02);
        assert_eq!(ATTR_SYSTEM, 0x04);
        assert_eq!(ATTR_ARCHIVE, 0x20);
    }
}
