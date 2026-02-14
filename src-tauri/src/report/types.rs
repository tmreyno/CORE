// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Data structures for forensic reports
//!
//! This module defines all the types used to represent forensic report data.
//! These structures are serializable and can be used with templates and output generators.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

/// Flexible datetime deserializer that handles ISO strings
pub fn deserialize_datetime_flexible<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DateTimeOrString {
        DateTime(DateTime<Utc>),
        String(String),
    }
    
    match DateTimeOrString::deserialize(deserializer)? {
        DateTimeOrString::DateTime(dt) => Ok(dt),
        DateTimeOrString::String(s) => {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .map(|ndt| ndt.and_utc())
                })
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
                        .map(|ndt| ndt.and_utc())
                })
                .map_err(|e| D::Error::custom(format!("Invalid datetime '{}': {}", s, e)))
        }
    }
}

/// Optional datetime deserializer
pub fn deserialize_datetime_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DateTimeOrString {
        DateTime(DateTime<Utc>),
        String(String),
        Null,
    }
    
    match Option::<DateTimeOrString>::deserialize(deserializer)? {
        None => Ok(None),
        Some(DateTimeOrString::Null) => Ok(None),
        Some(DateTimeOrString::DateTime(dt)) => Ok(Some(dt)),
        Some(DateTimeOrString::String(s)) if s.is_empty() => Ok(None),
        Some(DateTimeOrString::String(s)) => {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| Some(dt.with_timezone(&Utc)))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .map(|ndt| Some(ndt.and_utc()))
                })
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
                        .map(|ndt| Some(ndt.and_utc()))
                })
                .or_else(|_| {
                    chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                        .map(|nd| Some(nd.and_hms_opt(0, 0, 0).expect("midnight is valid time").and_utc()))
                })
                .map_err(|e| D::Error::custom(format!("Invalid datetime '{}': {}", s, e)))
        }
    }
}

/// Main forensic report structure containing all report data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ForensicReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Case information
    pub case_info: CaseInfo,
    /// Examiner information
    pub examiner: ExaminerInfo,
    /// Executive summary (can be AI-generated)
    pub executive_summary: Option<String>,
    /// Scope and objectives of the examination
    pub scope: Option<String>,
    /// Methodology used
    pub methodology: Option<String>,
    /// Evidence items examined
    pub evidence_items: Vec<EvidenceItem>,
    /// Chain of custody records
    pub chain_of_custody: Vec<CustodyRecord>,
    /// Findings from the examination
    pub findings: Vec<Finding>,
    /// Timeline of events (if applicable)
    pub timeline: Vec<TimelineEvent>,
    /// Hash verification records
    pub hash_records: Vec<HashRecord>,
    /// Tools and versions used
    pub tools: Vec<ToolInfo>,
    /// Conclusions
    pub conclusions: Option<String>,
    /// Appendices
    pub appendices: Vec<Appendix>,
    /// Signature/approval records (from frontend)
    #[serde(default)]
    pub signatures: Vec<SignatureRecord>,
    /// Additional notes
    pub notes: Option<String>,
}


impl ForensicReport {
    /// Create a new report builder
    pub fn builder() -> ForensicReportBuilder {
        ForensicReportBuilder::default()
    }

    /// Validate the report for completeness
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.case_info.case_number.is_empty() {
            errors.push("Case number is required".to_string());
        }
        if self.examiner.name.is_empty() {
            errors.push("Examiner name is required".to_string());
        }
        if self.evidence_items.is_empty() {
            errors.push("At least one evidence item is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Builder for creating ForensicReport instances
#[derive(Debug, Default)]
pub struct ForensicReportBuilder {
    metadata: Option<ReportMetadata>,
    case_info: Option<CaseInfo>,
    examiner: Option<ExaminerInfo>,
    executive_summary: Option<String>,
    scope: Option<String>,
    methodology: Option<String>,
    evidence_items: Vec<EvidenceItem>,
    chain_of_custody: Vec<CustodyRecord>,
    findings: Vec<Finding>,
    timeline: Vec<TimelineEvent>,
    hash_records: Vec<HashRecord>,
    tools: Vec<ToolInfo>,
    conclusions: Option<String>,
    appendices: Vec<Appendix>,
    notes: Option<String>,
}

impl ForensicReportBuilder {
    /// Set report metadata
    pub fn metadata(mut self, metadata: ReportMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set case number (creates minimal CaseInfo)
    pub fn case_number(mut self, case_number: impl Into<String>) -> Self {
        let mut info = self.case_info.unwrap_or_default();
        info.case_number = case_number.into();
        self.case_info = Some(info);
        self
    }

    /// Set full case info
    pub fn case_info(mut self, case_info: CaseInfo) -> Self {
        self.case_info = Some(case_info);
        self
    }

    /// Set examiner name (creates minimal ExaminerInfo)
    pub fn examiner_name(mut self, name: impl Into<String>) -> Self {
        let mut info = self.examiner.unwrap_or_default();
        info.name = name.into();
        self.examiner = Some(info);
        self
    }

    /// Set full examiner info
    pub fn examiner(mut self, examiner: ExaminerInfo) -> Self {
        self.examiner = Some(examiner);
        self
    }

    /// Set executive summary
    pub fn executive_summary(mut self, summary: impl Into<String>) -> Self {
        self.executive_summary = Some(summary.into());
        self
    }

    /// Set scope
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Set methodology
    pub fn methodology(mut self, methodology: impl Into<String>) -> Self {
        self.methodology = Some(methodology.into());
        self
    }

    /// Add an evidence item
    pub fn add_evidence(mut self, item: EvidenceItem) -> Self {
        self.evidence_items.push(item);
        self
    }

    /// Add multiple evidence items
    pub fn evidence_items(mut self, items: Vec<EvidenceItem>) -> Self {
        self.evidence_items = items;
        self
    }

    /// Add a custody record
    pub fn add_custody_record(mut self, record: CustodyRecord) -> Self {
        self.chain_of_custody.push(record);
        self
    }

    /// Add a finding
    pub fn add_finding(mut self, finding: Finding) -> Self {
        self.findings.push(finding);
        self
    }

    /// Add multiple findings
    pub fn findings(mut self, findings: Vec<Finding>) -> Self {
        self.findings = findings;
        self
    }

    /// Add a timeline event
    pub fn add_timeline_event(mut self, event: TimelineEvent) -> Self {
        self.timeline.push(event);
        self
    }

    /// Add a hash record
    pub fn add_hash_record(mut self, record: HashRecord) -> Self {
        self.hash_records.push(record);
        self
    }

    /// Add a tool info
    pub fn add_tool(mut self, tool: ToolInfo) -> Self {
        self.tools.push(tool);
        self
    }

    /// Set conclusions
    pub fn conclusions(mut self, conclusions: impl Into<String>) -> Self {
        self.conclusions = Some(conclusions.into());
        self
    }

    /// Add an appendix
    pub fn add_appendix(mut self, appendix: Appendix) -> Self {
        self.appendices.push(appendix);
        self
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Build the ForensicReport
    pub fn build(self) -> Result<ForensicReport, String> {
        let case_info = self.case_info.ok_or("Case info is required")?;
        let examiner = self.examiner.ok_or("Examiner info is required")?;

        let metadata = self.metadata.unwrap_or_else(|| ReportMetadata {
            title: format!("Forensic Examination Report - Case {}", case_info.case_number),
            report_number: format!("RPT-{}", case_info.case_number),
            version: "1.0".to_string(),
            classification: Classification::Confidential,
            generated_at: Utc::now(),
            generated_by: "FFX Forensic File Xplorer".to_string(),
        });

        Ok(ForensicReport {
            metadata,
            case_info,
            examiner,
            executive_summary: self.executive_summary,
            scope: self.scope,
            methodology: self.methodology,
            evidence_items: self.evidence_items,
            chain_of_custody: self.chain_of_custody,
            findings: self.findings,
            timeline: self.timeline,
            hash_records: self.hash_records,
            tools: self.tools,
            conclusions: self.conclusions,
            appendices: self.appendices,
            signatures: Vec::new(),
            notes: self.notes,
        })
    }
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportMetadata {
    /// Report title
    pub title: String,
    /// Report number/identifier
    pub report_number: String,
    /// Version
    pub version: String,
    /// Classification level
    pub classification: Classification,
    /// When the report was generated
    #[serde(deserialize_with = "deserialize_datetime_flexible")]
    pub generated_at: DateTime<Utc>,
    /// Tool that generated the report
    pub generated_by: String,
}

impl Default for ReportMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            report_number: String::new(),
            version: "1.0".to_string(),
            classification: Classification::Internal,
            generated_at: Utc::now(),
            generated_by: "CORE-FFX".to_string(),
        }
    }
}

/// Classification levels for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    /// Public information
    Public,
    /// Internal use only
    Internal,
    /// Confidential
    Confidential,
    /// Restricted access
    Restricted,
    /// Law enforcement sensitive
    LawEnforcementSensitive,
}

impl Classification {
    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            Classification::Public => "PUBLIC",
            Classification::Internal => "INTERNAL",
            Classification::Confidential => "CONFIDENTIAL",
            Classification::Restricted => "RESTRICTED",
            Classification::LawEnforcementSensitive => "LAW ENFORCEMENT SENSITIVE",
        }
    }
}

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
    #[serde(default, alias = "acquisition_date", deserialize_with = "deserialize_datetime_opt")]
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

/// Chain of custody record
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustodyRecord {
    /// Evidence item ID
    pub evidence_id: String,
    /// Date/time of transfer
    pub timestamp: DateTime<Utc>,
    /// Person releasing custody
    pub released_by: String,
    /// Person receiving custody
    pub received_by: String,
    /// Purpose of transfer
    pub purpose: Option<String>,
    /// Location
    pub location: Option<String>,
    /// Notes
    pub notes: Option<String>,
}

impl CustodyRecord {
    /// Create a new custody record
    #[inline]
    pub fn new(
        evidence_id: impl Into<String>,
        released_by: impl Into<String>,
        received_by: impl Into<String>,
    ) -> Self {
        Self {
            evidence_id: evidence_id.into(),
            timestamp: Utc::now(),
            released_by: released_by.into(),
            received_by: received_by.into(),
            ..Default::default()
        }
    }

    /// Set timestamp
    #[inline]
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set purpose
    #[inline]
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    /// Set location
    #[inline]
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set notes
    #[inline]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

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

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TimelineEvent {
    /// Event timestamp
    #[serde(deserialize_with = "deserialize_datetime_flexible")]
    pub timestamp: DateTime<Utc>,
    /// Timestamp type (created, modified, accessed, etc.)
    pub timestamp_type: String,
    /// Event description
    pub description: String,
    /// Source of the event
    pub source: String,
    /// Related file or artifact
    #[serde(default)]
    pub artifact: Option<String>,
    /// Related evidence ID
    #[serde(default)]
    pub evidence_id: Option<String>,
    /// Significance
    #[serde(default)]
    pub significance: Option<String>,
}

impl Default for TimelineEvent {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            timestamp_type: String::new(),
            description: String::new(),
            source: String::new(),
            artifact: None,
            evidence_id: None,
            significance: None,
        }
    }
}

/// Hash record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HashRecord {
    /// Item being hashed (also accepts item_reference from frontend)
    #[serde(alias = "item_reference")]
    pub item: String,
    /// Hash algorithm
    #[serde(deserialize_with = "deserialize_hash_algorithm")]
    pub algorithm: HashAlgorithm,
    /// Hash value (hex string)
    pub value: String,
    /// When the hash was computed (also accepts timestamp from frontend)
    #[serde(default, alias = "timestamp", deserialize_with = "deserialize_datetime_opt")]
    pub computed_at: Option<DateTime<Utc>>,
    /// Verification status
    #[serde(default)]
    pub verified: Option<bool>,
}

impl Default for HashRecord {
    fn default() -> Self {
        Self {
            item: String::new(),
            algorithm: HashAlgorithm::SHA256,
            value: String::new(),
            computed_at: None,
            verified: None,
        }
    }
}

/// Flexible hash algorithm deserializer that accepts enum or string
fn deserialize_hash_algorithm<'de, D>(deserializer: D) -> Result<HashAlgorithm, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AlgoOrString {
        Algo(HashAlgorithm),
        String(String),
    }
    
    match AlgoOrString::deserialize(deserializer)? {
        AlgoOrString::Algo(a) => Ok(a),
        AlgoOrString::String(s) => match s.to_uppercase().as_str() {
            "MD5" => Ok(HashAlgorithm::MD5),
            "SHA1" | "SHA-1" => Ok(HashAlgorithm::SHA1),
            "SHA256" | "SHA-256" => Ok(HashAlgorithm::SHA256),
            "SHA512" | "SHA-512" => Ok(HashAlgorithm::SHA512),
            "BLAKE2B" | "BLAKE2" => Ok(HashAlgorithm::Blake2b),
            "BLAKE3" => Ok(HashAlgorithm::Blake3),
            "XXH3" => Ok(HashAlgorithm::XXH3),
            "XXH64" => Ok(HashAlgorithm::XXH64),
            _ => Err(D::Error::custom(format!("Unknown hash algorithm: {}", s))),
        },
    }
}

/// Supported hash algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    MD5,
    SHA1,
    SHA256,
    SHA512,
    Blake2b,
    Blake3,
    XXH3,
    XXH64,
}

impl HashAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::MD5 => "MD5",
            HashAlgorithm::SHA1 => "SHA-1",
            HashAlgorithm::SHA256 => "SHA-256",
            HashAlgorithm::SHA512 => "SHA-512",
            HashAlgorithm::Blake2b => "BLAKE2b",
            HashAlgorithm::Blake3 => "BLAKE3",
            HashAlgorithm::XXH3 => "XXH3",
            HashAlgorithm::XXH64 => "XXH64",
        }
    }
}

/// Tool/software information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Version
    pub version: String,
    /// Vendor
    pub vendor: Option<String>,
    /// Purpose in the examination
    pub purpose: Option<String>,
}

/// Appendix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appendix {
    /// Appendix identifier (A, B, C, etc.)
    pub appendix_id: String,
    /// Title
    pub title: String,
    /// Content type
    pub content_type: AppendixType,
    /// Content (markdown or path to file)
    pub content: String,
}

/// Signature/approval record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRecord {
    /// Role of signer
    pub role: String,
    /// Signer name
    pub name: String,
    /// Signature data (base64 or text)
    #[serde(default)]
    pub signature: Option<String>,
    /// Date signed
    #[serde(default)]
    pub signed_date: Option<String>,
    /// Additional notes
    #[serde(default)]
    pub notes: Option<String>,
    /// Whether digitally certified
    #[serde(default)]
    pub certified: Option<bool>,
}

/// Types of appendix content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppendixType {
    /// Markdown text
    Markdown,
    /// Raw text
    Text,
    /// File listing
    FileListing,
    /// Hash table
    HashTable,
    /// External file reference
    FileReference,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Classification
    // =========================================================================

    #[test]
    fn test_classification_as_str_all_variants() {
        assert_eq!(Classification::Public.as_str(), "PUBLIC");
        assert_eq!(Classification::Internal.as_str(), "INTERNAL");
        assert_eq!(Classification::Confidential.as_str(), "CONFIDENTIAL");
        assert_eq!(Classification::Restricted.as_str(), "RESTRICTED");
        assert_eq!(Classification::LawEnforcementSensitive.as_str(), "LAW ENFORCEMENT SENSITIVE");
    }

    #[test]
    fn test_classification_equality() {
        assert_eq!(Classification::Public, Classification::Public);
        assert_ne!(Classification::Public, Classification::Internal);
    }

    #[test]
    fn test_classification_serialization() {
        let json = serde_json::to_string(&Classification::Confidential).unwrap();
        let back: Classification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Classification::Confidential);
    }

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
        assert_eq!(FindingCategory::InternetHistory.as_str(), "Internet History");
        assert_eq!(FindingCategory::Communication.as_str(), "Communication");
        assert_eq!(FindingCategory::Documents.as_str(), "Documents");
        assert_eq!(FindingCategory::Media.as_str(), "Media Files");
        assert_eq!(FindingCategory::DeletedData.as_str(), "Deleted Data");
        assert_eq!(FindingCategory::AntiForensics.as_str(), "Anti-Forensics");
        assert_eq!(FindingCategory::Malware.as_str(), "Malware/Suspicious Software");
        assert_eq!(FindingCategory::SystemEvents.as_str(), "System Events");
        assert_eq!(FindingCategory::NetworkActivity.as_str(), "Network Activity");
        assert_eq!(FindingCategory::ExternalDevices.as_str(), "External Devices");
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
    // HashAlgorithm
    // =========================================================================

    #[test]
    fn test_hash_algorithm_as_str_all_variants() {
        assert_eq!(HashAlgorithm::MD5.as_str(), "MD5");
        assert_eq!(HashAlgorithm::SHA1.as_str(), "SHA-1");
        assert_eq!(HashAlgorithm::SHA256.as_str(), "SHA-256");
        assert_eq!(HashAlgorithm::SHA512.as_str(), "SHA-512");
        assert_eq!(HashAlgorithm::Blake2b.as_str(), "BLAKE2b");
        assert_eq!(HashAlgorithm::Blake3.as_str(), "BLAKE3");
        assert_eq!(HashAlgorithm::XXH3.as_str(), "XXH3");
        assert_eq!(HashAlgorithm::XXH64.as_str(), "XXH64");
    }

    #[test]
    fn test_hash_algorithm_serialization_roundtrip() {
        for algo in [
            HashAlgorithm::MD5,
            HashAlgorithm::SHA1,
            HashAlgorithm::SHA256,
            HashAlgorithm::SHA512,
            HashAlgorithm::Blake2b,
            HashAlgorithm::Blake3,
            HashAlgorithm::XXH3,
            HashAlgorithm::XXH64,
        ] {
            let json = serde_json::to_string(&algo).unwrap();
            let back: HashAlgorithm = serde_json::from_str(&json).unwrap();
            assert_eq!(back, algo);
        }
    }

    // =========================================================================
    // deserialize_hash_algorithm
    // =========================================================================

    #[test]
    fn test_deserialize_hash_algorithm_from_string_variants() {
        // Test via HashRecord deserialization
        let cases = [
            ("MD5", HashAlgorithm::MD5),
            ("SHA1", HashAlgorithm::SHA1),
            ("SHA-1", HashAlgorithm::SHA1),
            ("SHA256", HashAlgorithm::SHA256),
            ("SHA-256", HashAlgorithm::SHA256),
            ("SHA512", HashAlgorithm::SHA512),
            ("SHA-512", HashAlgorithm::SHA512),
            ("BLAKE2B", HashAlgorithm::Blake2b),
            ("BLAKE2", HashAlgorithm::Blake2b),
            ("BLAKE3", HashAlgorithm::Blake3),
            ("XXH3", HashAlgorithm::XXH3),
            ("XXH64", HashAlgorithm::XXH64),
        ];
        for (input, expected) in cases {
            let json = format!(r#"{{"item":"test","algorithm":"{}","value":"abc"}}"#, input);
            let record: HashRecord = serde_json::from_str(&json).unwrap();
            assert_eq!(record.algorithm, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_deserialize_hash_algorithm_unknown_fails() {
        let json = r#"{"item":"test","algorithm":"UNKNOWN","value":"abc"}"#;
        let result: Result<HashRecord, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // =========================================================================
    // ForensicReport builder
    // =========================================================================

    #[test]
    fn test_report_builder_basic() {
        let report = ForensicReport::builder()
            .case_number("2026-001")
            .examiner_name("John Doe")
            .add_evidence(EvidenceItem {
                evidence_id: "E001".to_string(),
                description: "Test drive".to_string(),
                evidence_type: EvidenceType::HardDrive,
                ..Default::default()
            })
            .executive_summary("This is a test report.")
            .build()
            .expect("Failed to build report");

        assert_eq!(report.case_info.case_number, "2026-001");
        assert_eq!(report.examiner.name, "John Doe");
        assert_eq!(report.evidence_items.len(), 1);
        assert!(report.executive_summary.is_some());
    }

    #[test]
    fn test_report_builder_auto_metadata() {
        let report = ForensicReport::builder()
            .case_number("CASE-42")
            .examiner_name("Jane")
            .add_evidence(EvidenceItem::default())
            .build()
            .unwrap();

        assert!(report.metadata.title.contains("CASE-42"));
        assert!(report.metadata.report_number.contains("CASE-42"));
        assert_eq!(report.metadata.version, "1.0");
    }

    #[test]
    fn test_report_builder_fails_without_case_info() {
        let result = ForensicReport::builder()
            .examiner_name("Jane")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Case info"));
    }

    #[test]
    fn test_report_builder_fails_without_examiner() {
        let result = ForensicReport::builder()
            .case_number("C-1")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Examiner"));
    }

    #[test]
    fn test_report_builder_all_fields() {
        let report = ForensicReport::builder()
            .case_number("FULL-001")
            .examiner_name("Dr. Smith")
            .scope("Full disk analysis")
            .methodology("Standard forensic methodology")
            .conclusions("No significant findings")
            .notes("Additional notes here")
            .add_custody_record(CustodyRecord::new("E1", "Alice", "Bob"))
            .add_finding(Finding::new("F1", "Test finding", "Details"))
            .add_hash_record(HashRecord::default())
            .add_tool(ToolInfo {
                name: "FFX".to_string(),
                version: "1.0".to_string(),
                vendor: None,
                purpose: None,
            })
            .add_appendix(Appendix {
                appendix_id: "A".to_string(),
                title: "File listing".to_string(),
                content_type: AppendixType::FileListing,
                content: "list".to_string(),
            })
            .add_evidence(EvidenceItem::default())
            .build()
            .unwrap();

        assert_eq!(report.scope.unwrap(), "Full disk analysis");
        assert_eq!(report.methodology.unwrap(), "Standard forensic methodology");
        assert_eq!(report.conclusions.unwrap(), "No significant findings");
        assert_eq!(report.notes.unwrap(), "Additional notes here");
        assert_eq!(report.chain_of_custody.len(), 1);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.hash_records.len(), 1);
        assert_eq!(report.tools.len(), 1);
        assert_eq!(report.appendices.len(), 1);
    }

    #[test]
    fn test_report_builder_evidence_items_batch() {
        let items = vec![
            EvidenceItem { evidence_id: "E1".into(), ..Default::default() },
            EvidenceItem { evidence_id: "E2".into(), ..Default::default() },
        ];
        let report = ForensicReport::builder()
            .case_number("B-1")
            .examiner_name("X")
            .evidence_items(items)
            .build()
            .unwrap();
        assert_eq!(report.evidence_items.len(), 2);
    }

    #[test]
    fn test_report_builder_findings_batch() {
        let findings = vec![
            Finding::new("F1", "A", "a"),
            Finding::new("F2", "B", "b"),
            Finding::new("F3", "C", "c"),
        ];
        let report = ForensicReport::builder()
            .case_number("B-2")
            .examiner_name("X")
            .findings(findings)
            .add_evidence(EvidenceItem::default())
            .build()
            .unwrap();
        assert_eq!(report.findings.len(), 3);
    }

    // =========================================================================
    // ForensicReport::validate
    // =========================================================================

    #[test]
    fn test_report_validate_passes_with_all_required() {
        let report = ForensicReport::builder()
            .case_number("V-001")
            .examiner_name("Examiner")
            .add_evidence(EvidenceItem::default())
            .build()
            .unwrap();

        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_report_validate_fails_empty_case_number() {
        let mut report = ForensicReport::builder()
            .case_number("X")
            .examiner_name("Y")
            .add_evidence(EvidenceItem::default())
            .build()
            .unwrap();
        report.case_info.case_number = String::new();

        let errors = report.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Case number")));
    }

    #[test]
    fn test_report_validate_fails_empty_examiner() {
        let mut report = ForensicReport::builder()
            .case_number("X")
            .examiner_name("Y")
            .add_evidence(EvidenceItem::default())
            .build()
            .unwrap();
        report.examiner.name = String::new();

        let errors = report.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Examiner")));
    }

    #[test]
    fn test_report_validate_fails_no_evidence() {
        let report = ForensicReport::builder()
            .case_number("X")
            .examiner_name("Y")
            .build()
            .unwrap();

        let errors = report.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("evidence item")));
    }

    #[test]
    fn test_report_validate_multiple_errors() {
        let report = ForensicReport::default();
        let errors = report.validate().unwrap_err();
        // Missing case number, examiner name, and evidence items
        assert!(errors.len() >= 3);
    }

    // =========================================================================
    // CustodyRecord builder
    // =========================================================================

    #[test]
    fn test_custody_record_builder() {
        let record = CustodyRecord::new("E001", "Alice", "Bob")
            .with_purpose("Analysis")
            .with_location("Lab 1")
            .with_notes("Sealed bag");

        assert_eq!(record.evidence_id, "E001");
        assert_eq!(record.released_by, "Alice");
        assert_eq!(record.received_by, "Bob");
        assert_eq!(record.purpose.unwrap(), "Analysis");
        assert_eq!(record.location.unwrap(), "Lab 1");
        assert_eq!(record.notes.unwrap(), "Sealed bag");
    }

    #[test]
    fn test_custody_record_with_timestamp() {
        let ts = Utc::now();
        let record = CustodyRecord::new("E1", "A", "B").with_timestamp(ts);
        assert_eq!(record.timestamp, ts);
    }

    // =========================================================================
    // Finding builder
    // =========================================================================

    #[test]
    fn test_finding_builder() {
        let finding = Finding::new("F001", "Deleted files found", "Multiple deleted files recovered")
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
    // ReportMetadata defaults
    // =========================================================================

    #[test]
    fn test_report_metadata_default() {
        let meta = ReportMetadata::default();
        assert_eq!(meta.version, "1.0");
        assert_eq!(meta.generated_by, "CORE-FFX");
        assert_eq!(meta.classification, Classification::Internal);
        assert!(meta.title.is_empty());
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

    // =========================================================================
    // HashRecord defaults and deserialization
    // =========================================================================

    #[test]
    fn test_hash_record_default() {
        let record = HashRecord::default();
        assert_eq!(record.algorithm, HashAlgorithm::SHA256);
        assert!(record.item.is_empty());
        assert!(record.verified.is_none());
    }

    #[test]
    fn test_hash_record_item_reference_alias() {
        let json = r#"{"item_reference":"disk.E01","algorithm":"MD5","value":"abc123"}"#;
        let record: HashRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.item, "disk.E01");
    }

    // =========================================================================
    // deserialize_datetime_flexible
    // =========================================================================

    #[test]
    fn test_deserialize_datetime_flexible_rfc3339() {
        let json = r#"{"title":"","report_number":"","version":"1.0","classification":"Internal","generated_at":"2025-06-15T10:30:00Z","generated_by":"test"}"#;
        let meta: ReportMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.generated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "2025-06-15T10:30:00Z");
    }

    #[test]
    fn test_deserialize_datetime_flexible_without_timezone() {
        let json = r#"{"title":"","report_number":"","version":"1.0","classification":"Internal","generated_at":"2025-06-15T10:30:00","generated_by":"test"}"#;
        let meta: ReportMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.generated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "2025-06-15T10:30:00Z");
    }

    // =========================================================================
    // ForensicReport serialization
    // =========================================================================

    #[test]
    fn test_forensic_report_serialization_roundtrip() {
        let report = ForensicReport::builder()
            .case_number("SER-001")
            .examiner_name("Tester")
            .add_evidence(EvidenceItem::default())
            .build()
            .unwrap();

        let json = serde_json::to_string(&report).unwrap();
        let back: ForensicReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.case_info.case_number, "SER-001");
        assert_eq!(back.examiner.name, "Tester");
    }

    // =========================================================================
    // ExhibitType, AppendixType equality
    // =========================================================================

    #[test]
    fn test_exhibit_type_equality() {
        assert_eq!(ExhibitType::Screenshot, ExhibitType::Screenshot);
        assert_ne!(ExhibitType::Screenshot, ExhibitType::HexDump);
    }

    #[test]
    fn test_appendix_type_equality() {
        assert_eq!(AppendixType::Markdown, AppendixType::Markdown);
        assert_ne!(AppendixType::Markdown, AppendixType::Text);
    }

    // =========================================================================
    // TimelineEvent default
    // =========================================================================

    #[test]
    fn test_timeline_event_default() {
        let event = TimelineEvent::default();
        assert!(event.timestamp_type.is_empty());
        assert!(event.description.is_empty());
        assert!(event.artifact.is_none());
    }

    // =========================================================================
    // SignatureRecord serialization
    // =========================================================================

    #[test]
    fn test_signature_record_serialization() {
        let sig = SignatureRecord {
            role: "Lead Examiner".to_string(),
            name: "Jane Doe".to_string(),
            signature: None,
            signed_date: Some("2026-01-15".to_string()),
            notes: None,
            certified: Some(true),
        };
        let json = serde_json::to_string(&sig).unwrap();
        let back: SignatureRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.role, "Lead Examiner");
        assert_eq!(back.certified, Some(true));
    }
}
