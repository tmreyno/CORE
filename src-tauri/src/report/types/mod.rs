// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Data structures for forensic reports
//!
//! This module defines all the types used to represent forensic report data.
//! These structures are serializable and can be used with templates and output generators.

pub mod case;
pub mod evidence_collection;
pub mod findings;
pub mod records;

pub use case::*;
pub use evidence_collection::*;
pub use findings::*;
pub use records::*;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

/// Parse a datetime string into `DateTime<Utc>`, trying many forensic-relevant formats.
///
/// Formats tried (in order):
/// 1. RFC 3339 / ISO 8601 with timezone  (`2024-01-15T10:30:00Z`, `2024-01-15T10:30:00+05:00`)
/// 2. ISO 8601 with fractional seconds   (`2024-01-15T10:30:00.123Z`)
/// 3. ISO 8601 without timezone           (`2024-01-15T10:30:00`)
/// 4. Space-separated, zero-padded        (`2024-01-15 10:30:00`)
/// 5. Space-separated, non-padded         (`2004-9-22 9:6:4`)   ← common in EWF acquiry_date
/// 6. Date only, zero-padded              (`2024-01-15`)
/// 7. Date only, non-padded               (`2004-9-22`)
/// 8. US-style date                       (`01/15/2024`, `1/15/2024`)
/// 9. US-style datetime                   (`01/15/2024 10:30:00`)
fn parse_datetime_string(s: &str) -> Result<DateTime<Utc>, String> {
    let s = s.trim();

    // 1. RFC 3339 / ISO 8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // 2-3. ISO 8601 with 'T' separator (with/without fractional seconds and trailing Z)
    for fmt in &["%Y-%m-%dT%H:%M:%S%.fZ", "%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(ndt.and_utc());
        }
    }

    // 4-5. Space-separated datetime (zero-padded and non-padded)
    //      chrono's %m/%d/%H/%M/%S accept non-padded values during parsing,
    //      so "%Y-%m-%d %H:%M:%S" handles both "2024-01-15 10:30:00" and "2004-9-22 9:6:4".
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(ndt.and_utc());
    }

    // 6-7. Date only (zero-padded and non-padded)
    if let Ok(nd) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(nd.and_hms_opt(0, 0, 0).expect("midnight is valid").and_utc());
    }

    // 8-9. US-style date / datetime (common in some forensic tools)
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S") {
        return Ok(ndt.and_utc());
    }
    if let Ok(nd) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        return Ok(nd.and_hms_opt(0, 0, 0).expect("midnight is valid").and_utc());
    }

    Err(format!("Invalid datetime '{}': no recognized format matched", s))
}

/// Flexible datetime deserializer that handles ISO strings and forensic date formats
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
        DateTimeOrString::String(s) => parse_datetime_string(&s).map_err(D::Error::custom),
    }
}

/// Optional datetime deserializer — returns `None` for null / empty strings
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
        Some(DateTimeOrString::String(s)) if s.trim().is_empty() => Ok(None),
        Some(DateTimeOrString::String(s)) => {
            parse_datetime_string(&s).map(Some).map_err(D::Error::custom)
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
    /// Report type — determines output structure (e.g., "chain_of_custody", "evidence_collection")
    #[serde(default)]
    pub report_type: Option<String>,
    /// Chain of Custody items (per-evidence EPA CID Form 7-01 data)
    #[serde(default)]
    pub coc_items: Option<Vec<CocItem>>,
    /// Evidence Collection Report data (EPA CID Computer Forensics Lab form)
    #[serde(default)]
    pub evidence_collection: Option<EvidenceCollectionData>,
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
            report_type: None,
            coc_items: None,
            evidence_collection: None,
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
    // AppendixType equality
    // =========================================================================

    #[test]
    fn test_appendix_type_equality() {
        assert_eq!(AppendixType::Markdown, AppendixType::Markdown);
        assert_ne!(AppendixType::Markdown, AppendixType::Text);
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

    #[test]
    fn test_deserialize_datetime_space_separated() {
        // Zero-padded, space separator (common in some forensic tools)
        let json = r#"{"title":"","report_number":"","version":"1.0","classification":"Internal","generated_at":"2025-06-15 10:30:00","generated_by":"test"}"#;
        let meta: ReportMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.generated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "2025-06-15T10:30:00Z");
    }

    #[test]
    fn test_deserialize_datetime_nonpadded_ewf_acquiry_date() {
        // Non-zero-padded, space separator — EWF acquiry_date format ("2004-9-22 9:6:4")
        let json = r#"{"title":"","report_number":"","version":"1.0","classification":"Internal","generated_at":"2004-9-22 9:6:4","generated_by":"test"}"#;
        let meta: ReportMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.generated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "2004-09-22T09:06:04Z");
    }

    #[test]
    fn test_deserialize_datetime_opt_nonpadded() {
        // Non-padded datetime through the optional deserializer (used by acquisition_date, etc.)
        let json = r#"{"evidence_id":"EV001","description":"test","evidence_type":"HardDrive","acquisition_date":"2004-9-22 9:6:4"}"#;
        let item: EvidenceItem = serde_json::from_str(json).unwrap();
        assert!(item.received_date.is_some());
        assert_eq!(
            item.received_date.unwrap().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2004-09-22T09:06:04Z"
        );
    }

    #[test]
    fn test_deserialize_datetime_opt_date_only() {
        let json = r#"{"evidence_id":"EV001","description":"test","evidence_type":"HardDrive","acquisition_date":"2004-9-22"}"#;
        let item: EvidenceItem = serde_json::from_str(json).unwrap();
        assert!(item.received_date.is_some());
        assert_eq!(
            item.received_date.unwrap().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2004-09-22T00:00:00Z"
        );
    }

    #[test]
    fn test_parse_datetime_string_helper() {
        // Direct tests on the parse_datetime_string helper
        assert!(parse_datetime_string("2024-01-15T10:30:00Z").is_ok());
        assert!(parse_datetime_string("2024-01-15T10:30:00").is_ok());
        assert!(parse_datetime_string("2024-01-15 10:30:00").is_ok());
        assert!(parse_datetime_string("2004-9-22 9:6:4").is_ok());
        assert!(parse_datetime_string("2024-01-15").is_ok());
        assert!(parse_datetime_string("2004-9-22").is_ok());
        assert!(parse_datetime_string("01/15/2024").is_ok());
        assert!(parse_datetime_string("1/5/2024 9:06:04").is_ok());
        assert!(parse_datetime_string("").is_err());
        assert!(parse_datetime_string("not-a-date").is_err());
    }
}
