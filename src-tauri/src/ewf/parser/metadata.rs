// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Viewer metadata conversion for EWF files.
//!
//! Contains only the viewer-coupled function that cannot be in the ffx-ewf crate.
//! All parsing functions are in ffx-ewf::parser::metadata.

use crate::common::format_size;
use crate::viewer::{HeaderRegion, MetadataField, ParsedMetadata};

use super::types::{EwfDetailedInfo, EwfVariant, SECTION_HEADER_SIZE};

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

fn checked_section_offset(base: u64, delta: u64) -> Option<u64> {
    base.checked_add(delta)
}

fn checked_section_region(
    base: u64,
    start_delta: u64,
    end_delta: u64,
    name: String,
    color_class: &str,
    description: String,
) -> Option<HeaderRegion> {
    Some(HeaderRegion {
        start: checked_section_offset(base, start_delta)?,
        end: checked_section_offset(base, end_delta)?,
        name,
        color_class: color_class.to_string(),
        description,
    })
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
        if let Some(region) = checked_section_region(
            section.file_offset,
            0,
            16,
            format!("Section: {}", section.section_type),
            "region-section-type",
            format!("{} section type identifier", section.section_type),
        ) {
            regions.push(region);
        }

        if let Some(region) = checked_section_region(
            section.file_offset,
            16,
            24,
            "Next Offset".to_string(),
            "region-offset",
            format!("Next section at 0x{:X}", section.next_offset),
        ) {
            regions.push(region);
        }

        if let Some(region) = checked_section_region(
            section.file_offset,
            24,
            32,
            "Section Size".to_string(),
            "region-offset",
            format!("{} bytes", section.section_size),
        ) {
            regions.push(region);
        }

        if let Some(region) = checked_section_region(
            section.file_offset,
            32,
            72,
            "Padding".to_string(),
            "region-reserved",
            "Reserved padding bytes".to_string(),
        ) {
            regions.push(region);
        }

        if let Some(region) = checked_section_region(
            section.file_offset,
            72,
            SECTION_HEADER_SIZE as u64,
            "Checksum".to_string(),
            "region-checksum",
            format!("Adler-32: 0x{:08X}", section.checksum),
        ) {
            regions.push(region);
        }

        // Section data region (if not too large)
        if section.section_size > SECTION_HEADER_SIZE as u64 {
            if let (Some(data_start), Some(data_end)) = (
                checked_section_offset(section.file_offset, SECTION_HEADER_SIZE as u64),
                checked_section_offset(section.file_offset, section.section_size),
            ) {
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
                    end: data_end.min(data_start.saturating_add(1024)),
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

#[cfg(test)]
mod tests {
    use crate::ewf::parser::types::{
        EwfDetailedInfo, EwfSectionHeader, EwfVariant, SECTION_HEADER_SIZE,
    };
    use super::ewf_detailed_info_to_metadata;

    fn info_with_section(section: EwfSectionHeader) -> EwfDetailedInfo {
        EwfDetailedInfo {
            variant: EwfVariant::E01,
            version: 1,
            segment_number: 1,
            sections: vec![section],
            file_size: 4096,
            ..Default::default()
        }
    }

    #[test]
    fn metadata_skips_overflowed_section_regions() {
        let metadata = ewf_detailed_info_to_metadata(&info_with_section(EwfSectionHeader {
            section_type: "header".to_string(),
            next_offset: 0,
            section_size: SECTION_HEADER_SIZE as u64 + 128,
            checksum: 0x1234_5678,
            file_offset: u64::MAX - 8,
        }));

        let region_names = metadata
            .regions
            .iter()
            .map(|region| region.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(region_names, vec!["EWF Signature", "Segment Info"]);
    }

    #[test]
    fn metadata_keeps_valid_section_data_region() {
        let section_offset = 128;
        let metadata = ewf_detailed_info_to_metadata(&info_with_section(EwfSectionHeader {
            section_type: "hash".to_string(),
            next_offset: 0,
            section_size: SECTION_HEADER_SIZE as u64 + 2048,
            checksum: 0xABCD_EF01,
            file_offset: section_offset,
        }));

        let data_region = metadata
            .regions
            .iter()
            .find(|region| region.name == "hash Data")
            .expect("hash data region should be present");

        assert_eq!(data_region.start, section_offset + SECTION_HEADER_SIZE as u64);
        assert_eq!(data_region.end, data_region.start + 1024);
    }
}
