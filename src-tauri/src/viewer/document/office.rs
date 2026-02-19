// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Office Document Viewer - Read-Only Text Extraction
//!
//! Extracts readable text and metadata from Microsoft Office and OpenDocument
//! format files for forensic viewing. **Never modifies source files.**
//!
//! # Supported Formats
//!
//! | Format | Extension(s) | Method |
//! |--------|-------------|--------|
//! | Word (OOXML) | .docx | ZIP + XML (word/document.xml) |
//! | Word (legacy) | .doc | CFB/OLE2 stream ("WordDocument" or raw text scan) |
//! | PowerPoint (OOXML) | .pptx | ZIP + XML (ppt/slides/*.xml) |
//! | PowerPoint (legacy) | .ppt | CFB/OLE2 (basic text extraction) |
//! | OpenDocument Text | .odt | ZIP + XML (content.xml) |
//! | OpenDocument Presentation | .odp | ZIP + XML (content.xml) |
//! | Rich Text Format | .rtf | Plain text stripping of RTF control words |
//!
//! # Architecture
//!
//! - OOXML formats (.docx, .pptx) are ZIP archives containing XML files.
//!   We use `zip` + `quick-xml` (both already in deps) to parse them.
//! - Legacy formats (.doc, .ppt) use Compound File Binary Format (OLE2).
//!   We use the `cfb` crate to open streams and extract embedded text.
//! - OpenDocument formats (.odt, .odp) are also ZIP-based with XML content.
//! - RTF is parsed by stripping control words to extract plain text.

use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::{DocumentError, DocumentResult};

// =============================================================================
// Types
// =============================================================================

/// Format of the office document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OfficeFormat {
    Docx,
    Doc,
    Pptx,
    Ppt,
    Odt,
    Odp,
    Rtf,
    Unknown,
}

impl OfficeFormat {
    /// Determine format from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "docx" => Self::Docx,
            "doc" => Self::Doc,
            "pptx" => Self::Pptx,
            "ppt" => Self::Ppt,
            "odt" => Self::Odt,
            "odp" => Self::Odp,
            "rtf" => Self::Rtf,
            _ => Self::Unknown,
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Docx => "Microsoft Word Document (OOXML)",
            Self::Doc => "Microsoft Word Document (Legacy)",
            Self::Pptx => "Microsoft PowerPoint Presentation (OOXML)",
            Self::Ppt => "Microsoft PowerPoint Presentation (Legacy)",
            Self::Odt => "OpenDocument Text",
            Self::Odp => "OpenDocument Presentation",
            Self::Rtf => "Rich Text Format",
            Self::Unknown => "Unknown Office Document",
        }
    }
}

/// Metadata extracted from an office document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeMetadata {
    /// Document title (from core.xml or meta.xml)
    pub title: Option<String>,
    /// Document author/creator
    pub creator: Option<String>,
    /// Last modified by
    pub last_modified_by: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document description
    pub description: Option<String>,
    /// Date created (ISO 8601)
    pub created: Option<String>,
    /// Date last modified (ISO 8601)
    pub modified: Option<String>,
    /// Application that created the document
    pub application: Option<String>,
    /// Number of pages/slides (if available)
    pub page_count: Option<u32>,
    /// Number of words (if available)
    pub word_count: Option<u32>,
    /// Number of characters (if available)
    pub char_count: Option<u32>,
}

impl Default for OfficeMetadata {
    fn default() -> Self {
        Self {
            title: None,
            creator: None,
            last_modified_by: None,
            subject: None,
            description: None,
            created: None,
            modified: None,
            application: None,
            page_count: None,
            word_count: None,
            char_count: None,
        }
    }
}

/// A section of extracted text (paragraph or slide)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeTextSection {
    /// Section label (e.g., "Slide 1", "Page 1", or empty for continuous text)
    pub label: Option<String>,
    /// Extracted paragraphs of text
    pub paragraphs: Vec<String>,
}

/// Complete office document information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeDocumentInfo {
    /// File path
    pub path: String,
    /// Detected format
    pub format: OfficeFormat,
    /// Format description
    pub format_description: String,
    /// Document metadata
    pub metadata: OfficeMetadata,
    /// Extracted text sections
    pub sections: Vec<OfficeTextSection>,
    /// Total character count of extracted text
    pub total_chars: usize,
    /// Total word count of extracted text
    pub total_words: usize,
    /// Whether extraction was complete or partial (e.g., legacy format limitations)
    pub extraction_complete: bool,
    /// Any warnings during extraction
    pub warnings: Vec<String>,
}

// =============================================================================
// Public API
// =============================================================================

/// Extract text and metadata from an office document.
///
/// This is the main entry point. Detects format by extension and
/// delegates to the appropriate extractor.
pub fn read_office_document(path: impl AsRef<Path>) -> DocumentResult<OfficeDocumentInfo> {
    let path = path.as_ref();
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let format = OfficeFormat::from_extension(&ext);
    if format == OfficeFormat::Unknown {
        return Err(DocumentError::Parse(format!(
            "Unsupported office format: .{}", ext
        )));
    }

    let mut warnings = Vec::new();
    let metadata;
    let sections;
    let mut extraction_complete = true;

    match format {
        OfficeFormat::Docx => {
            metadata = extract_ooxml_metadata(path, "docProps/core.xml", "docProps/app.xml")?;
            sections = extract_docx_text(path)?;
        }
        OfficeFormat::Pptx => {
            metadata = extract_ooxml_metadata(path, "docProps/core.xml", "docProps/app.xml")?;
            sections = extract_pptx_text(path)?;
        }
        OfficeFormat::Doc => {
            metadata = OfficeMetadata::default();
            warnings.push("Legacy .doc metadata extraction is limited".to_string());
            match extract_doc_text(path) {
                Ok(s) => sections = s,
                Err(e) => {
                    warnings.push(format!("Text extraction partial: {}", e));
                    sections = vec![OfficeTextSection {
                        label: None,
                        paragraphs: vec!["[Could not extract text from legacy .doc file]".to_string()],
                    }];
                    extraction_complete = false;
                }
            }
        }
        OfficeFormat::Ppt => {
            metadata = OfficeMetadata::default();
            warnings.push("Legacy .ppt text extraction is limited".to_string());
            match extract_ppt_text(path) {
                Ok(s) => sections = s,
                Err(e) => {
                    warnings.push(format!("Text extraction partial: {}", e));
                    sections = vec![OfficeTextSection {
                        label: None,
                        paragraphs: vec!["[Could not extract text from legacy .ppt file]".to_string()],
                    }];
                    extraction_complete = false;
                }
            }
        }
        OfficeFormat::Odt => {
            metadata = extract_odf_metadata(path)?;
            sections = extract_odt_text(path)?;
        }
        OfficeFormat::Odp => {
            metadata = extract_odf_metadata(path)?;
            sections = extract_odp_text(path)?;
        }
        OfficeFormat::Rtf => {
            metadata = OfficeMetadata::default();
            sections = extract_rtf_text(path)?;
        }
        OfficeFormat::Unknown => unreachable!(),
    }

    // Compute totals
    let all_text: String = sections.iter()
        .flat_map(|s| s.paragraphs.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let total_chars = all_text.len();
    let total_words = all_text.split_whitespace().count();

    Ok(OfficeDocumentInfo {
        path: path.to_string_lossy().to_string(),
        format,
        format_description: format.description().to_string(),
        metadata,
        sections,
        total_chars,
        total_words,
        extraction_complete,
        warnings,
    })
}

// =============================================================================
// OOXML Metadata (shared by DOCX and PPTX)
// =============================================================================

/// Extract metadata from OOXML (ZIP-based) documents.
///
/// Reads `docProps/core.xml` for Dublin Core metadata and
/// `docProps/app.xml` for application-specific properties.
fn extract_ooxml_metadata(
    path: &Path,
    core_path: &str,
    app_path: &str,
) -> DocumentResult<OfficeMetadata> {
    let file = std::fs::File::open(path)
        ?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid ZIP/OOXML file: {}", e)))?;

    let mut meta = OfficeMetadata::default();

    // Parse core.xml (Dublin Core metadata)
    if let Ok(mut entry) = archive.by_name(core_path) {
        let mut xml_data = String::new();
        let _ = entry.read_to_string(&mut xml_data);
        parse_core_xml(&xml_data, &mut meta);
    }

    // Parse app.xml (application properties)
    if let Ok(mut entry) = archive.by_name(app_path) {
        let mut xml_data = String::new();
        let _ = entry.read_to_string(&mut xml_data);
        parse_app_xml(&xml_data, &mut meta);
    }

    Ok(meta)
}

/// Parse Dublin Core metadata from core.xml
fn parse_core_xml(xml: &str, meta: &mut OfficeMetadata) {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    let mut current_tag = String::new();
    let mut in_element = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                current_tag = name;
                in_element = true;
            }
            Ok(Event::Text(ref e)) if in_element => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    continue;
                }
                match current_tag.as_str() {
                    "title" => meta.title = Some(text),
                    "creator" => meta.creator = Some(text),
                    "subject" => meta.subject = Some(text),
                    "description" => meta.description = Some(text),
                    "lastModifiedBy" => meta.last_modified_by = Some(text),
                    "created" => meta.created = Some(text),
                    "modified" => meta.modified = Some(text),
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                in_element = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
}

/// Parse application properties from app.xml
fn parse_app_xml(xml: &str, meta: &mut OfficeMetadata) {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    let mut current_tag = String::new();
    let mut in_element = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                current_tag = name;
                in_element = true;
            }
            Ok(Event::Text(ref e)) if in_element => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    continue;
                }
                match current_tag.as_str() {
                    "Application" => meta.application = Some(text),
                    "Pages" | "Slides" => {
                        if let Ok(n) = text.parse::<u32>() {
                            meta.page_count = Some(n);
                        }
                    }
                    "Words" => {
                        if let Ok(n) = text.parse::<u32>() {
                            meta.word_count = Some(n);
                        }
                    }
                    "Characters" | "CharactersWithSpaces" => {
                        if meta.char_count.is_none() {
                            if let Ok(n) = text.parse::<u32>() {
                                meta.char_count = Some(n);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                in_element = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
}

// =============================================================================
// DOCX Text Extraction
// =============================================================================

/// Extract text from a DOCX file (word/document.xml).
///
/// Reads `<w:p>` (paragraph) and `<w:t>` (text run) elements.
fn extract_docx_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)
        ?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid DOCX: {}", e)))?;

    let mut xml_data = String::new();
    {
        let mut entry = archive.by_name("word/document.xml")
            .map_err(|e| DocumentError::Parse(format!("Missing word/document.xml: {}", e)))?;
        entry.read_to_string(&mut xml_data)
            ?;
    }

    let paragraphs = extract_ooxml_paragraphs(&xml_data, b"w:p", b"w:t");

    Ok(vec![OfficeTextSection {
        label: None,
        paragraphs,
    }])
}

// =============================================================================
// PPTX Text Extraction
// =============================================================================

/// Extract text from a PPTX file (ppt/slides/slide*.xml).
///
/// Each slide is returned as a separate section.
fn extract_pptx_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)
        ?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid PPTX: {}", e)))?;

    // Collect slide file names and sort them
    let mut slide_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let entry = archive.by_index(i).ok()?;
            let name = entry.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    slide_names.sort();

    let mut sections = Vec::new();
    for (idx, slide_name) in slide_names.iter().enumerate() {
        let mut xml_data = String::new();
        if let Ok(mut entry) = archive.by_name(slide_name) {
            let _ = entry.read_to_string(&mut xml_data);
        }

        let paragraphs = extract_ooxml_paragraphs(&xml_data, b"a:p", b"a:t");
        if !paragraphs.is_empty() {
            sections.push(OfficeTextSection {
                label: Some(format!("Slide {}", idx + 1)),
                paragraphs,
            });
        }
    }

    Ok(sections)
}

// =============================================================================
// Shared OOXML Paragraph Extraction
// =============================================================================

/// Extract paragraphs from OOXML XML content.
///
/// Looks for paragraph elements (`para_tag`) containing text elements (`text_tag`).
/// Concatenates text runs within a paragraph, separates paragraphs by newlines.
///
/// Tags are matched against the *full qualified name* (e.g., `b"w:p"`) using `name()`.
fn extract_ooxml_paragraphs(xml: &str, para_tag: &[u8], text_tag: &[u8]) -> Vec<String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    let mut paragraphs = Vec::new();
    let mut current_paragraph = String::new();
    let mut in_text = false;
    let mut in_paragraph = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name.as_ref() == para_tag {
                    in_paragraph = true;
                    current_paragraph.clear();
                } else if name.as_ref() == text_tag && in_paragraph {
                    in_text = true;
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                if let Ok(text) = e.unescape() {
                    current_paragraph.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == text_tag {
                    in_text = false;
                } else if name.as_ref() == para_tag {
                    in_paragraph = false;
                    let trimmed = current_paragraph.trim().to_string();
                    if !trimmed.is_empty() {
                        paragraphs.push(trimmed);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    paragraphs
}

// =============================================================================
// Legacy DOC Text Extraction (OLE2 / CFB)
// =============================================================================

/// Extract text from a legacy .doc file using the CFB crate.
///
/// Legacy Word Binary Format stores text in the "WordDocument" stream
/// with complex encoding. We attempt a best-effort text extraction:
/// 1. Try to read the "WordDocument" stream
/// 2. Extract printable text runs (UTF-16LE or ASCII)
fn extract_doc_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    // Validate the file is a valid OLE2/CFB container
    let _comp = cfb::open(path)
        .map_err(|e| DocumentError::Parse(format!("Not a valid OLE2/DOC file: {}", e)))?;

    // Try common stream names for text content
    let stream_names = [
        "/WordDocument",
        "/1Table",
        "/0Table",
    ];

    let mut all_text = String::new();

    // Re-open for each stream read (cfb borrows mutably)
    for stream_name in &stream_names {
        if let Ok(mut comp) = cfb::open(path) {
            if comp.is_stream(stream_name) {
                if let Ok(mut stream) = comp.open_stream(stream_name) {
                    let mut data = Vec::new();
                    if stream.read_to_end(&mut data).is_ok() {
                        // Extract printable text runs from binary data
                        let text = extract_printable_text(&data);
                        if !text.is_empty() {
                            if !all_text.is_empty() {
                                all_text.push('\n');
                            }
                            all_text.push_str(&text);
                        }
                    }
                }
            }
        }
    }

    if all_text.is_empty() {
        // Fallback: scan the entire file for printable text runs
        let data = std::fs::read(path)
            ?;
        all_text = extract_printable_text(&data);
    }

    if all_text.is_empty() {
        return Err(DocumentError::Parse(
            "No readable text found in .doc file".to_string()
        ));
    }

    let paragraphs: Vec<String> = all_text
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(vec![OfficeTextSection {
        label: None,
        paragraphs,
    }])
}

/// Extract printable text runs from binary data.
///
/// Scans for sequences of printable ASCII characters (min 4 chars).
/// Also attempts UTF-16LE decoding for Unicode documents.
fn extract_printable_text(data: &[u8]) -> String {
    let mut result = String::new();

    // Try UTF-16LE first (common in Word docs)
    let utf16_text = extract_utf16le_text(data);
    if utf16_text.len() > result.len() {
        result = utf16_text;
    }

    // Also try ASCII extraction if UTF-16 didn't yield much
    if result.len() < 100 {
        let ascii_text = extract_ascii_text(data);
        if ascii_text.len() > result.len() {
            result = ascii_text;
        }
    }

    result
}

/// Extract text from UTF-16LE encoded data
fn extract_utf16le_text(data: &[u8]) -> String {
    if data.len() < 2 {
        return String::new();
    }

    let mut result = Vec::new();
    let mut current_run = String::new();

    for chunk in data.chunks_exact(2) {
        let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if let Some(ch) = char::from_u32(code_unit as u32) {
            if ch.is_alphanumeric() || ch.is_whitespace() || ch.is_ascii_punctuation() {
                current_run.push(ch);
            } else if !current_run.is_empty() {
                if current_run.trim().len() >= 4 {
                    result.push(current_run.trim().to_string());
                }
                current_run.clear();
            }
        }
    }
    if current_run.trim().len() >= 4 {
        result.push(current_run.trim().to_string());
    }

    result.join("\n")
}

/// Extract ASCII text runs from binary data
fn extract_ascii_text(data: &[u8]) -> String {
    let mut result = Vec::new();
    let mut current_run = String::new();

    for &byte in data {
        if byte >= 0x20 && byte < 0x7F || byte == b'\n' || byte == b'\r' || byte == b'\t' {
            current_run.push(byte as char);
        } else if !current_run.is_empty() {
            if current_run.trim().len() >= 4 {
                result.push(current_run.trim().to_string());
            }
            current_run.clear();
        }
    }
    if current_run.trim().len() >= 4 {
        result.push(current_run.trim().to_string());
    }

    result.join("\n")
}

// =============================================================================
// Legacy PPT Text Extraction (OLE2 / CFB)
// =============================================================================

/// Extract text from a legacy .ppt file.
///
/// PPT files store slide text in the "PowerPoint Document" stream.
/// We extract printable text runs as a best-effort approach.
fn extract_ppt_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let mut comp = cfb::open(path)
        .map_err(|e| DocumentError::Parse(format!("Not a valid OLE2/PPT file: {}", e)))?;

    let mut all_text = String::new();

    // The main content stream in PPT files
    if comp.is_stream("/PowerPoint Document") {
        if let Ok(mut stream) = comp.open_stream("/PowerPoint Document") {
            let mut data = Vec::new();
            if stream.read_to_end(&mut data).is_ok() {
                all_text = extract_printable_text(&data);
            }
        }
    }

    if all_text.is_empty() {
        // Fallback: scan entire file
        let data = std::fs::read(path)
            ?;
        all_text = extract_printable_text(&data);
    }

    if all_text.is_empty() {
        return Err(DocumentError::Parse(
            "No readable text found in .ppt file".to_string()
        ));
    }

    let paragraphs: Vec<String> = all_text
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(vec![OfficeTextSection {
        label: Some("Presentation".to_string()),
        paragraphs,
    }])
}

// =============================================================================
// OpenDocument Metadata
// =============================================================================

/// Extract metadata from OpenDocument format files (ODT, ODP).
///
/// Reads `meta.xml` from the ZIP archive.
fn extract_odf_metadata(path: &Path) -> DocumentResult<OfficeMetadata> {
    let file = std::fs::File::open(path)
        ?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid ODF file: {}", e)))?;

    let mut meta = OfficeMetadata::default();

    if let Ok(mut entry) = archive.by_name("meta.xml") {
        let mut xml_data = String::new();
        let _ = entry.read_to_string(&mut xml_data);
        parse_odf_meta_xml(&xml_data, &mut meta);
    }

    Ok(meta)
}

/// Parse ODF meta.xml for document metadata
fn parse_odf_meta_xml(xml: &str, meta: &mut OfficeMetadata) {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    let mut current_tag = String::new();
    let mut in_element = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                current_tag = name;
                in_element = true;

                // Check for page-count in attributes of document-statistic
                if current_tag == "document-statistic" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "page-count" => meta.page_count = val.parse().ok(),
                            "word-count" => meta.word_count = val.parse().ok(),
                            "character-count" => meta.char_count = val.parse().ok(),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) if in_element => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    continue;
                }
                match current_tag.as_str() {
                    "title" => meta.title = Some(text),
                    "initial-creator" | "creator" => {
                        if meta.creator.is_none() {
                            meta.creator = Some(text);
                        }
                    }
                    "subject" => meta.subject = Some(text),
                    "description" => meta.description = Some(text),
                    "creation-date" | "date" => {
                        if meta.created.is_none() {
                            meta.created = Some(text);
                        } else if meta.modified.is_none() {
                            meta.modified = Some(text);
                        }
                    }
                    "generator" => meta.application = Some(text),
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                // Handle self-closing elements like <meta:document-statistic ... />
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if name == "document-statistic" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "page-count" => meta.page_count = val.parse().ok(),
                            "word-count" => meta.word_count = val.parse().ok(),
                            "character-count" => meta.char_count = val.parse().ok(),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(_)) => {
                in_element = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
}

// =============================================================================
// ODT Text Extraction
// =============================================================================

/// Extract text from an ODT file (content.xml).
///
/// OpenDocument uses `<text:p>` for paragraphs and stores text directly.
fn extract_odt_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)
        ?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid ODT: {}", e)))?;

    let mut xml_data = String::new();
    {
        let mut entry = archive.by_name("content.xml")
            .map_err(|e| DocumentError::Parse(format!("Missing content.xml: {}", e)))?;
        entry.read_to_string(&mut xml_data)
            ?;
    }

    let paragraphs = extract_odf_paragraphs(&xml_data);

    Ok(vec![OfficeTextSection {
        label: None,
        paragraphs,
    }])
}

// =============================================================================
// ODP Text Extraction
// =============================================================================

/// Extract text from an ODP file (content.xml).
///
/// Each `<draw:page>` is a slide; text is in `<text:p>` elements.
fn extract_odp_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)
        ?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid ODP: {}", e)))?;

    let mut xml_data = String::new();
    {
        let mut entry = archive.by_name("content.xml")
            .map_err(|e| DocumentError::Parse(format!("Missing content.xml: {}", e)))?;
        entry.read_to_string(&mut xml_data)
            ?;
    }

    // For ODP, extract all paragraphs as one section (slide separation
    // would require tracking <draw:page> boundaries, which adds complexity)
    let paragraphs = extract_odf_paragraphs(&xml_data);

    Ok(vec![OfficeTextSection {
        label: Some("Presentation".to_string()),
        paragraphs,
    }])
}

/// Extract paragraphs from ODF content.xml.
///
/// Looks for text content within `<text:p>` and `<text:span>` elements.
fn extract_odf_paragraphs(xml: &str) -> Vec<String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    let mut paragraphs = Vec::new();
    let mut current_paragraph = String::new();
    let mut depth = 0u32; // Track depth inside text:p

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if local == "p" || local == "h" {
                    if depth > 0 {
                        // Nested paragraph — flush current
                        let trimmed = current_paragraph.trim().to_string();
                        if !trimmed.is_empty() {
                            paragraphs.push(trimmed);
                        }
                        current_paragraph.clear();
                    }
                    depth += 1;
                } else if (local == "span" || local == "a") && depth > 0 {
                    // Text span or link inside paragraph — continue accumulating
                } else if local == "tab" && depth > 0 {
                    current_paragraph.push('\t');
                } else if local == "s" && depth > 0 {
                    current_paragraph.push(' ');
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if local == "tab" && depth > 0 {
                    current_paragraph.push('\t');
                } else if local == "s" && depth > 0 {
                    current_paragraph.push(' ');
                } else if local == "line-break" && depth > 0 {
                    current_paragraph.push('\n');
                }
            }
            Ok(Event::Text(ref e)) if depth > 0 => {
                if let Ok(text) = e.unescape() {
                    current_paragraph.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if local == "p" || local == "h" {
                    if depth > 0 {
                        depth -= 1;
                    }
                    if depth == 0 {
                        let trimmed = current_paragraph.trim().to_string();
                        if !trimmed.is_empty() {
                            paragraphs.push(trimmed);
                        }
                        current_paragraph.clear();
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    paragraphs
}

// =============================================================================
// RTF Text Extraction
// =============================================================================

/// Extract text from an RTF file by stripping control words.
///
/// RTF is a text-based format with backslash control words.
/// We strip these to extract the readable text content.
fn extract_rtf_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let data = std::fs::read_to_string(path)
        ?;

    // Verify it's actually RTF
    if !data.starts_with("{\\rtf") {
        return Err(DocumentError::Parse("Not a valid RTF file".to_string()));
    }

    let text = strip_rtf_to_text(&data);

    let paragraphs: Vec<String> = text
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(vec![OfficeTextSection {
        label: None,
        paragraphs,
    }])
}

/// Strip RTF control words to extract plain text.
///
/// Handles:
/// - `\par` → newline
/// - `\tab` → tab
/// - `\'XX` → hex-encoded character
/// - `{` and `}` → group delimiters (ignored)
/// - Other `\word` → ignored
fn strip_rtf_to_text(rtf: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = rtf.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut depth = 0i32;
    let mut skip_group = false;

    while i < len {
        let ch = chars[i];

        match ch {
            '{' => {
                depth += 1;
                // Skip certain destination groups
                if i + 1 < len && chars[i + 1] == '\\' {
                    // Look ahead for destination keywords to skip
                    let rest: String = chars[i+1..std::cmp::min(i+30, len)].iter().collect();
                    if rest.starts_with("\\fonttbl") || rest.starts_with("\\colortbl")
                        || rest.starts_with("\\stylesheet") || rest.starts_with("\\info")
                        || rest.starts_with("\\*\\")
                    {
                        skip_group = true;
                    }
                }
                i += 1;
            }
            '}' => {
                depth -= 1;
                if depth <= 0 {
                    skip_group = false;
                }
                if skip_group && depth <= 1 {
                    skip_group = false;
                }
                i += 1;
            }
            '\\' if !skip_group => {
                i += 1;
                if i >= len {
                    break;
                }

                let next = chars[i];

                // Escaped special characters
                if next == '{' || next == '}' || next == '\\' {
                    result.push(next);
                    i += 1;
                } else if next == '\'' {
                    // Hex-encoded character: \'XX
                    i += 1;
                    if i + 1 < len {
                        let hex: String = chars[i..i+2].iter().collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            if byte >= 0x20 {
                                result.push(byte as char);
                            }
                        }
                        i += 2;
                    }
                } else if next == '\n' || next == '\r' {
                    // Line break in RTF source — ignore
                    i += 1;
                } else {
                    // Control word: \wordN or \word-N
                    let mut word = String::new();
                    while i < len && chars[i].is_ascii_alphabetic() {
                        word.push(chars[i]);
                        i += 1;
                    }
                    // Skip optional numeric parameter
                    if i < len && (chars[i] == '-' || chars[i].is_ascii_digit()) {
                        while i < len && (chars[i] == '-' || chars[i].is_ascii_digit()) {
                            i += 1;
                        }
                    }
                    // Skip trailing space delimiter
                    if i < len && chars[i] == ' ' {
                        i += 1;
                    }

                    // Handle meaningful control words
                    match word.as_str() {
                        "par" | "line" => result.push('\n'),
                        "tab" => result.push('\t'),
                        "emspace" | "enspace" | "qmspace" => result.push(' '),
                        _ => {} // Ignore other control words
                    }
                }
            }
            _ if !skip_group => {
                // Regular character
                if ch != '\r' && ch != '\n' {
                    result.push(ch);
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // OfficeFormat tests
    // =========================================================================

    #[test]
    fn test_office_format_from_extension() {
        assert_eq!(OfficeFormat::from_extension("docx"), OfficeFormat::Docx);
        assert_eq!(OfficeFormat::from_extension("DOCX"), OfficeFormat::Docx);
        assert_eq!(OfficeFormat::from_extension("doc"), OfficeFormat::Doc);
        assert_eq!(OfficeFormat::from_extension("pptx"), OfficeFormat::Pptx);
        assert_eq!(OfficeFormat::from_extension("ppt"), OfficeFormat::Ppt);
        assert_eq!(OfficeFormat::from_extension("odt"), OfficeFormat::Odt);
        assert_eq!(OfficeFormat::from_extension("odp"), OfficeFormat::Odp);
        assert_eq!(OfficeFormat::from_extension("rtf"), OfficeFormat::Rtf);
        assert_eq!(OfficeFormat::from_extension("xyz"), OfficeFormat::Unknown);
    }

    #[test]
    fn test_office_format_description() {
        assert!(OfficeFormat::Docx.description().contains("Word"));
        assert!(OfficeFormat::Pptx.description().contains("PowerPoint"));
        assert!(OfficeFormat::Odt.description().contains("OpenDocument"));
        assert!(OfficeFormat::Rtf.description().contains("Rich Text"));
    }

    // =========================================================================
    // OOXML metadata parsing tests
    // =========================================================================

    #[test]
    fn test_parse_core_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                           xmlns:dc="http://purl.org/dc/elements/1.1/"
                           xmlns:dcterms="http://purl.org/dc/terms/">
            <dc:title>Test Document</dc:title>
            <dc:creator>John Doe</dc:creator>
            <dc:subject>Testing</dc:subject>
            <dc:description>A test document</dc:description>
            <cp:lastModifiedBy>Jane Smith</cp:lastModifiedBy>
            <dcterms:created>2024-01-15T10:30:00Z</dcterms:created>
            <dcterms:modified>2024-06-20T14:45:00Z</dcterms:modified>
        </cp:coreProperties>"#;

        let mut meta = OfficeMetadata::default();
        parse_core_xml(xml, &mut meta);

        assert_eq!(meta.title.as_deref(), Some("Test Document"));
        assert_eq!(meta.creator.as_deref(), Some("John Doe"));
        assert_eq!(meta.subject.as_deref(), Some("Testing"));
        assert_eq!(meta.description.as_deref(), Some("A test document"));
        assert_eq!(meta.last_modified_by.as_deref(), Some("Jane Smith"));
        assert_eq!(meta.created.as_deref(), Some("2024-01-15T10:30:00Z"));
        assert_eq!(meta.modified.as_deref(), Some("2024-06-20T14:45:00Z"));
    }

    #[test]
    fn test_parse_app_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
            <Application>Microsoft Office Word</Application>
            <Pages>5</Pages>
            <Words>1234</Words>
            <Characters>6789</Characters>
        </Properties>"#;

        let mut meta = OfficeMetadata::default();
        parse_app_xml(xml, &mut meta);

        assert_eq!(meta.application.as_deref(), Some("Microsoft Office Word"));
        assert_eq!(meta.page_count, Some(5));
        assert_eq!(meta.word_count, Some(1234));
        assert_eq!(meta.char_count, Some(6789));
    }

    #[test]
    fn test_parse_core_xml_empty() {
        let xml = r#"<?xml version="1.0"?><cp:coreProperties></cp:coreProperties>"#;
        let mut meta = OfficeMetadata::default();
        parse_core_xml(xml, &mut meta);
        assert!(meta.title.is_none());
        assert!(meta.creator.is_none());
    }

    // =========================================================================
    // OOXML paragraph extraction tests
    // =========================================================================

    #[test]
    fn test_extract_docx_paragraphs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:body>
                <w:p>
                    <w:r><w:t>Hello </w:t></w:r>
                    <w:r><w:t>World</w:t></w:r>
                </w:p>
                <w:p>
                    <w:r><w:t>Second paragraph</w:t></w:r>
                </w:p>
                <w:p></w:p>
            </w:body>
        </w:document>"#;

        let paragraphs = extract_ooxml_paragraphs(xml, b"w:p", b"w:t");
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0], "Hello World");
        assert_eq!(paragraphs[1], "Second paragraph");
    }

    #[test]
    fn test_extract_pptx_paragraphs() {
        let xml = r#"<?xml version="1.0"?>
        <p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
               xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
            <p:cSld>
                <p:spTree>
                    <p:sp>
                        <p:txBody>
                            <a:p><a:r><a:t>Slide Title</a:t></a:r></a:p>
                            <a:p><a:r><a:t>Bullet point one</a:t></a:r></a:p>
                        </p:txBody>
                    </p:sp>
                </p:spTree>
            </p:cSld>
        </p:sld>"#;

        let paragraphs = extract_ooxml_paragraphs(xml, b"a:p", b"a:t");
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0], "Slide Title");
        assert_eq!(paragraphs[1], "Bullet point one");
    }

    #[test]
    fn test_extract_ooxml_paragraphs_empty() {
        let xml = r#"<w:document><w:body></w:body></w:document>"#;
        let paragraphs = extract_ooxml_paragraphs(xml, b"w:p", b"w:t");
        assert!(paragraphs.is_empty());
    }

    // =========================================================================
    // ODF metadata parsing tests
    // =========================================================================

    #[test]
    fn test_parse_odf_meta_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
                              xmlns:dc="http://purl.org/dc/elements/1.1/"
                              xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0">
            <office:meta>
                <dc:title>ODT Test</dc:title>
                <dc:creator>LibreOffice User</dc:creator>
                <meta:creation-date>2024-03-01T09:00:00</meta:creation-date>
                <meta:generator>LibreOffice/7.6</meta:generator>
                <meta:document-statistic meta:page-count="3" meta:word-count="500" meta:character-count="2500"/>
            </office:meta>
        </office:document-meta>"#;

        let mut meta = OfficeMetadata::default();
        parse_odf_meta_xml(xml, &mut meta);

        assert_eq!(meta.title.as_deref(), Some("ODT Test"));
        assert_eq!(meta.creator.as_deref(), Some("LibreOffice User"));
        assert_eq!(meta.created.as_deref(), Some("2024-03-01T09:00:00"));
        assert_eq!(meta.application.as_deref(), Some("LibreOffice/7.6"));
        assert_eq!(meta.page_count, Some(3));
        assert_eq!(meta.word_count, Some(500));
        assert_eq!(meta.char_count, Some(2500));
    }

    // =========================================================================
    // ODF paragraph extraction tests
    // =========================================================================

    #[test]
    fn test_extract_odf_paragraphs() {
        let xml = r#"<?xml version="1.0"?>
        <office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
                                  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
            <office:body>
                <office:text>
                    <text:p>First paragraph of the document.</text:p>
                    <text:p>Second paragraph with <text:span>styled text</text:span>.</text:p>
                    <text:h>A Heading</text:h>
                </office:text>
            </office:body>
        </office:document-content>"#;

        let paragraphs = extract_odf_paragraphs(xml);
        assert_eq!(paragraphs.len(), 3);
        assert_eq!(paragraphs[0], "First paragraph of the document.");
        assert_eq!(paragraphs[1], "Second paragraph with styled text.");
        assert_eq!(paragraphs[2], "A Heading");
    }

    // =========================================================================
    // RTF text extraction tests
    // =========================================================================

    #[test]
    fn test_strip_rtf_basic() {
        let rtf = r"{\rtf1\ansi Hello World}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains("Hello World"), "Got: {}", text);
    }

    #[test]
    fn test_strip_rtf_paragraphs() {
        let rtf = r"{\rtf1\ansi First paragraph\par Second paragraph\par Third}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains("First paragraph"), "Got: {}", text);
        assert!(text.contains("Second paragraph"), "Got: {}", text);
        assert!(text.contains("Third"), "Got: {}", text);
    }

    #[test]
    fn test_strip_rtf_special_chars() {
        let rtf = r"{\rtf1 Braces: \{ and \} and backslash: \\}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains('{'), "Got: {}", text);
        assert!(text.contains('}'), "Got: {}", text);
        assert!(text.contains('\\'), "Got: {}", text);
    }

    #[test]
    fn test_strip_rtf_skip_groups() {
        let rtf = r"{\rtf1{\fonttbl{\f0 Arial;}}{\colortbl;\red0\green0\blue0;}Hello from RTF}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains("Hello from RTF"), "Got: {}", text);
        // Should NOT contain font table content
        assert!(!text.contains("Arial"), "Font table leaked: {}", text);
    }

    #[test]
    fn test_strip_rtf_hex_chars() {
        // \'e9 = é in Windows-1252
        let rtf = r"{\rtf1 caf\'e9}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains("caf"), "Got: {}", text);
    }

    // =========================================================================
    // Text extraction helpers tests
    // =========================================================================

    #[test]
    fn test_extract_ascii_text() {
        let data = b"Hello World\x00\x01\x02This is text\x00\x00end";
        let text = extract_ascii_text(data);
        assert!(text.contains("Hello World"), "Got: {}", text);
        assert!(text.contains("This is text"), "Got: {}", text);
    }

    #[test]
    fn test_extract_ascii_short_runs_filtered() {
        // Runs shorter than 4 chars should be filtered out
        let data = b"Hi\x00\x00Long enough text here";
        let text = extract_ascii_text(data);
        assert!(!text.contains("Hi"));
        assert!(text.contains("Long enough text here"));
    }

    #[test]
    fn test_extract_utf16le_text() {
        // "Hello" in UTF-16LE
        let data: Vec<u8> = "Hello World"
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let text = extract_utf16le_text(&data);
        assert!(text.contains("Hello World"), "Got: {}", text);
    }

    #[test]
    fn test_office_document_info_serialization() {
        let info = OfficeDocumentInfo {
            path: "/test/doc.docx".to_string(),
            format: OfficeFormat::Docx,
            format_description: "Microsoft Word Document (OOXML)".to_string(),
            metadata: OfficeMetadata::default(),
            sections: vec![OfficeTextSection {
                label: None,
                paragraphs: vec!["Test content".to_string()],
            }],
            total_chars: 12,
            total_words: 2,
            extraction_complete: true,
            warnings: vec![],
        };

        let json = serde_json::to_string(&info).expect("Should serialize");
        assert!(json.contains("formatDescription"));
        assert!(json.contains("extractionComplete"));
        assert!(json.contains("totalWords"));
    }

    #[test]
    fn test_unsupported_format() {
        let result = read_office_document("/fake/file.xyz");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unsupported office format"), "Got: {}", err);
    }
}
