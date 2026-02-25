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

        // Add evidence collection section
        if report.evidence_collection.is_some() {
            docx = self.add_evidence_collection_section(docx, report);
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

    fn add_evidence_collection_section(&self, docx: Docx, report: &ForensicReport) -> Docx {
        let ev = match &report.evidence_collection {
            Some(ec) => ec,
            None => return docx,
        };

        let mut docx = self.add_section_header(docx, "Evidence Collection");

        // Collection header info
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Collection Date: ").bold())
                .add_run(Run::new().add_text(&ev.collection_date))
        );
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Collecting Officer: ").bold())
                .add_run(Run::new().add_text(&ev.collecting_officer))
        );
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("Authorization: ").bold())
                .add_run(Run::new().add_text(&ev.authorization))
        );
        if let Some(ref aa) = ev.authorizing_authority {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text("Authorizing Authority: ").bold())
                    .add_run(Run::new().add_text(aa))
            );
        }
        if !ev.witnesses.is_empty() {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text("Witnesses: ").bold())
                    .add_run(Run::new().add_text(&ev.witnesses.join(", ")))
            );
        }

        docx = docx.add_paragraph(Paragraph::new()); // Spacer

        // Collected items table
        if !ev.collected_items.is_empty() {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&format!("Collected Items ({})", ev.collected_items.len())).bold().size(22))
            );

            let mut table = Table::new(vec![
                TableRow::new(vec![
                    self.header_cell("Item #"),
                    self.header_cell("Description"),
                    self.header_cell("Device Type"),
                    self.header_cell("Make/Model"),
                    self.header_cell("Serial #"),
                    self.header_cell("Location"),
                    self.header_cell("Format"),
                    self.header_cell("Condition"),
                ]),
            ]);

            for item in &ev.collected_items {
                let device = if !item.device_type.is_empty() {
                    item.device_type.as_str()
                } else {
                    item.item_type.as_str()
                };
                let make_model = format!(
                    "{}{}",
                    item.brand.as_deref().or(item.make.as_deref()).unwrap_or(""),
                    item.model.as_deref().map(|m| format!(" {}", m)).unwrap_or_default()
                );
                let location = if item.building.is_some() || item.room.is_some() {
                    format!(
                        "{}{}",
                        item.building.as_deref().unwrap_or(""),
                        item.room.as_deref().map(|r| format!(" / {}", r)).unwrap_or_default()
                    )
                } else {
                    item.found_location.clone()
                };

                table = table.add_row(TableRow::new(vec![
                    self.data_cell(&item.item_number),
                    self.data_cell(&item.description),
                    self.data_cell(device),
                    self.data_cell(&make_model),
                    self.data_cell(item.serial_number.as_deref().unwrap_or("-")),
                    self.data_cell(&location),
                    self.data_cell(item.image_format.as_deref().unwrap_or("-")),
                    self.data_cell(&item.condition),
                ]));
            }

            docx = docx.add_table(table);
        }

        if let Some(ref notes) = ev.documentation_notes {
            if !notes.is_empty() {
                docx = docx.add_paragraph(Paragraph::new());
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Documentation Notes: ").bold())
                        .add_run(Run::new().add_text(notes))
                );
            }
        }

        docx.add_paragraph(Paragraph::new()) // Spacer
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> ForensicReport {
        ForensicReport::builder()
            .case_number("2026-TEST")
            .examiner_name("Test Examiner")
            .build()
            .unwrap()
    }

    fn full_report() -> ForensicReport {
        let mut report = ForensicReport::builder()
            .case_number("2026-FULL")
            .examiner_name("Full Examiner")
            .build()
            .unwrap();

        report.executive_summary = Some("This is a summary".to_string());
        report.scope = Some("Scope of examination".to_string());
        report.methodology = Some("Methodology used".to_string());
        report.conclusions = Some("Conclusions reached".to_string());

        report.evidence_items.push(EvidenceItem {
            evidence_id: "E001".to_string(),
            description: "Test disk image".to_string(),
            evidence_type: EvidenceType::ForensicImage,
            acquisition_hashes: vec![HashRecord {
                item: "E001".to_string(),
                algorithm: HashAlgorithm::SHA256,
                value: "abcdef1234567890".to_string(),
                computed_at: None,
                verified: Some(true),
            }],
            ..Default::default()
        });

        report.findings.push(Finding::new(
            "F001",
            "Test finding",
            "Description of finding",
        ).with_severity(FindingSeverity::High));

        report.timeline.push(TimelineEvent {
            description: "User logged in".to_string(),
            source: "EventLog".to_string(),
            evidence_id: Some("E001".to_string()),
            ..Default::default()
        });

        report.tools.push(ToolInfo {
            name: "CORE-FFX".to_string(),
            version: "0.1.0".to_string(),
            purpose: Some("Evidence examination".to_string()),
            vendor: None,
        });

        report
    }

    #[test]
    fn test_docx_generator_new() {
        let gen = DocxGenerator::new();
        let _ = gen;
    }

    #[test]
    fn test_docx_generator_default() {
        let gen = DocxGenerator::default();
        let _ = gen;
    }

    #[test]
    fn test_docx_generate_minimal_report() {
        let gen = DocxGenerator::new();
        let report = sample_report();

        let dir = tempfile::tempdir().unwrap();
        let out_path = dir.path().join("test_report.docx");

        let result = gen.generate(&report, &out_path);
        assert!(result.is_ok(), "generate failed: {:?}", result.err());
        assert!(out_path.exists());

        let metadata = std::fs::metadata(&out_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_docx_generate_full_report() {
        let gen = DocxGenerator::new();
        let report = full_report();

        let dir = tempfile::tempdir().unwrap();
        let out_path = dir.path().join("full_report.docx");

        let result = gen.generate(&report, &out_path);
        assert!(result.is_ok(), "full generate failed: {:?}", result.err());
        assert!(out_path.exists());

        let metadata = std::fs::metadata(&out_path).unwrap();
        assert!(metadata.len() > 100, "Full report should be larger than 100 bytes");
    }

    #[test]
    fn test_docx_generate_invalid_path_returns_error() {
        let gen = DocxGenerator::new();
        let report = sample_report();

        let result = gen.generate(&report, "/nonexistent/path/report.docx");
        assert!(result.is_err());
    }

    #[test]
    fn test_docx_output_is_valid_zip() {
        let gen = DocxGenerator::new();
        let report = sample_report();

        let dir = tempfile::tempdir().unwrap();
        let out_path = dir.path().join("test.docx");
        gen.generate(&report, &out_path).unwrap();

        // DOCX files are ZIP archives; first 2 bytes should be PK (0x50, 0x4B)
        let data = std::fs::read(&out_path).unwrap();
        assert!(data.len() >= 4);
        assert_eq!(data[0], 0x50, "Expected PK magic byte 1");
        assert_eq!(data[1], 0x4B, "Expected PK magic byte 2");
    }

    #[test]
    fn test_docx_full_report_larger_than_minimal() {
        let gen = DocxGenerator::new();
        let dir = tempfile::tempdir().unwrap();

        let min_path = dir.path().join("minimal.docx");
        gen.generate(&sample_report(), &min_path).unwrap();

        let full_path = dir.path().join("full.docx");
        gen.generate(&full_report(), &full_path).unwrap();

        let min_size = std::fs::metadata(&min_path).unwrap().len();
        let full_size = std::fs::metadata(&full_path).unwrap().len();
        assert!(full_size > min_size, "Full report ({}) should be larger than minimal ({})", full_size, min_size);
    }

    #[test]
    fn test_docx_generate_report_with_all_classifications() {
        let gen = DocxGenerator::new();
        let dir = tempfile::tempdir().unwrap();

        for classification in [
            Classification::Public,
            Classification::LawEnforcementSensitive,
            Classification::Confidential,
        ] {
            let mut report = sample_report();
            report.metadata.classification = classification;

            let path = dir.path().join(format!("report_{}.docx", report.metadata.classification.as_str()));
            let result = gen.generate(&report, &path);
            assert!(result.is_ok(), "Classification {:?} failed: {:?}", report.metadata.classification, result.err());
        }
    }
}
