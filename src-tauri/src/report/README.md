# Report Generation Module

Backend report generation and template rendering.

## Files

- `mod.rs` - Module exports
- `types.rs` - Report schema
- `template.rs` - Template engine
- `pdf.rs` - PDF output (genpdf)
- `docx.rs` - DOCX output (docx-rs)
- `html.rs` - HTML output
- `markdown.rs` - Markdown output
- `commands.rs` - Tauri commands
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

## AI Features

AI generation is optional and requires the `ai-assistant` feature plus provider configuration.
