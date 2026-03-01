// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! ODF Format Handling (ODT, ODP)
//!
//! Extracts text and metadata from OpenDocument Format files
//! using `zip` + `quick-xml`.

use std::io::Read;
use std::path::Path;

use super::{OfficeMetadata, OfficeParagraph, OfficeTextSection};
use crate::viewer::document::error::{DocumentError, DocumentResult};

// =============================================================================
// OpenDocument Metadata
// =============================================================================

/// Extract metadata from OpenDocument format files (ODT, ODP).
///
/// Reads `meta.xml` from the ZIP archive.
pub(crate) fn extract_odf_metadata(path: &Path) -> DocumentResult<OfficeMetadata> {
    let file = std::fs::File::open(path)?;
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
    use quick_xml::events::Event;
    use quick_xml::Reader;

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
                        let key =
                            String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
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
                        let key =
                            String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
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
pub(crate) fn extract_odt_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid ODT: {}", e)))?;

    let mut xml_data = String::new();
    {
        let mut entry = archive
            .by_name("content.xml")
            .map_err(|e| DocumentError::Parse(format!("Missing content.xml: {}", e)))?;
        entry.read_to_string(&mut xml_data)?;
    }

    let paragraphs = extract_odf_paragraphs(&xml_data)
        .into_iter()
        .map(OfficeParagraph::normal)
        .collect();

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
pub(crate) fn extract_odp_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid ODP: {}", e)))?;

    let mut xml_data = String::new();
    {
        let mut entry = archive
            .by_name("content.xml")
            .map_err(|e| DocumentError::Parse(format!("Missing content.xml: {}", e)))?;
        entry.read_to_string(&mut xml_data)?;
    }

    // For ODP, extract all paragraphs as one section (slide separation
    // would require tracking <draw:page> boundaries, which adds complexity)
    let paragraphs = extract_odf_paragraphs(&xml_data)
        .into_iter()
        .map(OfficeParagraph::normal)
        .collect();

    Ok(vec![OfficeTextSection {
        label: Some("Presentation".to_string()),
        paragraphs,
    }])
}

/// Extract paragraphs from ODF content.xml.
///
/// Looks for text content within `<text:p>` and `<text:span>` elements.
fn extract_odf_paragraphs(xml: &str) -> Vec<String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

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
                    depth = depth.saturating_sub(1);
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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
}
