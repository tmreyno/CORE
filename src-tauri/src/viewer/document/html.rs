// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! HTML document handler
//!
//! This module provides reading and writing capabilities for HTML documents.

use std::path::Path;

use super::error::DocumentResult;
use super::types::*;
use super::DocumentFormat;
use crate::report::ForensicReport;

/// HTML document handler
pub struct HtmlDocument {
    /// Include print-optimized styles
    print_styles: bool,
}

impl HtmlDocument {
    /// Create a new HTML document handler
    pub fn new() -> Self {
        Self {
            print_styles: true,
        }
    }

    // =========================================================================
    // READING
    // =========================================================================

    /// Read HTML from file path
    pub fn read(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let data = std::fs::read(path)?;
        self.read_bytes(&data)
    }

    /// Read HTML from bytes
    pub fn read_bytes(&self, data: &[u8]) -> DocumentResult<DocumentContent> {
        let html = String::from_utf8_lossy(data).to_string();
        self.parse_html(&html)
    }

    /// Parse HTML content
    fn parse_html(&self, html: &str) -> DocumentResult<DocumentContent> {
        let mut metadata = DocumentMetadata {
            format: DocumentFormat::Html,
            ..Default::default()
        };
        
        let mut elements = Vec::new();

        // Extract title from <title> tag
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start..].find("</title>") {
                let title = &html[start + 7..start + end];
                metadata.title = Some(Self::decode_html_entities(title));
            }
        }

        // Simple HTML parsing - extract text content
        let body_start = html.find("<body")
            .and_then(|i| html[i..].find('>').map(|j| i + j + 1))
            .unwrap_or(0);
        let body_end = html.rfind("</body>").unwrap_or(html.len());
        let body = &html[body_start..body_end];

        // Parse common elements
        let mut remaining = body;
        while !remaining.is_empty() {
            // Find next tag
            if let Some(tag_start) = remaining.find('<') {
                // Text before tag
                let text_before = remaining[..tag_start].trim();
                if !text_before.is_empty() {
                    elements.push(DocumentElement::Paragraph(ParagraphElement {
                        text: Self::decode_html_entities(text_before),
                        style: TextStyle::default(),
                    }));
                }

                // Find tag end
                if let Some(tag_end) = remaining[tag_start..].find('>') {
                    let tag = &remaining[tag_start + 1..tag_start + tag_end];
                    let tag_name = tag.split_whitespace().next().unwrap_or("").to_lowercase();
                    
                    remaining = &remaining[tag_start + tag_end + 1..];

                    // Handle different tags
                    match tag_name.as_str() {
                        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            let level = tag_name.chars().nth(1).unwrap().to_digit(10).unwrap_or(1) as u8;
                            let close_tag = format!("</{}>", tag_name);
                            if let Some(close_idx) = remaining.find(&close_tag) {
                                let text = Self::strip_tags(&remaining[..close_idx]);
                                elements.push(DocumentElement::Heading(HeadingElement {
                                    text: Self::decode_html_entities(&text),
                                    level,
                                }));
                                remaining = &remaining[close_idx + close_tag.len()..];
                            }
                        }
                        "p" => {
                            if let Some(close_idx) = remaining.find("</p>") {
                                let text = Self::strip_tags(&remaining[..close_idx]);
                                if !text.trim().is_empty() {
                                    elements.push(DocumentElement::Paragraph(ParagraphElement {
                                        text: Self::decode_html_entities(&text),
                                        style: TextStyle::default(),
                                    }));
                                }
                                remaining = &remaining[close_idx + 4..];
                            }
                        }
                        "table" => {
                            if let Some(close_idx) = remaining.find("</table>") {
                                let table_html = &remaining[..close_idx];
                                if let Some(table) = self.parse_table(table_html) {
                                    elements.push(DocumentElement::Table(table));
                                }
                                remaining = &remaining[close_idx + 8..];
                            }
                        }
                        "ul" | "ol" => {
                            let ordered = tag_name == "ol";
                            let close_tag = format!("</{}>", tag_name);
                            if let Some(close_idx) = remaining.find(&close_tag) {
                                let list_html = &remaining[..close_idx];
                                let list = self.parse_list(list_html, ordered);
                                elements.push(DocumentElement::List(list));
                                remaining = &remaining[close_idx + close_tag.len()..];
                            }
                        }
                        "hr" | "br" => {
                            // Skip self-closing tags
                        }
                        _ if tag.starts_with('/') || tag.ends_with('/') => {
                            // Skip closing or self-closing tags
                        }
                        _ => {
                            // Skip unknown tags but continue parsing
                        }
                    }
                } else {
                    break;
                }
            } else {
                // No more tags, add remaining text
                let text = remaining.trim();
                if !text.is_empty() {
                    elements.push(DocumentElement::Paragraph(ParagraphElement {
                        text: Self::decode_html_entities(text),
                        style: TextStyle::default(),
                    }));
                }
                break;
            }
        }

        if elements.is_empty() {
            elements.push(DocumentElement::Paragraph(ParagraphElement {
                text: Self::strip_tags(body),
                style: TextStyle::default(),
            }));
        }

        Ok(DocumentContent {
            metadata,
            pages: vec![DocumentPage {
                page_number: 1,
                elements,
            }],
        })
    }

    /// Parse HTML table
    fn parse_table(&self, html: &str) -> Option<TableElement> {
        let mut rows = Vec::new();
        let mut remaining = html;
        let mut has_header = false;

        while let Some(tr_start) = remaining.find("<tr") {
            if let Some(tr_content_start) = remaining[tr_start..].find('>') {
                remaining = &remaining[tr_start + tr_content_start + 1..];
                
                if let Some(tr_end) = remaining.find("</tr>") {
                    let row_html = &remaining[..tr_end];
                    let mut cells = Vec::new();
                    
                    // Parse cells (th or td)
                    let mut cell_remaining = row_html;
                    while let Some(cell_start) = cell_remaining.find('<') {
                        let tag_end = cell_remaining[cell_start..].find('>').unwrap_or(0);
                        let tag = &cell_remaining[cell_start + 1..cell_start + tag_end];
                        let tag_name = tag.split_whitespace().next().unwrap_or("").to_lowercase();
                        
                        if tag_name == "th" || tag_name == "td" {
                            if tag_name == "th" {
                                has_header = true;
                            }
                            
                            cell_remaining = &cell_remaining[cell_start + tag_end + 1..];
                            let close_tag = format!("</{}>", tag_name);
                            
                            if let Some(close_idx) = cell_remaining.find(&close_tag) {
                                let text = Self::strip_tags(&cell_remaining[..close_idx]);
                                cells.push(TableCell {
                                    text: Self::decode_html_entities(&text),
                                    style: TextStyle::default(),
                                });
                                cell_remaining = &cell_remaining[close_idx + close_tag.len()..];
                            }
                        } else if cell_start + 1 < cell_remaining.len() {
                            cell_remaining = &cell_remaining[cell_start + 1..];
                        } else {
                            break;
                        }
                    }
                    
                    if !cells.is_empty() {
                        rows.push(TableRow { cells });
                    }
                    
                    remaining = &remaining[tr_end + 5..];
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if rows.is_empty() {
            None
        } else {
            Some(TableElement { rows, has_header })
        }
    }

    /// Parse HTML list
    fn parse_list(&self, html: &str, ordered: bool) -> ListElement {
        let mut items = Vec::new();
        let mut remaining = html;

        while let Some(li_start) = remaining.find("<li") {
            if let Some(li_content_start) = remaining[li_start..].find('>') {
                remaining = &remaining[li_start + li_content_start + 1..];
                
                if let Some(li_end) = remaining.find("</li>") {
                    let text = Self::strip_tags(&remaining[..li_end]);
                    items.push(ListItem {
                        text: Self::decode_html_entities(&text),
                        nested: None,
                    });
                    remaining = &remaining[li_end + 5..];
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        ListElement { items, ordered }
    }

    /// Strip HTML tags from text
    fn strip_tags(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        
        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }
        
        // Normalize whitespace
        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Decode common HTML entities
    fn decode_html_entities(s: &str) -> String {
        s.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ")
            .replace("&mdash;", "—")
            .replace("&ndash;", "–")
            .replace("&copy;", "©")
            .replace("&reg;", "®")
    }

    /// Get metadata (minimal for HTML)
    pub fn get_metadata(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentMetadata> {
        let file_meta = std::fs::metadata(path.as_ref())?;
        let content = self.read(path)?;
        
        Ok(DocumentMetadata {
            title: content.metadata.title,
            file_size: file_meta.len(),
            format: DocumentFormat::Html,
            ..Default::default()
        })
    }

    // =========================================================================
    // WRITING
    // =========================================================================

    /// Write a forensic report to HTML
    pub fn write_report(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> DocumentResult<()> {
        let html = self.render_report(report);
        std::fs::write(output_path, html)?;
        Ok(())
    }

    /// Render report to HTML string
    pub fn render_report(&self, report: &ForensicReport) -> String {
        let classification_color = match report.metadata.classification {
            crate::report::Classification::Public => "#28a745",
            crate::report::Classification::Internal => "#007bff",
            crate::report::Classification::Confidential => "#fd7e14",
            crate::report::Classification::Restricted => "#dc3545",
            crate::report::Classification::LawEnforcementSensitive => "#6f42c1",
        };

        let mut html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{}</title>
<style>
:root {{
    --primary: #2c3e50;
    --accent: #3498db;
    --classification: {};
    --border: #ddd;
    --bg-light: #f8f9fa;
}}
body {{
    font-family: 'Segoe UI', Tahoma, sans-serif;
    line-height: 1.6;
    max-width: 1000px;
    margin: 0 auto;
    padding: 20px;
    background: #fff;
    color: #333;
}}
.classification-banner {{
    background: var(--classification);
    color: white;
    text-align: center;
    padding: 8px;
    font-weight: bold;
    margin: -20px -20px 20px;
}}
h1, h2, h3 {{
    color: var(--primary);
}}
h1 {{
    border-bottom: 3px solid var(--accent);
    padding-bottom: 0.3em;
}}
h2 {{
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.2em;
    margin-top: 2em;
}}
table {{
    width: 100%;
    border-collapse: collapse;
    margin: 1em 0;
}}
th, td {{
    border: 1px solid var(--border);
    padding: 8px 12px;
    text-align: left;
}}
th {{
    background: var(--primary);
    color: white;
}}
tr:nth-child(even) {{
    background: var(--bg-light);
}}
.meta {{
    color: #666;
    font-style: italic;
}}
.finding {{
    background: var(--bg-light);
    padding: 1em;
    margin: 1em 0;
    border-left: 4px solid var(--accent);
}}
.finding-header {{
    font-weight: bold;
    font-size: 1.1em;
}}
.severity-critical {{ color: #dc3545; }}
.severity-high {{ color: #fd7e14; }}
.severity-medium {{ color: #ffc107; }}
.severity-low {{ color: #28a745; }}
.severity-info {{ color: #17a2b8; }}
.footer {{
    margin-top: 3em;
    padding-top: 1em;
    border-top: 1px solid var(--border);
    text-align: center;
    font-size: 0.9em;
    color: #666;
}}
{}</style>
</head>
<body>
"##, 
            Self::escape_html(&report.metadata.title),
            classification_color,
            if self.print_styles {
                "@media print { body { max-width: 100%; } .classification-banner { position: fixed; top: 0; left: 0; right: 0; } }"
            } else {
                ""
            }
        );

        // Classification banner
        html.push_str(&format!(
            "<div class=\"classification-banner\">{}</div>\n",
            report.metadata.classification.as_str()
        ));

        // Title
        html.push_str(&format!(
            "<h1>{}</h1>\n<p class=\"meta\">Report #{} | Version {} | {}</p>\n",
            Self::escape_html(&report.metadata.title),
            Self::escape_html(&report.metadata.report_number),
            Self::escape_html(&report.metadata.version),
            report.metadata.generated_at.format("%B %d, %Y")
        ));

        // Case Information
        html.push_str("<h2>Case Information</h2>\n<table>\n");
        html.push_str(&format!("<tr><th>Case Number</th><td>{}</td></tr>\n", Self::escape_html(&report.case_info.case_number)));
        if let Some(ref name) = report.case_info.case_name {
            html.push_str(&format!("<tr><th>Case Name</th><td>{}</td></tr>\n", Self::escape_html(name)));
        }
        if let Some(ref agency) = report.case_info.agency {
            html.push_str(&format!("<tr><th>Agency</th><td>{}</td></tr>\n", Self::escape_html(agency)));
        }
        html.push_str(&format!("<tr><th>Examiner</th><td>{}</td></tr>\n", Self::escape_html(&report.examiner.name)));
        if let Some(ref org) = report.examiner.organization {
            html.push_str(&format!("<tr><th>Organization</th><td>{}</td></tr>\n", Self::escape_html(org)));
        }
        html.push_str("</table>\n");

        // Executive Summary
        if let Some(ref summary) = report.executive_summary {
            html.push_str("<h2>Executive Summary</h2>\n");
            html.push_str(&format!("<p>{}</p>\n", Self::escape_html(summary)));
        }

        // Evidence
        if !report.evidence_items.is_empty() {
            html.push_str("<h2>Evidence Examined</h2>\n<table>\n");
            html.push_str("<tr><th>ID</th><th>Description</th><th>Type</th><th>Serial/Model</th></tr>\n");
            for item in &report.evidence_items {
                let serial = item.serial_number.as_deref().or(item.model.as_deref()).unwrap_or("-");
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    Self::escape_html(&item.evidence_id),
                    Self::escape_html(&item.description),
                    item.evidence_type.as_str(),
                    Self::escape_html(serial)
                ));
            }
            html.push_str("</table>\n");
        }

        // Findings
        if !report.findings.is_empty() {
            html.push_str("<h2>Findings</h2>\n");
            for finding in &report.findings {
                let severity_class = format!("severity-{}", finding.severity.as_str().to_lowercase());
                html.push_str(&format!(
                    "<div class=\"finding\">\n<div class=\"finding-header\">{}: {}</div>\n",
                    Self::escape_html(&finding.finding_id),
                    Self::escape_html(&finding.title)
                ));
                html.push_str(&format!(
                    "<p class=\"meta\">Severity: <span class=\"{}\">{}</span> | Category: {}</p>\n",
                    severity_class,
                    finding.severity.as_str(),
                    finding.category.as_str()
                ));
                html.push_str(&format!("<p>{}</p>\n", Self::escape_html(&finding.description)));
                if !finding.related_files.is_empty() {
                    html.push_str("<p><strong>Related Files:</strong></p><ul>\n");
                    for file in &finding.related_files {
                        html.push_str(&format!("<li>{}</li>\n", Self::escape_html(file)));
                    }
                    html.push_str("</ul>\n");
                }
                html.push_str("</div>\n");
            }
        }

        // Timeline
        if !report.timeline.is_empty() {
            html.push_str("<h2>Timeline</h2>\n<table>\n");
            html.push_str("<tr><th>Timestamp</th><th>Type</th><th>Description</th><th>Source</th></tr>\n");
            for event in &report.timeline {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    Self::escape_html(&event.timestamp_type),
                    Self::escape_html(&event.description),
                    Self::escape_html(&event.source)
                ));
            }
            html.push_str("</table>\n");
        }

        // Conclusions
        if let Some(ref conclusions) = report.conclusions {
            html.push_str("<h2>Conclusions</h2>\n");
            html.push_str(&format!("<p>{}</p>\n", Self::escape_html(conclusions)));
        }

        // Footer
        html.push_str(&format!(
            "<div class=\"footer\">\n<p>Generated by {} on {}</p>\n<p><strong>{}</strong></p>\n</div>\n",
            Self::escape_html(&report.metadata.generated_by),
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.metadata.classification.as_str()
        ));

        html.push_str("</body>\n</html>");
        html
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

impl Default for HtmlDocument {
    fn default() -> Self {
        Self::new()
    }
}
