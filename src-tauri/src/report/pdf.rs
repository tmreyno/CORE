// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! PDF report generator using genpdf
//!
//! Generates professional PDF reports from forensic report data.

use std::path::Path;
use genpdf::{
    elements::{Break, LinearLayout, Paragraph, TableLayout, Text},
    fonts, style, Alignment, Document, Element,
};

use super::error::{ReportError, ReportResult};
use super::types::*;

/// PDF generator for forensic reports
pub struct PdfGenerator {
    /// Font family for the document
    #[allow(dead_code)]
    font_family: Option<fonts::FontFamily<fonts::FontData>>,
}

impl PdfGenerator {
    /// Create a new PDF generator
    pub fn new() -> Self {
        Self { font_family: None }
    }

    /// Create a PDF generator with a custom font
    pub fn with_font(_font_path: impl AsRef<Path>) -> ReportResult<Self> {
        // Load custom font if provided
        // For now, we'll use the default font
        Ok(Self { font_family: None })
    }

    /// Try to load fonts from various locations
    fn load_fonts() -> ReportResult<fonts::FontFamily<fonts::FontData>> {
        // Try bundled fonts first
        if let Ok(font) = fonts::from_files("./fonts", "LiberationSans", None) {
            return Ok(font);
        }
        
        // macOS system fonts - use Arial as it's commonly available
        if let Ok(font) = fonts::from_files("/Library/Fonts", "Arial", None) {
            return Ok(font);
        }
        
        // Try Times New Roman on macOS
        if let Ok(font) = fonts::from_files("/Library/Fonts", "Times New Roman", None) {
            return Ok(font);
        }
        
        // Linux Liberation fonts
        if let Ok(font) = fonts::from_files("/usr/share/fonts/truetype/liberation", "LiberationSans", None) {
            return Ok(font);
        }
        
        // Linux DejaVu fonts
        if let Ok(font) = fonts::from_files("/usr/share/fonts/truetype/dejavu", "DejaVuSans", None) {
            return Ok(font);
        }
        
        // Windows fonts
        if let Ok(font) = fonts::from_files("C:\\Windows\\Fonts", "arial", None) {
            return Ok(font);
        }
        
        Err(ReportError::Pdf(
            "No suitable fonts found. PDF generation requires TrueType fonts. \
            Please ensure fonts are available in ./fonts/, /Library/Fonts/ (macOS), \
            or /usr/share/fonts/truetype/ (Linux).".to_string()
        ))
    }

    /// Generate a PDF report
    pub fn generate(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> ReportResult<()> {
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
        
        if !report.evidence_items.is_empty() {
            self.add_evidence_section(&mut doc, report)?;
        }
        
        if !report.findings.is_empty() {
            self.add_findings_section(&mut doc, report)?;
        }
        
        if !report.timeline.is_empty() {
            self.add_timeline_section(&mut doc, report)?;
        }
        
        if !report.tools.is_empty() {
            self.add_tools_section(&mut doc, report)?;
        }
        
        if report.conclusions.is_some() {
            self.add_conclusions(&mut doc, report)?;
        }
        
        self.add_footer(&mut doc, report)?;

        // Render to file
        doc.render_to_file(output_path)
            .map_err(|e| ReportError::Pdf(e.to_string()))?;

        Ok(())
    }

    fn add_header(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
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
        
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_case_info(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
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
        
        if !report.examiner.certifications.is_empty() {
            let certs = report.examiner.certifications.join(", ");
            examiner_layout.push(self.info_row("Certifications:", &certs));
        }
        
        doc.push(examiner_layout);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_executive_summary(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
        self.add_section_header(doc, "Executive Summary");
        
        if let Some(ref summary) = report.executive_summary {
            doc.push(Paragraph::new(summary).styled(style::Style::new().with_font_size(10)));
        }
        
        doc.push(Break::new(1.0));
        Ok(())
    }

    fn add_evidence_section(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
        self.add_section_header(doc, "Evidence Examined");
        
        // Create table
        let mut table = TableLayout::new(vec![1, 3, 2, 2]);
        table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
        
        // Header row
        table.row()
            .element(Text::new("ID").styled(style::Style::new().bold()))
            .element(Text::new("Description").styled(style::Style::new().bold()))
            .element(Text::new("Type").styled(style::Style::new().bold()))
            .element(Text::new("Serial").styled(style::Style::new().bold()))
            .push()
            .map_err(|e| ReportError::Pdf(e.to_string()))?;
        
        // Data rows
        for item in &report.evidence_items {
            table.row()
                .element(Text::new(&item.evidence_id))
                .element(Text::new(&item.description))
                .element(Text::new(item.evidence_type.as_str()))
                .element(Text::new(item.serial_number.as_deref().unwrap_or("-")))
                .push()
                .map_err(|e| ReportError::Pdf(e.to_string()))?;
        }
        
        doc.push(table);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_findings_section(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
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
            
            doc.push(Break::new(0.75));
        }
        
        Ok(())
    }

    fn add_timeline_section(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
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
            .map_err(|e| ReportError::Pdf(e.to_string()))?;
        
        for event in &report.timeline {
            let ts = event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            table.row()
                .element(Text::new(ts))
                .element(Text::new(&event.timestamp_type))
                .element(Text::new(&event.description))
                .element(Text::new(&event.source))
                .push()
                .map_err(|e| ReportError::Pdf(e.to_string()))?;
        }
        
        doc.push(table);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_tools_section(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
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
            .map_err(|e| ReportError::Pdf(e.to_string()))?;
        
        for tool in &report.tools {
            table.row()
                .element(Text::new(&tool.name))
                .element(Text::new(&tool.version))
                .element(Text::new(tool.vendor.as_deref().unwrap_or("-")))
                .element(Text::new(tool.purpose.as_deref().unwrap_or("-")))
                .push()
                .map_err(|e| ReportError::Pdf(e.to_string()))?;
        }
        
        doc.push(table);
        doc.push(Break::new(1.0));
        
        Ok(())
    }

    fn add_conclusions(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
        self.add_section_header(doc, "Conclusions");
        
        if let Some(ref conclusions) = report.conclusions {
            doc.push(Paragraph::new(conclusions).styled(style::Style::new().with_font_size(10)));
        }
        
        doc.push(Break::new(1.0));
        Ok(())
    }

    fn add_footer(&self, doc: &mut Document, report: &ForensicReport) -> ReportResult<()> {
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

impl Default for PdfGenerator {
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
            .case_number("2026-PDF")
            .examiner_name("PDF Examiner")
            .build()
            .unwrap()
    }

    #[test]
    fn test_pdf_generator_new() {
        let gen = PdfGenerator::new();
        assert!(gen.font_family.is_none());
    }

    #[test]
    fn test_pdf_generator_default() {
        let gen = PdfGenerator::default();
        assert!(gen.font_family.is_none());
    }

    #[test]
    fn test_pdf_generator_with_font_nonexistent() {
        let result = PdfGenerator::with_font("/nonexistent/font.ttf");
        // with_font currently always returns Ok (stub)
        assert!(result.is_ok());
    }

    #[test]
    fn test_pdf_load_fonts_succeeds_on_macos() {
        // This test validates that font loading works on macOS where
        // system fonts (/Library/Fonts/Arial) are typically available.
        // It will be skipped on systems without suitable fonts.
        let result = PdfGenerator::load_fonts();
        if result.is_err() {
            eprintln!("Skipping PDF font test: no suitable fonts found (CI environment)");
            return;
        }
        // If we get here, fonts loaded successfully
        let _font = result.unwrap();
    }

    #[test]
    fn test_pdf_generate_minimal_report() {
        // PDF generation requires fonts. Skip gracefully if unavailable.
        if PdfGenerator::load_fonts().is_err() {
            eprintln!("Skipping PDF generate test: no fonts available");
            return;
        }

        let gen = PdfGenerator::new();
        let report = sample_report();

        let dir = tempfile::tempdir().unwrap();
        let out_path = dir.path().join("test_report.pdf");

        let result = gen.generate(&report, &out_path);
        assert!(result.is_ok(), "PDF generate failed: {:?}", result.err());
        assert!(out_path.exists());

        // PDF files start with %PDF
        let data = std::fs::read(&out_path).unwrap();
        assert!(data.len() >= 5);
        assert_eq!(&data[0..5], b"%PDF-", "Expected PDF magic header");
    }

    #[test]
    fn test_pdf_generate_invalid_path_returns_error() {
        // PDF generation requires fonts. Skip gracefully if unavailable.
        if PdfGenerator::load_fonts().is_err() {
            eprintln!("Skipping PDF error path test: no fonts available");
            return;
        }

        let gen = PdfGenerator::new();
        let report = sample_report();

        let result = gen.generate(&report, "/nonexistent/dir/report.pdf");
        assert!(result.is_err());
    }
}
