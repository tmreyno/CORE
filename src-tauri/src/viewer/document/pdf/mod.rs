// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified PDF document handler
//!
//! This module provides both reading and writing capabilities for PDF documents.
//!
//! # Reading (`mod.rs`)
//! - Uses `pdf-extract` for text extraction
//! - Uses `lopdf` for structure parsing
//!
//! # Writing (`writer.rs`)
//! - Uses `genpdf` for report generation with proper fonts
//!
//! The frontend receives rendered HTML for display, eliminating the need for
//! pdf.js in most cases.

use std::path::Path;

use super::error::{DocumentError, DocumentResult};
use super::types::*;
use super::DocumentFormat;

pub mod writer;

const MAX_PDF_SOURCE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_PDF_EXTRACTED_TEXT_CHARS: usize = 5_000_000;
const MAX_PDF_RESPONSE_PAGES: usize = 2_000;
const MAX_PDF_ELEMENTS_PER_PAGE: usize = 5_000;
const MAX_PDF_ELEMENT_TEXT_CHARS: usize = 16_384;
const MAX_PDF_METADATA_CHARS: usize = 4_096;
const MAX_PDF_KEYWORDS: usize = 512;

fn ensure_pdf_size_allowed(size: u64) -> DocumentResult<()> {
    if size > MAX_PDF_SOURCE_BYTES {
        return Err(DocumentError::Pdf(format!(
            "PDF file too large for extraction ({:.1} MiB, max {} MiB)",
            size as f64 / (1024.0 * 1024.0),
            MAX_PDF_SOURCE_BYTES / (1024 * 1024)
        )));
    }
    Ok(())
}

fn truncate_pdf_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("...");
    truncated
}

fn truncate_option_string(value: &mut Option<String>, max_chars: usize) {
    if let Some(text) = value {
        *text = truncate_pdf_text(text, max_chars);
    }
}

/// PDF document handler with read/write capabilities
pub struct PdfDocument {
    /// Cached font family for writing (currently unused, reserved for future caching)
    #[allow(dead_code)]
    font_family: Option<genpdf::fonts::FontFamily<genpdf::fonts::FontData>>,
}

impl PdfDocument {
    /// Create a new PDF document handler
    pub fn new() -> Self {
        Self { font_family: None }
    }

    // =========================================================================
    // READING
    // =========================================================================

    /// Read PDF from file path
    pub fn read(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let path = path.as_ref();
        ensure_pdf_size_allowed(std::fs::metadata(path)?.len())?;
        let data = std::fs::read(path)?;
        self.read_bytes(&data)
    }

    /// Read PDF from bytes
    pub fn read_bytes(&self, data: &[u8]) -> DocumentResult<DocumentContent> {
        ensure_pdf_size_allowed(data.len() as u64)?;

        // Extract text using pdf-extract crate
        let text = truncate_pdf_text(
            &self.extract_text_from_bytes(data)?,
            MAX_PDF_EXTRACTED_TEXT_CHARS,
        );

        // Try to parse structure with lopdf for better formatting
        let (metadata, pages) = self.parse_structure(data, &text)?;

        Ok(DocumentContent { metadata, pages })
    }

    /// Extract text from PDF bytes
    fn extract_text_from_bytes(&self, data: &[u8]) -> DocumentResult<String> {
        // Use pdf_extract crate
        pdf_extract::extract_text_from_mem(data)
            .map_err(|e| DocumentError::Pdf(format!("Text extraction failed: {}", e)))
    }

    /// Parse PDF structure to get metadata and page breaks
    fn parse_structure(
        &self,
        data: &[u8],
        extracted_text: &str,
    ) -> DocumentResult<(DocumentMetadata, Vec<DocumentPage>)> {
        // Try to use lopdf for structure
        let doc = lopdf::Document::load_mem(data)
            .map_err(|e| DocumentError::Pdf(format!("Failed to parse PDF: {}", e)))?;

        // Extract metadata from document info dictionary
        let mut metadata = self.extract_metadata(&doc)?;
        Self::normalize_metadata(&mut metadata);

        // Get page count
        let page_count = doc.get_pages().len();

        // Split text into pages (approximate)
        let pages = self.split_into_pages(extracted_text, page_count);

        Ok((metadata, pages))
    }

    /// Extract metadata from PDF document
    fn extract_metadata(&self, doc: &lopdf::Document) -> DocumentResult<DocumentMetadata> {
        let mut metadata = DocumentMetadata {
            format: DocumentFormat::Pdf,
            page_count: Some(doc.get_pages().len()),
            ..Default::default()
        };

        // Metadata is optional, but a present malformed Info dictionary should be visible.
        if let Ok(info_obj) = doc.trailer.get(b"Info") {
            let info_dict = match info_obj {
                lopdf::Object::Reference(info_ref) => doc
                    .get_object(*info_ref)
                    .map_err(|e| {
                        DocumentError::Pdf(format!("Failed to read PDF Info dictionary: {}", e))
                    })?
                    .as_dict()
                    .map_err(|e| {
                        DocumentError::Pdf(format!("PDF Info object is not a dictionary: {}", e))
                    })?,
                lopdf::Object::Dictionary(dict) => dict,
                lopdf::Object::Null => return Ok(metadata),
                _ => {
                    return Err(DocumentError::Pdf(
                        "PDF trailer Info entry is not a dictionary reference".to_string(),
                    ));
                }
            };
            self.extract_metadata_from_info_dict(info_dict, &mut metadata)?;
        }

        Ok(metadata)
    }

    fn extract_metadata_from_info_dict(
        &self,
        dict: &lopdf::Dictionary,
        metadata: &mut DocumentMetadata,
    ) -> DocumentResult<()> {
        if let Ok(title) = dict.get(b"Title") {
            metadata.title = Some(Self::pdf_metadata_string_to_string("Title", title)?);
        }
        if let Ok(author) = dict.get(b"Author") {
            metadata.author = Some(Self::pdf_metadata_string_to_string("Author", author)?);
        }
        if let Ok(subject) = dict.get(b"Subject") {
            metadata.subject = Some(Self::pdf_metadata_string_to_string("Subject", subject)?);
        }
        if let Ok(creator) = dict.get(b"Creator") {
            metadata.creator = Some(Self::pdf_metadata_string_to_string("Creator", creator)?);
        }
        if let Ok(producer) = dict.get(b"Producer") {
            metadata.producer = Some(Self::pdf_metadata_string_to_string("Producer", producer)?);
        }
        if let Ok(keywords) = dict.get(b"Keywords") {
            let kw = Self::pdf_metadata_string_to_string("Keywords", keywords)?;
            metadata.keywords = kw
                .split(',')
                .take(MAX_PDF_KEYWORDS)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .map(|s| truncate_pdf_text(&s, MAX_PDF_METADATA_CHARS))
                .collect();
        }
        Ok(())
    }

    fn normalize_metadata(metadata: &mut DocumentMetadata) {
        truncate_option_string(&mut metadata.title, MAX_PDF_METADATA_CHARS);
        truncate_option_string(&mut metadata.author, MAX_PDF_METADATA_CHARS);
        truncate_option_string(&mut metadata.subject, MAX_PDF_METADATA_CHARS);
        truncate_option_string(&mut metadata.creator, MAX_PDF_METADATA_CHARS);
        truncate_option_string(&mut metadata.producer, MAX_PDF_METADATA_CHARS);
        metadata.keywords = metadata
            .keywords
            .iter()
            .take(MAX_PDF_KEYWORDS)
            .map(|keyword| truncate_pdf_text(keyword, MAX_PDF_METADATA_CHARS))
            .collect();
    }

    fn pdf_metadata_string_to_string(
        field_name: &str,
        obj: &lopdf::Object,
    ) -> DocumentResult<String> {
        Self::pdf_string_to_string(obj)
            .map(|value| truncate_pdf_text(&value, MAX_PDF_METADATA_CHARS))
            .map_err(|e| {
                DocumentError::Pdf(format!(
                    "Failed to decode PDF metadata field '{}': {}",
                    field_name, e
                ))
            })
    }

    /// Convert PDF string object to Rust String
    fn pdf_string_to_string(obj: &lopdf::Object) -> Result<String, lopdf::Error> {
        match obj {
            lopdf::Object::String(bytes, _) => {
                if bytes.starts_with(b"\xFE\xFF") && (bytes.len() - 2) % 2 != 0 {
                    return Err(lopdf::Error::StringDecode);
                }
                lopdf::decode_text_string(obj)
            }
            _ => Err(lopdf::Error::Type),
        }
    }

    /// Split extracted text into approximate pages
    fn split_into_pages(&self, text: &str, page_count: usize) -> Vec<DocumentPage> {
        if page_count == 0 || text.is_empty() {
            return vec![DocumentPage {
                page_number: 1,
                elements: vec![DocumentElement::Paragraph(ParagraphElement {
                    text: truncate_pdf_text(text, MAX_PDF_ELEMENT_TEXT_CHARS),
                    style: TextStyle::default(),
                })],
            }];
        }

        fn next_char_boundary(text: &str, index: usize) -> usize {
            if index >= text.len() || text.is_char_boundary(index) {
                return index.min(text.len());
            }

            let mut boundary = index;
            while boundary < text.len() && !text.is_char_boundary(boundary) {
                boundary += 1;
            }
            boundary.min(text.len())
        }

        // Try to find form feed characters (page breaks)
        let pages_by_ff: Vec<&str> = text.split('\u{000C}').collect();

        if pages_by_ff.len() > 1 {
            // Use form feed splits
            return pages_by_ff
                .into_iter()
                .take(MAX_PDF_RESPONSE_PAGES)
                .enumerate()
                .map(|(i, page_text)| DocumentPage {
                    page_number: i + 1,
                    elements: self.parse_text_elements(page_text),
                })
                .collect();
        }

        // Approximate split by text length while honoring UTF-8 character boundaries.
        let response_page_count = page_count.clamp(1, MAX_PDF_RESPONSE_PAGES);
        let bytes_per_page = (text.len() / response_page_count).max(1);
        let mut pages = Vec::with_capacity(response_page_count);
        let mut remaining = text;
        let mut page_num = 1;

        while !remaining.is_empty() && page_num <= response_page_count {
            let split_at = if page_num == response_page_count {
                remaining.len()
            } else {
                // Try to split at paragraph boundary
                let target = next_char_boundary(remaining, bytes_per_page.min(remaining.len()));
                remaining[..target]
                    .rfind("\n\n")
                    .map(|p| p + 2)
                    .or_else(|| remaining[..target].rfind('\n').map(|p| p + 1))
                    .unwrap_or(target)
            };

            let (page_text, rest) = remaining.split_at(split_at);
            pages.push(DocumentPage {
                page_number: page_num,
                elements: self.parse_text_elements(page_text),
            });
            remaining = rest;
            page_num += 1;
        }

        if pages.is_empty() {
            pages.push(DocumentPage {
                page_number: 1,
                elements: vec![DocumentElement::Paragraph(ParagraphElement {
                    text: truncate_pdf_text(text, MAX_PDF_ELEMENT_TEXT_CHARS),
                    style: TextStyle::default(),
                })],
            });
        }

        pages
    }

    /// Parse text into document elements (headings, paragraphs, etc.)
    fn parse_text_elements(&self, text: &str) -> Vec<DocumentElement> {
        let mut elements = Vec::new();

        for paragraph in text.split("\n\n").take(MAX_PDF_ELEMENTS_PER_PAGE) {
            let trimmed = paragraph.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Detect headings (all caps, short lines)
            if trimmed.len() < 100
                && trimmed
                    .chars()
                    .filter(|c| c.is_alphabetic())
                    .all(|c| c.is_uppercase())
                && trimmed.chars().any(|c| c.is_alphabetic())
            {
                elements.push(DocumentElement::Heading(HeadingElement {
                    text: truncate_pdf_text(trimmed, MAX_PDF_ELEMENT_TEXT_CHARS),
                    level: 2,
                }));
            } else {
                elements.push(DocumentElement::Paragraph(ParagraphElement {
                    text: truncate_pdf_text(trimmed, MAX_PDF_ELEMENT_TEXT_CHARS),
                    style: TextStyle::default(),
                }));
            }
        }

        if elements.is_empty() {
            elements.push(DocumentElement::Paragraph(ParagraphElement {
                text: truncate_pdf_text(text.trim(), MAX_PDF_ELEMENT_TEXT_CHARS),
                style: TextStyle::default(),
            }));
        }

        elements
    }

    /// Get metadata without reading full document
    pub fn get_metadata(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentMetadata> {
        let path = path.as_ref();
        let file_size = std::fs::metadata(path)?.len();
        ensure_pdf_size_allowed(file_size)?;

        let data = std::fs::read(path)?;
        let doc = lopdf::Document::load_mem(&data)
            .map_err(|e| DocumentError::Pdf(format!("Failed to parse PDF: {}", e)))?;

        let mut metadata = self.extract_metadata(&doc)?;
        Self::normalize_metadata(&mut metadata);
        metadata.file_size = file_size;
        Ok(metadata)
    }
}

impl Default for PdfDocument {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // =========================================================================
    // Constructor / Default
    // =========================================================================

    #[test]
    fn test_new_creates_instance() {
        let doc = PdfDocument::new();
        assert!(doc.font_family.is_none());
    }

    #[test]
    fn test_default_creates_instance() {
        let doc = PdfDocument::default();
        assert!(doc.font_family.is_none());
    }

    #[test]
    fn test_pdf_size_guard_rejects_oversized_source() {
        let err = ensure_pdf_size_allowed(MAX_PDF_SOURCE_BYTES + 1).unwrap_err();

        assert!(err.to_string().contains("too large for extraction"));
    }

    #[test]
    fn test_pdf_read_rejects_sparse_oversized_file_before_full_read() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"%PDF-1.7\n").unwrap();
        tmp.as_file_mut().set_len(MAX_PDF_SOURCE_BYTES + 1).unwrap();

        let err = PdfDocument::new().read(tmp.path()).unwrap_err();

        assert!(err.to_string().contains("too large for extraction"));
    }

    #[test]
    fn truncate_pdf_text_is_unicode_safe() {
        let value = "é".repeat(MAX_PDF_METADATA_CHARS + 1);

        let truncated = truncate_pdf_text(&value, MAX_PDF_METADATA_CHARS);

        assert!(truncated.ends_with("..."));
        assert_eq!(
            truncated.trim_end_matches("...").chars().count(),
            MAX_PDF_METADATA_CHARS
        );
    }

    // =========================================================================
    // pdf_string_to_string
    // =========================================================================

    #[test]
    fn test_pdf_string_to_string_utf8() {
        let obj = lopdf::Object::String(b"Hello World".to_vec(), lopdf::StringFormat::Literal);
        let result = PdfDocument::pdf_string_to_string(&obj).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_pdf_string_to_string_empty() {
        let obj = lopdf::Object::String(vec![], lopdf::StringFormat::Literal);
        let result = PdfDocument::pdf_string_to_string(&obj).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_pdf_string_to_string_utf16be_bom() {
        // UTF-16 BE BOM (0xFE 0xFF) followed by "AB" in UTF-16 BE
        let bytes = vec![0xFE, 0xFF, 0x00, 0x41, 0x00, 0x42];
        let obj = lopdf::Object::String(bytes, lopdf::StringFormat::Literal);
        let result = PdfDocument::pdf_string_to_string(&obj).unwrap();
        assert_eq!(result, "AB");
    }

    #[test]
    fn test_pdf_string_to_string_utf16be_unicode() {
        // UTF-16 BE BOM + "café" = U+0063 U+0061 U+0066 U+00E9
        let bytes = vec![
            0xFE, 0xFF, // BOM
            0x00, 0x63, // 'c'
            0x00, 0x61, // 'a'
            0x00, 0x66, // 'f'
            0x00, 0xE9, // 'é'
        ];
        let obj = lopdf::Object::String(bytes, lopdf::StringFormat::Literal);
        let result = PdfDocument::pdf_string_to_string(&obj).unwrap();
        assert_eq!(result, "café");
    }

    #[test]
    fn test_pdf_string_to_string_non_string_type() {
        let obj = lopdf::Object::Integer(42);
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_string_to_string_boolean_type() {
        let obj = lopdf::Object::Boolean(true);
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_string_to_string_name_type() {
        let obj = lopdf::Object::Name(b"SomeName".to_vec());
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_string_to_string_decodes_pdf_doc_encoding() {
        // No BOM means PDFDocEncoding, where 0x8B decodes to U+2030.
        let bytes = vec![b't', b'e', b'x', b't', 0x8B];
        let obj = lopdf::Object::String(bytes, lopdf::StringFormat::Literal);
        let result = PdfDocument::pdf_string_to_string(&obj).unwrap();

        assert_eq!(result, "text‰");
    }

    #[test]
    fn test_pdf_string_to_string_rejects_odd_utf16be() {
        let obj = lopdf::Object::String(vec![0xFE, 0xFF, 0x00], lopdf::StringFormat::Hexadecimal);
        let result = PdfDocument::pdf_string_to_string(&obj);

        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_string_to_string_ascii() {
        let obj = lopdf::Object::String(
            b"Simple ASCII text 123!@#".to_vec(),
            lopdf::StringFormat::Hexadecimal,
        );
        let result = PdfDocument::pdf_string_to_string(&obj).unwrap();
        assert_eq!(result, "Simple ASCII text 123!@#");
    }

    #[test]
    fn extract_metadata_from_info_dict_decodes_fields() {
        let doc = PdfDocument::new();
        let mut dict = lopdf::Dictionary::new();
        dict.set(
            b"Title".to_vec(),
            lopdf::Object::String(b"Report".to_vec(), lopdf::StringFormat::Literal),
        );
        dict.set(
            b"Keywords".to_vec(),
            lopdf::Object::String(
                b"forensics, evidence".to_vec(),
                lopdf::StringFormat::Literal,
            ),
        );
        let mut metadata = DocumentMetadata::default();

        doc.extract_metadata_from_info_dict(&dict, &mut metadata)
            .unwrap();

        assert_eq!(metadata.title.as_deref(), Some("Report"));
        assert_eq!(metadata.keywords, vec!["forensics", "evidence"]);
    }

    #[test]
    fn extract_metadata_from_info_dict_rejects_bad_string() {
        let doc = PdfDocument::new();
        let mut dict = lopdf::Dictionary::new();
        dict.set(
            b"Title".to_vec(),
            lopdf::Object::String(vec![0xFE, 0xFF, 0x00], lopdf::StringFormat::Hexadecimal),
        );
        let mut metadata = DocumentMetadata::default();

        let err = doc
            .extract_metadata_from_info_dict(&dict, &mut metadata)
            .unwrap_err();

        assert!(err.to_string().contains("Title"));
    }

    #[test]
    fn normalize_metadata_caps_fields_and_keywords() {
        let mut metadata = DocumentMetadata {
            title: Some("é".repeat(MAX_PDF_METADATA_CHARS + 1)),
            keywords: vec!["é".repeat(MAX_PDF_METADATA_CHARS + 1); MAX_PDF_KEYWORDS + 8],
            ..Default::default()
        };

        PdfDocument::normalize_metadata(&mut metadata);

        let title = metadata.title.as_deref().unwrap();
        assert!(title.ends_with("..."));
        assert_eq!(
            title.trim_end_matches("...").chars().count(),
            MAX_PDF_METADATA_CHARS
        );

        assert_eq!(metadata.keywords.len(), MAX_PDF_KEYWORDS);
        assert!(metadata.keywords[0].ends_with("..."));
        assert_eq!(
            metadata.keywords[0].trim_end_matches("...").chars().count(),
            MAX_PDF_METADATA_CHARS
        );
    }

    // =========================================================================
    // split_into_pages
    // =========================================================================

    #[test]
    fn test_split_into_pages_empty_text() {
        let doc = PdfDocument::new();
        let pages = doc.split_into_pages("", 0);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].page_number, 1);
    }

    #[test]
    fn test_split_into_pages_single_page() {
        let doc = PdfDocument::new();
        let pages = doc.split_into_pages("Hello World", 1);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].page_number, 1);
        // Should contain the text as a paragraph element
        assert!(!pages[0].elements.is_empty());
    }

    #[test]
    fn test_split_into_pages_with_form_feeds() {
        let doc = PdfDocument::new();
        let text = "Page 1 content\x0CPage 2 content\x0CPage 3 content";
        let pages = doc.split_into_pages(text, 3);
        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].page_number, 1);
        assert_eq!(pages[1].page_number, 2);
        assert_eq!(pages[2].page_number, 3);
    }

    #[test]
    fn test_split_into_pages_form_feed_overrides_page_count() {
        let doc = PdfDocument::new();
        // 2 form feeds = 3 sections, even though page_count says 5
        let text = "Part A\x0CPart B\x0CPart C";
        let pages = doc.split_into_pages(text, 5);
        assert_eq!(pages.len(), 3);
    }

    #[test]
    fn test_split_into_pages_caps_form_feed_pages() {
        let doc = PdfDocument::new();
        let text = std::iter::repeat_n("page", MAX_PDF_RESPONSE_PAGES + 8)
            .collect::<Vec<_>>()
            .join("\x0C");

        let pages = doc.split_into_pages(&text, MAX_PDF_RESPONSE_PAGES + 8);

        assert_eq!(pages.len(), MAX_PDF_RESPONSE_PAGES);
    }

    #[test]
    fn test_split_into_pages_caps_approximate_pages() {
        let doc = PdfDocument::new();
        let text = "word ".repeat(MAX_PDF_RESPONSE_PAGES + 8);

        let pages = doc.split_into_pages(&text, MAX_PDF_RESPONSE_PAGES + 8);

        assert_eq!(pages.len(), MAX_PDF_RESPONSE_PAGES);
    }

    #[test]
    fn test_split_into_pages_approximate_split() {
        let doc = PdfDocument::new();
        // No form feeds, split by character count
        let text = "First paragraph of content.\n\nSecond paragraph of content.\n\nThird paragraph of content.";
        let pages = doc.split_into_pages(text, 3);
        assert_eq!(pages.len(), 3);
        for (i, page) in pages.iter().enumerate() {
            assert_eq!(page.page_number, i + 1);
        }
    }

    #[test]
    fn test_split_into_pages_zero_page_count_with_text() {
        let doc = PdfDocument::new();
        let pages = doc.split_into_pages("Some text here", 0);
        // With page_count == 0, returns single page with all text
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn test_split_into_pages_preserves_all_text() {
        let doc = PdfDocument::new();
        let text = "Content A\x0CContent B";
        let pages = doc.split_into_pages(text, 2);
        // Verify both pages have elements
        for page in &pages {
            assert!(!page.elements.is_empty());
        }
    }

    #[test]
    fn test_split_into_pages_handles_multibyte_utf8_boundaries() {
        let doc = PdfDocument::new();
        let text = "😀😀😀😀";
        let pages = doc.split_into_pages(text, 3);

        let reconstructed = pages
            .iter()
            .flat_map(|page| page.elements.iter())
            .filter_map(|element| match element {
                DocumentElement::Paragraph(p) => Some(p.text.as_str()),
                _ => None,
            })
            .collect::<String>();

        assert_eq!(reconstructed, text);
    }

    // =========================================================================
    // parse_text_elements
    // =========================================================================

    #[test]
    fn test_parse_text_elements_simple_paragraph() {
        let doc = PdfDocument::new();
        let elements = doc.parse_text_elements("Hello world, this is a test.");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            DocumentElement::Paragraph(p) => {
                assert_eq!(p.text, "Hello world, this is a test.");
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_parse_text_elements_multiple_paragraphs() {
        let doc = PdfDocument::new();
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let elements = doc.parse_text_elements(text);
        assert_eq!(elements.len(), 3);
        for elem in &elements {
            match elem {
                DocumentElement::Paragraph(_) => {}
                _ => panic!("Expected Paragraph elements"),
            }
        }
    }

    #[test]
    fn test_parse_text_elements_caps_element_count() {
        let doc = PdfDocument::new();
        let text = std::iter::repeat_n("paragraph", MAX_PDF_ELEMENTS_PER_PAGE + 8)
            .collect::<Vec<_>>()
            .join("\n\n");

        let elements = doc.parse_text_elements(&text);

        assert_eq!(elements.len(), MAX_PDF_ELEMENTS_PER_PAGE);
    }

    #[test]
    fn test_parse_text_elements_truncates_long_paragraph() {
        let doc = PdfDocument::new();
        let text = "é".repeat(MAX_PDF_ELEMENT_TEXT_CHARS + 1);

        let elements = doc.parse_text_elements(&text);

        match &elements[0] {
            DocumentElement::Paragraph(p) => {
                assert!(p.text.ends_with("..."));
                assert_eq!(
                    p.text.trim_end_matches("...").chars().count(),
                    MAX_PDF_ELEMENT_TEXT_CHARS
                );
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_parse_text_elements_heading_detection() {
        let doc = PdfDocument::new();
        let text = "INTRODUCTION\n\nThis is the body text.";
        let elements = doc.parse_text_elements(text);
        assert_eq!(elements.len(), 2);
        match &elements[0] {
            DocumentElement::Heading(h) => {
                assert_eq!(h.text, "INTRODUCTION");
                assert_eq!(h.level, 2);
            }
            _ => panic!("Expected Heading element for all-caps short text"),
        }
        match &elements[1] {
            DocumentElement::Paragraph(p) => {
                assert_eq!(p.text, "This is the body text.");
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_parse_text_elements_heading_with_numbers() {
        let doc = PdfDocument::new();
        // Numbers don't prevent heading detection - "SECTION 1" has alpha chars all uppercase
        let text = "SECTION 1\n\nContent here.";
        let elements = doc.parse_text_elements(text);
        assert_eq!(elements.len(), 2);
        match &elements[0] {
            DocumentElement::Heading(h) => {
                assert_eq!(h.text, "SECTION 1");
            }
            _ => panic!("Expected Heading for 'SECTION 1'"),
        }
    }

    #[test]
    fn test_parse_text_elements_long_uppercase_not_heading() {
        let doc = PdfDocument::new();
        // Lines >= 100 chars should not be detected as headings even if all caps
        let long_caps = "A".repeat(100);
        let elements = doc.parse_text_elements(&long_caps);
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            DocumentElement::Paragraph(_) => {}
            _ => panic!("Long all-caps text should be a Paragraph, not a Heading"),
        }
    }

    #[test]
    fn test_parse_text_elements_mixed_case_not_heading() {
        let doc = PdfDocument::new();
        let text = "Introduction\n\nBody text.";
        let elements = doc.parse_text_elements(text);
        // "Introduction" has lowercase letters, should be a paragraph
        match &elements[0] {
            DocumentElement::Paragraph(p) => {
                assert_eq!(p.text, "Introduction");
            }
            _ => panic!("Mixed case should be a Paragraph"),
        }
    }

    #[test]
    fn test_parse_text_elements_empty_paragraphs_skipped() {
        let doc = PdfDocument::new();
        let text = "First.\n\n\n\n\n\nSecond.";
        let elements = doc.parse_text_elements(text);
        // Empty paragraphs between should be skipped
        assert_eq!(elements.len(), 2);
    }

    #[test]
    fn test_parse_text_elements_whitespace_only() {
        let doc = PdfDocument::new();
        let elements = doc.parse_text_elements("   \n\n   \n\n   ");
        // All whitespace paragraphs should be skipped, fallback to single element
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            DocumentElement::Paragraph(p) => {
                assert!(p.text.trim().is_empty());
            }
            _ => panic!("Expected empty paragraph fallback"),
        }
    }

    #[test]
    fn test_parse_text_elements_numbers_only_not_heading() {
        let doc = PdfDocument::new();
        // "12345" has no alphabetic characters, should not be a heading
        let elements = doc.parse_text_elements("12345");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            DocumentElement::Paragraph(_) => {}
            _ => panic!("Numeric-only text should be a Paragraph"),
        }
    }

    #[test]
    fn test_parse_text_elements_preserves_text_content() {
        let doc = PdfDocument::new();
        let text = "Special chars: <>&\"'\n\nAnother paragraph!";
        let elements = doc.parse_text_elements(text);
        assert_eq!(elements.len(), 2);
        match &elements[0] {
            DocumentElement::Paragraph(p) => {
                assert_eq!(p.text, "Special chars: <>&\"'");
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    // =========================================================================
    // add_section_header helper (constructor smoke test)
    // =========================================================================

    #[test]
    fn test_add_section_header_does_not_panic() {
        // This test verifies the helper works without a full document context.
        // We can't easily inspect genpdf Document contents, but we can verify no panic.
        // add_section_header requires a genpdf Document, which requires fonts.
        // Just verify new() and default() don't panic.
        let _ = PdfDocument::new();
        let _ = PdfDocument::default();
    }
}
