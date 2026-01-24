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
#[derive(Debug, Clone, Serialize, Default)]
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

impl UfedInfo {
    /// Create new UfedInfo with format
    #[inline]
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            ..Default::default()
        }
    }

    /// Set file size
    #[inline]
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Set parent folder
    #[inline]
    pub fn with_parent_folder(mut self, folder: impl Into<String>) -> Self {
        self.parent_folder = Some(folder.into());
        self
    }

    /// Set case info
    #[inline]
    pub fn with_case_info(mut self, case_info: CaseInfo) -> Self {
        self.case_info = Some(case_info);
        self
    }

    /// Set device info
    #[inline]
    pub fn with_device_info(mut self, device_info: DeviceInfo) -> Self {
        self.device_info = Some(device_info);
        self
    }

    /// Set extraction info
    #[inline]
    pub fn with_extraction_info(mut self, extraction_info: ExtractionInfo) -> Self {
        self.extraction_info = Some(extraction_info);
        self
    }

    /// Add an associated file
    #[inline]
    pub fn add_associated_file(mut self, file: AssociatedFile) -> Self {
        self.associated_files.push(file);
        self
    }

    /// Mark as extraction set
    #[inline]
    pub fn as_extraction_set(mut self) -> Self {
        self.is_extraction_set = true;
        self
    }
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
#[derive(Debug, Clone, Serialize, Default)]
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

impl CaseInfo {
    /// Create new empty CaseInfo
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case identifier
    #[inline]
    pub fn with_case_identifier(mut self, id: impl Into<String>) -> Self {
        self.case_identifier = Some(id.into());
        self
    }

    /// Set crime type
    #[inline]
    pub fn with_crime_type(mut self, crime_type: impl Into<String>) -> Self {
        self.crime_type = Some(crime_type.into());
        self
    }

    /// Set department
    #[inline]
    pub fn with_department(mut self, department: impl Into<String>) -> Self {
        self.department = Some(department.into());
        self
    }

    /// Set examiner name
    #[inline]
    pub fn with_examiner(mut self, examiner: impl Into<String>) -> Self {
        self.examiner_name = Some(examiner.into());
        self
    }

    /// Set location
    #[inline]
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Check if case info is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.case_identifier.is_none()
            && self.crime_type.is_none()
            && self.department.is_none()
            && self.examiner_name.is_none()
    }
}

/// Device information from [DeviceInfo] section
#[derive(Debug, Clone, Serialize, Default)]
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

impl DeviceInfo {
    /// Create new empty DeviceInfo
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set vendor
    #[inline]
    pub fn with_vendor(mut self, vendor: impl Into<String>) -> Self {
        self.vendor = Some(vendor.into());
        self
    }

    /// Set model
    #[inline]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set full name
    #[inline]
    pub fn with_full_name(mut self, name: impl Into<String>) -> Self {
        self.full_name = Some(name.into());
        self
    }

    /// Set IMEI (first)
    #[inline]
    pub fn with_imei(mut self, imei: impl Into<String>) -> Self {
        self.imei = Some(imei.into());
        self
    }

    /// Set IMEI2 (dual SIM)
    #[inline]
    pub fn with_imei2(mut self, imei2: impl Into<String>) -> Self {
        self.imei2 = Some(imei2.into());
        self
    }

    /// Set ICCID
    #[inline]
    pub fn with_iccid(mut self, iccid: impl Into<String>) -> Self {
        self.iccid = Some(iccid.into());
        self
    }

    /// Set OS version
    #[inline]
    pub fn with_os_version(mut self, version: impl Into<String>) -> Self {
        self.os_version = Some(version.into());
        self
    }

    /// Set serial number
    #[inline]
    pub fn with_serial(mut self, serial: impl Into<String>) -> Self {
        self.serial_number = Some(serial.into());
        self
    }

    /// Get device display name (vendor + model or full_name)
    #[inline]
    pub fn display_name(&self) -> Option<String> {
        if let Some(name) = &self.full_name {
            return Some(name.clone());
        }
        match (&self.vendor, &self.model) {
            (Some(v), Some(m)) => Some(format!("{} {}", v, m)),
            (Some(v), None) => Some(v.clone()),
            (None, Some(m)) => Some(m.clone()),
            (None, None) => None,
        }
    }
}

/// Extraction/acquisition information from `\[General\]` section
#[derive(Debug, Clone, Serialize, Default)]
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

impl ExtractionInfo {
    /// Create new empty ExtractionInfo
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set acquisition tool
    #[inline]
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.acquisition_tool = Some(tool.into());
        self
    }

    /// Set tool version
    #[inline]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.tool_version = Some(version.into());
        self
    }

    /// Set extraction type
    #[inline]
    pub fn with_extraction_type(mut self, ext_type: impl Into<String>) -> Self {
        self.extraction_type = Some(ext_type.into());
        self
    }

    /// Set connection type
    #[inline]
    pub fn with_connection_type(mut self, conn_type: impl Into<String>) -> Self {
        self.connection_type = Some(conn_type.into());
        self
    }

    /// Set start time
    #[inline]
    pub fn with_start_time(mut self, time: impl Into<String>) -> Self {
        self.start_time = Some(time.into());
        self
    }

    /// Set end time
    #[inline]
    pub fn with_end_time(mut self, time: impl Into<String>) -> Self {
        self.end_time = Some(time.into());
        self
    }

    /// Set time range
    #[inline]
    pub fn with_time_range(mut self, start: impl Into<String>, end: impl Into<String>) -> Self {
        self.start_time = Some(start.into());
        self.end_time = Some(end.into());
        self
    }

    /// Get tool display string (tool + version)
    #[inline]
    pub fn tool_display(&self) -> Option<String> {
        match (&self.acquisition_tool, &self.tool_version) {
            (Some(t), Some(v)) => Some(format!("{} v{}", t, v)),
            (Some(t), None) => Some(t.clone()),
            _ => None,
        }
    }
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

impl StoredHash {
    /// Create new StoredHash
    #[inline]
    pub fn new(filename: impl Into<String>, algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            algorithm: algorithm.into(),
            hash: hash.into(),
            timestamp: None,
        }
    }

    /// Create SHA256 hash
    #[inline]
    pub fn sha256(filename: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(filename, "SHA256", hash)
    }

    /// Create SHA1 hash
    #[inline]
    pub fn sha1(filename: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(filename, "SHA1", hash)
    }

    /// Create MD5 hash
    #[inline]
    pub fn md5(filename: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(filename, "MD5", hash)
    }

    /// Set timestamp
    #[inline]
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }
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

impl AssociatedFile {
    /// Create new AssociatedFile
    #[inline]
    pub fn new(filename: impl Into<String>, file_type: impl Into<String>, size: u64) -> Self {
        Self {
            filename: filename.into(),
            file_type: file_type.into(),
            size,
            stored_hash: None,
        }
    }

    /// Set stored hash
    #[inline]
    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.stored_hash = Some(hash.into());
        self
    }

    /// Create from path with auto-detected type
    #[inline]
    pub fn from_path(path: &str, size: u64) -> Self {
        let filename = path.rsplit('/').next()
            .or_else(|| path.rsplit('\\').next())
            .unwrap_or(path)
            .to_string();
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        let file_type = match ext.as_str() {
            "ufdr" => "UFDR",
            "ufdx" => "UFDX",
            "ufd" => "UFD",
            "zip" => "ZIP",
            "txt" | "log" => "Log",
            "xml" => "XML",
            _ => "Unknown",
        }.to_string();
        Self::new(filename, file_type, size)
    }
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
