// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Document types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::DocumentFormat;

/// Unified document content model
/// 
/// This structure represents the content of any document in a format-agnostic way,
/// allowing for conversion between formats and unified rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Document pages/sections
    pub pages: Vec<DocumentPage>,
}

impl DocumentContent {
    /// Create empty document content
    pub fn new() -> Self {
        Self {
            metadata: DocumentMetadata::default(),
            pages: Vec::new(),
        }
    }

    /// Create document content from plain text
    pub fn from_text(text: String) -> Self {
        let mut content = Self::new();
        content.pages.push(DocumentPage {
            page_number: 1,
            elements: vec![DocumentElement::Paragraph(ParagraphElement {
                text,
                style: TextStyle::default(),
            })],
        });
        content
    }

    /// Add a page to the document
    pub fn add_page(&mut self, page: DocumentPage) {
        self.pages.push(page);
    }

    /// Extract all text content as a single string
    pub fn to_plain_text(&self) -> String {
        let mut text = String::new();
        for page in &self.pages {
            for element in &page.elements {
                match element {
                    DocumentElement::Paragraph(p) => {
                        text.push_str(&p.text);
                        text.push('\n');
                    }
                    DocumentElement::Heading(h) => {
                        text.push_str(&h.text);
                        text.push('\n');
                    }
                    DocumentElement::Table(t) => {
                        for row in &t.rows {
                            for (i, cell) in row.cells.iter().enumerate() {
                                if i > 0 {
                                    text.push('\t');
                                }
                                text.push_str(&cell.text);
                            }
                            text.push('\n');
                        }
                    }
                    DocumentElement::List(list) => {
                        for item in &list.items {
                            text.push_str("• ");
                            text.push_str(&item.text);
                            text.push('\n');
                        }
                    }
                    DocumentElement::Image(_) => {
                        text.push_str("[Image]\n");
                    }
                    DocumentElement::Break => {
                        text.push('\n');
                    }
                }
            }
            text.push_str("\n---\n"); // Page separator
        }
        text.trim_end().to_string()
    }

    /// Convert to HTML for web rendering
    pub fn to_html(&self) -> String {
        let mut html = String::from(r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
:root {
    --bg: #0f1419;
    --text: #d4d4d4;
    --border: #2a3441;
    --accent: #569cd6;
    --heading: #4fc3f7;
    --table-header: #1a2028;
}
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: var(--bg);
    color: var(--text);
    line-height: 1.5;
    padding: 0;
    margin: 0;
}
h1, h2, h3, h4, h5, h6 {
    color: var(--heading);
    margin-top: 1em;
    margin-bottom: 0.5em;
}
h1 { font-size: 1.5em; border-bottom: 1px solid var(--accent); padding-bottom: 0.3em; }
h2 { font-size: 1.3em; }
h3 { font-size: 1.1em; }
p { margin: 0.5em 0; }
table {
    width: 100%;
    border-collapse: collapse;
    margin: 0.5em 0;
    font-size: 0.85em;
}
th, td {
    border: 1px solid var(--border);
    padding: 0.4em 0.6em;
    text-align: left;
    white-space: nowrap;
}
th {
    background: var(--table-header);
    font-weight: 600;
    position: sticky;
    top: 0;
}
tr:nth-child(even) { background: rgba(255,255,255,0.02); }
tr:hover { background: rgba(255,255,255,0.05); }
ul, ol { margin: 0.5em 0; padding-left: 1.5em; }
li { margin: 0.2em 0; }
.page {
    margin-bottom: 1rem;
}
.page-header {
    font-size: 0.75em;
    color: #6b7280;
    margin-bottom: 0.5em;
    padding-bottom: 0.25em;
    border-bottom: 1px solid var(--border);
}
img { max-width: 100%; height: auto; }
.image-container { text-align: center; margin: 1em 0; }
.image-caption { font-size: 0.85em; color: #888; margin-top: 0.5em; }
hr { border: none; border-top: 1px solid var(--border); margin: 1em 0; }
</style>
</head>
<body>
"#);

        // Add title if available
        if let Some(ref title) = self.metadata.title {
            html.push_str(&format!("<h1>{}</h1>\n", Self::escape_html(title)));
        }

        // Add author/metadata
        if let Some(ref author) = self.metadata.author {
            html.push_str(&format!("<p><em>Author: {}</em></p>\n", Self::escape_html(author)));
        }

        // Render pages
        for (i, page) in self.pages.iter().enumerate() {
            html.push_str(&format!(
                "<div class=\"page\">\n<div class=\"page-header\">Page {}</div>\n",
                i + 1
            ));
            
            for element in &page.elements {
                html.push_str(&element.to_html());
            }
            
            html.push_str("</div>\n");
        }

        html.push_str("</body>\n</html>");
        html
    }

    /// Escape HTML special characters
    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}

impl Default for DocumentContent {
    fn default() -> Self {
        Self::new()
    }
}

/// Document metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document title
    pub title: Option<String>,
    /// Author name
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Keywords/tags
    pub keywords: Vec<String>,
    /// Creator application
    pub creator: Option<String>,
    /// Producer application (for PDFs)
    pub producer: Option<String>,
    /// Creation date
    pub creation_date: Option<DateTime<Utc>>,
    /// Last modification date
    pub modification_date: Option<DateTime<Utc>>,
    /// Number of pages
    pub page_count: Option<usize>,
    /// Word count estimate
    pub word_count: Option<usize>,
    /// File size in bytes
    #[serde(default)]
    pub file_size: u64,
    /// Document format
    #[serde(default = "default_format")]
    pub format: DocumentFormat,
}

impl DocumentMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the document title
    #[inline]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    /// Set the document author
    #[inline]
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
    
    /// Set the document subject
    #[inline]
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }
    
    /// Add a keyword
    #[inline]
    pub fn with_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keywords.push(keyword.into());
        self
    }
    
    /// Set keywords from a list
    #[inline]
    pub fn with_keywords(mut self, keywords: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keywords.extend(keywords.into_iter().map(|k| k.into()));
        self
    }
    
    /// Set the creator application
    #[inline]
    pub fn with_creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }
    
    /// Set the producer application (PDF)
    #[inline]
    pub fn with_producer(mut self, producer: impl Into<String>) -> Self {
        self.producer = Some(producer.into());
        self
    }
    
    /// Set creation date
    #[inline]
    pub fn with_creation_date(mut self, date: DateTime<Utc>) -> Self {
        self.creation_date = Some(date);
        self
    }
    
    /// Set modification date
    #[inline]
    pub fn with_modification_date(mut self, date: DateTime<Utc>) -> Self {
        self.modification_date = Some(date);
        self
    }
    
    /// Set page count
    #[inline]
    pub fn with_page_count(mut self, count: usize) -> Self {
        self.page_count = Some(count);
        self
    }
    
    /// Set word count
    #[inline]
    pub fn with_word_count(mut self, count: usize) -> Self {
        self.word_count = Some(count);
        self
    }
    
    /// Set file size
    #[inline]
    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = size;
        self
    }
    
    /// Set document format
    #[inline]
    pub fn with_format(mut self, format: DocumentFormat) -> Self {
        self.format = format;
        self
    }
    
    /// Check if metadata has any meaningful content
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.author.is_none()
            && self.subject.is_none()
            && self.keywords.is_empty()
            && self.creator.is_none()
            && self.producer.is_none()
            && self.creation_date.is_none()
            && self.modification_date.is_none()
            && self.page_count.is_none()
            && self.word_count.is_none()
    }
}

fn default_format() -> DocumentFormat {
    DocumentFormat::Text
}

/// A page or section of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPage {
    /// Page number (1-indexed)
    pub page_number: usize,
    /// Content elements on this page
    pub elements: Vec<DocumentElement>,
}

/// Document content elements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentElement {
    /// Text paragraph
    Paragraph(ParagraphElement),
    /// Heading (h1-h6)
    Heading(HeadingElement),
    /// Table
    Table(TableElement),
    /// Bulleted or numbered list
    List(ListElement),
    /// Embedded image
    Image(ImageElement),
    /// Page/section break
    Break,
}

impl DocumentElement {
    /// Convert element to HTML
    pub fn to_html(&self) -> String {
        match self {
            Self::Paragraph(p) => {
                format!("<p>{}</p>\n", DocumentContent::escape_html(&p.text))
            }
            Self::Heading(h) => {
                let level = h.level.clamp(1, 6);
                format!("<h{}>{}</h{}>\n", level, DocumentContent::escape_html(&h.text), level)
            }
            Self::Table(t) => {
                let mut html = String::from("<table>\n");
                for (i, row) in t.rows.iter().enumerate() {
                    html.push_str("<tr>");
                    let tag = if i == 0 && t.has_header { "th" } else { "td" };
                    for cell in &row.cells {
                        html.push_str(&format!(
                            "<{}>{}</{}>",
                            tag,
                            DocumentContent::escape_html(&cell.text),
                            tag
                        ));
                    }
                    html.push_str("</tr>\n");
                }
                html.push_str("</table>\n");
                html
            }
            Self::List(list) => {
                let tag = if list.ordered { "ol" } else { "ul" };
                let mut html = format!("<{}>\n", tag);
                for item in &list.items {
                    html.push_str(&format!(
                        "<li>{}</li>\n",
                        DocumentContent::escape_html(&item.text)
                    ));
                }
                html.push_str(&format!("</{}>\n", tag));
                html
            }
            Self::Image(img) => {
                let mut html = String::from("<div class=\"image-container\">\n");
                if let Some(ref data) = img.data_base64 {
                    html.push_str(&format!(
                        "<img src=\"data:{};base64,{}\" alt=\"{}\">\n",
                        img.mime_type.as_deref().unwrap_or("image/png"),
                        data,
                        DocumentContent::escape_html(img.alt_text.as_deref().unwrap_or(""))
                    ));
                }
                if let Some(ref caption) = img.caption {
                    html.push_str(&format!(
                        "<div class=\"image-caption\">{}</div>\n",
                        DocumentContent::escape_html(caption)
                    ));
                }
                html.push_str("</div>\n");
                html
            }
            Self::Break => String::from("<hr>\n"),
        }
    }
}

/// Text paragraph element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphElement {
    /// Text content
    pub text: String,
    /// Text styling
    #[serde(default)]
    pub style: TextStyle,
}

/// Heading element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingElement {
    /// Heading text
    pub text: String,
    /// Heading level (1-6)
    pub level: u8,
}

/// Table element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableElement {
    /// Table rows
    pub rows: Vec<TableRow>,
    /// Whether first row is a header
    #[serde(default)]
    pub has_header: bool,
}

/// Table row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// Cells in this row
    pub cells: Vec<TableCell>,
}

/// Table cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// Cell text content
    pub text: String,
    /// Cell styling
    #[serde(default)]
    pub style: TextStyle,
}

/// List element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListElement {
    /// List items
    pub items: Vec<ListItem>,
    /// Whether list is ordered (numbered)
    #[serde(default)]
    pub ordered: bool,
}

/// List item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    /// Item text
    pub text: String,
    /// Nested list (for sub-items)
    pub nested: Option<Box<ListElement>>,
}

/// Image element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageElement {
    /// Base64 encoded image data
    pub data_base64: Option<String>,
    /// MIME type (e.g., "image/png")
    pub mime_type: Option<String>,
    /// Image width in pixels
    pub width: Option<u32>,
    /// Image height in pixels
    pub height: Option<u32>,
    /// Alt text for accessibility
    pub alt_text: Option<String>,
    /// Image caption
    pub caption: Option<String>,
}

/// Text styling options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextStyle {
    /// Bold text
    #[serde(default)]
    pub bold: bool,
    /// Italic text
    #[serde(default)]
    pub italic: bool,
    /// Underlined text
    #[serde(default)]
    pub underline: bool,
    /// Strikethrough text
    #[serde(default)]
    pub strikethrough: bool,
    /// Font size in points
    pub font_size: Option<f32>,
    /// Font family name
    pub font_family: Option<String>,
    /// Text color (hex)
    pub color: Option<String>,
    /// Background/highlight color (hex)
    pub background: Option<String>,
    /// Text alignment
    pub alignment: Option<TextAlignment>,
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

/// Page render output (for frontend display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRender {
    /// Page number (1-indexed)
    pub page_number: usize,
    /// Rendered HTML content
    pub html: String,
    /// Optional PNG thumbnail (base64)
    pub thumbnail: Option<String>,
    /// Page width in points
    pub width: Option<f32>,
    /// Page height in points
    pub height: Option<f32>,
}

/// Document render result for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRender {
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Total page count
    pub page_count: usize,
    /// Full HTML render of document
    pub html: String,
    /// Individual page renders (for pagination)
    pub pages: Vec<PageRender>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // =========================================================================
    // DocumentContent
    // =========================================================================

    #[test]
    fn document_content_new_is_empty() {
        let doc = DocumentContent::new();
        assert!(doc.pages.is_empty());
        assert!(doc.metadata.title.is_none());
        assert!(doc.metadata.author.is_none());
    }

    #[test]
    fn document_content_default_equals_new() {
        let a = DocumentContent::new();
        let b = DocumentContent::default();
        assert_eq!(a.pages.len(), b.pages.len());
    }

    #[test]
    fn document_content_from_text() {
        let doc = DocumentContent::from_text("Hello, world!".into());
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.pages[0].page_number, 1);
        assert_eq!(doc.pages[0].elements.len(), 1);
        match &doc.pages[0].elements[0] {
            DocumentElement::Paragraph(p) => assert_eq!(p.text, "Hello, world!"),
            other => panic!("Expected Paragraph, got {:?}", other),
        }
    }

    #[test]
    fn document_content_add_page() {
        let mut doc = DocumentContent::new();
        assert_eq!(doc.pages.len(), 0);
        doc.add_page(DocumentPage {
            page_number: 1,
            elements: vec![],
        });
        assert_eq!(doc.pages.len(), 1);
        doc.add_page(DocumentPage {
            page_number: 2,
            elements: vec![DocumentElement::Break],
        });
        assert_eq!(doc.pages.len(), 2);
    }

    #[test]
    fn document_content_to_plain_text_paragraph() {
        let doc = DocumentContent::from_text("Line one".into());
        let text = doc.to_plain_text();
        assert!(text.contains("Line one"));
    }

    #[test]
    fn document_content_to_plain_text_heading() {
        let mut doc = DocumentContent::new();
        doc.add_page(DocumentPage {
            page_number: 1,
            elements: vec![DocumentElement::Heading(HeadingElement {
                text: "Title".into(),
                level: 1,
            })],
        });
        let text = doc.to_plain_text();
        assert!(text.contains("Title"));
    }

    #[test]
    fn document_content_to_plain_text_table() {
        let mut doc = DocumentContent::new();
        doc.add_page(DocumentPage {
            page_number: 1,
            elements: vec![DocumentElement::Table(TableElement {
                rows: vec![TableRow {
                    cells: vec![
                        TableCell { text: "A".into(), style: TextStyle::default() },
                        TableCell { text: "B".into(), style: TextStyle::default() },
                    ],
                }],
                has_header: false,
            })],
        });
        let text = doc.to_plain_text();
        assert!(text.contains("A\tB"));
    }

    #[test]
    fn document_content_to_plain_text_list() {
        let mut doc = DocumentContent::new();
        doc.add_page(DocumentPage {
            page_number: 1,
            elements: vec![DocumentElement::List(ListElement {
                items: vec![
                    ListItem { text: "First".into(), nested: None },
                    ListItem { text: "Second".into(), nested: None },
                ],
                ordered: false,
            })],
        });
        let text = doc.to_plain_text();
        assert!(text.contains("• First"));
        assert!(text.contains("• Second"));
    }

    #[test]
    fn document_content_to_plain_text_image_placeholder() {
        let mut doc = DocumentContent::new();
        doc.add_page(DocumentPage {
            page_number: 1,
            elements: vec![DocumentElement::Image(ImageElement {
                data_base64: None,
                mime_type: None,
                width: None,
                height: None,
                alt_text: None,
                caption: None,
            })],
        });
        let text = doc.to_plain_text();
        assert!(text.contains("[Image]"));
    }

    #[test]
    fn document_content_to_plain_text_break() {
        let mut doc = DocumentContent::new();
        doc.add_page(DocumentPage {
            page_number: 1,
            elements: vec![
                DocumentElement::Paragraph(ParagraphElement {
                    text: "Before".into(),
                    style: TextStyle::default(),
                }),
                DocumentElement::Break,
                DocumentElement::Paragraph(ParagraphElement {
                    text: "After".into(),
                    style: TextStyle::default(),
                }),
            ],
        });
        let text = doc.to_plain_text();
        assert!(text.contains("Before"));
        assert!(text.contains("After"));
    }

    #[test]
    fn document_content_to_plain_text_page_separator() {
        let mut doc = DocumentContent::new();
        doc.add_page(DocumentPage {
            page_number: 1,
            elements: vec![DocumentElement::Paragraph(ParagraphElement {
                text: "Page 1".into(),
                style: TextStyle::default(),
            })],
        });
        doc.add_page(DocumentPage {
            page_number: 2,
            elements: vec![DocumentElement::Paragraph(ParagraphElement {
                text: "Page 2".into(),
                style: TextStyle::default(),
            })],
        });
        let text = doc.to_plain_text();
        assert!(text.contains("---"));
        assert!(text.contains("Page 1"));
        assert!(text.contains("Page 2"));
    }

    #[test]
    fn document_content_escape_html() {
        // escape_html is private, so test via to_html with special characters
        let doc = DocumentContent::from_text("<script>alert('xss')</script>".into());
        let html = doc.to_html();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&#39;"));
    }

    #[test]
    fn document_content_to_html_has_structure() {
        let doc = DocumentContent::from_text("Test content".into());
        let html = doc.to_html();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html>"));
        assert!(html.contains("</html>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("Test content"));
    }

    #[test]
    fn document_content_to_html_with_metadata() {
        let mut doc = DocumentContent::from_text("Body".into());
        doc.metadata.title = Some("My Title".into());
        doc.metadata.author = Some("Author Name".into());
        let html = doc.to_html();
        assert!(html.contains("<h1>My Title</h1>"));
        assert!(html.contains("Author: Author Name"));
    }

    #[test]
    fn document_content_to_html_page_numbers() {
        let mut doc = DocumentContent::new();
        doc.add_page(DocumentPage { page_number: 1, elements: vec![] });
        doc.add_page(DocumentPage { page_number: 2, elements: vec![] });
        let html = doc.to_html();
        assert!(html.contains("Page 1"));
        assert!(html.contains("Page 2"));
    }

    // =========================================================================
    // DocumentMetadata builder
    // =========================================================================

    #[test]
    fn metadata_new_is_empty() {
        let meta = DocumentMetadata::new();
        assert!(meta.is_empty());
        assert!(meta.title.is_none());
        assert!(meta.author.is_none());
        assert!(meta.keywords.is_empty());
        assert_eq!(meta.file_size, 0);
    }

    #[test]
    fn metadata_builder_chain() {
        let now = Utc::now();
        let meta = DocumentMetadata::new()
            .with_title("Report")
            .with_author("Examiner")
            .with_subject("Forensic Analysis")
            .with_keyword("evidence")
            .with_creator("CORE-FFX")
            .with_producer("CORE-FFX v1.0")
            .with_creation_date(now)
            .with_modification_date(now)
            .with_page_count(10)
            .with_word_count(5000)
            .with_file_size(1024)
            .with_format(DocumentFormat::Pdf);

        assert_eq!(meta.title.as_deref(), Some("Report"));
        assert_eq!(meta.author.as_deref(), Some("Examiner"));
        assert_eq!(meta.subject.as_deref(), Some("Forensic Analysis"));
        assert_eq!(meta.keywords, vec!["evidence"]);
        assert_eq!(meta.creator.as_deref(), Some("CORE-FFX"));
        assert_eq!(meta.producer.as_deref(), Some("CORE-FFX v1.0"));
        assert_eq!(meta.creation_date, Some(now));
        assert_eq!(meta.modification_date, Some(now));
        assert_eq!(meta.page_count, Some(10));
        assert_eq!(meta.word_count, Some(5000));
        assert_eq!(meta.file_size, 1024);
        assert_eq!(meta.format, DocumentFormat::Pdf);
        assert!(!meta.is_empty());
    }

    #[test]
    fn metadata_with_keywords() {
        let meta = DocumentMetadata::new()
            .with_keywords(vec!["a", "b", "c"]);
        assert_eq!(meta.keywords, vec!["a", "b", "c"]);
    }

    #[test]
    fn metadata_with_keyword_appends() {
        let meta = DocumentMetadata::new()
            .with_keyword("first")
            .with_keyword("second");
        assert_eq!(meta.keywords, vec!["first", "second"]);
    }

    #[test]
    fn metadata_is_empty_with_only_file_size() {
        // file_size and format don't count for is_empty
        let meta = DocumentMetadata::new()
            .with_file_size(999)
            .with_format(DocumentFormat::Html);
        assert!(meta.is_empty());
    }

    #[test]
    fn metadata_not_empty_with_title() {
        let meta = DocumentMetadata::new().with_title("T");
        assert!(!meta.is_empty());
    }

    #[test]
    fn metadata_not_empty_with_keyword() {
        let meta = DocumentMetadata::new().with_keyword("k");
        assert!(!meta.is_empty());
    }

    #[test]
    fn metadata_not_empty_with_page_count() {
        let meta = DocumentMetadata::new().with_page_count(1);
        assert!(!meta.is_empty());
    }

    // =========================================================================
    // DocumentElement::to_html
    // =========================================================================

    #[test]
    fn element_paragraph_to_html() {
        let el = DocumentElement::Paragraph(ParagraphElement {
            text: "Hello".into(),
            style: TextStyle::default(),
        });
        assert_eq!(el.to_html(), "<p>Hello</p>\n");
    }

    #[test]
    fn element_paragraph_escapes_html() {
        let el = DocumentElement::Paragraph(ParagraphElement {
            text: "<b>bold</b>".into(),
            style: TextStyle::default(),
        });
        let html = el.to_html();
        assert!(html.contains("&lt;b&gt;"));
        assert!(!html.contains("<b>bold</b>"));
    }

    #[test]
    fn element_heading_to_html() {
        let el = DocumentElement::Heading(HeadingElement {
            text: "Title".into(),
            level: 2,
        });
        assert_eq!(el.to_html(), "<h2>Title</h2>\n");
    }

    #[test]
    fn element_heading_clamps_level() {
        let el = DocumentElement::Heading(HeadingElement {
            text: "Deep".into(),
            level: 10,
        });
        assert_eq!(el.to_html(), "<h6>Deep</h6>\n");

        let el_zero = DocumentElement::Heading(HeadingElement {
            text: "Zero".into(),
            level: 0,
        });
        assert_eq!(el_zero.to_html(), "<h1>Zero</h1>\n");
    }

    #[test]
    fn element_table_to_html_no_header() {
        let el = DocumentElement::Table(TableElement {
            rows: vec![
                TableRow {
                    cells: vec![
                        TableCell { text: "A".into(), style: TextStyle::default() },
                        TableCell { text: "B".into(), style: TextStyle::default() },
                    ],
                },
            ],
            has_header: false,
        });
        let html = el.to_html();
        assert!(html.contains("<td>A</td>"));
        assert!(html.contains("<td>B</td>"));
        assert!(!html.contains("<th>"));
    }

    #[test]
    fn element_table_to_html_with_header() {
        let el = DocumentElement::Table(TableElement {
            rows: vec![
                TableRow {
                    cells: vec![
                        TableCell { text: "Col1".into(), style: TextStyle::default() },
                    ],
                },
                TableRow {
                    cells: vec![
                        TableCell { text: "Val1".into(), style: TextStyle::default() },
                    ],
                },
            ],
            has_header: true,
        });
        let html = el.to_html();
        assert!(html.contains("<th>Col1</th>"));
        assert!(html.contains("<td>Val1</td>"));
    }

    #[test]
    fn element_unordered_list_to_html() {
        let el = DocumentElement::List(ListElement {
            items: vec![
                ListItem { text: "One".into(), nested: None },
                ListItem { text: "Two".into(), nested: None },
            ],
            ordered: false,
        });
        let html = el.to_html();
        assert!(html.contains("<ul>"));
        assert!(html.contains("</ul>"));
        assert!(html.contains("<li>One</li>"));
        assert!(html.contains("<li>Two</li>"));
    }

    #[test]
    fn element_ordered_list_to_html() {
        let el = DocumentElement::List(ListElement {
            items: vec![ListItem { text: "Step".into(), nested: None }],
            ordered: true,
        });
        let html = el.to_html();
        assert!(html.contains("<ol>"));
        assert!(html.contains("</ol>"));
    }

    #[test]
    fn element_image_with_data() {
        let el = DocumentElement::Image(ImageElement {
            data_base64: Some("abc123".into()),
            mime_type: Some("image/jpeg".into()),
            width: Some(100),
            height: Some(50),
            alt_text: Some("Photo".into()),
            caption: Some("A caption".into()),
        });
        let html = el.to_html();
        assert!(html.contains("data:image/jpeg;base64,abc123"));
        assert!(html.contains("alt=\"Photo\""));
        assert!(html.contains("A caption"));
    }

    #[test]
    fn element_image_without_data() {
        let el = DocumentElement::Image(ImageElement {
            data_base64: None,
            mime_type: None,
            width: None,
            height: None,
            alt_text: None,
            caption: None,
        });
        let html = el.to_html();
        assert!(html.contains("image-container"));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn element_break_to_html() {
        let el = DocumentElement::Break;
        assert_eq!(el.to_html(), "<hr>\n");
    }

    // =========================================================================
    // DocumentFormat
    // =========================================================================

    #[test]
    fn document_format_default_is_text() {
        assert_eq!(DocumentFormat::default(), DocumentFormat::Text);
    }

    #[test]
    fn document_format_from_extension() {
        assert_eq!(DocumentFormat::from_extension("file.pdf"), Some(DocumentFormat::Pdf));
        assert_eq!(DocumentFormat::from_extension("file.docx"), Some(DocumentFormat::Docx));
        assert_eq!(DocumentFormat::from_extension("file.doc"), Some(DocumentFormat::Docx));
        assert_eq!(DocumentFormat::from_extension("file.html"), Some(DocumentFormat::Html));
        assert_eq!(DocumentFormat::from_extension("file.htm"), Some(DocumentFormat::Html));
        assert_eq!(DocumentFormat::from_extension("file.md"), Some(DocumentFormat::Markdown));
        assert_eq!(DocumentFormat::from_extension("file.markdown"), Some(DocumentFormat::Markdown));
        assert_eq!(DocumentFormat::from_extension("file.txt"), Some(DocumentFormat::Text));
        assert_eq!(DocumentFormat::from_extension("file.rtf"), Some(DocumentFormat::Rtf));
        assert_eq!(DocumentFormat::from_extension("file.xlsx"), Some(DocumentFormat::Spreadsheet));
        assert_eq!(DocumentFormat::from_extension("file.csv"), Some(DocumentFormat::Spreadsheet));
        assert_eq!(DocumentFormat::from_extension("file.ods"), Some(DocumentFormat::Spreadsheet));
        assert_eq!(DocumentFormat::from_extension("file.unknown"), None);
        assert_eq!(DocumentFormat::from_extension("noext"), None);
    }

    #[test]
    fn document_format_extension() {
        assert_eq!(DocumentFormat::Pdf.extension(), "pdf");
        assert_eq!(DocumentFormat::Docx.extension(), "docx");
        assert_eq!(DocumentFormat::Html.extension(), "html");
        assert_eq!(DocumentFormat::Markdown.extension(), "md");
        assert_eq!(DocumentFormat::Text.extension(), "txt");
        assert_eq!(DocumentFormat::Rtf.extension(), "rtf");
        assert_eq!(DocumentFormat::Spreadsheet.extension(), "xlsx");
    }

    #[test]
    fn document_format_mime_type() {
        assert_eq!(DocumentFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(DocumentFormat::Text.mime_type(), "text/plain");
        assert_eq!(DocumentFormat::Html.mime_type(), "text/html");
        assert_eq!(DocumentFormat::Markdown.mime_type(), "text/markdown");
    }

    // =========================================================================
    // TextStyle default
    // =========================================================================

    #[test]
    fn text_style_default() {
        let style = TextStyle::default();
        assert!(!style.bold);
        assert!(!style.italic);
        assert!(!style.underline);
        assert!(!style.strikethrough);
        assert!(style.font_size.is_none());
        assert!(style.font_family.is_none());
        assert!(style.color.is_none());
        assert!(style.alignment.is_none());
    }
}
