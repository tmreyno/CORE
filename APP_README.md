# CORE-FFX Application Guide

CORE-FFX (Forensic File Explorer) is a desktop application for forensic evidence discovery and review. It combines a SolidJS frontend with a Rust/Tauri backend for performance and safe, read-only access to evidence containers.

## Quick Start

```bash
npm install
npm run tauri dev
```

## What CORE-FFX Does

- Scans evidence directories and identifies container files
- Loads container metadata, segment lists, and stored hashes
- Verifies integrity with multiple hash algorithms
- Provides hex and text viewers for evidence entries (AD1)
- Discovers processed databases (AXIOM parsing; other tools detected)
- Generates reports (PDF, DOCX, HTML, Markdown)

## Interface Overview

### Header

- Branding and current status indicator
- Status message reflects background activity (scan, hash, report)

### Toolbar

- Open Directory and scan path input
- Recursive scan toggle
- Hash algorithm selector (forensic + fast options)
- Hash selected files
- Load all container metadata
- Save/Load project
- Report wizard

### Panels

- Left panel: Evidence tree or Processed Databases (tab switch)
- Center panel: Evidence file list
- Right panel: Detail view (Info, Hex, Text) for the active tab

### Status Bar

- Current status, counts, and system stats (CPU/memory)

## Evidence Workflow

1) Open a directory
- Choose an evidence root folder
- The Project Setup Wizard can optionally scan for processed databases

2) Scan for evidence
- The scanner identifies known container formats by extension
- File list displays container type, size, and segment count

3) Load metadata
- Click "Load All" or select individual files to load container info
- Stored hashes and segment completeness are shown when available

4) Browse container contents
- AD1 containers expose a lazy-loaded tree for internal files
- Other container types currently expose metadata and verification only

5) Hash verification
- Choose algorithm (MD5, SHA1, SHA256, SHA512, BLAKE3, BLAKE2b, XXH3, XXH64, CRC32)
- Hash selected files and compare to stored hashes when present

## Processed Databases

- The scanner detects several processed database formats
- AXIOM parsing is implemented (case info and artifact categories)
- Other tool detections (Cellebrite PA, X-Ways, Autopsy, EnCase, FTK) are discovery-only for now

## Project Files (.ffxproj)

- Auto-save runs every 5 minutes
- Manual save uses the toolbar
- Default save path is the parent of the evidence root, named after the root folder
- Saved state includes open tabs, hash history, and UI layout

See `src-tauri/FFX_PROJECT_FORMAT.md` for the on-disk schema.

## Report Generation

Report Wizard steps:

1) Case Information
2) Evidence Selection
3) Findings
4) Preview
5) Export

Output formats:

- PDF (genpdf)
- DOCX (docx-rs)
- HTML
- Markdown
- Typst (optional build feature)

AI-assisted narrative generation is available when the backend is built with the `ai-assistant` feature and a provider is configured.

## Keyboard Navigation

File list navigation:

- Arrow keys: move selection
- Enter: open active file/tab
- Space: toggle selection
- Home/End: jump to first/last
- Escape: clear filter and focus

Global keyboard shortcuts are not currently wired in the app.

## Troubleshooting

- Scan shows no files
  - Confirm the directory contains container extensions (E01, AD1, L01, raw, ZIP/7z, UFED)
  - Check recursive scanning if files are nested

- Hash verification fails
  - Verify all segments are present
  - Confirm algorithm selection matches stored hash type

- Report generation fails
  - Ensure output path is writable
  - Try HTML or Markdown for quick validation

## Technical Reference

### Architecture

- Frontend: SolidJS + TypeScript
- Backend: Rust + Tauri v2
- IPC: `tauri::command` + `invoke()` with event streaming
- Storage: SQLite case DB + `.ffxproj` session files

### Key Docs

- Design System: `ui-template/DESIGN_DOCUMENT.md`
- Code Directory: `CODE_BIBLE.md`
- Extensions: `src/extensions/README.md`
- Backend API: `src-tauri/src/README.md`

## License

MIT License - See `LICENSE`.
