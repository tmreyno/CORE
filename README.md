# CORE-FFX - Forensic File Explorer

CORE-FFX is a professional-grade forensic file explorer built with Tauri v2 and SolidJS. It focuses on evidence discovery, container metadata, verification workflows, and report generation while keeping evidence read-only.

## Highlights

- Evidence directory scanning with streaming discovery
- Container metadata, segment awareness, and integrity verification
- Hex and text viewers with container-aware navigation
- Project files (`.ffxproj`) for session restore and audit continuity
- Processed database discovery (AXIOM parsing; additional detectors for other tools)
- Report generation (PDF, DOCX, HTML, Markdown; Typst optional)
- Extension registry for future parsers, viewers, and export formats

## Format Support

### Fully Parsed Containers

These formats are parsed for metadata, verification, and browsing:

- AD1 (AccessData logical)
- E01/Ex01 (Expert Witness Format)
- L01/Lx01 (EnCase logical)
- Raw images (.dd, .raw, .img, .001)
- UFED extractions (.ufd, .ufdr)
- Archives (ZIP/7z/RAR metadata; ZIP extraction supported)

### Detected For Triage

These are detected during scans to surface candidate evidence or related files. Metadata support varies by format and may be limited:

- AFF/AFF4
- Virtual disks (VMDK, VHD, VHDX, QCOW2)
- Optical images (ISO, DMG)
- Compressed archives (TAR, GZIP, XZ, BZIP2, ZSTD, LZ4)

## Quick Start

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Project Structure

```
AD1-tools/
|-- src/                       # Frontend (SolidJS + TypeScript)
|-- src-tauri/                 # Backend (Rust + Tauri v2)
|-- docs/                      # Technical docs and analyses
|-- ui-template/               # Reusable UI template (not part of runtime)
|-- CODE_BIBLE.md              # Code directory + glossary
|-- APP_README.md              # Application guide and usage
|-- HELP.md                    # Quick help reference
```

## Documentation Index

- Application Guide: `APP_README.md`
- Quick Help: `HELP.md`
- Code Directory and Glossary: `CODE_BIBLE.md`
- Frontend Modules: `src/components/README.md`, `src/hooks/README.md`, `src/report/README.md`
- Backend Modules: `src-tauri/src/README.md`, `src-tauri/src/containers/README.md`
- Project File Spec: `src-tauri/FFX_PROJECT_FORMAT.md`
- AXIOM Reference: `src-tauri/AXIOM_DATABASE_BIBLE.md`

## License

MIT License - See `LICENSE`.
