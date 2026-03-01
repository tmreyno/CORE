// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Metadata extraction and UI conversion functions for EWF format analysis.
//!
//! Handles parsing of case info from header/header2 sections, hash extraction
//! from hash/digest sections, error sector parsing, and conversion of parsed
//! data into `ParsedMetadata` for the hex viewer UI.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::common::format_size;
use crate::containers::ContainerError;
use crate::viewer::{HeaderRegion, MetadataField, ParsedMetadata};

use super::types::{
    EwfCaseInfo, EwfDetailedInfo, EwfErrorEntry, EwfHashInfo, EwfSectionHeader, EwfVariant,
    SECTION_HEADER_SIZE,
};

// ============================================================================
// Section Data Extraction
// ============================================================================

/// Parse case information from header/header2 sections
pub(super) fn parse_case_info(
    file: &mut File,
    sections: &[EwfSectionHeader],
) -> Result<EwfCaseInfo, ContainerError> {
    let mut case_info = EwfCaseInfo::default();

    // Prefer header2 (UTF-16) over header (ASCII)
    let header_section = sections
        .iter()
        .find(|s| s.section_type == "header2")
        .or_else(|| sections.iter().find(|s| s.section_type == "header"));

    if let Some(section) = header_section {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;
        let data_size = section
            .section_size
            .saturating_sub(SECTION_HEADER_SIZE as u64) as usize;

        // Limit to reasonable size
        let read_size = data_size.min(65536);

        file.seek(SeekFrom::Start(data_offset))?;

        let mut data = vec![0u8; read_size];
        if file.read_exact(&mut data).is_err() {
            return Ok(case_info);
        }

        // Check if data is zlib compressed (starts with 0x78)
        // Zlib header: first byte is 0x78 (compression method + flags),
        // second byte varies based on compression level (0x01, 0x5E, 0x9C, 0xDA)
        if data.len() >= 2 && data[0] == 0x78 {
            // Try to decompress
            if let Ok(decompressed) = decompress_zlib(&data) {
                case_info = parse_header_content(&decompressed, section.section_type == "header2");
            }
        } else {
            // Try parsing as-is
            case_info = parse_header_content(&data, section.section_type == "header2");
        }
    }

    Ok(case_info)
}

/// Parse decompressed header content
fn parse_header_content(data: &[u8], is_utf16: bool) -> EwfCaseInfo {
    let mut case_info = EwfCaseInfo::default();

    // Convert to string
    let content = if is_utf16 {
        // UTF-16 LE decoding
        let utf16_data: Vec<u16> = data
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(u16::from_le_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect();
        String::from_utf16_lossy(&utf16_data)
    } else {
        String::from_utf8_lossy(data).to_string()
    };

    // Parse key=value pairs
    let fields = parse_ewf_header_fields(&content);

    // Map known fields
    case_info.description = fields.get("a").cloned();
    case_info.case_number = fields.get("c").cloned();
    case_info.evidence_number = fields.get("n").cloned();
    case_info.examiner = fields.get("e").cloned();
    case_info.notes = fields.get("t").cloned();
    case_info.acquisition_date = fields.get("m").cloned();
    case_info.system_date = fields.get("u").cloned();
    case_info.acquisition_software = fields.get("av").cloned();
    case_info.acquisition_os = fields.get("ov").cloned();
    case_info.device_model = fields.get("md").cloned();
    case_info.device_serial = fields.get("sn").cloned();
    case_info.device_label = fields.get("l").cloned();

    // Parse total bytes if present
    if let Some(tb) = fields.get("tb") {
        case_info.device_total_bytes = tb.parse().ok();
    }

    case_info
}

/// Parse EWF header key=value fields
fn parse_ewf_header_fields(content: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();

    // Split by tabs and newlines
    for line in content.split(['\n', '\r']) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Look for key=value or key\tvalue patterns
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            if !key.is_empty() && !value.is_empty() {
                fields.insert(key.to_string(), value.to_string());
            }
        } else if let Some(tab_pos) = line.find('\t') {
            let key = line[..tab_pos].trim();
            let value = line[tab_pos + 1..].trim();
            if !key.is_empty() && !value.is_empty() {
                fields.insert(key.to_string(), value.to_string());
            }
        }
    }

    fields
}

/// Parse hash information from hash/digest sections
pub(super) fn parse_hash_info(
    file: &mut File,
    sections: &[EwfSectionHeader],
) -> Result<EwfHashInfo, ContainerError> {
    let mut hash_info = EwfHashInfo::default();

    // Check digest section first (preferred, has more hashes)
    if let Some(section) = sections.iter().find(|s| s.section_type == "digest") {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;

        file.seek(SeekFrom::Start(data_offset))?;

        let mut data = [0u8; 72]; // MD5(16) + SHA1(20) + SHA256(32) + checksum(4)
        let bytes_read = file.read(&mut data).unwrap_or(0);

        if bytes_read >= 16 {
            hash_info.md5 = Some(super::bytes_to_hex(&data[0..16]));
        }
        if bytes_read >= 36 {
            hash_info.sha1 = Some(super::bytes_to_hex(&data[16..36]));
        }
        if bytes_read >= 68 {
            // Check if SHA256 is present (not all zeros)
            if data[36..68].iter().any(|&b| b != 0) {
                hash_info.sha256 = Some(super::bytes_to_hex(&data[36..68]));
            }
        }
    }
    // Fallback to hash section (MD5 only)
    else if let Some(section) = sections.iter().find(|s| s.section_type == "hash") {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;

        file.seek(SeekFrom::Start(data_offset))?;

        let mut data = [0u8; 16];
        if file.read_exact(&mut data).is_ok() {
            hash_info.md5 = Some(super::bytes_to_hex(&data));
        }
    }

    Ok(hash_info)
}

/// Parse error information from error2 section
pub(super) fn parse_error_info(
    file: &mut File,
    sections: &[EwfSectionHeader],
) -> Result<Vec<EwfErrorEntry>, ContainerError> {
    let mut errors = Vec::new();

    if let Some(section) = sections.iter().find(|s| s.section_type == "error2") {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;

        file.seek(SeekFrom::Start(data_offset))?;

        let mut header = [0u8; 12];
        if file.read_exact(&mut header).is_err() {
            return Ok(errors);
        }

        let error_count = u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize;

        // Limit to reasonable number
        let max_errors = error_count.min(10000);

        for _ in 0..max_errors {
            let mut entry = [0u8; 8];
            if file.read_exact(&mut entry).is_err() {
                break;
            }

            let first_sector = u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
            let sector_count = u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]]);

            errors.push(EwfErrorEntry {
                first_sector,
                sector_count,
            });
        }
    }

    Ok(errors)
}

/// Simple zlib decompression using flate2
fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, ContainerError> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Format media type as string
fn format_media_type(media_type: u32) -> &'static str {
    match media_type {
        0 => "Removable",
        1 => "Fixed Disk",
        3 => "Optical Disc",
        _ => "Unknown",
    }
}

/// Format compression level
fn format_compression(level: u8) -> String {
    match level {
        0 => "None".to_string(),
        1..=9 => format!("zlib level {}", level),
        _ => format!("Unknown ({})", level),
    }
}

// ============================================================================
// Conversion to ParsedMetadata for UI Display
// ============================================================================

/// Convert EwfDetailedInfo to ParsedMetadata for the HexViewer/MetadataPanel
pub fn ewf_detailed_info_to_metadata(info: &EwfDetailedInfo) -> ParsedMetadata {
    let mut fields = Vec::new();
    let mut regions = Vec::new();

    // Find section offsets for linking metadata fields to hex positions
    let header_offset = info
        .sections
        .iter()
        .find(|s| s.section_type == "header2" || s.section_type == "header")
        .map(|s| s.file_offset);
    let volume_offset = info
        .sections
        .iter()
        .find(|s| s.section_type == "volume")
        .map(|s| s.file_offset);
    let hash_offset = info
        .sections
        .iter()
        .find(|s| s.section_type == "hash" || s.section_type == "digest")
        .map(|s| s.file_offset);

    // ---- Format Information ----
    let format_desc = match info.variant {
        EwfVariant::E01 => "E01 (Physical Image)",
        EwfVariant::L01 => "L01 (Logical Evidence)",
        EwfVariant::Ex01 => "Ex01 (Physical Image v2)",
        EwfVariant::Lx01 => "Lx01 (Logical Evidence v2)",
        EwfVariant::Unknown => "Unknown",
    };

    fields.push(MetadataField {
        key: "Format".to_string(),
        value: format_desc.to_string(),
        category: "Format".to_string(),
        linked_region: Some("signature".to_string()),
        source_offset: Some(0),
    });

    fields.push(MetadataField {
        key: "EWF Version".to_string(),
        value: format!("v{}", info.version),
        category: "Format".to_string(),
        linked_region: Some("signature".to_string()),
        source_offset: Some(0),
    });

    fields.push(MetadataField {
        key: "Segment Number".to_string(),
        value: format!("{}", info.segment_number),
        category: "Format".to_string(),
        linked_region: Some("segment".to_string()),
        source_offset: Some(8),
    });

    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(info.file_size),
        category: "Format".to_string(),
        ..Default::default()
    });

    fields.push(MetadataField {
        key: "Sections Found".to_string(),
        value: format!("{}", info.sections.len()),
        category: "Format".to_string(),
        ..Default::default()
    });

    // ---- Header Regions ----

    // Signature region
    regions.push(HeaderRegion {
        start: 0,
        end: 8,
        name: "EWF Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: format!("{} file signature", info.variant),
    });

    // Segment info region
    regions.push(HeaderRegion {
        start: 8,
        end: 13,
        name: "Segment Info".to_string(),
        color_class: "region-segment".to_string(),
        description: format!("Segment {} identifier", info.segment_number),
    });

    // Section header regions
    for section in &info.sections {
        let header_end = section.file_offset + SECTION_HEADER_SIZE as u64;

        // Section type field (16 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset,
            end: section.file_offset + 16,
            name: format!("Section: {}", section.section_type),
            color_class: "region-section-type".to_string(),
            description: format!("{} section type identifier", section.section_type),
        });

        // Next offset field (8 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset + 16,
            end: section.file_offset + 24,
            name: "Next Offset".to_string(),
            color_class: "region-offset".to_string(),
            description: format!("Next section at 0x{:X}", section.next_offset),
        });

        // Section size field (8 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset + 24,
            end: section.file_offset + 32,
            name: "Section Size".to_string(),
            color_class: "region-offset".to_string(),
            description: format!("{} bytes", section.section_size),
        });

        // Padding (40 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset + 32,
            end: section.file_offset + 72,
            name: "Padding".to_string(),
            color_class: "region-reserved".to_string(),
            description: "Reserved padding bytes".to_string(),
        });

        // Checksum (4 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset + 72,
            end: header_end,
            name: "Checksum".to_string(),
            color_class: "region-checksum".to_string(),
            description: format!("Adler-32: 0x{:08X}", section.checksum),
        });

        // Section data region (if not too large)
        if section.section_size > SECTION_HEADER_SIZE as u64 {
            let data_start = header_end;
            let data_end = section.file_offset + section.section_size;

            let data_class = match section.section_type.as_str() {
                "header" | "header2" => "region-metadata",
                "volume" | "disk" => "region-metadata",
                "sectors" | "data" => "region-data",
                "table" | "table2" => "region-offset",
                "hash" | "digest" => "region-hash",
                "error2" => "region-error",
                _ => "region-data",
            };

            regions.push(HeaderRegion {
                start: data_start,
                end: data_end.min(data_start + 1024), // Limit displayed region
                name: format!("{} Data", section.section_type),
                color_class: data_class.to_string(),
                description: format!(
                    "{} section data ({} bytes)",
                    section.section_type,
                    section.section_size - SECTION_HEADER_SIZE as u64
                ),
            });
        }
    }

    // ---- Case Information ----
    if let Some(ref desc) = info.case_info.description {
        fields.push(MetadataField {
            key: "Description".to_string(),
            value: desc.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(ref case_num) = info.case_info.case_number {
        fields.push(MetadataField {
            key: "Case Number".to_string(),
            value: case_num.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(ref evidence_num) = info.case_info.evidence_number {
        fields.push(MetadataField {
            key: "Evidence Number".to_string(),
            value: evidence_num.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(ref examiner) = info.case_info.examiner {
        fields.push(MetadataField {
            key: "Examiner".to_string(),
            value: examiner.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(ref notes) = info.case_info.notes {
        fields.push(MetadataField {
            key: "Notes".to_string(),
            value: notes.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(ref acq_date) = info.case_info.acquisition_date {
        fields.push(MetadataField {
            key: "Acquisition Date".to_string(),
            value: acq_date.clone(),
            category: "Acquisition".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(ref acq_sw) = info.case_info.acquisition_software {
        fields.push(MetadataField {
            key: "Acquisition Software".to_string(),
            value: acq_sw.clone(),
            category: "Acquisition".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(ref acq_os) = info.case_info.acquisition_os {
        fields.push(MetadataField {
            key: "Acquisition OS".to_string(),
            value: acq_os.clone(),
            category: "Acquisition".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    // ---- Device Information ----
    if let Some(ref model) = info.case_info.device_model {
        fields.push(MetadataField {
            key: "Device Model".to_string(),
            value: model.clone(),
            category: "Device".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(ref serial) = info.case_info.device_serial {
        fields.push(MetadataField {
            key: "Serial Number".to_string(),
            value: serial.clone(),
            category: "Device".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    if let Some(total_bytes) = info.case_info.device_total_bytes {
        fields.push(MetadataField {
            key: "Total Bytes".to_string(),
            value: format_size(total_bytes),
            category: "Device".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }

    // ---- Volume Information ----
    if let Some(ref volume) = info.volume {
        fields.push(MetadataField {
            key: "Chunk Count".to_string(),
            value: format!("{}", volume.chunk_count),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });

        fields.push(MetadataField {
            key: "Sectors per Chunk".to_string(),
            value: format!("{}", volume.sectors_per_chunk),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });

        fields.push(MetadataField {
            key: "Bytes per Sector".to_string(),
            value: format!("{}", volume.bytes_per_sector),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });

        fields.push(MetadataField {
            key: "Total Sectors".to_string(),
            value: format!("{}", volume.sector_count),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });

        let image_size = volume.sector_count * volume.bytes_per_sector as u64;
        fields.push(MetadataField {
            key: "Image Size".to_string(),
            value: format_size(image_size),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });

        fields.push(MetadataField {
            key: "Chunk Size".to_string(),
            value: format_size((volume.sectors_per_chunk * volume.bytes_per_sector) as u64),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });

        fields.push(MetadataField {
            key: "Media Type".to_string(),
            value: format_media_type(volume.media_type).to_string(),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });

        fields.push(MetadataField {
            key: "Compression".to_string(),
            value: format_compression(volume.compression_level),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });

        if volume.chs_cylinders > 0 {
            fields.push(MetadataField {
                key: "CHS Geometry".to_string(),
                value: format!(
                    "{} / {} / {}",
                    volume.chs_cylinders, volume.chs_heads, volume.chs_sectors
                ),
                category: "Volume".to_string(),
                linked_region: Some("volume".to_string()),
                source_offset: volume_offset,
            });
        }

        if let Some(ref guid) = volume.guid {
            fields.push(MetadataField {
                key: "GUID".to_string(),
                value: guid.clone(),
                category: "Volume".to_string(),
                linked_region: Some("volume".to_string()),
                source_offset: volume_offset,
            });
        }
    }

    // ---- Hash Information ----
    if let Some(ref md5) = info.hashes.md5 {
        fields.push(MetadataField {
            key: "MD5".to_string(),
            value: md5.clone(),
            category: "Hashes".to_string(),
            linked_region: Some("hash".to_string()),
            source_offset: hash_offset,
        });
    }

    if let Some(ref sha1) = info.hashes.sha1 {
        fields.push(MetadataField {
            key: "SHA1".to_string(),
            value: sha1.clone(),
            category: "Hashes".to_string(),
            linked_region: Some("hash".to_string()),
            source_offset: hash_offset,
        });
    }

    if let Some(ref sha256) = info.hashes.sha256 {
        fields.push(MetadataField {
            key: "SHA256".to_string(),
            value: sha256.clone(),
            category: "Hashes".to_string(),
            linked_region: Some("hash".to_string()),
            source_offset: hash_offset,
        });
    }

    // ---- Error Information ----
    if !info.errors.is_empty() {
        fields.push(MetadataField {
            key: "Acquisition Errors".to_string(),
            value: format!("{} error regions", info.errors.len()),
            category: "Errors".to_string(),
            ..Default::default()
        });

        // Show first few errors
        for (i, error) in info.errors.iter().take(5).enumerate() {
            fields.push(MetadataField {
                key: format!("Error Region {}", i + 1),
                value: format!(
                    "Sectors {} - {} ({} sectors)",
                    error.first_sector,
                    error.first_sector + error.sector_count - 1,
                    error.sector_count
                ),
                category: "Errors".to_string(),
                ..Default::default()
            });
        }

        if info.errors.len() > 5 {
            fields.push(MetadataField {
                key: "...".to_string(),
                value: format!("and {} more error regions", info.errors.len() - 5),
                category: "Errors".to_string(),
                ..Default::default()
            });
        }
    }

    // ---- Section List ----
    for (i, section) in info.sections.iter().enumerate() {
        fields.push(MetadataField {
            key: format!("Section {}: {}", i + 1, section.section_type),
            value: format!("{} bytes", section.section_size),
            category: "Sections".to_string(),
            linked_region: Some(section.section_type.clone()),
            source_offset: Some(section.file_offset),
        });
    }

    ParsedMetadata {
        format: info.variant.to_string(),
        version: Some(format!("EWF v{}", info.version)),
        fields,
        regions,
    }
}
