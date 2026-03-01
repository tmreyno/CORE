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
pub fn parse_raw_header(
    header: &[u8],
    extension: &str,
    file_size: u64,
) -> Result<ParsedMetadata, ContainerError> {
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
            0,
            446,
            "Boot Code",
            "region-code",
            "MBR bootstrap code (446 bytes)",
        ));

        // Partition table region (0x1BE-0x1FE)
        regions.push(HeaderRegion::new(
            446,
            510,
            "Partition Table",
            "region-data",
            "Four 16-byte partition entries",
        ));

        // Boot signature
        regions.push(HeaderRegion::new(
            510,
            512,
            "Boot Signature",
            "region-signature",
            "MBR signature (55 AA)",
        ));

        fields.push(
            MetadataField::new(
                "Partition Table",
                if has_protective_mbr {
                    "GPT (with Protective MBR)"
                } else {
                    "MBR"
                },
                "Structure",
            )
            .with_offset(510),
        );

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

                    fields.push(
                        MetadataField::new(
                            format!("Partition {}", i + 1),
                            format!(
                                "{} (0x{:02X}), Start: LBA {}, Size: {} sectors",
                                type_name, part_type, lba_start, lba_count
                            ),
                            "Partitions",
                        )
                        .with_offset(entry_offset as u64),
                    );
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

    fields.push(MetadataField::new(
        "File Size",
        format_size(file_size),
        "General",
    ));

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
        0,
        4,
        "Signature",
        "region-signature",
        "VMDK sparse header magic (KDMV)",
    ));

    fields
        .push(MetadataField::new("Format", "VMDK (VMware Virtual Disk)", "Format").with_offset(0));

    // Version (offset 4-8)
    if header.len() >= 8 {
        let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);

        regions.push(HeaderRegion::new(
            4,
            8,
            "Version",
            "region-version",
            "VMDK format version",
        ));

        fields.push(MetadataField::new("Version", version.to_string(), "Format").with_offset(4));
    }

    // Flags (offset 8-12)
    if header.len() >= 12 {
        let flags = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);

        regions.push(HeaderRegion::new(
            8,
            12,
            "Flags",
            "region-flags",
            "VMDK flags",
        ));

        let mut flag_names = vec![];
        if flags & 0x01 != 0 {
            flag_names.push("NewLineTest");
        }
        if flags & 0x02 != 0 {
            flag_names.push("UseSecondaryGT");
        }
        if flags & 0x10000 != 0 {
            flag_names.push("ZeroedGrainGT");
        }
        if flags & 0x20000 != 0 {
            flag_names.push("CompressedGrains");
        }
        if flags & 0x40000 != 0 {
            flag_names.push("HasMarkers");
        }

        fields.push(
            MetadataField::new(
                "Flags",
                if flag_names.is_empty() {
                    format!("0x{:08X}", flags)
                } else {
                    flag_names.join(", ")
                },
                "Format",
            )
            .with_offset(8),
        );
    }

    // Capacity in sectors (offset 12-20)
    if header.len() >= 20 {
        let capacity = u64::from_le_bytes([
            header[12], header[13], header[14], header[15], header[16], header[17], header[18],
            header[19],
        ]);

        regions.push(HeaderRegion::new(
            12,
            20,
            "Capacity",
            "region-size",
            "Virtual disk capacity in sectors",
        ));

        let capacity_bytes = capacity * 512;
        fields.push(
            MetadataField::new("Virtual Capacity", format_size(capacity_bytes), "Structure")
                .with_offset(12),
        );
    }

    fields.push(MetadataField::new(
        "File Size",
        format_size(file_size),
        "General",
    ));

    Ok(ParsedMetadata {
        format: "VMDK".to_string(),
        version: if header.len() >= 8 {
            Some(format!(
                "{}",
                u32::from_le_bytes([header[4], header[5], header[6], header[7]])
            ))
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
        0,
        8,
        "Signature",
        "region-signature",
        "VHDx file type identifier",
    ));

    fields.push(
        MetadataField::new("Format", "VHDx (Microsoft Virtual Hard Disk v2)", "Format")
            .with_offset(0),
    );

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
                8,
                520,
                "Creator",
                "region-metadata",
                "Application that created this VHDx",
            ));

            fields.push(MetadataField::new("Creator", creator, "Format").with_offset(8));
        }
    }

    fields.push(MetadataField::new(
        "File Size",
        format_size(file_size),
        "General",
    ));

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
        0,
        4,
        "Signature",
        "region-signature",
        "QCOW2 magic (QFI\\xfb)",
    ));

    fields.push(
        MetadataField::new("Format", "QCOW2 (QEMU Copy-On-Write v2)", "Format").with_offset(0),
    );

    // Version (offset 4-8, big-endian)
    if header.len() >= 8 {
        let version = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);

        regions.push(HeaderRegion::new(
            4,
            8,
            "Version",
            "region-version",
            "QCOW format version",
        ));

        fields.push(MetadataField::new("Version", version.to_string(), "Format").with_offset(4));
    }

    // Backing file offset (offset 8-16, big-endian)
    if header.len() >= 16 {
        let backing_offset = u64::from_be_bytes([
            header[8], header[9], header[10], header[11], header[12], header[13], header[14],
            header[15],
        ]);

        if backing_offset > 0 {
            regions.push(HeaderRegion::new(
                8,
                16,
                "Backing File Offset",
                "region-offset",
                "Offset to backing file name",
            ));

            fields.push(
                MetadataField::new(
                    "Backing File Offset",
                    format!("0x{:X}", backing_offset),
                    "Structure",
                )
                .with_offset(8),
            );
        }
    }

    // Cluster bits (offset 20-24, big-endian)
    if header.len() >= 24 {
        let cluster_bits = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
        let cluster_size = 1u64 << cluster_bits;

        regions.push(HeaderRegion::new(
            20,
            24,
            "Cluster Bits",
            "region-size",
            "Log2 of cluster size",
        ));

        fields.push(
            MetadataField::new("Cluster Size", format_size(cluster_size), "Structure")
                .with_offset(20),
        );
    }

    // Virtual size (offset 24-32, big-endian)
    if header.len() >= 32 {
        let virtual_size = u64::from_be_bytes([
            header[24], header[25], header[26], header[27], header[28], header[29], header[30],
            header[31],
        ]);

        regions.push(HeaderRegion::new(
            24,
            32,
            "Virtual Size",
            "region-size",
            "Virtual disk size in bytes",
        ));

        fields.push(
            MetadataField::new("Virtual Size", format_size(virtual_size), "Structure")
                .with_offset(24),
        );
    }

    fields.push(MetadataField::new(
        "File Size",
        format_size(file_size),
        "General",
    ));

    Ok(ParsedMetadata {
        format: "QCOW2".to_string(),
        version: if header.len() >= 8 {
            Some(format!(
                "{}",
                u32::from_be_bytes([header[4], header[5], header[6], header[7]])
            ))
        } else {
            None
        },
        fields,
        regions,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Raw disk image parser tests
    // =========================================================================

    /// Build a 512-byte MBR with boot signature
    fn make_mbr(partitions: &[(u8, u32, u32)]) -> Vec<u8> {
        let mut buf = vec![0u8; 512];
        // Boot signature at offset 510-511
        buf[510] = 0x55;
        buf[511] = 0xAA;
        // Partition entries at offset 446, 16 bytes each
        for (i, (ptype, lba_start, lba_count)) in partitions.iter().enumerate() {
            if i >= 4 {
                break;
            }
            let offset = 446 + i * 16;
            buf[offset + 4] = *ptype;
            buf[offset + 8..offset + 12].copy_from_slice(&lba_start.to_le_bytes());
            buf[offset + 12..offset + 16].copy_from_slice(&lba_count.to_le_bytes());
        }
        buf
    }

    #[test]
    fn raw_basic_parse() {
        let header = vec![0u8; 512];
        let result = parse_raw_header(&header, "dd", 1_048_576).unwrap();
        assert_eq!(result.format, "Raw Disk Image");
        assert!(result.version.is_none());
    }

    #[test]
    fn raw_mbr_detected() {
        let header = make_mbr(&[(0x07, 2048, 1_000_000)]);
        let result = parse_raw_header(&header, "dd", 512_000_000).unwrap();
        let pt = result
            .fields
            .iter()
            .find(|f| f.key == "Partition Table")
            .unwrap();
        assert_eq!(pt.value, "MBR");
    }

    #[test]
    fn raw_gpt_detected() {
        // GPT protective MBR: partition type 0xEE at offset 450
        let mut header = make_mbr(&[(0xEE, 1, 0xFFFFFFFF)]);
        // Need to set partition type at correct offset for GPT detection
        header[450] = 0xEE;
        let result = parse_raw_header(&header, "dd", 1_000_000_000).unwrap();
        let pt = result
            .fields
            .iter()
            .find(|f| f.key == "Partition Table")
            .unwrap();
        assert!(pt.value.contains("GPT"));
    }

    #[test]
    fn raw_no_partition_table() {
        let header = vec![0u8; 512]; // No boot signature
        let result = parse_raw_header(&header, "dd", 100).unwrap();
        let pt = result
            .fields
            .iter()
            .find(|f| f.key == "Partition Table")
            .unwrap();
        assert!(pt.value.contains("None"));
    }

    #[test]
    fn raw_partition_types() {
        let header = make_mbr(&[
            (0x07, 2048, 500_000),    // NTFS
            (0x83, 502_048, 500_000), // Linux
        ]);
        let result = parse_raw_header(&header, "dd", 1_000_000_000).unwrap();
        let p1 = result
            .fields
            .iter()
            .find(|f| f.key == "Partition 1")
            .unwrap();
        assert!(p1.value.contains("NTFS"));
        let p2 = result
            .fields
            .iter()
            .find(|f| f.key == "Partition 2")
            .unwrap();
        assert!(p2.value.contains("Linux"));
    }

    #[test]
    fn raw_partition_count() {
        let header = make_mbr(&[(0x0C, 100, 200), (0x83, 300, 400)]);
        let result = parse_raw_header(&header, "dd", 1_000_000).unwrap();
        let count = result
            .fields
            .iter()
            .find(|f| f.key == "Active Partitions")
            .unwrap();
        assert_eq!(count.value, "2");
    }

    #[test]
    fn raw_sector_count() {
        let result = parse_raw_header(&vec![0u8; 512], "dd", 1_048_576).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Sector Count")
            .unwrap();
        assert!(field.value.contains("2048")); // 1MB / 512
    }

    #[test]
    fn raw_multi_segment_extension() {
        let header = vec![0u8; 512];
        let result = parse_raw_header(&header, "001", 1000).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Segment Extension")
            .unwrap();
        assert!(field.value.contains("multi-segment"));
    }

    #[test]
    fn raw_non_numeric_extension_no_segment() {
        let header = vec![0u8; 512];
        let result = parse_raw_header(&header, "dd", 1000).unwrap();
        assert!(!result.fields.iter().any(|f| f.key == "Segment Extension"));
    }

    #[test]
    fn raw_boot_code_region() {
        let header = make_mbr(&[(0x07, 0, 0)]);
        let result = parse_raw_header(&header, "dd", 1000).unwrap();
        let region = result
            .regions
            .iter()
            .find(|r| r.name == "Boot Code")
            .unwrap();
        assert_eq!(region.start, 0);
        assert_eq!(region.end, 446);
    }

    #[test]
    fn raw_partition_table_region() {
        let header = make_mbr(&[(0x07, 0, 0)]);
        let result = parse_raw_header(&header, "dd", 1000).unwrap();
        let region = result
            .regions
            .iter()
            .find(|r| r.name == "Partition Table")
            .unwrap();
        assert_eq!(region.start, 446);
        assert_eq!(region.end, 510);
    }

    #[test]
    fn raw_fat32_partition() {
        let header = make_mbr(&[(0x0B, 100, 200)]);
        let result = parse_raw_header(&header, "dd", 1000).unwrap();
        let p1 = result
            .fields
            .iter()
            .find(|f| f.key == "Partition 1")
            .unwrap();
        assert!(p1.value.contains("FAT32"));
    }

    // =========================================================================
    // VMDK parser tests
    // =========================================================================

    /// Build a VMDK sparse header
    fn make_vmdk_header(version: u32, flags: u32, capacity_sectors: u64) -> Vec<u8> {
        let mut buf = Vec::new();
        // Signature: KDMV
        buf.extend_from_slice(b"KDMV");
        // Version (offset 4)
        buf.extend_from_slice(&version.to_le_bytes());
        // Flags (offset 8)
        buf.extend_from_slice(&flags.to_le_bytes());
        // Capacity (offset 12)
        buf.extend_from_slice(&capacity_sectors.to_le_bytes());
        buf
    }

    #[test]
    fn vmdk_basic_parse() {
        let header = make_vmdk_header(1, 0, 0);
        let result = parse_vmdk_header(&header, 1000).unwrap();
        assert_eq!(result.format, "VMDK");
    }

    #[test]
    fn vmdk_version() {
        let header = make_vmdk_header(3, 0, 0);
        let result = parse_vmdk_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Version").unwrap();
        assert_eq!(field.value, "3");
        assert_eq!(result.version.as_deref(), Some("3"));
    }

    #[test]
    fn vmdk_flags_compressed() {
        let header = make_vmdk_header(1, 0x20000, 0);
        let result = parse_vmdk_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Flags").unwrap();
        assert!(field.value.contains("CompressedGrains"));
    }

    #[test]
    fn vmdk_flags_multiple() {
        let header = make_vmdk_header(1, 0x01 | 0x40000, 0);
        let result = parse_vmdk_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Flags").unwrap();
        assert!(field.value.contains("NewLineTest"));
        assert!(field.value.contains("HasMarkers"));
    }

    #[test]
    fn vmdk_flags_zero_shows_hex() {
        let header = make_vmdk_header(1, 0, 0);
        let result = parse_vmdk_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Flags").unwrap();
        assert_eq!(field.value, "0x00000000");
    }

    #[test]
    fn vmdk_capacity() {
        // 2_000_000 sectors * 512 = 1 GB
        let header = make_vmdk_header(1, 0, 2_000_000);
        let result = parse_vmdk_header(&header, 500_000).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Virtual Capacity")
            .unwrap();
        assert!(!field.value.is_empty());
    }

    // =========================================================================
    // VHDx parser tests
    // =========================================================================

    /// Build a VHDx file type identifier with optional creator
    fn make_vhdx_header(creator: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        // "vhdxfile" signature (8 bytes)
        buf.extend_from_slice(b"vhdxfile");
        // Creator field: UTF-16LE, up to 512 bytes (256 chars)
        for ch in creator.chars() {
            buf.extend_from_slice(&(ch as u16).to_le_bytes());
        }
        // Pad to offset 520
        while buf.len() < 520 {
            buf.push(0);
        }
        buf
    }

    #[test]
    fn vhdx_basic_parse() {
        let header = make_vhdx_header("");
        let result = parse_vhdx_header(&header, 1000).unwrap();
        assert_eq!(result.format, "VHDx");
        assert_eq!(result.version.as_deref(), Some("2.0"));
    }

    #[test]
    fn vhdx_creator_field() {
        let header = make_vhdx_header("Hyper-V");
        let result = parse_vhdx_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Creator").unwrap();
        assert_eq!(field.value, "Hyper-V");
    }

    #[test]
    fn vhdx_empty_creator_omitted() {
        let header = make_vhdx_header("");
        let result = parse_vhdx_header(&header, 0).unwrap();
        // Empty creator string should not produce a Creator field
        assert!(!result.fields.iter().any(|f| f.key == "Creator"));
    }

    // =========================================================================
    // QCOW2 parser tests
    // =========================================================================

    /// Build a QCOW2 header (big-endian)
    fn make_qcow2_header(
        version: u32,
        backing_offset: u64,
        cluster_bits: u32,
        virtual_size: u64,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        // Signature: QFI\xfb
        buf.extend_from_slice(&[0x51, 0x46, 0x49, 0xFB]);
        // Version (BE, offset 4)
        buf.extend_from_slice(&version.to_be_bytes());
        // Backing file offset (BE, offset 8)
        buf.extend_from_slice(&backing_offset.to_be_bytes());
        // Backing file size (BE, offset 16, u32)
        buf.extend_from_slice(&0u32.to_be_bytes());
        // Cluster bits (BE, offset 20)
        buf.extend_from_slice(&cluster_bits.to_be_bytes());
        // Virtual size (BE, offset 24)
        buf.extend_from_slice(&virtual_size.to_be_bytes());
        buf
    }

    #[test]
    fn qcow2_basic_parse() {
        let header = make_qcow2_header(3, 0, 16, 1_073_741_824);
        let result = parse_qcow2_header(&header, 500_000).unwrap();
        assert_eq!(result.format, "QCOW2");
        assert_eq!(result.version.as_deref(), Some("3"));
    }

    #[test]
    fn qcow2_version_field() {
        let header = make_qcow2_header(2, 0, 16, 0);
        let result = parse_qcow2_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Version").unwrap();
        assert_eq!(field.value, "2");
    }

    #[test]
    fn qcow2_backing_file_offset() {
        let header = make_qcow2_header(3, 0x5000, 16, 0);
        let result = parse_qcow2_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Backing File Offset")
            .unwrap();
        assert!(field.value.contains("5000"));
    }

    #[test]
    fn qcow2_no_backing_file() {
        let header = make_qcow2_header(3, 0, 16, 0);
        let result = parse_qcow2_header(&header, 0).unwrap();
        assert!(!result.fields.iter().any(|f| f.key == "Backing File Offset"));
    }

    #[test]
    fn qcow2_cluster_size() {
        // cluster_bits=16 → cluster size = 2^16 = 65536
        let header = make_qcow2_header(3, 0, 16, 0);
        let result = parse_qcow2_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Cluster Size")
            .unwrap();
        assert!(!field.value.is_empty());
    }

    #[test]
    fn qcow2_virtual_size() {
        let header = make_qcow2_header(3, 0, 16, 10_737_418_240);
        let result = parse_qcow2_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Virtual Size")
            .unwrap();
        assert!(!field.value.is_empty());
    }

    #[test]
    fn qcow2_signature_region() {
        let header = make_qcow2_header(3, 0, 16, 0);
        let result = parse_qcow2_header(&header, 0).unwrap();
        let sig = result
            .regions
            .iter()
            .find(|r| r.name == "Signature")
            .unwrap();
        assert_eq!(sig.start, 0);
        assert_eq!(sig.end, 4);
    }

    #[test]
    fn qcow2_short_header() {
        // Only 6 bytes — should still parse version but skip rest
        let header = vec![0x51, 0x46, 0x49, 0xFB, 0x00, 0x03];
        let result = parse_qcow2_header(&header, 0).unwrap();
        assert_eq!(result.format, "QCOW2");
    }
}
