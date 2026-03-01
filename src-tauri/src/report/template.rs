// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Template engine for report generation
//!
//! Uses Tera templates (Jinja2-like) for flexible report formatting.

use std::path::Path;
use tera::{Context, Tera};

use super::error::ReportResult;
use super::types::ForensicReport;
use crate::common::hex::format_size_compact;

/// Default HTML report template
const DEFAULT_HTML_TEMPLATE: &str = include_str!("templates/report.html");

/// Default Markdown report template  
const DEFAULT_MARKDOWN_TEMPLATE: &str = include_str!("templates/report.md");

/// Template engine for rendering reports
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Create a new template engine with default templates
    pub fn new() -> ReportResult<Self> {
        let mut tera = Tera::default();

        // Add default templates
        tera.add_raw_template("report.html", DEFAULT_HTML_TEMPLATE)?;
        tera.add_raw_template("report.md", DEFAULT_MARKDOWN_TEMPLATE)?;

        // Register custom filters
        Self::register_filters(&mut tera);

        Ok(Self { tera })
    }

    /// Create a template engine loading templates from a directory
    pub fn with_directory(template_dir: impl AsRef<Path>) -> ReportResult<Self> {
        let glob_pattern = format!("{}/**/*", template_dir.as_ref().display());
        let mut tera = Tera::new(&glob_pattern)?;

        // Also add default templates as fallbacks
        if !tera.get_template_names().any(|n| n == "report.html") {
            tera.add_raw_template("report.html", DEFAULT_HTML_TEMPLATE)?;
        }
        if !tera.get_template_names().any(|n| n == "report.md") {
            tera.add_raw_template("report.md", DEFAULT_MARKDOWN_TEMPLATE)?;
        }

        Self::register_filters(&mut tera);

        Ok(Self { tera })
    }

    /// Register custom Tera filters
    fn register_filters(tera: &mut Tera) {
        // Format bytes as human readable
        tera.register_filter(
            "bytes",
            |value: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| {
                if let Some(n) = value.as_u64() {
                    Ok(tera::Value::String(format_size_compact(n)))
                } else {
                    Ok(value.clone())
                }
            },
        );

        // Format classification as badge
        tera.register_filter(
            "classification_badge",
            |value: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| {
                if let Some(s) = value.as_str() {
                    let class = match s {
                        "Public" => "badge-public",
                        "Internal" => "badge-internal",
                        "Confidential" => "badge-confidential",
                        "Restricted" => "badge-restricted",
                        "LawEnforcementSensitive" => "badge-les",
                        _ => "badge-default",
                    };
                    Ok(tera::Value::String(format!(
                        r#"<span class="{}">{}</span>"#,
                        class, s
                    )))
                } else {
                    Ok(value.clone())
                }
            },
        );

        // Severity color
        tera.register_filter(
            "severity_color",
            |value: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| {
                if let Some(s) = value.as_str() {
                    let color = match s {
                        "Info" => "#6c757d",
                        "Low" => "#28a745",
                        "Medium" => "#ffc107",
                        "High" => "#fd7e14",
                        "Critical" => "#dc3545",
                        _ => "#000000",
                    };
                    Ok(tera::Value::String(color.to_string()))
                } else {
                    Ok(value.clone())
                }
            },
        );
    }

    /// Add a custom template
    pub fn add_template(&mut self, name: &str, content: &str) -> ReportResult<()> {
        self.tera.add_raw_template(name, content)?;
        Ok(())
    }

    /// Render a report to HTML
    pub fn render_html(&self, report: &ForensicReport) -> ReportResult<String> {
        self.render("report.html", report)
    }

    /// Render a report to Markdown
    pub fn render_markdown(&self, report: &ForensicReport) -> ReportResult<String> {
        self.render("report.md", report)
    }

    /// Render a report using a specific template
    pub fn render(&self, template_name: &str, report: &ForensicReport) -> ReportResult<String> {
        let context = self.build_context(report)?;
        let rendered = self.tera.render(template_name, &context)?;
        Ok(rendered)
    }

    /// Build Tera context from report
    fn build_context(&self, report: &ForensicReport) -> ReportResult<Context> {
        let mut context = Context::new();

        // Serialize report to JSON and insert into context
        let report_json = serde_json::to_value(report)?;
        context.insert("report", &report_json);

        // Also add top-level shortcuts for convenience
        context.insert("metadata", &report.metadata);
        context.insert("case", &report.case_info);
        context.insert("examiner", &report.examiner);
        context.insert("evidence_items", &report.evidence_items);
        context.insert("findings", &report.findings);
        context.insert("timeline", &report.timeline);
        context.insert("tools", &report.tools);
        context.insert("appendices", &report.appendices);

        // Add helper values
        context.insert("current_year", &chrono::Utc::now().format("%Y").to_string());
        context.insert("generated_at", &chrono::Utc::now().to_rfc3339());

        Ok(context)
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<&str> {
        self.tera.get_template_names().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_compact() {
        // Tests for the canonical format_size_compact from common::hex
        assert_eq!(format_size_compact(500), "500 bytes");
        assert_eq!(format_size_compact(1024), "1.00 KB");
        assert_eq!(format_size_compact(1024 * 1024), "1.00 MB");
        assert_eq!(format_size_compact(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.is_ok());
    }
}
