// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified Document Handling Module
//!
//! This module provides a unified interface for reading and writing various
//! document formats (PDF, DOCX, HTML, Markdown). It consolidates the previously
//! separate report generation and document viewing capabilities.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                      Document Service (Rust)                             │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
//! │  │   PDF       │  │   DOCX      │  │   HTML      │  │  Markdown   │   │
//! │  │ Read/Write  │  │ Read/Write  │  │ Read/Write  │  │ Read/Write  │   │
//! │  │  (lopdf)    │  │ (docx-rs)   │  │  (native)   │  │  (native)   │   │
//! │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘   │
//! │         │                │                │                │           │
//! │         ▼                ▼                ▼                ▼           │
//! │  ┌─────────────────────────────────────────────────────────────────┐  │
//! │  │              Unified Document Model (DocumentContent)            │  │
//! │  │  - Pages/Sections                                                │  │
//! │  │  - Text blocks                                                   │  │
//! │  │  - Tables                                                        │  │
//! │  │  - Images                                                        │  │
//! │  │  - Metadata                                                      │  │
//! │  └─────────────────────────────────────────────────────────────────┘  │
//! │         │                                                              │
//! │         ▼                                                              │
//! │  ┌─────────────────────────────────────────────────────────────────┐  │
//! │  │                     Render Outputs                               │  │
//! │  │  - HTML (for web viewer)                                        │  │
//! │  │  - Text (plain text extraction)                                 │  │
//! │  │  - PNG images (page thumbnails)                                 │  │
//! │  └─────────────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use document::{DocumentService, DocumentFormat};
//!
//! let service = DocumentService::new();
//!
//! // Read a document
//! let doc = service.read("report.pdf")?;
//! let text = doc.extract_text()?;
//! let html = doc.to_html()?;
//!
//! // Write a document (using ForensicReport data)
//! let report = ForensicReport::builder()
//!     .case_number("2026-001")
//!     .build()?;
//! service.write_report(&report, "output.pdf", DocumentFormat::Pdf)?;
//! ```

pub mod binary;
pub mod commands;
pub mod database_viewer;
pub mod docx;
pub mod email;
pub mod error;
pub mod exif;
pub mod html;
pub mod markdown;
pub mod office;
pub mod pdf;
pub mod plist_viewer;
pub mod pst;
pub mod registry_viewer;
pub mod spreadsheet;
pub mod types;
pub mod universal;

// Re-exports
pub use docx::DocxDocument;
pub use error::{DocumentError, DocumentResult};
pub use html::HtmlDocument;
pub use markdown::MarkdownDocument;
pub use pdf::PdfDocument;
pub use types::*;
pub use universal::{FileInfo, UniversalFormat, ViewerHint, ViewerType};

use std::path::Path;

/// Supported document formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentFormat {
    /// PDF document
    Pdf,
    /// Microsoft Word document (OOXML)
    Docx,
    /// HTML document
    Html,
    /// Markdown document
    Markdown,
    /// Plain text
    #[default]
    Text,
    /// Rich Text Format
    Rtf,
    /// Spreadsheet formats (XLSX, XLS, ODS, CSV)
    Spreadsheet,
    /// Microsoft PowerPoint (OOXML)
    Pptx,
    /// OpenDocument Text (ODT)
    Odt,
}

impl DocumentFormat {
    /// Detect format from file extension
    pub fn from_extension(path: impl AsRef<Path>) -> Option<Self> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())?;

        match ext.as_str() {
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "doc" => Some(Self::Docx), // Treat as DOCX, may need conversion
            "html" | "htm" => Some(Self::Html),
            "md" | "markdown" => Some(Self::Markdown),
            // Plain text & config files
            "txt" | "log" | "ini" | "cfg" | "conf" | "env" | "properties" => Some(Self::Text),
            // Script / code files (viewed as plain text)
            "bat" | "cmd" | "sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" => Some(Self::Text),
            "py" | "pyw" | "js" | "ts" | "jsx" | "tsx" | "mjs" | "cjs" => Some(Self::Text),
            "rs" | "go" | "java" | "kt" | "kts" | "cs" | "csx" => Some(Self::Text),
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" => Some(Self::Text),
            "rb" | "rake" | "php" | "phtml" | "pl" | "pm" | "lua" | "r" | "swift" => {
                Some(Self::Text)
            }
            "vb" | "vbs" | "vba" | "awk" | "sed" => Some(Self::Text),
            "css" | "scss" | "sass" | "less" => Some(Self::Text),
            "json" | "xml" | "yaml" | "yml" | "toml" | "sql" => Some(Self::Text),
            "reg" | "inf" => Some(Self::Text),
            // Rich text
            "rtf" => Some(Self::Rtf),
            // Presentations
            "pptx" => Some(Self::Pptx),
            "ppt" => Some(Self::Pptx), // Legacy PPT treated as PPTX path
            "odp" => Some(Self::Pptx), // ODP uses same text extraction
            // OpenDocument
            "odt" => Some(Self::Odt),
            // Spreadsheets
            "xlsx" | "xls" | "xlsm" | "xlsb" | "ods" | "csv" | "tsv" | "numbers" => {
                Some(Self::Spreadsheet)
            }
            _ => None,
        }
    }

    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Html => "html",
            Self::Markdown => "md",
            Self::Text => "txt",
            Self::Rtf => "rtf",
            Self::Spreadsheet => "xlsx",
            Self::Pptx => "pptx",
            Self::Odt => "odt",
        }
    }

    /// Get MIME type for format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Html => "text/html",
            Self::Markdown => "text/markdown",
            Self::Text => "text/plain",
            Self::Rtf => "application/rtf",
            Self::Spreadsheet => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            Self::Odt => "application/vnd.oasis.opendocument.text",
        }
    }
}

/// Unified document service for reading and writing documents
pub struct DocumentService {
    /// PDF handler
    pdf: PdfDocument,
    /// DOCX handler
    docx: DocxDocument,
    /// HTML handler
    html: HtmlDocument,
    /// Markdown handler
    markdown: MarkdownDocument,
    // spreadsheet handling via module functions
}

impl DocumentService {
    /// Create a new document service
    pub fn new() -> Self {
        Self {
            pdf: PdfDocument::new(),
            docx: DocxDocument::new(),
            html: HtmlDocument::new(),
            markdown: MarkdownDocument::new(),
        }
    }

    /// Read a document and extract its content
    pub fn read(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let path = path.as_ref();
        // Try extension-based format detection first.
        // If the extension is unknown, attempt to read as plain text — this handles
        // content-detected text files with non-standard extensions (e.g., .PRV, .DAT, .1ST, .TMP).
        let format = DocumentFormat::from_extension(path).unwrap_or(DocumentFormat::Text);

        match format {
            DocumentFormat::Pdf => self.pdf.read(path),
            DocumentFormat::Docx => self.docx.read(path),
            DocumentFormat::Html => self.html.read(path),
            DocumentFormat::Markdown => self.markdown.read(path),
            DocumentFormat::Spreadsheet => {
                // Convert spreadsheet to a simple DocumentContent (tables per sheet)
                use crate::viewer::document::spreadsheet as ss;
                use crate::viewer::document::types::{
                    DocumentContent, DocumentElement, DocumentPage, TableCell, TableElement,
                    TableRow,
                };

                let info = ss::read_spreadsheet_info(path)?;
                let mut content = DocumentContent::new();
                content.metadata.title =
                    path.file_name().and_then(|n| n.to_str()).map(String::from);
                content.metadata.format = DocumentFormat::Spreadsheet;

                for (i, sheet) in info.sheets.iter().enumerate() {
                    // Try to read first 200 rows; fall back to empty
                    let rows =
                        ss::read_xlsx_sheet_range(path, &sheet.name, 0, 200).unwrap_or_default();
                    let mut table = TableElement {
                        rows: Vec::new(),
                        has_header: false,
                    };
                    for (r_idx, r) in rows.iter().enumerate() {
                        let mut cells = Vec::new();
                        for c in r.iter() {
                            let text = match c {
                                ss::CellValue::Empty => String::new(),
                                ss::CellValue::String(s) => s.clone(),
                                ss::CellValue::Int(i) => i.to_string(),
                                ss::CellValue::Float(f) => f.to_string(),
                                ss::CellValue::Bool(b) => b.to_string(),
                                ss::CellValue::DateTime(s) => s.clone(),
                                ss::CellValue::Error(e) => e.clone(),
                            };
                            cells.push(TableCell {
                                text,
                                style: Default::default(),
                            });
                        }
                        table.rows.push(TableRow { cells });
                        if r_idx == 0 {
                            table.has_header = true;
                        }
                    }
                    let page = DocumentPage {
                        page_number: i + 1,
                        elements: vec![DocumentElement::Table(table)],
                    };
                    content.add_page(page);
                }

                Ok(content)
            }
            DocumentFormat::Text => self.read_text(path),
            DocumentFormat::Rtf => self.read_rtf(path),
            DocumentFormat::Pptx => self.read_pptx(path),
            DocumentFormat::Odt => self.read_odt(path),
        }
    }

    /// Read a document from bytes
    pub fn read_bytes(
        &self,
        data: &[u8],
        format: DocumentFormat,
    ) -> DocumentResult<DocumentContent> {
        match format {
            DocumentFormat::Pdf => self.pdf.read_bytes(data),
            DocumentFormat::Docx => self.docx.read_bytes(data),
            DocumentFormat::Html => self.html.read_bytes(data),
            DocumentFormat::Markdown => self.markdown.read_bytes(data),
            DocumentFormat::Text => Ok(DocumentContent::from_text(
                String::from_utf8_lossy(data).to_string(),
            )),
            DocumentFormat::Rtf => {
                let text = String::from_utf8_lossy(data).to_string();
                let plain = Self::strip_rtf(&text);
                Ok(DocumentContent::from_text(plain))
            }
            DocumentFormat::Spreadsheet => Err(DocumentError::UnsupportedFormat(
                "spreadsheet_bytes".to_string(),
            )),
            DocumentFormat::Pptx => Self::extract_pptx_from_bytes(data),
            DocumentFormat::Odt => Self::extract_odt_from_bytes(data),
        }
    }

    /// Extract plain text from a document
    pub fn extract_text(&self, path: impl AsRef<Path>) -> DocumentResult<String> {
        let content = self.read(path)?;
        Ok(content.to_plain_text())
    }

    /// Render document to HTML for web viewing
    pub fn render_html(&self, path: impl AsRef<Path>) -> DocumentResult<String> {
        let content = self.read(path)?;
        Ok(content.to_html())
    }

    /// Render document to HTML from bytes
    pub fn render_html_from_bytes(
        &self,
        data: &[u8],
        format: DocumentFormat,
    ) -> DocumentResult<String> {
        let content = self.read_bytes(data, format)?;
        Ok(content.to_html())
    }

    /// Write a forensic report to a document
    pub fn write_report(
        &self,
        report: &crate::report::ForensicReport,
        path: impl AsRef<Path>,
        format: DocumentFormat,
    ) -> DocumentResult<()> {
        match format {
            DocumentFormat::Pdf => self.pdf.write_report(report, path),
            DocumentFormat::Docx => self.docx.write_report(report, path),
            DocumentFormat::Html => self.html.write_report(report, path),
            DocumentFormat::Markdown => self.markdown.write_report(report, path),
            _ => Err(DocumentError::UnsupportedFormat(
                format.extension().to_string(),
            )),
        }
    }

    /// Get document metadata without reading full content
    pub fn get_metadata(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentMetadata> {
        let path = path.as_ref();
        let format = DocumentFormat::from_extension(path).ok_or_else(|| {
            DocumentError::UnsupportedFormat(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            )
        })?;

        match format {
            DocumentFormat::Pdf => self.pdf.get_metadata(path),
            DocumentFormat::Docx => self.docx.get_metadata(path),
            DocumentFormat::Spreadsheet => {
                let info = spreadsheet::read_spreadsheet_info(path)?;
                let file_meta = std::fs::metadata(path)?;
                Ok(DocumentMetadata {
                    title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                    author: None,
                    subject: None,
                    keywords: Vec::new(),
                    creator: None,
                    producer: None,
                    creation_date: None,
                    modification_date: None,
                    page_count: Some(info.total_sheets),
                    word_count: None,
                    file_size: file_meta.len(),
                    format,
                })
            }
            _ => {
                // For formats without rich metadata, return basic info
                let file_meta = std::fs::metadata(path)?;
                Ok(DocumentMetadata {
                    title: path.file_name().and_then(|n| n.to_str()).map(String::from),
                    author: None,
                    subject: None,
                    keywords: Vec::new(),
                    creator: None,
                    producer: None,
                    creation_date: None,
                    modification_date: None,
                    page_count: None,
                    word_count: None,
                    file_size: file_meta.len(),
                    format,
                })
            }
        }
    }

    /// Read plain text file
    /// Uses lossy UTF-8 conversion to handle files with binary content or
    /// non-UTF-8 encodings (e.g., Windows-1252 .ini files from forensic images)
    fn read_text(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        // Cap text reads at 50 MB to prevent OOM on huge log/text files
        const MAX_TEXT_SIZE: u64 = 50 * 1024 * 1024;
        let bytes = if file_size > MAX_TEXT_SIZE {
            let mut f = std::fs::File::open(&path)?;
            let mut buf = vec![0u8; MAX_TEXT_SIZE as usize];
            std::io::Read::read(&mut f, &mut buf)?;
            buf
        } else {
            std::fs::read(&path)?
        };
        let text = String::from_utf8_lossy(&bytes).to_string();
        let mut content = DocumentContent::from_text(text);
        if file_size > MAX_TEXT_SIZE {
            let size_mb = file_size as f64 / (1024.0 * 1024.0);
            content.metadata.title = Some(format!(
                "{} (showing first 50 MB of {:.1} MB)",
                path.as_ref()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file"),
                size_mb
            ));
        }
        Ok(content)
    }

    /// Read RTF file and extract plain text
    fn read_rtf(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        // Cap RTF reads at 50 MB to prevent OOM
        const MAX_RTF_SIZE: u64 = 50 * 1024 * 1024;
        if file_size > MAX_RTF_SIZE {
            return Err(DocumentError::Parse(format!(
                "RTF file too large ({:.1} MB, max 50 MB)",
                file_size as f64 / (1024.0 * 1024.0)
            )));
        }
        let bytes = std::fs::read(&path)?;
        let rtf_data = String::from_utf8_lossy(&bytes).to_string();
        let plain_text = Self::strip_rtf(&rtf_data);

        let mut content = DocumentContent::from_text(plain_text);
        content.metadata.format = DocumentFormat::Rtf;
        content.metadata.title = path
            .as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from);

        let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        content.metadata.file_size = file_size;

        Ok(content)
    }

    /// Strip RTF control words and extract plain text
    ///
    /// Handles the most common RTF constructs:
    /// - Removes control words (e.g., \par, \b, \i)  
    /// - Converts \par and \line to newlines
    /// - Handles hex escapes (\'xx)
    /// - Handles Unicode escapes (\uN)
    /// - Strips group delimiters { }
    /// - Preserves actual text content
    fn strip_rtf(rtf: &str) -> String {
        let mut result = String::new();
        let mut chars = rtf.chars().peekable();
        let mut depth: i32 = 0;
        // Track groups to skip (e.g., \fonttbl, \colortbl, \stylesheet, \info)
        let mut skip_depth: i32 = -1;

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    depth += 1;
                }
                '}' => {
                    if depth == skip_depth {
                        skip_depth = -1;
                    }
                    depth -= 1;
                }
                '\\' if skip_depth < 0 => {
                    // Control word or symbol
                    match chars.peek() {
                        Some('\'') => {
                            // Hex escape: \'xx
                            chars.next(); // consume '
                            let mut hex = String::new();
                            for _ in 0..2 {
                                if let Some(&c) = chars.peek() {
                                    if c.is_ascii_hexdigit() {
                                        hex.push(c);
                                        chars.next();
                                    }
                                }
                            }
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                result.push(byte as char);
                            }
                        }
                        Some('u') => {
                            // Unicode escape: \uN (followed by replacement char)
                            chars.next(); // consume 'u'
                            let mut num = String::new();
                            // May have a negative sign
                            if chars.peek() == Some(&'-') {
                                num.push('-');
                                chars.next();
                            }
                            while let Some(&c) = chars.peek() {
                                if c.is_ascii_digit() {
                                    num.push(c);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if let Ok(code_point) = num.parse::<i32>() {
                                let cp = if code_point < 0 {
                                    (code_point + 65536) as u32
                                } else {
                                    code_point as u32
                                };
                                if let Some(c) = char::from_u32(cp) {
                                    result.push(c);
                                }
                            }
                            // Skip replacement character (usually ?)
                            if chars.peek() == Some(&' ') {
                                chars.next();
                            }
                        }
                        Some('\\') => {
                            chars.next();
                            result.push('\\');
                        }
                        Some('{') => {
                            chars.next();
                            result.push('{');
                        }
                        Some('}') => {
                            chars.next();
                            result.push('}');
                        }
                        Some('~') => {
                            chars.next();
                            result.push('\u{00A0}'); // Non-breaking space
                        }
                        Some('-') => {
                            chars.next();
                            // Optional hyphen — skip
                        }
                        Some('_') => {
                            chars.next();
                            result.push('-'); // Non-breaking hyphen
                        }
                        Some('\n') | Some('\r') => {
                            chars.next();
                            result.push('\n');
                        }
                        _ => {
                            // Read control word name
                            let mut word = String::new();
                            while let Some(&c) = chars.peek() {
                                if c.is_ascii_alphabetic() {
                                    word.push(c);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            // Read optional numeric parameter
                            let mut _param = String::new();
                            if chars.peek() == Some(&'-') {
                                _param.push('-');
                                chars.next();
                            }
                            while let Some(&c) = chars.peek() {
                                if c.is_ascii_digit() {
                                    _param.push(c);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            // Consume trailing space delimiter
                            if chars.peek() == Some(&' ') {
                                chars.next();
                            }

                            // Handle specific control words
                            match word.as_str() {
                                "par" | "line" => result.push('\n'),
                                "tab" => result.push('\t'),
                                // Skip destination groups (metadata tables)
                                "fonttbl" | "colortbl" | "stylesheet" | "info" | "header"
                                | "footer" | "headerl" | "headerr" | "footerl" | "footerr"
                                | "pict" | "object" | "fldinst" => {
                                    skip_depth = depth;
                                }
                                _ => {
                                    // Ignore other control words
                                }
                            }
                        }
                    }
                }
                _ => {
                    if skip_depth < 0 && depth > 0 {
                        // Regular text character
                        if ch != '\r' && ch != '\n' {
                            result.push(ch);
                        }
                    }
                }
            }
        }

        // Clean up: collapse multiple blank lines, trim
        let cleaned: Vec<&str> = result.lines().collect();
        let mut final_text = String::new();
        let mut blank_count = 0;
        for line in cleaned {
            if line.trim().is_empty() {
                blank_count += 1;
                if blank_count <= 2 {
                    final_text.push('\n');
                }
            } else {
                blank_count = 0;
                final_text.push_str(line);
                final_text.push('\n');
            }
        }

        final_text.trim().to_string()
    }

    /// Read PPTX (or legacy PPT/ODP) file and extract slide text
    ///
    /// PPTX files are ZIP archives containing XML slides in `ppt/slides/slideN.xml`.
    /// ODP files use `content.xml` with OpenDocument namespaces.
    /// Legacy PPT files are OLE compound documents — we extract text heuristically.
    fn read_pptx(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let data = std::fs::read(&path)?;

        // Check for OLE compound document (legacy .ppt)
        if data.len() >= 4 && data[..4] == [0xD0, 0xCF, 0x11, 0xE0] {
            return self.read_legacy_presentation(&data, path.as_ref());
        }

        Self::extract_pptx_from_bytes(&data)
    }

    /// Extract text from PPTX/ODP bytes (ZIP-based)
    fn extract_pptx_from_bytes(data: &[u8]) -> DocumentResult<DocumentContent> {
        use std::io::Cursor;
        use zip::ZipArchive;

        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor).map_err(|e| {
            DocumentError::InvalidDocument(format!("Not a valid OOXML/ODP archive: {}", e))
        })?;

        let mut content = DocumentContent::new();
        content.metadata.format = DocumentFormat::Pptx;
        content.metadata.file_size = data.len() as u64;

        // Check if this is ODP (content.xml) vs PPTX (ppt/slides/*)
        let is_odp = archive.by_name("content.xml").is_ok();

        if is_odp {
            // OpenDocument Presentation — text is in content.xml
            let text = Self::extract_odp_text(&mut archive)?;
            // Split into separate paragraph elements on blank lines
            let elements: Vec<DocumentElement> = text
                .split("\n\n")
                .map(|chunk| chunk.trim())
                .filter(|chunk| !chunk.is_empty())
                .map(|chunk| {
                    DocumentElement::Paragraph(ParagraphElement {
                        text: chunk.to_string(),
                        style: TextStyle::default(),
                    })
                })
                .collect();
            if !elements.is_empty() {
                let page = DocumentPage {
                    page_number: 1,
                    elements,
                };
                content.add_page(page);
            }
        } else {
            // OOXML PPTX — iterate ppt/slides/slide1.xml, slide2.xml, etc.
            let mut slide_names: Vec<String> = archive
                .file_names()
                .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
                .map(String::from)
                .collect();
            slide_names.sort_by(|a, b| {
                let num_a = Self::extract_slide_number(a);
                let num_b = Self::extract_slide_number(b);
                num_a.cmp(&num_b)
            });

            for (i, slide_name) in slide_names.iter().enumerate() {
                if let Ok(text) = Self::extract_pptx_slide_text(&mut archive, slide_name) {
                    if !text.trim().is_empty() {
                        // Split slide text into separate paragraph elements
                        let elements: Vec<DocumentElement> = text
                            .split("\n\n")
                            .map(|chunk| chunk.trim())
                            .filter(|chunk| !chunk.is_empty())
                            .map(|chunk| {
                                DocumentElement::Paragraph(ParagraphElement {
                                    text: chunk.to_string(),
                                    style: TextStyle::default(),
                                })
                            })
                            .collect();
                        if !elements.is_empty() {
                            let page = DocumentPage {
                                page_number: i + 1,
                                elements,
                            };
                            content.add_page(page);
                        }
                    }
                }
            }
        }

        content.metadata.page_count = Some(content.pages.len());
        Ok(content)
    }

    /// Extract slide number from path like "ppt/slides/slide3.xml" → 3
    fn extract_slide_number(path: &str) -> usize {
        path.trim_start_matches("ppt/slides/slide")
            .trim_end_matches(".xml")
            .parse::<usize>()
            .unwrap_or(0)
    }

    /// Extract text from a single PPTX slide XML
    fn extract_pptx_slide_text(
        archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
        name: &str,
    ) -> DocumentResult<String> {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        use std::io::Read;

        let mut file = archive
            .by_name(name)
            .map_err(|e| DocumentError::InvalidDocument(format!("Cannot read {}: {}", name, e)))?;
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;

        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);

        let mut text_parts: Vec<String> = Vec::new();
        let mut in_text = false;
        let mut current_text = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let local = e.local_name();
                    let name_bytes = local.as_ref();
                    // a:t = text run in DrawingML
                    if name_bytes == b"t" {
                        in_text = true;
                        current_text.clear();
                    }
                }
                Ok(Event::Text(e)) if in_text => {
                    if let Ok(t) = e.unescape() {
                        current_text.push_str(&t);
                    }
                }
                Ok(Event::End(e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"t" && in_text {
                        in_text = false;
                        if !current_text.trim().is_empty() {
                            text_parts.push(current_text.clone());
                        }
                    }
                    // a:p = paragraph boundary
                    if local.as_ref() == b"p" && !text_parts.is_empty() {
                        // Add a newline between paragraphs
                        text_parts.push(String::new());
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
        }

        Ok(text_parts.join("\n").trim().to_string())
    }

    /// Extract text from ODP content.xml
    fn extract_odp_text(
        archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    ) -> DocumentResult<String> {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        use std::io::Read;

        let mut file = archive.by_name("content.xml").map_err(|e| {
            DocumentError::InvalidDocument(format!("Cannot read content.xml: {}", e))
        })?;
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;

        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);

        let mut text_parts: Vec<String> = Vec::new();
        let mut in_text = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let local = e.local_name();
                    let name_bytes = local.as_ref();
                    // text:p, text:h, text:span contain text in ODF
                    if name_bytes == b"p" || name_bytes == b"h" || name_bytes == b"span" {
                        in_text = true;
                    }
                }
                Ok(Event::Text(e)) if in_text => {
                    if let Ok(t) = e.unescape() {
                        let trimmed = t.trim();
                        if !trimmed.is_empty() {
                            text_parts.push(trimmed.to_string());
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let local = e.local_name();
                    let name_bytes = local.as_ref();
                    if name_bytes == b"p" || name_bytes == b"h" {
                        in_text = false;
                        text_parts.push(String::new()); // paragraph break
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
        }

        Ok(text_parts.join("\n").trim().to_string())
    }

    /// Read legacy presentation (OLE compound document like .ppt)
    fn read_legacy_presentation(
        &self,
        data: &[u8],
        path: &Path,
    ) -> DocumentResult<DocumentContent> {
        // Legacy .ppt is OLE compound — extract text heuristically like legacy .doc
        let mut content = DocumentContent::new();
        content.metadata.format = DocumentFormat::Pptx;
        content.metadata.file_size = data.len() as u64;
        content.metadata.title = path.file_name().and_then(|n| n.to_str()).map(String::from);

        // Reuse the OLE text extraction from DocxDocument
        let text = self.docx.extract_text_from_ole(data);

        if text.trim().is_empty() {
            let page = DocumentPage {
                page_number: 1,
                elements: vec![DocumentElement::Paragraph(ParagraphElement {
                    text: "[Legacy .ppt format — no text could be extracted]".to_string(),
                    style: TextStyle::default(),
                })],
            };
            content.add_page(page);
        } else {
            // Split extracted text into separate paragraph elements
            let elements: Vec<DocumentElement> = text
                .split("\n\n")
                .map(|chunk| chunk.trim())
                .filter(|chunk| !chunk.is_empty())
                .map(|chunk| {
                    DocumentElement::Paragraph(ParagraphElement {
                        text: chunk.to_string(),
                        style: TextStyle::default(),
                    })
                })
                .collect();
            if !elements.is_empty() {
                let page = DocumentPage {
                    page_number: 1,
                    elements,
                };
                content.add_page(page);
            }
        }

        Ok(content)
    }

    /// Read ODT (OpenDocument Text) file and extract content
    fn read_odt(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let data = std::fs::read(&path)?;
        Self::extract_odt_from_bytes(&data)
    }

    /// Extract text from ODT bytes
    fn extract_odt_from_bytes(data: &[u8]) -> DocumentResult<DocumentContent> {
        use std::io::Cursor;
        use zip::ZipArchive;

        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor).map_err(|e| {
            DocumentError::InvalidDocument(format!("Not a valid ODT archive: {}", e))
        })?;

        let mut content = DocumentContent::new();
        content.metadata.format = DocumentFormat::Odt;
        content.metadata.file_size = data.len() as u64;

        // ODT stores text in content.xml using ODF namespaces
        let text = Self::extract_odt_content_text(&mut archive)?;

        if !text.trim().is_empty() {
            let word_count = text.split_whitespace().count();
            content.metadata.word_count = Some(word_count);

            // Split into separate paragraph elements on blank lines
            let elements: Vec<DocumentElement> = text
                .split("\n\n")
                .map(|chunk| chunk.trim())
                .filter(|chunk| !chunk.is_empty())
                .map(|chunk| {
                    DocumentElement::Paragraph(ParagraphElement {
                        text: chunk.to_string(),
                        style: TextStyle::default(),
                    })
                })
                .collect();
            if !elements.is_empty() {
                let page = DocumentPage {
                    page_number: 1,
                    elements,
                };
                content.add_page(page);
            }
        }

        Ok(content)
    }

    /// Extract text from ODT content.xml (same as ODP but for text documents)
    fn extract_odt_content_text(
        archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    ) -> DocumentResult<String> {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        use std::io::Read;

        let mut file = archive.by_name("content.xml").map_err(|e| {
            DocumentError::InvalidDocument(format!("Cannot read content.xml: {}", e))
        })?;
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;

        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);

        let mut text_parts: Vec<String> = Vec::new();
        let mut in_body = false;
        let mut depth: usize = 0;

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let local = e.local_name();
                    let name_bytes = local.as_ref();
                    if name_bytes == b"body" {
                        in_body = true;
                    }
                    if in_body {
                        depth += 1;
                    }
                }
                Ok(Event::Text(e)) if in_body => {
                    if let Ok(t) = e.unescape() {
                        let trimmed = t.trim();
                        if !trimmed.is_empty() {
                            text_parts.push(trimmed.to_string());
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let local = e.local_name();
                    let name_bytes = local.as_ref();
                    if in_body {
                        if name_bytes == b"p" || name_bytes == b"h" {
                            text_parts.push(String::new());
                        }
                        depth = depth.saturating_sub(1);
                        if name_bytes == b"body" {
                            in_body = false;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
        }

        Ok(text_parts.join("\n").trim().to_string())
    }

    // =========================================================================
    // STATIC METHODS (for convenience)
    // =========================================================================

    /// Detect document format from file path (static)
    pub fn detect_format(path: impl AsRef<Path>) -> Option<DocumentFormat> {
        DocumentFormat::from_extension(path)
    }

    /// Check if a file is a supported document format (static)
    pub fn is_supported(path: impl AsRef<Path>) -> bool {
        DocumentFormat::from_extension(path).is_some()
    }

    /// Get list of supported extensions (static)
    pub fn supported_extensions() -> &'static [&'static str] {
        &[
            // Documents
            "pdf",
            "docx",
            "doc",
            "html",
            "htm",
            "md",
            "markdown",
            "rtf",
            // Presentations
            "pptx",
            "ppt",
            "odp",
            // OpenDocument
            "odt",
            // Plain text & config
            "txt",
            "log",
            "ini",
            "cfg",
            "conf",
            "env",
            "properties",
            // Scripts & code
            "bat",
            "cmd",
            "sh",
            "bash",
            "zsh",
            "fish",
            "ps1",
            "psm1",
            "py",
            "pyw",
            "js",
            "ts",
            "jsx",
            "tsx",
            "mjs",
            "cjs",
            "rs",
            "go",
            "java",
            "kt",
            "kts",
            "cs",
            "csx",
            "c",
            "cpp",
            "cc",
            "cxx",
            "h",
            "hpp",
            "hxx",
            "rb",
            "rake",
            "php",
            "phtml",
            "pl",
            "pm",
            "lua",
            "r",
            "swift",
            "vb",
            "vbs",
            "vba",
            "awk",
            "sed",
            "css",
            "scss",
            "sass",
            "less",
            "json",
            "xml",
            "yaml",
            "yml",
            "toml",
            "sql",
            "reg",
            "inf",
            // Spreadsheets
            "xlsx",
            "xls",
            "xlsm",
            "xlsb",
            "ods",
            "csv",
            "tsv",
            "numbers",
        ]
    }
}

impl Default for DocumentService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_rtf_basic() {
        let rtf = r"{\rtf1\ansi Hello World}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(result.contains("Hello World"), "Got: {}", result);
    }

    #[test]
    fn test_strip_rtf_par_newlines() {
        let rtf = r"{\rtf1 Line one\par Line two\par Line three}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(result.contains("Line one"), "Got: {}", result);
        assert!(result.contains("Line two"), "Got: {}", result);
        assert!(result.contains("Line three"), "Got: {}", result);
        // \par should produce newlines
        let lines: Vec<&str> = result.lines().collect();
        assert!(
            lines.len() >= 3,
            "Expected 3+ lines, got {}: {:?}",
            lines.len(),
            lines
        );
    }

    #[test]
    fn test_strip_rtf_fonttbl_skipped() {
        let rtf = r"{\rtf1{\fonttbl{\f0 Arial;}{\f1 Courier;}}Actual text here}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(!result.contains("Arial"), "Font table should be stripped");
        assert!(!result.contains("Courier"), "Font table should be stripped");
        assert!(result.contains("Actual text here"), "Got: {}", result);
    }

    #[test]
    fn test_strip_rtf_colortbl_skipped() {
        let rtf = r"{\rtf1{\colortbl;\red0\green0\blue0;}Content after color table}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(!result.contains("red0"), "Color table should be stripped");
        assert!(
            result.contains("Content after color table"),
            "Got: {}",
            result
        );
    }

    #[test]
    fn test_strip_rtf_hex_escapes() {
        // \'e9 = é (Latin small letter e with acute)
        let rtf = r"{\rtf1 caf\'e9}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(result.contains("caf"), "Got: {}", result);
    }

    #[test]
    fn test_strip_rtf_unicode_escapes() {
        // \u8212 = — (em dash)
        let rtf = r"{\rtf1 hello\u8212?world}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(result.contains("hello"), "Got: {}", result);
        assert!(result.contains("world"), "Got: {}", result);
    }

    #[test]
    fn test_strip_rtf_tab() {
        let rtf = r"{\rtf1 col1\tab col2\tab col3}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(result.contains("col1"), "Got: {}", result);
        assert!(
            result.contains('\t'),
            "Expected tab characters, got: {:?}",
            result
        );
    }

    #[test]
    fn test_strip_rtf_escaped_braces() {
        let rtf = r"{\rtf1 open \{ and close \} braces}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(result.contains('{'), "Got: {}", result);
        assert!(result.contains('}'), "Got: {}", result);
    }

    #[test]
    fn test_strip_rtf_empty_input() {
        let result = DocumentService::strip_rtf("");
        assert!(result.is_empty(), "Got: {}", result);
    }

    #[test]
    fn test_strip_rtf_plain_text_only() {
        let rtf = r"{\rtf1 Just plain text with no formatting}";
        let result = DocumentService::strip_rtf(rtf);
        assert_eq!(result, "Just plain text with no formatting");
    }

    #[test]
    fn test_strip_rtf_info_group_skipped() {
        let rtf = r"{\rtf1{\info{\title My Secret Doc}{\author John}}Visible content}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(
            !result.contains("My Secret Doc"),
            "Info group should be stripped"
        );
        assert!(!result.contains("John"), "Info group should be stripped");
        assert!(result.contains("Visible content"), "Got: {}", result);
    }

    #[test]
    fn test_strip_rtf_non_breaking_space() {
        let rtf = r"{\rtf1 word1\~word2}";
        let result = DocumentService::strip_rtf(rtf);
        assert!(result.contains("word1"), "Got: {}", result);
        assert!(result.contains("word2"), "Got: {}", result);
    }

    #[test]
    fn test_document_format_rtf() {
        let format = DocumentFormat::from_extension("document.rtf");
        assert_eq!(format, Some(DocumentFormat::Rtf));
    }

    #[test]
    fn test_document_format_pptx() {
        assert_eq!(
            DocumentFormat::from_extension("slides.pptx"),
            Some(DocumentFormat::Pptx)
        );
    }

    #[test]
    fn test_document_format_ppt_legacy() {
        assert_eq!(
            DocumentFormat::from_extension("slides.ppt"),
            Some(DocumentFormat::Pptx)
        );
    }

    #[test]
    fn test_document_format_odp() {
        assert_eq!(
            DocumentFormat::from_extension("slides.odp"),
            Some(DocumentFormat::Pptx)
        );
    }

    #[test]
    fn test_document_format_odt() {
        assert_eq!(
            DocumentFormat::from_extension("document.odt"),
            Some(DocumentFormat::Odt)
        );
    }

    #[test]
    fn test_pptx_extension() {
        assert_eq!(DocumentFormat::Pptx.extension(), "pptx");
    }

    #[test]
    fn test_odt_extension() {
        assert_eq!(DocumentFormat::Odt.extension(), "odt");
    }

    #[test]
    fn test_pptx_mime_type() {
        assert!(DocumentFormat::Pptx.mime_type().contains("presentation"));
    }

    #[test]
    fn test_odt_mime_type() {
        assert!(DocumentFormat::Odt.mime_type().contains("opendocument"));
    }

    #[test]
    fn test_extract_slide_number() {
        assert_eq!(
            DocumentService::extract_slide_number("ppt/slides/slide1.xml"),
            1
        );
        assert_eq!(
            DocumentService::extract_slide_number("ppt/slides/slide12.xml"),
            12
        );
        assert_eq!(
            DocumentService::extract_slide_number("ppt/slides/slide3.xml"),
            3
        );
    }

    #[test]
    fn test_supported_extensions_include_new_formats() {
        let exts = DocumentService::supported_extensions();
        assert!(exts.contains(&"pptx"), "Missing pptx");
        assert!(exts.contains(&"ppt"), "Missing ppt");
        assert!(exts.contains(&"odp"), "Missing odp");
        assert!(exts.contains(&"odt"), "Missing odt");
        assert!(exts.contains(&"rtf"), "Missing rtf");
    }
}
