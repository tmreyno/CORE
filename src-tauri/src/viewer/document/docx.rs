// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified DOCX document handler
//!
//! This module provides both reading and writing capabilities for DOCX documents.
//!
//! # Reading
//! - Extracts text and structure from DOCX files
//! - Parses document.xml from the OOXML package
//!
//! # Writing  
//! - Uses `docx-rs` for report generation

use std::fs::File;
use std::io::{Read, Cursor};
use std::path::Path;

// Use docx_rs for writing
use docx_rs::{Docx, Paragraph, Run, Table, AlignmentType, WidthType, Shading};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use super::error::{DocumentError, DocumentResult};
use super::types::{
    DocumentContent, DocumentMetadata, DocumentPage, DocumentElement,
    ParagraphElement, HeadingElement, TextStyle, TableElement, 
    TableRow as DocTableRow, TableCell as DocTableCell,
};
use super::DocumentFormat;
use crate::report::ForensicReport;

/// DOCX document handler with read/write capabilities
pub struct DocxDocument;

impl DocxDocument {
    /// Create a new DOCX document handler
    pub fn new() -> Self {
        Self
    }

    // =========================================================================
    // READING
    // =========================================================================

    /// Read DOCX from file path
    pub fn read(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let data = std::fs::read(&path)?;
        
        // Check for OLE compound document magic bytes (legacy .doc format)
        // OLE magic: D0 CF 11 E0 A1 B1 1A E1
        if data.len() >= 8 && data[..4] == [0xD0, 0xCF, 0x11, 0xE0] {
            return self.read_legacy_doc(&data, path.as_ref());
        }
        
        self.read_bytes(&data)
    }
    
    /// Read legacy binary .doc format (OLE compound document)
    /// 
    /// Legacy .doc files use Microsoft's OLE2 compound binary format.
    /// We extract readable text by scanning for printable character runs
    /// in the binary data, since we don't have a full OLE parser.
    fn read_legacy_doc(&self, data: &[u8], path: &Path) -> DocumentResult<DocumentContent> {
        let mut metadata = DocumentMetadata {
            format: DocumentFormat::Docx,
            file_size: data.len() as u64,
            ..Default::default()
        };
        metadata.title = path.file_name()
            .and_then(|n| n.to_str())
            .map(String::from);
        
        // Extract text by finding runs of printable UTF-16LE characters
        // .doc files store text as UTF-16LE in the document stream
        let text = self.extract_text_from_ole(data);
        
        if text.is_empty() {
            return Err(DocumentError::Docx(
                "Legacy .doc file: unable to extract text. Use hex or text view to inspect.".to_string()
            ));
        }
        
        // Count words
        metadata.word_count = Some(text.split_whitespace().count());
        
        // Build document content with a single page of paragraphs
        let elements: Vec<DocumentElement> = text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| DocumentElement::Paragraph(ParagraphElement {
                text: line.to_string(),
                style: TextStyle::default(),
            }))
            .collect();
        
        let page = DocumentPage {
            page_number: 1,
            elements,
        };
        
        Ok(DocumentContent {
            metadata,
            pages: vec![page],
        })
    }
    
    /// Extract readable text from OLE compound document binary data
    /// 
    /// Scans for UTF-16LE encoded text runs (common in .doc files).
    /// Falls back to ASCII text extraction if UTF-16 yields nothing.
    pub(crate) fn extract_text_from_ole(&self, data: &[u8]) -> String {
        // Strategy 1: Try UTF-16LE extraction
        // .doc files store the main text stream as UTF-16LE
        let utf16_text = self.extract_utf16le_text(data);
        if utf16_text.len() > 100 {
            return utf16_text;
        }
        
        // Strategy 2: Extract ASCII text runs (printable chars, min length 4)
        let ascii_text = self.extract_ascii_text(data);
        if !ascii_text.is_empty() {
            return ascii_text;
        }
        
        utf16_text
    }
    
    /// Extract UTF-16LE text from binary data
    fn extract_utf16le_text(&self, data: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;
        let mut current_run = String::new();
        
        while i + 1 < data.len() {
            let code = u16::from_le_bytes([data[i], data[i + 1]]);
            i += 2;
            
            if let Some(ch) = char::from_u32(code as u32) {
                if ch.is_alphanumeric() || ch.is_whitespace() || ".,;:!?'\"-()[]{}/@#$%&*+=<>~`^_|\\".contains(ch) {
                    current_run.push(ch);
                    continue;
                }
            }
            
            // End of run — keep it if substantial
            if current_run.len() >= 8 {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(current_run.trim());
            }
            current_run.clear();
        }
        
        // Don't forget last run
        if current_run.len() >= 8 {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(current_run.trim());
        }
        
        result
    }
    
    /// Extract ASCII text runs from binary data
    fn extract_ascii_text(&self, data: &[u8]) -> String {
        let mut result = String::new();
        let mut current_run = String::new();
        
        for &byte in data {
            if byte >= 0x20 && byte < 0x7F || byte == b'\n' || byte == b'\r' || byte == b'\t' {
                current_run.push(byte as char);
            } else {
                if current_run.len() >= 20 {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str(current_run.trim());
                }
                current_run.clear();
            }
        }
        
        if current_run.len() >= 20 {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(current_run.trim());
        }
        
        result
    }

    /// Read DOCX from bytes
    pub fn read_bytes(&self, data: &[u8]) -> DocumentResult<DocumentContent> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| DocumentError::Docx(format!("Failed to open DOCX: {}", e)))?;

        // Extract metadata from docProps/core.xml
        let metadata = self.extract_metadata(&mut archive)?;

        // Extract content from word/document.xml
        let pages = self.extract_content(&mut archive)?;

        Ok(DocumentContent { metadata, pages })
    }

    /// Extract metadata from core.xml
    fn extract_metadata<R: Read + std::io::Seek>(&self, archive: &mut ZipArchive<R>) -> DocumentResult<DocumentMetadata> {
        let mut metadata = DocumentMetadata {
            format: DocumentFormat::Docx,
            ..Default::default()
        };

        // Try to read core.xml
        if let Ok(mut core_file) = archive.by_name("docProps/core.xml") {
            let mut xml_content = String::new();
            core_file.read_to_string(&mut xml_content)?;

            let mut reader = Reader::from_str(&xml_content);
            reader.config_mut().trim_text(true);
            
            let mut current_element = String::new();
            
            loop {
                match reader.read_event() {
                    Ok(Event::Start(e)) => {
                        current_element = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    }
                    Ok(Event::Text(e)) => {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_element.as_str() {
                            "title" => metadata.title = Some(text),
                            "creator" | "author" => metadata.author = Some(text),
                            "subject" => metadata.subject = Some(text),
                            "keywords" => {
                                metadata.keywords = text.split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => {
                        tracing::warn!("Error parsing core.xml: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Try to read app.xml for additional info
        if let Ok(mut app_file) = archive.by_name("docProps/app.xml") {
            let mut xml_content = String::new();
            app_file.read_to_string(&mut xml_content)?;

            let mut reader = Reader::from_str(&xml_content);
            reader.config_mut().trim_text(true);
            
            let mut current_element = String::new();
            
            loop {
                match reader.read_event() {
                    Ok(Event::Start(e)) => {
                        current_element = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    }
                    Ok(Event::Text(e)) => {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_element.as_str() {
                            "Application" => metadata.creator = Some(text),
                            "Pages" => {
                                if let Ok(count) = text.parse() {
                                    metadata.page_count = Some(count);
                                }
                            }
                            "Words" => {
                                if let Ok(count) = text.parse() {
                                    metadata.word_count = Some(count);
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
        }

        Ok(metadata)
    }

    /// Extract content from document.xml
    fn extract_content<R: Read + std::io::Seek>(&self, archive: &mut ZipArchive<R>) -> DocumentResult<Vec<DocumentPage>> {
        let mut doc_file = archive.by_name("word/document.xml")
            .map_err(|e| DocumentError::Docx(format!("Missing document.xml: {}", e)))?;

        let mut xml_content = String::new();
        doc_file.read_to_string(&mut xml_content)?;

        let mut reader = Reader::from_str(&xml_content);
        reader.config_mut().trim_text(true);

        let mut elements = Vec::new();
        let mut current_text = String::new();
        let mut in_paragraph = false;
        let mut in_table = false;
        let mut current_table: Option<TableElement> = None;
        let mut current_row: Option<DocTableRow> = None;
        let mut current_cell_text = String::new();
        let mut heading_level: Option<u8> = None;

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match name.as_str() {
                        "p" => {
                            in_paragraph = true;
                            current_text.clear();
                            heading_level = None;
                        }
                        "pStyle" => {
                            // Check for heading style
                            // Note: pStyle is typically self-closing (<w:pStyle w:val="Heading1"/>)
                            // which generates Event::Empty in quick_xml
                            for attr in e.attributes().filter_map(|a| a.ok()) {
                                if attr.key.as_ref() == b"w:val" {
                                    let val = String::from_utf8_lossy(&attr.value).to_string();
                                    if val.starts_with("Heading") {
                                        if let Ok(level) = val.trim_start_matches("Heading").parse::<u8>() {
                                            heading_level = Some(level);
                                        }
                                    }
                                }
                            }
                        }
                        "tbl" => {
                            in_table = true;
                            current_table = Some(TableElement {
                                rows: Vec::new(),
                                has_header: true,
                            });
                        }
                        "tr" => {
                            current_row = Some(DocTableRow { cells: Vec::new() });
                        }
                        "tc" => {
                            current_cell_text.clear();
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match name.as_str() {
                        "p" => {
                            if in_paragraph && !current_text.trim().is_empty() {
                                if !in_table {
                                    if let Some(level) = heading_level {
                                        elements.push(DocumentElement::Heading(HeadingElement {
                                            text: current_text.trim().to_string(),
                                            level,
                                        }));
                                    } else {
                                        elements.push(DocumentElement::Paragraph(ParagraphElement {
                                            text: current_text.trim().to_string(),
                                            style: TextStyle::default(),
                                        }));
                                    }
                                } else {
                                    current_cell_text.push_str(current_text.trim());
                                    current_cell_text.push(' ');
                                }
                            }
                            in_paragraph = false;
                            current_text.clear();
                        }
                        "tc" => {
                            if let Some(ref mut row) = current_row {
                                row.cells.push(DocTableCell {
                                    text: current_cell_text.trim().to_string(),
                                    style: TextStyle::default(),
                                });
                            }
                            current_cell_text.clear();
                        }
                        "tr" => {
                            if let (Some(ref mut table), Some(row)) = (&mut current_table, current_row.take()) {
                                table.rows.push(row);
                            }
                        }
                        "tbl" => {
                            if let Some(table) = current_table.take() {
                                elements.push(DocumentElement::Table(table));
                            }
                            in_table = false;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_paragraph {
                        current_text.push_str(&e.unescape().unwrap_or_default());
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    tracing::warn!("Error parsing document.xml: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // Create a single page with all content (DOCX doesn't have explicit pages)
        if elements.is_empty() {
            elements.push(DocumentElement::Paragraph(ParagraphElement {
                text: String::new(),
                style: TextStyle::default(),
            }));
        }

        Ok(vec![DocumentPage {
            page_number: 1,
            elements,
        }])
    }

    /// Get metadata without reading full document
    pub fn get_metadata(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentMetadata> {
        let file = File::open(path.as_ref())?;
        let file_size = file.metadata()?.len();
        let mut archive = ZipArchive::new(file)
            .map_err(|e| DocumentError::Docx(format!("Failed to open DOCX: {}", e)))?;

        let mut metadata = self.extract_metadata(&mut archive)?;
        metadata.file_size = file_size;
        Ok(metadata)
    }

    // =========================================================================
    // WRITING
    // =========================================================================

    /// Write a forensic report to DOCX
    pub fn write_report(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> DocumentResult<()> {
        let mut docx = Docx::new();

        // Add classification header
        docx = self.add_classification_header(docx, report);
        
        // Add title
        docx = self.add_title(docx, report);
        
        // Add case information
        docx = self.add_case_info(docx, report);
        
        // Add executive summary if present
        if report.executive_summary.is_some() {
            docx = self.add_executive_summary(docx, report);
        }
        
        // Add scope if present
        if report.scope.is_some() {
            docx = self.add_scope(docx, report);
        }
        
        // Add methodology if present
        if report.methodology.is_some() {
            docx = self.add_methodology(docx, report);
        }
        
        // Add evidence section
        if !report.evidence_items.is_empty() {
            docx = self.add_evidence_section(docx, report);
        }
        
        // Add chain of custody
        if !report.chain_of_custody.is_empty() {
            docx = self.add_chain_of_custody(docx, report);
        }
        
        // Add findings
        if !report.findings.is_empty() {
            docx = self.add_findings_section(docx, report);
        }
        
        // Add timeline
        if !report.timeline.is_empty() {
            docx = self.add_timeline_section(docx, report);
        }
        
        // Add hash verification
        if !report.hash_records.is_empty() {
            docx = self.add_hash_section(docx, report);
        }
        
        // Add tools
        if !report.tools.is_empty() {
            docx = self.add_tools_section(docx, report);
        }
        
        // Add conclusions
        if report.conclusions.is_some() {
            docx = self.add_conclusions(docx, report);
        }
        
        // Add appendices
        if !report.appendices.is_empty() {
            docx = self.add_appendices(docx, report);
        }
        
        // Add footer
        docx = self.add_footer(docx, report);

        // Write to file
        let file = File::create(output_path.as_ref())?;
        docx.build()
            .pack(file)
            .map_err(|e| DocumentError::Docx(e.to_string()))?;

        Ok(())
    }

    fn add_classification_header(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let classification = report.metadata.classification.as_str();
        
        docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(classification).bold().size(24))
                .align(AlignmentType::Center)
        )
        .add_paragraph(Paragraph::new())
    }

    fn add_title(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let subtitle = format!(
            "Report #{} | Version {}",
            report.metadata.report_number,
            report.metadata.version
        );
        
        let date = report.metadata.generated_at.format("%B %d, %Y").to_string();
        
        docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(&report.metadata.title).bold().size(36))
                .align(AlignmentType::Center)
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(&subtitle).size(20))
                .align(AlignmentType::Center)
        )
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(&date).size(20))
                .align(AlignmentType::Center)
        )
        .add_paragraph(Paragraph::new())
    }

    fn add_case_info(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Case Information");
        
        let mut rows = vec![
            ("Case Number", report.case_info.case_number.clone()),
        ];
        
        if let Some(ref name) = report.case_info.case_name {
            rows.push(("Case Name", name.clone()));
        }
        if let Some(ref agency) = report.case_info.agency {
            rows.push(("Agency", agency.clone()));
        }
        if let Some(ref requestor) = report.case_info.requestor {
            rows.push(("Requestor", requestor.clone()));
        }
        if let Some(ref date) = report.case_info.request_date {
            rows.push(("Request Date", date.format("%Y-%m-%d").to_string()));
        }
        
        let docx = self.add_info_table(docx, &rows);
        
        // Examiner section
        let docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Examiner").bold().size(24))
        );
        
        let mut examiner_rows = vec![
            ("Name", report.examiner.name.clone()),
        ];
        
        if let Some(ref title) = report.examiner.title {
            examiner_rows.push(("Title", title.clone()));
        }
        if let Some(ref org) = report.examiner.organization {
            examiner_rows.push(("Organization", org.clone()));
        }
        if let Some(ref email) = report.examiner.email {
            examiner_rows.push(("Email", email.clone()));
        }
        if !report.examiner.certifications.is_empty() {
            examiner_rows.push(("Certifications", report.examiner.certifications.join(", ")));
        }
        
        self.add_info_table(docx, &examiner_rows)
            .add_paragraph(Paragraph::new())
    }

    fn add_executive_summary(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Executive Summary");
        
        if let Some(ref summary) = report.executive_summary {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(summary).size(20))
            )
            .add_paragraph(Paragraph::new())
        } else {
            docx
        }
    }

    fn add_scope(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Scope of Examination");
        
        if let Some(ref scope) = report.scope {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(scope).size(20))
            )
            .add_paragraph(Paragraph::new())
        } else {
            docx
        }
    }

    fn add_methodology(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Methodology");
        
        if let Some(ref methodology) = report.methodology {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(methodology).size(20))
            )
            .add_paragraph(Paragraph::new())
        } else {
            docx
        }
    }

    fn add_evidence_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Evidence Examined");
        
        let mut table = Table::new(vec![
            docx_rs::TableRow::new(vec![
                self.header_cell("ID"),
                self.header_cell("Description"),
                self.header_cell("Type"),
                self.header_cell("Serial/Model"),
            ]),
        ]);
        
        for item in &report.evidence_items {
            let serial = item.serial_number.as_deref()
                .or(item.model.as_deref())
                .unwrap_or("-");
            table = table.add_row(docx_rs::TableRow::new(vec![
                self.data_cell(&item.evidence_id),
                self.data_cell(&item.description),
                self.data_cell(item.evidence_type.as_str()),
                self.data_cell(serial),
            ]));
        }
        
        docx.add_table(table).add_paragraph(Paragraph::new())
    }

    fn add_chain_of_custody(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Chain of Custody");
        
        let mut table = Table::new(vec![
            docx_rs::TableRow::new(vec![
                self.header_cell("Date/Time"),
                self.header_cell("Released By"),
                self.header_cell("Received By"),
                self.header_cell("Purpose"),
            ]),
        ]);
        
        for record in &report.chain_of_custody {
            let dt = record.timestamp.format("%Y-%m-%d %H:%M").to_string();
            table = table.add_row(docx_rs::TableRow::new(vec![
                self.data_cell(&dt),
                self.data_cell(&record.released_by),
                self.data_cell(&record.received_by),
                self.data_cell(record.purpose.as_deref().unwrap_or("-")),
            ]));
        }
        
        docx.add_table(table).add_paragraph(Paragraph::new())
    }

    fn add_findings_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let mut docx = self.add_section_header(docx, "Findings");
        
        for finding in &report.findings {
            let title = format!("{}: {}", finding.finding_id, finding.title);
            let meta = format!(
                "Severity: {} | Category: {}",
                finding.severity.as_str(),
                finding.category.as_str()
            );
            
            docx = docx
                .add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(&title).bold().size(22))
                )
                .add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(&meta).italic().size(18))
                )
                .add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(&finding.description).size(20))
                );
            
            if !finding.related_files.is_empty() {
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text("Related Files:").bold().size(18))
                );
                
                for file in &finding.related_files {
                    docx = docx.add_paragraph(
                        Paragraph::new().add_run(Run::new().add_text(format!("• {}", file)).size(18))
                    );
                }
            }
            
            if let Some(ref notes) = finding.notes {
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Notes: ").bold().size(18))
                        .add_run(Run::new().add_text(notes).italic().size(18))
                );
            }
            
            docx = docx.add_paragraph(Paragraph::new());
        }
        
        docx
    }

    fn add_timeline_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Timeline of Events");
        
        let mut table = Table::new(vec![
            docx_rs::TableRow::new(vec![
                self.header_cell("Timestamp"),
                self.header_cell("Type"),
                self.header_cell("Description"),
                self.header_cell("Source"),
            ]),
        ]);
        
        for event in &report.timeline {
            let ts = event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            table = table.add_row(docx_rs::TableRow::new(vec![
                self.data_cell(&ts),
                self.data_cell(&event.timestamp_type),
                self.data_cell(&event.description),
                self.data_cell(&event.source),
            ]));
        }
        
        docx.add_table(table).add_paragraph(Paragraph::new())
    }

    fn add_hash_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Hash Verification");
        
        // Create table for hash records
        let mut table = Table::new(vec![
            docx_rs::TableRow::new(vec![
                self.header_cell("Item"),
                self.header_cell("Algorithm"),
                self.header_cell("Hash Value"),
                self.header_cell("Verified"),
            ]),
        ]);
        
        for record in &report.hash_records {
            let verified_str = match record.verified {
                Some(true) => "✓ Yes",
                Some(false) => "✗ No",
                None => "-",
            };
            
            table = table.add_row(docx_rs::TableRow::new(vec![
                self.data_cell(&record.item),
                self.data_cell(record.algorithm.as_str()),
                self.data_cell(&record.value),
                self.data_cell(verified_str),
            ]));
        }
        
        docx.add_table(table).add_paragraph(Paragraph::new())
    }

    fn add_tools_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Tools Used");
        
        let mut table = Table::new(vec![
            docx_rs::TableRow::new(vec![
                self.header_cell("Tool"),
                self.header_cell("Version"),
                self.header_cell("Vendor"),
                self.header_cell("Purpose"),
            ]),
        ]);
        
        for tool in &report.tools {
            table = table.add_row(docx_rs::TableRow::new(vec![
                self.data_cell(&tool.name),
                self.data_cell(&tool.version),
                self.data_cell(tool.vendor.as_deref().unwrap_or("-")),
                self.data_cell(tool.purpose.as_deref().unwrap_or("-")),
            ]));
        }
        
        docx.add_table(table).add_paragraph(Paragraph::new())
    }

    fn add_conclusions(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Conclusions");
        
        if let Some(ref conclusions) = report.conclusions {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(conclusions).size(20))
            )
            .add_paragraph(Paragraph::new())
        } else {
            docx
        }
    }

    fn add_appendices(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let mut docx = self.add_section_header(docx, "Appendices");
        
        for (i, appendix) in report.appendices.iter().enumerate() {
            docx = docx
                .add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text(format!("Appendix {}: {}", (b'A' + i as u8) as char, appendix.title)).bold().size(24))
                )
                .add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(&appendix.content).size(18))
                )
                .add_paragraph(Paragraph::new());
        }
        
        docx
    }

    fn add_footer(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let generated = format!(
            "Generated by {} on {}",
            report.metadata.generated_by,
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        docx.add_paragraph(Paragraph::new())
            .add_paragraph(Paragraph::new())
            .add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&generated).size(16))
                    .align(AlignmentType::Center)
            )
            .add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(report.metadata.classification.as_str()).bold().size(20))
                    .align(AlignmentType::Center)
            )
    }

    fn add_section_header(&self, docx: Docx, title: &str) -> Docx {
        docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(title).bold().size(28))
        )
    }

    fn add_info_table(&self, docx: Docx, rows: &[(&str, String)]) -> Docx {
        let mut table = Table::new(vec![]);
        
        for (label, value) in rows {
            table = table.add_row(docx_rs::TableRow::new(vec![
                docx_rs::TableCell::new()
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(*label).bold().size(20)))
                    .width(2000, WidthType::Dxa),
                docx_rs::TableCell::new()
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(value).size(20))),
            ]));
        }
        
        docx.add_table(table)
    }

    fn header_cell(&self, text: &str) -> docx_rs::TableCell {
        docx_rs::TableCell::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text(text).bold().size(18)))
            .shading(Shading::new().fill("1a365d"))
    }

    fn data_cell(&self, text: &str) -> docx_rs::TableCell {
        docx_rs::TableCell::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text(text).size(18)))
    }
}

impl Default for DocxDocument {
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

    /// Build a minimal valid DOCX (zip) in memory with the specified XML files.
    fn build_docx_bytes(files: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            for (name, content) in files {
                zip.start_file(*name, options).unwrap();
                zip.write_all(content.as_bytes()).unwrap();
            }
            zip.finish().unwrap();
        }
        buf
    }

    /// A minimal [Content_Types].xml required by zip/OOXML.
    const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
</Types>"#;

    /// Minimal document.xml with a single paragraph.
    fn simple_document_xml(text: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>{}</w:t></w:r></w:p>
  </w:body>
</w:document>"#,
            text
        )
    }

    /// Document.xml with a heading and paragraph.
    fn heading_document_xml(heading: &str, level: u8, body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:pStyle w:val="Heading{level}"/></w:pPr>
      <w:r><w:t>{heading}</w:t></w:r>
    </w:p>
    <w:p><w:r><w:t>{body}</w:t></w:r></w:p>
  </w:body>
</w:document>"#,
        )
    }

    /// Document.xml with a table.
    fn table_document_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:tbl>
      <w:tr>
        <w:tc><w:p><w:r><w:t>Header1</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:t>Header2</w:t></w:r></w:p></w:tc>
      </w:tr>
      <w:tr>
        <w:tc><w:p><w:r><w:t>Cell1</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:t>Cell2</w:t></w:r></w:p></w:tc>
      </w:tr>
    </w:tbl>
  </w:body>
</w:document>"#.to_string()
    }

    /// Core.xml with metadata.
    fn core_xml(title: &str, author: &str, subject: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
  xmlns:dc="http://purl.org/dc/elements/1.1/"
  xmlns:dcterms="http://purl.org/dc/terms/">
  <dc:title>{title}</dc:title>
  <dc:creator>{author}</dc:creator>
  <dc:subject>{subject}</dc:subject>
</cp:coreProperties>"#,
        )
    }

    /// App.xml with word count and page count.
    fn app_xml(pages: u32, words: u32, app: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>{app}</Application>
  <Pages>{pages}</Pages>
  <Words>{words}</Words>
</Properties>"#,
        )
    }

    // =========================================================================
    // Constructor
    // =========================================================================

    #[test]
    fn test_new_creates_instance() {
        let _ = DocxDocument::new();
    }

    #[test]
    fn test_default_creates_instance() {
        let _ = DocxDocument::default();
    }

    // =========================================================================
    // read_bytes - basic content extraction
    // =========================================================================

    #[test]
    fn test_read_bytes_simple_paragraph() {
        let doc_xml = simple_document_xml("Hello World");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        assert_eq!(content.pages.len(), 1);
        assert_eq!(content.pages[0].page_number, 1);
        assert!(!content.pages[0].elements.is_empty());

        // First element should be a paragraph with "Hello World"
        match &content.pages[0].elements[0] {
            DocumentElement::Paragraph(p) => {
                assert_eq!(p.text, "Hello World");
            }
            other => panic!("Expected Paragraph, got {:?}", other),
        }
    }

    #[test]
    fn test_read_bytes_heading() {
        let doc_xml = heading_document_xml("Introduction", 1, "Some body text");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        let elements = &content.pages[0].elements;
        assert!(elements.len() >= 2);

        match &elements[0] {
            DocumentElement::Heading(h) => {
                assert_eq!(h.text, "Introduction");
                assert_eq!(h.level, 1);
            }
            other => panic!("Expected Heading, got {:?}", other),
        }

        match &elements[1] {
            DocumentElement::Paragraph(p) => {
                assert_eq!(p.text, "Some body text");
            }
            other => panic!("Expected Paragraph, got {:?}", other),
        }
    }

    #[test]
    fn test_read_bytes_heading_level_2() {
        let doc_xml = heading_document_xml("Section", 2, "Details");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        match &content.pages[0].elements[0] {
            DocumentElement::Heading(h) => {
                assert_eq!(h.level, 2);
            }
            other => panic!("Expected Heading, got {:?}", other),
        }
    }

    #[test]
    fn test_read_bytes_table() {
        let doc_xml = table_document_xml();
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        let elements = &content.pages[0].elements;

        // Should have one table element
        let table = elements.iter().find(|e| matches!(e, DocumentElement::Table(_)));
        assert!(table.is_some(), "Expected a Table element");

        match table.unwrap() {
            DocumentElement::Table(t) => {
                assert_eq!(t.rows.len(), 2);
                assert_eq!(t.rows[0].cells.len(), 2);
                assert_eq!(t.rows[0].cells[0].text, "Header1");
                assert_eq!(t.rows[0].cells[1].text, "Header2");
                assert_eq!(t.rows[1].cells[0].text, "Cell1");
                assert_eq!(t.rows[1].cells[1].text, "Cell2");
                assert!(t.has_header);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_read_bytes_empty_document() {
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body></w:body>
</w:document>"#;

        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        // Empty document should still have one page with at least one element
        assert_eq!(content.pages.len(), 1);
        assert!(!content.pages[0].elements.is_empty());
    }

    // =========================================================================
    // Metadata extraction
    // =========================================================================

    #[test]
    fn test_metadata_from_core_xml() {
        let core = core_xml("Test Report", "John Doe", "Forensic Analysis");
        let doc_xml = simple_document_xml("Content");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
            ("docProps/core.xml", &core),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        assert_eq!(content.metadata.title, Some("Test Report".to_string()));
        assert_eq!(content.metadata.author, Some("John Doe".to_string()));
        assert_eq!(content.metadata.subject, Some("Forensic Analysis".to_string()));
    }

    #[test]
    fn test_metadata_format_is_docx() {
        let doc_xml = simple_document_xml("Test");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        assert_eq!(content.metadata.format, DocumentFormat::Docx);
    }

    #[test]
    fn test_metadata_from_app_xml() {
        let app = app_xml(5, 1200, "Microsoft Word");
        let doc_xml = simple_document_xml("Content");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
            ("docProps/app.xml", &app),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        assert_eq!(content.metadata.page_count, Some(5));
        assert_eq!(content.metadata.word_count, Some(1200));
        // Application goes into creator field from app.xml
        assert_eq!(content.metadata.creator, Some("Microsoft Word".to_string()));
    }

    #[test]
    fn test_metadata_core_and_app_combined() {
        let core = core_xml("Report Title", "Examiner Smith", "Digital Forensics");
        let app = app_xml(10, 5000, "LibreOffice");
        let doc_xml = simple_document_xml("Content");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
            ("docProps/core.xml", &core),
            ("docProps/app.xml", &app),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        assert_eq!(content.metadata.title, Some("Report Title".to_string()));
        assert_eq!(content.metadata.author, Some("Examiner Smith".to_string()));
        assert_eq!(content.metadata.subject, Some("Digital Forensics".to_string()));
        assert_eq!(content.metadata.page_count, Some(10));
        assert_eq!(content.metadata.word_count, Some(5000));
    }

    #[test]
    fn test_metadata_missing_core_xml() {
        let doc_xml = simple_document_xml("Content");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        // No core.xml → metadata fields should be None
        assert_eq!(content.metadata.title, None);
        assert_eq!(content.metadata.author, None);
        assert_eq!(content.metadata.subject, None);
    }

    #[test]
    fn test_metadata_keywords() {
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
  xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>Report</dc:title>
  <cp:keywords>forensics, evidence, digital</cp:keywords>
</cp:coreProperties>"#;

        let doc_xml = simple_document_xml("Content");
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", &doc_xml),
            ("docProps/core.xml", core),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        assert_eq!(content.metadata.keywords, vec!["forensics", "evidence", "digital"]);
    }

    // =========================================================================
    // Error handling
    // =========================================================================

    #[test]
    fn test_read_bytes_invalid_zip() {
        let doc = DocxDocument::new();
        let result = doc.read_bytes(b"not a zip file");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_bytes_missing_document_xml() {
        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
        ]);

        let doc = DocxDocument::new();
        let result = doc.read_bytes(&data);
        assert!(result.is_err());
    }

    // =========================================================================
    // Multiple paragraphs
    // =========================================================================

    #[test]
    fn test_read_bytes_multiple_paragraphs() {
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>First paragraph.</w:t></w:r></w:p>
    <w:p><w:r><w:t>Second paragraph.</w:t></w:r></w:p>
    <w:p><w:r><w:t>Third paragraph.</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        let elements = &content.pages[0].elements;
        assert_eq!(elements.len(), 3);

        let texts: Vec<&str> = elements.iter().filter_map(|e| match e {
            DocumentElement::Paragraph(p) => Some(p.text.as_str()),
            _ => None,
        }).collect();
        assert_eq!(texts, vec!["First paragraph.", "Second paragraph.", "Third paragraph."]);
    }

    #[test]
    fn test_read_bytes_mixed_content() {
        // Heading + paragraphs + table in one document
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
      <w:r><w:t>Title</w:t></w:r>
    </w:p>
    <w:p><w:r><w:t>Body text.</w:t></w:r></w:p>
    <w:tbl>
      <w:tr>
        <w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
        <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>
      </w:tr>
    </w:tbl>
    <w:p><w:r><w:t>After table.</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

        let data = build_docx_bytes(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("word/document.xml", doc_xml),
        ]);

        let doc = DocxDocument::new();
        let content = doc.read_bytes(&data).unwrap();
        let elements = &content.pages[0].elements;

        // Should have: Heading, Paragraph, Table, Paragraph
        assert!(elements.len() >= 4, "Expected at least 4 elements, got {}", elements.len());

        assert!(matches!(&elements[0], DocumentElement::Heading(_)));
        assert!(matches!(&elements[1], DocumentElement::Paragraph(_)));

        let has_table = elements.iter().any(|e| matches!(e, DocumentElement::Table(_)));
        assert!(has_table, "Expected a Table element in mixed content");
    }
}
