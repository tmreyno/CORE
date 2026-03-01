// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Format Parsing Utilities
//!
//! ## Section Brief
//! Parsing functions for AD1 container format:
//!
//! ### Header Parsing
//! - `read_segment_header()` - Read segment header from file
//! - `read_logical_header()` - Read logical header from file
//! - `copy_into_array()` - Copy string into fixed-size byte array
//!
//! ### Input Validation
//! - `validate_ad1()` - Validate file exists and is readable
//! - `validate_format()` - Convenience wrapper (no segment check)
//!
//! ### Metadata Extraction
//! - `find_hash()` - Find hash in metadata by type
//! - `find_sha256_hash()` - Find SHA256 hash in metadata
//! - `find_timestamp()` - Find timestamp in metadata
//! - `extract_attributes()` - Extract file attributes from metadata
//! - `parse_volume_info()` - Parse volume information
//! - `parse_companion_log()` - Parse companion log (delegates to shared)
//!
//! ### Timestamp Parsing
//! - `parse_timestamp()` - Parse AD1 timestamp string to FileTime

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use chrono::{Local, NaiveDateTime, TimeZone};
use filetime::FileTime;
use tracing::trace;

use crate::ad1::types::*;
use crate::common::binary::{read_string_at, read_u32_at, read_u64_at};
use crate::containers::companion::find_companion_log as find_shared_companion_log;
use crate::containers::ContainerError;

use super::bytes_to_string;

// =============================================================================
// Header Parsing
// =============================================================================

/// Read segment header from file
pub fn read_segment_header(file: &mut File) -> Result<SegmentHeader, ContainerError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|e| ContainerError::IoError(format!("Failed to seek segment header: {e}")))?;
    let mut signature = [0u8; 16];
    file.read_exact(&mut signature)
        .map_err(|e| ContainerError::IoError(format!("Failed to read segment signature: {e}")))?;
    if &signature[..15] != AD1_SIGNATURE {
        return Err(ContainerError::InvalidFormat(
            "File is not of AD1 format".to_string(),
        ));
    }

    Ok(SegmentHeader {
        signature,
        segment_index: read_u32_at(file, 0x18)?,
        segment_number: read_u32_at(file, 0x1c)?,
        fragments_size: read_u32_at(file, 0x22)?,
        header_size: read_u32_at(file, 0x28)?,
    })
}

/// Read logical header from file
pub fn read_logical_header(file: &mut File) -> Result<LogicalHeader, ContainerError> {
    let signature = read_string_at(file, AD1_LOGICAL_MARGIN, 15)?;
    let image_version = read_u32_at(file, 0x210)?;
    let zlib_chunk_size = read_u32_at(file, 0x218)?;
    let logical_metadata_addr = read_u64_at(file, 0x21c)?;
    let first_item_addr = read_u64_at(file, 0x224)?;
    let data_source_name_length = read_u32_at(file, 0x22c)?;
    let ad_signature = read_string_at(file, 0x230, 3)?;
    let data_source_name_addr = read_u64_at(file, 0x234)?;
    let attrguid_footer_addr = read_u64_at(file, 0x23c)?;
    let locsguid_footer_addr = read_u64_at(file, 0x24c)?;
    let data_source_name = read_string_at(file, 0x25c, data_source_name_length as usize)?;

    Ok(LogicalHeader {
        signature: copy_into_array(&signature, 16)?,
        image_version,
        zlib_chunk_size,
        logical_metadata_addr,
        first_item_addr,
        data_source_name_length,
        ad_signature: copy_into_array(&ad_signature, 4)?,
        data_source_name_addr,
        attrguid_footer_addr,
        locsguid_footer_addr,
        data_source_name,
    })
}

/// Copy string into fixed-size byte array
pub fn copy_into_array<const N: usize>(
    value: &str,
    max_len: usize,
) -> Result<[u8; N], ContainerError> {
    let mut buf = [0u8; N];
    let bytes = value.as_bytes();
    let len = bytes.len().min(max_len).min(N);
    buf[..len].copy_from_slice(&bytes[..len]);
    Ok(buf)
}

// =============================================================================
// Input Validation
// =============================================================================

/// Validate AD1 file format
///
/// # Arguments
/// * `path` - Path to the AD1 file
/// * `check_segments` - If true, verify all segment files exist (strict mode)
///
/// # Returns
/// Ok(()) if valid, Err with description if invalid
pub fn validate_ad1(path: &str, check_segments: bool) -> Result<(), ContainerError> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(path.to_string()));
    }

    let mut file = File::open(path_obj)
        .map_err(|e| ContainerError::IoError(format!("Failed to open input file: {e}")))?;
    let mut signature = [0u8; 16];
    file.read_exact(&mut signature)
        .map_err(|e| ContainerError::IoError(format!("Failed to read file signature: {e}")))?;
    if &signature[..15] != AD1_SIGNATURE {
        return Err(ContainerError::InvalidFormat(
            "File is not an AD1 segmented image".to_string(),
        ));
    }

    let segment_count = read_u32_at(&mut file, 0x1c)?;
    if segment_count == 0 {
        return Err(ContainerError::InvalidFormat(
            "Invalid AD1 segment count".to_string(),
        ));
    }

    if check_segments {
        for index in 1..=segment_count {
            let segment_path = crate::common::segments::build_ad1_segment_path(path, index);
            if !Path::new(&segment_path).exists() {
                return Err(ContainerError::SegmentError(format!(
                    "Missing AD1 segment: {segment_path}"
                )));
            }
        }
    }

    Ok(())
}

/// Validate AD1 file format (does not check segments)
/// Convenience wrapper for `validate_ad1(path, false)`
#[inline]
#[allow(dead_code)]
pub fn validate_format(path: &str) -> Result<(), ContainerError> {
    validate_ad1(path, false)
}

// =============================================================================
// Timestamp Parsing
// =============================================================================

/// Parse AD1 timestamp string to FileTime
/// AD1 timestamps are in format: %Y%m%dT%H%M%S (e.g., "20240115T143022")
pub fn parse_timestamp(value: &str) -> Option<FileTime> {
    let parsed = parse_ad1_timestamp(value)?;
    let local = Local
        .from_local_datetime(&parsed)
        .single()
        .unwrap_or_else(|| Local.from_utc_datetime(&parsed));
    Some(FileTime::from_unix_time(local.timestamp(), 0))
}

/// Parse AD1 timestamp string to NaiveDateTime
/// Core parsing logic used by both FileTime and ISO conversions
fn parse_ad1_timestamp(value: &str) -> Option<NaiveDateTime> {
    let trimmed = value.trim_matches('\0').trim();
    if trimmed.len() < 15 {
        return None;
    }
    NaiveDateTime::parse_from_str(trimmed, "%Y%m%dT%H%M%S").ok()
}

/// Parse AD1 timestamp to ISO 8601 format
/// Uses shared parsing logic from parse_ad1_timestamp
fn parse_timestamp_to_iso(value: &str) -> Option<String> {
    let parsed = parse_ad1_timestamp(value)?;
    Some(parsed.format("%Y-%m-%dT%H:%M:%S").to_string())
}

// =============================================================================
// Metadata Extraction
// =============================================================================

/// Find hash value in metadata
pub fn find_hash(metadata: &[Metadata], key: u32) -> Option<String> {
    // Debug: log all hash-related metadata entries
    for meta in metadata {
        if meta.category == HASH_INFO {
            let value = bytes_to_string(&meta.data, true);
            trace!(
                category = meta.category,
                key = format!("0x{:04x}", meta.key),
                expected_key = format!("0x{:04x}", key),
                value = %value,
                "Found hash metadata entry"
            );
        }
    }

    metadata
        .iter()
        .find(|meta| meta.category == HASH_INFO && meta.key == key)
        .map(|meta| bytes_to_string(&meta.data, true))
        .map(|value| {
            // Clean up the hash value - remove any whitespace or non-hex characters
            let cleaned: String = value.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            cleaned.to_lowercase()
        })
}

/// Find SHA256 hash in metadata
#[allow(dead_code)]
pub fn find_sha256_hash(metadata: &[Metadata]) -> Option<String> {
    find_hash_by_key(metadata, SHA256_HASH)
}

/// Find hash by specific key
#[allow(dead_code)]
fn find_hash_by_key(metadata: &[Metadata], key: u32) -> Option<String> {
    metadata
        .iter()
        .find(|meta| meta.category == HASH_INFO && meta.key == key)
        .map(|meta| bytes_to_string(&meta.data, true))
        .map(|value| {
            let cleaned: String = value.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            cleaned.to_lowercase()
        })
}

/// Extract timestamp from metadata by key
pub fn find_timestamp(metadata: &[Metadata], key: u32) -> Option<String> {
    metadata
        .iter()
        .find(|meta| meta.category == TIMESTAMP && meta.key == key)
        .and_then(|meta| {
            let value = bytes_to_string(&meta.data, true);
            parse_timestamp_to_iso(&value)
        })
}

/// Extract file attributes from metadata
pub(crate) fn extract_attributes(metadata: &[Metadata]) -> Option<Vec<String>> {
    let mut attrs = Vec::new();

    for meta in metadata {
        if meta.category != ATTRIBUTES {
            continue;
        }
        // Check attribute flags
        let attr_name = match meta.key {
            READONLY => Some("readonly"),
            HIDDEN => Some("hidden"),
            SYSTEM => Some("system"),
            ARCHIVE => Some("archive"),
            ENCRYPTED => Some("encrypted"),
            COMPRESSED => Some("compressed"),
            _ => None,
        };

        if let Some(name) = attr_name {
            // Check if the attribute value indicates true (non-zero)
            if !meta.data.is_empty() && meta.data.iter().any(|&b| b != 0) {
                attrs.push(name.to_string());
            }
        }
    }

    if attrs.is_empty() {
        None
    } else {
        Some(attrs)
    }
}

// =============================================================================
// Volume & Companion Log Parsing
// =============================================================================

/// Parse volume info from AD1 header region
pub fn parse_volume_info(file: &mut File) -> Option<VolumeInfo> {
    // Volume info is typically at offset 0x2A0+ in the logical header
    // Format: "C:\:NONAME [NTFS]" followed by OS info like "Windows XP (NTFS 3.1)"

    let mut info = VolumeInfo::default();

    // Read volume label region (around 0x2A0-0x2C0)
    if let Ok(volume_str) = read_string_at(file, 0x2A8, 64) {
        let volume_trimmed = volume_str.trim_matches(char::from(0)).trim();
        if !volume_trimmed.is_empty() && volume_trimmed.contains(':') {
            // Parse "C:\:NONAME [NTFS]" format
            if let Some(bracket_start) = volume_trimmed.find('[') {
                if let Some(bracket_end) = volume_trimmed.find(']') {
                    info.filesystem =
                        Some(volume_trimmed[bracket_start + 1..bracket_end].to_string());
                }
                info.volume_label = Some(volume_trimmed[..bracket_start].trim().to_string());
            } else {
                info.volume_label = Some(volume_trimmed.to_string());
            }
        }
    }

    // Read OS info region (around 0x370-0x3A0)
    if let Ok(os_str) = read_string_at(file, 0x370, 64) {
        let os_trimmed = os_str.trim_matches(char::from(0)).trim();
        if !os_trimmed.is_empty()
            && (os_trimmed.contains("Windows")
                || os_trimmed.contains("NTFS")
                || os_trimmed.contains("Linux"))
        {
            info.os_info = Some(os_trimmed.to_string());
        }
    }

    // Read block size (typically at 0x2E8)
    if let Ok(block_size_str) = read_string_at(file, 0x2E8, 8) {
        let block_trimmed = block_size_str.trim_matches(char::from(0)).trim();
        if let Ok(block_size) = block_trimmed.parse::<u32>() {
            if block_size > 0 && block_size <= 65536 {
                info.block_size = Some(block_size);
            }
        }
    }

    // Only return if we found something useful
    if info.volume_label.is_some() || info.filesystem.is_some() || info.os_info.is_some() {
        Some(info)
    } else {
        None
    }
}

/// Parse companion log file (.ad1.txt, .log, .csv) for case metadata
///
/// Uses shared companion log finder from containers/companion.rs for file discovery,
/// then converts to AD1-specific CompanionLogInfo struct.
///
/// Supports multiple companion file formats:
/// - `filename.ad1.txt` - Standard FTK companion log
/// - `filename.txt` - Simple text companion  
/// - `filename.ad1.log` - Alternative log format
/// - `filename_log.txt` - FTK log naming convention
/// - `filename.ad1.csv` - CSV export format
pub fn parse_companion_log(ad1_path: &str) -> Option<CompanionLogInfo> {
    // Use shared companion log finder
    let shared_info = find_shared_companion_log(ad1_path)?;

    // Convert from shared CompanionLogInfo to AD1-specific format
    // The shared struct has more fields; we extract what AD1 needs
    let mut info = CompanionLogInfo {
        case_number: shared_info.case_number,
        evidence_number: shared_info.evidence_number,
        examiner: shared_info.examiner,
        notes: shared_info.notes,
        md5_hash: None,
        sha1_hash: None,
        sha256_hash: None,
        acquisition_date: shared_info.acquisition_started,
        source_device: shared_info.unique_description,
        source_path: None,
        acquisition_tool: shared_info.created_by,
        total_items: None,
        total_size: None,
        acquisition_method: None,
        organization: None,
    };

    // Extract hashes from stored_hashes Vec
    for hash in &shared_info.stored_hashes {
        let algo_lower = hash.algorithm.to_lowercase();
        if algo_lower.contains("md5") {
            info.md5_hash = Some(hash.hash.clone());
        } else if algo_lower.contains("sha1") || algo_lower.contains("sha-1") {
            info.sha1_hash = Some(hash.hash.clone());
        } else if algo_lower.contains("sha256") || algo_lower.contains("sha-256") {
            info.sha256_hash = Some(hash.hash.clone());
        }
    }

    Some(info)
}

/// Parse a single field from companion log into CompanionLogInfo
/// NOTE: Kept for backward compatibility but no longer used since we delegate
/// to the shared companion parser from containers/companion.rs
#[allow(dead_code)]
fn parse_companion_field(
    info: &mut CompanionLogInfo,
    key: &str,
    value: &str,
    notes_lines: &mut Vec<String>,
) {
    match key {
        // Case identification
        "case number" | "case" | "case #" | "case no" | "case_number" | "casenumber" => {
            info.case_number = Some(value.to_string());
        }
        "evidence number" | "evidence" | "evidence #" | "evidence no" | "evidence_number"
        | "item" | "item number" | "item #" | "exhibit" => {
            info.evidence_number = Some(value.to_string());
        }
        "examiner name" | "examiner" | "analyst" | "investigator" | "operator" => {
            info.examiner = Some(value.to_string());
        }
        "organization" | "agency" | "department" | "company" => {
            info.organization = Some(value.to_string());
        }

        // Hash values
        "md5" | "md5 hash" | "md5 checksum" | "md5_hash" => {
            info.md5_hash = Some(value.to_lowercase());
        }
        "sha1" | "sha1 hash" | "sha-1" | "sha1 checksum" | "sha1_hash" => {
            info.sha1_hash = Some(value.to_lowercase());
        }
        "sha256" | "sha256 hash" | "sha-256" | "sha256 checksum" | "sha256_hash" => {
            info.sha256_hash = Some(value.to_lowercase());
        }

        // Acquisition details
        "acquisition date" | "acquired" | "date" | "acquisition_date" | "created"
        | "start time" | "acquisition time" => {
            info.acquisition_date = Some(value.to_string());
        }
        "source" | "source device" | "device" | "source_device" | "media" | "source media"
        | "drive" => {
            info.source_device = Some(value.to_string());
        }
        "source path" | "path" | "source_path" | "location" | "source location" => {
            info.source_path = Some(value.to_string());
        }
        "acquisition tool" | "tool" | "acquisition_tool" | "software" | "program" | "ftk"
        | "ftk imager" | "encase" | "axiom" => {
            info.acquisition_tool = Some(value.to_string());
        }
        "acquisition method" | "method" | "acquisition_method" | "type" | "image type" => {
            info.acquisition_method = Some(value.to_string());
        }

        // Notes and description
        "notes" | "description" | "comments" | "remarks" => {
            if !value.is_empty() {
                notes_lines.push(value.to_string());
            }
        }
        _ => {}
    }
}

/// Extract hex hash from a line
/// NOTE: Kept for reference but no longer used - shared parser handles this
#[allow(dead_code)]
fn extract_hash(line: &str, expected_len: usize) -> Option<String> {
    // Find consecutive hex string of expected length
    let mut hex_chars = String::new();
    let mut found_start = false;

    for c in line.chars() {
        if c.is_ascii_hexdigit() {
            hex_chars.push(c);
            found_start = true;
        } else if found_start && hex_chars.len() >= expected_len {
            // We have enough, stop
            break;
        } else if found_start && !hex_chars.is_empty() {
            // Reset if we hit a non-hex char before getting enough
            if hex_chars.len() < expected_len {
                hex_chars.clear();
                found_start = false;
            }
        }
    }

    if hex_chars.len() >= expected_len {
        Some(hex_chars[..expected_len].to_lowercase())
    } else {
        None
    }
}

/// Extract a number from a line (for item counts)
/// NOTE: Kept for reference but no longer used - shared parser handles this
#[allow(dead_code)]
fn extract_number(line: &str) -> Option<u64> {
    // Find first number in the line (ignoring common non-count numbers)
    let digits: String = line
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| c.is_ascii_digit())
        .collect();

    if digits.is_empty() {
        return None;
    }

    digits.parse().ok()
}

/// Extract size value from a line (handles KB, MB, GB suffixes)
/// NOTE: Kept for reference but no longer used - shared parser handles this
#[allow(dead_code)]
fn extract_size(line: &str) -> Option<u64> {
    // Look for patterns like "1.5 GB", "1024 MB", "1,024,000 bytes"
    let line_lower = line.to_lowercase();

    // Find numeric value (including decimals and commas)
    let num_str: String = line
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .filter(|c| *c != ',')
        .collect();

    let num: f64 = num_str.parse().ok()?;

    // Determine multiplier based on suffix
    let multiplier = if line_lower.contains("tb") || line_lower.contains("terabyte") {
        1_099_511_627_776u64
    } else if line_lower.contains("gb") || line_lower.contains("gigabyte") {
        1_073_741_824u64
    } else if line_lower.contains("mb") || line_lower.contains("megabyte") {
        1_048_576u64
    } else if line_lower.contains("kb") || line_lower.contains("kilobyte") {
        1_024u64
    } else {
        1u64 // bytes
    };

    Some((num * multiplier as f64) as u64)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_into_array() {
        let result: [u8; 4] = copy_into_array("test", 4).unwrap();
        assert_eq!(&result, b"test");

        let result: [u8; 8] = copy_into_array("hi", 8).unwrap();
        assert_eq!(&result[..2], b"hi");
        assert_eq!(&result[2..], &[0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_extract_hash_md5() {
        // Standard MD5 hash line
        let hash = extract_hash("MD5: d41d8cd98f00b204e9800998ecf8427e", 32);
        assert_eq!(hash, Some("d41d8cd98f00b204e9800998ecf8427e".to_string()));

        // Hash without label
        let hash = extract_hash("d41d8cd98f00b204e9800998ecf8427e", 32);
        assert_eq!(hash, Some("d41d8cd98f00b204e9800998ecf8427e".to_string()));

        // Hash with spaces
        let hash = extract_hash("MD5 Hash: D41D8CD98F00B204E9800998ECF8427E", 32);
        assert_eq!(hash, Some("d41d8cd98f00b204e9800998ecf8427e".to_string()));

        // Too short
        let hash = extract_hash("MD5: d41d8cd98f", 32);
        assert_eq!(hash, None);
    }

    #[test]
    fn test_extract_hash_sha1() {
        // Standard SHA1 hash line
        let hash = extract_hash("SHA1: da39a3ee5e6b4b0d3255bfef95601890afd80709", 40);
        assert_eq!(
            hash,
            Some("da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string())
        );

        // SHA-1 format
        let hash = extract_hash("SHA-1: DA39A3EE5E6B4B0D3255BFEF95601890AFD80709", 40);
        assert_eq!(
            hash,
            Some("da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string())
        );
    }

    #[test]
    fn test_extract_hash_sha256() {
        // Standard SHA256 hash line
        let hash = extract_hash(
            "SHA256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            64,
        );
        assert_eq!(
            hash,
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string())
        );
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("Total items: 1234"), Some(1234));
        assert_eq!(extract_number("Count: 1,000,000"), Some(1000000));
        assert_eq!(extract_number("Files processed: 42 files"), Some(42));
        assert_eq!(extract_number("No numbers here"), None);
    }

    #[test]
    fn test_extract_size() {
        // Various size formats
        assert_eq!(extract_size("Size: 1024 bytes"), Some(1024));
        assert_eq!(extract_size("Total: 1 KB"), Some(1024));
        assert_eq!(extract_size("Size: 1.5 MB"), Some(1572864)); // 1.5 * 1024 * 1024
        assert_eq!(extract_size("Total: 2 GB"), Some(2147483648));
        assert_eq!(extract_size("Size: 1,000 KB"), Some(1024000));
    }

    #[test]
    fn test_parse_companion_field() {
        let mut info = CompanionLogInfo::default();
        let mut notes = Vec::new();

        // Test case number variations
        parse_companion_field(&mut info, "case number", "CASE-001", &mut notes);
        assert_eq!(info.case_number, Some("CASE-001".to_string()));

        // Test examiner variations
        let mut info2 = CompanionLogInfo::default();
        parse_companion_field(&mut info2, "analyst", "John Doe", &mut notes);
        assert_eq!(info2.examiner, Some("John Doe".to_string()));

        // Test hash fields
        let mut info3 = CompanionLogInfo::default();
        parse_companion_field(&mut info3, "sha256 hash", "abc123", &mut notes);
        assert_eq!(info3.sha256_hash, Some("abc123".to_string()));

        // Test source device
        let mut info4 = CompanionLogInfo::default();
        parse_companion_field(&mut info4, "source media", "USB Drive", &mut notes);
        assert_eq!(info4.source_device, Some("USB Drive".to_string()));
    }

    #[test]
    fn test_parse_timestamp_to_iso() {
        // Valid AD1 timestamp format
        let result = parse_timestamp_to_iso("20240115T143022");
        assert_eq!(result, Some("2024-01-15T14:30:22".to_string()));

        // With null terminators
        let result = parse_timestamp_to_iso("20240115T143022\0\0\0");
        assert_eq!(result, Some("2024-01-15T14:30:22".to_string()));

        // Too short
        let result = parse_timestamp_to_iso("20240115");
        assert_eq!(result, None);

        // Empty
        let result = parse_timestamp_to_iso("");
        assert_eq!(result, None);
    }
}
