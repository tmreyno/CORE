// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Findings types: findings, severity/category enums, and exhibits.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A finding from the examination
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Finding {
    /// Finding identifier
    pub finding_id: String,
    /// Title/summary
    pub title: String,
    /// Severity/significance
    pub severity: FindingSeverity,
    /// Category of finding
    pub category: FindingCategory,
    /// Detailed description
    pub description: String,
    /// Evidence supporting this finding
    pub supporting_evidence: Vec<String>,
    /// Related file paths
    pub related_files: Vec<String>,
    /// Related timestamps
    pub timestamps: Vec<DateTime<Utc>>,
    /// Screenshots or exhibits
    pub exhibits: Vec<Exhibit>,
    /// Additional notes
    pub notes: Option<String>,
}

impl Finding {
    /// Create a new finding
    #[inline]
    pub fn new(
        finding_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            finding_id: finding_id.into(),
            title: title.into(),
            description: description.into(),
            ..Default::default()
        }
    }

    /// Set severity
    #[inline]
    pub fn with_severity(mut self, severity: FindingSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set category
    #[inline]
    pub fn with_category(mut self, category: FindingCategory) -> Self {
        self.category = category;
        self
    }

    /// Add supporting evidence
    #[inline]
    pub fn add_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.supporting_evidence.push(evidence.into());
        self
    }

    /// Add related file
    #[inline]
    pub fn add_file(mut self, path: impl Into<String>) -> Self {
        self.related_files.push(path.into());
        self
    }

    /// Add timestamp
    #[inline]
    pub fn add_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamps.push(timestamp);
        self
    }

    /// Add exhibit
    #[inline]
    pub fn add_exhibit(mut self, exhibit: Exhibit) -> Self {
        self.exhibits.push(exhibit);
        self
    }

    /// Set notes
    #[inline]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Severity levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FindingSeverity {
    /// Informational
    #[default]
    Info,
    /// Low significance
    Low,
    /// Medium significance
    Medium,
    /// High significance
    High,
    /// Critical finding
    Critical,
}

impl FindingSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingSeverity::Info => "Informational",
            FindingSeverity::Low => "Low",
            FindingSeverity::Medium => "Medium",
            FindingSeverity::High => "High",
            FindingSeverity::Critical => "Critical",
        }
    }
}

/// Categories for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FindingCategory {
    /// User activity
    UserActivity,
    /// File system artifacts
    #[default]
    FileSystem,
    /// Internet/browser history
    InternetHistory,
    /// Communication (email, chat)
    Communication,
    /// Document access
    Documents,
    /// Media files
    Media,
    /// Deleted data
    DeletedData,
    /// Anti-forensics
    AntiForensics,
    /// Malware/suspicious software
    Malware,
    /// System events
    SystemEvents,
    /// Network activity
    NetworkActivity,
    /// External devices
    ExternalDevices,
    /// Encryption
    Encryption,
    /// Timeline
    Timeline,
    /// Other
    Other,
}

impl FindingCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingCategory::UserActivity => "User Activity",
            FindingCategory::FileSystem => "File System",
            FindingCategory::InternetHistory => "Internet History",
            FindingCategory::Communication => "Communication",
            FindingCategory::Documents => "Documents",
            FindingCategory::Media => "Media Files",
            FindingCategory::DeletedData => "Deleted Data",
            FindingCategory::AntiForensics => "Anti-Forensics",
            FindingCategory::Malware => "Malware/Suspicious Software",
            FindingCategory::SystemEvents => "System Events",
            FindingCategory::NetworkActivity => "Network Activity",
            FindingCategory::ExternalDevices => "External Devices",
            FindingCategory::Encryption => "Encryption",
            FindingCategory::Timeline => "Timeline Analysis",
            FindingCategory::Other => "Other",
        }
    }
}

/// An exhibit (screenshot, file excerpt, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exhibit {
    /// Exhibit identifier
    pub exhibit_id: String,
    /// Title/caption
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Type of exhibit
    pub exhibit_type: ExhibitType,
    /// File path (for images) or content (for text)
    pub content: String,
    /// Related finding IDs
    pub related_findings: Vec<String>,
}

/// Types of exhibits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExhibitType {
    /// Screenshot image
    Screenshot,
    /// File hex dump
    HexDump,
    /// Text excerpt
    TextExcerpt,
    /// Table/structured data
    Table,
    /// Diagram
    Diagram,
    /// Photo
    Photo,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // FindingSeverity
    // =========================================================================

    #[test]
    fn test_finding_severity_as_str_all_variants() {
        assert_eq!(FindingSeverity::Info.as_str(), "Informational");
        assert_eq!(FindingSeverity::Low.as_str(), "Low");
        assert_eq!(FindingSeverity::Medium.as_str(), "Medium");
        assert_eq!(FindingSeverity::High.as_str(), "High");
        assert_eq!(FindingSeverity::Critical.as_str(), "Critical");
    }

    #[test]
    fn test_finding_severity_default() {
        let severity = FindingSeverity::default();
        assert_eq!(severity, FindingSeverity::Info);
    }

    // =========================================================================
    // FindingCategory
    // =========================================================================

    #[test]
    fn test_finding_category_as_str_all_variants() {
        assert_eq!(FindingCategory::UserActivity.as_str(), "User Activity");
        assert_eq!(FindingCategory::FileSystem.as_str(), "File System");
        assert_eq!(
            FindingCategory::InternetHistory.as_str(),
            "Internet History"
        );
        assert_eq!(FindingCategory::Communication.as_str(), "Communication");
        assert_eq!(FindingCategory::Documents.as_str(), "Documents");
        assert_eq!(FindingCategory::Media.as_str(), "Media Files");
        assert_eq!(FindingCategory::DeletedData.as_str(), "Deleted Data");
        assert_eq!(FindingCategory::AntiForensics.as_str(), "Anti-Forensics");
        assert_eq!(
            FindingCategory::Malware.as_str(),
            "Malware/Suspicious Software"
        );
        assert_eq!(FindingCategory::SystemEvents.as_str(), "System Events");
        assert_eq!(
            FindingCategory::NetworkActivity.as_str(),
            "Network Activity"
        );
        assert_eq!(
            FindingCategory::ExternalDevices.as_str(),
            "External Devices"
        );
        assert_eq!(FindingCategory::Encryption.as_str(), "Encryption");
        assert_eq!(FindingCategory::Timeline.as_str(), "Timeline Analysis");
        assert_eq!(FindingCategory::Other.as_str(), "Other");
    }

    #[test]
    fn test_finding_category_default() {
        let category = FindingCategory::default();
        assert_eq!(category, FindingCategory::FileSystem);
    }

    // =========================================================================
    // Finding builder
    // =========================================================================

    #[test]
    fn test_finding_builder() {
        let finding = Finding::new(
            "F001",
            "Deleted files found",
            "Multiple deleted files recovered",
        )
        .with_severity(FindingSeverity::High)
        .with_category(FindingCategory::DeletedData)
        .add_evidence("E001")
        .add_file("/Users/suspect/deleted.txt")
        .add_timestamp(Utc::now())
        .add_exhibit(Exhibit {
            exhibit_id: "EX1".to_string(),
            title: "Screenshot".to_string(),
            description: None,
            exhibit_type: ExhibitType::Screenshot,
            content: "base64...".to_string(),
            related_findings: vec![],
        })
        .with_notes("Important finding");

        assert_eq!(finding.finding_id, "F001");
        assert_eq!(finding.title, "Deleted files found");
        assert_eq!(finding.severity, FindingSeverity::High);
        assert_eq!(finding.category, FindingCategory::DeletedData);
        assert_eq!(finding.supporting_evidence.len(), 1);
        assert_eq!(finding.related_files.len(), 1);
        assert_eq!(finding.timestamps.len(), 1);
        assert_eq!(finding.exhibits.len(), 1);
        assert!(finding.notes.is_some());
    }

    // =========================================================================
    // ExhibitType equality
    // =========================================================================

    #[test]
    fn test_exhibit_type_equality() {
        assert_eq!(ExhibitType::Screenshot, ExhibitType::Screenshot);
        assert_ne!(ExhibitType::Screenshot, ExhibitType::HexDump);
    }
}
