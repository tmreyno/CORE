# FFX Report Templates & Schema Guide

This document describes the report schema and the template variables used by the Rust report engine.

## Overview

Reports are generated from a `ForensicReport` model (see `src-tauri/src/report/types.rs`) and rendered using Tera templates. The backend can export:

- PDF
- DOCX
- HTML
- Markdown
- Typst (optional build feature)

## Core Schema (Simplified)

```rust
pub struct ForensicReport {
    pub metadata: ReportMetadata,
    pub case_info: CaseInfo,
    pub examiner: ExaminerInfo,
    pub executive_summary: Option<String>,
    pub scope: Option<String>,
    pub methodology: Option<String>,
    pub evidence_items: Vec<EvidenceItem>,
    pub chain_of_custody: Vec<CustodyRecord>,
    pub findings: Vec<Finding>,
    pub timeline: Vec<TimelineEvent>,
    pub hash_records: Vec<HashRecord>,
    pub tools: Vec<ToolInfo>,
    pub conclusions: Option<String>,
    pub appendices: Vec<Appendix>,
    pub notes: Option<String>,
}
```

## Template Variables

Templates receive the following top-level variables:

- `metadata`
- `case` (maps to `case_info`)
- `examiner`
- `evidence_items`
- `findings`
- `timeline`
- `hash_records`
- `tools`
- `appendices`
- `report` (full model)

See `src-tauri/src/report/templates/report.md` for a working example.

## JSON Export

The backend can export/import report JSON for persistence via:

- `export_report_json`
- `import_report_json`

## Notes

- PDF uses the genpdf backend
- Typst and AI-assisted features require build flags
