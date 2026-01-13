# CORE-FFX Report Components

UI components for forensic report generation.

## ReportWizard.tsx

Multi-step wizard for building and exporting reports.

Steps:

1) Case Information
2) Evidence Selection
3) Findings
4) Preview
5) Export

Supported outputs (backend): PDF, DOCX, HTML, Markdown. Typst is optional when built with the `typst-reports` feature.

AI-assisted narrative generation is available when the backend is built with the `ai-assistant` feature and a provider is configured.

## Integration

- Frontend API: `src/report/api.ts`
- Rust backend: `src-tauri/src/report/`
- Templates: `src-tauri/src/report/templates/`
