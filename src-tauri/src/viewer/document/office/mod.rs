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

mod ooxml;
mod cfb;
mod odf;
mod rtf;

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
            metadata = ooxml::extract_ooxml_metadata(path, "docProps/core.xml", "docProps/app.xml")?;
            sections = ooxml::extract_docx_text(path)?;
        }
        OfficeFormat::Pptx => {
            metadata = ooxml::extract_ooxml_metadata(path, "docProps/core.xml", "docProps/app.xml")?;
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
                        paragraphs: vec!["[Could not extract text from legacy .doc file]".to_string()],
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
                        paragraphs: vec!["[Could not extract text from legacy .ppt file]".to_string()],
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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
