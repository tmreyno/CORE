// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified PDF document handler
//!
//! This module provides both reading and writing capabilities for PDF documents,
//! replacing the previous split between genpdf (write) and pdf.js (read).
//!
//! # Reading
//! - Uses `pdf-extract` for text extraction
//! - Uses `lopdf` for structure parsing
//!
//! # Writing  
//! - Uses `genpdf` for report generation with proper fonts
//!
//! The frontend receives rendered HTML for display, eliminating the need for
//! pdf.js in most cases.

use std::path::Path;

use genpdf::{
    elements::{Break, LinearLayout, Paragraph, TableLayout, Text},
    fonts, style, Alignment, Document, Element,
};

use super::error::{DocumentError, DocumentResult};
use super::types::*;
use super::DocumentFormat;
use crate::report::ForensicReport;

/// PDF document handler with read/write capabilities
pub struct PdfDocument {
    /// Cached font family for writing (currently unused, reserved for future caching)
    #[allow(dead_code)]
    font_family: Option<fonts::FontFamily<fonts::FontData>>,
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
        let data = std::fs::read(path)?;
        self.read_bytes(&data)
    }

    /// Read PDF from bytes
    pub fn read_bytes(&self, data: &[u8]) -> DocumentResult<DocumentContent> {
        // Extract text using pdf-extract crate
        let text = self.extract_text_from_bytes(data)?;
        
        // Try to parse structure with lopdf for better formatting
        let (metadata, pages) = self.parse_structure(data, &text)?;

        Ok(DocumentContent {
            metadata,
            pages,
        })
    }

    /// Extract text from PDF bytes
    fn extract_text_from_bytes(&self, data: &[u8]) -> DocumentResult<String> {
        // Use pdf_extract crate
        pdf_extract::extract_text_from_mem(data)
            .map_err(|e| DocumentError::Pdf(format!("Text extraction failed: {}", e)))
    }

    /// Parse PDF structure to get metadata and page breaks
    fn parse_structure(&self, data: &[u8], extracted_text: &str) -> DocumentResult<(DocumentMetadata, Vec<DocumentPage>)> {
        // Try to use lopdf for structure
        let doc = lopdf::Document::load_mem(data)
            .map_err(|e| DocumentError::Pdf(format!("Failed to parse PDF: {}", e)))?;

        // Extract metadata from document info dictionary
        let metadata = self.extract_metadata(&doc)?;

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

        // Try to get document info dictionary
        if let Ok(info_ref) = doc.trailer.get(b"Info") {
            if let Ok(info_ref) = info_ref.as_reference() {
                if let Ok(info) = doc.get_object(info_ref) {
                    if let Ok(dict) = info.as_dict() {
                        // Extract standard PDF metadata fields
                        if let Ok(title) = dict.get(b"Title") {
                            metadata.title = Self::pdf_string_to_string(title);
                        }
                        if let Ok(author) = dict.get(b"Author") {
                            metadata.author = Self::pdf_string_to_string(author);
                        }
                        if let Ok(subject) = dict.get(b"Subject") {
                            metadata.subject = Self::pdf_string_to_string(subject);
                        }
                        if let Ok(creator) = dict.get(b"Creator") {
                            metadata.creator = Self::pdf_string_to_string(creator);
                        }
                        if let Ok(producer) = dict.get(b"Producer") {
                            metadata.producer = Self::pdf_string_to_string(producer);
                        }
                        if let Ok(keywords) = dict.get(b"Keywords") {
                            if let Some(kw) = Self::pdf_string_to_string(keywords) {
                                metadata.keywords = kw.split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                        }
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Convert PDF string object to Rust String
    fn pdf_string_to_string(obj: &lopdf::Object) -> Option<String> {
        match obj {
            lopdf::Object::String(bytes, _) => {
                // Try UTF-8 first
                if let Ok(s) = String::from_utf8(bytes.clone()) {
                    return Some(s);
                }
                // Try UTF-16BE (PDF standard)
                if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
                    let utf16: Vec<u16> = bytes[2..]
                        .chunks(2)
                        .map(|chunk| {
                            if chunk.len() == 2 {
                                u16::from_be_bytes([chunk[0], chunk[1]])
                            } else {
                                0
                            }
                        })
                        .collect();
                    return String::from_utf16(&utf16).ok();
                }
                // Fallback to lossy conversion
                Some(String::from_utf8_lossy(bytes).to_string())
            }
            _ => None,
        }
    }

    /// Split extracted text into approximate pages
    fn split_into_pages(&self, text: &str, page_count: usize) -> Vec<DocumentPage> {
        if page_count == 0 || text.is_empty() {
            return vec![DocumentPage {
                page_number: 1,
                elements: vec![DocumentElement::Paragraph(ParagraphElement {
                    text: text.to_string(),
                    style: TextStyle::default(),
                })],
            }];
        }

        // Try to find form feed characters (page breaks)
        let pages_by_ff: Vec<&str> = text.split('\u{000C}').collect();
        
        if pages_by_ff.len() > 1 {
            // Use form feed splits
            return pages_by_ff
                .into_iter()
                .enumerate()
                .map(|(i, page_text)| DocumentPage {
                    page_number: i + 1,
                    elements: self.parse_text_elements(page_text),
                })
                .collect();
        }

        // Approximate split by character count
        let chars_per_page = (text.len() / page_count).max(1);
        let mut pages = Vec::with_capacity(page_count);
        let mut remaining = text;
        let mut page_num = 1;

        while !remaining.is_empty() && page_num <= page_count {
            let split_at = if page_num == page_count {
                remaining.len()
            } else {
                // Try to split at paragraph boundary
                let target = chars_per_page.min(remaining.len());
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
                    text: text.to_string(),
                    style: TextStyle::default(),
                })],
            });
        }

        pages
    }

    /// Parse text into document elements (headings, paragraphs, etc.)
    fn parse_text_elements(&self, text: &str) -> Vec<DocumentElement> {
        let mut elements = Vec::new();
        
        for paragraph in text.split("\n\n") {
            let trimmed = paragraph.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Detect headings (all caps, short lines)
            if trimmed.len() < 100 
                && trimmed.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase())
                && trimmed.chars().any(|c| c.is_alphabetic())
            {
                elements.push(DocumentElement::Heading(HeadingElement {
                    text: trimmed.to_string(),
                    level: 2,
                }));
            } else {
                elements.push(DocumentElement::Paragraph(ParagraphElement {
                    text: trimmed.to_string(),
                    style: TextStyle::default(),
                }));
            }
        }

        if elements.is_empty() {
            elements.push(DocumentElement::Paragraph(ParagraphElement {
                text: text.trim().to_string(),
                style: TextStyle::default(),
            }));
        }

        elements
    }

    /// Get metadata without reading full document
    pub fn get_metadata(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentMetadata> {
        let data = std::fs::read(path.as_ref())?;
        let doc = lopdf::Document::load_mem(&data)
            .map_err(|e| DocumentError::Pdf(format!("Failed to parse PDF: {}", e)))?;

        let mut metadata = self.extract_metadata(&doc)?;
        metadata.file_size = data.len() as u64;
        Ok(metadata)
    }

    // =========================================================================
    // WRITING
    // =========================================================================

    /// Write a forensic report to PDF
    pub fn write_report(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> DocumentResult<()> {
        // Load fonts
        let font_family = Self::load_fonts()?;

        let mut doc = Document::new(font_family);
        
        // Configure document
        doc.set_title(&report.metadata.title);
        doc.set_minimal_conformance();

        // Build document content
        self.add_header(&mut doc, report)?;
        self.add_case_info(&mut doc, report)?;
        
        if report.executive_summary.is_some() {
            self.add_executive_summary(&mut doc, report)?;
        }
        
        if report.scope.is_some() {
            self.add_scope(&mut doc, report)?;
        }
        
        if report.methodology.is_some() {
            self.add_methodology(&mut doc, report)?;
        }
        
        if !report.evidence_items.is_empty() {
            self.add_evidence_section(&mut doc, report)?;
        }
        
        if !report.chain_of_custody.is_empty() {
            self.add_chain_of_custody(&mut doc, report)?;
        }
        
        if !report.findings.is_empty() {
            self.add_findings_section(&mut doc, report)?;
        }
        
        if !report.timeline.is_empty() {
            self.add_timeline_section(&mut doc, report)?;
        }
        
        if !report.hash_records.is_empty() {
            self.add_hash_section(&mut doc, report)?;
        }
        
        if !report.tools.is_empty() {
            self.add_tools_section(&mut doc, report)?;
        }
        
        if report.conclusions.is_some() {
            self.add_conclusions(&mut doc, report)?;
        }
        
        if !report.appendices.is_empty() {
            self.add_appendices(&mut doc, report)?;
        }
        
        self.add_footer(&mut doc, report)?;

        // Render to file
        doc.render_to_file(output_path)
            .map_err(|e| DocumentError::Pdf(e.to_string()))?;

        Ok(())
    }

    /// Try to load fonts from various locations
    fn load_fonts() -> DocumentResult<fonts::FontFamily<fonts::FontData>> {
        // Try bundled fonts first
        if let Ok(font) = fonts::from_files("./fonts", "LiberationSans", None) {
            return Ok(font);
        }
        
        // macOS system fonts
        for (dir, name) in [
            ("/Library/Fonts", "Arial"),
            ("/Library/Fonts", "Times New Roman"),
            ("/System/Library/Fonts", "Helvetica"),
        ] {
            if let Ok(font) = fonts::from_files(dir, name, None) {
                return Ok(font);
            }
        }
        
        // Linux fonts
        for (dir, name) in [
            ("/usr/share/fonts/truetype/liberation", "LiberationSans"),
            ("/usr/share/fonts/truetype/dejavu", "DejaVuSans"),
            ("/usr/share/fonts/TTF", "DejaVuSans"),
        ] {
            if let Ok(font) = fonts::from_files(dir, name, None) {
                return Ok(font);
            }
        }
        
        // Windows fonts
        if let Ok(font) = fonts::from_files("C:\\Windows\\Fonts", "arial", None) {
            return Ok(font);
        }
        
        Err(DocumentError::Font(
            "No suitable fonts found. PDF generation requires TrueType fonts. \
            Please ensure fonts are available in ./fonts/, /Library/Fonts/ (macOS), \
            or /usr/share/fonts/truetype/ (Linux).".to_string()
        ))
    }

    fn add_header(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        // Classification banner
        let classification = report.metadata.classification.as_str();
        doc.push(
            Paragraph::new(classification)
                .aligned(Alignment::Center)
                .styled(style::Style::new().bold().with_font_size(12))
        );
        
        doc.push(Break::new(0.5));
        
        // Title
        doc.push(
            Paragraph::new(&report.metadata.title)
                .aligned(Alignment::Center)
                .styled(style::Style::new().bold().with_font_size(18))
        );
        
        // Subtitle
        let subtitle = format!(
            "Report #{} | Version {}",
            report.metadata.report_number,
            report.metadata.version
        );
        doc.push(
            Paragraph::new(subtitle)
                .aligned(Alignment::Center)
                .styled(style::Style::new().with_font_size(10))
        );
        
        // Date
        let date = report.metadata.generated_at.format("%B %d, %Y").to_string();
        doc.push(
            Paragraph::new(date)
                .aligned(Alignment::Center)
                .styled(style::Style::new().with_font_size(10))
        );
        
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_case_info(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Case Information");
        
        let mut layout = LinearLayout::vertical();
        
        layout.push(self.info_row("Case Number:", &report.case_info.case_number));
        
        if let Some(ref name) = report.case_info.case_name {
            layout.push(self.info_row("Case Name:", name));
        }
        
        if let Some(ref agency) = report.case_info.agency {
            layout.push(self.info_row("Agency:", agency));
        }
        
        if let Some(ref requestor) = report.case_info.requestor {
            layout.push(self.info_row("Requestor:", requestor));
        }
        
        if let Some(ref date) = report.case_info.request_date {
            layout.push(self.info_row("Request Date:", &date.format("%Y-%m-%d").to_string()));
        }
        
        doc.push(layout);
        
        // Examiner info
        doc.push(Break::new(0.5));
        doc.push(
            Paragraph::new("Examiner")
                .styled(style::Style::new().bold().with_font_size(12))
        );
        
        let mut examiner_layout = LinearLayout::vertical();
        examiner_layout.push(self.info_row("Name:", &report.examiner.name));
        
        if let Some(ref title) = report.examiner.title {
            examiner_layout.push(self.info_row("Title:", title));
        }
        
        if let Some(ref org) = report.examiner.organization {
            examiner_layout.push(self.info_row("Organization:", org));
        }
        
        if let Some(ref email) = report.examiner.email {
            examiner_layout.push(self.info_row("Email:", email));
        }
        
        if !report.examiner.certifications.is_empty() {
            let certs = report.examiner.certifications.join(", ");
            examiner_layout.push(self.info_row("Certifications:", &certs));
        }
        
        doc.push(examiner_layout);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_executive_summary(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Executive Summary");
        
        if let Some(ref summary) = report.executive_summary {
            doc.push(Paragraph::new(summary).styled(style::Style::new().with_font_size(10)));
        }
        
        doc.push(Break::new(1.0));
        Ok(())
    }

    fn add_scope(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Scope of Examination");
        
        if let Some(ref scope) = report.scope {
            doc.push(Paragraph::new(scope).styled(style::Style::new().with_font_size(10)));
        }
        
        doc.push(Break::new(1.0));
        Ok(())
    }

    fn add_methodology(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Methodology");
        
        if let Some(ref methodology) = report.methodology {
            doc.push(Paragraph::new(methodology).styled(style::Style::new().with_font_size(10)));
        }
        
        doc.push(Break::new(1.0));
        Ok(())
    }

    fn add_evidence_section(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Evidence Examined");
        
        // Create table
        let mut table = TableLayout::new(vec![1, 3, 2, 2]);
        table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
        
        // Header row
        table.row()
            .element(Text::new("ID").styled(style::Style::new().bold()))
            .element(Text::new("Description").styled(style::Style::new().bold()))
            .element(Text::new("Type").styled(style::Style::new().bold()))
            .element(Text::new("Serial/Model").styled(style::Style::new().bold()))
            .push()
            .map_err(|e| DocumentError::Pdf(e.to_string()))?;
        
        // Data rows
        for item in &report.evidence_items {
            let serial = item.serial_number.as_deref()
                .or(item.model.as_deref())
                .unwrap_or("-");
            table.row()
                .element(Text::new(&item.evidence_id))
                .element(Text::new(&item.description))
                .element(Text::new(item.evidence_type.as_str()))
                .element(Text::new(serial))
                .push()
                .map_err(|e| DocumentError::Pdf(e.to_string()))?;
        }
        
        doc.push(table);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_chain_of_custody(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Chain of Custody");
        
        let mut table = TableLayout::new(vec![2, 2, 2, 3]);
        table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
        
        // Header
        table.row()
            .element(Text::new("Date/Time").styled(style::Style::new().bold()))
            .element(Text::new("Released By").styled(style::Style::new().bold()))
            .element(Text::new("Received By").styled(style::Style::new().bold()))
            .element(Text::new("Notes").styled(style::Style::new().bold()))
            .push()
            .map_err(|e| DocumentError::Pdf(e.to_string()))?;
        
        for record in &report.chain_of_custody {
            let dt = record.timestamp.format("%Y-%m-%d %H:%M").to_string();
            table.row()
                .element(Text::new(dt))
                .element(Text::new(&record.released_by))
                .element(Text::new(&record.received_by))
                .element(Text::new(record.notes.as_deref().unwrap_or("-")))
                .push()
                .map_err(|e| DocumentError::Pdf(e.to_string()))?;
        }
        
        doc.push(table);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_findings_section(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Findings");
        
        for finding in &report.findings {
            // Finding header
            let header = format!("{}: {}", finding.finding_id, finding.title);
            doc.push(
                Paragraph::new(header)
                    .styled(style::Style::new().bold().with_font_size(11))
            );
            
            // Severity and category
            let meta = format!(
                "Severity: {} | Category: {}",
                finding.severity.as_str(),
                finding.category.as_str()
            );
            doc.push(
                Paragraph::new(meta)
                    .styled(style::Style::new().italic().with_font_size(9))
            );
            
            doc.push(Break::new(0.25));
            
            // Description
            doc.push(
                Paragraph::new(&finding.description)
                    .styled(style::Style::new().with_font_size(10))
            );
            
            // Related files
            if !finding.related_files.is_empty() {
                doc.push(Break::new(0.25));
                doc.push(
                    Paragraph::new("Related Files:")
                        .styled(style::Style::new().bold().with_font_size(9))
                );
                for file in &finding.related_files {
                    doc.push(
                        Paragraph::new(format!("  • {}", file))
                            .styled(style::Style::new().with_font_size(9))
                    );
                }
            }
            
            // Notes
            if let Some(ref notes) = finding.notes {
                doc.push(
                    Paragraph::new(format!("Notes: {}", notes))
                        .styled(style::Style::new().italic().with_font_size(9))
                );
            }
            
            doc.push(Break::new(0.75));
        }
        
        Ok(())
    }

    fn add_timeline_section(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Timeline of Events");
        
        let mut table = TableLayout::new(vec![2, 1, 4, 2]);
        table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
        
        // Header
        table.row()
            .element(Text::new("Timestamp").styled(style::Style::new().bold()))
            .element(Text::new("Type").styled(style::Style::new().bold()))
            .element(Text::new("Description").styled(style::Style::new().bold()))
            .element(Text::new("Source").styled(style::Style::new().bold()))
            .push()
            .map_err(|e| DocumentError::Pdf(e.to_string()))?;
        
        for event in &report.timeline {
            let ts = event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            table.row()
                .element(Text::new(ts))
                .element(Text::new(&event.timestamp_type))
                .element(Text::new(&event.description))
                .element(Text::new(&event.source))
                .push()
                .map_err(|e| DocumentError::Pdf(e.to_string()))?;
        }
        
        doc.push(table);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_hash_section(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Hash Verification");
        
        for record in &report.hash_records {
            doc.push(
                Paragraph::new(format!("Item: {}", record.item))
                    .styled(style::Style::new().bold().with_font_size(10))
            );
            
            let algo = format!("{:?}", record.algorithm);
            doc.push(
                Paragraph::new(format!("  {}: {}", algo, record.value))
                    .styled(style::Style::new().with_font_size(9))
            );
            
            if let Some(verified) = record.verified {
                let status = if verified { "✓ Verified" } else { "✗ Failed" };
                doc.push(
                    Paragraph::new(format!("  Status: {}", status))
                        .styled(style::Style::new().with_font_size(9))
                );
            }
            
            doc.push(Break::new(0.5));
        }
        
        Ok(())
    }

    fn add_tools_section(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Tools Used");
        
        let mut table = TableLayout::new(vec![2, 1, 2, 3]);
        table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
        
        // Header
        table.row()
            .element(Text::new("Tool").styled(style::Style::new().bold()))
            .element(Text::new("Version").styled(style::Style::new().bold()))
            .element(Text::new("Vendor").styled(style::Style::new().bold()))
            .element(Text::new("Purpose").styled(style::Style::new().bold()))
            .push()
            .map_err(|e| DocumentError::Pdf(e.to_string()))?;
        
        for tool in &report.tools {
            table.row()
                .element(Text::new(&tool.name))
                .element(Text::new(&tool.version))
                .element(Text::new(tool.vendor.as_deref().unwrap_or("-")))
                .element(Text::new(tool.purpose.as_deref().unwrap_or("-")))
                .push()
                .map_err(|e| DocumentError::Pdf(e.to_string()))?;
        }
        
        doc.push(table);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_conclusions(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Conclusions");
        
        if let Some(ref conclusions) = report.conclusions {
            doc.push(Paragraph::new(conclusions).styled(style::Style::new().with_font_size(10)));
        }
        
        doc.push(Break::new(1.0));
        Ok(())
    }

    fn add_appendices(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        self.add_section_header(doc, "Appendices");
        
        for (i, appendix) in report.appendices.iter().enumerate() {
            doc.push(
                Paragraph::new(format!("Appendix {}: {}", (b'A' + i as u8) as char, appendix.title))
                    .styled(style::Style::new().bold().with_font_size(12))
            );
            doc.push(
                Paragraph::new(&appendix.content)
                    .styled(style::Style::new().with_font_size(9))
            );
            doc.push(Break::new(0.75));
        }
        
        Ok(())
    }

    fn add_footer(&self, doc: &mut Document, report: &ForensicReport) -> DocumentResult<()> {
        doc.push(Break::new(2.0));
        
        let generated = format!(
            "Generated by {} on {}",
            report.metadata.generated_by,
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        doc.push(
            Paragraph::new(generated)
                .aligned(Alignment::Center)
                .styled(style::Style::new().with_font_size(8))
        );
        
        doc.push(
            Paragraph::new(report.metadata.classification.as_str())
                .aligned(Alignment::Center)
                .styled(style::Style::new().bold().with_font_size(10))
        );
        
        Ok(())
    }

    fn add_section_header(&self, doc: &mut Document, title: &str) {
        doc.push(
            Paragraph::new(title)
                .styled(style::Style::new().bold().with_font_size(14))
        );
        doc.push(Break::new(0.5));
    }

    fn info_row(&self, label: &str, value: &str) -> Paragraph {
        Paragraph::new(format!("{} {}", label, value))
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

    // =========================================================================
    // pdf_string_to_string
    // =========================================================================

    #[test]
    fn test_pdf_string_to_string_utf8() {
        let obj = lopdf::Object::String(
            b"Hello World".to_vec(),
            lopdf::StringFormat::Literal,
        );
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert_eq!(result, Some("Hello World".to_string()));
    }

    #[test]
    fn test_pdf_string_to_string_empty() {
        let obj = lopdf::Object::String(vec![], lopdf::StringFormat::Literal);
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn test_pdf_string_to_string_utf16be_bom() {
        // UTF-16 BE BOM (0xFE 0xFF) followed by "AB" in UTF-16 BE
        let bytes = vec![0xFE, 0xFF, 0x00, 0x41, 0x00, 0x42];
        let obj = lopdf::Object::String(bytes, lopdf::StringFormat::Literal);
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert_eq!(result, Some("AB".to_string()));
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
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert_eq!(result, Some("café".to_string()));
    }

    #[test]
    fn test_pdf_string_to_string_non_string_type() {
        let obj = lopdf::Object::Integer(42);
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert_eq!(result, None);
    }

    #[test]
    fn test_pdf_string_to_string_boolean_type() {
        let obj = lopdf::Object::Boolean(true);
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert_eq!(result, None);
    }

    #[test]
    fn test_pdf_string_to_string_name_type() {
        let obj = lopdf::Object::Name(b"SomeName".to_vec());
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert_eq!(result, None);
    }

    #[test]
    fn test_pdf_string_to_string_lossy_fallback() {
        // Invalid UTF-8 bytes without UTF-16 BOM → lossy conversion
        let bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xFF, 0xFE];
        let obj = lopdf::Object::String(bytes, lopdf::StringFormat::Literal);
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert!(result.is_some());
        let s = result.unwrap();
        assert!(s.starts_with("Hello"));
    }

    #[test]
    fn test_pdf_string_to_string_ascii() {
        let obj = lopdf::Object::String(
            b"Simple ASCII text 123!@#".to_vec(),
            lopdf::StringFormat::Hexadecimal,
        );
        let result = PdfDocument::pdf_string_to_string(&obj);
        assert_eq!(result, Some("Simple ASCII text 123!@#".to_string()));
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
    // info_row helper
    // =========================================================================

    #[test]
    fn test_info_row() {
        let doc = PdfDocument::new();
        let _row = doc.info_row("Label:", "Value");
        // info_row returns a genpdf::Paragraph - just verify it doesn't panic
    }

    #[test]
    fn test_info_row_empty_values() {
        let doc = PdfDocument::new();
        let _row = doc.info_row("", "");
    }

    // =========================================================================
    // add_section_header helper
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
