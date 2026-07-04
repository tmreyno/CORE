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

const MAX_HEADER_DECOMPRESSED_BYTES: u64 = 256 * 1024;
const MAX_EWF_ERROR_ENTRIES: usize = 10_000;

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
        let data_offset = checked_metadata_data_offset(section)?;
        let data_size = checked_metadata_payload_size(section)?;

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

fn checked_metadata_data_offset(section: &EwfSectionHeader) -> Result<u64, ContainerError> {
    section
        .file_offset
        .checked_add(SECTION_HEADER_SIZE as u64)
        .ok_or_else(|| {
            ContainerError::ParseError(format!(
                "EWF {} metadata data offset overflow",
                section.section_type
            ))
        })
}

fn checked_metadata_payload_size(section: &EwfSectionHeader) -> Result<usize, ContainerError> {
    let payload_size = section
        .section_size
        .checked_sub(SECTION_HEADER_SIZE as u64)
        .ok_or_else(|| {
            ContainerError::ParseError(format!(
                "EWF {} metadata section smaller than header",
                section.section_type
            ))
        })?;

    usize::try_from(payload_size).map_err(|_| {
        ContainerError::ParseError(format!(
            "EWF {} metadata payload too large",
            section.section_type
        ))
    })
}

/// Parse hash information from hash/digest sections
pub(super) fn parse_hash_info(
    file: &mut File,
    sections: &[EwfSectionHeader],
) -> Result<EwfHashInfo, ContainerError> {
    let mut hash_info = EwfHashInfo::default();

    // Check digest section first (preferred, has more hashes)
    if let Some(section) = sections.iter().find(|s| s.section_type == "digest") {
        let data_offset = checked_metadata_data_offset(section)?;

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
        let data_offset = checked_metadata_data_offset(section)?;

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
        let data_offset = checked_metadata_data_offset(section)?;
        let payload_size = checked_metadata_payload_size(section)?;
        if payload_size < 12 {
            return Ok(errors);
        }

        file.seek(SeekFrom::Start(data_offset))?;

        let mut header = [0u8; 12];
        if file.read_exact(&mut header).is_err() {
            return Ok(errors);
        }

        let error_count = u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize;

        let available_entries = (payload_size - 12) / 8;
        let max_errors = error_count
            .min(available_entries)
            .min(MAX_EWF_ERROR_ENTRIES);

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

    let decoder = ZlibDecoder::new(data);
    let mut limited_decoder = decoder.take(MAX_HEADER_DECOMPRESSED_BYTES + 1);
    let mut decompressed = Vec::new();
    limited_decoder.read_to_end(&mut decompressed)?;
    if decompressed.len() as u64 > MAX_HEADER_DECOMPRESSED_BYTES {
        return Err(ContainerError::ParseError(format!(
            "EWF header metadata expands beyond limit {MAX_HEADER_DECOMPRESSED_BYTES}"
        )));
    }
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::{Seek, Write};

    fn test_section(section_type: &str, file_offset: u64, section_size: u64) -> EwfSectionHeader {
        EwfSectionHeader {
            section_type: section_type.to_string(),
            next_offset: 0,
            section_size,
            checksum: 0,
            file_offset,
        }
    }

    fn zlib_compress(data: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn test_decompress_zlib_allows_small_header() {
        let compressed = zlib_compress(b"case\tvalue");

        let decompressed = decompress_zlib(&compressed).unwrap();

        assert_eq!(decompressed, b"case\tvalue");
    }

    #[test]
    fn test_decompress_zlib_rejects_oversized_header() {
        let compressed = zlib_compress(&vec![0x42; MAX_HEADER_DECOMPRESSED_BYTES as usize + 1]);

        let err = decompress_zlib(&compressed).unwrap_err();

        assert!(
            matches!(err, ContainerError::ParseError(message) if message.contains("expands beyond limit"))
        );
    }

    #[test]
    fn test_checked_metadata_data_offset_rejects_overflow() {
        let section = test_section("header", u64::MAX, SECTION_HEADER_SIZE as u64);

        let err = checked_metadata_data_offset(&section)
            .expect_err("metadata data offset overflow should fail");

        assert!(
            matches!(err, ContainerError::ParseError(message) if message.contains("offset overflow"))
        );
    }

    #[test]
    fn test_checked_metadata_payload_size_rejects_undersized_section() {
        let section = test_section("header", 0, SECTION_HEADER_SIZE as u64 - 1);

        let err = checked_metadata_payload_size(&section)
            .expect_err("metadata payload smaller than section header should fail");

        assert!(
            matches!(err, ContainerError::ParseError(message) if message.contains("smaller than header"))
        );
    }

    #[test]
    fn test_checked_metadata_payload_size_allows_payload() {
        let section = test_section("digest", 0, SECTION_HEADER_SIZE as u64 + 72);

        assert_eq!(checked_metadata_payload_size(&section).unwrap(), 72);
    }

    #[test]
    fn test_parse_error_info_clamps_declared_count_to_payload_entries() {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(&[0u8; SECTION_HEADER_SIZE]).unwrap();
        file.write_all(&3u32.to_le_bytes()).unwrap();
        file.write_all(&[0u8; 8]).unwrap();
        file.write_all(&1u32.to_le_bytes()).unwrap();
        file.write_all(&1u32.to_le_bytes()).unwrap();
        file.write_all(&7u32.to_le_bytes()).unwrap();
        file.write_all(&2u32.to_le_bytes()).unwrap();
        file.rewind().unwrap();

        let section = test_section("error2", 0, SECTION_HEADER_SIZE as u64 + 12 + 16);
        let errors = parse_error_info(&mut file, &[section]).unwrap();

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[1].first_sector, 7);
        assert_eq!(errors[1].sector_count, 2);
    }
}
