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
        let data = std::fs::read(path)?;
        self.read_bytes(&data)
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
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match name.as_str() {
                        "p" => {
                            in_paragraph = true;
                            current_text.clear();
                            heading_level = None;
                        }
                        "pStyle" => {
                            // Check for heading style
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
