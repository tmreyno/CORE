// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Forensic Report Generator Module
//!
//! This module provides comprehensive report generation capabilities for forensic investigations.
//! It supports multiple output formats (PDF, DOCX) with template-based generation and optional
//! AI-assisted narrative writing.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Report Generator                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
//! │  │   Evidence   │───▶│  AI Writer   │───▶│   Template   │  │
//! │  │   Data       │    │  (Optional)  │    │    Engine    │  │
//! │  └──────────────┘    └──────────────┘    └──────────────┘  │
//! │         │                   │                    │          │
//! │         ▼                   ▼                    ▼          │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │                  Output Formats                       │  │
//! │  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐     │  │
//! │  │  │  PDF   │  │  DOCX  │  │  HTML  │  │ Typst  │     │  │
//! │  │  │genpdf  │  │docx-rs │  │  tera  │  │ future │     │  │
//! │  │  └────────┘  └────────┘  └────────┘  └────────┘     │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - **Template-based**: Use Tera templates for consistent report structure
//! - **Multi-format**: Export to PDF, DOCX, HTML, Markdown
//! - **AI-assisted** (optional): Generate narratives with LLM support
//! - **Extensible**: Easy to add Typst or other formats later
//!
//! # Usage
//!
//! ```rust,ignore
//! use report::{ForensicReport, ReportGenerator, OutputFormat};
//!
//! let report = ForensicReport::builder()
//!     .case_number("2026-001")
//!     .examiner("John Doe")
//!     .add_evidence(evidence_item)
//!     .add_finding(finding)
//!     .build()?;
//!
//! let generator = ReportGenerator::new();
//! generator.generate(&report, OutputFormat::Pdf, "output/report.pdf")?;
//! ```

pub mod types;
pub mod template;
pub mod pdf;
pub mod docx;
pub mod html;
pub mod markdown;
pub mod error;
pub mod commands;

#[cfg(feature = "ai-assistant")]
pub mod ai;

#[cfg(feature = "typst-reports")]
pub mod typst_gen;

// Re-exports for convenience
pub use types::*;
pub use template::TemplateEngine;
pub use pdf::PdfGenerator;
pub use docx::DocxGenerator;
pub use html::HtmlGenerator;
pub use markdown::MarkdownGenerator;
pub use error::{ReportError, ReportResult};

#[cfg(feature = "ai-assistant")]
pub use ai::AiAssistant;

#[cfg(feature = "typst-reports")]
pub use typst_gen::TypstGenerator;

use std::path::Path;

/// Supported output formats for report generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OutputFormat {
    /// PDF document (using genpdf)
    Pdf,
    /// Microsoft Word document (using docx-rs)
    Docx,
    /// HTML document (using Tera templates)
    Html,
    /// Markdown document
    Markdown,
    /// Typst document (future support)
    #[serde(rename = "typst")]
    Typst,
}

impl OutputFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Docx => "docx",
            OutputFormat::Html => "html",
            OutputFormat::Markdown => "md",
            OutputFormat::Typst => "typ",
        }
    }

    /// Check if this format is currently supported
    pub fn is_supported(&self) -> bool {
        #[cfg(feature = "typst-reports")]
        {
            matches!(self, OutputFormat::Pdf | OutputFormat::Docx | OutputFormat::Html | OutputFormat::Markdown | OutputFormat::Typst)
        }
        #[cfg(not(feature = "typst-reports"))]
        {
            matches!(self, OutputFormat::Pdf | OutputFormat::Docx | OutputFormat::Html | OutputFormat::Markdown)
        }
    }
}

/// Main report generator that coordinates all output formats
pub struct ReportGenerator {
    template_engine: TemplateEngine,
    pdf_generator: PdfGenerator,
    docx_generator: DocxGenerator,
    html_generator: HtmlGenerator,
    markdown_generator: MarkdownGenerator,
    #[cfg(feature = "typst-reports")]
    typst_generator: TypstGenerator,
    #[cfg(feature = "ai-assistant")]
    ai_assistant: Option<AiAssistant>,
}

impl ReportGenerator {
    /// Create a new ReportGenerator with default settings
    pub fn new() -> ReportResult<Self> {
        Ok(Self {
            template_engine: TemplateEngine::new()?,
            pdf_generator: PdfGenerator::new(),
            docx_generator: DocxGenerator::new(),
            html_generator: HtmlGenerator::new(),
            markdown_generator: MarkdownGenerator::new(),
            #[cfg(feature = "typst-reports")]
            typst_generator: TypstGenerator::new(),
            #[cfg(feature = "ai-assistant")]
            ai_assistant: None,
        })
    }

    /// Create a ReportGenerator with custom template directory
    pub fn with_templates(template_dir: impl AsRef<Path>) -> ReportResult<Self> {
        Ok(Self {
            template_engine: TemplateEngine::with_directory(template_dir)?,
            pdf_generator: PdfGenerator::new(),
            docx_generator: DocxGenerator::new(),
            html_generator: HtmlGenerator::new(),
            markdown_generator: MarkdownGenerator::new(),
            #[cfg(feature = "typst-reports")]
            typst_generator: TypstGenerator::new(),
            #[cfg(feature = "ai-assistant")]
            ai_assistant: None,
        })
    }

    #[cfg(feature = "ai-assistant")]
    /// Enable AI assistance with the specified provider
    pub fn with_ai(mut self, assistant: AiAssistant) -> Self {
        self.ai_assistant = Some(assistant);
        self
    }

    /// Generate a report in the specified format
    pub fn generate(
        &self,
        report: &ForensicReport,
        format: OutputFormat,
        output_path: impl AsRef<Path>,
    ) -> ReportResult<()> {
        match format {
            OutputFormat::Pdf => self.pdf_generator.generate(report, output_path),
            OutputFormat::Docx => self.docx_generator.generate(report, output_path),
            OutputFormat::Html => self.html_generator.generate(report, output_path),
            OutputFormat::Markdown => self.markdown_generator.generate(report, output_path),
            #[cfg(feature = "typst-reports")]
            OutputFormat::Typst => self.typst_generator.generate(report, output_path),
            #[cfg(not(feature = "typst-reports"))]
            OutputFormat::Typst => Err(ReportError::UnsupportedFormat(
                "Typst support requires the 'typst-reports' feature. Rebuild with: cargo build --features typst-reports".to_string()
            )),
        }
    }

    /// Generate HTML report using templates (legacy method, uses template engine)
    /// 
    /// Use this when you need custom templates instead of the built-in HtmlGenerator.
    #[allow(dead_code)] // Available for custom template workflows
    fn generate_html_template(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> ReportResult<()> {
        let html = self.template_engine.render_html(report)?;
        std::fs::write(output_path, html)?;
        Ok(())
    }

    /// Generate Markdown report using templates (legacy method, uses template engine)
    /// 
    /// Use this when you need custom templates instead of the built-in MarkdownGenerator.
    #[allow(dead_code)] // Available for custom template workflows
    fn generate_markdown_template(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> ReportResult<()> {
        let markdown = self.template_engine.render_markdown(report)?;
        std::fs::write(output_path, markdown)?;
        Ok(())
    }

    /// Get a reference to the template engine
    pub fn template_engine(&self) -> &TemplateEngine {
        &self.template_engine
    }

    /// Get a mutable reference to the template engine
    pub fn template_engine_mut(&mut self) -> &mut TemplateEngine {
        &mut self.template_engine
    }

    #[cfg(feature = "ai-assistant")]
    /// Generate AI-assisted narrative for a section
    pub async fn generate_narrative(
        &self,
        context: &str,
        section_type: NarrativeType,
    ) -> ReportResult<String> {
        match &self.ai_assistant {
            Some(ai) => ai.generate_narrative(context, section_type).await,
            None => Err(ReportError::AiNotConfigured),
        }
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default ReportGenerator")
    }
}

/// Types of narratives that can be AI-generated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NarrativeType {
    /// Executive summary for non-technical readers
    ExecutiveSummary,
    /// Detailed finding description
    FindingDescription,
    /// Timeline narrative
    TimelineNarrative,
    /// Evidence description
    EvidenceDescription,
    /// Methodology explanation
    Methodology,
    /// Conclusion
    Conclusion,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_extensions() {
        assert_eq!(OutputFormat::Pdf.extension(), "pdf");
        assert_eq!(OutputFormat::Docx.extension(), "docx");
        assert_eq!(OutputFormat::Html.extension(), "html");
        assert_eq!(OutputFormat::Markdown.extension(), "md");
        assert_eq!(OutputFormat::Typst.extension(), "typ");
    }

    #[test]
    fn test_output_format_support() {
        assert!(OutputFormat::Pdf.is_supported());
        assert!(OutputFormat::Docx.is_supported());
        assert!(OutputFormat::Html.is_supported());
        assert!(OutputFormat::Markdown.is_supported());
        assert!(!OutputFormat::Typst.is_supported());
    }
}
