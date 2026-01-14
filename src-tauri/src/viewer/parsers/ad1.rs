// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 header parser for hex view metadata

use crate::common::format_size;
use crate::containers::ContainerError;
use crate::viewer::types::{HeaderRegion, MetadataField, ParsedMetadata};

/// Parse AD1 (AccessData Logical Image) header and extract metadata with regions
pub fn parse_ad1_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Signature region (full 16 bytes including null terminator)
    regions.push(HeaderRegion::new(
        0, 16,
        "Signature",
        "region-signature",
        "ADSEGMENTEDFILE signature (15 bytes + null)",
    ));
    
    // Extract and validate signature
    let signature = if header.len() >= 15 {
        String::from_utf8_lossy(&header[0..15]).to_string()
    } else {
        String::from_utf8_lossy(&header[0..header.len().min(8)]).to_string()
    };
    
    fields.push(MetadataField::new(
        "Format",
        "AD1 (AccessData Logical Image)",
        "Format",
    ));
    
    fields.push(MetadataField::new(
        "Signature",
        signature,
        "Format",
    ));
    
    // Segment Index at offset 16 (u32 LE)
    if header.len() > 20 {
        let segment_index = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);
        
        regions.push(HeaderRegion::new(
            16, 20,
            "Segment Index",
            "region-segment",
            "Internal segment index",
        ));
        
        fields.push(MetadataField::new(
            "Segment Index",
            format!("{}", segment_index),
            "Segment",
        ));
    }
    
    // Segment Number at offset 20 (u32 LE)
    if header.len() > 24 {
        let segment_number = u32::from_le_bytes([header[20], header[21], header[22], header[23]]);
        
        regions.push(HeaderRegion::new(
            20, 24,
            "Segment Number",
            "region-version",
            "Segment number (1=.ad1, 2=.ad2, etc.)",
        ));
        
        let segment_ext = if segment_number > 0 {
            format!("{} (.ad{})", segment_number, segment_number)
        } else {
            "0 (Unknown)".to_string()
        };
        
        fields.push(MetadataField::new(
            "Segment Number",
            segment_ext,
            "Segment",
        ));
    }
    
    // Fragments Size at offset 24 (u32 LE)
    if header.len() > 28 {
        let fragments_size = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
        
        regions.push(HeaderRegion::new(
            24, 28,
            "Fragments Size",
            "region-geometry",
            "Size of fragment table",
        ));
        
        fields.push(MetadataField::new(
            "Fragments Size",
            format!("{} bytes", fragments_size),
            "Structure",
        ));
    }
    
    // Header Size at offset 28 (u32 LE)
    if header.len() > 32 {
        let header_size = u32::from_le_bytes([header[28], header[29], header[30], header[31]]);
        
        regions.push(HeaderRegion::new(
            28, 32,
            "Header Size",
            "region-geometry",
            "Total segment header size",
        ));
        
        fields.push(MetadataField::new(
            "Header Size",
            format!("{} bytes", header_size),
            "Structure",
        ));
    }
    
    // Look for logical header signature "ADLOGICAL" at offset 512
    let logical_offset = 512usize;
    if header.len() > logical_offset + 64 {
        let logical_sig = String::from_utf8_lossy(&header[logical_offset..logical_offset + 9]).to_string();
        if logical_sig == "ADLOGICAL" {
            regions.push(HeaderRegion::new(
                logical_offset as u64,
                (logical_offset + 16) as u64,
                "Logical Signature",
                "region-signature",
                "ADLOGICAL header signature",
            ));
            
            fields.push(MetadataField::new(
                "Logical Header",
                "Present at offset 512",
                "Structure",
            ));
            
            // Image version at logical_offset + 16 (u32 LE)
            if header.len() > logical_offset + 20 {
                let image_version = u32::from_le_bytes([
                    header[logical_offset + 16],
                    header[logical_offset + 17],
                    header[logical_offset + 18],
                    header[logical_offset + 19],
                ]);
                
                regions.push(HeaderRegion::new(
                    (logical_offset + 16) as u64,
                    (logical_offset + 20) as u64,
                    "Image Version",
                    "region-version",
                    "AD1 image format version",
                ));
                
                fields.push(MetadataField::new(
                    "Image Version",
                    format!("{}", image_version),
                    "Format",
                ));
            }
            
            // Zlib chunk size at logical_offset + 20 (u32 LE)
            if header.len() > logical_offset + 24 {
                let zlib_chunk_size = u32::from_le_bytes([
                    header[logical_offset + 20],
                    header[logical_offset + 21],
                    header[logical_offset + 22],
                    header[logical_offset + 23],
                ]);
                
                regions.push(HeaderRegion::new(
                    (logical_offset + 20) as u64,
                    (logical_offset + 24) as u64,
                    "Zlib Chunk Size",
                    "region-compression",
                    "Size of compressed data chunks",
                ));
                
                fields.push(MetadataField::new(
                    "Compression Chunk",
                    format_size(zlib_chunk_size as u64),
                    "Compression",
                ));
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
                
                regions.push(HeaderRegion::new(
                    (logical_offset + 40) as u64,
                    (logical_offset + 48) as u64,
                    "First Item Addr",
                    "region-offset",
                    "Address of first file/folder item",
                ));
                
                fields.push(MetadataField::new(
                    "First Item Offset",
                    format!("0x{:X}", first_item_addr),
                    "Structure",
                ));
            }
        }
    }
    
    // Volume info at offset 0x2A8 (680)
    let volume_offset = 680usize;
    if header.len() > volume_offset + 64 {
        let mut label_end = volume_offset;
        while label_end < header.len() && label_end < volume_offset + 64 && header[label_end] != 0 {
            label_end += 1;
        }
        if label_end > volume_offset {
            let volume_label = String::from_utf8_lossy(&header[volume_offset..label_end]).to_string();
            if !volume_label.is_empty() && volume_label.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                regions.push(HeaderRegion::new(
                    volume_offset as u64,
                    label_end as u64,
                    "Volume Label",
                    "region-name",
                    "Source volume label",
                ));
                
                fields.push(MetadataField::new(
                    "Volume Label",
                    volume_label,
                    "Source",
                ));
            }
        }
    }
    
    fields.push(MetadataField::new(
        "File Size",
        format_size(file_size),
        "General",
    ));
    
    Ok(ParsedMetadata {
        format: "AD1".to_string(),
        version: Some("AccessData Logical Image".to_string()),
        fields,
        regions,
    })
}
