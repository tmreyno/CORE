// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Metadata extraction functions for EWF format analysis.
//!
//! Handles parsing of case info from header/header2 sections, hash extraction
//! from hash/digest sections, and error sector parsing.
//!
//! Note: The `ewf_detailed_info_to_metadata()` UI conversion function lives in
//! the main application's `ewf` shim module because it depends on viewer types.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use ffx_errors::ContainerError;

use super::types::{
    EwfCaseInfo, EwfErrorEntry, EwfHashInfo, EwfSectionHeader, SECTION_HEADER_SIZE,
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
