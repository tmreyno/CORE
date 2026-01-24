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
                let level = h.level.min(6).max(1);
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
