// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 header parser for hex view metadata

use crate::common::format_size;
use crate::containers::ContainerError;
use crate::viewer::types::{HeaderRegion, MetadataField, ParsedMetadata};

fn checked_slice(data: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    let end = offset.checked_add(len)?;
    data.get(offset..end)
}

fn read_le_u32_at(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        checked_slice(data, offset, 4)?.try_into().ok()?,
    ))
}

fn read_le_u64_at(data: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        checked_slice(data, offset, 8)?.try_into().ok()?,
    ))
}

fn bounded_region_end(header: &[u8], start: usize, len: usize) -> u64 {
    start
        .checked_add(len)
        .map(|end| end.min(header.len()) as u64)
        .unwrap_or(header.len() as u64)
}

/// Parse AD1 (AccessData Logical Image) header and extract metadata with regions
pub fn parse_ad1_header(header: &[u8], file_size: u64) -> Result<ParsedMetadata, ContainerError> {
    let mut fields = vec![];
    let mut regions = vec![];

    // Signature region (full 16 bytes including null terminator)
    regions.push(HeaderRegion::new(
        0,
        bounded_region_end(header, 0, 16),
        "Signature",
        "region-signature",
        "ADSEGMENTEDFILE signature (15 bytes + null)",
    ));

    // Extract and validate signature
    let signature = if let Some(signature) = checked_slice(header, 0, 15) {
        String::from_utf8_lossy(signature).to_string()
    } else {
        String::from_utf8_lossy(checked_slice(header, 0, header.len().min(8)).unwrap_or_default())
            .to_string()
    };

    fields.push(MetadataField::new(
        "Format",
        "AD1 (AccessData Logical Image)",
        "Format",
    ));

    fields.push(MetadataField::new("Signature", signature, "Format"));

    // Segment Index at offset 16 (u32 LE)
    if let Some(segment_index) = read_le_u32_at(header, 16) {
        regions.push(HeaderRegion::new(
            16,
            20,
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
    if let Some(segment_number) = read_le_u32_at(header, 20) {
        regions.push(HeaderRegion::new(
            20,
            24,
            "Segment Number",
            "region-version",
            "Segment number (1=.ad1, 2=.ad2, etc.)",
        ));

        let segment_ext = if segment_number > 0 {
            format!("{} (.ad{})", segment_number, segment_number)
        } else {
            "0 (Unknown)".to_string()
        };

        fields.push(MetadataField::new("Segment Number", segment_ext, "Segment"));
    }

    // Fragments Size at offset 24 (u32 LE)
    if let Some(fragments_size) = read_le_u32_at(header, 24) {
        regions.push(HeaderRegion::new(
            24,
            28,
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
    if let Some(header_size) = read_le_u32_at(header, 28) {
        regions.push(HeaderRegion::new(
            28,
            32,
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
    if let Some(logical_sig_bytes) = checked_slice(header, logical_offset, 9) {
        let logical_sig = String::from_utf8_lossy(logical_sig_bytes).to_string();
        if logical_sig == "ADLOGICAL" {
            regions.push(HeaderRegion::new(
                logical_offset as u64,
                bounded_region_end(header, logical_offset, 16),
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
            if let Some(image_version) = read_le_u32_at(header, logical_offset + 16) {
                regions.push(HeaderRegion::new(
                    (logical_offset + 16) as u64,
                    bounded_region_end(header, logical_offset + 16, 4),
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
            if let Some(zlib_chunk_size) = read_le_u32_at(header, logical_offset + 20) {
                regions.push(HeaderRegion::new(
                    (logical_offset + 20) as u64,
                    bounded_region_end(header, logical_offset + 20, 4),
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
            if let Some(first_item_addr) = read_le_u64_at(header, logical_offset + 40) {
                regions.push(HeaderRegion::new(
                    (logical_offset + 40) as u64,
                    bounded_region_end(header, logical_offset + 40, 8),
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
    if volume_offset < header.len() {
        let mut label_end = volume_offset;
        let label_limit = volume_offset.saturating_add(64).min(header.len());
        while label_end < label_limit && header[label_end] != 0 {
            label_end += 1;
        }
        if label_end > volume_offset {
            let volume_label = String::from_utf8_lossy(
                checked_slice(header, volume_offset, label_end - volume_offset).unwrap_or_default(),
            )
            .to_string();
            if !volume_label.is_empty()
                && volume_label
                    .chars()
                    .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
            {
                regions.push(HeaderRegion::new(
                    volume_offset as u64,
                    label_end as u64,
                    "Volume Label",
                    "region-name",
                    "Source volume label",
                ));

                fields.push(MetadataField::new("Volume Label", volume_label, "Source"));
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal AD1 header with ADSEGMENTEDFILE signature + null + segment fields
    fn make_ad1_header(
        segment_index: u32,
        segment_number: u32,
        fragments_size: u32,
        header_size: u32,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        // Signature: "ADSEGMENTEDFILE" (15 bytes) + null terminator
        buf.extend_from_slice(b"ADSEGMENTEDFILE\0");
        // Segment Index (offset 16, u32 LE)
        buf.extend_from_slice(&segment_index.to_le_bytes());
        // Segment Number (offset 20, u32 LE)
        buf.extend_from_slice(&segment_number.to_le_bytes());
        // Fragments Size (offset 24, u32 LE)
        buf.extend_from_slice(&fragments_size.to_le_bytes());
        // Header Size (offset 28, u32 LE)
        buf.extend_from_slice(&header_size.to_le_bytes());
        // Pad one extra byte so header.len() > 32 passes
        buf.push(0);
        buf
    }

    fn make_exact_ad1_header(
        segment_index: u32,
        segment_number: u32,
        fragments_size: u32,
        header_size: u32,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"ADSEGMENTEDFILE\0");
        buf.extend_from_slice(&segment_index.to_le_bytes());
        buf.extend_from_slice(&segment_number.to_le_bytes());
        buf.extend_from_slice(&fragments_size.to_le_bytes());
        buf.extend_from_slice(&header_size.to_le_bytes());
        buf
    }

    /// Build a full AD1 header with ADLOGICAL section at offset 512
    fn make_full_ad1_header(image_version: u32, zlib_chunk: u32, first_item: u64) -> Vec<u8> {
        let mut buf = make_ad1_header(0, 1, 256, 1024);
        // Pad to offset 512
        buf.resize(512, 0);
        // "ADLOGICAL" signature (9 bytes) + padding to 16 bytes
        buf.extend_from_slice(b"ADLOGICAL\0\0\0\0\0\0\0");
        // Image version (offset 512+16 = 528, u32 LE)
        buf.extend_from_slice(&image_version.to_le_bytes());
        // Zlib chunk size (offset 512+20 = 532, u32 LE)
        buf.extend_from_slice(&zlib_chunk.to_le_bytes());
        // Pad to offset 512+40 = 552
        buf.resize(552, 0);
        // First item address (offset 552, u64 LE)
        buf.extend_from_slice(&first_item.to_le_bytes());
        // Pad to 512 + 64 + 1 = enough to pass all checks
        buf.resize(600, 0);
        buf
    }

    #[test]
    fn ad1_basic_header_parses() {
        let header = make_ad1_header(0, 1, 256, 1024);
        let result = parse_ad1_header(&header, 1_000_000).unwrap();
        assert_eq!(result.format, "AD1");
        assert_eq!(result.version.as_deref(), Some("AccessData Logical Image"));
    }

    #[test]
    fn ad1_extracts_signature() {
        let header = make_ad1_header(0, 1, 0, 0);
        let result = parse_ad1_header(&header, 0).unwrap();
        let sig_field = result.fields.iter().find(|f| f.key == "Signature").unwrap();
        assert_eq!(sig_field.value, "ADSEGMENTEDFILE");
    }

    #[test]
    fn ad1_extracts_segment_index() {
        let header = make_ad1_header(42, 1, 0, 0);
        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Segment Index")
            .unwrap();
        assert_eq!(field.value, "42");
    }

    #[test]
    fn ad1_extracts_segment_number() {
        let header = make_ad1_header(0, 3, 0, 0);
        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Segment Number")
            .unwrap();
        assert!(field.value.contains("3"));
        assert!(field.value.contains(".ad3"));
    }

    #[test]
    fn ad1_segment_number_zero() {
        let header = make_ad1_header(0, 0, 0, 0);
        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Segment Number")
            .unwrap();
        assert!(field.value.contains("Unknown"));
    }

    #[test]
    fn ad1_extracts_fragments_size() {
        let header = make_ad1_header(0, 1, 512, 0);
        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Fragments Size")
            .unwrap();
        assert_eq!(field.value, "512 bytes");
    }

    #[test]
    fn ad1_extracts_header_size() {
        let header = make_ad1_header(0, 1, 0, 2048);
        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Header Size")
            .unwrap();
        assert_eq!(field.value, "2048 bytes");
    }

    #[test]
    fn ad1_exact_length_fixed_fields_are_parsed() {
        let header = make_exact_ad1_header(42, 3, 512, 2048);
        let result = parse_ad1_header(&header, 0).unwrap();

        assert_eq!(
            result
                .fields
                .iter()
                .find(|f| f.key == "Segment Index")
                .unwrap()
                .value,
            "42"
        );
        assert!(result
            .fields
            .iter()
            .find(|f| f.key == "Segment Number")
            .unwrap()
            .value
            .contains(".ad3"));
        assert_eq!(
            result
                .fields
                .iter()
                .find(|f| f.key == "Fragments Size")
                .unwrap()
                .value,
            "512 bytes"
        );
        assert_eq!(
            result
                .fields
                .iter()
                .find(|f| f.key == "Header Size")
                .unwrap()
                .value,
            "2048 bytes"
        );
    }

    #[test]
    fn ad1_has_signature_region() {
        let header = make_ad1_header(0, 1, 0, 0);
        let result = parse_ad1_header(&header, 0).unwrap();
        let sig_region = result
            .regions
            .iter()
            .find(|r| r.name == "Signature")
            .unwrap();
        assert_eq!(sig_region.start, 0);
        assert_eq!(sig_region.end, 16);
        assert_eq!(sig_region.color_class, "region-signature");
    }

    #[test]
    fn ad1_file_size_included() {
        let header = make_ad1_header(0, 1, 0, 0);
        let result = parse_ad1_header(&header, 5_000_000).unwrap();
        let field = result.fields.iter().find(|f| f.key == "File Size").unwrap();
        assert!(!field.value.is_empty());
    }

    #[test]
    fn ad1_logical_header_detection() {
        let header = make_full_ad1_header(3, 65536, 0x1000);
        let result = parse_ad1_header(&header, 10_000_000).unwrap();

        let logical = result
            .fields
            .iter()
            .find(|f| f.key == "Logical Header")
            .unwrap();
        assert_eq!(logical.value, "Present at offset 512");

        let version = result
            .fields
            .iter()
            .find(|f| f.key == "Image Version")
            .unwrap();
        assert_eq!(version.value, "3");
    }

    #[test]
    fn ad1_logical_zlib_chunk() {
        let header = make_full_ad1_header(3, 65536, 0x1000);
        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Compression Chunk")
            .unwrap();
        assert!(!field.value.is_empty());
    }

    #[test]
    fn ad1_logical_first_item_offset() {
        let header = make_full_ad1_header(3, 65536, 0xABCD);
        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "First Item Offset")
            .unwrap();
        assert!(field.value.contains("ABCD"));
    }

    #[test]
    fn ad1_exact_length_logical_first_item_offset_is_parsed() {
        let mut header = make_exact_ad1_header(0, 1, 256, 1024);
        header.resize(512, 0);
        header.extend_from_slice(b"ADLOGICAL\0\0\0\0\0\0\0");
        header.extend_from_slice(&3u32.to_le_bytes());
        header.extend_from_slice(&65536u32.to_le_bytes());
        header.resize(552, 0);
        header.extend_from_slice(&0xABCDu64.to_le_bytes());

        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "First Item Offset")
            .unwrap();

        assert!(field.value.contains("ABCD"));
    }

    #[test]
    fn ad1_volume_label_parses_without_full_sixty_four_byte_field() {
        let mut header = make_exact_ad1_header(0, 1, 256, 1024);
        header.resize(680, 0);
        header.extend_from_slice(b"Evidence\0");

        let result = parse_ad1_header(&header, 0).unwrap();
        let field = result
            .fields
            .iter()
            .find(|f| f.key == "Volume Label")
            .unwrap();

        assert_eq!(field.value, "Evidence");
    }

    #[test]
    fn ad1_checked_helpers_reject_overflowing_ranges() {
        assert!(checked_slice(b"abc", usize::MAX, 1).is_none());
        assert!(read_le_u32_at(b"abc", usize::MAX).is_none());
        assert!(read_le_u64_at(b"abc", usize::MAX).is_none());
    }

    #[test]
    fn ad1_no_logical_header_when_missing() {
        let header = make_ad1_header(0, 1, 0, 0);
        let result = parse_ad1_header(&header, 0).unwrap();
        assert!(!result.fields.iter().any(|f| f.key == "Logical Header"));
    }

    #[test]
    fn ad1_short_header_graceful() {
        // Only 10 bytes — shorter than 15 so uses .min(8) branch
        let header = b"ADSEGMENTE".to_vec();
        let result = parse_ad1_header(&header, 100).unwrap();
        assert_eq!(result.format, "AD1");
        // Short header: uses header[0..header.len().min(8)] = "ADSEGMEN"
        let sig = result.fields.iter().find(|f| f.key == "Signature").unwrap();
        assert!(sig.value.starts_with("ADSEGMEN"));
        let sig_region = result
            .regions
            .iter()
            .find(|r| r.name == "Signature")
            .unwrap();
        assert_eq!(sig_region.end, header.len() as u64);
    }

    #[test]
    fn ad1_empty_header() {
        let header = vec![];
        let result = parse_ad1_header(&header, 0).unwrap();
        assert_eq!(result.format, "AD1");
        // Should have format field at least
        assert!(result.fields.iter().any(|f| f.key == "Format"));
        let sig_region = result
            .regions
            .iter()
            .find(|r| r.name == "Signature")
            .unwrap();
        assert_eq!(sig_region.start, 0);
        assert_eq!(sig_region.end, 0);
    }

    #[test]
    fn ad1_all_region_offsets_valid() {
        let header = make_full_ad1_header(3, 65536, 0x1000);
        let result = parse_ad1_header(&header, 1_000_000).unwrap();
        for region in &result.regions {
            assert!(
                region.start <= region.end,
                "Region {} has start > end",
                region.name
            );
            assert!(
                region.end <= header.len() as u64 || region.name == "Volume Data",
                "Region {} end {} exceeds header len {}",
                region.name,
                region.end,
                header.len()
            );
        }
    }
}
