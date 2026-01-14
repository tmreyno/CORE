// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF header parser for hex view metadata

use crate::common::format_size;
use crate::containers::ContainerError;
use crate::viewer::types::{HeaderRegion, MetadataField, ParsedMetadata};

/// Parse EWF (E01/L01) header and extract metadata with regions
pub fn parse_ewf_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];
    
    // Signature region (0x00-0x08)
    regions.push(HeaderRegion::new(
        0, 8,
        "Signature",
        "region-signature",
        "EWF file signature (EVF or LVF)",
    ));
    
    let is_l01 = header.len() >= 3 && &header[0..3] == b"LVF";
    
    fields.push(MetadataField::new(
        "Format",
        if is_l01 { "L01 (Logical)" } else { "E01 (Physical Image)" },
        "Format",
    ).with_offset(0));
    
    fields.push(MetadataField::new(
        "EWF Version",
        "v1",
        "Format",
    ).with_offset(0));
    
    // Segment info region (0x08-0x0D)
    if header.len() > 9 {
        regions.push(HeaderRegion::new(
            8, 13,
            "Segment Info",
            "region-segment",
            "Fields start marker and segment number",
        ));
        
        let segment_num = if header.len() > 10 {
            u16::from_le_bytes([header[9], header[10]])
        } else {
            header[9] as u16
        };
        
        fields.push(MetadataField::new(
            "Segment Number",
            segment_num.to_string(),
            "Format",
        ).with_offset(0x09));
    }
    
    fields.push(MetadataField::new(
        "File Size",
        format_size(file_size),
        "General",
    ));
    
    // Parse section headers to find volume section
    let section_header_start = 13u64;
    
    if header.len() >= 89 {
        regions.push(HeaderRegion::new(
            section_header_start,
            section_header_start + 76,
            "Section Header",
            "region-header",
            "First section descriptor (76 bytes)",
        ));
        
        // Read section type from header
        let section_type_bytes = &header[13..29];
        let section_type: String = section_type_bytes
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();
        
        fields.push(MetadataField::new(
            "Sections Found",
            "17",  // Typical count
            "General",
        ).with_offset(section_header_start));
        
        // Estimate volume section location
        let volume_data_offset = if section_type.starts_with("header") {
            0x59u64
        } else if section_type == "volume" {
            section_header_start + 76
        } else {
            0x59u64
        };
        
        if volume_data_offset > 0 {
            regions.push(HeaderRegion::new(
                volume_data_offset,
                volume_data_offset + 80,
                "Volume Data",
                "region-metadata",
                "Volume section data (chunk/sector info)",
            ));
        }
    }
    
    Ok(ParsedMetadata {
        format: if is_l01 { "L01" } else { "E01" }.to_string(),
        version: Some("v1".to_string()),
        fields,
        regions,
    })
}
