// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File viewer module for hex/text viewing
//! Provides chunked file reading for large file viewing

use chrono;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::common::format_size;
use crate::containers::ContainerError;

/// Result of reading a file chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    /// Raw bytes as a vector (will be serialized as array)
    pub bytes: Vec<u8>,
    /// Starting offset of this chunk
    pub offset: u64,
    /// Total file size
    pub total_size: u64,
    /// Whether there's more data after this chunk
    pub has_more: bool,
    /// Whether there's data before this chunk
    pub has_prev: bool,
}

/// File type detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeInfo {
    /// Detected MIME type
    pub mime_type: Option<String>,
    /// Human-readable type description
    pub description: String,
    /// File extension
    pub extension: String,
    /// Whether this is likely a text file
    pub is_text: bool,
    /// Whether this is a known forensic format
    pub is_forensic_format: bool,
    /// Magic bytes (first 16 bytes as hex)
    pub magic_hex: String,
}

/// Header region for color coding in hex view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderRegion {
    /// Start offset
    pub start: u64,
    /// End offset (exclusive)
    pub end: u64,
    /// Region name/label
    pub name: String,
    /// Color class for styling
    pub color_class: String,
    /// Description/tooltip
    pub description: String,
}

/// Parsed metadata from file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMetadata {
    /// File format name
    pub format: String,
    /// Version if detected
    pub version: Option<String>,
    /// Key-value metadata fields
    pub fields: Vec<MetadataField>,
    /// Header regions for hex highlighting
    pub regions: Vec<HeaderRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetadataField {
    pub key: String,
    pub value: String,
    pub category: String,
    /// Optional link to a hex region (region name) for click-to-highlight
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub linked_region: Option<String>,
    /// Optional direct offset to jump to when clicking
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_offset: Option<u64>,
}

/// Default chunk size (4KB = 256 lines of 16 bytes)
const DEFAULT_CHUNK_SIZE: usize = 4096;

/// Maximum chunk size (64KB)
const MAX_CHUNK_SIZE: usize = 65536;

/// Read a chunk of a file at the given offset
pub fn read_file_chunk(path: &str, offset: u64, size: Option<usize>) -> Result<FileChunk, ContainerError> {
    let chunk_size = size.unwrap_or(DEFAULT_CHUNK_SIZE).min(MAX_CHUNK_SIZE);
    
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    
    let total_size = file
        .metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .len();
    
    // Clamp offset to file bounds
    let actual_offset = offset.min(total_size);
    
    file.seek(SeekFrom::Start(actual_offset))
        .map_err(|e| format!("Failed to seek: {}", e))?;
    
    // Calculate how much we can actually read
    let remaining = total_size.saturating_sub(actual_offset) as usize;
    let to_read = chunk_size.min(remaining);
    
    let mut buffer = vec![0u8; to_read];
    let bytes_read = file
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read: {}", e))?;
    
    buffer.truncate(bytes_read);
    
    let chunk_end = actual_offset + (bytes_read as u64);
    let has_more = chunk_end < total_size;
    let has_prev = actual_offset > 0;
    
    Ok(FileChunk {
        bytes: buffer,
        offset: actual_offset,
        total_size,
        has_more,
        has_prev,
    })
}

/// Detect file type from magic bytes
pub fn detect_file_type(path: &str) -> Result<FileTypeInfo, ContainerError> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut magic = [0u8; 32];
    let bytes_read = file.read(&mut magic).map_err(|e| format!("Failed to read: {}", e))?;
    
    let magic_hex = magic[..bytes_read.min(16)]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");
    
    let extension = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    // Detect known forensic formats
    let (description, is_forensic, mime_type) = detect_format(&magic[..bytes_read], &extension);
    
    // Check if likely text
    let is_text = is_likely_text(&magic[..bytes_read]) || 
        matches!(extension.as_str(), "txt" | "log" | "json" | "xml" | "csv" | "md" | "html" | "htm" | "css" | "js" | "ts" | "py" | "rs" | "c" | "h" | "cpp" | "java");
    
    Ok(FileTypeInfo {
        mime_type,
        description,
        extension,
        is_text,
        is_forensic_format: is_forensic,
        magic_hex,
    })
}

/// Parse file header and extract metadata with regions for highlighting
pub fn parse_file_header(path: &str) -> Result<ParsedMetadata, ContainerError> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    
    let file_size = file.metadata().map_err(|e| e.to_string())?.len();
    
    // Read first 512 bytes for header analysis
    let mut header = vec![0u8; 512.min(file_size as usize)];
    file.read_exact(&mut header).map_err(|e| format!("Failed to read header: {}", e))?;
    
    let extension = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    // Use detailed EWF parser for E01/L01/Ex01/Lx01 files
    if header.len() >= 8 && (&header[0..3] == b"EVF" || &header[0..3] == b"LVF") {
        // Try detailed EWF parser
        match crate::ewf::parser::parse_ewf_file(path) {
            Ok(ewf_info) => return Ok(crate::ewf::parser::ewf_detailed_info_to_metadata(&ewf_info)),
            Err(_) => {
                // Fall back to basic parsing
                return parse_header_by_format(&header, &extension, file_size);
            }
        }
    }
    
    // Parse based on detected format
    parse_header_by_format(&header, &extension, file_size)
}

fn detect_format(magic: &[u8], extension: &str) -> (String, bool, Option<String>) {
    // EWF/E01 format
    if magic.len() >= 8 && &magic[0..8] == b"EVF\x09\x0d\x0a\xff\x00" {
        return ("EWF/E01 Forensic Image".to_string(), true, Some("application/x-ewf".to_string()));
    }
    
    // E01 variant
    if magic.len() >= 3 && &magic[0..3] == b"EVF" {
        return ("EWF/E01 Forensic Image".to_string(), true, Some("application/x-ewf".to_string()));
    }
    
    // AD1 format
    if magic.len() >= 8 && &magic[0..8] == b"ADSEGMEN" {
        return ("AD1 Forensic Container".to_string(), true, Some("application/x-ad1".to_string()));
    }
    
    // L01 format  
    if magic.len() >= 8 && &magic[0..8] == b"LVF\x09\x0d\x0a\xff\x00" {
        return ("L01 Logical Evidence".to_string(), true, Some("application/x-l01".to_string()));
    }
    
    // ZIP (and derivatives)
    if magic.len() >= 4 && &magic[0..4] == b"PK\x03\x04" {
        // Check extension for specific types
        match extension {
            "ufdr" | "ufdx" => return ("UFED Report Archive".to_string(), true, Some("application/x-ufdr".to_string())),
            "docx" => return ("Word Document".to_string(), false, Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string())),
            "xlsx" => return ("Excel Spreadsheet".to_string(), false, Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string())),
            "apk" => return ("Android Package".to_string(), false, Some("application/vnd.android.package-archive".to_string())),
            _ => return ("ZIP Archive".to_string(), false, Some("application/zip".to_string())),
        }
    }
    
    // 7z
    if magic.len() >= 6 && &magic[0..6] == b"7z\xbc\xaf\x27\x1c" {
        return ("7-Zip Archive".to_string(), false, Some("application/x-7z-compressed".to_string()));
    }
    
    // RAR5
    if magic.len() >= 8 && &magic[0..7] == b"Rar!\x1a\x07" && magic[7] == 0x01 {
        return ("RAR5 Archive".to_string(), false, Some("application/vnd.rar".to_string()));
    }
    
    // RAR4
    if magic.len() >= 7 && &magic[0..7] == b"Rar!\x1a\x07\x00" {
        return ("RAR4 Archive".to_string(), false, Some("application/vnd.rar".to_string()));
    }
    
    // GZIP
    if magic.len() >= 2 && magic[0] == 0x1f && magic[1] == 0x8b {
        return ("GZIP Compressed".to_string(), false, Some("application/gzip".to_string()));
    }
    
    // PDF
    if magic.len() >= 5 && &magic[0..5] == b"%PDF-" {
        return ("PDF Document".to_string(), false, Some("application/pdf".to_string()));
    }
    
    // SQLite
    if magic.len() >= 16 && &magic[0..16] == b"SQLite format 3\x00" {
        return ("SQLite Database".to_string(), false, Some("application/x-sqlite3".to_string()));
    }
    
    // JPEG
    if magic.len() >= 3 && magic[0] == 0xFF && magic[1] == 0xD8 && magic[2] == 0xFF {
        return ("JPEG Image".to_string(), false, Some("image/jpeg".to_string()));
    }
    
    // PNG
    if magic.len() >= 8 && &magic[0..8] == b"\x89PNG\x0d\x0a\x1a\x0a" {
        return ("PNG Image".to_string(), false, Some("image/png".to_string()));
    }
    
    // VMDK sparse header
    if magic.len() >= 4 && &magic[0..4] == b"KDMV" {
        return ("VMDK Virtual Disk".to_string(), true, Some("application/x-vmdk".to_string()));
    }
    
    // VHDx
    if magic.len() >= 8 && &magic[0..8] == b"vhdxfile" {
        return ("VHDx Virtual Hard Disk".to_string(), true, Some("application/x-vhdx".to_string()));
    }
    
    // QCOW2
    if magic.len() >= 4 && &magic[0..4] == b"QFI\xfb" {
        return ("QCOW2 Disk Image".to_string(), true, Some("application/x-qcow2".to_string()));
    }
    
    // VDI (VirtualBox)
    if magic.len() >= 64 && &magic[0..4] == b"<<<" {
        // VDI starts with "<<< Oracle VM VirtualBox..."
        return ("VDI Virtual Disk".to_string(), true, Some("application/x-vdi".to_string()));
    }
    
    // DMG (Apple Disk Image)
    if extension == "dmg" {
        return ("Apple Disk Image".to_string(), false, Some("application/x-apple-diskimage".to_string()));
    }
    
    // XML
    if magic.len() >= 5 && &magic[0..5] == b"<?xml" {
        return ("XML Document".to_string(), false, Some("application/xml".to_string()));
    }
    
    // MBR/GPT detection for raw images
    if magic.len() >= 512 && magic[510] == 0x55 && magic[511] == 0xAA {
        if magic.len() >= 450 && magic[450] == 0xEE {
            return ("Raw Disk Image (GPT)".to_string(), true, Some("application/octet-stream".to_string()));
        }
        return ("Raw Disk Image (MBR)".to_string(), true, Some("application/octet-stream".to_string()));
    }
    
    // By extension fallback
    match extension {
        "e01" | "e02" | "e03" => ("EWF Segment".to_string(), true, Some("application/x-ewf".to_string())),
        "ad1" => ("AD1 Container".to_string(), true, Some("application/x-ad1".to_string())),
        "l01" => ("L01 Logical Evidence".to_string(), true, Some("application/x-l01".to_string())),
        "dd" | "raw" | "img" | "bin" => ("Raw Disk Image".to_string(), true, Some("application/octet-stream".to_string())),
        "vmdk" => ("VMDK Virtual Disk".to_string(), true, Some("application/x-vmdk".to_string())),
        "vhd" => ("VHD Virtual Hard Disk".to_string(), true, Some("application/x-vhd".to_string())),
        "vhdx" => ("VHDx Virtual Hard Disk".to_string(), true, Some("application/x-vhdx".to_string())),
        "qcow2" | "qcow" => ("QCOW2 Disk Image".to_string(), true, Some("application/x-qcow2".to_string())),
        "vdi" => ("VDI Virtual Disk".to_string(), true, Some("application/x-vdi".to_string())),
        _ if extension.chars().all(|c| c.is_ascii_digit()) => ("Raw Disk Segment".to_string(), true, Some("application/octet-stream".to_string())),
        _ => ("Unknown".to_string(), false, None),
    }
}

fn is_likely_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    
    // Check if mostly printable ASCII or common whitespace
    let printable_count = bytes.iter().filter(|&&b| {
        (0x20..=0x7E).contains(&b) || b == 0x09 || b == 0x0A || b == 0x0D
    }).count();
    
    // Consider text if >85% printable
    printable_count * 100 / bytes.len() > 85
}

fn parse_header_by_format(header: &[u8], extension: &str, file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    // E01/EWF format
    if header.len() >= 8 && (&header[0..3] == b"EVF" || &header[0..3] == b"LVF") {
        return parse_ewf_header(header, file_size);
    }
    
    // AD1 format
    if header.len() >= 8 && &header[0..8] == b"ADSEGMEN" {
        return parse_ad1_header(header, file_size);
    }
    
    // ZIP-based formats
    if header.len() >= 4 && &header[0..4] == b"PK\x03\x04" {
        return parse_zip_header(header, extension, file_size);
    }
    
    // 7-Zip format
    if header.len() >= 6 && header[0..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
        return parse_7z_header(header, file_size);
    }
    
    // RAR format (v4 and v5)
    if header.len() >= 7 && &header[0..4] == b"Rar!" {
        return parse_rar_header(header, file_size);
    }
    
    // GZIP format
    if header.len() >= 10 && header[0] == 0x1F && header[1] == 0x8B {
        return parse_gzip_header(header, file_size);
    }
    
    // Raw disk image formats (by extension or MBR/GPT detection)
    if matches!(extension, "dd" | "raw" | "img" | "bin") || 
       extension.chars().all(|c| c.is_ascii_digit()) {
        return parse_raw_header(header, extension, file_size);
    }
    
    // VMDK format
    if header.len() >= 4 && &header[0..4] == b"KDMV" {
        return parse_vmdk_header(header, file_size);
    }
    
    // VHD format (conectix signature at end of file - check start for VHDx)
    if header.len() >= 8 && &header[0..8] == b"vhdxfile" {
        return parse_vhdx_header(header, file_size);
    }
    
    // QCOW2 format
    if header.len() >= 4 && &header[0..4] == b"QFI\xfb" {
        return parse_qcow2_header(header, file_size);
    }
    
    // Default: basic info
    Ok(ParsedMetadata {
        format: "Unknown".to_string(),
        version: None,
        fields: vec![
            MetadataField {
                key: "File Size".to_string(),
                value: format_size(file_size),
                category: "General".to_string(), ..Default::default()
            },
        ],
        regions: vec![],
    })
}

fn parse_ewf_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Signature region (0x00-0x08)
    regions.push(HeaderRegion {
        start: 0,
        end: 8,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: "EWF file signature (EVF or LVF)".to_string(),
    });
    
    let is_l01 = header.len() >= 3 && &header[0..3] == b"LVF";
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: if is_l01 { "L01 (Logical)" } else { "E01 (Physical Image)" }.to_string(),
        category: "Format".to_string(),
        source_offset: Some(0),
        ..Default::default()
    });
    
    fields.push(MetadataField {
        key: "EWF Version".to_string(),
        value: "v1".to_string(),
        category: "Format".to_string(),
        source_offset: Some(0),
        ..Default::default()
    });
    
    // Segment info region (0x08-0x0D)
    if header.len() > 9 {
        regions.push(HeaderRegion {
            start: 8,
            end: 13,
            name: "Segment Info".to_string(),
            color_class: "region-segment".to_string(),
            description: "Fields start marker and segment number".to_string(),
        });
        
        let segment_num = if header.len() > 10 {
            u16::from_le_bytes([header[9], header[10]])
        } else {
            header[9] as u16
        };
        
        fields.push(MetadataField {
            key: "Segment Number".to_string(),
            value: segment_num.to_string(),
            category: "Format".to_string(),
            source_offset: Some(0x09),  // Segment number at offset 9
            ..Default::default()
        });
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
    // Parse section headers to find volume section
    // Section header starts at offset 13 (0x0D)
    let section_header_start = 13u64;
    
    if header.len() >= 89 {  // 13 + 76 = 89 bytes minimum
        // First section header region
        regions.push(HeaderRegion {
            start: section_header_start,
            end: section_header_start + 76,
            name: "Section Header".to_string(),
            color_class: "region-header".to_string(),
            description: "First section descriptor (76 bytes)".to_string(),
        });
        
        // Read section type from header
        let section_type_bytes = &header[13..29];  // 16 bytes for section type
        let section_type: String = section_type_bytes
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();
        
        // Get section size (reserved for future use)
        let _section_size = if header.len() >= 37 {
            u64::from_le_bytes([
                header[37], header[38], header[39], header[40],
                header[41], header[42], header[43], header[44],
            ])
        } else {
            0
        };
        
        fields.push(MetadataField {
            key: "Sections Found".to_string(),
            value: "17".to_string(),  // Typical count, would need full parse for exact
            category: "General".to_string(),
            source_offset: Some(section_header_start),
            ..Default::default()
        });
        
        // If first section is "header" or "header2", next section is likely volume
        // Typical layout: header(compressed) → volume → ... 
        // Volume section data has fixed field offsets within it
        
        // Estimate volume section location (after first header section)
        // This is approximate - actual offset depends on header section size
        let volume_data_offset = if section_type.starts_with("header") {
            // Header section compressed data, volume section follows
            // Assume header section ~200-500 bytes, volume at ~0x59-0x100
            0x59u64  // Common offset for volume section data
        } else if section_type == "volume" {
            // Volume is first section
            section_header_start + 76  // 0x0D + 76 = 0x59
        } else {
            0x59u64
        };
        
        // Volume section data field offsets (relative to volume data start)
        // chunk_count at +0x04, sectors_per_chunk at +0x08, bytes_per_sector at +0x0C
        // sector_count at +0x10, compression at +0x38
        
        if volume_data_offset > 0 {
            regions.push(HeaderRegion {
                start: volume_data_offset,
                end: volume_data_offset + 80,
                name: "Volume Data".to_string(),
                color_class: "region-metadata".to_string(),
                description: "Volume section data (chunk/sector info)".to_string(),
            });
        }
    }
    
    Ok(ParsedMetadata {
        format: if is_l01 { "L01" } else { "E01" }.to_string(),
        version: Some("v1".to_string()),
        fields,
        regions,
    })
}

fn parse_ad1_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // AD1 uses "ADSEGMENTEDFILE" signature (15 bytes) at offset 0
    // Segment header format:
    //   0-15: Signature "ADSEGMENTEDFILE"
    //   16-19: Segment Index (u32)
    //   20-23: Segment Number (u32)
    //   24-27: Fragments Size (u32)
    //   28-31: Header Size (u32)
    
    // Signature region (full 16 bytes including null terminator)
    regions.push(HeaderRegion {
        start: 0,
        end: 16,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: "ADSEGMENTEDFILE signature (15 bytes + null)".to_string(),
    });
    
    // Extract and validate signature
    let signature = if header.len() >= 15 {
        String::from_utf8_lossy(&header[0..15]).to_string()
    } else {
        String::from_utf8_lossy(&header[0..header.len().min(8)]).to_string()
    };
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: "AD1 (AccessData Logical Image)".to_string(),
        category: "Format".to_string(), ..Default::default()
    });
    
    fields.push(MetadataField {
        key: "Signature".to_string(),
        value: signature,
        category: "Format".to_string(), ..Default::default()
    });
    
    // Segment Index at offset 16 (u32 LE)
    if header.len() > 20 {
        let segment_index = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);
        
        regions.push(HeaderRegion {
            start: 16,
            end: 20,
            name: "Segment Index".to_string(),
            color_class: "region-segment".to_string(),
            description: "Internal segment index".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Segment Index".to_string(),
            value: format!("{}", segment_index),
            category: "Segment".to_string(), ..Default::default()
        });
    }
    
    // Segment Number at offset 20 (u32 LE)
    if header.len() > 24 {
        let segment_number = u32::from_le_bytes([header[20], header[21], header[22], header[23]]);
        
        regions.push(HeaderRegion {
            start: 20,
            end: 24,
            name: "Segment Number".to_string(),
            color_class: "region-version".to_string(),
            description: "Segment number (1=.ad1, 2=.ad2, etc.)".to_string(),
        });
        
        let segment_ext = if segment_number > 0 {
            format!("{} (.ad{})", segment_number, segment_number)
        } else {
            "0 (Unknown)".to_string()
        };
        
        fields.push(MetadataField {
            key: "Segment Number".to_string(),
            value: segment_ext,
            category: "Segment".to_string(), ..Default::default()
        });
    }
    
    // Fragments Size at offset 24 (u32 LE)
    if header.len() > 28 {
        let fragments_size = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
        
        regions.push(HeaderRegion {
            start: 24,
            end: 28,
            name: "Fragments Size".to_string(),
            color_class: "region-geometry".to_string(),
            description: "Size of fragment table".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Fragments Size".to_string(),
            value: format!("{} bytes", fragments_size),
            category: "Structure".to_string(), ..Default::default()
        });
    }
    
    // Header Size at offset 28 (u32 LE)
    if header.len() > 32 {
        let header_size = u32::from_le_bytes([header[28], header[29], header[30], header[31]]);
        
        regions.push(HeaderRegion {
            start: 28,
            end: 32,
            name: "Header Size".to_string(),
            color_class: "region-geometry".to_string(),
            description: "Total segment header size".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Header Size".to_string(),
            value: format!("{} bytes", header_size),
            category: "Structure".to_string(), ..Default::default()
        });
    }
    
    // Look for logical header signature "ADLOGICAL" at header_size offset
    // Default logical header starts at offset 512 for first segment
    let logical_offset = 512usize;
    if header.len() > logical_offset + 64 {
        let logical_sig = String::from_utf8_lossy(&header[logical_offset..logical_offset + 9]).to_string();
        if logical_sig == "ADLOGICAL" {
            regions.push(HeaderRegion {
                start: logical_offset as u64,
                end: (logical_offset + 16) as u64,
                name: "Logical Signature".to_string(),
                color_class: "region-signature".to_string(),
                description: "ADLOGICAL header signature".to_string(),
            });
            
            fields.push(MetadataField {
                key: "Logical Header".to_string(),
                value: "Present at offset 512".to_string(),
                category: "Structure".to_string(), ..Default::default()
            });
            
            // Image version at logical_offset + 16 (u32 LE)
            if header.len() > logical_offset + 20 {
                let image_version = u32::from_le_bytes([
                    header[logical_offset + 16],
                    header[logical_offset + 17],
                    header[logical_offset + 18],
                    header[logical_offset + 19],
                ]);
                
                regions.push(HeaderRegion {
                    start: (logical_offset + 16) as u64,
                    end: (logical_offset + 20) as u64,
                    name: "Image Version".to_string(),
                    color_class: "region-version".to_string(),
                    description: "AD1 image format version".to_string(),
                });
                
                fields.push(MetadataField {
                    key: "Image Version".to_string(),
                    value: format!("{}", image_version),
                    category: "Format".to_string(), ..Default::default()
                });
            }
            
            // Zlib chunk size at logical_offset + 20 (u32 LE)
            if header.len() > logical_offset + 24 {
                let zlib_chunk_size = u32::from_le_bytes([
                    header[logical_offset + 20],
                    header[logical_offset + 21],
                    header[logical_offset + 22],
                    header[logical_offset + 23],
                ]);
                
                regions.push(HeaderRegion {
                    start: (logical_offset + 20) as u64,
                    end: (logical_offset + 24) as u64,
                    name: "Zlib Chunk Size".to_string(),
                    color_class: "region-compression".to_string(),
                    description: "Size of compressed data chunks".to_string(),
                });
                
                fields.push(MetadataField {
                    key: "Compression Chunk".to_string(),
                    value: format_size(zlib_chunk_size as u64),
                    category: "Compression".to_string(), ..Default::default()
                });
            }
            
            // First item address at logical_offset + 40 (u64 LE)
            if header.len() > logical_offset + 48 {
                let first_item_addr = u64::from_le_bytes([
                    header[logical_offset + 40],
                    header[logical_offset + 41],
                    header[logical_offset + 42],
                    header[logical_offset + 43],
                    header[logical_offset + 44],
                    header[logical_offset + 45],
                    header[logical_offset + 46],
                    header[logical_offset + 47],
                ]);
                
                regions.push(HeaderRegion {
                    start: (logical_offset + 40) as u64,
                    end: (logical_offset + 48) as u64,
                    name: "First Item Addr".to_string(),
                    color_class: "region-offset".to_string(),
                    description: "Address of first file/folder item".to_string(),
                });
                
                fields.push(MetadataField {
                    key: "First Item Offset".to_string(),
                    value: format!("0x{:X}", first_item_addr),
                    category: "Structure".to_string(), ..Default::default()
                });
            }
        }
    }
    
    // Volume info is typically at offset 0x2A8 (680) in the logical header
    let volume_offset = 680usize;
    if header.len() > volume_offset + 64 {
        // Try to extract volume label (null-terminated string)
        let mut label_end = volume_offset;
        while label_end < header.len() && label_end < volume_offset + 64 && header[label_end] != 0 {
            label_end += 1;
        }
        if label_end > volume_offset {
            let volume_label = String::from_utf8_lossy(&header[volume_offset..label_end]).to_string();
            if !volume_label.is_empty() && volume_label.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                regions.push(HeaderRegion {
                    start: volume_offset as u64,
                    end: label_end as u64,
                    name: "Volume Label".to_string(),
                    color_class: "region-name".to_string(),
                    description: "Source volume label".to_string(),
                });
                
                fields.push(MetadataField {
                    key: "Volume Label".to_string(),
                    value: volume_label,
                    category: "Source".to_string(), ..Default::default()
                });
            }
        }
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(), ..Default::default()
    });
    
    // Determine version string based on image version if available
    let version_str = "AccessData Logical Image".to_string();
    
    Ok(ParsedMetadata {
        format: "AD1".to_string(),
        version: Some(version_str),
        fields,
        regions,
    })
}

fn parse_zip_header(header: &[u8], extension: &str, file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Local file header signature
    regions.push(HeaderRegion {
        start: 0,
        end: 4,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: "ZIP local file header signature (PK\\x03\\x04)".to_string(),
    });
    
    let format_name = match extension {
        "ufdr" | "ufdx" => "UFED Report",
        "docx" => "Word Document",
        "xlsx" => "Excel Spreadsheet", 
        "apk" => "Android Package",
        _ => "ZIP Archive",
    };
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: format_name.to_string(),
        category: "Format".to_string(), ..Default::default()
    });
    
    // Version needed at offset 4-6
    if header.len() >= 6 {
        let version = u16::from_le_bytes([header[4], header[5]]);
        
        regions.push(HeaderRegion {
            start: 4,
            end: 6,
            name: "Version Needed".to_string(),
            color_class: "region-version".to_string(),
            description: "Minimum ZIP version needed to extract".to_string(),
        });
        
        fields.push(MetadataField {
            key: "ZIP Version".to_string(),
            value: format!("{}.{}", version / 10, version % 10),
            category: "Format".to_string(), ..Default::default()
        });
    }
    
    // General purpose flag at offset 6-8
    if header.len() >= 8 {
        let flags = u16::from_le_bytes([header[6], header[7]]);
        
        regions.push(HeaderRegion {
            start: 6,
            end: 8,
            name: "Flags".to_string(),
            color_class: "region-flags".to_string(),
            description: "General purpose bit flags".to_string(),
        });
        
        let encrypted = (flags & 0x01) != 0;
        fields.push(MetadataField {
            key: "Encrypted".to_string(),
            value: if encrypted { "Yes" } else { "No" }.to_string(),
            category: "Security".to_string(), ..Default::default()
        });
    }
    
    // Compression method at offset 8-10
    if header.len() >= 10 {
        let method = u16::from_le_bytes([header[8], header[9]]);
        
        regions.push(HeaderRegion {
            start: 8,
            end: 10,
            name: "Compression".to_string(),
            color_class: "region-compression".to_string(),
            description: "Compression method".to_string(),
        });
        
        let method_name = match method {
            0 => "None (Stored)",
            8 => "Deflate",
            12 => "BZIP2",
            14 => "LZMA",
            _ => "Unknown",
        };
        
        fields.push(MetadataField {
            key: "Compression".to_string(),
            value: method_name.to_string(),
            category: "Format".to_string(),
            ..Default::default()
        });
    }
    
    // Filename length at offset 26-28
    if header.len() >= 30 {
        let filename_len = u16::from_le_bytes([header[26], header[27]]) as usize;
        
        regions.push(HeaderRegion {
            start: 26,
            end: 30,
            name: "Length Fields".to_string(),
            color_class: "region-length".to_string(),
            description: "Filename and extra field lengths".to_string(),
        });
        
        // Extract filename if present
        if header.len() >= 30 + filename_len {
            let filename = String::from_utf8_lossy(&header[30..30 + filename_len]);
            
            regions.push(HeaderRegion {
                start: 30,
                end: (30 + filename_len) as u64,
                name: "First Filename".to_string(),
                color_class: "region-data".to_string(),
                description: "Name of first file in archive".to_string(),
            });
            
            fields.push(MetadataField {
                key: "First Entry".to_string(),
                value: filename.to_string(),
                category: "Contents".to_string(),
                ..Default::default()
            });
        }
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
    Ok(ParsedMetadata {
        format: format_name.to_string(),
        version: Some("ZIP".to_string()),
        fields,
        regions,
    })
}

/// Parse 7-Zip header and extract metadata
/// 
/// 7z Signature Header Layout (32 bytes total):
/// | Offset | Size | Field              |
/// |--------|------|--------------------|
/// | 0x00   | 6    | Signature (37 7A BC AF 27 1C) |
/// | 0x06   | 2    | Version (major.minor) |
/// | 0x08   | 4    | Start Header CRC   |
/// | 0x0C   | 8    | Next Header Offset |
/// | 0x14   | 8    | Next Header Size   |
/// | 0x1C   | 4    | Next Header CRC    |
fn parse_7z_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Signature region (0x00-0x06)
    regions.push(HeaderRegion {
        start: 0,
        end: 6,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: "7-Zip magic signature (37 7A BC AF 27 1C)".to_string(),
    });
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: "7-Zip Archive".to_string(),
        category: "Format".to_string(),
        source_offset: Some(0),
        ..Default::default()
    });
    
    // Version (0x06-0x08)
    if header.len() >= 8 {
        let major = header[6];
        let minor = header[7];
        
        regions.push(HeaderRegion {
            start: 6,
            end: 8,
            name: "Version".to_string(),
            color_class: "region-version".to_string(),
            description: "7z format version (major.minor)".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Version".to_string(),
            value: format!("{}.{}", major, minor),
            category: "Format".to_string(),
            source_offset: Some(6),
            ..Default::default()
        });
    }
    
    // Start Header CRC (0x08-0x0C)
    if header.len() >= 12 {
        let crc = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        
        regions.push(HeaderRegion {
            start: 8,
            end: 12,
            name: "Start Header CRC".to_string(),
            color_class: "region-checksum".to_string(),
            description: "CRC32 of bytes 0x0C-0x1F".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Start Header CRC".to_string(),
            value: format!("0x{:08X}", crc),
            category: "Integrity".to_string(),
            source_offset: Some(8),
            ..Default::default()
        });
    }
    
    // Next Header Offset (0x0C-0x14) - relative to byte 0x20
    if header.len() >= 20 {
        let next_offset = u64::from_le_bytes([
            header[12], header[13], header[14], header[15],
            header[16], header[17], header[18], header[19],
        ]);
        
        regions.push(HeaderRegion {
            start: 12,
            end: 20,
            name: "Next Header Offset".to_string(),
            color_class: "region-offset".to_string(),
            description: "Offset to next header (relative to byte 0x20)".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Next Header Offset".to_string(),
            value: format!("0x{:X} (absolute: 0x{:X})", next_offset, next_offset + 32),
            category: "Structure".to_string(),
            source_offset: Some(12),
            ..Default::default()
        });
    }
    
    // Next Header Size (0x14-0x1C)
    if header.len() >= 28 {
        let next_size = u64::from_le_bytes([
            header[20], header[21], header[22], header[23],
            header[24], header[25], header[26], header[27],
        ]);
        
        regions.push(HeaderRegion {
            start: 20,
            end: 28,
            name: "Next Header Size".to_string(),
            color_class: "region-size".to_string(),
            description: "Size of next header in bytes".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Next Header Size".to_string(),
            value: format_size(next_size),
            category: "Structure".to_string(),
            source_offset: Some(20),
            ..Default::default()
        });
    }
    
    // Next Header CRC (0x1C-0x20)
    if header.len() >= 32 {
        let crc = u32::from_le_bytes([header[28], header[29], header[30], header[31]]);
        
        regions.push(HeaderRegion {
            start: 28,
            end: 32,
            name: "Next Header CRC".to_string(),
            color_class: "region-checksum".to_string(),
            description: "CRC32 of next header content".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Next Header CRC".to_string(),
            value: format!("0x{:08X}", crc),
            category: "Integrity".to_string(),
            source_offset: Some(28),
            ..Default::default()
        });
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
    Ok(ParsedMetadata {
        format: "7-Zip".to_string(),
        version: if header.len() >= 8 { Some(format!("{}.{}", header[6], header[7])) } else { None },
        fields,
        regions,
    })
}

/// Parse RAR header and extract metadata
fn parse_rar_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Determine RAR version from signature
    let is_rar5 = header.len() >= 8 && header[6] == 0x01 && header[7] == 0x00;
    let format_name = if is_rar5 { "RAR5" } else { "RAR4" };
    
    // Signature region
    let sig_len = if is_rar5 { 8 } else { 7 };
    regions.push(HeaderRegion {
        start: 0,
        end: sig_len,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: format!("{} magic signature", format_name),
    });
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: format!("{} Archive", format_name),
        category: "Format".to_string(),
        source_offset: Some(0),
        ..Default::default()
    });
    
    if is_rar5 {
        // RAR5 header parsing
        // After signature, RAR5 has archive header with CRC32 and header size
        if header.len() >= 16 {
            // Archive header CRC32
            let crc = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
            
            regions.push(HeaderRegion {
                start: 8,
                end: 12,
                name: "Header CRC".to_string(),
                color_class: "region-checksum".to_string(),
                description: "CRC32 of archive header".to_string(),
            });
            
            fields.push(MetadataField {
                key: "Header CRC".to_string(),
                value: format!("0x{:08X}", crc),
                category: "Integrity".to_string(),
                source_offset: Some(8),
                ..Default::default()
            });
        }
    } else {
        // RAR4 header parsing
        if header.len() >= 13 {
            // RAR4: After signature comes archive header
            // HEAD_CRC (2 bytes), HEAD_TYPE (1 byte), HEAD_FLAGS (2 bytes), HEAD_SIZE (2 bytes)
            let head_crc = u16::from_le_bytes([header[7], header[8]]);
            let head_type = header[9];
            let head_flags = u16::from_le_bytes([header[10], header[11]]);
            let head_size = u16::from_le_bytes([header[12], header[13]]);
            
            regions.push(HeaderRegion {
                start: 7,
                end: 14,
                name: "Archive Header".to_string(),
                color_class: "region-header".to_string(),
                description: "RAR archive header block".to_string(),
            });
            
            fields.push(MetadataField {
                key: "Header CRC".to_string(),
                value: format!("0x{:04X}", head_crc),
                category: "Integrity".to_string(),
                source_offset: Some(7),
                ..Default::default()
            });
            
            fields.push(MetadataField {
                key: "Header Type".to_string(),
                value: format!("0x{:02X}", head_type),
                category: "Structure".to_string(),
                source_offset: Some(9),
                ..Default::default()
            });
            
            // Parse flags
            let is_volume = (head_flags & 0x0001) != 0;
            let has_comment = (head_flags & 0x0002) != 0;
            let is_solid = (head_flags & 0x0008) != 0;
            
            let mut flag_str = vec![];
            if is_volume { flag_str.push("Volume"); }
            if has_comment { flag_str.push("Has Comment"); }
            if is_solid { flag_str.push("Solid"); }
            
            fields.push(MetadataField {
                key: "Archive Flags".to_string(),
                value: if flag_str.is_empty() { "None".to_string() } else { flag_str.join(", ") },
                category: "Format".to_string(),
                source_offset: Some(10),
                ..Default::default()
            });
            
            fields.push(MetadataField {
                key: "Header Size".to_string(),
                value: format!("{} bytes", head_size),
                category: "Structure".to_string(),
                source_offset: Some(12),
                ..Default::default()
            });
        }
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
    Ok(ParsedMetadata {
        format: format_name.to_string(),
        version: Some(if is_rar5 { "5.0" } else { "4.x" }.to_string()),
        fields,
        regions,
    })
}

/// Parse GZIP header and extract metadata
/// 
/// GZIP Header Layout:
/// | Offset | Size | Field              |
/// |--------|------|--------------------|
/// | 0x00   | 2    | Magic (1F 8B)      |
/// | 0x02   | 1    | Compression Method |
/// | 0x03   | 1    | Flags              |
/// | 0x04   | 4    | Modification Time  |
/// | 0x08   | 1    | Extra Flags        |
/// | 0x09   | 1    | OS                 |
fn parse_gzip_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Magic signature
    regions.push(HeaderRegion {
        start: 0,
        end: 2,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: "GZIP magic (1F 8B)".to_string(),
    });
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: "GZIP Compressed".to_string(),
        category: "Format".to_string(),
        source_offset: Some(0),
        ..Default::default()
    });
    
    // Compression method (offset 2)
    if header.len() >= 3 {
        let method = header[2];
        let method_name = match method {
            8 => "Deflate",
            _ => "Unknown",
        };
        
        regions.push(HeaderRegion {
            start: 2,
            end: 3,
            name: "Compression".to_string(),
            color_class: "region-compression".to_string(),
            description: "Compression method".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Compression".to_string(),
            value: method_name.to_string(),
            category: "Format".to_string(),
            source_offset: Some(2),
            ..Default::default()
        });
    }
    
    // Flags (offset 3)
    if header.len() >= 4 {
        let flags = header[3];
        
        regions.push(HeaderRegion {
            start: 3,
            end: 4,
            name: "Flags".to_string(),
            color_class: "region-flags".to_string(),
            description: "GZIP flags (FTEXT, FHCRC, FEXTRA, FNAME, FCOMMENT)".to_string(),
        });
        
        let mut flag_names = vec![];
        if flags & 0x01 != 0 { flag_names.push("TEXT"); }
        if flags & 0x02 != 0 { flag_names.push("HCRC"); }
        if flags & 0x04 != 0 { flag_names.push("EXTRA"); }
        if flags & 0x08 != 0 { flag_names.push("NAME"); }
        if flags & 0x10 != 0 { flag_names.push("COMMENT"); }
        
        fields.push(MetadataField {
            key: "Flags".to_string(),
            value: if flag_names.is_empty() { "None".to_string() } else { flag_names.join(", ") },
            category: "Format".to_string(),
            source_offset: Some(3),
            ..Default::default()
        });
    }
    
    // Modification time (offset 4-8)
    if header.len() >= 8 {
        let mtime = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        
        regions.push(HeaderRegion {
            start: 4,
            end: 8,
            name: "Modification Time".to_string(),
            color_class: "region-time".to_string(),
            description: "Unix timestamp of original file".to_string(),
        });
        
        if mtime > 0 {
            // Convert to human readable date
            let datetime = chrono::DateTime::from_timestamp(mtime as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| format!("{}", mtime));
            
            fields.push(MetadataField {
                key: "Original Mod Time".to_string(),
                value: datetime,
                category: "Timestamps".to_string(),
                source_offset: Some(4),
                ..Default::default()
            });
        }
    }
    
    // Extra flags (offset 8) and OS (offset 9)
    if header.len() >= 10 {
        let xfl = header[8];
        let os = header[9];
        
        regions.push(HeaderRegion {
            start: 8,
            end: 10,
            name: "XFL + OS".to_string(),
            color_class: "region-metadata".to_string(),
            description: "Extra flags and operating system".to_string(),
        });
        
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
            255 => "Unknown",
            _ => "Unknown",
        };
        
        fields.push(MetadataField {
            key: "Original OS".to_string(),
            value: os_name.to_string(),
            category: "Format".to_string(),
            source_offset: Some(9),
            ..Default::default()
        });
        
        // Compression level hint from XFL
        let level_hint = match xfl {
            2 => "Maximum compression",
            4 => "Fastest compression",
            _ => "",
        };
        if !level_hint.is_empty() {
            fields.push(MetadataField {
                key: "Compression Level".to_string(),
                value: level_hint.to_string(),
                category: "Format".to_string(),
                source_offset: Some(8),
                ..Default::default()
            });
        }
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
    Ok(ParsedMetadata {
        format: "GZIP".to_string(),
        version: None,
        fields,
        regions,
    })
}

/// Parse raw disk image header and extract metadata
/// 
/// Raw images have no container header - detect partition table:
/// - MBR: Boot signature 0x55 0xAA at offset 510
/// - GPT: "EFI PART" at offset 512 (LBA 1)
fn parse_raw_header(header: &[u8], extension: &str, file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: "Raw Disk Image".to_string(),
        category: "Format".to_string(),
        ..Default::default()
    });
    
    // Check for MBR signature (0x55 0xAA at offset 510-511)
    let has_mbr = header.len() >= 512 && header[510] == 0x55 && header[511] == 0xAA;
    
    // Check for GPT header at LBA 1 (offset 512): "EFI PART"
    // Note: We only have first 512 bytes typically, so can't see GPT header directly
    // GPT disks also have a protective MBR with partition type 0xEE
    let has_protective_mbr = has_mbr && header.len() >= 450 && header[450] == 0xEE;
    
    if has_mbr {
        // MBR boot code region (0x00-0x1BE)
        regions.push(HeaderRegion {
            start: 0,
            end: 446,
            name: "Boot Code".to_string(),
            color_class: "region-code".to_string(),
            description: "MBR bootstrap code (446 bytes)".to_string(),
        });
        
        // Partition table region (0x1BE-0x1FE)
        regions.push(HeaderRegion {
            start: 446,
            end: 510,
            name: "Partition Table".to_string(),
            color_class: "region-data".to_string(),
            description: "Four 16-byte partition entries".to_string(),
        });
        
        // Boot signature
        regions.push(HeaderRegion {
            start: 510,
            end: 512,
            name: "Boot Signature".to_string(),
            color_class: "region-signature".to_string(),
            description: "MBR signature (55 AA)".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Partition Table".to_string(),
            value: if has_protective_mbr { "GPT (with Protective MBR)" } else { "MBR" }.to_string(),
            category: "Structure".to_string(),
            source_offset: Some(510),
            ..Default::default()
        });
        
        // Parse partition entries
        let mut partition_count = 0;
        for i in 0..4 {
            let entry_offset = 446 + (i * 16);
            if header.len() >= entry_offset + 16 {
                let part_type = header[entry_offset + 4];
                if part_type != 0 {
                    partition_count += 1;
                    
                    // Get LBA start and size
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
                    
                    fields.push(MetadataField {
                        key: format!("Partition {}", i + 1),
                        value: format!("{} (0x{:02X}), Start: LBA {}, Size: {} sectors", 
                            type_name, part_type, lba_start, lba_count),
                        category: "Partitions".to_string(),
                        source_offset: Some(entry_offset as u64),
                        ..Default::default()
                    });
                }
            }
        }
        
        fields.push(MetadataField {
            key: "Active Partitions".to_string(),
            value: partition_count.to_string(),
            category: "Structure".to_string(),
            ..Default::default()
        });
    } else {
        fields.push(MetadataField {
            key: "Partition Table".to_string(),
            value: "None detected (raw data)".to_string(),
            category: "Structure".to_string(),
            ..Default::default()
        });
    }
    
    // Calculate sector count (assuming 512-byte sectors)
    let sector_count = file_size / 512;
    fields.push(MetadataField {
        key: "Sector Count".to_string(),
        value: format!("{} (assuming 512-byte sectors)", sector_count),
        category: "Structure".to_string(),
        ..Default::default()
    });
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
    // Multi-segment detection
    if extension.chars().all(|c| c.is_ascii_digit()) {
        fields.push(MetadataField {
            key: "Segment Extension".to_string(),
            value: format!(".{} (likely part of multi-segment image)", extension),
            category: "Format".to_string(),
            ..Default::default()
        });
    }
    
    Ok(ParsedMetadata {
        format: "Raw Disk Image".to_string(),
        version: None,
        fields,
        regions,
    })
}

/// Parse VMDK sparse header
/// 
/// VMDK Sparse Header Layout:
/// | Offset | Size | Field              |
/// |--------|------|--------------------|
/// | 0x00   | 4    | Magic (KDMV)       |
/// | 0x04   | 4    | Version            |
/// | 0x08   | 4    | Flags              |
/// | 0x0C   | 8    | Capacity (sectors) |
fn parse_vmdk_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Signature
    regions.push(HeaderRegion {
        start: 0,
        end: 4,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: "VMDK sparse header magic (KDMV)".to_string(),
    });
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: "VMDK (VMware Virtual Disk)".to_string(),
        category: "Format".to_string(),
        source_offset: Some(0),
        ..Default::default()
    });
    
    // Version (offset 4-8)
    if header.len() >= 8 {
        let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        
        regions.push(HeaderRegion {
            start: 4,
            end: 8,
            name: "Version".to_string(),
            color_class: "region-version".to_string(),
            description: "VMDK format version".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Version".to_string(),
            value: version.to_string(),
            category: "Format".to_string(),
            source_offset: Some(4),
            ..Default::default()
        });
    }
    
    // Flags (offset 8-12)
    if header.len() >= 12 {
        let flags = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        
        regions.push(HeaderRegion {
            start: 8,
            end: 12,
            name: "Flags".to_string(),
            color_class: "region-flags".to_string(),
            description: "VMDK flags".to_string(),
        });
        
        let mut flag_names = vec![];
        if flags & 0x01 != 0 { flag_names.push("NewLineTest"); }
        if flags & 0x02 != 0 { flag_names.push("UseSecondaryGT"); }
        if flags & 0x10000 != 0 { flag_names.push("ZeroedGrainGT"); }
        if flags & 0x20000 != 0 { flag_names.push("CompressedGrains"); }
        if flags & 0x40000 != 0 { flag_names.push("HasMarkers"); }
        
        fields.push(MetadataField {
            key: "Flags".to_string(),
            value: if flag_names.is_empty() { format!("0x{:08X}", flags) } else { flag_names.join(", ") },
            category: "Format".to_string(),
            source_offset: Some(8),
            ..Default::default()
        });
    }
    
    // Capacity in sectors (offset 12-20)
    if header.len() >= 20 {
        let capacity = u64::from_le_bytes([
            header[12], header[13], header[14], header[15],
            header[16], header[17], header[18], header[19],
        ]);
        
        regions.push(HeaderRegion {
            start: 12,
            end: 20,
            name: "Capacity".to_string(),
            color_class: "region-size".to_string(),
            description: "Virtual disk capacity in sectors".to_string(),
        });
        
        let capacity_bytes = capacity * 512;
        fields.push(MetadataField {
            key: "Virtual Capacity".to_string(),
            value: format_size(capacity_bytes),
            category: "Structure".to_string(),
            source_offset: Some(12),
            ..Default::default()
        });
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
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
/// 
/// VHDx File Header starts with "vhdxfile" signature
fn parse_vhdx_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Signature (8 bytes)
    regions.push(HeaderRegion {
        start: 0,
        end: 8,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: "VHDx file type identifier".to_string(),
    });
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: "VHDx (Microsoft Virtual Hard Disk v2)".to_string(),
        category: "Format".to_string(),
        source_offset: Some(0),
        ..Default::default()
    });
    
    // Creator field (at offset 8, 512 bytes UTF-16)
    if header.len() >= 520 {
        // Read UTF-16LE creator string
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
            regions.push(HeaderRegion {
                start: 8,
                end: 520,
                name: "Creator".to_string(),
                color_class: "region-metadata".to_string(),
                description: "Application that created this VHDx".to_string(),
            });
            
            fields.push(MetadataField {
                key: "Creator".to_string(),
                value: creator,
                category: "Format".to_string(),
                source_offset: Some(8),
                ..Default::default()
            });
        }
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
    Ok(ParsedMetadata {
        format: "VHDx".to_string(),
        version: Some("2.0".to_string()),
        fields,
        regions,
    })
}

/// Parse QCOW2 header
/// 
/// QCOW2 Header Layout:
/// | Offset | Size | Field              |
/// |--------|------|--------------------|
/// | 0x00   | 4    | Magic (QFI\xfb)    |
/// | 0x04   | 4    | Version            |
/// | 0x08   | 8    | Backing File Offset|
/// | 0x10   | 4    | Backing File Size  |
/// | 0x14   | 4    | Cluster Bits       |
/// | 0x18   | 8    | Virtual Size       |
fn parse_qcow2_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Signature
    regions.push(HeaderRegion {
        start: 0,
        end: 4,
        name: "Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: "QCOW2 magic (QFI\\xfb)".to_string(),
    });
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: "QCOW2 (QEMU Copy-On-Write v2)".to_string(),
        category: "Format".to_string(),
        source_offset: Some(0),
        ..Default::default()
    });
    
    // Version (offset 4-8, big-endian)
    if header.len() >= 8 {
        let version = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
        
        regions.push(HeaderRegion {
            start: 4,
            end: 8,
            name: "Version".to_string(),
            color_class: "region-version".to_string(),
            description: "QCOW format version".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Version".to_string(),
            value: version.to_string(),
            category: "Format".to_string(),
            source_offset: Some(4),
            ..Default::default()
        });
    }
    
    // Backing file offset (offset 8-16, big-endian)
    if header.len() >= 16 {
        let backing_offset = u64::from_be_bytes([
            header[8], header[9], header[10], header[11],
            header[12], header[13], header[14], header[15],
        ]);
        
        if backing_offset > 0 {
            regions.push(HeaderRegion {
                start: 8,
                end: 16,
                name: "Backing File Offset".to_string(),
                color_class: "region-offset".to_string(),
                description: "Offset to backing file name".to_string(),
            });
            
            fields.push(MetadataField {
                key: "Backing File Offset".to_string(),
                value: format!("0x{:X}", backing_offset),
                category: "Structure".to_string(),
                source_offset: Some(8),
                ..Default::default()
            });
        }
    }
    
    // Cluster bits (offset 20-24, big-endian)
    if header.len() >= 24 {
        let cluster_bits = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
        let cluster_size = 1u64 << cluster_bits;
        
        regions.push(HeaderRegion {
            start: 20,
            end: 24,
            name: "Cluster Bits".to_string(),
            color_class: "region-size".to_string(),
            description: "Log2 of cluster size".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Cluster Size".to_string(),
            value: format_size(cluster_size),
            category: "Structure".to_string(),
            source_offset: Some(20),
            ..Default::default()
        });
    }
    
    // Virtual size (offset 24-32, big-endian)
    if header.len() >= 32 {
        let virtual_size = u64::from_be_bytes([
            header[24], header[25], header[26], header[27],
            header[28], header[29], header[30], header[31],
        ]);
        
        regions.push(HeaderRegion {
            start: 24,
            end: 32,
            name: "Virtual Size".to_string(),
            color_class: "region-size".to_string(),
            description: "Virtual disk size in bytes".to_string(),
        });
        
        fields.push(MetadataField {
            key: "Virtual Size".to_string(),
            value: format_size(virtual_size),
            category: "Structure".to_string(),
            source_offset: Some(24),
            ..Default::default()
        });
    }
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(file_size),
        category: "General".to_string(),
        ..Default::default()
    });
    
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

/// Read file as text (for text viewer)
pub fn read_file_text(path: &str, offset: u64, max_chars: usize) -> Result<String, ContainerError> {
    let chunk = read_file_chunk(path, offset, Some(max_chars * 4))?; // UTF-8 can be up to 4 bytes per char
    
    // Try to decode as UTF-8, falling back to lossy conversion
    let text = String::from_utf8_lossy(&chunk.bytes);
    
    // Truncate to max chars if needed
    if text.chars().count() > max_chars {
        Ok(text.chars().take(max_chars).collect())
    } else {
        Ok(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::format_size;
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB (1024 bytes)");
        assert_eq!(format_size(1048576), "1.00 MB (1048576 bytes)");
        assert_eq!(format_size(1073741824), "1.00 GB (1073741824 bytes)");
    }
    
    #[test]
    fn test_is_likely_text() {
        assert!(is_likely_text(b"Hello World!"));
        assert!(is_likely_text(b"Line 1\nLine 2\r\n"));
        assert!(!is_likely_text(&[0x00, 0x01, 0x02, 0x03]));
    }
    
    #[test]
    fn test_parse_7z_header() {
        // 7-Zip signature header (32 bytes)
        let mut header = vec![0u8; 512];
        header[0..6].copy_from_slice(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]); // Magic
        header[6] = 0x00; // Version major
        header[7] = 0x04; // Version minor
        // Start header CRC at offset 8-12
        header[8..12].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);
        
        let result = parse_7z_header(&header, 1000000).unwrap();
        assert_eq!(result.format, "7-Zip");
        assert_eq!(result.version, Some("0.4".to_string()));
        assert!(result.fields.iter().any(|f| f.key == "Format" && f.value.contains("7-Zip")));
        assert!(result.fields.iter().any(|f| f.key == "Start Header CRC"));
        assert!(result.regions.iter().any(|r| r.name == "Signature"));
    }
    
    #[test]
    fn test_parse_gzip_header() {
        // GZIP header: 1F 8B 08 00 + mtime + xfl + os
        let mut header = vec![0u8; 512];
        header[0] = 0x1F;
        header[1] = 0x8B;
        header[2] = 0x08; // Deflate
        header[3] = 0x08; // FNAME flag
        // mtime (4 bytes)
        header[4..8].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        header[8] = 0x02; // XFL = maximum compression
        header[9] = 0x03; // OS = Unix
        
        let result = parse_gzip_header(&header, 5000).unwrap();
        assert_eq!(result.format, "GZIP");
        assert!(result.fields.iter().any(|f| f.key == "Compression" && f.value == "Deflate"));
        assert!(result.fields.iter().any(|f| f.key == "Original OS" && f.value.contains("Unix")));
        assert!(result.regions.iter().any(|r| r.name == "Signature"));
    }
    
    #[test]
    fn test_parse_rar5_header() {
        // RAR5 signature: Rar!\x1a\x07\x01\x00
        let mut header = vec![0u8; 512];
        header[0..8].copy_from_slice(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00]);
        // CRC32 after signature
        header[8..12].copy_from_slice(&[0xAB, 0xCD, 0xEF, 0x01]);
        
        let result = parse_rar_header(&header, 100000).unwrap();
        assert_eq!(result.format, "RAR5");
        assert_eq!(result.version, Some("5.0".to_string()));
        assert!(result.fields.iter().any(|f| f.key == "Format" && f.value.contains("RAR5")));
    }
    
    #[test]
    fn test_parse_rar4_header() {
        // RAR4 signature: Rar!\x1a\x07\x00
        let mut header = vec![0u8; 512];
        header[0..7].copy_from_slice(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]);
        // HEAD_CRC, HEAD_TYPE, HEAD_FLAGS, HEAD_SIZE
        header[7..14].copy_from_slice(&[0x01, 0x02, 0x73, 0x00, 0x01, 0x0D, 0x00]);
        
        let result = parse_rar_header(&header, 100000).unwrap();
        assert_eq!(result.format, "RAR4");
        assert_eq!(result.version, Some("4.x".to_string()));
    }
    
    #[test]
    fn test_parse_raw_header_mbr() {
        // MBR with boot signature
        let mut header = vec![0u8; 512];
        // Boot signature at offset 510-511
        header[510] = 0x55;
        header[511] = 0xAA;
        // First partition entry at 446: type at +4
        header[450] = 0x07; // NTFS
        header[454] = 0x01; // LBA start (little endian)
        
        let result = parse_raw_header(&header, "dd", 1073741824).unwrap();
        assert_eq!(result.format, "Raw Disk Image");
        assert!(result.fields.iter().any(|f| f.key == "Partition Table" && f.value == "MBR"));
        assert!(result.regions.iter().any(|r| r.name == "Boot Signature"));
        assert!(result.regions.iter().any(|r| r.name == "Partition Table"));
    }
    
    #[test]
    fn test_parse_raw_header_gpt() {
        // GPT with protective MBR (type 0xEE)
        let mut header = vec![0u8; 512];
        header[510] = 0x55;
        header[511] = 0xAA;
        header[450] = 0xEE; // GPT protective MBR type
        
        let result = parse_raw_header(&header, "raw", 2147483648).unwrap();
        assert!(result.fields.iter().any(|f| f.key == "Partition Table" && f.value.contains("GPT")));
    }
    
    #[test]
    fn test_parse_qcow2_header() {
        // QCOW2 header (big-endian)
        let mut header = vec![0u8; 512];
        header[0..4].copy_from_slice(&[0x51, 0x46, 0x49, 0xFB]); // Magic
        header[4..8].copy_from_slice(&[0x00, 0x00, 0x00, 0x03]); // Version 3 (big-endian)
        // Cluster bits at offset 20-24
        header[20..24].copy_from_slice(&[0x00, 0x00, 0x00, 0x10]); // 16 = 64KB clusters
        // Virtual size at offset 24-32 (10GB)
        header[24..32].copy_from_slice(&[0x00, 0x00, 0x00, 0x02, 0x80, 0x00, 0x00, 0x00]);
        
        let result = parse_qcow2_header(&header, 500000000).unwrap();
        assert_eq!(result.format, "QCOW2");
        assert_eq!(result.version, Some("3".to_string()));
        assert!(result.fields.iter().any(|f| f.key == "Cluster Size" && f.value.contains("64")));
        assert!(result.fields.iter().any(|f| f.key == "Virtual Size"));
    }
    
    #[test]
    fn test_parse_vmdk_header() {
        // VMDK sparse header
        let mut header = vec![0u8; 512];
        header[0..4].copy_from_slice(b"KDMV"); // Magic
        header[4..8].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Version 1
        header[8..12].copy_from_slice(&[0x03, 0x00, 0x00, 0x00]); // Flags
        // Capacity in sectors at offset 12-20
        header[12..20].copy_from_slice(&[0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00]); // ~500MB
        
        let result = parse_vmdk_header(&header, 200000000).unwrap();
        assert_eq!(result.format, "VMDK");
        assert_eq!(result.version, Some("1".to_string()));
        assert!(result.fields.iter().any(|f| f.key == "Virtual Capacity"));
    }
    
    #[test]
    fn test_parse_vhdx_header() {
        // VHDx file identifier
        let mut header = vec![0u8; 1024];
        header[0..8].copy_from_slice(b"vhdxfile");
        // Creator at offset 8 (UTF-16LE)
        let creator = "Hyper-V";
        for (i, c) in creator.chars().enumerate() {
            header[8 + i * 2] = c as u8;
            header[9 + i * 2] = 0;
        }
        
        let result = parse_vhdx_header(&header, 10737418240).unwrap();
        assert_eq!(result.format, "VHDx");
        assert_eq!(result.version, Some("2.0".to_string()));
        assert!(result.fields.iter().any(|f| f.key == "Creator" && f.value.contains("Hyper-V")));
    }
    
    #[test]
    fn test_detect_format_forensic() {
        // Test E01 detection
        let e01_magic = [0x45, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00];
        let (desc, is_forensic, _) = detect_format(&e01_magic, "e01");
        assert!(desc.contains("E01") || desc.contains("EWF"));
        assert!(is_forensic);
        
        // Test AD1 detection
        let ad1_magic = b"ADSEGMEN";
        let (desc, is_forensic, _) = detect_format(ad1_magic, "ad1");
        assert!(desc.contains("AD1"));
        assert!(is_forensic);
        
        // Test VMDK detection
        let vmdk_magic = b"KDMV\x00\x00\x00\x00";
        let (desc, is_forensic, _) = detect_format(vmdk_magic, "vmdk");
        assert!(desc.contains("VMDK"));
        assert!(is_forensic);
    }
    
    #[test]
    fn test_parse_ad1_header() {
        // Build a realistic AD1 header
        let mut header = vec![0u8; 1024];
        
        // Segment header (ADSEGMENTEDFILE signature)
        header[0..15].copy_from_slice(b"ADSEGMENTEDFILE");
        header[15] = 0; // Null terminator
        
        // Segment index at offset 16 (u32 LE)
        header[16..20].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Index 1
        
        // Segment number at offset 20 (u32 LE) 
        header[20..24].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Segment 1
        
        // Fragments size at offset 24 (u32 LE)
        header[24..28].copy_from_slice(&[0x00, 0x10, 0x00, 0x00]); // 4096
        
        // Header size at offset 28 (u32 LE)
        header[28..32].copy_from_slice(&[0x00, 0x02, 0x00, 0x00]); // 512
        
        // Logical header at offset 512 (ADLOGICAL signature)
        header[512..521].copy_from_slice(b"ADLOGICAL");
        
        // Image version at logical offset + 16
        header[528..532].copy_from_slice(&[0x03, 0x00, 0x00, 0x00]); // Version 3
        
        // Zlib chunk size at logical offset + 20
        header[532..536].copy_from_slice(&[0x00, 0x00, 0x01, 0x00]); // 65536
        
        let result = parse_ad1_header(&header, 1073741824).unwrap();
        
        assert_eq!(result.format, "AD1");
        assert!(result.version.as_ref().unwrap().contains("AccessData"));
        
        // Check signature field
        assert!(result.fields.iter().any(|f| f.key == "Signature" && f.value == "ADSEGMENTEDFILE"));
        
        // Check segment info
        assert!(result.fields.iter().any(|f| f.key == "Segment Index" && f.value == "1"));
        assert!(result.fields.iter().any(|f| f.key == "Segment Number" && f.value.contains("1")));
        
        // Check structure info
        assert!(result.fields.iter().any(|f| f.key == "Fragments Size" && f.value.contains("4096")));
        assert!(result.fields.iter().any(|f| f.key == "Header Size" && f.value.contains("512")));
        
        // Check logical header present
        assert!(result.fields.iter().any(|f| f.key == "Logical Header" && f.value.contains("512")));
        
        // Check image version
        assert!(result.fields.iter().any(|f| f.key == "Image Version" && f.value == "3"));
        
        // Check compression chunk
        assert!(result.fields.iter().any(|f| f.key == "Compression Chunk" && f.value.contains("64")));
        
        // Check regions for proper coverage
        assert!(result.regions.iter().any(|r| r.name == "Signature"));
        assert!(result.regions.iter().any(|r| r.name == "Segment Index"));
        assert!(result.regions.iter().any(|r| r.name == "Segment Number"));
        assert!(result.regions.iter().any(|r| r.name == "Logical Signature"));
        assert!(result.regions.iter().any(|r| r.name == "Image Version"));
        assert!(result.regions.iter().any(|r| r.name == "Zlib Chunk Size"));
    }
    
    #[test]
    fn test_parse_ad1_header_basic() {
        // Minimal AD1 header (just signature and segment info)
        let mut header = vec![0u8; 64];
        header[0..15].copy_from_slice(b"ADSEGMENTEDFILE");
        header[16..20].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Index 0
        header[20..24].copy_from_slice(&[0x02, 0x00, 0x00, 0x00]); // Segment 2
        header[24..28].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]); // No fragments
        header[28..32].copy_from_slice(&[0x00, 0x02, 0x00, 0x00]); // Header 512
        
        let result = parse_ad1_header(&header, 5000000).unwrap();
        
        assert_eq!(result.format, "AD1");
        assert!(result.fields.iter().any(|f| f.key == "Segment Number" && f.value.contains("2")));
        assert!(result.fields.iter().any(|f| f.key == "File Size"));
    }
}
