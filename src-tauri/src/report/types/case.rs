// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Case-related types: case info, examiner, evidence items, and forensic image metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::deserialize_datetime_opt;
use super::records::HashRecord;

/// Case information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CaseInfo {
    /// Case number/identifier
    #[serde(default)]
    pub case_number: String,
    /// Case name/title
    #[serde(default)]
    pub case_name: Option<String>,
    /// Agency or organization
    #[serde(default)]
    pub agency: Option<String>,
    /// Requesting party
    #[serde(default)]
    pub requestor: Option<String>,
    /// Date examination was requested
    #[serde(default, deserialize_with = "deserialize_datetime_opt")]
    pub request_date: Option<DateTime<Utc>>,
    /// Examination start date
    #[serde(default, deserialize_with = "deserialize_datetime_opt")]
    pub exam_start_date: Option<DateTime<Utc>>,
    /// Examination end date
    #[serde(default, deserialize_with = "deserialize_datetime_opt")]
    pub exam_end_date: Option<DateTime<Utc>>,
    /// Type of investigation
    #[serde(default)]
    pub investigation_type: Option<String>,
    /// Brief description
    #[serde(default)]
    pub description: Option<String>,
}

/// Examiner information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExaminerInfo {
    /// Examiner's full name
    pub name: String,
    /// Title/position
    pub title: Option<String>,
    /// Organization/agency
    pub organization: Option<String>,
    /// Contact email
    pub email: Option<String>,
    /// Contact phone
    pub phone: Option<String>,
    /// Certifications
    pub certifications: Vec<String>,
    /// Badge/ID number
    pub badge_number: Option<String>,
}

/// Evidence item examined
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EvidenceItem {
    /// Evidence tag/identifier
    pub evidence_id: String,
    /// Description of the item
    pub description: String,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Make/manufacturer
    #[serde(default)]
    pub make: Option<String>,
    /// Model
    #[serde(default)]
    pub model: Option<String>,
    /// Serial number
    #[serde(default)]
    pub serial_number: Option<String>,
    /// Capacity/size
    #[serde(default)]
    pub capacity: Option<String>,
    /// Condition when received
    #[serde(default)]
    pub condition: Option<String>,
    /// Date received (also accepts acquisition_date from frontend)
    #[serde(
        default,
        alias = "acquisition_date",
        deserialize_with = "deserialize_datetime_opt"
    )]
    pub received_date: Option<DateTime<Utc>>,
    /// Who submitted the evidence
    #[serde(default)]
    pub submitted_by: Option<String>,
    /// Hash values at acquisition
    #[serde(default)]
    pub acquisition_hashes: Vec<HashRecord>,
    /// Verification hashes (from frontend)
    #[serde(default)]
    pub verification_hashes: Vec<HashRecord>,
    /// Forensic image information
    #[serde(default)]
    pub image_info: Option<ImageInfo>,
    /// Notes about this item
    #[serde(default)]
    pub notes: Option<String>,
    /// Acquisition method (from frontend)
    #[serde(default)]
    pub acquisition_method: Option<String>,
    /// Acquisition tool (from frontend)
    #[serde(default)]
    pub acquisition_tool: Option<String>,
}

impl Default for EvidenceItem {
    fn default() -> Self {
        Self {
            evidence_id: String::new(),
            description: String::new(),
            evidence_type: EvidenceType::Other,
            make: None,
            model: None,
            serial_number: None,
            capacity: None,
            condition: None,
            received_date: None,
            submitted_by: None,
            acquisition_hashes: Vec::new(),
            verification_hashes: Vec::new(),
            image_info: None,
            notes: None,
            acquisition_method: None,
            acquisition_tool: None,
        }
    }
}

/// Types of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Hard drive (internal)
    HardDrive,
    /// Solid state drive
    SSD,
    /// USB flash drive
    UsbDrive,
    /// External hard drive
    ExternalDrive,
    /// SD card / memory card
    MemoryCard,
    /// Mobile phone
    MobilePhone,
    /// Tablet
    Tablet,
    /// Computer (whole system)
    Computer,
    /// Laptop
    Laptop,
    /// Optical disc (CD/DVD/Blu-ray)
    OpticalDisc,
    /// Cloud storage
    CloudStorage,
    /// Network capture
    NetworkCapture,
    /// Forensic image file
    ForensicImage,
    /// Other
    Other,
}

impl EvidenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidenceType::HardDrive => "Hard Drive",
            EvidenceType::SSD => "Solid State Drive",
            EvidenceType::UsbDrive => "USB Flash Drive",
            EvidenceType::ExternalDrive => "External Drive",
            EvidenceType::MemoryCard => "Memory Card",
            EvidenceType::MobilePhone => "Mobile Phone",
            EvidenceType::Tablet => "Tablet",
            EvidenceType::Computer => "Computer",
            EvidenceType::Laptop => "Laptop",
            EvidenceType::OpticalDisc => "Optical Disc",
            EvidenceType::CloudStorage => "Cloud Storage",
            EvidenceType::NetworkCapture => "Network Capture",
            EvidenceType::ForensicImage => "Forensic Image",
            EvidenceType::Other => "Other",
        }
    }
}

/// Information about a forensic image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Image format (E01, AD1, RAW, etc.)
    pub format: String,
    /// Image file name(s)
    pub file_names: Vec<String>,
    /// Total size in bytes
    pub total_size: u64,
    /// Number of segments
    pub segments: Option<u32>,
    /// Compression type
    pub compression: Option<String>,
    /// Acquisition tool
    pub acquisition_tool: Option<String>,
    /// Acquisition date
    pub acquisition_date: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // EvidenceType
    // =========================================================================

    #[test]
    fn test_evidence_type_as_str_all_variants() {
        assert_eq!(EvidenceType::HardDrive.as_str(), "Hard Drive");
        assert_eq!(EvidenceType::SSD.as_str(), "Solid State Drive");
        assert_eq!(EvidenceType::UsbDrive.as_str(), "USB Flash Drive");
        assert_eq!(EvidenceType::ExternalDrive.as_str(), "External Drive");
        assert_eq!(EvidenceType::MemoryCard.as_str(), "Memory Card");
        assert_eq!(EvidenceType::MobilePhone.as_str(), "Mobile Phone");
        assert_eq!(EvidenceType::Tablet.as_str(), "Tablet");
        assert_eq!(EvidenceType::Computer.as_str(), "Computer");
        assert_eq!(EvidenceType::Laptop.as_str(), "Laptop");
        assert_eq!(EvidenceType::OpticalDisc.as_str(), "Optical Disc");
        assert_eq!(EvidenceType::CloudStorage.as_str(), "Cloud Storage");
        assert_eq!(EvidenceType::NetworkCapture.as_str(), "Network Capture");
        assert_eq!(EvidenceType::ForensicImage.as_str(), "Forensic Image");
        assert_eq!(EvidenceType::Other.as_str(), "Other");
    }

    #[test]
    fn test_evidence_type_serialization_roundtrip() {
        let json = serde_json::to_string(&EvidenceType::MobilePhone).unwrap();
        let back: EvidenceType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, EvidenceType::MobilePhone);
    }

    // =========================================================================
    // EvidenceItem defaults
    // =========================================================================

    #[test]
    fn test_evidence_item_default() {
        let item = EvidenceItem::default();
        assert!(item.evidence_id.is_empty());
        assert_eq!(item.evidence_type, EvidenceType::Other);
        assert!(item.make.is_none());
        assert!(item.acquisition_hashes.is_empty());
    }
}
