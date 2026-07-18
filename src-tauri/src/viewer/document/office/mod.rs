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

mod cfb;
mod odf;
mod ooxml;
mod rtf;

use std::io::Cursor;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::{DocumentError, DocumentResult};

const MAX_OFFICE_SECTIONS: usize = 512;
const MAX_OFFICE_PARAGRAPHS_PER_SECTION: usize = 5_000;
const MAX_OFFICE_PARAGRAPH_CHARS: usize = 16_384;
const MAX_OFFICE_METADATA_CHARS: usize = 4_096;
const MAX_OFFICE_WARNINGS: usize = 128;
const MAX_OFFICE_WARNING_CHARS: usize = 2_048;

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
#[derive(Default)]
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

/// Paragraph-level style hint for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum ParagraphHint {
    #[default]
    Normal,
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Title,
    Subtitle,
    ListItem,
    Quote,
}

/// A styled paragraph of extracted text
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeParagraph {
    /// The text content
    pub text: String,
    /// Formatting hint for the frontend renderer
    pub hint: ParagraphHint,
}

impl OfficeParagraph {
    /// Create a normal (unstyled) paragraph
    pub fn normal(text: String) -> Self {
        Self {
            text,
            hint: ParagraphHint::Normal,
        }
    }
}

/// A section of extracted text (paragraph or slide)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeTextSection {
    /// Section label (e.g., "Slide 1", "Page 1", or empty for continuous text)
    pub label: Option<String>,
    /// Extracted paragraphs of text with style hints
    pub paragraphs: Vec<OfficeParagraph>,
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
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let format = OfficeFormat::from_extension(&ext);
    if format == OfficeFormat::Unknown {
        return Err(unsupported_office_format_error(&ext));
    }

    let mut warnings = Vec::new();
    let metadata;
    let sections;
    let mut extraction_complete = true;

    match format {
        OfficeFormat::Docx => {
            metadata =
                ooxml::extract_ooxml_metadata(path, "docProps/core.xml", "docProps/app.xml")?;
            sections = ooxml::extract_docx_text(path)?;
        }
        OfficeFormat::Pptx => {
            metadata =
                ooxml::extract_ooxml_metadata(path, "docProps/core.xml", "docProps/app.xml")?;
            sections = ooxml::extract_pptx_text(path)?;
        }
        OfficeFormat::Doc => {
            metadata = OfficeMetadata::default();
            warnings.push("Legacy .doc metadata extraction is limited".to_string());
            match cfb::extract_doc_text(path) {
                Ok(s) => sections = s,
                Err(e) => {
                    warnings.push(format!("Text extraction partial: {}", e));
                    sections = vec![OfficeTextSection {
                        label: None,
                        paragraphs: vec![OfficeParagraph::normal(
                            "[Could not extract text from legacy .doc file]".to_string(),
                        )],
                    }];
                    extraction_complete = false;
                }
            }
        }
        OfficeFormat::Ppt => {
            metadata = OfficeMetadata::default();
            warnings.push("Legacy .ppt text extraction is limited".to_string());
            match cfb::extract_ppt_text(path) {
                Ok(s) => sections = s,
                Err(e) => {
                    warnings.push(format!("Text extraction partial: {}", e));
                    sections = vec![OfficeTextSection {
                        label: None,
                        paragraphs: vec![OfficeParagraph::normal(
                            "[Could not extract text from legacy .ppt file]".to_string(),
                        )],
                    }];
                    extraction_complete = false;
                }
            }
        }
        OfficeFormat::Odt => {
            metadata = odf::extract_odf_metadata(path)?;
            sections = odf::extract_odt_text(path)?;
        }
        OfficeFormat::Odp => {
            metadata = odf::extract_odf_metadata(path)?;
            sections = odf::extract_odp_text(path)?;
        }
        OfficeFormat::Rtf => {
            metadata = OfficeMetadata::default();
            sections = rtf::extract_rtf_text(path)?;
        }
        OfficeFormat::Unknown => return Err(unsupported_office_format_error(&ext)),
    }

    Ok(build_office_document_info(
        path.to_string_lossy().to_string(),
        format,
        metadata,
        sections,
        extraction_complete,
        warnings,
    ))
}

/// Extract text and metadata from office document bytes.
///
/// `source_id` is used for extension-based format selection and provenance in
/// the returned metadata. It may be a local path or a container display ID.
pub fn read_office_document_bytes(
    source_id: impl Into<String>,
    data: &[u8],
) -> DocumentResult<OfficeDocumentInfo> {
    let source_id = source_id.into();
    let ext = Path::new(&source_id)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let format = OfficeFormat::from_extension(&ext);
    if format == OfficeFormat::Unknown {
        return Err(unsupported_office_format_error(&ext));
    }

    let mut warnings = Vec::new();
    let metadata;
    let sections;
    let mut extraction_complete = true;

    match format {
        OfficeFormat::Docx => {
            metadata = ooxml::extract_ooxml_metadata_from_reader(
                Cursor::new(data),
                "docProps/core.xml",
                "docProps/app.xml",
            )?;
            sections = ooxml::extract_docx_text_from_reader(Cursor::new(data))?;
        }
        OfficeFormat::Pptx => {
            metadata = ooxml::extract_ooxml_metadata_from_reader(
                Cursor::new(data),
                "docProps/core.xml",
                "docProps/app.xml",
            )?;
            sections = ooxml::extract_pptx_text_from_reader(Cursor::new(data))?;
        }
        OfficeFormat::Doc => {
            metadata = OfficeMetadata::default();
            warnings.push("Legacy .doc metadata extraction is limited".to_string());
            match cfb::extract_doc_text_from_bytes(data) {
                Ok(s) => sections = s,
                Err(e) => {
                    warnings.push(format!("Text extraction partial: {}", e));
                    sections = vec![OfficeTextSection {
                        label: None,
                        paragraphs: vec![OfficeParagraph::normal(
                            "[Could not extract text from legacy .doc file]".to_string(),
                        )],
                    }];
                    extraction_complete = false;
                }
            }
        }
        OfficeFormat::Ppt => {
            metadata = OfficeMetadata::default();
            warnings.push("Legacy .ppt text extraction is limited".to_string());
            match cfb::extract_ppt_text_from_bytes(data) {
                Ok(s) => sections = s,
                Err(e) => {
                    warnings.push(format!("Text extraction partial: {}", e));
                    sections = vec![OfficeTextSection {
                        label: None,
                        paragraphs: vec![OfficeParagraph::normal(
                            "[Could not extract text from legacy .ppt file]".to_string(),
                        )],
                    }];
                    extraction_complete = false;
                }
            }
        }
        OfficeFormat::Odt => {
            metadata = odf::extract_odf_metadata_from_reader(Cursor::new(data))?;
            sections = odf::extract_odt_text_from_reader(Cursor::new(data))?;
        }
        OfficeFormat::Odp => {
            metadata = odf::extract_odf_metadata_from_reader(Cursor::new(data))?;
            sections = odf::extract_odp_text_from_reader(Cursor::new(data))?;
        }
        OfficeFormat::Rtf => {
            metadata = OfficeMetadata::default();
            sections = rtf::extract_rtf_text_from_bytes(data)?;
        }
        OfficeFormat::Unknown => return Err(unsupported_office_format_error(&ext)),
    }

    Ok(build_office_document_info(
        source_id,
        format,
        metadata,
        sections,
        extraction_complete,
        warnings,
    ))
}

fn unsupported_office_format_error(ext: &str) -> DocumentError {
    DocumentError::Parse(format!("Unsupported office format: .{}", ext))
}

fn build_office_document_info(
    path: String,
    format: OfficeFormat,
    mut metadata: OfficeMetadata,
    sections: Vec<OfficeTextSection>,
    extraction_complete: bool,
    warnings: Vec<String>,
) -> OfficeDocumentInfo {
    normalize_office_metadata(&mut metadata);
    let sections = normalize_office_sections(sections);
    let warnings = normalize_office_warnings(warnings);

    let all_text: String = sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let total_chars = all_text.len();
    let total_words = all_text.split_whitespace().count();

    OfficeDocumentInfo {
        path,
        format,
        format_description: format.description().to_string(),
        metadata,
        sections,
        total_chars,
        total_words,
        extraction_complete,
        warnings,
    }
}

fn normalize_office_metadata(metadata: &mut OfficeMetadata) {
    truncate_option_string(&mut metadata.title, MAX_OFFICE_METADATA_CHARS);
    truncate_option_string(&mut metadata.creator, MAX_OFFICE_METADATA_CHARS);
    truncate_option_string(&mut metadata.last_modified_by, MAX_OFFICE_METADATA_CHARS);
    truncate_option_string(&mut metadata.subject, MAX_OFFICE_METADATA_CHARS);
    truncate_option_string(&mut metadata.description, MAX_OFFICE_METADATA_CHARS);
    truncate_option_string(&mut metadata.created, MAX_OFFICE_METADATA_CHARS);
    truncate_option_string(&mut metadata.modified, MAX_OFFICE_METADATA_CHARS);
    truncate_option_string(&mut metadata.application, MAX_OFFICE_METADATA_CHARS);
}

fn normalize_office_sections(sections: Vec<OfficeTextSection>) -> Vec<OfficeTextSection> {
    sections
        .into_iter()
        .take(MAX_OFFICE_SECTIONS)
        .map(|mut section| {
            truncate_option_string(&mut section.label, MAX_OFFICE_METADATA_CHARS);
            section.paragraphs = section
                .paragraphs
                .into_iter()
                .take(MAX_OFFICE_PARAGRAPHS_PER_SECTION)
                .map(|mut paragraph| {
                    paragraph.text = truncate_chars(&paragraph.text, MAX_OFFICE_PARAGRAPH_CHARS);
                    paragraph
                })
                .collect();
            section
        })
        .collect()
}

fn normalize_office_warnings(warnings: Vec<String>) -> Vec<String> {
    warnings
        .into_iter()
        .take(MAX_OFFICE_WARNINGS)
        .map(|warning| truncate_chars(&warning, MAX_OFFICE_WARNING_CHARS))
        .collect()
}

fn truncate_option_string(value: &mut Option<String>, max_chars: usize) {
    if let Some(text) = value {
        *text = truncate_chars(text, max_chars);
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("...");
    truncated
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn build_docx_bytes() -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();

        zip.start_file("docProps/core.xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?><cp:coreProperties><dc:title>Evidence Memo</dc:title><dc:creator>Analyst</dc:creator></cp:coreProperties>"#,
        )
        .unwrap();

        zip.start_file("docProps/app.xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?><Properties><Application>CORE Test</Application><Words>2</Words></Properties>"#,
        )
        .unwrap();

        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello source</w:t></w:r></w:p></w:body></w:document>"#,
        )
        .unwrap();

        zip.finish().unwrap().into_inner()
    }

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

    #[test]
    fn test_office_document_info_serialization() {
        let info = OfficeDocumentInfo {
            path: "/test/doc.docx".to_string(),
            format: OfficeFormat::Docx,
            format_description: "Microsoft Word Document (OOXML)".to_string(),
            metadata: OfficeMetadata::default(),
            sections: vec![OfficeTextSection {
                label: None,
                paragraphs: vec![OfficeParagraph::normal("Test content".to_string())],
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

    #[test]
    fn truncate_chars_is_unicode_safe() {
        let value = "é".repeat(MAX_OFFICE_METADATA_CHARS + 1);

        let truncated = truncate_chars(&value, MAX_OFFICE_METADATA_CHARS);

        assert!(truncated.ends_with("..."));
        assert_eq!(
            truncated.trim_end_matches("...").chars().count(),
            MAX_OFFICE_METADATA_CHARS
        );
    }

    #[test]
    fn build_office_document_info_caps_response_sections_and_paragraphs() {
        let sections = (0..(MAX_OFFICE_SECTIONS + 8))
            .map(|section| OfficeTextSection {
                label: Some(format!("Section {section}")),
                paragraphs: (0..(MAX_OFFICE_PARAGRAPHS_PER_SECTION + 8))
                    .map(|paragraph| OfficeParagraph {
                        text: format!("paragraph {section}-{paragraph}"),
                        hint: ParagraphHint::Normal,
                    })
                    .collect(),
            })
            .collect();

        let info = build_office_document_info(
            "source.docx".to_string(),
            OfficeFormat::Docx,
            OfficeMetadata::default(),
            sections,
            true,
            Vec::new(),
        );

        assert_eq!(info.sections.len(), MAX_OFFICE_SECTIONS);
        assert_eq!(
            info.sections[0].paragraphs.len(),
            MAX_OFFICE_PARAGRAPHS_PER_SECTION
        );
    }

    #[test]
    fn build_office_document_info_truncates_metadata_warnings_and_paragraphs() {
        let metadata_value = "é".repeat(MAX_OFFICE_METADATA_CHARS + 1);
        let paragraph_value = "é".repeat(MAX_OFFICE_PARAGRAPH_CHARS + 1);
        let warning_value = "é".repeat(MAX_OFFICE_WARNING_CHARS + 1);

        let info = build_office_document_info(
            "source.docx".to_string(),
            OfficeFormat::Docx,
            OfficeMetadata {
                title: Some(metadata_value),
                ..Default::default()
            },
            vec![OfficeTextSection {
                label: Some("section".to_string()),
                paragraphs: vec![OfficeParagraph {
                    text: paragraph_value,
                    hint: ParagraphHint::Normal,
                }],
            }],
            true,
            vec![warning_value; MAX_OFFICE_WARNINGS + 8],
        );

        let title = info.metadata.title.as_deref().unwrap();
        assert!(title.ends_with("..."));
        assert_eq!(
            title.trim_end_matches("...").chars().count(),
            MAX_OFFICE_METADATA_CHARS
        );

        let paragraph = &info.sections[0].paragraphs[0].text;
        assert!(paragraph.ends_with("..."));
        assert_eq!(
            paragraph.trim_end_matches("...").chars().count(),
            MAX_OFFICE_PARAGRAPH_CHARS
        );

        assert_eq!(info.warnings.len(), MAX_OFFICE_WARNINGS);
        assert!(info.warnings[0].ends_with("..."));
        assert_eq!(
            info.warnings[0].trim_end_matches("...").chars().count(),
            MAX_OFFICE_WARNING_CHARS
        );
    }

    #[test]
    fn read_office_document_bytes_reads_docx_source() {
        let info = read_office_document_bytes("container.ad1:docs/memo.docx", &build_docx_bytes())
            .unwrap();

        assert_eq!(info.path, "container.ad1:docs/memo.docx");
        assert_eq!(info.format, OfficeFormat::Docx);
        assert_eq!(info.metadata.title.as_deref(), Some("Evidence Memo"));
        assert_eq!(info.metadata.creator.as_deref(), Some("Analyst"));
        assert_eq!(info.sections[0].paragraphs[0].text, "Hello source");
    }

    #[test]
    fn read_office_document_bytes_reads_rtf_source() {
        let info = read_office_document_bytes(
            "container.ad1:docs/note.rtf",
            br"{\rtf1\ansi First\par Second}",
        )
        .unwrap();

        assert_eq!(info.format, OfficeFormat::Rtf);
        assert_eq!(info.sections[0].paragraphs.len(), 2);
        assert_eq!(info.total_words, 2);
    }
}
