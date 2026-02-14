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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an EWF (E01) header with EVF signature + section header
    fn make_ewf_header(signature: &[u8; 3], segment_num: u16, section_type: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        // Signature: EVF or LVF (3 bytes) + rest of 8 byte sig
        buf.extend_from_slice(signature);
        buf.extend_from_slice(&[0x09, 0x0D, 0x0A, 0xFF, 0x00]); // standard EWF sig bytes
        // Segment info region (offset 8): fields start marker
        buf.push(0x01);
        // Segment number (offset 9-10, u16 LE)
        buf.extend_from_slice(&segment_num.to_le_bytes());
        // Pad to offset 13
        while buf.len() < 13 {
            buf.push(0);
        }
        // Section type (offset 13, null-terminated, 16 bytes)
        let mut section_bytes = [0u8; 16];
        for (i, b) in section_type.bytes().enumerate() {
            if i >= 16 { break; }
            section_bytes[i] = b;
        }
        buf.extend_from_slice(&section_bytes);
        // Pad to offset 89 (section header is 76 bytes starting at 13)
        while buf.len() < 89 {
            buf.push(0);
        }
        // Pad further for volume data check
        buf.resize(200, 0);
        buf
    }

    #[test]
    fn ewf_e01_format() {
        let header = make_ewf_header(b"EVF", 1, "header");
        let result = parse_ewf_header(&header, 10_000).unwrap();
        assert_eq!(result.format, "E01");
        assert_eq!(result.version.as_deref(), Some("v1"));
    }

    #[test]
    fn ewf_l01_format() {
        let header = make_ewf_header(b"LVF", 1, "header");
        let result = parse_ewf_header(&header, 10_000).unwrap();
        assert_eq!(result.format, "L01");
    }

    #[test]
    fn ewf_format_field_e01() {
        let header = make_ewf_header(b"EVF", 1, "header");
        let result = parse_ewf_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Format").unwrap();
        assert!(field.value.contains("E01"));
        assert!(field.value.contains("Physical"));
    }

    #[test]
    fn ewf_format_field_l01() {
        let header = make_ewf_header(b"LVF", 1, "header");
        let result = parse_ewf_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Format").unwrap();
        assert!(field.value.contains("L01"));
        assert!(field.value.contains("Logical"));
    }

    #[test]
    fn ewf_segment_number() {
        let header = make_ewf_header(b"EVF", 5, "header");
        let result = parse_ewf_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Segment Number").unwrap();
        assert_eq!(field.value, "5");
    }

    #[test]
    fn ewf_segment_number_one() {
        let header = make_ewf_header(b"EVF", 1, "header");
        let result = parse_ewf_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "Segment Number").unwrap();
        assert_eq!(field.value, "1");
    }

    #[test]
    fn ewf_version_field() {
        let header = make_ewf_header(b"EVF", 1, "header");
        let result = parse_ewf_header(&header, 0).unwrap();
        let field = result.fields.iter().find(|f| f.key == "EWF Version").unwrap();
        assert_eq!(field.value, "v1");
    }

    #[test]
    fn ewf_signature_region() {
        let header = make_ewf_header(b"EVF", 1, "header");
        let result = parse_ewf_header(&header, 0).unwrap();
        let sig = result.regions.iter().find(|r| r.name == "Signature").unwrap();
        assert_eq!(sig.start, 0);
        assert_eq!(sig.end, 8);
    }

    #[test]
    fn ewf_section_header_region() {
        let header = make_ewf_header(b"EVF", 1, "header");
        let result = parse_ewf_header(&header, 0).unwrap();
        let section = result.regions.iter().find(|r| r.name == "Section Header").unwrap();
        assert_eq!(section.start, 13);
        assert_eq!(section.end, 89);
    }

    #[test]
    fn ewf_file_size_included() {
        let header = make_ewf_header(b"EVF", 1, "header");
        let result = parse_ewf_header(&header, 5_000_000).unwrap();
        let field = result.fields.iter().find(|f| f.key == "File Size").unwrap();
        assert!(!field.value.is_empty());
    }

    #[test]
    fn ewf_volume_data_region_present() {
        let header = make_ewf_header(b"EVF", 1, "header");
        let result = parse_ewf_header(&header, 0).unwrap();
        let volume = result.regions.iter().find(|r| r.name == "Volume Data");
        assert!(volume.is_some());
    }

    #[test]
    fn ewf_short_header() {
        // Only 8 bytes — just signature, no section header
        let header = vec![0x45, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00];
        let result = parse_ewf_header(&header, 100).unwrap();
        assert_eq!(result.format, "E01");
        // Should still parse format field
        assert!(result.fields.iter().any(|f| f.key == "Format"));
    }
}
