// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! DOCX report generator using docx-rs
//!
//! Generates Microsoft Word documents from forensic report data.

use std::fs::File;
use std::path::Path;

use docx_rs::*;

use super::error::{ReportError, ReportResult};
use super::types::*;

/// DOCX generator for forensic reports
pub struct DocxGenerator {
    // Configuration options can be added here
}

impl DocxGenerator {
    /// Create a new DOCX generator
    pub fn new() -> Self {
        Self {}
    }

    /// Generate a DOCX report
    pub fn generate(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> ReportResult<()> {
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
        
        // Add findings
        if !report.findings.is_empty() {
            docx = self.add_findings_section(docx, report);
        }
        
        // Add timeline
        if !report.timeline.is_empty() {
            docx = self.add_timeline_section(docx, report);
        }
        
        // Add tools
        if !report.tools.is_empty() {
            docx = self.add_tools_section(docx, report);
        }
        
        // Add conclusions
        if report.conclusions.is_some() {
            docx = self.add_conclusions(docx, report);
        }
        
        // Add footer
        docx = self.add_footer(docx, report);

        // Write to file - pack() takes a writer that implements Write + Seek
        // File implements both traits
        let file = File::create(output_path.as_ref())?;
        docx.build()
            .pack(file)
            .map_err(|e| ReportError::Docx(e.to_string()))?;

        Ok(())
    }

    fn add_classification_header(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let classification = report.metadata.classification.as_str();
        
        // Classification banner - centered, bold, with background
        let para = Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(classification)
                    .bold()
                    .size(24) // 12pt
            )
            .align(AlignmentType::Center);
        
        docx.add_paragraph(para)
            .add_paragraph(Paragraph::new()) // Empty line
    }

    fn add_title(&self, docx: Docx, report: &ForensicReport) -> Docx {
        // Main title
        let title = Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(&report.metadata.title)
                    .bold()
                    .size(36) // 18pt
            )
            .align(AlignmentType::Center);
        
        // Subtitle
        let subtitle = format!(
            "Report #{} | Version {}",
            report.metadata.report_number,
            report.metadata.version
        );
        let sub = Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(&subtitle)
                    .size(20) // 10pt
            )
            .align(AlignmentType::Center);
        
        docx.add_paragraph(title)
            .add_paragraph(sub)
            .add_paragraph(Paragraph::new()) // Spacer
    }

    fn add_case_info(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Case Information");
        
        // Case details table
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
            .add_paragraph(Paragraph::new()) // Spacer
    }

    fn add_executive_summary(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Executive Summary");
        
        if let Some(ref summary) = report.executive_summary {
            docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(summary).size(20))
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
                Paragraph::new()
                    .add_run(Run::new().add_text(scope).size(20))
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
                Paragraph::new()
                    .add_run(Run::new().add_text(methodology).size(20))
            )
            .add_paragraph(Paragraph::new())
        } else {
            docx
        }
    }

    fn add_evidence_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Evidence Examined");
        
        // Create evidence table
        let mut table = Table::new(vec![
            // Header row
            TableRow::new(vec![
                self.header_cell("ID"),
                self.header_cell("Description"),
                self.header_cell("Type"),
                self.header_cell("Serial Number"),
                self.header_cell("Capacity"),
            ]),
        ]);
        
        // Add data rows
        for item in &report.evidence_items {
            table = table.add_row(TableRow::new(vec![
                self.data_cell(&item.evidence_id),
                self.data_cell(&item.description),
                self.data_cell(item.evidence_type.as_str()),
                self.data_cell(item.serial_number.as_deref().unwrap_or("-")),
                self.data_cell(item.capacity.as_deref().unwrap_or("-")),
            ]));
        }
        
        docx.add_table(table)
            .add_paragraph(Paragraph::new()) // Spacer
    }

    fn add_findings_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let mut docx = self.add_section_header(docx, "Findings");
        
        for finding in &report.findings {
            // Finding title
            let title = format!("{}: {}", finding.finding_id, finding.title);
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&title).bold().size(22))
            );
            
            // Severity and category
            let meta = format!(
                "Severity: {} | Category: {}",
                finding.severity.as_str(),
                finding.category.as_str()
            );
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&meta).italic().size(18))
            );
            
            // Description
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&finding.description).size(20))
            );
            
            // Related files
            if !finding.related_files.is_empty() {
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Related Files:").bold().size(18))
                );
                
                for file in &finding.related_files {
                    docx = docx.add_paragraph(
                        Paragraph::new()
                            .add_run(Run::new().add_text(format!("• {}", file)).size(18))
                    );
                }
            }
            
            // Notes
            if let Some(ref notes) = finding.notes {
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Notes: ").bold().size(18))
                        .add_run(Run::new().add_text(notes).size(18))
                );
            }
            
            docx = docx.add_paragraph(Paragraph::new()); // Spacer
        }
        
        docx
    }

    fn add_timeline_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Timeline of Events");
        
        let mut table = Table::new(vec![
            TableRow::new(vec![
                self.header_cell("Timestamp"),
                self.header_cell("Type"),
                self.header_cell("Description"),
                self.header_cell("Source"),
            ]),
        ]);
        
        for event in &report.timeline {
            let ts = event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            table = table.add_row(TableRow::new(vec![
                self.data_cell(&ts),
                self.data_cell(&event.timestamp_type),
                self.data_cell(&event.description),
                self.data_cell(&event.source),
            ]));
        }
        
        docx.add_table(table)
            .add_paragraph(Paragraph::new())
    }

    fn add_tools_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Tools Used");
        
        let mut table = Table::new(vec![
            TableRow::new(vec![
                self.header_cell("Tool"),
                self.header_cell("Version"),
                self.header_cell("Vendor"),
                self.header_cell("Purpose"),
            ]),
        ]);
        
        for tool in &report.tools {
            table = table.add_row(TableRow::new(vec![
                self.data_cell(&tool.name),
                self.data_cell(&tool.version),
                self.data_cell(tool.vendor.as_deref().unwrap_or("-")),
                self.data_cell(tool.purpose.as_deref().unwrap_or("-")),
            ]));
        }
        
        docx.add_table(table)
            .add_paragraph(Paragraph::new())
    }

    fn add_conclusions(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let docx = self.add_section_header(docx, "Conclusions");
        
        if let Some(ref conclusions) = report.conclusions {
            docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(conclusions).size(20))
            )
            .add_paragraph(Paragraph::new())
        } else {
            docx
        }
    }

    fn add_footer(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let generated = format!(
            "Generated by {} on {}",
            report.metadata.generated_by,
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        docx.add_paragraph(Paragraph::new()) // Spacer
            .add_paragraph(Paragraph::new()) // Spacer
            .add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&generated).size(16))
                    .align(AlignmentType::Center)
            )
            .add_paragraph(
                Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(report.metadata.classification.as_str())
                            .bold()
                            .size(20)
                    )
                    .align(AlignmentType::Center)
            )
    }

    fn add_section_header(&self, docx: Docx, title: &str) -> Docx {
        docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(title).bold().size(28)) // 14pt
        )
    }

    fn add_info_table(&self, docx: Docx, rows: &[(&str, String)]) -> Docx {
        let mut table = Table::new(vec![]);
        
        for (label, value) in rows {
            table = table.add_row(TableRow::new(vec![
                TableCell::new()
                    .add_paragraph(
                        Paragraph::new()
                            .add_run(Run::new().add_text(*label).bold().size(20))
                    )
                    .width(2000, WidthType::Dxa),
                TableCell::new()
                    .add_paragraph(
                        Paragraph::new()
                            .add_run(Run::new().add_text(value).size(20))
                    ),
            ]));
        }
        
        docx.add_table(table)
    }

    fn header_cell(&self, text: &str) -> TableCell {
        TableCell::new()
            .add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(text).bold().size(18))
            )
            .shading(Shading::new().fill("1a365d")) // Dark blue background
    }

    fn data_cell(&self, text: &str) -> TableCell {
        TableCell::new()
            .add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(text).size(18))
            )
    }
}

impl Default for DocxGenerator {
    fn default() -> Self {
        Self::new()
    }
}
