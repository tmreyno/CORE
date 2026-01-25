// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Markdown document handler
//!
//! This module provides reading and writing capabilities for Markdown documents.

use std::path::Path;

use super::error::DocumentResult;
use super::types::*;
use super::DocumentFormat;
use crate::report::ForensicReport;

/// Markdown document handler
pub struct MarkdownDocument {
    /// Use CommonMark compatible output (reserved for future rendering options)
    #[allow(dead_code)]
    commonmark: bool,
}

impl MarkdownDocument {
    /// Create a new Markdown document handler
    pub fn new() -> Self {
        Self { commonmark: true }
    }

    // =========================================================================
    // READING
    // =========================================================================

    /// Read Markdown from file path
    pub fn read(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentContent> {
        let data = std::fs::read(path)?;
        self.read_bytes(&data)
    }

    /// Read Markdown from bytes
    pub fn read_bytes(&self, data: &[u8]) -> DocumentResult<DocumentContent> {
        let text = String::from_utf8_lossy(data).to_string();
        self.parse_markdown(&text)
    }

    /// Parse Markdown content into document structure
    fn parse_markdown(&self, text: &str) -> DocumentResult<DocumentContent> {
        let mut metadata = DocumentMetadata {
            format: DocumentFormat::Markdown,
            ..Default::default()
        };
        let mut elements = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut line_idx = 0;

        // Check for YAML front matter
        if !lines.is_empty() && lines[0].trim() == "---" {
            let mut fm_end = 1;
            while fm_end < lines.len() && lines[fm_end].trim() != "---" {
                // Parse front matter key-value pairs
                let line = lines[fm_end].trim();
                if let Some(colon_idx) = line.find(':') {
                    let key = line[..colon_idx].trim().to_lowercase();
                    let value = line[colon_idx + 1..].trim().trim_matches('"').trim_matches('\'');
                    match key.as_str() {
                        "title" => metadata.title = Some(value.to_string()),
                        "author" => metadata.author = Some(value.to_string()),
                        _ => {}
                    }
                }
                fm_end += 1;
            }
            line_idx = fm_end + 1;
        }

        // Parse markdown content
        while line_idx < lines.len() {
            let line = lines[line_idx];

            // Heading (ATX style: #, ##, etc.)
            if line.starts_with('#') {
                let level = line.chars().take_while(|c| *c == '#').count() as u8;
                let text = line[level as usize..].trim().trim_end_matches('#').trim();
                
                // First heading becomes title if not set
                if level == 1 && metadata.title.is_none() {
                    metadata.title = Some(text.to_string());
                }

                elements.push(DocumentElement::Heading(HeadingElement {
                    text: text.to_string(),
                    level,
                }));
                line_idx += 1;
                continue;
            }

            // Setext-style headings (=== or ---)
            if line_idx + 1 < lines.len() {
                let next_line = lines[line_idx + 1].trim();
                if next_line.chars().all(|c| c == '=') && next_line.len() >= 3 {
                    elements.push(DocumentElement::Heading(HeadingElement {
                        text: line.trim().to_string(),
                        level: 1,
                    }));
                    line_idx += 2;
                    continue;
                }
                if next_line.chars().all(|c| c == '-') && next_line.len() >= 3 && !line.trim().is_empty() {
                    elements.push(DocumentElement::Heading(HeadingElement {
                        text: line.trim().to_string(),
                        level: 2,
                    }));
                    line_idx += 2;
                    continue;
                }
            }

            // Horizontal rule
            let trimmed = line.trim();
            if trimmed.len() >= 3 && (
                trimmed.chars().all(|c| c == '-' || c == ' ') ||
                trimmed.chars().all(|c| c == '*' || c == ' ') ||
                trimmed.chars().all(|c| c == '_' || c == ' ')
            ) && trimmed.chars().filter(|c| *c != ' ').count() >= 3 {
                elements.push(DocumentElement::Break);
                line_idx += 1;
                continue;
            }

            // Unordered list
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
                let mut items = Vec::new();
                while line_idx < lines.len() {
                    let list_line = lines[line_idx].trim();
                    if list_line.starts_with("- ") || list_line.starts_with("* ") || list_line.starts_with("+ ") {
                        items.push(ListItem {
                            text: list_line[2..].trim().to_string(),
                            nested: None,
                        });
                        line_idx += 1;
                    } else if list_line.is_empty() {
                        line_idx += 1;
                        break;
                    } else {
                        break;
                    }
                }
                elements.push(DocumentElement::List(ListElement {
                    items,
                    ordered: false,
                }));
                continue;
            }

            // Ordered list
            if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                if let Some(dot_idx) = trimmed.find(". ") {
                    if trimmed[..dot_idx].chars().all(|c| c.is_ascii_digit()) {
                        let mut items = Vec::new();
                        while line_idx < lines.len() {
                            let list_line = lines[line_idx].trim();
                            if let Some(d_idx) = list_line.find(". ") {
                                if list_line[..d_idx].chars().all(|c| c.is_ascii_digit()) {
                                    items.push(ListItem {
                                        text: list_line[d_idx + 2..].trim().to_string(),
                                        nested: None,
                                    });
                                    line_idx += 1;
                                    continue;
                                }
                            }
                            if list_line.is_empty() {
                                line_idx += 1;
                            }
                            break;
                        }
                        elements.push(DocumentElement::List(ListElement {
                            items,
                            ordered: true,
                        }));
                        continue;
                    }
                }
            }

            // Table
            if line.contains('|') {
                let mut rows = Vec::new();
                let mut has_header = false;
                
                // First row
                let first_row = self.parse_table_row(line);
                rows.push(first_row);
                line_idx += 1;

                // Check for separator row
                if line_idx < lines.len() {
                    let sep_line = lines[line_idx].trim();
                    if sep_line.contains('|') && sep_line.contains('-') {
                        has_header = true;
                        line_idx += 1;
                    }
                }

                // Data rows
                while line_idx < lines.len() {
                    let row_line = lines[line_idx];
                    if row_line.contains('|') {
                        rows.push(self.parse_table_row(row_line));
                        line_idx += 1;
                    } else {
                        break;
                    }
                }

                elements.push(DocumentElement::Table(TableElement { rows, has_header }));
                continue;
            }

            // Code block (fenced)
            if trimmed.starts_with("```") {
                line_idx += 1;
                let mut code_lines = Vec::new();
                while line_idx < lines.len() {
                    let code_line = lines[line_idx];
                    if code_line.trim().starts_with("```") {
                        line_idx += 1;
                        break;
                    }
                    code_lines.push(code_line);
                    line_idx += 1;
                }
                elements.push(DocumentElement::Paragraph(ParagraphElement {
                    text: code_lines.join("\n"),
                    style: TextStyle {
                        font_family: Some("monospace".to_string()),
                        ..Default::default()
                    },
                }));
                continue;
            }

            // Blockquote
            if trimmed.starts_with("> ") || trimmed == ">" {
                let mut quote_lines = Vec::new();
                while line_idx < lines.len() {
                    let quote_line = lines[line_idx].trim();
                    if quote_line.starts_with('>') {
                        let content = quote_line.strip_prefix('>').unwrap_or("").trim();
                        quote_lines.push(content);
                        line_idx += 1;
                    } else {
                        // Empty line or non-quote line ends the blockquote
                        break;
                    }
                }
                elements.push(DocumentElement::Paragraph(ParagraphElement {
                    text: quote_lines.join("\n"),
                    style: TextStyle {
                        italic: true,
                        ..Default::default()
                    },
                }));
                continue;
            }

            // Regular paragraph
            if !trimmed.is_empty() {
                let mut para_lines = vec![line];
                line_idx += 1;
                while line_idx < lines.len() {
                    let para_line = lines[line_idx];
                    if para_line.trim().is_empty() 
                        || para_line.starts_with('#')
                        || para_line.trim().starts_with("- ")
                        || para_line.trim().starts_with("* ")
                        || para_line.contains('|')
                        || para_line.trim().starts_with("```")
                        || para_line.trim().starts_with("> ")
                    {
                        break;
                    }
                    para_lines.push(para_line);
                    line_idx += 1;
                }
                
                let text = para_lines.join(" ").trim().to_string();
                elements.push(DocumentElement::Paragraph(ParagraphElement {
                    text,
                    style: TextStyle::default(),
                }));
            } else {
                line_idx += 1;
            }
        }

        Ok(DocumentContent {
            metadata,
            pages: vec![DocumentPage {
                page_number: 1,
                elements,
            }],
        })
    }

    /// Parse a table row
    fn parse_table_row(&self, line: &str) -> TableRow {
        let cells: Vec<TableCell> = line
            .split('|')
            .filter(|s| !s.trim().is_empty())
            .map(|cell| TableCell {
                text: cell.trim().to_string(),
                style: TextStyle::default(),
            })
            .collect();
        TableRow { cells }
    }

    /// Get metadata
    pub fn get_metadata(&self, path: impl AsRef<Path>) -> DocumentResult<DocumentMetadata> {
        let file_meta = std::fs::metadata(path.as_ref())?;
        let content = self.read(path)?;
        
        Ok(DocumentMetadata {
            title: content.metadata.title,
            author: content.metadata.author,
            file_size: file_meta.len(),
            format: DocumentFormat::Markdown,
            ..Default::default()
        })
    }

    // =========================================================================
    // WRITING
    // =========================================================================

    /// Write a forensic report to Markdown
    pub fn write_report(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> DocumentResult<()> {
        let md = self.render_report(report);
        std::fs::write(output_path, md)?;
        Ok(())
    }

    /// Render report to Markdown string
    pub fn render_report(&self, report: &ForensicReport) -> String {
        let mut md = String::new();

        // YAML front matter
        md.push_str("---\n");
        md.push_str(&format!("title: \"{}\"\n", Self::escape_yaml(&report.metadata.title)));
        md.push_str(&format!("report_number: \"{}\"\n", report.metadata.report_number));
        md.push_str(&format!("version: \"{}\"\n", report.metadata.version));
        md.push_str(&format!("classification: \"{}\"\n", report.metadata.classification.as_str()));
        md.push_str(&format!("generated_at: \"{}\"\n", report.metadata.generated_at.format("%Y-%m-%dT%H:%M:%SZ")));
        md.push_str(&format!("generated_by: \"{}\"\n", Self::escape_yaml(&report.metadata.generated_by)));
        md.push_str("---\n\n");

        // Title and classification
        md.push_str(&format!("# {}\n\n", report.metadata.title));
        md.push_str(&format!("> **Classification:** {}\n\n", report.metadata.classification.as_str()));

        // Case Information
        md.push_str("## Case Information\n\n");
        md.push_str("| Field | Value |\n");
        md.push_str("| ----- | ----- |\n");
        md.push_str(&format!("| Case Number | {} |\n", report.case_info.case_number));
        if let Some(ref name) = report.case_info.case_name {
            md.push_str(&format!("| Case Name | {} |\n", name));
        }
        if let Some(ref agency) = report.case_info.agency {
            md.push_str(&format!("| Agency | {} |\n", agency));
        }
        if let Some(ref desc) = report.case_info.description {
            md.push_str(&format!("| Description | {} |\n", desc));
        }
        md.push('\n');

        // Examiner
        md.push_str("## Examiner Information\n\n");
        md.push_str(&format!("**{}**", report.examiner.name));
        if let Some(ref title) = report.examiner.title {
            md.push_str(&format!(", {}", title));
        }
        md.push('\n');
        if let Some(ref org) = report.examiner.organization {
            md.push_str(&format!("- Organization: {}\n", org));
        }
        if let Some(ref email) = report.examiner.email {
            md.push_str(&format!("- Email: {}\n", email));
        }
        if let Some(ref phone) = report.examiner.phone {
            md.push_str(&format!("- Phone: {}\n", phone));
        }
        md.push('\n');

        // Executive Summary
        if let Some(ref summary) = report.executive_summary {
            md.push_str("## Executive Summary\n\n");
            md.push_str(summary);
            md.push_str("\n\n");
        }

        // Evidence Items
        if !report.evidence_items.is_empty() {
            md.push_str("## Evidence Examined\n\n");
            md.push_str("| ID | Description | Type | Serial/Model |\n");
            md.push_str("| -- | ----------- | ---- | ------------ |\n");
            for item in &report.evidence_items {
                let serial = item.serial_number.as_deref()
                    .or(item.model.as_deref())
                    .unwrap_or("-");
                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    item.evidence_id,
                    item.description,
                    item.evidence_type.as_str(),
                    serial
                ));
            }
            md.push('\n');
        }

        // Methodology
        if let Some(ref methodology) = report.methodology {
            md.push_str("## Methodology\n\n");
            md.push_str(methodology);
            md.push_str("\n\n");
        }

        // Findings
        if !report.findings.is_empty() {
            md.push_str("## Findings\n\n");
            for finding in &report.findings {
                md.push_str(&format!("### {}: {}\n\n", finding.finding_id, finding.title));
                md.push_str(&format!("- **Severity:** {}\n", finding.severity.as_str()));
                md.push_str(&format!("- **Category:** {}\n", finding.category.as_str()));
                if !finding.timestamps.is_empty() {
                    let ts_str = finding.timestamps.first()
                        .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default();
                    md.push_str(&format!("- **Timestamp:** {}\n", ts_str));
                }
                md.push('\n');
                md.push_str(&format!("{}\n\n", finding.description));
                
                if !finding.related_files.is_empty() {
                    md.push_str("**Related Files:**\n");
                    for file in &finding.related_files {
                        md.push_str(&format!("- `{}`\n", file));
                    }
                    md.push('\n');
                }

                if !finding.supporting_evidence.is_empty() {
                    md.push_str("**Supporting Evidence:**\n");
                    for evidence in &finding.supporting_evidence {
                        md.push_str(&format!("- {}\n", evidence));
                    }
                    md.push('\n');
                }
            }
        }

        // Timeline
        if !report.timeline.is_empty() {
            md.push_str("## Timeline\n\n");
            md.push_str("| Timestamp | Type | Description | Source |\n");
            md.push_str("| --------- | ---- | ----------- | ------ |\n");
            for event in &report.timeline {
                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    event.timestamp_type,
                    event.description.replace('\n', " "),
                    event.source
                ));
            }
            md.push('\n');
        }

        // Tools Used
        if !report.tools.is_empty() {
            md.push_str("## Tools Used\n\n");
            for tool in &report.tools {
                md.push_str(&format!("- **{}** v{}", tool.name, tool.version));
                if let Some(ref purpose) = tool.purpose {
                    md.push_str(&format!(" - {}", purpose));
                }
                md.push('\n');
            }
            md.push('\n');
        }

        // Conclusions
        if let Some(ref conclusions) = report.conclusions {
            md.push_str("## Conclusions\n\n");
            md.push_str(conclusions);
            md.push_str("\n\n");
        }

        // Footer
        md.push_str("---\n\n");
        md.push_str(&format!(
            "*Generated by {} on {} UTC*\n\n",
            report.metadata.generated_by,
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        md.push_str(&format!("**{}**\n", report.metadata.classification.as_str()));

        md
    }

    /// Escape YAML special characters
    fn escape_yaml(s: &str) -> String {
        s.replace('"', "\\\"")
    }
}

impl Default for MarkdownDocument {
    fn default() -> Self {
        Self::new()
    }
}
