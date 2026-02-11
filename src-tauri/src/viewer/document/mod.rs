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

pub mod types;
pub mod pdf;
pub mod docx;
pub mod html;
pub mod markdown;
pub mod error;
pub mod commands;
pub mod universal;
pub mod exif;
pub mod spreadsheet;
pub mod email;
pub mod binary;
pub mod plist_viewer;
pub mod registry_viewer;
pub mod database_viewer;

// Re-exports
pub use types::*;
pub use pdf::PdfDocument;
pub use docx::DocxDocument;
pub use html::HtmlDocument;
pub use markdown::MarkdownDocument;
pub use error::{DocumentError, DocumentResult};
pub use universal::{UniversalFormat, ViewerType, FileInfo, ViewerHint};

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
}

impl DocumentFormat {
    /// Detect format from file extension
    pub fn from_extension(path: impl AsRef<Path>) -> Option<Self> {
        let ext = path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())?;
        
        match ext.as_str() {
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "doc" => Some(Self::Docx), // Treat as DOCX, may need conversion
            "html" | "htm" => Some(Self::Html),
            "md" | "markdown" => Some(Self::Markdown),
            "txt" => Some(Self::Text),
            "rtf" => Some(Self::Rtf),
            "xlsx" | "xls" | "xlsm" | "xlsb" | "ods" | "csv" => Some(Self::Spreadsheet),
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
            Self::Spreadsheet => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
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
        let format = DocumentFormat::from_extension(path)
            .ok_or_else(|| DocumentError::UnsupportedFormat(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            ))?;

        match format {
            DocumentFormat::Pdf => self.pdf.read(path),
            DocumentFormat::Docx => self.docx.read(path),
            DocumentFormat::Html => self.html.read(path),
            DocumentFormat::Markdown => self.markdown.read(path),
            DocumentFormat::Spreadsheet => {
                // Convert spreadsheet to a simple DocumentContent (tables per sheet)
                use crate::viewer::document::spreadsheet as ss;
                use crate::viewer::document::types::{DocumentContent, DocumentPage, DocumentElement, TableElement, TableRow, TableCell};

                let info = ss::read_spreadsheet_info(path)?;
                let mut content = DocumentContent::new();
                content.metadata.title = path.file_name().and_then(|n| n.to_str()).map(String::from);
                content.metadata.format = DocumentFormat::Spreadsheet;

                for (i, sheet) in info.sheets.iter().enumerate() {
                    // Try to read first 200 rows; fall back to empty
                    let rows = ss::read_xlsx_sheet_range(path, &sheet.name, 0, 200).unwrap_or_default();
                    let mut table = TableElement { rows: Vec::new(), has_header: false };
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
                            cells.push(TableCell { text, style: Default::default() });
                        }
                        table.rows.push(TableRow { cells });
                        if r_idx == 0 { table.has_header = true; }
                    }
                    let page = DocumentPage { page_number: i + 1, elements: vec![DocumentElement::Table(table)] };
                    content.add_page(page);
                }

                Ok(content)
            }
            DocumentFormat::Text => self.read_text(path),
            DocumentFormat::Rtf => Err(DocumentError::UnsupportedFormat("rtf".to_string())),
        }
    }

    /// Read a document from bytes
    pub fn read_bytes(&self, data: &[u8], format: DocumentFormat) -> DocumentResult<DocumentContent> {
        match format {
            DocumentFormat::Pdf => self.pdf.read_bytes(data),
            DocumentFormat::Docx => self.docx.read_bytes(data),
            DocumentFormat::Html => self.html.read_bytes(data),
            DocumentFormat::Markdown => self.markdown.read_bytes(data),
            DocumentFormat::Text => Ok(DocumentContent::from_text(
                String::from_utf8_lossy(data).to_string()
            )),
            DocumentFormat::Rtf => Err(DocumentError::UnsupportedFormat("rtf".to_string())),
            DocumentFormat::Spreadsheet => Err(DocumentError::UnsupportedFormat("spreadsheet_bytes".to_string())),
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
    pub fn render_html_from_bytes(&self, data: &[u8], format: DocumentFormat) -> DocumentResult<String> {
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
            _ => Err(DocumentError::UnsupportedFormat(format.extension().to_string())),
        }
    }

    /// Get document metadata without reading full content
    pub fn get_metadata(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentMetadata> {
        let path = path.as_ref();
        let format = DocumentFormat::from_extension(path)
            .ok_or_else(|| DocumentError::UnsupportedFormat(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            ))?;

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
                    title: path.file_name()
                        .and_then(|n| n.to_str())
                        .map(String::from),
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
    fn read_text(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let text = std::fs::read_to_string(path)?;
        Ok(DocumentContent::from_text(text))
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
        &["pdf", "docx", "doc", "html", "htm", "md", "markdown", "txt", "rtf", "xlsx", "xls", "xlsm", "xlsb", "ods", "csv"]
    }
}

impl Default for DocumentService {
    fn default() -> Self {
        Self::new()
    }
}
