// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Disk image format header parsers (Raw, VMDK, VHDx, QCOW2)

use crate::common::format_size;
use crate::containers::ContainerError;
use crate::viewer::types::{HeaderRegion, MetadataField, ParsedMetadata};

/// Parse raw disk image header and extract metadata
/// 
/// Raw images have no container header - detect partition table:
/// - MBR: Boot signature 0x55 0xAA at offset 510
/// - GPT: "EFI PART" at offset 512 (LBA 1)
pub fn parse_raw_header(header: &[u8], extension: &str, file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    fields.push(MetadataField::new("Format", "Raw Disk Image", "Format"));
    
    // Check for MBR signature (0x55 0xAA at offset 510-511)
    let has_mbr = header.len() >= 512 && header[510] == 0x55 && header[511] == 0xAA;
    
    // Check for protective MBR (GPT)
    let has_protective_mbr = has_mbr && header.len() >= 450 && header[450] == 0xEE;
    
    if has_mbr {
        // MBR boot code region (0x00-0x1BE)
        regions.push(HeaderRegion::new(
            0, 446,
            "Boot Code",
            "region-code",
            "MBR bootstrap code (446 bytes)",
        ));
        
        // Partition table region (0x1BE-0x1FE)
        regions.push(HeaderRegion::new(
            446, 510,
            "Partition Table",
            "region-data",
            "Four 16-byte partition entries",
        ));
        
        // Boot signature
        regions.push(HeaderRegion::new(
            510, 512,
            "Boot Signature",
            "region-signature",
            "MBR signature (55 AA)",
        ));
        
        fields.push(MetadataField::new(
            "Partition Table",
            if has_protective_mbr { "GPT (with Protective MBR)" } else { "MBR" },
            "Structure",
        ).with_offset(510));
        
        // Parse partition entries
        let mut partition_count = 0;
        for i in 0..4 {
            let entry_offset = 446 + (i * 16);
            if header.len() >= entry_offset + 16 {
                let part_type = header[entry_offset + 4];
                if part_type != 0 {
                    partition_count += 1;
                    
                    let lba_start = u32::from_le_bytes([
                        header[entry_offset + 8],
                        header[entry_offset + 9],
                        header[entry_offset + 10],
                        header[entry_offset + 11],
                    ]);
                    let lba_count = u32::from_le_bytes([
                        header[entry_offset + 12],
                        header[entry_offset + 13],
                        header[entry_offset + 14],
                        header[entry_offset + 15],
                    ]);
                    
                    let type_name = match part_type {
                        0x01 => "FAT12",
                        0x04 | 0x06 => "FAT16",
                        0x07 => "NTFS/exFAT",
                        0x0B | 0x0C => "FAT32",
                        0x82 => "Linux Swap",
                        0x83 => "Linux",
                        0x8E => "Linux LVM",
                        0xEE => "GPT Protective",
                        0xEF => "EFI System",
                        0xFD => "Linux RAID",
                        _ => "Unknown",
                    };
                    
                    fields.push(MetadataField::new(
                        format!("Partition {}", i + 1),
                        format!("{} (0x{:02X}), Start: LBA {}, Size: {} sectors", 
                            type_name, part_type, lba_start, lba_count),
                        "Partitions",
                    ).with_offset(entry_offset as u64));
                }
            }
        }
        
        fields.push(MetadataField::new(
            "Active Partitions",
            partition_count.to_string(),
            "Structure",
        ));
    } else {
        fields.push(MetadataField::new(
            "Partition Table",
            "None detected (raw data)",
            "Structure",
        ));
    }
    
    // Calculate sector count (assuming 512-byte sectors)
    let sector_count = file_size / 512;
    fields.push(MetadataField::new(
        "Sector Count",
        format!("{} (assuming 512-byte sectors)", sector_count),
        "Structure",
    ));
    
    fields.push(MetadataField::new("File Size", format_size(file_size), "General"));
    
    // Multi-segment detection
    if extension.chars().all(|c| c.is_ascii_digit()) {
        fields.push(MetadataField::new(
            "Segment Extension",
            format!(".{} (likely part of multi-segment image)", extension),
            "Format",
        ));
    }
    
    Ok(ParsedMetadata {
        format: "Raw Disk Image".to_string(),
        version: None,
        fields,
        regions,
    })
}

/// Parse VMDK sparse header
pub fn parse_vmdk_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    regions.push(HeaderRegion::new(
        0, 4,
        "Signature",
        "region-signature",
        "VMDK sparse header magic (KDMV)",
    ));
    
    fields.push(MetadataField::new(
        "Format",
        "VMDK (VMware Virtual Disk)",
        "Format",
    ).with_offset(0));
    
    // Version (offset 4-8)
    if header.len() >= 8 {
        let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        
        regions.push(HeaderRegion::new(
            4, 8,
            "Version",
            "region-version",
            "VMDK format version",
        ));
        
        fields.push(MetadataField::new(
            "Version",
            version.to_string(),
            "Format",
        ).with_offset(4));
    }
    
    // Flags (offset 8-12)
    if header.len() >= 12 {
        let flags = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        
        regions.push(HeaderRegion::new(
            8, 12,
            "Flags",
            "region-flags",
            "VMDK flags",
        ));
        
        let mut flag_names = vec![];
        if flags & 0x01 != 0 { flag_names.push("NewLineTest"); }
        if flags & 0x02 != 0 { flag_names.push("UseSecondaryGT"); }
        if flags & 0x10000 != 0 { flag_names.push("ZeroedGrainGT"); }
        if flags & 0x20000 != 0 { flag_names.push("CompressedGrains"); }
        if flags & 0x40000 != 0 { flag_names.push("HasMarkers"); }
        
        fields.push(MetadataField::new(
            "Flags",
            if flag_names.is_empty() { format!("0x{:08X}", flags) } else { flag_names.join(", ") },
            "Format",
        ).with_offset(8));
    }
    
    // Capacity in sectors (offset 12-20)
    if header.len() >= 20 {
        let capacity = u64::from_le_bytes([
            header[12], header[13], header[14], header[15],
            header[16], header[17], header[18], header[19],
        ]);
        
        regions.push(HeaderRegion::new(
            12, 20,
            "Capacity",
            "region-size",
            "Virtual disk capacity in sectors",
        ));
        
        let capacity_bytes = capacity * 512;
        fields.push(MetadataField::new(
            "Virtual Capacity",
            format_size(capacity_bytes),
            "Structure",
        ).with_offset(12));
    }
    
    fields.push(MetadataField::new("File Size", format_size(file_size), "General"));
    
    Ok(ParsedMetadata {
        format: "VMDK".to_string(),
        version: if header.len() >= 8 { 
            Some(format!("{}", u32::from_le_bytes([header[4], header[5], header[6], header[7]])))
        } else { 
            None 
        },
        fields,
        regions,
    })
}

/// Parse VHDx header
pub fn parse_vhdx_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    regions.push(HeaderRegion::new(
        0, 8,
        "Signature",
        "region-signature",
        "VHDx file type identifier",
    ));
    
    fields.push(MetadataField::new(
        "Format",
        "VHDx (Microsoft Virtual Hard Disk v2)",
        "Format",
    ).with_offset(0));
    
    // Creator field (at offset 8, 512 bytes UTF-16)
    if header.len() >= 520 {
        let creator_bytes = &header[8..520];
        let creator: String = creator_bytes
            .chunks(2)
            .filter_map(|c| {
                if c.len() == 2 {
                    let val = u16::from_le_bytes([c[0], c[1]]);
                    if val > 0 && val < 128 {
                        Some(val as u8 as char)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .take_while(|&c| c != '\0')
            .collect();
        
        if !creator.is_empty() {
            regions.push(HeaderRegion::new(
                8, 520,
                "Creator",
                "region-metadata",
                "Application that created this VHDx",
            ));
            
            fields.push(MetadataField::new(
                "Creator",
                creator,
                "Format",
            ).with_offset(8));
        }
    }
    
    fields.push(MetadataField::new("File Size", format_size(file_size), "General"));
    
    Ok(ParsedMetadata {
        format: "VHDx".to_string(),
        version: Some("2.0".to_string()),
        fields,
        regions,
    })
}

/// Parse QCOW2 header
pub fn parse_qcow2_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    regions.push(HeaderRegion::new(
        0, 4,
        "Signature",
        "region-signature",
        "QCOW2 magic (QFI\\xfb)",
    ));
    
    fields.push(MetadataField::new(
        "Format",
        "QCOW2 (QEMU Copy-On-Write v2)",
        "Format",
    ).with_offset(0));
    
    // Version (offset 4-8, big-endian)
    if header.len() >= 8 {
        let version = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
        
        regions.push(HeaderRegion::new(
            4, 8,
            "Version",
            "region-version",
            "QCOW format version",
        ));
        
        fields.push(MetadataField::new(
            "Version",
            version.to_string(),
            "Format",
        ).with_offset(4));
    }
    
    // Backing file offset (offset 8-16, big-endian)
    if header.len() >= 16 {
        let backing_offset = u64::from_be_bytes([
            header[8], header[9], header[10], header[11],
            header[12], header[13], header[14], header[15],
        ]);
        
        if backing_offset > 0 {
            regions.push(HeaderRegion::new(
                8, 16,
                "Backing File Offset",
                "region-offset",
                "Offset to backing file name",
            ));
            
            fields.push(MetadataField::new(
                "Backing File Offset",
                format!("0x{:X}", backing_offset),
                "Structure",
            ).with_offset(8));
        }
    }
    
    // Cluster bits (offset 20-24, big-endian)
    if header.len() >= 24 {
        let cluster_bits = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
        let cluster_size = 1u64 << cluster_bits;
        
        regions.push(HeaderRegion::new(
            20, 24,
            "Cluster Bits",
            "region-size",
            "Log2 of cluster size",
        ));
        
        fields.push(MetadataField::new(
            "Cluster Size",
            format_size(cluster_size),
            "Structure",
        ).with_offset(20));
    }
    
    // Virtual size (offset 24-32, big-endian)
    if header.len() >= 32 {
        let virtual_size = u64::from_be_bytes([
            header[24], header[25], header[26], header[27],
            header[28], header[29], header[30], header[31],
        ]);
        
        regions.push(HeaderRegion::new(
            24, 32,
            "Virtual Size",
            "region-size",
            "Virtual disk size in bytes",
        ));
        
        fields.push(MetadataField::new(
            "Virtual Size",
            format_size(virtual_size),
            "Structure",
        ).with_offset(24));
    }
    
    fields.push(MetadataField::new("File Size", format_size(file_size), "General"));
    
    Ok(ParsedMetadata {
        format: "QCOW2".to_string(),
        version: if header.len() >= 8 { 
            Some(format!("{}", u32::from_be_bytes([header[4], header[5], header[6], header[7]])))
        } else { 
            None 
        },
        fields,
        regions,
    })
}
