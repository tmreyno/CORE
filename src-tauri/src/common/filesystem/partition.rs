// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Partition Table Parsing
//!
//! Detects and parses MBR and GPT partition tables from disk images.

use crate::common::vfs::VfsError;
use super::traits::{BlockDevice, FilesystemType};

// =============================================================================
// Partition Table Types
// =============================================================================

/// Partition table type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableType {
    /// Master Boot Record (legacy BIOS)
    Mbr,
    /// GUID Partition Table (UEFI)
    Gpt,
    /// No partition table (whole disk is one filesystem)
    None,
    /// Unknown/unrecognized
    Unknown,
}

/// Partition table with entries
#[derive(Debug, Clone)]
pub struct PartitionTable {
    /// Table type
    pub table_type: PartitionTableType,
    /// Disk size in bytes
    pub disk_size: u64,
    /// Partition entries
    pub partitions: Vec<PartitionEntry>,
}

/// Single partition entry
#[derive(Debug, Clone)]
pub struct PartitionEntry {
    /// Partition number (1-based)
    pub number: u32,
    /// Start offset in bytes
    pub start_offset: u64,
    /// Size in bytes
    pub size: u64,
    /// Partition type (MBR type byte or GPT type GUID)
    pub partition_type: String,
    /// Partition name (GPT only)
    pub name: Option<String>,
    /// Whether this is the active/bootable partition
    pub bootable: bool,
    /// Detected filesystem type
    pub filesystem_type: Option<FilesystemType>,
}

// =============================================================================
// MBR Constants
// =============================================================================

const MBR_SIGNATURE: [u8; 2] = [0x55, 0xAA];
const MBR_PARTITION_TABLE_OFFSET: usize = 446;
const MBR_PARTITION_ENTRY_SIZE: usize = 16;
const MBR_PARTITION_COUNT: usize = 4;

// MBR partition type codes
const MBR_TYPE_EMPTY: u8 = 0x00;
const MBR_TYPE_FAT12: u8 = 0x01;
const MBR_TYPE_FAT16_SMALL: u8 = 0x04;
const MBR_TYPE_EXTENDED: u8 = 0x05;
const MBR_TYPE_FAT16: u8 = 0x06;
const MBR_TYPE_NTFS: u8 = 0x07;
const MBR_TYPE_FAT32: u8 = 0x0B;
const MBR_TYPE_FAT32_LBA: u8 = 0x0C;
const MBR_TYPE_FAT16_LBA: u8 = 0x0E;
const MBR_TYPE_EXTENDED_LBA: u8 = 0x0F;
const MBR_TYPE_LINUX: u8 = 0x83;
const MBR_TYPE_LINUX_LVM: u8 = 0x8E;
const MBR_TYPE_GPT_PROTECTIVE: u8 = 0xEE;

// =============================================================================
// GPT Constants
// =============================================================================

const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";
const GPT_HEADER_OFFSET: u64 = 512; // LBA 1
const SECTOR_SIZE: u64 = 512;

// =============================================================================
// Detection Functions
// =============================================================================

/// Detect partition table type and parse entries
pub fn detect_partition_table(device: &dyn BlockDevice) -> Result<PartitionTable, VfsError> {
    let disk_size = device.size();
    
    // Read first sector (MBR)
    let mut mbr = vec![0u8; 512];
    let bytes_read = device.read_at(0, &mut mbr)
        .map_err(|e| VfsError::IoError(e.to_string()))?;
    
    if bytes_read < 512 {
        return Ok(PartitionTable {
            table_type: PartitionTableType::Unknown,
            disk_size,
            partitions: Vec::new(),
        });
    }
    
    // Check MBR signature
    if mbr[510..512] != MBR_SIGNATURE {
        // No valid MBR - might be whole-disk filesystem
        return Ok(PartitionTable {
            table_type: PartitionTableType::None,
            disk_size,
            partitions: vec![PartitionEntry {
                number: 1,
                start_offset: 0,
                size: disk_size,
                partition_type: "Whole Disk".to_string(),
                name: None,
                bootable: false,
                filesystem_type: None,
            }],
        });
    }
    
    // Check if this is a GPT protective MBR
    let mbr_type = mbr[MBR_PARTITION_TABLE_OFFSET + 4];
    if mbr_type == MBR_TYPE_GPT_PROTECTIVE {
        return parse_gpt(device, disk_size);
    }
    
    // Parse MBR partition table
    parse_mbr(device, &mbr, disk_size)
}

/// Parse MBR partition table
fn parse_mbr(device: &dyn BlockDevice, mbr: &[u8], disk_size: u64) -> Result<PartitionTable, VfsError> {
    let mut partitions = Vec::new();
    
    for i in 0..MBR_PARTITION_COUNT {
        let offset = MBR_PARTITION_TABLE_OFFSET + (i * MBR_PARTITION_ENTRY_SIZE);
        let entry = &mbr[offset..offset + MBR_PARTITION_ENTRY_SIZE];
        
        let status = entry[0];
        let partition_type = entry[4];
        let start_lba = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]);
        let sector_count = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]);
        
        // Skip empty entries
        if partition_type == MBR_TYPE_EMPTY || sector_count == 0 {
            continue;
        }
        
        let start_offset = start_lba as u64 * SECTOR_SIZE;
        let size = sector_count as u64 * SECTOR_SIZE;
        
        // Detect filesystem type by reading partition boot sector
        let fs_type = detect_partition_filesystem(device, start_offset);
        
        partitions.push(PartitionEntry {
            number: (i + 1) as u32,
            start_offset,
            size,
            partition_type: mbr_type_to_string(partition_type),
            name: None,
            bootable: status == 0x80,
            filesystem_type: fs_type,
        });
    }
    
    Ok(PartitionTable {
        table_type: PartitionTableType::Mbr,
        disk_size,
        partitions,
    })
}

/// Parse GPT partition table
fn parse_gpt(device: &dyn BlockDevice, disk_size: u64) -> Result<PartitionTable, VfsError> {
    // Read GPT header (LBA 1)
    let mut header = vec![0u8; 512];
    device.read_at(GPT_HEADER_OFFSET, &mut header)
        .map_err(|e| VfsError::IoError(e.to_string()))?;
    
    // Verify GPT signature
    if &header[0..8] != GPT_SIGNATURE {
        return Err(VfsError::Internal("Invalid GPT signature".to_string()));
    }
    
    // Parse GPT header fields
    let partition_entry_lba = u64::from_le_bytes([
        header[72], header[73], header[74], header[75],
        header[76], header[77], header[78], header[79],
    ]);
    let num_partition_entries = u32::from_le_bytes([
        header[80], header[81], header[82], header[83],
    ]);
    let partition_entry_size = u32::from_le_bytes([
        header[84], header[85], header[86], header[87],
    ]);
    
    // Read partition entries
    let entries_size = (num_partition_entries * partition_entry_size) as usize;
    let mut entries_buf = vec![0u8; entries_size];
    device.read_at(partition_entry_lba * SECTOR_SIZE, &mut entries_buf)
        .map_err(|e| VfsError::IoError(e.to_string()))?;
    
    let mut partitions = Vec::new();
    let mut partition_num = 1u32;
    
    for i in 0..num_partition_entries as usize {
        let offset = i * partition_entry_size as usize;
        let entry = &entries_buf[offset..offset + partition_entry_size as usize];
        
        // Check if entry is used (type GUID is not all zeros)
        let type_guid = &entry[0..16];
        if type_guid.iter().all(|&b| b == 0) {
            continue;
        }
        
        let start_lba = u64::from_le_bytes([
            entry[32], entry[33], entry[34], entry[35],
            entry[36], entry[37], entry[38], entry[39],
        ]);
        let end_lba = u64::from_le_bytes([
            entry[40], entry[41], entry[42], entry[43],
            entry[44], entry[45], entry[46], entry[47],
        ]);
        
        // Parse partition name (UTF-16LE, up to 72 bytes / 36 chars)
        let name_bytes = &entry[56..128];
        let name = parse_utf16le_name(name_bytes);
        
        let start_offset = start_lba * SECTOR_SIZE;
        let size = (end_lba - start_lba + 1) * SECTOR_SIZE;
        
        // Detect filesystem type
        let fs_type = detect_partition_filesystem(device, start_offset);
        
        partitions.push(PartitionEntry {
            number: partition_num,
            start_offset,
            size,
            partition_type: gpt_type_guid_to_string(type_guid),
            name: if name.is_empty() { None } else { Some(name) },
            bootable: false, // GPT doesn't have bootable flag per-partition
            filesystem_type: fs_type,
        });
        
        partition_num += 1;
    }
    
    Ok(PartitionTable {
        table_type: PartitionTableType::Gpt,
        disk_size,
        partitions,
    })
}

/// Detect filesystem type on a partition
fn detect_partition_filesystem(device: &dyn BlockDevice, offset: u64) -> Option<FilesystemType> {
    let mut buf = vec![0u8; 512];
    if device.read_at(offset, &mut buf).is_err() {
        return None;
    }
    
    // NTFS
    if buf.len() >= 11 && &buf[3..11] == b"NTFS    " {
        return Some(FilesystemType::Ntfs);
    }
    
    // FAT32
    if buf.len() >= 90 && &buf[82..90] == b"FAT32   " {
        return Some(FilesystemType::Fat32);
    }
    
    // FAT16
    if buf.len() >= 62 && &buf[54..62] == b"FAT16   " {
        return Some(FilesystemType::Fat16);
    }
    
    // FAT12
    if buf.len() >= 62 && &buf[54..62] == b"FAT12   " {
        return Some(FilesystemType::Fat12);
    }
    
    // exFAT
    if buf.len() >= 11 && &buf[3..11] == b"EXFAT   " {
        return Some(FilesystemType::ExFat);
    }
    
    None
}

/// Get filesystem type for a partition entry
pub fn get_partition_filesystem_type(entry: &PartitionEntry) -> FilesystemType {
    entry.filesystem_type.unwrap_or(FilesystemType::Unknown)
}

// =============================================================================
// Helper Functions
// =============================================================================

fn mbr_type_to_string(type_byte: u8) -> String {
    match type_byte {
        MBR_TYPE_FAT12 => "FAT12".to_string(),
        MBR_TYPE_FAT16_SMALL | MBR_TYPE_FAT16 | MBR_TYPE_FAT16_LBA => "FAT16".to_string(),
        MBR_TYPE_FAT32 | MBR_TYPE_FAT32_LBA => "FAT32".to_string(),
        MBR_TYPE_NTFS => "NTFS/exFAT".to_string(),
        MBR_TYPE_EXTENDED | MBR_TYPE_EXTENDED_LBA => "Extended".to_string(),
        MBR_TYPE_LINUX => "Linux".to_string(),
        MBR_TYPE_LINUX_LVM => "Linux LVM".to_string(),
        MBR_TYPE_GPT_PROTECTIVE => "GPT Protective".to_string(),
        _ => format!("0x{:02X}", type_byte),
    }
}

fn gpt_type_guid_to_string(guid: &[u8]) -> String {
    // Common GPT type GUIDs
    // Microsoft Basic Data: EBD0A0A2-B9E5-4433-87C0-68B6B72699C7
    // EFI System: C12A7328-F81F-11D2-BA4B-00A0C93EC93B
    // Linux filesystem: 0FC63DAF-8483-4772-8E79-3D69D8477DE4
    
    // For now, just format as GUID
    if guid.len() >= 16 {
        format!(
            "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            guid[3], guid[2], guid[1], guid[0],
            guid[5], guid[4],
            guid[7], guid[6],
            guid[8], guid[9],
            guid[10], guid[11], guid[12], guid[13], guid[14], guid[15]
        )
    } else {
        "Invalid GUID".to_string()
    }
}

fn parse_utf16le_name(bytes: &[u8]) -> String {
    let mut chars = Vec::new();
    for chunk in bytes.chunks(2) {
        if chunk.len() < 2 {
            break;
        }
        let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if code_unit == 0 {
            break;
        }
        if let Some(c) = char::from_u32(code_unit as u32) {
            chars.push(c);
        }
    }
    chars.into_iter().collect()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mbr_type_to_string() {
        assert_eq!(mbr_type_to_string(0x07), "NTFS/exFAT");
        assert_eq!(mbr_type_to_string(0x0B), "FAT32");
        assert_eq!(mbr_type_to_string(0x83), "Linux");
    }

    #[test]
    fn test_parse_utf16le_name() {
        // "Test" in UTF-16LE
        let bytes = [0x54, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x00, 0x00];
        assert_eq!(parse_utf16le_name(&bytes), "Test");
    }
}
