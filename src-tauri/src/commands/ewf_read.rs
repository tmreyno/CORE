// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF image info reader — extracts metadata from E01/Ex01/L01/Lx01 containers
//! using the libewf C library (via libewf-ffi).

use serde::Serialize;
use tracing::info;

// =============================================================================
// Types
// =============================================================================

/// Serializable case metadata from an EWF container
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EwfReadCaseInfoResponse {
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner_name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub acquiry_software_version: Option<String>,
    pub acquiry_date: Option<String>,
    pub acquiry_operating_system: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
}

/// Serializable image metadata from an EWF container
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EwfImageInfoResponse {
    /// Detected format name (e.g., "EnCase 5", "EnCase 7 V2")
    pub format: String,
    /// File extension for this format (e.g., ".E01", ".Ex01")
    pub format_extension: String,
    /// Whether this is a logical evidence format
    pub is_logical: bool,
    /// Whether this is a V2 (EWF2) format
    pub is_v2: bool,
    /// Total media size in bytes
    pub media_size: u64,
    /// Bytes per sector
    pub bytes_per_sector: u32,
    /// Sectors per chunk
    pub sectors_per_chunk: u32,
    /// Compression level (-1=default, 0=none, 1=fast, 2=best)
    pub compression_level: i8,
    /// Compression method name (e.g., "Deflate", "BZIP2")
    pub compression_method: String,
    /// Media type constant
    pub media_type: u8,
    /// Media flags
    pub media_flags: u8,
    /// Segment file version (e.g., "1.0", "2.0")
    pub segment_file_version: Option<String>,
    /// Whether any segment files are corrupted
    pub is_corrupted: bool,
    /// Whether the image is encrypted
    pub is_encrypted: bool,
    /// Case/evidence metadata
    pub case_info: EwfReadCaseInfoResponse,
    /// Stored MD5 hash (hex string, if present)
    pub md5_hash: Option<String>,
    /// Stored SHA1 hash (hex string, if present)
    pub sha1_hash: Option<String>,
}

// =============================================================================
// Tauri Command
// =============================================================================

/// Read detailed image metadata from an E01/Ex01/L01/Lx01 container using libewf
///
/// Opens the EWF container (auto-discovers all segment files) and extracts
/// format info, case metadata, stored hashes, and image parameters. This uses
/// the libewf C library (via libewf-ffi) for comprehensive format support,
/// complementing the pure-Rust EWF parser used for tree browsing.
#[tauri::command]
pub async fn ewf_read_image_info(path: String) -> Result<EwfImageInfoResponse, String> {
    info!("Reading EWF image info via libewf: {}", path);

    // EwfReader::open is not Send — run on blocking thread
    let result = tokio::task::spawn_blocking(move || {
        let reader = libewf_ffi::EwfReader::open(&path)
            .map_err(|e| format!("Failed to open EWF container: {}", e))?;

        let info = reader
            .image_info()
            .map_err(|e| format!("Failed to read image info: {}", e))?;

        let case = &info.case_info;

        Ok::<EwfImageInfoResponse, String>(EwfImageInfoResponse {
            format: info.format.name().to_string(),
            format_extension: info.format.extension().to_string(),
            is_logical: info.format.is_logical(),
            is_v2: info.format.is_v2(),
            media_size: info.media_size,
            bytes_per_sector: info.bytes_per_sector,
            sectors_per_chunk: info.sectors_per_chunk,
            compression_level: info.compression_level,
            compression_method: info.compression_method.name().to_string(),
            media_type: info.media_type,
            media_flags: info.media_flags,
            segment_file_version: info
                .segment_file_version
                .map(|(major, minor)| format!("{}.{}", major, minor)),
            is_corrupted: info.is_corrupted,
            is_encrypted: info.is_encrypted,
            case_info: EwfReadCaseInfoResponse {
                case_number: case.case_number.clone(),
                evidence_number: case.evidence_number.clone(),
                examiner_name: case.examiner_name.clone(),
                description: case.description.clone(),
                notes: case.notes.clone(),
                acquiry_software_version: case.acquiry_software_version.clone(),
                acquiry_date: case.acquiry_date.clone(),
                acquiry_operating_system: case.acquiry_operating_system.clone(),
                model: case.model.clone(),
                serial_number: case.serial_number.clone(),
            },
            md5_hash: info.md5_hash,
            sha1_hash: info.sha1_hash,
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_info_response_serialization() {
        let response = EwfImageInfoResponse {
            format: "EnCase 5".to_string(),
            format_extension: ".E01".to_string(),
            is_logical: false,
            is_v2: false,
            media_size: 1_073_741_824,
            bytes_per_sector: 512,
            sectors_per_chunk: 64,
            compression_level: 1,
            compression_method: "Deflate".to_string(),
            media_type: 0,
            media_flags: 1,
            segment_file_version: Some("1.0".to_string()),
            is_corrupted: false,
            is_encrypted: false,
            case_info: EwfReadCaseInfoResponse {
                case_number: Some("CASE-001".to_string()),
                evidence_number: Some("EV-01".to_string()),
                examiner_name: Some("Jane Smith".to_string()),
                description: Some("Test disk image".to_string()),
                notes: None,
                acquiry_software_version: Some("EnCase 8.0".to_string()),
                acquiry_date: Some("2024-01-15".to_string()),
                acquiry_operating_system: Some("Windows 10".to_string()),
                model: Some("Samsung SSD 860 EVO".to_string()),
                serial_number: Some("S3Z9NB0K123456".to_string()),
            },
            md5_hash: Some("d41d8cd98f00b204e9800998ecf8427e".to_string()),
            sha1_hash: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"format\":"));
        assert!(json.contains("\"formatExtension\":"));
        assert!(json.contains("\"isLogical\":"));
        assert!(json.contains("\"isV2\":"));
        assert!(json.contains("\"mediaSize\":"));
        assert!(json.contains("\"bytesPerSector\":"));
        assert!(json.contains("\"sectorsPerChunk\":"));
        assert!(json.contains("\"compressionLevel\":"));
        assert!(json.contains("\"compressionMethod\":"));
        assert!(json.contains("\"mediaType\":"));
        assert!(json.contains("\"mediaFlags\":"));
        assert!(json.contains("\"segmentFileVersion\":"));
        assert!(json.contains("\"isCorrupted\":"));
        assert!(json.contains("\"isEncrypted\":"));
        assert!(json.contains("\"caseInfo\":"));
        assert!(json.contains("\"md5Hash\":"));
        assert!(json.contains("\"sha1Hash\":"));
        // Verify NOT snake_case
        assert!(!json.contains("format_extension"));
        assert!(!json.contains("is_logical"));
        assert!(!json.contains("media_size"));
    }

    #[test]
    fn test_case_info_response_serialization() {
        let case_info = EwfReadCaseInfoResponse {
            case_number: Some("2024-TEST".to_string()),
            evidence_number: Some("001".to_string()),
            examiner_name: Some("John Doe".to_string()),
            description: Some("Forensic image of suspect drive".to_string()),
            notes: Some("Evidence collected under warrant 2024-W-789".to_string()),
            acquiry_software_version: Some("FTK Imager 4.7".to_string()),
            acquiry_date: Some("2024-06-15T10:30:00".to_string()),
            acquiry_operating_system: Some("Windows 11".to_string()),
            model: None,
            serial_number: None,
        };
        let json = serde_json::to_string(&case_info).unwrap();
        assert!(json.contains("\"caseNumber\":"));
        assert!(json.contains("\"evidenceNumber\":"));
        assert!(json.contains("\"examinerName\":"));
        assert!(json.contains("\"acquirySoftwareVersion\":"));
        assert!(json.contains("\"acquiryDate\":"));
        assert!(json.contains("\"acquiryOperatingSystem\":"));
        assert!(!json.contains("case_number"));
        assert!(!json.contains("evidence_number"));
    }

    #[test]
    fn test_image_info_response_logical_v2() {
        let response = EwfImageInfoResponse {
            format: "Logical EnCase 7 V2".to_string(),
            format_extension: ".Lx01".to_string(),
            is_logical: true,
            is_v2: true,
            media_size: 0,
            bytes_per_sector: 512,
            sectors_per_chunk: 64,
            compression_level: 2,
            compression_method: "BZIP2".to_string(),
            media_type: 0,
            media_flags: 0,
            segment_file_version: Some("2.0".to_string()),
            is_corrupted: false,
            is_encrypted: true,
            case_info: EwfReadCaseInfoResponse {
                case_number: None,
                evidence_number: None,
                examiner_name: None,
                description: None,
                notes: None,
                acquiry_software_version: None,
                acquiry_date: None,
                acquiry_operating_system: None,
                model: None,
                serial_number: None,
            },
            md5_hash: None,
            sha1_hash: Some("da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"isLogical\":true"));
        assert!(json.contains("\"isV2\":true"));
        assert!(json.contains("\"isEncrypted\":true"));
        assert!(json.contains("\"sha1Hash\":"));
    }
}
