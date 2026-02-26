// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! OOXML Format Handling (DOCX, PPTX)
//!
//! Extracts text and metadata from ZIP-based OOXML documents
//! using `zip` + `quick-xml`.

use std::io::Read;
use std::path::Path;

use crate::viewer::document::error::{DocumentError, DocumentResult};
use super::{OfficeMetadata, OfficeTextSection};

// =============================================================================
// OOXML Metadata (shared by DOCX and PPTX)
// =============================================================================

/// Extract metadata from OOXML (ZIP-based) documents.
///
/// Reads `docProps/core.xml` for Dublin Core metadata and
/// `docProps/app.xml` for application-specific properties.
pub(crate) fn extract_ooxml_metadata(
    path: &Path,
    core_path: &str,
    app_path: &str,
) -> DocumentResult<OfficeMetadata> {
    let file = std::fs::File::open(path)?;
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
pub(crate) fn extract_docx_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid DOCX: {}", e)))?;

    let mut xml_data = String::new();
    {
        let mut entry = archive.by_name("word/document.xml")
            .map_err(|e| DocumentError::Parse(format!("Missing word/document.xml: {}", e)))?;
        entry.read_to_string(&mut xml_data)?;
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
pub(crate) fn extract_pptx_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)?;
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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
}
