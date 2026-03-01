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

use super::{OfficeMetadata, OfficeParagraph, OfficeTextSection, ParagraphHint};
use crate::viewer::document::error::{DocumentError, DocumentResult};

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
/// Reads `<w:p>` (paragraph) and `<w:t>` (text run) elements,
/// detecting heading styles from `<w:pStyle>` for rendering hints.
pub(crate) fn extract_docx_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::Parse(format!("Not a valid DOCX: {}", e)))?;

    let mut xml_data = String::new();
    {
        let mut entry = archive
            .by_name("word/document.xml")
            .map_err(|e| DocumentError::Parse(format!("Missing word/document.xml: {}", e)))?;
        entry.read_to_string(&mut xml_data)?;
    }

    let paragraphs = extract_docx_styled_paragraphs(&xml_data);

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

        let paragraphs = extract_ooxml_paragraphs_simple(&xml_data, b"a:p", b"a:t");
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

/// Map a DOCX paragraph style name to a ParagraphHint.
///
/// Handles common Word style names (case-insensitive).
fn style_to_hint(style: &str) -> ParagraphHint {
    let lower = style.to_lowercase();
    // Heading styles: "Heading1", "heading 1", "Heading1", "Titre1" (French), etc.
    if lower.starts_with("heading") || lower.starts_with("titre") {
        // Extract the digit
        let digit = lower.chars().find(|c| c.is_ascii_digit());
        return match digit {
            Some('1') => ParagraphHint::Heading1,
            Some('2') => ParagraphHint::Heading2,
            Some('3') => ParagraphHint::Heading3,
            _ => ParagraphHint::Heading4,
        };
    }
    match lower.as_str() {
        "title" | "titel" => ParagraphHint::Title,
        "subtitle" | "untertitel" => ParagraphHint::Subtitle,
        "listparagraph" | "list paragraph" | "listenabsatz" => ParagraphHint::ListItem,
        "quote" | "intensequote" | "blocktext" | "zitat" => ParagraphHint::Quote,
        _ => ParagraphHint::Normal,
    }
}

/// Extract styled paragraphs from a DOCX XML document.
///
/// Detects `<w:pStyle>` to identify headings, titles, lists, and quotes.
fn extract_docx_styled_paragraphs(xml: &str) -> Vec<OfficeParagraph> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    let mut paragraphs = Vec::new();
    let mut current_text = String::new();
    let mut current_hint = ParagraphHint::Normal;
    let mut in_text = false;
    let mut in_paragraph = false;
    let mut in_ppr = false; // inside <w:pPr>
    let mut has_num_pr = false; // has <w:numPr> (list numbering)

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let name_ref = name.as_ref();
                if name_ref == b"w:p" {
                    in_paragraph = true;
                    current_text.clear();
                    current_hint = ParagraphHint::Normal;
                    has_num_pr = false;
                } else if name_ref == b"w:pPr" && in_paragraph {
                    in_ppr = true;
                } else if name_ref == b"w:pStyle" && in_ppr {
                    // Read the w:val attribute for the style name
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            let val = String::from_utf8_lossy(&attr.value);
                            current_hint = style_to_hint(&val);
                        }
                    }
                } else if name_ref == b"w:numPr" && in_ppr {
                    has_num_pr = true;
                } else if name_ref == b"w:t" && in_paragraph {
                    in_text = true;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let name_ref = name.as_ref();
                if name_ref == b"w:pStyle" && in_ppr {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"val" {
                            let val = String::from_utf8_lossy(&attr.value);
                            current_hint = style_to_hint(&val);
                        }
                    }
                } else if name_ref == b"w:numPr" && in_ppr {
                    has_num_pr = true;
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                if let Ok(text) = e.unescape() {
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let name_ref = name.as_ref();
                if name_ref == b"w:t" {
                    in_text = false;
                } else if name_ref == b"w:pPr" {
                    in_ppr = false;
                } else if name_ref == b"w:p" {
                    in_paragraph = false;
                    let trimmed = current_text.trim().to_string();
                    if !trimmed.is_empty() {
                        // If numPr is present and no explicit heading/title style, mark as list item
                        if has_num_pr && current_hint == ParagraphHint::Normal {
                            current_hint = ParagraphHint::ListItem;
                        }
                        paragraphs.push(OfficeParagraph {
                            text: trimmed,
                            hint: current_hint,
                        });
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

/// Extract paragraphs from OOXML XML content (simple mode, no style detection).
///
/// Used for PPTX slides where paragraph styles are less meaningful.
/// Returns `OfficeParagraph` with `Normal` hint.
fn extract_ooxml_paragraphs_simple(
    xml: &str,
    para_tag: &[u8],
    text_tag: &[u8],
) -> Vec<OfficeParagraph> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

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
                        paragraphs.push(OfficeParagraph::normal(trimmed));
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

        let paragraphs = extract_docx_styled_paragraphs(xml);
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0].text, "Hello World");
        assert_eq!(paragraphs[0].hint, ParagraphHint::Normal);
        assert_eq!(paragraphs[1].text, "Second paragraph");
    }

    #[test]
    fn test_extract_docx_headings() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:body>
                <w:p>
                    <w:pPr><w:pStyle w:val="Title"/></w:pPr>
                    <w:r><w:t>Document Title</w:t></w:r>
                </w:p>
                <w:p>
                    <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
                    <w:r><w:t>Chapter One</w:t></w:r>
                </w:p>
                <w:p>
                    <w:r><w:t>Body text here.</w:t></w:r>
                </w:p>
                <w:p>
                    <w:pPr><w:pStyle w:val="Heading2"/></w:pPr>
                    <w:r><w:t>Section 1.1</w:t></w:r>
                </w:p>
                <w:p>
                    <w:pPr><w:pStyle w:val="ListParagraph"/><w:numPr/></w:pPr>
                    <w:r><w:t>List item</w:t></w:r>
                </w:p>
            </w:body>
        </w:document>"#;

        let paragraphs = extract_docx_styled_paragraphs(xml);
        assert_eq!(paragraphs.len(), 5);
        assert_eq!(paragraphs[0].hint, ParagraphHint::Title);
        assert_eq!(paragraphs[1].hint, ParagraphHint::Heading1);
        assert_eq!(paragraphs[2].hint, ParagraphHint::Normal);
        assert_eq!(paragraphs[3].hint, ParagraphHint::Heading2);
        assert_eq!(paragraphs[4].hint, ParagraphHint::ListItem);
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

        let paragraphs = extract_ooxml_paragraphs_simple(xml, b"a:p", b"a:t");
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(paragraphs[0].text, "Slide Title");
        assert_eq!(paragraphs[1].text, "Bullet point one");
    }

    #[test]
    fn test_extract_ooxml_paragraphs_empty() {
        let xml = r#"<w:document><w:body></w:body></w:document>"#;
        let paragraphs = extract_docx_styled_paragraphs(xml);
        assert!(paragraphs.is_empty());
    }

    #[test]
    fn test_style_to_hint() {
        assert_eq!(style_to_hint("Heading1"), ParagraphHint::Heading1);
        assert_eq!(style_to_hint("heading 2"), ParagraphHint::Heading2);
        assert_eq!(style_to_hint("Heading3"), ParagraphHint::Heading3);
        assert_eq!(style_to_hint("Title"), ParagraphHint::Title);
        assert_eq!(style_to_hint("Subtitle"), ParagraphHint::Subtitle);
        assert_eq!(style_to_hint("ListParagraph"), ParagraphHint::ListItem);
        assert_eq!(style_to_hint("Quote"), ParagraphHint::Quote);
        assert_eq!(style_to_hint("Normal"), ParagraphHint::Normal);
        assert_eq!(style_to_hint("SomeCustomStyle"), ParagraphHint::Normal);
    }
}
