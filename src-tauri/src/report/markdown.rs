// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Markdown report generator
//!
//! Generates Markdown reports from forensic report data.
//! The output can be used with static site generators, converted to other
//! formats with Pandoc, or viewed directly on GitHub/GitLab.

use std::path::Path;

use super::error::ReportResult;
use super::types::*;

/// Markdown generator for forensic reports
pub struct MarkdownGenerator {
    /// Use GitHub Flavored Markdown tables
    gfm_tables: bool,
    /// Include YAML frontmatter
    frontmatter: bool,
}

impl MarkdownGenerator {
    /// Create a new Markdown generator with default settings
    pub fn new() -> Self {
        Self {
            gfm_tables: true,
            frontmatter: true,
        }
    }

    /// Enable or disable GFM tables
    pub fn gfm_tables(mut self, enabled: bool) -> Self {
        self.gfm_tables = enabled;
        self
    }

    /// Enable or disable YAML frontmatter
    pub fn frontmatter(mut self, enabled: bool) -> Self {
        self.frontmatter = enabled;
        self
    }

    /// Generate a Markdown report file
    pub fn generate(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> ReportResult<()> {
        let markdown = self.render_markdown(report);
        std::fs::write(output_path, markdown)?;
        Ok(())
    }

    /// Render the report to a Markdown string
    pub fn render_markdown(&self, report: &ForensicReport) -> String {
        let mut md = String::new();

        // YAML frontmatter
        if self.frontmatter {
            md.push_str(&self.render_frontmatter(report));
        }

        // Classification banner
        md.push_str(&format!(
            "> **{}**\n\n",
            report.metadata.classification.as_str()
        ));

        // Title
        md.push_str(&format!("# {}\n\n", Self::escape_md(&report.metadata.title)));

        // Subtitle/case name
        if let Some(ref name) = report.case_info.case_name {
            md.push_str(&format!("*{}*\n\n", Self::escape_md(name)));
        }

        // Report metadata
        md.push_str(&format!("**Case Number:** {}  \n", Self::escape_md(&report.case_info.case_number)));
        md.push_str(&format!("**Report Number:** {}  \n", Self::escape_md(&report.metadata.report_number)));
        md.push_str(&format!("**Examiner:** {}  \n", Self::escape_md(&report.examiner.name)));
        md.push_str(&format!("**Date:** {}  \n\n", report.metadata.generated_at.format("%B %d, %Y")));

        md.push_str("---\n\n");

        // Table of contents
        md.push_str(&self.render_toc(report));

        // Case information
        md.push_str(&self.render_case_info(report));

        // Executive summary
        if let Some(ref summary) = report.executive_summary {
            md.push_str("## Executive Summary\n\n");
            md.push_str(&Self::escape_md(summary));
            md.push_str("\n\n");
        }

        // Scope
        if let Some(ref scope) = report.scope {
            md.push_str("## Scope of Examination\n\n");
            md.push_str(&Self::escape_md(scope));
            md.push_str("\n\n");
        }

        // Methodology
        if let Some(ref methodology) = report.methodology {
            md.push_str("## Methodology\n\n");
            md.push_str(&Self::escape_md(methodology));
            md.push_str("\n\n");
        }

        // Evidence
        if !report.evidence_items.is_empty() {
            md.push_str(&self.render_evidence_section(report));
        }

        // Chain of custody
        if !report.chain_of_custody.is_empty() {
            md.push_str(&self.render_chain_of_custody(report));
        }

        // Findings
        if !report.findings.is_empty() {
            md.push_str(&self.render_findings_section(report));
        }

        // Timeline
        if !report.timeline.is_empty() {
            md.push_str(&self.render_timeline_section(report));
        }

        // Hash records
        if !report.hash_records.is_empty() {
            md.push_str(&self.render_hash_section(report));
        }

        // Tools
        if !report.tools.is_empty() {
            md.push_str(&self.render_tools_section(report));
        }

        // Conclusions
        if let Some(ref conclusions) = report.conclusions {
            md.push_str("## Conclusions\n\n");
            md.push_str(&Self::escape_md(conclusions));
            md.push_str("\n\n");
        }

        // Appendices
        if !report.appendices.is_empty() {
            md.push_str(&self.render_appendices(report));
        }

        // Footer
        md.push_str(&self.render_footer(report));

        md
    }

    /// Render YAML frontmatter
    fn render_frontmatter(&self, report: &ForensicReport) -> String {
        let mut fm = String::from("---\n");
        fm.push_str(&format!("title: \"{}\"\n", Self::escape_yaml(&report.metadata.title)));
        fm.push_str(&format!("case_number: \"{}\"\n", Self::escape_yaml(&report.case_info.case_number)));
        fm.push_str(&format!("report_number: \"{}\"\n", Self::escape_yaml(&report.metadata.report_number)));
        fm.push_str(&format!("examiner: \"{}\"\n", Self::escape_yaml(&report.examiner.name)));
        fm.push_str(&format!("classification: \"{}\"\n", report.metadata.classification.as_str()));
        fm.push_str(&format!("date: \"{}\"\n", report.metadata.generated_at.format("%Y-%m-%d")));
        fm.push_str(&format!("generated_by: \"{}\"\n", Self::escape_yaml(&report.metadata.generated_by)));
        fm.push_str("---\n\n");
        fm
    }

    /// Render table of contents
    fn render_toc(&self, report: &ForensicReport) -> String {
        let mut toc = String::from("## Table of Contents\n\n");

        toc.push_str("- [Case Information](#case-information)\n");

        if report.executive_summary.is_some() {
            toc.push_str("- [Executive Summary](#executive-summary)\n");
        }
        if report.scope.is_some() {
            toc.push_str("- [Scope of Examination](#scope-of-examination)\n");
        }
        if report.methodology.is_some() {
            toc.push_str("- [Methodology](#methodology)\n");
        }
        if !report.evidence_items.is_empty() {
            toc.push_str("- [Evidence Examined](#evidence-examined)\n");
        }
        if !report.chain_of_custody.is_empty() {
            toc.push_str("- [Chain of Custody](#chain-of-custody)\n");
        }
        if !report.findings.is_empty() {
            toc.push_str("- [Findings](#findings)\n");
        }
        if !report.timeline.is_empty() {
            toc.push_str("- [Timeline](#timeline)\n");
        }
        if !report.hash_records.is_empty() {
            toc.push_str("- [Hash Verification](#hash-verification)\n");
        }
        if !report.tools.is_empty() {
            toc.push_str("- [Tools Used](#tools-used)\n");
        }
        if report.conclusions.is_some() {
            toc.push_str("- [Conclusions](#conclusions)\n");
        }
        if !report.appendices.is_empty() {
            toc.push_str("- [Appendices](#appendices)\n");
        }

        toc.push_str("\n---\n\n");
        toc
    }

    /// Render case information
    fn render_case_info(&self, report: &ForensicReport) -> String {
        let mut md = String::from("## Case Information\n\n");

        if self.gfm_tables {
            md.push_str("| Field | Value |\n");
            md.push_str("|-------|-------|\n");
            md.push_str(&format!("| Case Number | {} |\n", Self::escape_md(&report.case_info.case_number)));

            if let Some(ref name) = report.case_info.case_name {
                md.push_str(&format!("| Case Name | {} |\n", Self::escape_md(name)));
            }
            if let Some(ref agency) = report.case_info.agency {
                md.push_str(&format!("| Agency | {} |\n", Self::escape_md(agency)));
            }
            if let Some(ref requestor) = report.case_info.requestor {
                md.push_str(&format!("| Requestor | {} |\n", Self::escape_md(requestor)));
            }

            md.push_str(&format!("| Examiner | {} |\n", Self::escape_md(&report.examiner.name)));

            if let Some(ref title) = report.examiner.title {
                md.push_str(&format!("| Title | {} |\n", Self::escape_md(title)));
            }
            if let Some(ref org) = report.examiner.organization {
                md.push_str(&format!("| Organization | {} |\n", Self::escape_md(org)));
            }
        } else {
            md.push_str(&format!("- **Case Number:** {}\n", Self::escape_md(&report.case_info.case_number)));
            if let Some(ref name) = report.case_info.case_name {
                md.push_str(&format!("- **Case Name:** {}\n", Self::escape_md(name)));
            }
            md.push_str(&format!("- **Examiner:** {}\n", Self::escape_md(&report.examiner.name)));
        }

        md.push('\n');
        md
    }

    /// Render evidence section
    fn render_evidence_section(&self, report: &ForensicReport) -> String {
        let mut md = String::from("## Evidence Examined\n\n");

        for item in &report.evidence_items {
            md.push_str(&format!("### {} - {}\n\n", 
                Self::escape_md(&item.evidence_id),
                Self::escape_md(&item.description)
            ));

            md.push_str(&format!("- **Type:** {}\n", item.evidence_type.as_str()));

            if let Some(ref make) = item.make {
                md.push_str(&format!("- **Make:** {}\n", Self::escape_md(make)));
            }
            if let Some(ref model) = item.model {
                md.push_str(&format!("- **Model:** {}\n", Self::escape_md(model)));
            }
            if let Some(ref serial) = item.serial_number {
                md.push_str(&format!("- **Serial Number:** {}\n", Self::escape_md(serial)));
            }
            if let Some(ref capacity) = item.capacity {
                md.push_str(&format!("- **Capacity:** {}\n", Self::escape_md(capacity)));
            }

            // Hash values
            if !item.acquisition_hashes.is_empty() {
                md.push_str("\n**Hash Values:**\n\n");
                if self.gfm_tables {
                    md.push_str("| Algorithm | Value |\n");
                    md.push_str("|-----------|-------|\n");
                    for hash in &item.acquisition_hashes {
                        md.push_str(&format!("| {} | `{}` |\n", 
                            hash.algorithm.as_str(),
                            Self::escape_md(&hash.value)
                        ));
                    }
                } else {
                    for hash in &item.acquisition_hashes {
                        md.push_str(&format!("- **{}:** `{}`\n", 
                            hash.algorithm.as_str(),
                            Self::escape_md(&hash.value)
                        ));
                    }
                }
            }

            md.push('\n');
        }

        md
    }

    /// Render chain of custody
    fn render_chain_of_custody(&self, report: &ForensicReport) -> String {
        let mut md = String::from("## Chain of Custody\n\n");

        if self.gfm_tables {
            md.push_str("| Date/Time | Evidence ID | Released By | Received By | Purpose | Location | Notes |\n");
            md.push_str("|-----------|-------------|-------------|-------------|---------|----------|-------|\n");

            for record in &report.chain_of_custody {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {} |\n",
                    record.timestamp.format("%Y-%m-%d %H:%M"),
                    Self::escape_md(&record.evidence_id),
                    Self::escape_md(&record.released_by),
                    Self::escape_md(&record.received_by),
                    record.purpose.as_deref().map(Self::escape_md).unwrap_or_default(),
                    record.location.as_deref().map(Self::escape_md).unwrap_or_default(),
                    record.notes.as_deref().map(Self::escape_md).unwrap_or_default()
                ));
            }
        } else {
            for record in &report.chain_of_custody {
                md.push_str(&format!(
                    "- **{}** - {} transferred from {} to {} {}\n",
                    record.timestamp.format("%Y-%m-%d %H:%M"),
                    Self::escape_md(&record.evidence_id),
                    Self::escape_md(&record.released_by),
                    Self::escape_md(&record.received_by),
                    record.location.as_deref().map(|l| format!("at {}", Self::escape_md(l))).unwrap_or_default()
                ));
            }
        }

        md.push('\n');
        md
    }

    /// Render findings section
    fn render_findings_section(&self, report: &ForensicReport) -> String {
        let mut md = String::from("## Findings\n\n");

        for finding in &report.findings {
            let severity_emoji = match finding.severity {
                FindingSeverity::Critical => "🔴",
                FindingSeverity::High => "🟠",
                FindingSeverity::Medium => "🟡",
                FindingSeverity::Low => "🟢",
                FindingSeverity::Info => "🔵",
            };

            md.push_str(&format!(
                "### {} {} - {}\n\n",
                severity_emoji,
                Self::escape_md(&finding.finding_id),
                Self::escape_md(&finding.title)
            ));

            md.push_str(&format!("**Severity:** {} | **Category:** {}\n\n",
                finding.severity.as_str(),
                finding.category.as_str()
            ));

            md.push_str(&Self::escape_md(&finding.description));
            md.push_str("\n\n");

            if !finding.related_files.is_empty() {
                md.push_str("**Related Files:**\n");
                for file in &finding.related_files {
                    md.push_str(&format!("- `{}`\n", Self::escape_md(file)));
                }
                md.push('\n');
            }

            if !finding.supporting_evidence.is_empty() {
                md.push_str(&format!("**Supporting Evidence:** {}\n\n",
                    finding.supporting_evidence.join(", ")
                ));
            }

            if let Some(ref notes) = finding.notes {
                md.push_str(&format!("**Notes:** {}\n\n", Self::escape_md(notes)));
            }
        }

        md
    }

    /// Render timeline section
    fn render_timeline_section(&self, report: &ForensicReport) -> String {
        let mut md = String::from("## Timeline\n\n");

        if self.gfm_tables {
            md.push_str("| Timestamp | Type | Description | Source |\n");
            md.push_str("|-----------|------|-------------|--------|\n");

            for event in &report.timeline {
                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    Self::escape_md(&event.timestamp_type),
                    Self::escape_md(&event.description),
                    Self::escape_md(&event.source)
                ));
            }
        } else {
            for event in &report.timeline {
                md.push_str(&format!(
                    "- **{}** [{}] - {} *(Source: {})*\n",
                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    Self::escape_md(&event.timestamp_type),
                    Self::escape_md(&event.description),
                    Self::escape_md(&event.source)
                ));
            }
        }

        md.push('\n');
        md
    }

    /// Render hash verification section
    fn render_hash_section(&self, report: &ForensicReport) -> String {
        let mut md = String::from("## Hash Verification\n\n");

        if self.gfm_tables {
            md.push_str("| Item | Algorithm | Hash Value | Verified |\n");
            md.push_str("|------|-----------|------------|----------|\n");

            for record in &report.hash_records {
                let verified = record.verified.map_or("N/A", |v| if v { "✓" } else { "✗" });
                md.push_str(&format!(
                    "| {} | {} | `{}` | {} |\n",
                    Self::escape_md(&record.item),
                    record.algorithm.as_str(),
                    Self::escape_md(&record.value),
                    verified
                ));
            }
        } else {
            for record in &report.hash_records {
                let verified = record.verified.map_or("N/A".to_string(), |v| if v { "✓ Verified".to_string() } else { "✗ Failed".to_string() });
                md.push_str(&format!(
                    "- **{}** ({}): `{}` - {}\n",
                    Self::escape_md(&record.item),
                    record.algorithm.as_str(),
                    Self::escape_md(&record.value),
                    verified
                ));
            }
        }

        md.push('\n');
        md
    }

    /// Render tools section
    fn render_tools_section(&self, report: &ForensicReport) -> String {
        let mut md = String::from("## Tools Used\n\n");

        if self.gfm_tables {
            md.push_str("| Tool | Version | Vendor | Purpose |\n");
            md.push_str("|------|---------|--------|--------|\n");

            for tool in &report.tools {
                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    Self::escape_md(&tool.name),
                    Self::escape_md(&tool.version),
                    tool.vendor.as_deref().map(Self::escape_md).unwrap_or_else(|| "-".to_string()),
                    tool.purpose.as_deref().map(Self::escape_md).unwrap_or_else(|| "-".to_string())
                ));
            }
        } else {
            for tool in &report.tools {
                md.push_str(&format!(
                    "- **{}** v{}\n",
                    Self::escape_md(&tool.name),
                    Self::escape_md(&tool.version)
                ));
                if let Some(ref purpose) = tool.purpose {
                    md.push_str(&format!("  - Purpose: {}\n", Self::escape_md(purpose)));
                }
            }
        }

        md.push('\n');
        md
    }

    /// Render appendices
    fn render_appendices(&self, report: &ForensicReport) -> String {
        let mut md = String::from("## Appendices\n\n");

        for (i, appendix) in report.appendices.iter().enumerate() {
            md.push_str(&format!(
                "### Appendix {}: {}\n\n",
                (b'A' + i as u8) as char,
                Self::escape_md(&appendix.title)
            ));
            md.push_str(&Self::escape_md(&appendix.content));
            md.push_str("\n\n");
        }

        md
    }

    /// Render footer
    fn render_footer(&self, report: &ForensicReport) -> String {
        let mut md = String::from("---\n\n");
        md.push_str(&format!(
            "*This report was generated by **{}** on {} UTC.*\n\n",
            Self::escape_md(&report.metadata.generated_by),
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        md.push_str(&format!(
            "**Classification: {}**\n",
            report.metadata.classification.as_str()
        ));
        md
    }

    /// Escape Markdown special characters
    fn escape_md(s: &str) -> String {
        s.replace('\\', "\\\\")
         .replace('*', "\\*")
         .replace('_', "\\_")
         .replace('`', "\\`")
         .replace('#', "\\#")
         .replace('|', "\\|")
         .replace('[', "\\[")
         .replace(']', "\\]")
    }

    /// Escape YAML special characters
    fn escape_yaml(s: &str) -> String {
        s.replace('\\', "\\\\")
         .replace('"', "\\\"")
    }
}

impl Default for MarkdownGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_markdown_escape() {
        assert_eq!(MarkdownGenerator::escape_md("**bold**"), "\\*\\*bold\\*\\*");
        assert_eq!(MarkdownGenerator::escape_md("_italic_"), "\\_italic\\_");
        assert_eq!(MarkdownGenerator::escape_md("`code`"), "\\`code\\`");
        assert_eq!(MarkdownGenerator::escape_md("| pipe |"), "\\| pipe \\|");
    }

    #[test]
    fn test_yaml_escape() {
        assert_eq!(MarkdownGenerator::escape_yaml("test \"quote\""), "test \\\"quote\\\"");
    }

    #[test]
    fn test_markdown_generation() {
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
            notes: None,
        };

        let generator = MarkdownGenerator::new();
        let md = generator.render_markdown(&report);

        assert!(md.contains("---\n")); // YAML frontmatter delimiter
        assert!(md.contains("# Test Report"));
        assert!(md.contains("CASE-001"));
        assert!(md.contains("Test Examiner"));
        assert!(md.contains("CONFIDENTIAL"));
    }

    #[test]
    fn test_markdown_generator_default() {
        let generator = MarkdownGenerator::default();
        assert!(generator.gfm_tables);
        assert!(generator.frontmatter);
    }

    #[test]
    fn test_markdown_without_frontmatter() {
        let generator = MarkdownGenerator::new().frontmatter(false);
        let report = ForensicReport {
            metadata: ReportMetadata {
                title: "Test".to_string(),
                report_number: "T-001".to_string(),
                version: "1.0".to_string(),
                classification: Classification::Public,
                generated_at: Utc::now(),
                generated_by: "Test".to_string(),
            },
            case_info: CaseInfo {
                case_number: "C-001".to_string(),
                ..Default::default()
            },
            examiner: ExaminerInfo {
                name: "Tester".to_string(),
                ..Default::default()
            },
            executive_summary: None,
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
            notes: None,
        };

        let md = generator.render_markdown(&report);
        // Should not start with YAML frontmatter
        assert!(!md.starts_with("---\ntitle:"));
    }
}
