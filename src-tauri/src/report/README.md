# Report Generation Module

Backend report generation, preview rendering, and specialized evidence form exports.

## Architecture

- `commands.rs` owns the Tauri entry points: `generate_report`, `preview_report`, `get_output_formats`, and `export_evidence_collection`.
- `mod.rs` owns `ReportGenerator`, which is the single dispatch point for all standard report outputs.
- `html.rs`, `markdown.rs`, `pdf.rs`, and `docx.rs` are the canonical standard-report renderers.
- `format_helpers.rs` normalizes freeform narrative text into paragraphs and bullet lists so all renderers present investigator notes consistently.
- `template.rs` remains available for legacy or custom template workflows, but the report wizard preview now uses the canonical `HtmlGenerator` output so preview and exported HTML stay aligned.
- `pdf_coc_form7.rs` and `pdf_evidence_collection.rs` are specialized PDF form renderers. They are intentionally separate from the standard narrative PDF renderer.
- `evidence_collection_export.rs` owns standalone evidence-collection CSV/XLSX/HTML exports, while `commands.rs` also supports canonical JSON package export for evidence collections.

## Standard Output Coverage

The standard report renderers should all include the same major sections when data is present:

- Case Information
- Executive Summary
- Scope of Examination
- Methodology
- Evidence Examined
- Evidence Collection
- Chain of Custody
- Findings
- Timeline
- Hash Verification
- Tools Used
- Conclusions
- Additional Notes
- Appendices
- Approvals and Signatures

PDF and DOCX use simpler layouts than HTML, but they should not silently omit these sections anymore.

## Files

- `mod.rs` - Module exports and `ReportGenerator`
- `types/` - Modular report schema (`mod.rs`, `case.rs`, `findings.rs`, `records.rs`, `evidence_collection.rs`)
- `format_helpers.rs` - Shared text normalization helpers for all renderers
- `template.rs` - Legacy template engine / custom template support
- `pdf.rs` - Standard PDF output (genpdf)
- `docx.rs` - Standard DOCX output (docx-rs)
- `html.rs` - Standard HTML output and preview renderer
- `markdown.rs` - Standard Markdown output
- `commands.rs` - Tauri commands
- `pdf_coc_form7.rs` - Specialized chain-of-custody PDF form
- `pdf_evidence_collection.rs` - Specialized evidence-collection PDF form
- `evidence_collection_export.rs` - Standalone evidence-collection exports
- `ai.rs` - AI narrative generation (feature gated)
- `typst_gen.rs` - Typst support (feature gated)

## Supported Outputs

- PDF
- DOCX
- HTML
- Markdown
- Typst (optional build feature)

## Templates

Templates live in `src-tauri/src/report/templates/`.

Use `template.rs` only when a workflow explicitly needs a custom template path. For the standard report wizard preview/export flow, prefer the built-in renderers.

## Do Not

- Do not route `preview_report` back through `template_engine.render_html()` for the default wizard flow. Preview must match the canonical HTML export output.
- Do not bypass `format_helpers.rs` when rendering freeform narrative text. Raw newline-to-`<br>` replacement or single-paragraph dumps make the reports harder to read.
- Do not merge the standard PDF renderer with `pdf_coc_form7.rs` or `pdf_evidence_collection.rs`. Those files intentionally preserve form-specific layouts.
- Do not add new standard-report sections to only one renderer. HTML, Markdown, PDF, and DOCX must stay broadly aligned in section coverage.

## AI Features

AI generation is optional and requires the `ai-assistant` feature plus provider configuration.
