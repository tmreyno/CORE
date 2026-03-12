// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Case document discovery for forensic workflows
//!
//! This module handles discovery of case-related documents such as:
//! - Chain of Custody (COC) forms
//! - Case intake forms
//! - Examiner notes
//! - Evidence submission forms
//!
//! # Typical folder structures:
//! ```text
//! Case Folder/
//! ├── 1.Evidence/
//! ├── 2.Processed.Database/
//! ├── 3.Reports/
//! └── 4.Case.Documents/
//!     ├── 25-049_COC_25-06988-08.pdf
//!     ├── Evidence_Intake_Form.pdf
//!     └── Case_Notes.docx
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Types of case documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseDocumentType {
    /// Chain of Custody form
    ChainOfCustody,
    /// Evidence intake/submission form
    EvidenceIntake,
    /// Case notes or examiner notes
    CaseNotes,
    /// Evidence receipt
    EvidenceReceipt,
    /// Lab request form
    LabRequest,
    /// Forensic examination report (external)
    ExternalReport,
    /// Other case document
    Other,
}

impl CaseDocumentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaseDocumentType::ChainOfCustody => "Chain of Custody",
            CaseDocumentType::EvidenceIntake => "Evidence Intake",
            CaseDocumentType::CaseNotes => "Case Notes",
            CaseDocumentType::EvidenceReceipt => "Evidence Receipt",
            CaseDocumentType::LabRequest => "Lab Request",
            CaseDocumentType::ExternalReport => "External Report",
            CaseDocumentType::Other => "Other Document",
        }
    }
}

/// A discovered case document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseDocument {
    /// Full path to the document
    pub path: String,
    /// Filename
    pub filename: String,
    /// Document type
    pub document_type: CaseDocumentType,
    /// File size in bytes
    pub size: u64,
    /// File format (e.g., "PDF", "DOCX", "TXT")
    pub format: String,
    /// Case number extracted from filename (if found)
    pub case_number: Option<String>,
    /// Evidence ID extracted from filename (if found)
    pub evidence_id: Option<String>,
    /// Last modified timestamp (ISO 8601)
    pub modified: Option<String>,
}

/// Configuration for case document search
#[derive(Debug, Clone, Default)]
pub struct CaseDocumentSearchConfig {
    /// Search recursively in subdirectories
    pub recursive: bool,
    /// Specific document types to search for (empty = all types)
    pub document_types: Vec<CaseDocumentType>,
    /// Maximum depth for recursive search (0 = unlimited)
    pub max_depth: usize,
    /// Preview mode - skip content-based detection for faster results
    pub preview_only: bool,
}

/// Patterns for identifying chain of custody forms
const COC_PATTERNS: &[&str] = &[
    r"(?i)[_\-]coc[_\-]",       // _COC_ or -COC- anywhere in filename
    r"(?i)^coc[_\-]",           // COC_ or COC- at start of filename
    r"(?i)chain.*custody",      // chain of custody, chain-of-custody, etc.
    r"(?i)coc[-_]?\d+",         // COC-123, COC_123, COC123
    r"(?i)custody[-_]form",     // custody_form, custody-form
    r"(?i)evidence[-_]custody", // evidence_custody, evidence-custody
    r"(?i)^coc\.pdf$",          // Just "COC.pdf"
];

/// Patterns for evidence intake/submission forms
const INTAKE_PATTERNS: &[&str] = &[
    r"(?i)intake[-_]form",
    r"(?i)evidence[-_]intake",
    r"(?i)submission[-_]form",
    r"(?i)evidence[-_]submit",
    r"(?i)request[-_]form",
];

/// Patterns for case notes
const NOTES_PATTERNS: &[&str] = &[
    r"(?i)case[-_]notes?",
    r"(?i)examiner[-_]notes?",
    r"(?i)lab[-_]notes?",
    r"(?i)investigation[-_]notes?",
];

/// Patterns for evidence receipts
const RECEIPT_PATTERNS: &[&str] = &[
    r"(?i)evidence[-_]receipt",
    r"(?i)property[-_]receipt",
    r"(?i)item[-_]receipt",
];

/// Case number extraction pattern
const CASE_NUMBER_PATTERN: &str = r"(\d{2,4}[-_]?\d{2,6})";

/// Evidence ID extraction pattern  
const EVIDENCE_ID_PATTERN: &str = r"(?i)(?:EV|E|ITEM)[-_]?(\d+)";

/// Folder names that typically contain case documents
/// These patterns match against the lowercased folder name
const CASE_DOCUMENT_FOLDERS: &[&str] = &[
    "case.documents",
    "case documents",
    "casedocuments",
    "case_documents",
    "documents",
    "forms",
    "paperwork",
    "intake",
    "custody",
    "chain of custody",
    "coc",
    "case.notes",
    "case notes",
    "casenotes",
    "case_notes",
];

/// Find case documents in a directory
///
/// This function searches for common case document types including COC forms,
/// evidence intake forms, and case notes.
pub fn find_case_documents(
    base_path: &str,
    config: &CaseDocumentSearchConfig,
) -> Vec<CaseDocument> {
    let path = Path::new(base_path);
    if !path.exists() || !path.is_dir() {
        debug!(
            "Case document search: path does not exist or is not a directory: {}",
            base_path
        );
        return Vec::new();
    }

    let mut documents = Vec::new();
    find_documents_recursive(path, config, 0, &mut documents);

    // Sort by document type then by filename
    documents.sort_by(|a, b| match (&a.document_type, &b.document_type) {
        (CaseDocumentType::ChainOfCustody, CaseDocumentType::ChainOfCustody) => {
            a.filename.cmp(&b.filename)
        }
        (CaseDocumentType::ChainOfCustody, _) => std::cmp::Ordering::Less,
        (_, CaseDocumentType::ChainOfCustody) => std::cmp::Ordering::Greater,
        _ => a.filename.cmp(&b.filename),
    });

    documents
}

/// Search for case document folders relative to an evidence path
///
/// Given an evidence file path, this function looks for case document folders
/// in parent directories following common forensic folder structures.
pub fn find_case_document_folders(evidence_path: &str) -> Vec<PathBuf> {
    let path = Path::new(evidence_path);
    let mut folders = Vec::new();

    // Get the parent directory (if evidence_path is a file)
    let start_dir = if path.is_file() {
        path.parent()
    } else {
        Some(path)
    };

    let Some(start) = start_dir else {
        return folders;
    };

    // Search up to 5 levels up for case document folders (to handle deep evidence paths)
    let mut current = start.to_path_buf();
    for level in 0..5 {
        debug!(
            "Searching for case doc folders at level {}: {:?}",
            level, current
        );
        // Check if this directory contains any case document folder
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let dir_name = entry_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();

                    for pattern in CASE_DOCUMENT_FOLDERS {
                        if dir_name.contains(pattern) || pattern.contains(&dir_name) {
                            debug!(
                                "Found case doc folder: {:?} (matched pattern: {})",
                                entry_path, pattern
                            );
                            folders.push(entry_path.clone());
                            break;
                        }
                    }
                }
            }
        }

        // Move up one directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    folders
}

/// Find Chain of Custody (COC) forms specifically
pub fn find_coc_forms(base_path: &str, recursive: bool) -> Vec<CaseDocument> {
    let config = CaseDocumentSearchConfig {
        recursive,
        document_types: vec![CaseDocumentType::ChainOfCustody],
        max_depth: if recursive { 5 } else { 0 },
        preview_only: false,
    };

    find_case_documents(base_path, &config)
}

fn find_documents_recursive(
    dir: &Path,
    config: &CaseDocumentSearchConfig,
    depth: usize,
    results: &mut Vec<CaseDocument>,
) {
    // Check depth limit
    if config.max_depth > 0 && depth > config.max_depth {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            debug!("Failed to read directory {:?}: {}", dir, e);
            return;
        }
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();

        if entry_path.is_dir() {
            if config.recursive {
                find_documents_recursive(&entry_path, config, depth + 1, results);
            }
            continue;
        }

        if !entry_path.is_file() {
            continue;
        }

        let filename = match entry_path.file_name().and_then(|n| n.to_str()) {
            Some(f) => f.to_string(),
            None => continue,
        };

        let lower = filename.to_lowercase();

        // Check if it's a supported document format
        let format = match detect_document_format(&lower) {
            Some(f) => f,
            None => continue, // Skip non-document files
        };

        // Detect document type - first by filename, then by content if needed
        let document_type =
            detect_document_type_with_content(&filename, &entry_path, &format, config.preview_only);

        // Filter by requested document types
        if !config.document_types.is_empty() && !config.document_types.contains(&document_type) {
            continue;
        }

        // Get file metadata
        let metadata = entry.metadata().ok();
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata.and_then(|m| m.modified().ok()).map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.to_rfc3339()
        });

        // Extract case number and evidence ID from filename
        let case_number = extract_case_number(&filename);
        let evidence_id = extract_evidence_id(&filename);

        let path_str = entry_path.to_string_lossy().to_string();

        debug!(
            "Found case document: {} (type: {:?})",
            filename, document_type
        );

        results.push(CaseDocument {
            path: path_str,
            filename,
            document_type,
            size,
            format,
            case_number,
            evidence_id,
            modified,
        });
    }
}

fn detect_document_format(filename_lower: &str) -> Option<String> {
    if filename_lower.ends_with(".pdf") {
        Some("PDF".to_string())
    } else if filename_lower.ends_with(".docx") {
        Some("DOCX".to_string())
    } else if filename_lower.ends_with(".doc") {
        Some("DOC".to_string())
    } else if filename_lower.ends_with(".xlsx") {
        Some("XLSX".to_string())
    } else if filename_lower.ends_with(".xls") {
        Some("XLS".to_string())
    } else if filename_lower.ends_with(".txt") {
        Some("TXT".to_string())
    } else if filename_lower.ends_with(".rtf") {
        Some("RTF".to_string())
    } else if filename_lower.ends_with(".odt") {
        Some("ODT".to_string())
    } else if filename_lower.ends_with(".png")
        || filename_lower.ends_with(".jpg")
        || filename_lower.ends_with(".jpeg")
        || filename_lower.ends_with(".tif")
        || filename_lower.ends_with(".tiff")
    {
        Some("IMAGE".to_string())
    } else {
        None
    }
}

/// Detect document type by filename first, then check content if type is "Other"
fn detect_document_type_with_content(
    filename: &str,
    path: &Path,
    format: &str,
    preview_only: bool,
) -> CaseDocumentType {
    // First try filename-based detection
    let doc_type = detect_document_type_by_filename(filename);

    // If we found a specific type from filename, use it
    if doc_type != CaseDocumentType::Other {
        return doc_type;
    }

    // In preview mode, skip expensive content-based detection
    if preview_only {
        return doc_type;
    }

    // For "Other" types, check file content for COC patterns
    // This helps identify COC forms with generic filenames
    if let Some(content_type) = detect_coc_by_content(path, format) {
        return content_type;
    }

    doc_type
}

/// Patterns to search for in file content (case-insensitive)
const CONTENT_COC_PATTERNS: &[&str] = &[
    "chain of custody",
    "chain-of-custody",
    "chainofcustody",
    "evidence custody",
    "custody form",
    "custody record",
    "property receipt",
    "evidence tracking",
    "item received from",
    "released to",
    "received by",
    "relinquished by",
    "custody transfer",
];

/// Check file content for COC-related text patterns
/// Reads first 32KB of file and searches for patterns
fn detect_coc_by_content(path: &Path, format: &str) -> Option<CaseDocumentType> {
    // Only check certain file types where we can reliably extract text
    match format {
        "PDF" => detect_coc_in_pdf(path),
        "TXT" | "RTF" => detect_coc_in_text_file(path),
        "DOC" | "DOCX" => detect_coc_in_office_file(path),
        _ => None,
    }
}

/// Check PDF content for COC patterns
/// Uses pdf-extract to properly extract text from compressed PDF streams
fn detect_coc_in_pdf(path: &Path) -> Option<CaseDocumentType> {
    // First try proper PDF text extraction (handles compressed streams)
    #[cfg(feature = "flavor-review")]
    {
        if let Ok(text) = pdf_extract::extract_text(path) {
            let content = text.to_lowercase();

            // Search for COC patterns in extracted text
            for pattern in CONTENT_COC_PATTERNS {
                if content.contains(pattern) {
                    debug!(
                        "Found COC pattern '{}' in extracted PDF text: {:?}",
                        pattern, path
                    );
                    return Some(CaseDocumentType::ChainOfCustody);
                }
            }

            // Check for "COC" as a standalone term (common abbreviation)
            if content.contains(" coc ")
                || content.contains("(coc)")
                || content.contains("[coc]")
                || content.contains("\ncoc\n")
                || content.contains("\ncoc ")
                || content.contains(" coc\n")
            {
                debug!("Found 'COC' abbreviation in extracted PDF text: {:?}", path);
                return Some(CaseDocumentType::ChainOfCustody);
            }

            return None;
        }
    }

    // Fallback: raw byte search for PDFs that fail extraction
    // (e.g., encrypted PDFs, malformed PDFs, or scanned images)
    use std::io::Read;

    let mut file = fs::File::open(path).ok()?;
    let mut buffer = vec![0u8; 64 * 1024]; // Read up to 64KB
    let bytes_read = file.read(&mut buffer).ok()?;
    buffer.truncate(bytes_read);

    // Convert to lowercase string for searching (lossy for binary content)
    let content = String::from_utf8_lossy(&buffer).to_lowercase();

    // Search for COC patterns in raw bytes (catches metadata and uncompressed text)
    for pattern in CONTENT_COC_PATTERNS {
        if content.contains(pattern) {
            debug!(
                "Found COC pattern '{}' in PDF raw bytes (fallback): {:?}",
                pattern, path
            );
            return Some(CaseDocumentType::ChainOfCustody);
        }
    }

    // Also check for "COC" as a standalone term
    if content.contains(" coc ")
        || content.contains("(coc)")
        || content.contains("[coc]")
        || content.contains("\ncoc\n")
        || content.contains("\ncoc ")
        || content.contains(" coc\n")
        || content.contains(">coc<")
        || content.contains("/coc/")
    {
        debug!(
            "Found 'COC' abbreviation in PDF raw bytes (fallback): {:?}",
            path
        );
        return Some(CaseDocumentType::ChainOfCustody);
    }

    None
}

/// Check text file content for COC patterns
fn detect_coc_in_text_file(path: &Path) -> Option<CaseDocumentType> {
    use std::io::Read;

    let file = fs::File::open(path).ok()?;
    let mut buffer = String::new();

    // Read up to 32KB
    let mut limited_reader = file.take(32 * 1024);
    limited_reader.read_to_string(&mut buffer).ok()?;

    let content = buffer.to_lowercase();

    for pattern in CONTENT_COC_PATTERNS {
        if content.contains(pattern) {
            debug!("Found COC pattern '{}' in text file: {:?}", pattern, path);
            return Some(CaseDocumentType::ChainOfCustody);
        }
    }

    None
}

/// Check Office document content for COC patterns
/// DOCX files are ZIP archives with XML content
fn detect_coc_in_office_file(path: &Path) -> Option<CaseDocumentType> {
    use std::io::Read;

    // For .docx (ZIP-based), try to extract text from document.xml
    let filename = path.file_name()?.to_str()?.to_lowercase();

    if filename.ends_with(".docx") {
        if let Ok(file) = fs::File::open(path) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                // Try to read word/document.xml
                if let Ok(mut doc_xml) = archive.by_name("word/document.xml") {
                    let mut content = String::new();
                    if doc_xml.read_to_string(&mut content).is_ok() {
                        let content_lower = content.to_lowercase();
                        for pattern in CONTENT_COC_PATTERNS {
                            if content_lower.contains(pattern) {
                                debug!("Found COC pattern '{}' in DOCX: {:?}", pattern, path);
                                return Some(CaseDocumentType::ChainOfCustody);
                            }
                        }
                    }
                }
            }
        }
    } else if filename.ends_with(".doc") {
        // For legacy .doc files, do a raw byte search (less reliable but catches some)
        let mut file = fs::File::open(path).ok()?;
        let mut buffer = vec![0u8; 64 * 1024];
        let bytes_read = file.read(&mut buffer).ok()?;
        buffer.truncate(bytes_read);

        let content = String::from_utf8_lossy(&buffer).to_lowercase();
        for pattern in CONTENT_COC_PATTERNS {
            if content.contains(pattern) {
                debug!("Found COC pattern '{}' in DOC: {:?}", pattern, path);
                return Some(CaseDocumentType::ChainOfCustody);
            }
        }
    }

    None
}

fn detect_document_type_by_filename(filename: &str) -> CaseDocumentType {
    // Check for Chain of Custody patterns first (highest priority)
    for pattern in COC_PATTERNS {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(filename) {
                return CaseDocumentType::ChainOfCustody;
            }
        }
    }

    // Check for intake/submission patterns
    for pattern in INTAKE_PATTERNS {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(filename) {
                return CaseDocumentType::EvidenceIntake;
            }
        }
    }

    // Check for notes patterns
    for pattern in NOTES_PATTERNS {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(filename) {
                return CaseDocumentType::CaseNotes;
            }
        }
    }

    // Check for receipt patterns
    for pattern in RECEIPT_PATTERNS {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(filename) {
                return CaseDocumentType::EvidenceReceipt;
            }
        }
    }

    CaseDocumentType::Other
}

fn extract_case_number(filename: &str) -> Option<String> {
    let regex = Regex::new(CASE_NUMBER_PATTERN).ok()?;
    regex
        .captures(filename)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn extract_evidence_id(filename: &str) -> Option<String> {
    let regex = Regex::new(EVIDENCE_ID_PATTERN).ok()?;
    regex
        .captures(filename)
        .and_then(|c| c.get(1))
        .map(|m| format!("E{}", m.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_detect_document_type_coc() {
        assert_eq!(
            detect_document_type_by_filename("25-049_COC_25-06988-08.pdf"),
            CaseDocumentType::ChainOfCustody
        );
        assert_eq!(
            detect_document_type_by_filename("chain_of_custody.pdf"),
            CaseDocumentType::ChainOfCustody
        );
        assert_eq!(
            detect_document_type_by_filename("Chain-Of-Custody-Form.pdf"),
            CaseDocumentType::ChainOfCustody
        );
        assert_eq!(
            detect_document_type_by_filename("COC-123.pdf"),
            CaseDocumentType::ChainOfCustody
        );
        assert_eq!(
            detect_document_type_by_filename("evidence_custody_form.pdf"),
            CaseDocumentType::ChainOfCustody
        );
    }

    #[test]
    fn test_detect_document_type_intake() {
        assert_eq!(
            detect_document_type_by_filename("evidence_intake_form.pdf"),
            CaseDocumentType::EvidenceIntake
        );
        assert_eq!(
            detect_document_type_by_filename("submission_form.docx"),
            CaseDocumentType::EvidenceIntake
        );
    }

    #[test]
    fn test_detect_document_type_notes() {
        assert_eq!(
            detect_document_type_by_filename("case_notes.txt"),
            CaseDocumentType::CaseNotes
        );
        assert_eq!(
            detect_document_type_by_filename("examiner_notes.pdf"),
            CaseDocumentType::CaseNotes
        );
    }

    #[test]
    fn test_detect_document_format() {
        assert_eq!(detect_document_format("file.pdf"), Some("PDF".to_string()));
        assert_eq!(
            detect_document_format("file.docx"),
            Some("DOCX".to_string())
        );
        assert_eq!(detect_document_format("file.txt"), Some("TXT".to_string()));
        assert_eq!(detect_document_format("file.exe"), None);
        assert_eq!(detect_document_format("file.ad1"), None);
    }

    #[test]
    fn test_extract_case_number() {
        assert_eq!(
            extract_case_number("25-049_COC_25-06988-08.pdf"),
            Some("25-049".to_string())
        );
        assert_eq!(
            extract_case_number("2025-12345.pdf"),
            Some("2025-12345".to_string())
        );
        assert_eq!(extract_case_number("case_notes.pdf"), None);
    }

    #[test]
    fn test_extract_evidence_id() {
        assert_eq!(
            extract_evidence_id("EV001_form.pdf"),
            Some("E001".to_string())
        );
        assert_eq!(
            extract_evidence_id("ITEM-5_receipt.pdf"),
            Some("E5".to_string())
        );
        assert_eq!(extract_evidence_id("random_file.pdf"), None);
    }

    #[test]
    fn test_find_case_documents() {
        let temp = TempDir::new().unwrap();

        // Create test files
        let coc_path = temp.path().join("25-049_COC_123.pdf");
        let intake_path = temp.path().join("evidence_intake_form.pdf");
        let notes_path = temp.path().join("case_notes.txt");
        let other_path = temp.path().join("random.pdf");

        File::create(&coc_path).unwrap().write_all(b"test").unwrap();
        File::create(&intake_path)
            .unwrap()
            .write_all(b"test")
            .unwrap();
        File::create(&notes_path)
            .unwrap()
            .write_all(b"test")
            .unwrap();
        File::create(&other_path)
            .unwrap()
            .write_all(b"test")
            .unwrap();

        let config = CaseDocumentSearchConfig::default();
        let docs = find_case_documents(temp.path().to_str().unwrap(), &config);

        assert_eq!(docs.len(), 4);

        // COC should be first
        assert_eq!(docs[0].document_type, CaseDocumentType::ChainOfCustody);
        assert_eq!(docs[0].filename, "25-049_COC_123.pdf");
    }

    #[test]
    fn test_find_coc_forms_only() {
        let temp = TempDir::new().unwrap();

        let coc_path = temp.path().join("COC_form.pdf");
        let other_path = temp.path().join("random.pdf");

        File::create(&coc_path).unwrap().write_all(b"test").unwrap();
        File::create(&other_path)
            .unwrap()
            .write_all(b"test")
            .unwrap();

        let docs = find_coc_forms(temp.path().to_str().unwrap(), false);

        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].document_type, CaseDocumentType::ChainOfCustody);
    }

    #[test]
    fn test_content_based_coc_detection_text() {
        let temp = TempDir::new().unwrap();

        // Create a text file with generic filename but COC content
        let text_path = temp.path().join("form123.txt");
        File::create(&text_path).unwrap()
            .write_all(b"Case Document\n\nChain of Custody Record\n\nItem 1: Device iPhone 12\nReleased to: Lab Technician\n")
            .unwrap();

        // Create another text file without COC content
        let other_text = temp.path().join("notes.txt");
        File::create(&other_text)
            .unwrap()
            .write_all(b"These are just regular notes about the case.\nNothing special here.")
            .unwrap();

        let config = CaseDocumentSearchConfig::default();
        let docs = find_case_documents(temp.path().to_str().unwrap(), &config);

        // The form123.txt should be detected as COC based on content
        let coc_docs: Vec<_> = docs
            .iter()
            .filter(|d| d.document_type == CaseDocumentType::ChainOfCustody)
            .collect();

        assert_eq!(
            coc_docs.len(),
            1,
            "Should find 1 COC form via content detection"
        );
        assert_eq!(coc_docs[0].filename, "form123.txt");
    }

    #[test]
    fn test_content_based_coc_detection_pdf() {
        let temp = TempDir::new().unwrap();

        // Create a fake PDF with COC text in it
        // PDFs have text embedded - we search raw bytes
        let pdf_path = temp.path().join("document.pdf");
        File::create(&pdf_path).unwrap()
            .write_all(b"%PDF-1.4\nsome binary data...\n(Evidence Custody Form)\n(Received by: John Smith)\nendobj\n")
            .unwrap();

        let config = CaseDocumentSearchConfig::default();
        let docs = find_case_documents(temp.path().to_str().unwrap(), &config);

        let coc_docs: Vec<_> = docs
            .iter()
            .filter(|d| d.document_type == CaseDocumentType::ChainOfCustody)
            .collect();

        assert_eq!(
            coc_docs.len(),
            1,
            "Should find 1 COC PDF via content detection"
        );
        assert_eq!(coc_docs[0].filename, "document.pdf");
    }

    #[test]
    fn test_find_case_document_folders_real_path() {
        // Test with a real evidence path to debug folder discovery
        let evidence_path = r"J:\06988-0500 Wisconsin Cheese Group\25-049\1.Evidence\06988-0500-1-1NVRHF\UFED Samsung GSM SM-S918U Galaxy S23 Ultra 2025_05_28 (001)\AdvancedLogical 01\Samsung GSM_SM-S918U Galaxy S23 Ultra.zip";

        // This test will only work if the path exists
        if std::path::Path::new(evidence_path).exists() {
            let folders = find_case_document_folders(evidence_path);
            println!("Found {} case document folders:", folders.len());
            for folder in &folders {
                println!("  - {:?}", folder);
            }
            // Should find 4.Case.Documents and 6.Case.Notes
            assert!(
                !folders.is_empty(),
                "Should find at least one case document folder"
            );
        } else {
            println!("Skipping test - evidence path does not exist");
        }
    }
}
