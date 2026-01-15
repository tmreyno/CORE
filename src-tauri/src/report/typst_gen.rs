// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Typst report generator
//!
//! Generates high-quality typeset reports using Typst markup.
//! Typst is a modern alternative to LaTeX with cleaner syntax and faster compilation.
//!
//! This module generates `.typ` source files. To compile to PDF:
//! - Install Typst CLI: `cargo install typst-cli`
//! - Run: `typst compile report.typ report.pdf`
//!
//! This module is feature-gated under `typst-reports`.

use std::path::Path;

use super::error::ReportResult;
use super::types::*;

/// Typst generator for forensic reports
/// 
/// Generates Typst markup that can be compiled to professional PDF documents
/// using the Typst CLI or integrated compiler.
pub struct TypstGenerator {
    /// Custom template path (optional)
    custom_template: Option<String>,
}

impl TypstGenerator {
    /// Create a new Typst generator with default template
    pub fn new() -> Self {
        Self {
            custom_template: None,
        }
    }

    /// Create a Typst generator with a custom template
    pub fn with_template(template: impl Into<String>) -> Self {
        Self {
            custom_template: Some(template.into()),
        }
    }

    /// Generate a Typst source file
    /// 
    /// The output is a `.typ` file that can be compiled to PDF using:
    /// `typst compile output.typ output.pdf`
    pub fn generate(&self, report: &ForensicReport, output_path: impl AsRef<Path>) -> ReportResult<()> {
        let typst_source = self.render_typst(report)?;
        std::fs::write(output_path, typst_source)?;
        Ok(())
    }

    /// Render the report to Typst source code
    fn render_typst(&self, report: &ForensicReport) -> ReportResult<String> {
        if let Some(ref template) = self.custom_template {
            // Use custom template
            self.render_with_template(report, template)
        } else {
            // Use built-in template
            Ok(self.render_default(report))
        }
    }

    /// Render with custom template using variable substitution
    /// 
    /// Supported placeholders:
    /// - `{{report_title}}` - Report title
    /// - `{{case_number}}` - Case number
    /// - `{{case_name}}` - Case name
    /// - `{{examiner_name}}` - Examiner name
    /// - `{{examiner_title}}` - Examiner title
    /// - `{{examiner_agency}}` - Examiner agency/organization
    /// - `{{date}}` - Current date
    /// - `{{executive_summary}}` - Executive summary text
    /// - `{{scope}}` - Scope and objectives
    /// - `{{methodology}}` - Methodology section
    /// - `{{conclusions}}` - Conclusions section
    /// - `{{notes}}` - Additional notes
    /// - `{{evidence_table}}` - Evidence items as a table
    /// - `{{findings_section}}` - Findings section with details
    /// - `{{timeline_table}}` - Timeline events as a table
    /// - `{{hash_table}}` - Hash verification records table
    /// - `{{tools_table}}` - Tools and versions table
    fn render_with_template(&self, report: &ForensicReport, template: &str) -> ReportResult<String> {
        let mut output = template.to_string();
        
        // Basic report metadata
        output = output.replace("{{report_title}}", &report.metadata.title);
        output = output.replace("{{case_number}}", &report.case_info.case_number);
        output = output.replace("{{case_name}}", &report.case_info.case_name.clone().unwrap_or_default());
        
        // Examiner info
        output = output.replace("{{examiner_name}}", &report.examiner.name);
        output = output.replace("{{examiner_title}}", &report.examiner.title.clone().unwrap_or_default());
        output = output.replace("{{examiner_agency}}", &report.examiner.organization.clone().unwrap_or_default());
        
        // Date
        output = output.replace("{{date}}", &chrono::Utc::now().format("%Y-%m-%d").to_string());
        
        // Optional text sections
        output = output.replace("{{executive_summary}}", &report.executive_summary.clone().unwrap_or_default());
        output = output.replace("{{scope}}", &report.scope.clone().unwrap_or_default());
        output = output.replace("{{methodology}}", &report.methodology.clone().unwrap_or_default());
        output = output.replace("{{conclusions}}", &report.conclusions.clone().unwrap_or_default());
        output = output.replace("{{notes}}", &report.notes.clone().unwrap_or_default());
        
        // Complex sections
        if output.contains("{{evidence_table}}") {
            output = output.replace("{{evidence_table}}", &self.render_evidence_table_simple(report));
        }
        if output.contains("{{findings_section}}") {
            output = output.replace("{{findings_section}}", &self.render_findings_simple(report));
        }
        if output.contains("{{timeline_table}}") {
            output = output.replace("{{timeline_table}}", &self.render_timeline_table_simple(report));
        }
        if output.contains("{{hash_table}}") {
            output = output.replace("{{hash_table}}", &self.render_hash_table_simple(report));
        }
        if output.contains("{{tools_table}}") {
            output = output.replace("{{tools_table}}", &self.render_tools_table_simple(report));
        }
        
        Ok(output)
    }
    
    /// Render evidence as a simple Typst table
    fn render_evidence_table_simple(&self, report: &ForensicReport) -> String {
        if report.evidence_items.is_empty() {
            return String::new();
        }
        
        let mut table = String::from("#table(\n  columns: (auto, 1fr, auto, auto),\n  [*ID*], [*Description*], [*Type*], [*Capacity*],\n");
        for item in &report.evidence_items {
            table.push_str(&format!(
                "  [{}], [{}], [{}], [{}],\n",
                Self::escape_typst(&item.evidence_id),
                Self::escape_typst(&item.description),
                item.evidence_type.as_str(),
                item.capacity.as_ref().map_or("-".to_string(), |s| s.clone())
            ));
        }
        table.push_str(")\n");
        table
    }
    
    /// Render findings as simple Typst content
    fn render_findings_simple(&self, report: &ForensicReport) -> String {
        if report.findings.is_empty() {
            return String::new();
        }
        
        let mut output = String::new();
        for (i, finding) in report.findings.iter().enumerate() {
            output.push_str(&format!("== Finding {}: {}\n\n", i + 1, Self::escape_typst(&finding.title)));
            output.push_str(&format!("*Severity:* {}\n\n", finding.severity.as_str()));
            output.push_str(&Self::escape_typst(&finding.description));
            output.push_str("\n\n");
            if let Some(ref notes) = finding.notes {
                output.push_str(&format!("*Notes:* {}\n\n", Self::escape_typst(notes)));
            }
        }
        output
    }
    
    /// Render timeline as a simple Typst table
    fn render_timeline_table_simple(&self, report: &ForensicReport) -> String {
        if report.timeline.is_empty() {
            return String::new();
        }
        
        let mut table = String::from("#table(\n  columns: (auto, 1fr, auto),\n  [*Date/Time*], [*Event*], [*Source*],\n");
        for event in &report.timeline {
            table.push_str(&format!(
                "  [{}], [{}], [{}],\n",
                event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                Self::escape_typst(&event.description),
                Self::escape_typst(&event.source)
            ));
        }
        table.push_str(")\n");
        table
    }
    
    /// Render hash records as a simple Typst table
    fn render_hash_table_simple(&self, report: &ForensicReport) -> String {
        if report.hash_records.is_empty() {
            return String::new();
        }
        
        let mut table = String::from("#table(\n  columns: (1fr, auto, auto),\n  [*Item*], [*Algorithm*], [*Hash*],\n");
        for record in &report.hash_records {
            table.push_str(&format!(
                "  [{}], [{}], [`{}`],\n",
                Self::escape_typst(&record.item),
                record.algorithm.as_str(),
                &record.value
            ));
        }
        table.push_str(")\n");
        table
    }
    
    /// Render tools as a simple Typst table
    fn render_tools_table_simple(&self, report: &ForensicReport) -> String {
        if report.tools.is_empty() {
            return String::new();
        }
        
        let mut table = String::from("#table(\n  columns: (auto, auto, 1fr),\n  [*Tool*], [*Version*], [*Purpose*],\n");
        for tool in &report.tools {
            table.push_str(&format!(
                "  [{}], [{}], [{}],\n",
                Self::escape_typst(&tool.name),
                Self::escape_typst(&tool.version),
                Self::escape_typst(&tool.purpose.clone().unwrap_or_default())
            ));
        }
        table.push_str(")\n");
        table
    }

    /// Render using the built-in default template
    fn render_default(&self, report: &ForensicReport) -> String {
        let mut output = String::new();

        // Document setup and styling
        output.push_str(&self.render_preamble(report));
        
        // Title page
        output.push_str(&self.render_title_page(report));
        
        // Table of contents
        output.push_str("\n#outline(title: \"Table of Contents\", depth: 2)\n");
        output.push_str("#pagebreak()\n\n");
        
        // Case information
        output.push_str(&self.render_case_info(report));
        
        // Executive summary
        if let Some(ref summary) = report.executive_summary {
            output.push_str(&self.render_section("Executive Summary", summary));
        }
        
        // Scope
        if let Some(ref scope) = report.scope {
            output.push_str(&self.render_section("Scope of Examination", scope));
        }
        
        // Methodology
        if let Some(ref methodology) = report.methodology {
            output.push_str(&self.render_section("Methodology", methodology));
        }
        
        // Evidence
        if !report.evidence_items.is_empty() {
            output.push_str(&self.render_evidence_section(report));
        }
        
        // Findings
        if !report.findings.is_empty() {
            output.push_str(&self.render_findings_section(report));
        }
        
        // Timeline
        if !report.timeline.is_empty() {
            output.push_str(&self.render_timeline_section(report));
        }
        
        // Tools
        if !report.tools.is_empty() {
            output.push_str(&self.render_tools_section(report));
        }
        
        // Conclusions
        if let Some(ref conclusions) = report.conclusions {
            output.push_str(&self.render_section("Conclusions", conclusions));
        }
        
        // Footer
        output.push_str(&self.render_footer(report));
        
        output
    }

    /// Render document preamble with styling
    fn render_preamble(&self, report: &ForensicReport) -> String {
        let classification = report.metadata.classification.as_str();
        let classification_color = match report.metadata.classification {
            Classification::Public => "green",
            Classification::Internal => "blue", 
            Classification::Confidential => "orange",
            Classification::Restricted => "red",
            Classification::LawEnforcementSensitive => "purple",
        };

        format!(r##"// FFX Forensic Report - Generated by FFX Forensic File Xplorer
// Classification: {classification}
// 
// To compile this file to PDF:
//   typst compile report.typ report.pdf
//
// Or use Typst's watch mode for live preview:
//   typst watch report.typ

#set document(
  title: "{title}",
  author: "{examiner}",
)

#set page(
  paper: "us-letter",
  margin: (x: 1in, y: 1in),
  header: align(center)[
    #set text(size: 9pt, weight: "bold", fill: {classification_color})
    {classification}
  ],
  footer: context [
    #set text(size: 9pt)
    #align(center)[
      #counter(page).display("1 of 1", both: true)
    ]
  ],
)

#set text(
  font: "New Computer Modern",
  size: 11pt,
)

#set heading(numbering: "1.1")

#set par(
  justify: true,
  leading: 0.65em,
)

// Custom styling functions
#let info-box(title, content) = {{
  block(
    fill: luma(245),
    inset: 10pt,
    radius: 4pt,
    width: 100%,
  )[
    *#title*
    #v(0.5em)
    #content
  ]
}}

#let severity-badge(level) = {{
  let (color, label) = if level == "Critical" {{
    (rgb("#dc2626"), "CRITICAL")
  }} else if level == "High" {{
    (rgb("#ea580c"), "HIGH")  
  }} else if level == "Medium" {{
    (rgb("#ca8a04"), "MEDIUM")
  }} else if level == "Low" {{
    (rgb("#16a34a"), "LOW")
  }} else {{
    (rgb("#6b7280"), "INFO")
  }}
  box(
    fill: color,
    inset: (x: 6pt, y: 3pt),
    radius: 3pt,
  )[#text(fill: white, weight: "bold", size: 8pt)[#label]]
}}

"##,
            title = Self::escape_typst(&report.metadata.title),
            examiner = Self::escape_typst(&report.examiner.name),
            classification = classification,
            classification_color = classification_color,
        )
    }

    /// Render title page
    fn render_title_page(&self, report: &ForensicReport) -> String {
        let classification = report.metadata.classification.as_str();
        
        format!(r##"
// Title Page
#align(center)[
  #v(2in)
  
  #text(size: 24pt, weight: "bold")[
    {title}
  ]
  
  #v(0.5em)
  
  #text(size: 14pt)[
    Report \#{report_number} | Version {version}
  ]
  
  #v(2em)
  
  #text(size: 12pt)[
    Case Number: *{case_number}*
  ]
  
  #v(0.5em)
  
  #text(size: 12pt)[
    {case_name}
  ]
  
  #v(3em)
  
  #text(size: 11pt)[
    Prepared by: \
    *{examiner_name}* \
    {examiner_title} \
    {examiner_org}
  ]
  
  #v(2em)
  
  #text(size: 10pt)[
    Generated: {generated_date}
  ]
  
  #v(fill)
  
  #text(size: 12pt, weight: "bold")[
    {classification}
  ]
]

#pagebreak()

"##,
            title = Self::escape_typst(&report.metadata.title),
            report_number = Self::escape_typst(&report.metadata.report_number),
            version = Self::escape_typst(&report.metadata.version),
            case_number = Self::escape_typst(&report.case_info.case_number),
            case_name = report.case_info.case_name.as_deref().map(Self::escape_typst).unwrap_or_default(),
            examiner_name = Self::escape_typst(&report.examiner.name),
            examiner_title = report.examiner.title.as_deref().map(Self::escape_typst).unwrap_or_default(),
            examiner_org = report.examiner.organization.as_deref().map(Self::escape_typst).unwrap_or_default(),
            generated_date = report.metadata.generated_at.format("%Y-%m-%d"),
            classification = classification,
        )
    }

    /// Render case information section
    fn render_case_info(&self, report: &ForensicReport) -> String {
        let mut s = String::from("= Case Information\n\n");
        
        s.push_str("#table(\n");
        s.push_str("  columns: (auto, 1fr),\n");
        s.push_str("  stroke: none,\n");
        s.push_str("  row-gutter: 0.5em,\n");
        
        s.push_str(&format!("  [*Case Number*], [{}],\n", Self::escape_typst(&report.case_info.case_number)));
        
        if let Some(ref name) = report.case_info.case_name {
            s.push_str(&format!("  [*Case Name*], [{}],\n", Self::escape_typst(name)));
        }
        if let Some(ref agency) = report.case_info.agency {
            s.push_str(&format!("  [*Agency*], [{}],\n", Self::escape_typst(agency)));
        }
        if let Some(ref requestor) = report.case_info.requestor {
            s.push_str(&format!("  [*Requestor*], [{}],\n", Self::escape_typst(requestor)));
        }
        if let Some(ref inv_type) = report.case_info.investigation_type {
            s.push_str(&format!("  [*Investigation Type*], [{}],\n", Self::escape_typst(inv_type)));
        }
        
        s.push_str(")\n\n");
        
        // Examiner info
        s.push_str("== Examiner\n\n");
        s.push_str("#table(\n");
        s.push_str("  columns: (auto, 1fr),\n");
        s.push_str("  stroke: none,\n");
        s.push_str("  row-gutter: 0.5em,\n");
        s.push_str(&format!("  [*Name*], [{}],\n", Self::escape_typst(&report.examiner.name)));
        
        if let Some(ref title) = report.examiner.title {
            s.push_str(&format!("  [*Title*], [{}],\n", Self::escape_typst(title)));
        }
        if let Some(ref org) = report.examiner.organization {
            s.push_str(&format!("  [*Organization*], [{}],\n", Self::escape_typst(org)));
        }
        if let Some(ref email) = report.examiner.email {
            s.push_str(&format!("  [*Email*], [{}],\n", Self::escape_typst(email)));
        }
        if let Some(ref phone) = report.examiner.phone {
            s.push_str(&format!("  [*Phone*], [{}],\n", Self::escape_typst(phone)));
        }
        
        s.push_str(")\n\n");
        
        if !report.examiner.certifications.is_empty() {
            s.push_str("*Certifications:* ");
            s.push_str(&report.examiner.certifications.iter()
                .map(|c| Self::escape_typst(c))
                .collect::<Vec<_>>()
                .join(", "));
            s.push_str("\n\n");
        }
        
        s.push_str("#pagebreak()\n\n");
        s
    }

    /// Render a simple text section
    fn render_section(&self, title: &str, content: &str) -> String {
        format!(
            "= {}\n\n{}\n\n#pagebreak()\n\n",
            title,
            Self::escape_typst(content)
        )
    }

    /// Render evidence section
    fn render_evidence_section(&self, report: &ForensicReport) -> String {
        let mut s = String::from("= Evidence Examined\n\n");
        
        s.push_str("#table(\n");
        s.push_str("  columns: (auto, 2fr, auto, auto, auto),\n");
        s.push_str("  align: (left, left, left, left, left),\n");
        s.push_str("  fill: (x, y) => if y == 0 { luma(230) } else { none },\n");
        s.push_str("  [*ID*], [*Description*], [*Type*], [*Serial*], [*Capacity*],\n");
        
        for item in &report.evidence_items {
            s.push_str(&format!(
                "  [{}], [{}], [{}], [{}], [{}],\n",
                Self::escape_typst(&item.evidence_id),
                Self::escape_typst(&item.description),
                item.evidence_type.as_str(),
                item.serial_number.as_deref().unwrap_or("-"),
                item.capacity.as_deref().unwrap_or("-"),
            ));
        }
        
        s.push_str(")\n\n");
        
        // Detailed evidence info
        for item in &report.evidence_items {
            s.push_str(&format!("== Evidence {} Details\n\n", Self::escape_typst(&item.evidence_id)));
            
            if !item.acquisition_hashes.is_empty() {
                s.push_str("*Hash Values:*\n\n");
                s.push_str("#table(\n");
                s.push_str("  columns: (auto, 1fr),\n");
                s.push_str("  [*Algorithm*], [*Value*],\n");
                for hash in &item.acquisition_hashes {
                    s.push_str(&format!(
                        "  [{}], [```{}```],\n",
                        hash.algorithm.as_str(),
                        &hash.value,
                    ));
                }
                s.push_str(")\n\n");
            }
            
            if let Some(ref notes) = item.notes {
                s.push_str(&format!("*Notes:* {}\n\n", Self::escape_typst(notes)));
            }
        }
        
        s.push_str("#pagebreak()\n\n");
        s
    }

    /// Render findings section
    fn render_findings_section(&self, report: &ForensicReport) -> String {
        let mut s = String::from("= Findings\n\n");
        
        for finding in &report.findings {
            s.push_str(&format!(
                "== {} - {}\n\n",
                Self::escape_typst(&finding.finding_id),
                Self::escape_typst(&finding.title)
            ));
            
            s.push_str(&format!(
                "#severity-badge(\"{}\") #h(1em) *Category:* {}\n\n",
                finding.severity.as_str(),
                finding.category.as_str()
            ));
            
            s.push_str(&format!("{}\n\n", Self::escape_typst(&finding.description)));
            
            if !finding.related_files.is_empty() {
                s.push_str("*Related Files:*\n");
                for file in &finding.related_files {
                    s.push_str(&format!("- `{}`\n", Self::escape_typst(file)));
                }
                s.push('\n');
            }
            
            if !finding.supporting_evidence.is_empty() {
                s.push_str(&format!(
                    "*Supporting Evidence:* {}\n\n",
                    finding.supporting_evidence.join(", ")
                ));
            }
        }
        
        s.push_str("#pagebreak()\n\n");
        s
    }

    /// Render timeline section
    fn render_timeline_section(&self, report: &ForensicReport) -> String {
        let mut s = String::from("= Timeline of Events\n\n");
        
        s.push_str("#table(\n");
        s.push_str("  columns: (auto, auto, 2fr, auto),\n");
        s.push_str("  fill: (x, y) => if y == 0 { luma(230) } else { none },\n");
        s.push_str("  [*Timestamp*], [*Type*], [*Description*], [*Source*],\n");
        
        for event in &report.timeline {
            s.push_str(&format!(
                "  [{}], [{}], [{}], [{}],\n",
                event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                Self::escape_typst(&event.timestamp_type),
                Self::escape_typst(&event.description),
                Self::escape_typst(&event.source),
            ));
        }
        
        s.push_str(")\n\n");
        s.push_str("#pagebreak()\n\n");
        s
    }

    /// Render tools section
    fn render_tools_section(&self, report: &ForensicReport) -> String {
        let mut s = String::from("= Tools Used\n\n");
        
        s.push_str("#table(\n");
        s.push_str("  columns: (1fr, auto, 1fr, 2fr),\n");
        s.push_str("  fill: (x, y) => if y == 0 { luma(230) } else { none },\n");
        s.push_str("  [*Tool*], [*Version*], [*Vendor*], [*Purpose*],\n");
        
        for tool in &report.tools {
            s.push_str(&format!(
                "  [{}], [{}], [{}], [{}],\n",
                Self::escape_typst(&tool.name),
                Self::escape_typst(&tool.version),
                tool.vendor.as_deref().map(Self::escape_typst).unwrap_or_else(|| "-".to_string()),
                tool.purpose.as_deref().map(Self::escape_typst).unwrap_or_else(|| "-".to_string()),
            ));
        }
        
        s.push_str(")\n\n");
        s.push_str("#pagebreak()\n\n");
        s
    }

    /// Render footer
    fn render_footer(&self, report: &ForensicReport) -> String {
        format!(r##"
// End of Report
#v(2em)
#align(center)[
  #line(length: 50%)
  #v(1em)
  #text(size: 9pt)[
    This report was generated by *{}* on {} UTC.
  ]
  #v(0.5em)
  #text(size: 10pt, weight: "bold")[
    {}
  ]
]
"##,
            Self::escape_typst(&report.metadata.generated_by),
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S"),
            report.metadata.classification.as_str(),
        )
    }

    /// Escape special Typst characters
    fn escape_typst(s: &str) -> String {
        s.replace('\\', "\\\\")
         .replace('#', "\\#")
         .replace('*', "\\*")
         .replace('_', "\\_")
         .replace('@', "\\@")
         .replace('$', "\\$")
         .replace('<', "\\<")
         .replace('>', "\\>")
    }
}

impl Default for TypstGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_typst_escape() {
        assert_eq!(TypstGenerator::escape_typst("Hello #world"), "Hello \\#world");
        assert_eq!(TypstGenerator::escape_typst("Test *bold*"), "Test \\*bold\\*");
    }

    #[test]
    fn test_typst_source_generation() {
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
        };

        let generator = TypstGenerator::new();
        let source = generator.render_typst(&report).unwrap();
        
        assert!(source.contains("#set document"));
        assert!(source.contains("Test Report"));
        assert!(source.contains("CASE-001"));
    }
}

