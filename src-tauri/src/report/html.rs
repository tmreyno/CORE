// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! HTML report generator
//!
//! Generates professional HTML reports from forensic report data.
//! The output is a self-contained HTML file with embedded CSS.

use std::path::Path;

use super::error::ReportResult;
use super::types::*;

/// HTML generator for forensic reports
pub struct HtmlGenerator {
    /// Include print-optimized styles
    print_styles: bool,
    /// Custom CSS to include
    custom_css: Option<String>,
}

impl HtmlGenerator {
    /// Create a new HTML generator with default settings
    pub fn new() -> Self {
        Self {
            print_styles: true,
            custom_css: None,
        }
    }

    /// Create an HTML generator with custom CSS
    pub fn with_css(css: impl Into<String>) -> Self {
        Self {
            print_styles: true,
            custom_css: Some(css.into()),
        }
    }

    /// Enable or disable print-optimized styles
    pub fn print_styles(mut self, enabled: bool) -> Self {
        self.print_styles = enabled;
        self
    }

    /// Generate an HTML report file
    pub fn generate(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> ReportResult<()> {
        let html = self.render_html(report);
        std::fs::write(output_path, html)?;
        Ok(())
    }

    /// Render the report to an HTML string
    pub fn render_html(&self, report: &ForensicReport) -> String {
        let mut html = String::new();

        // HTML header and styles
        html.push_str(&self.render_header(report));

        // Body content
        html.push_str("<body>\n");
        html.push_str(&self.render_classification_banner(report));
        html.push_str(&self.render_title_page(report));
        html.push_str(&self.render_toc(report));
        html.push_str(&self.render_case_info(report));

        if report.executive_summary.is_some() {
            html.push_str(&self.render_executive_summary(report));
        }

        if report.scope.is_some() {
            html.push_str(&self.render_scope(report));
        }

        if report.methodology.is_some() {
            html.push_str(&self.render_methodology(report));
        }

        if !report.evidence_items.is_empty() {
            html.push_str(&self.render_evidence_section(report));
        }

        if report.evidence_collection.is_some() {
            html.push_str(&self.render_evidence_collection(report));
        }

        if !report.chain_of_custody.is_empty() {
            html.push_str(&self.render_chain_of_custody(report));
        }

        if !report.findings.is_empty() {
            html.push_str(&self.render_findings_section(report));
        }

        if !report.timeline.is_empty() {
            html.push_str(&self.render_timeline_section(report));
        }

        if !report.hash_records.is_empty() {
            html.push_str(&self.render_hash_section(report));
        }

        if !report.tools.is_empty() {
            html.push_str(&self.render_tools_section(report));
        }

        if report.conclusions.is_some() {
            html.push_str(&self.render_conclusions(report));
        }

        if !report.appendices.is_empty() {
            html.push_str(&self.render_appendices(report));
        }

        html.push_str(&self.render_footer(report));
        html.push_str("</body>\n</html>");

        html
    }

    /// Render HTML document header with embedded styles
    fn render_header(&self, report: &ForensicReport) -> String {
        let classification_color = match report.metadata.classification {
            Classification::Public => "#28a745",
            Classification::Internal => "#007bff",
            Classification::Confidential => "#fd7e14",
            Classification::Restricted => "#dc3545",
            Classification::LawEnforcementSensitive => "#6f42c1",
        };

        let mut css = format!(r##"
:root {{
    --primary-color: #2c3e50;
    --secondary-color: #34495e;
    --accent-color: #3498db;
    --classification-color: {classification_color};
    --border-color: #ddd;
    --bg-light: #f8f9fa;
    --text-color: #333;
}}

* {{
    box-sizing: border-box;
}}

body {{
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
    background: #fff;
}}

h1, h2, h3, h4 {{
    color: var(--primary-color);
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}}

h1 {{ font-size: 2.2em; border-bottom: 3px solid var(--accent-color); padding-bottom: 0.3em; }}
h2 {{ font-size: 1.8em; border-bottom: 2px solid var(--border-color); padding-bottom: 0.2em; }}
h3 {{ font-size: 1.4em; }}

.classification-banner {{
    background: var(--classification-color);
    color: white;
    text-align: center;
    padding: 8px;
    font-weight: bold;
    position: sticky;
    top: 0;
    z-index: 1000;
    margin: -20px -20px 20px -20px;
}}

.title-page {{
    text-align: center;
    padding: 60px 20px;
    border-bottom: 2px solid var(--border-color);
    margin-bottom: 40px;
}}

.title-page h1 {{
    font-size: 2.5em;
    border: none;
    margin-bottom: 10px;
}}

.title-page .subtitle {{
    font-size: 1.2em;
    color: var(--secondary-color);
}}

.title-page .report-meta {{
    margin-top: 30px;
    font-size: 1.1em;
}}

.toc {{
    background: var(--bg-light);
    padding: 20px 30px;
    border-radius: 8px;
    margin-bottom: 40px;
}}

.toc h2 {{
    margin-top: 0;
    border: none;
}}

.toc ul {{
    list-style: none;
    padding-left: 0;
}}

.toc li {{
    padding: 5px 0;
}}

.toc a {{
    color: var(--accent-color);
    text-decoration: none;
}}

.toc a:hover {{
    text-decoration: underline;
}}

.section {{
    margin-bottom: 40px;
    page-break-inside: avoid;
}}

.info-box {{
    background: var(--bg-light);
    border-left: 4px solid var(--accent-color);
    padding: 15px 20px;
    margin: 20px 0;
    border-radius: 0 8px 8px 0;
}}

.info-box h3 {{
    margin-top: 0;
    color: var(--accent-color);
}}

table {{
    width: 100%;
    border-collapse: collapse;
    margin: 20px 0;
}}

th, td {{
    padding: 12px;
    text-align: left;
    border: 1px solid var(--border-color);
}}

th {{
    background: var(--primary-color);
    color: white;
    font-weight: 600;
}}

tr:nth-child(even) {{
    background: var(--bg-light);
}}

tr:hover {{
    background: #e9ecef;
}}

.severity-badge {{
    display: inline-block;
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 0.85em;
    font-weight: bold;
    text-transform: uppercase;
}}

.severity-critical {{ background: #dc3545; color: white; }}
.severity-high {{ background: #fd7e14; color: white; }}
.severity-medium {{ background: #ffc107; color: #333; }}
.severity-low {{ background: #28a745; color: white; }}
.severity-informational {{ background: #6c757d; color: white; }}

.hash-value {{
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 0.9em;
    background: #f4f4f4;
    padding: 2px 6px;
    border-radius: 3px;
    word-break: break-all;
}}

.footer {{
    margin-top: 60px;
    padding-top: 20px;
    border-top: 2px solid var(--border-color);
    text-align: center;
    color: var(--secondary-color);
    font-size: 0.9em;
}}

.finding {{
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 20px;
    margin: 20px 0;
    background: white;
}}

.finding-header {{
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
}}

.finding h3 {{
    margin: 0;
}}

.evidence-item {{
    background: var(--bg-light);
    border-radius: 8px;
    padding: 20px;
    margin: 15px 0;
}}

.evidence-item h4 {{
    margin-top: 0;
    color: var(--primary-color);
}}

.timeline-event {{
    display: flex;
    margin: 15px 0;
    padding-left: 20px;
    border-left: 3px solid var(--accent-color);
}}

.timeline-time {{
    min-width: 180px;
    font-weight: bold;
    color: var(--secondary-color);
}}
"##);

        // Add print styles if enabled
        if self.print_styles {
            css.push_str(r##"
@media print {{
    .classification-banner {{
        position: static;
        margin: 0 0 20px 0;
    }}
    
    body {{
        max-width: none;
        padding: 0;
    }}
    
    .section {{
        page-break-inside: avoid;
    }}
    
    h2 {{
        page-break-after: avoid;
    }}
    
    table {{
        page-break-inside: avoid;
    }}
    
    .toc {{
        page-break-after: always;
    }}
}}
"##);
        }

        // Add custom CSS if provided
        if let Some(ref custom) = self.custom_css {
            css.push_str("\n/* Custom CSS */\n");
            css.push_str(custom);
        }

        format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="generator" content="FFX Forensic File Xplorer">
    <meta name="classification" content="{}">
    <title>{} - {}</title>
    <style>{}</style>
</head>
"##,
            report.metadata.classification.as_str(),
            Self::escape_html(&report.metadata.title),
            Self::escape_html(&report.case_info.case_number),
            css
        )
    }

    /// Render classification banner
    fn render_classification_banner(&self, report: &ForensicReport) -> String {
        format!(
            "<div class=\"classification-banner\">{}</div>\n",
            report.metadata.classification.as_str()
        )
    }

    /// Render title page
    fn render_title_page(&self, report: &ForensicReport) -> String {
        let case_name = report.case_info.case_name.as_deref().unwrap_or("");
        let organization = report.examiner.organization.as_deref().unwrap_or("");

        format!(r##"<div class="title-page">
    <h1>{}</h1>
    <div class="subtitle">{}</div>
    <div class="report-meta">
        <p><strong>Case Number:</strong> {}</p>
        <p><strong>Report Number:</strong> {}</p>
        <p><strong>Examiner:</strong> {}</p>
        {}
        <p><strong>Date:</strong> {}</p>
    </div>
</div>
"##,
            Self::escape_html(&report.metadata.title),
            Self::escape_html(case_name),
            Self::escape_html(&report.case_info.case_number),
            Self::escape_html(&report.metadata.report_number),
            Self::escape_html(&report.examiner.name),
            if !organization.is_empty() {
                format!("<p><strong>Organization:</strong> {}</p>", Self::escape_html(organization))
            } else {
                String::new()
            },
            report.metadata.generated_at.format("%B %d, %Y")
        )
    }

    /// Render table of contents
    fn render_toc(&self, report: &ForensicReport) -> String {
        let mut toc = String::from("<div class=\"toc\">\n<h2>Table of Contents</h2>\n<ul>\n");

        toc.push_str("<li><a href=\"#case-info\">Case Information</a></li>\n");

        if report.executive_summary.is_some() {
            toc.push_str("<li><a href=\"#executive-summary\">Executive Summary</a></li>\n");
        }
        if report.scope.is_some() {
            toc.push_str("<li><a href=\"#scope\">Scope of Examination</a></li>\n");
        }
        if report.methodology.is_some() {
            toc.push_str("<li><a href=\"#methodology\">Methodology</a></li>\n");
        }
        if !report.evidence_items.is_empty() {
            toc.push_str("<li><a href=\"#evidence\">Evidence Examined</a></li>\n");
        }
        if !report.chain_of_custody.is_empty() {
            toc.push_str("<li><a href=\"#chain-of-custody\">Chain of Custody</a></li>\n");
        }
        if !report.findings.is_empty() {
            toc.push_str("<li><a href=\"#findings\">Findings</a></li>\n");
        }
        if !report.timeline.is_empty() {
            toc.push_str("<li><a href=\"#timeline\">Timeline</a></li>\n");
        }
        if !report.hash_records.is_empty() {
            toc.push_str("<li><a href=\"#hashes\">Hash Verification</a></li>\n");
        }
        if !report.tools.is_empty() {
            toc.push_str("<li><a href=\"#tools\">Tools Used</a></li>\n");
        }
        if report.conclusions.is_some() {
            toc.push_str("<li><a href=\"#conclusions\">Conclusions</a></li>\n");
        }
        if !report.appendices.is_empty() {
            toc.push_str("<li><a href=\"#appendices\">Appendices</a></li>\n");
        }

        toc.push_str("</ul>\n</div>\n");
        toc
    }

    /// Render case information section
    fn render_case_info(&self, report: &ForensicReport) -> String {
        let mut html = String::from("<div class=\"section\" id=\"case-info\">\n<h2>Case Information</h2>\n");
        html.push_str("<div class=\"info-box\">\n<table>\n");

        html.push_str(&format!(
            "<tr><th>Case Number</th><td>{}</td></tr>\n",
            Self::escape_html(&report.case_info.case_number)
        ));

        if let Some(ref name) = report.case_info.case_name {
            html.push_str(&format!(
                "<tr><th>Case Name</th><td>{}</td></tr>\n",
                Self::escape_html(name)
            ));
        }

        if let Some(ref agency) = report.case_info.agency {
            html.push_str(&format!(
                "<tr><th>Agency</th><td>{}</td></tr>\n",
                Self::escape_html(agency)
            ));
        }

        if let Some(ref requestor) = report.case_info.requestor {
            html.push_str(&format!(
                "<tr><th>Requestor</th><td>{}</td></tr>\n",
                Self::escape_html(requestor)
            ));
        }

        // Examiner info
        html.push_str(&format!(
            "<tr><th>Examiner</th><td>{}</td></tr>\n",
            Self::escape_html(&report.examiner.name)
        ));

        if let Some(ref title) = report.examiner.title {
            html.push_str(&format!(
                "<tr><th>Title</th><td>{}</td></tr>\n",
                Self::escape_html(title)
            ));
        }

        if let Some(ref org) = report.examiner.organization {
            html.push_str(&format!(
                "<tr><th>Organization</th><td>{}</td></tr>\n",
                Self::escape_html(org)
            ));
        }

        html.push_str("</table>\n</div>\n</div>\n");
        html
    }

    /// Render executive summary section
    fn render_executive_summary(&self, report: &ForensicReport) -> String {
        if let Some(ref summary) = report.executive_summary {
            format!(
                "<div class=\"section\" id=\"executive-summary\">\n<h2>Executive Summary</h2>\n<p>{}</p>\n</div>\n",
                Self::escape_html(summary).replace('\n', "<br>\n")
            )
        } else {
            String::new()
        }
    }

    /// Render scope section
    fn render_scope(&self, report: &ForensicReport) -> String {
        if let Some(ref scope) = report.scope {
            format!(
                "<div class=\"section\" id=\"scope\">\n<h2>Scope of Examination</h2>\n<p>{}</p>\n</div>\n",
                Self::escape_html(scope).replace('\n', "<br>\n")
            )
        } else {
            String::new()
        }
    }

    /// Render methodology section
    fn render_methodology(&self, report: &ForensicReport) -> String {
        if let Some(ref methodology) = report.methodology {
            format!(
                "<div class=\"section\" id=\"methodology\">\n<h2>Methodology</h2>\n<p>{}</p>\n</div>\n",
                Self::escape_html(methodology).replace('\n', "<br>\n")
            )
        } else {
            String::new()
        }
    }

    /// Render evidence section
    fn render_evidence_section(&self, report: &ForensicReport) -> String {
        let mut html = String::from("<div class=\"section\" id=\"evidence\">\n<h2>Evidence Examined</h2>\n");

        for item in &report.evidence_items {
            html.push_str("<div class=\"evidence-item\">\n");
            html.push_str(&format!(
                "<h4>{} - {}</h4>\n",
                Self::escape_html(&item.evidence_id),
                Self::escape_html(&item.description)
            ));

            html.push_str("<table>\n");
            html.push_str(&format!(
                "<tr><th>Type</th><td>{}</td></tr>\n",
                item.evidence_type.as_str()
            ));

            if let Some(ref make) = item.make {
                html.push_str(&format!(
                    "<tr><th>Make</th><td>{}</td></tr>\n",
                    Self::escape_html(make)
                ));
            }

            if let Some(ref model) = item.model {
                html.push_str(&format!(
                    "<tr><th>Model</th><td>{}</td></tr>\n",
                    Self::escape_html(model)
                ));
            }

            if let Some(ref serial) = item.serial_number {
                html.push_str(&format!(
                    "<tr><th>Serial Number</th><td>{}</td></tr>\n",
                    Self::escape_html(serial)
                ));
            }

            if let Some(ref capacity) = item.capacity {
                html.push_str(&format!(
                    "<tr><th>Capacity</th><td>{}</td></tr>\n",
                    Self::escape_html(capacity)
                ));
            }

            html.push_str("</table>\n");

            // Acquisition hashes
            if !item.acquisition_hashes.is_empty() {
                html.push_str("<h5>Hash Values</h5>\n<table>\n");
                html.push_str("<tr><th>Algorithm</th><th>Value</th></tr>\n");
                for hash in &item.acquisition_hashes {
                    html.push_str(&format!(
                        "<tr><td>{}</td><td class=\"hash-value\">{}</td></tr>\n",
                        hash.algorithm.as_str(),
                        Self::escape_html(&hash.value)
                    ));
                }
                html.push_str("</table>\n");
            }

            html.push_str("</div>\n");
        }

        html.push_str("</div>\n");
        html
    }

    /// Render chain of custody section
    fn render_chain_of_custody(&self, report: &ForensicReport) -> String {
        let mut html = String::from("<div class=\"section\" id=\"chain-of-custody\">\n<h2>Chain of Custody</h2>\n");
        html.push_str("<table>\n");
        html.push_str("<tr><th>Date/Time</th><th>Evidence ID</th><th>Released By</th><th>Received By</th><th>Purpose</th><th>Location</th><th>Notes</th></tr>\n");

        for record in &report.chain_of_custody {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                record.timestamp.format("%Y-%m-%d %H:%M"),
                Self::escape_html(&record.evidence_id),
                Self::escape_html(&record.released_by),
                Self::escape_html(&record.received_by),
                record.purpose.as_deref().map(Self::escape_html).unwrap_or_default(),
                record.location.as_deref().map(Self::escape_html).unwrap_or_default(),
                record.notes.as_deref().map(Self::escape_html).unwrap_or_default()
            ));
        }

        html.push_str("</table>\n</div>\n");
        html
    }

    /// Render evidence collection section
    fn render_evidence_collection(&self, report: &ForensicReport) -> String {
        let ev = match &report.evidence_collection {
            Some(ec) => ec,
            None => return String::new(),
        };

        let mut html = String::from("<div class=\"section\" id=\"evidence-collection\">\n<h2>Evidence Collection</h2>\n");

        // Header info
        html.push_str("<div class=\"info-grid\">\n");
        html.push_str(&format!(
            "<div class=\"info-item\"><span class=\"info-label\">Collection Date</span><span class=\"info-value\">{}</span></div>\n",
            Self::escape_html(&ev.collection_date)
        ));
        if let Some(ref sdt) = ev.system_date_time {
            html.push_str(&format!(
                "<div class=\"info-item\"><span class=\"info-label\">System Date/Time</span><span class=\"info-value\">{}</span></div>\n",
                Self::escape_html(sdt)
            ));
        }
        html.push_str(&format!(
            "<div class=\"info-item\"><span class=\"info-label\">Collecting Officer</span><span class=\"info-value\">{}</span></div>\n",
            Self::escape_html(&ev.collecting_officer)
        ));
        html.push_str(&format!(
            "<div class=\"info-item\"><span class=\"info-label\">Authorization</span><span class=\"info-value\">{}</span></div>\n",
            Self::escape_html(&ev.authorization)
        ));
        if let Some(ref ad) = ev.authorization_date {
            html.push_str(&format!(
                "<div class=\"info-item\"><span class=\"info-label\">Authorization Date</span><span class=\"info-value\">{}</span></div>\n",
                Self::escape_html(ad)
            ));
        }
        if let Some(ref aa) = ev.authorizing_authority {
            html.push_str(&format!(
                "<div class=\"info-item\"><span class=\"info-label\">Authorizing Authority</span><span class=\"info-value\">{}</span></div>\n",
                Self::escape_html(aa)
            ));
        }
        html.push_str("</div>\n");

        if !ev.witnesses.is_empty() {
            html.push_str(&format!(
                "<p><strong>Witnesses:</strong> {}</p>\n",
                Self::escape_html(&ev.witnesses.join(", "))
            ));
        }
        if let Some(ref cond) = ev.conditions {
            if !cond.is_empty() {
                html.push_str(&format!(
                    "<p><strong>Environmental Conditions:</strong> {}</p>\n",
                    Self::escape_html(cond)
                ));
            }
        }

        // Collected items
        if !ev.collected_items.is_empty() {
            html.push_str(&format!(
                "<h3>Collected Items ({})</h3>\n",
                ev.collected_items.len()
            ));
            html.push_str("<table>\n");
            html.push_str("<tr><th>Item #</th><th>Description</th><th>Device Type</th><th>Make/Model</th><th>Serial #</th><th>Location</th><th>Format</th><th>Condition</th></tr>\n");

            for item in &ev.collected_items {
                let device = if !item.device_type.is_empty() {
                    Self::escape_html(&item.device_type)
                } else {
                    Self::escape_html(&item.item_type)
                };
                let make_model = format!(
                    "{}{}",
                    item.brand.as_deref().or(item.make.as_deref()).unwrap_or(""),
                    item.model.as_deref().map(|m| format!(" {}", m)).unwrap_or_default()
                );
                let location = if item.building.is_some() || item.room.is_some() {
                    format!(
                        "{}{}{}",
                        item.building.as_deref().unwrap_or(""),
                        item.room.as_deref().map(|r| format!(" / {}", r)).unwrap_or_default(),
                        item.location_other.as_deref().map(|l| format!(" / {}", l)).unwrap_or_default()
                    )
                } else {
                    item.found_location.clone()
                };
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    Self::escape_html(&item.item_number),
                    Self::escape_html(&item.description),
                    device,
                    Self::escape_html(&make_model),
                    item.serial_number.as_deref().map(Self::escape_html).unwrap_or_default(),
                    Self::escape_html(&location),
                    item.image_format.as_deref().map(Self::escape_html).unwrap_or_default(),
                    Self::escape_html(&item.condition),
                ));
            }

            html.push_str("</table>\n");
        }

        if let Some(ref notes) = ev.documentation_notes {
            if !notes.is_empty() {
                html.push_str(&format!(
                    "<p><strong>Documentation Notes:</strong> {}</p>\n",
                    Self::escape_html(notes)
                ));
            }
        }

        html.push_str("</div>\n");
        html
    }

    /// Render findings section
    fn render_findings_section(&self, report: &ForensicReport) -> String {
        let mut html = String::from("<div class=\"section\" id=\"findings\">\n<h2>Findings</h2>\n");

        for finding in &report.findings {
            let severity_class = match finding.severity {
                FindingSeverity::Critical => "severity-critical",
                FindingSeverity::High => "severity-high",
                FindingSeverity::Medium => "severity-medium",
                FindingSeverity::Low => "severity-low",
                FindingSeverity::Info => "severity-informational",
            };

            html.push_str("<div class=\"finding\">\n");
            html.push_str("<div class=\"finding-header\">\n");
            html.push_str(&format!(
                "<h3>{}: {}</h3>\n",
                Self::escape_html(&finding.finding_id),
                Self::escape_html(&finding.title)
            ));
            html.push_str(&format!(
                "<span class=\"severity-badge {}\">{}</span>\n",
                severity_class,
                finding.severity.as_str()
            ));
            html.push_str("</div>\n");

            html.push_str(&format!(
                "<p><strong>Category:</strong> {}</p>\n",
                finding.category.as_str()
            ));

            html.push_str(&format!(
                "<p>{}</p>\n",
                Self::escape_html(&finding.description).replace('\n', "<br>\n")
            ));

            if !finding.related_files.is_empty() {
                html.push_str("<p><strong>Related Files:</strong></p>\n<ul>\n");
                for file in &finding.related_files {
                    html.push_str(&format!("<li><code>{}</code></li>\n", Self::escape_html(file)));
                }
                html.push_str("</ul>\n");
            }

            if let Some(ref notes) = finding.notes {
                html.push_str(&format!(
                    "<p><strong>Notes:</strong> {}</p>\n",
                    Self::escape_html(notes)
                ));
            }

            html.push_str("</div>\n");
        }

        html.push_str("</div>\n");
        html
    }

    /// Render timeline section
    fn render_timeline_section(&self, report: &ForensicReport) -> String {
        let mut html = String::from("<div class=\"section\" id=\"timeline\">\n<h2>Timeline of Events</h2>\n");

        for event in &report.timeline {
            html.push_str("<div class=\"timeline-event\">\n");
            html.push_str(&format!(
                "<div class=\"timeline-time\">{}</div>\n",
                event.timestamp.format("%Y-%m-%d %H:%M:%S")
            ));
            html.push_str(&format!(
                "<div class=\"timeline-content\"><strong>{}:</strong> {} <em>(Source: {})</em></div>\n",
                Self::escape_html(&event.timestamp_type),
                Self::escape_html(&event.description),
                Self::escape_html(&event.source)
            ));
            html.push_str("</div>\n");
        }

        html.push_str("</div>\n");
        html
    }

    /// Render hash verification section
    fn render_hash_section(&self, report: &ForensicReport) -> String {
        let mut html = String::from("<div class=\"section\" id=\"hashes\">\n<h2>Hash Verification</h2>\n");
        html.push_str("<table>\n");
        html.push_str("<tr><th>Item</th><th>Algorithm</th><th>Hash Value</th><th>Verified</th></tr>\n");

        for record in &report.hash_records {
            let verified_str = record.verified.map_or("N/A", |v| if v { "✓ Yes" } else { "✗ No" });
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td class=\"hash-value\">{}</td><td>{}</td></tr>\n",
                Self::escape_html(&record.item),
                record.algorithm.as_str(),
                Self::escape_html(&record.value),
                verified_str
            ));
        }

        html.push_str("</table>\n</div>\n");
        html
    }

    /// Render tools section
    fn render_tools_section(&self, report: &ForensicReport) -> String {
        let mut html = String::from("<div class=\"section\" id=\"tools\">\n<h2>Tools Used</h2>\n");
        html.push_str("<table>\n");
        html.push_str("<tr><th>Tool</th><th>Version</th><th>Vendor</th><th>Purpose</th></tr>\n");

        for tool in &report.tools {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                Self::escape_html(&tool.name),
                Self::escape_html(&tool.version),
                tool.vendor.as_deref().map(Self::escape_html).unwrap_or_else(|| "-".to_string()),
                tool.purpose.as_deref().map(Self::escape_html).unwrap_or_else(|| "-".to_string())
            ));
        }

        html.push_str("</table>\n</div>\n");
        html
    }

    /// Render conclusions section
    fn render_conclusions(&self, report: &ForensicReport) -> String {
        if let Some(ref conclusions) = report.conclusions {
            format!(
                "<div class=\"section\" id=\"conclusions\">\n<h2>Conclusions</h2>\n<p>{}</p>\n</div>\n",
                Self::escape_html(conclusions).replace('\n', "<br>\n")
            )
        } else {
            String::new()
        }
    }

    /// Render appendices
    fn render_appendices(&self, report: &ForensicReport) -> String {
        if report.appendices.is_empty() {
            return String::new();
        }

        let mut html = String::from("<div class=\"section\" id=\"appendices\">\n<h2>Appendices</h2>\n");

        for (i, appendix) in report.appendices.iter().enumerate() {
            html.push_str(&format!(
                "<h3>Appendix {}: {}</h3>\n",
                (b'A' + i as u8) as char,
                Self::escape_html(&appendix.title)
            ));
            html.push_str(&format!(
                "<div class=\"info-box\">\n<p>{}</p>\n</div>\n",
                Self::escape_html(&appendix.content).replace('\n', "<br>\n")
            ));
        }

        html.push_str("</div>\n");
        html
    }

    /// Render footer
    fn render_footer(&self, report: &ForensicReport) -> String {
        format!(
            r##"<div class="footer">
    <p>This report was generated by <strong>{}</strong> on {} UTC.</p>
    <p><strong>{}</strong></p>
    <p>Report Version: {} | Report Number: {}</p>
</div>
"##,
            Self::escape_html(&report.metadata.generated_by),
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S"),
            report.metadata.classification.as_str(),
            Self::escape_html(&report.metadata.version),
            Self::escape_html(&report.metadata.report_number)
        )
    }

    /// Escape HTML special characters
    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
         .replace('<', "&lt;")
         .replace('>', "&gt;")
         .replace('"', "&quot;")
         .replace('\'', "&#39;")
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_html_escape() {
        assert_eq!(HtmlGenerator::escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(HtmlGenerator::escape_html("a & b"), "a &amp; b");
        assert_eq!(HtmlGenerator::escape_html("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_html_generation() {
        let report = ForensicReport {
            metadata: ReportMetadata {
                title: "Test Report".to_string(),
                report_number: "TEST-001".to_string(),
                version: "1.0".to_string(),
                classification: Classification::Confidential,
                generated_at: Utc::now(),
                generated_by: "Test".to_string(),
            },
            case_info: CaseInfo {
                case_number: "CASE-001".to_string(),
                ..Default::default()
            },
            examiner: ExaminerInfo {
                name: "Test Examiner".to_string(),
                ..Default::default()
            },
            executive_summary: Some("Test summary".to_string()),
            scope: None,
            methodology: None,
            evidence_items: vec![],
            chain_of_custody: vec![],
            findings: vec![],
            timeline: vec![],
            hash_records: vec![],
            tools: vec![],
            conclusions: None,
            appendices: vec![],
            signatures: vec![],
            notes: None,
            report_type: None,
            coc_items: None,
            evidence_collection: None,
        };

        let generator = HtmlGenerator::new();
        let html = generator.render_html(&report);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Report"));
        assert!(html.contains("CASE-001"));
        assert!(html.contains("Test Examiner"));
        assert!(html.contains("CONFIDENTIAL"));
    }

    #[test]
    fn test_html_generator_default() {
        let generator = HtmlGenerator::default();
        assert!(generator.print_styles);
        assert!(generator.custom_css.is_none());
    }

    #[test]
    fn test_html_generator_with_css() {
        let generator = HtmlGenerator::with_css("body { color: red; }");
        assert!(generator.custom_css.is_some());
        assert!(generator.custom_css.unwrap().contains("color: red"));
    }
}
