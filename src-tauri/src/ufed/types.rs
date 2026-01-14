// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED types and data structures
//!
//! Types for Universal Forensic Extraction Data (UFED) container formats
//! commonly used in mobile device forensics.

use serde::Serialize;

/// UFED file extensions for detection
pub const UFED_EXTENSIONS: &[&str] = &[".ufdr", ".ufdx", ".ufd"];

/// UFED container format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum UfedFormat {
    /// UFD - Universal Forensic Device extraction metadata (INI format)
    Ufd,
    /// UFDR - UFED Reader format (standalone extraction file)
    Ufdr,
    /// UFDX - Extraction index/metadata file (XML format)
    Ufdx,
    /// ZIP - UFED extraction archive with sibling UFD metadata
    UfedZip,
}

impl std::fmt::Display for UfedFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UfedFormat::Ufd => write!(f, "UFD"),
            UfedFormat::Ufdr => write!(f, "UFDR"),
            UfedFormat::Ufdx => write!(f, "UFDX"),
            UfedFormat::UfedZip => write!(f, "ZIP"),
        }
    }
}

/// UFED container information
#[derive(Debug, Clone, Serialize)]
pub struct UfedInfo {
    /// Format type (UFD, UFDR, UFDX)
    pub format: String,
    /// File size in bytes
    pub size: u64,
    /// Parent directory name (often contains device info)
    pub parent_folder: Option<String>,
    /// Associated files found in the same directory
    pub associated_files: Vec<AssociatedFile>,
    /// Whether this appears to be part of a complete UFED extraction set
    pub is_extraction_set: bool,
    /// Device info extracted from path/filename if available
    pub device_hint: Option<String>,
    /// Case/Crime information from UFD file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_info: Option<CaseInfo>,
    /// Device details from UFD file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_info: Option<DeviceInfo>,
    /// Extraction/acquisition details from UFD file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_info: Option<ExtractionInfo>,
    /// SHA256 hashes from UFD file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored_hashes: Option<Vec<StoredHash>>,
    /// Evidence number derived from folder structure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_number: Option<String>,
    /// Collection-level info from EvidenceCollection.ufdx
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_info: Option<CollectionInfo>,
}

/// Collection-level information from EvidenceCollection.ufdx
#[derive(Debug, Clone, Serialize)]
pub struct CollectionInfo {
    /// Evidence ID (GUID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
    /// Device vendor from UFDX
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// Device model from UFDX
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Device GUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_guid: Option<String>,
    /// List of extraction paths in this collection
    pub extractions: Vec<String>,
    /// UFDX file path
    pub ufdx_path: String,
}

/// Case/Crime information from [Crime Case] section
#[derive(Debug, Clone, Serialize)]
pub struct CaseInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examiner_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Device information from [DeviceInfo] section
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imei: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imei2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iccid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
}

/// Extraction/acquisition information from `\[General\]` section
#[derive(Debug, Clone, Serialize)]
pub struct ExtractionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_name: Option<String>,
}

/// Stored hash value from `\[SHA256\]`, `\[SHA1\]`, `\[MD5\]` sections
#[derive(Debug, Clone, Serialize)]
pub struct StoredHash {
    pub filename: String,
    pub algorithm: String,
    pub hash: String,
    /// When the hash was recorded (from extraction timestamp or file modification)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Associated file in a UFED extraction
#[derive(Debug, Clone, Serialize)]
pub struct AssociatedFile {
    pub filename: String,
    pub file_type: String,
    pub size: u64,
    /// SHA256 hash if available from UFD file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored_hash: Option<String>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ufed_extensions() {
        assert_eq!(UFED_EXTENSIONS.len(), 3);
        assert!(UFED_EXTENSIONS.contains(&".ufdr"));
        assert!(UFED_EXTENSIONS.contains(&".ufdx"));
        assert!(UFED_EXTENSIONS.contains(&".ufd"));
    }

    #[test]
    fn test_ufed_format_display() {
        assert_eq!(format!("{}", UfedFormat::Ufd), "UFD");
        assert_eq!(format!("{}", UfedFormat::Ufdr), "UFDR");
        assert_eq!(format!("{}", UfedFormat::Ufdx), "UFDX");
        assert_eq!(format!("{}", UfedFormat::UfedZip), "ZIP");
    }

    #[test]
    fn test_ufed_format_equality() {
        assert_eq!(UfedFormat::Ufd, UfedFormat::Ufd);
        assert_ne!(UfedFormat::Ufd, UfedFormat::Ufdr);
    }

    #[test]
    fn test_case_info_creation() {
        let case = CaseInfo {
            case_identifier: Some("CASE-2024-001".to_string()),
            crime_type: Some("Digital Forensics".to_string()),
            department: Some("Forensic Lab".to_string()),
            device_name: Some("iPhone 15 Pro".to_string()),
            examiner_name: Some("John Smith".to_string()),
            location: Some("Lab A".to_string()),
        };
        
        assert_eq!(case.case_identifier, Some("CASE-2024-001".to_string()));
        assert_eq!(case.examiner_name, Some("John Smith".to_string()));
    }

    #[test]
    fn test_device_info_creation() {
        let device = DeviceInfo {
            vendor: Some("Apple".to_string()),
            model: Some("iPhone 15 Pro".to_string()),
            full_name: Some("Apple iPhone 15 Pro 256GB".to_string()),
            imei: Some("123456789012345".to_string()),
            imei2: None,
            iccid: Some("8901234567890123456".to_string()),
            os_version: Some("iOS 17.2".to_string()),
            serial_number: Some("ABCD1234EFGH".to_string()),
        };
        
        assert_eq!(device.vendor, Some("Apple".to_string()));
        assert!(device.imei2.is_none());
    }

    #[test]
    fn test_extraction_info_creation() {
        let extraction = ExtractionInfo {
            acquisition_tool: Some("UFED 4PC".to_string()),
            tool_version: Some("7.52.0.59".to_string()),
            unit_id: Some("UNIT-001".to_string()),
            extraction_type: Some("Full File System".to_string()),
            connection_type: Some("USB".to_string()),
            start_time: Some("2024-01-01T10:00:00Z".to_string()),
            end_time: Some("2024-01-01T11:30:00Z".to_string()),
            guid: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
            machine_name: Some("FORENSIC-WKS-01".to_string()),
        };
        
        assert_eq!(extraction.acquisition_tool, Some("UFED 4PC".to_string()));
        assert_eq!(extraction.extraction_type, Some("Full File System".to_string()));
    }

    #[test]
    fn test_stored_hash_creation() {
        let hash = StoredHash {
            filename: "phone_image.bin".to_string(),
            algorithm: "SHA256".to_string(),
            hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            timestamp: Some("2025-05-28 10:30:00".to_string()),
        };
        
        assert_eq!(hash.filename, "phone_image.bin");
        assert_eq!(hash.algorithm, "SHA256");
        assert_eq!(hash.hash.len(), 64); // SHA256 is 64 hex chars
        assert!(hash.timestamp.is_some());
    }

    #[test]
    fn test_associated_file_creation() {
        let file = AssociatedFile {
            filename: "extraction.zip".to_string(),
            file_type: "ZIP".to_string(),
            size: 1073741824, // 1 GB
            stored_hash: Some("abc123".to_string()),
        };
        
        assert_eq!(file.filename, "extraction.zip");
        assert_eq!(file.file_type, "ZIP");
        assert!(file.stored_hash.is_some());
    }

    #[test]
    fn test_associated_file_no_hash() {
        let file = AssociatedFile {
            filename: "readme.txt".to_string(),
            file_type: "TXT".to_string(),
            size: 1024,
            stored_hash: None,
        };
        
        assert!(file.stored_hash.is_none());
    }

    #[test]
    fn test_collection_info_creation() {
        let collection = CollectionInfo {
            evidence_id: Some("EV-001".to_string()),
            vendor: Some("Apple".to_string()),
            model: Some("iPhone".to_string()),
            device_guid: Some("guid-123".to_string()),
            extractions: vec![
                "/path/to/extraction1".to_string(),
                "/path/to/extraction2".to_string(),
            ],
            ufdx_path: "/path/to/EvidenceCollection.ufdx".to_string(),
        };
        
        assert_eq!(collection.extractions.len(), 2);
        assert!(collection.evidence_id.is_some());
    }

    #[test]
    fn test_ufed_info_minimal() {
        let info = UfedInfo {
            format: "UFDR".to_string(),
            size: 1024,
            parent_folder: None,
            associated_files: vec![],
            is_extraction_set: false,
            device_hint: None,
            case_info: None,
            device_info: None,
            extraction_info: None,
            stored_hashes: None,
            evidence_number: None,
            collection_info: None,
        };
        
        assert_eq!(info.format, "UFDR");
        assert!(!info.is_extraction_set);
        assert!(info.case_info.is_none());
    }

    #[test]
    fn test_ufed_info_full() {
        let info = UfedInfo {
            format: "UFD".to_string(),
            size: 2048,
            parent_folder: Some("iPhone_Extraction_2024".to_string()),
            associated_files: vec![
                AssociatedFile {
                    filename: "data.zip".to_string(),
                    file_type: "ZIP".to_string(),
                    size: 1000000,
                    stored_hash: None,
                },
            ],
            is_extraction_set: true,
            device_hint: Some("iPhone".to_string()),
            case_info: Some(CaseInfo {
                case_identifier: Some("CASE-001".to_string()),
                crime_type: None,
                department: None,
                device_name: None,
                examiner_name: None,
                location: None,
            }),
            device_info: None,
            extraction_info: None,
            stored_hashes: Some(vec![
                StoredHash {
                    filename: "file.bin".to_string(),
                    algorithm: "SHA256".to_string(),
                    hash: "abc123".to_string(),
                    timestamp: None,
                },
            ]),
            evidence_number: Some("EV-001".to_string()),
            collection_info: None,
        };
        
        assert!(info.is_extraction_set);
        assert_eq!(info.associated_files.len(), 1);
        assert!(info.case_info.is_some());
        assert!(info.stored_hashes.is_some());
    }
}
