// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF (Expert Witness Format) Hex Parser Module
//!
//! Parses E01/L01/Ex01/Lx01 forensic image headers and extracts
//! detailed metadata for display in the hex viewer.
//!
//! This module handles all EWF variants:
//! - E01: Physical disk image (EWF v1)
//! - L01: Logical evidence file (EWF v1)
//! - Ex01: Physical disk image (EWF v2)
//! - Lx01: Logical evidence file (EWF v2)

pub mod types;
mod metadata;

// Re-export all public types
pub use self::types::{
    EwfVariant, EwfSectionHeader, EwfVolumeInfo, EwfCaseInfo,
    EwfHashInfo, EwfErrorEntry, EwfDetailedInfo,
};

// Re-export metadata conversion function
pub use self::metadata::ewf_detailed_info_to_metadata;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::containers::ContainerError;

// Import all signatures from the ewf types module (single source of truth)
use super::types::{EWF_SIGNATURE, EWF2_SIGNATURE, LVF_SIGNATURE, LVF2_SIGNATURE};

use self::types::{SECTION_HEADER_SIZE, SECTION_TYPES};

// ============================================================================
// Parser Implementation
// ============================================================================

/// Parse an EWF file (E01/L01/Ex01/Lx01) and extract all metadata
pub fn parse_ewf_file(path: &str) -> Result<EwfDetailedInfo, ContainerError> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();

    // Read signature and segment info (first 13 bytes minimum)
    let mut header = [0u8; 32];
    file.read_exact(&mut header)?;

    // Detect variant
    let (variant, version) = detect_ewf_variant(&header)?;

    // Read segment number
    let segment_number = if version == 1 {
        // Segment number at offset 9 (2 bytes LE)
        u16::from_le_bytes([header[9], header[10]])
    } else {
        // EWF v2 has different layout
        u16::from_le_bytes([header[9], header[10]])
    };

    // Parse all sections
    let sections = parse_all_sections(&mut file, 13, file_size)?;

    // Extract volume info
    let volume = parse_volume_section(&mut file, &sections)?;

    // Extract case info from header/header2
    let case_info = metadata::parse_case_info(&mut file, &sections)?;

    // Extract hash info from hash/digest
    let hashes = metadata::parse_hash_info(&mut file, &sections)?;

    // Extract error info from error2
    let errors = metadata::parse_error_info(&mut file, &sections)?;

    Ok(EwfDetailedInfo {
        variant,
        version,
        segment_number,
        sections,
        volume,
        case_info,
        hashes,
        errors,
        file_size,
    })
}

/// Detect EWF format variant from signature
/// Supports E01, L01, Ex01, Lx01
pub fn detect_ewf_variant(header: &[u8]) -> Result<(EwfVariant, u8), ContainerError> {
    if header.len() < 8 {
        return Err(ContainerError::ParseError("Header too short".to_string()));
    }

    // Check for E01 (physical, v1)
    if &header[0..8] == EWF_SIGNATURE {
        return Ok((EwfVariant::E01, 1));
    }
    // Check for L01 (logical, v1)
    if &header[0..8] == LVF_SIGNATURE {
        return Ok((EwfVariant::L01, 1));
    }
    // Check for Ex01 (physical, v2)
    if &header[0..8] == EWF2_SIGNATURE {
        return Ok((EwfVariant::Ex01, 2));
    }
    // Check for Lx01 (logical, v2)
    if &header[0..8] == LVF2_SIGNATURE {
        return Ok((EwfVariant::Lx01, 2));
    }

    // Partial match for EVF/LVF (legacy compatibility)
    if &header[0..3] == b"EVF" {
        return Ok((EwfVariant::E01, 1));
    }
    if &header[0..3] == b"LVF" {
        return Ok((EwfVariant::L01, 1));
    }

    Err(ContainerError::InvalidFormat("Not a valid EWF file".to_string()))
}

/// Check if a file is any EWF variant (E01/L01/Ex01/Lx01)
pub fn is_ewf_file(path: &str) -> Result<bool, ContainerError> {
    let mut file = File::open(path)?;
    let mut sig = [0u8; 8];
    if file.read_exact(&mut sig).is_err() {
        return Ok(false);
    }

    Ok(&sig == EWF_SIGNATURE ||
       &sig == LVF_SIGNATURE ||
       &sig == EWF2_SIGNATURE ||
       &sig == LVF2_SIGNATURE)
}

/// Check if a file is L01 format specifically
pub fn is_l01_file(path: &str) -> Result<bool, ContainerError> {
    let mut file = File::open(path)?;
    let mut sig = [0u8; 8];
    if file.read_exact(&mut sig).is_err() {
        return Ok(false);
    }

    Ok(&sig == LVF_SIGNATURE || &sig == LVF2_SIGNATURE)
}

/// Parse all section headers in the file
fn parse_all_sections(file: &mut File, start_offset: u64, file_size: u64) -> Result<Vec<EwfSectionHeader>, ContainerError> {
    let mut sections = Vec::new();
    let mut offset = start_offset;

    // Safety limit - max 1000 sections
    let max_sections = 1000;

    while offset < file_size && sections.len() < max_sections {
        // Ensure enough space for section header
        if offset + SECTION_HEADER_SIZE as u64 > file_size {
            break;
        }

        file.seek(SeekFrom::Start(offset))?;

        let mut header = [0u8; SECTION_HEADER_SIZE];
        if file.read_exact(&mut header).is_err() {
            break;
        }

        // Parse section type (first 16 bytes, null-terminated)
        let type_bytes = &header[0..16];
        let section_type = type_bytes
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect::<String>();

        // Validate section type
        if section_type.is_empty() || !is_valid_section_type(&section_type) {
            // Not a valid section, might be data or end of sections
            break;
        }

        // Parse offsets and size
        let next_offset = u64::from_le_bytes([
            header[16], header[17], header[18], header[19],
            header[20], header[21], header[22], header[23],
        ]);

        let section_size = u64::from_le_bytes([
            header[24], header[25], header[26], header[27],
            header[28], header[29], header[30], header[31],
        ]);

        // Checksum at offset 72
        let checksum = u32::from_le_bytes([
            header[72], header[73], header[74], header[75],
        ]);

        let section = EwfSectionHeader {
            section_type: section_type.clone(),
            next_offset,
            section_size,
            checksum,
            file_offset: offset,
        };

        sections.push(section);

        // Check for terminal sections
        if section_type == "done" {
            break;
        }

        // Move to next section
        if next_offset > 0 && next_offset > offset {
            offset = next_offset;
        } else if section_size > 0 {
            offset += section_size;
        } else {
            break;
        }
    }

    Ok(sections)
}

/// Check if a section type is valid
fn is_valid_section_type(section_type: &str) -> bool {
    SECTION_TYPES.contains(&section_type)
}

/// Parse volume section data
fn parse_volume_section(file: &mut File, sections: &[EwfSectionHeader]) -> Result<Option<EwfVolumeInfo>, ContainerError> {
    let volume_section = sections.iter().find(|s| s.section_type == "volume");

    if let Some(section) = volume_section {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;

        file.seek(SeekFrom::Start(data_offset))?;

        let mut data = [0u8; 80];
        if file.read_exact(&mut data).is_err() {
            return Ok(None);
        }

        // Parse volume fields
        let chunk_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let sectors_per_chunk = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let bytes_per_sector = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let sector_count = u64::from_le_bytes([
            data[16], data[17], data[18], data[19],
            data[20], data[21], data[22], data[23],
        ]);

        let chs_cylinders = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let chs_heads = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        let chs_sectors = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        let media_type = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);
        let compression_level = data[56];

        // GUID at offset 60 (16 bytes)
        let guid = if data.len() > 76 {
            let guid_bytes = &data[60..76];
            if guid_bytes.iter().any(|&b| b != 0) {
                Some(format_guid(guid_bytes))
            } else {
                None
            }
        } else {
            None
        };

        return Ok(Some(EwfVolumeInfo {
            chunk_count,
            sectors_per_chunk,
            bytes_per_sector,
            sector_count,
            chs_cylinders,
            chs_heads,
            chs_sectors,
            media_type,
            compression_level,
            guid,
        }));
    }

    Ok(None)
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Format bytes as hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Format GUID bytes
fn format_guid(bytes: &[u8]) -> String {
    if bytes.len() < 16 {
        return bytes_to_hex(bytes);
    }
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_le_bytes([bytes[4], bytes[5]]),
        u16::from_le_bytes([bytes[6], bytes[7]]),
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::format_size;

    #[test]
    fn test_detect_ewf_variant() {
        // E01
        let e01_header = b"EVF\x09\x0d\x0a\xff\x00\x01\x00\x01\x00\x00";
        let (variant, version) = detect_ewf_variant(e01_header).unwrap();
        assert_eq!(variant, EwfVariant::E01);
        assert_eq!(version, 1);

        // L01
        let l01_header = b"LVF\x09\x0d\x0a\xff\x00\x01\x00\x01\x00\x00";
        let (variant, version) = detect_ewf_variant(l01_header).unwrap();
        assert_eq!(variant, EwfVariant::L01);
        assert_eq!(version, 1);

        // Ex01
        let ex01_header = b"EVF2\x0d\x0a\x81\x00\x01\x00\x01\x00";
        let (variant, version) = detect_ewf_variant(ex01_header).unwrap();
        assert_eq!(variant, EwfVariant::Ex01);
        assert_eq!(version, 2);

        // Lx01
        let lx01_header = b"LVF2\x0d\x0a\x81\x00\x01\x00\x01\x00";
        let (variant, version) = detect_ewf_variant(lx01_header).unwrap();
        assert_eq!(variant, EwfVariant::Lx01);
        assert_eq!(version, 2);
    }

    #[test]
    fn test_ewf_variant_methods() {
        assert!(EwfVariant::L01.is_logical());
        assert!(EwfVariant::Lx01.is_logical());
        assert!(!EwfVariant::E01.is_logical());

        assert!(EwfVariant::E01.is_physical());
        assert!(EwfVariant::Ex01.is_physical());
        assert!(!EwfVariant::L01.is_physical());

        assert!(EwfVariant::Ex01.is_v2());
        assert!(EwfVariant::Lx01.is_v2());
        assert!(!EwfVariant::E01.is_v2());
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = [0x45, 0x56, 0x46, 0x09];
        assert_eq!(bytes_to_hex(&bytes), "45564609");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1024), "1.00 KB (1024 bytes)");
        assert_eq!(format_size(1048576), "1.00 MB (1048576 bytes)");
    }
}
