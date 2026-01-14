// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive format header parsers (ZIP, 7Z, RAR, GZIP)

use crate::common::format_size;
use crate::containers::ContainerError;
use crate::viewer::types::{HeaderRegion, MetadataField, ParsedMetadata};

/// Parse ZIP header and extract metadata
pub fn parse_zip_header(header: &[u8], extension: &str, file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    regions.push(HeaderRegion::new(
        0, 4,
        "Signature",
        "region-signature",
        "ZIP local file header signature (PK\\x03\\x04)",
    ));
    
    let format_name = match extension {
        "ufdr" | "ufdx" => "UFED Report",
        "docx" => "Word Document",
        "xlsx" => "Excel Spreadsheet", 
        "apk" => "Android Package",
        _ => "ZIP Archive",
    };
    
    fields.push(MetadataField::new("Format", format_name, "Format"));
    
    // Version needed at offset 4-6
    if header.len() >= 6 {
        let version = u16::from_le_bytes([header[4], header[5]]);
        
        regions.push(HeaderRegion::new(
            4, 6,
            "Version Needed",
            "region-version",
            "Minimum ZIP version needed to extract",
        ));
        
        fields.push(MetadataField::new(
            "ZIP Version",
            format!("{}.{}", version / 10, version % 10),
            "Format",
        ));
    }
    
    // General purpose flag at offset 6-8
    if header.len() >= 8 {
        let flags = u16::from_le_bytes([header[6], header[7]]);
        
        regions.push(HeaderRegion::new(
            6, 8,
            "Flags",
            "region-flags",
            "General purpose bit flags",
        ));
        
        let encrypted = (flags & 0x01) != 0;
        fields.push(MetadataField::new(
            "Encrypted",
            if encrypted { "Yes" } else { "No" },
            "Security",
        ));
    }
    
    // Compression method at offset 8-10
    if header.len() >= 10 {
        let method = u16::from_le_bytes([header[8], header[9]]);
        
        regions.push(HeaderRegion::new(
            8, 10,
            "Compression",
            "region-compression",
            "Compression method",
        ));
        
        let method_name = match method {
            0 => "None (Stored)",
            8 => "Deflate",
            12 => "BZIP2",
            14 => "LZMA",
            _ => "Unknown",
        };
        
        fields.push(MetadataField::new("Compression", method_name, "Format"));
    }
    
    // Filename length at offset 26-28
    if header.len() >= 30 {
        let filename_len = u16::from_le_bytes([header[26], header[27]]) as usize;
        
        regions.push(HeaderRegion::new(
            26, 30,
            "Length Fields",
            "region-length",
            "Filename and extra field lengths",
        ));
        
        // Extract filename if present
        if header.len() >= 30 + filename_len {
            let filename = String::from_utf8_lossy(&header[30..30 + filename_len]);
            
            regions.push(HeaderRegion::new(
                30,
                (30 + filename_len) as u64,
                "First Filename",
                "region-data",
                "Name of first file in archive",
            ));
            
            fields.push(MetadataField::new("First Entry", filename.to_string(), "Contents"));
        }
    }
    
    fields.push(MetadataField::new("File Size", format_size(file_size), "General"));
    
    Ok(ParsedMetadata {
        format: format_name.to_string(),
        version: Some("ZIP".to_string()),
        fields,
        regions,
    })
}

/// Parse 7-Zip header and extract metadata
pub fn parse_7z_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    regions.push(HeaderRegion::new(
        0, 6,
        "Signature",
        "region-signature",
        "7-Zip magic signature (37 7A BC AF 27 1C)",
    ));
    
    fields.push(MetadataField::new("Format", "7-Zip Archive", "Format").with_offset(0));
    
    // Version (0x06-0x08)
    if header.len() >= 8 {
        let major = header[6];
        let minor = header[7];
        
        regions.push(HeaderRegion::new(
            6, 8,
            "Version",
            "region-version",
            "7z format version (major.minor)",
        ));
        
        fields.push(MetadataField::new(
            "Version",
            format!("{}.{}", major, minor),
            "Format",
        ).with_offset(6));
    }
    
    // Start Header CRC (0x08-0x0C)
    if header.len() >= 12 {
        let crc = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        
        regions.push(HeaderRegion::new(
            8, 12,
            "Start Header CRC",
            "region-checksum",
            "CRC32 of bytes 0x0C-0x1F",
        ));
        
        fields.push(MetadataField::new(
            "Start Header CRC",
            format!("0x{:08X}", crc),
            "Integrity",
        ).with_offset(8));
    }
    
    // Next Header Offset (0x0C-0x14)
    if header.len() >= 20 {
        let next_offset = u64::from_le_bytes([
            header[12], header[13], header[14], header[15],
            header[16], header[17], header[18], header[19],
        ]);
        
        regions.push(HeaderRegion::new(
            12, 20,
            "Next Header Offset",
            "region-offset",
            "Offset to next header (relative to byte 0x20)",
        ));
        
        fields.push(MetadataField::new(
            "Next Header Offset",
            format!("0x{:X} (absolute: 0x{:X})", next_offset, next_offset + 32),
            "Structure",
        ).with_offset(12));
    }
    
    // Next Header Size (0x14-0x1C)
    if header.len() >= 28 {
        let next_size = u64::from_le_bytes([
            header[20], header[21], header[22], header[23],
            header[24], header[25], header[26], header[27],
        ]);
        
        regions.push(HeaderRegion::new(
            20, 28,
            "Next Header Size",
            "region-size",
            "Size of next header in bytes",
        ));
        
        fields.push(MetadataField::new(
            "Next Header Size",
            format_size(next_size),
            "Structure",
        ).with_offset(20));
    }
    
    // Next Header CRC (0x1C-0x20)
    if header.len() >= 32 {
        let crc = u32::from_le_bytes([header[28], header[29], header[30], header[31]]);
        
        regions.push(HeaderRegion::new(
            28, 32,
            "Next Header CRC",
            "region-checksum",
            "CRC32 of next header content",
        ));
        
        fields.push(MetadataField::new(
            "Next Header CRC",
            format!("0x{:08X}", crc),
            "Integrity",
        ).with_offset(28));
    }
    
    fields.push(MetadataField::new("File Size", format_size(file_size), "General"));
    
    Ok(ParsedMetadata {
        format: "7-Zip".to_string(),
        version: if header.len() >= 8 { Some(format!("{}.{}", header[6], header[7])) } else { None },
        fields,
        regions,
    })
}

/// Parse RAR header and extract metadata
pub fn parse_rar_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    let is_rar5 = header.len() >= 8 && header[6] == 0x01 && header[7] == 0x00;
    let format_name = if is_rar5 { "RAR5" } else { "RAR4" };
    
    let sig_len = if is_rar5 { 8 } else { 7 };
    regions.push(HeaderRegion::new(
        0, sig_len,
        "Signature",
        "region-signature",
        format!("{} magic signature", format_name),
    ));
    
    fields.push(MetadataField::new(
        "Format",
        format!("{} Archive", format_name),
        "Format",
    ).with_offset(0));
    
    if is_rar5 {
        if header.len() >= 16 {
            let crc = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
            
            regions.push(HeaderRegion::new(
                8, 12,
                "Header CRC",
                "region-checksum",
                "CRC32 of archive header",
            ));
            
            fields.push(MetadataField::new(
                "Header CRC",
                format!("0x{:08X}", crc),
                "Integrity",
            ).with_offset(8));
        }
    } else {
        if header.len() >= 13 {
            let head_crc = u16::from_le_bytes([header[7], header[8]]);
            let head_type = header[9];
            let head_flags = u16::from_le_bytes([header[10], header[11]]);
            let head_size = u16::from_le_bytes([header[12], header[13]]);
            
            regions.push(HeaderRegion::new(
                7, 14,
                "Archive Header",
                "region-header",
                "RAR archive header block",
            ));
            
            fields.push(MetadataField::new(
                "Header CRC",
                format!("0x{:04X}", head_crc),
                "Integrity",
            ).with_offset(7));
            
            fields.push(MetadataField::new(
                "Header Type",
                format!("0x{:02X}", head_type),
                "Structure",
            ).with_offset(9));
            
            let is_volume = (head_flags & 0x0001) != 0;
            let has_comment = (head_flags & 0x0002) != 0;
            let is_solid = (head_flags & 0x0008) != 0;
            
            let mut flag_str = vec![];
            if is_volume { flag_str.push("Volume"); }
            if has_comment { flag_str.push("Has Comment"); }
            if is_solid { flag_str.push("Solid"); }
            
            fields.push(MetadataField::new(
                "Archive Flags",
                if flag_str.is_empty() { "None".to_string() } else { flag_str.join(", ") },
                "Format",
            ).with_offset(10));
            
            fields.push(MetadataField::new(
                "Header Size",
                format!("{} bytes", head_size),
                "Structure",
            ).with_offset(12));
        }
    }
    
    fields.push(MetadataField::new("File Size", format_size(file_size), "General"));
    
    Ok(ParsedMetadata {
        format: format_name.to_string(),
        version: Some(if is_rar5 { "5.0" } else { "4.x" }.to_string()),
        fields,
        regions,
    })
}

/// Parse GZIP header and extract metadata
pub fn parse_gzip_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    regions.push(HeaderRegion::new(
        0, 2,
        "Signature",
        "region-signature",
        "GZIP magic (1F 8B)",
    ));
    
    fields.push(MetadataField::new("Format", "GZIP Compressed", "Format").with_offset(0));
    
    // Compression method (offset 2)
    if header.len() >= 3 {
        let method = header[2];
        let method_name = match method {
            8 => "Deflate",
            _ => "Unknown",
        };
        
        regions.push(HeaderRegion::new(
            2, 3,
            "Compression",
            "region-compression",
            "Compression method",
        ));
        
        fields.push(MetadataField::new("Compression", method_name, "Format").with_offset(2));
    }
    
    // Flags (offset 3)
    if header.len() >= 4 {
        let flags = header[3];
        
        regions.push(HeaderRegion::new(
            3, 4,
            "Flags",
            "region-flags",
            "GZIP flags (FTEXT, FHCRC, FEXTRA, FNAME, FCOMMENT)",
        ));
        
        let mut flag_names = vec![];
        if flags & 0x01 != 0 { flag_names.push("TEXT"); }
        if flags & 0x02 != 0 { flag_names.push("HCRC"); }
        if flags & 0x04 != 0 { flag_names.push("EXTRA"); }
        if flags & 0x08 != 0 { flag_names.push("NAME"); }
        if flags & 0x10 != 0 { flag_names.push("COMMENT"); }
        
        fields.push(MetadataField::new(
            "Flags",
            if flag_names.is_empty() { "None".to_string() } else { flag_names.join(", ") },
            "Format",
        ).with_offset(3));
    }
    
    // Modification time (offset 4-8)
    if header.len() >= 8 {
        let mtime = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        
        regions.push(HeaderRegion::new(
            4, 8,
            "Modification Time",
            "region-time",
            "Unix timestamp of original file",
        ));
        
        if mtime > 0 {
            let datetime = chrono::DateTime::from_timestamp(mtime as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| format!("{}", mtime));
            
            fields.push(MetadataField::new(
                "Original Mod Time",
                datetime,
                "Timestamps",
            ).with_offset(4));
        }
    }
    
    // Extra flags (offset 8) and OS (offset 9)
    if header.len() >= 10 {
        let xfl = header[8];
        let os = header[9];
        
        regions.push(HeaderRegion::new(
            8, 10,
            "XFL + OS",
            "region-metadata",
            "Extra flags and operating system",
        ));
        
        let os_name = match os {
            0 => "FAT filesystem (MS-DOS)",
            1 => "Amiga",
            2 => "VMS",
            3 => "Unix",
            4 => "VM/CMS",
            5 => "Atari TOS",
            6 => "HPFS filesystem (OS/2, NT)",
            7 => "Macintosh",
            8 => "Z-System",
            9 => "CP/M",
            10 => "TOPS-20",
            11 => "NTFS filesystem (NT)",
            12 => "QDOS",
            13 => "Acorn RISCOS",
            255 | _ => "Unknown",
        };
        
        fields.push(MetadataField::new("Original OS", os_name, "Format").with_offset(9));
        
        let level_hint = match xfl {
            2 => "Maximum compression",
            4 => "Fastest compression",
            _ => "",
        };
        if !level_hint.is_empty() {
            fields.push(MetadataField::new("Compression Level", level_hint, "Format").with_offset(8));
        }
    }
    
    fields.push(MetadataField::new("File Size", format_size(file_size), "General"));
    
    Ok(ParsedMetadata {
        format: "GZIP".to_string(),
        version: None,
        fields,
        regions,
    })
}
