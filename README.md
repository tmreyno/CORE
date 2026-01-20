# CORE-FFX - Forensic File Explorer# CORE-FFX - Forensic File Explorer# CORE-FFX - Forensic File Explorer



CORE-FFX is a professional-grade forensic file explorer built with **Tauri v2** and **SolidJS**. It focuses on evidence discovery, container metadata, verification workflows, and report generation while maintaining strict **read-only** evidence handling.



## HighlightsCORE-FFX is a professional-grade forensic file explorer built with **Tauri v2** and **SolidJS**. It focuses on evidence discovery, container metadata, verification workflows, and report generation while maintaining strict **read-only** evidence handling.CORE-FFX is a professional-grade forensic file explorer built with Tauri v2 and SolidJS. It focuses on evidence discovery, container metadata, verification workflows, and report generation while keeping evidence read-only.



- 📁 Evidence directory scanning with streaming discovery

- 🔍 Container metadata, segment awareness, and integrity verification

- 🔬 Hex/text viewers with container-aware navigation## Highlights## Highlights

- 📄 Universal file viewers (PDF, images, email, binaries, spreadsheets, plists)

- 💾 Project files (`.ffxproj`) for session restore and audit continuity

- 🗄️ Processed database discovery (AXIOM parsing; additional detectors)

- 📊 Report generation (PDF, DOCX, HTML, Markdown)- 📁 Evidence directory scanning with streaming discovery- Evidence directory scanning with streaming discovery

- 🔌 Extension registry for custom parsers, viewers, and exporters

- 🔍 Container metadata, segment awareness, and integrity verification- Container metadata, segment awareness, and integrity verification

## Format Support

- 🔬 Hex/text viewers with container-aware navigation- Hex and text viewers with container-aware navigation

### Fully Parsed Containers

- 📄 Universal file viewers (PDF, images, email, binaries, spreadsheets, plists)- Project files (`.ffxproj`) for session restore and audit continuity

| Format | Extensions | Capabilities |

| ------ | ---------- | ------------ |- 💾 Project files (`.ffxproj`) for session restore and audit continuity- Processed database discovery (AXIOM parsing; additional detectors for other tools)

| AD1 | `.ad1`, `.ad2`... | Tree browsing, extraction, hash verification |

| E01/Ex01 | `.E01`, `.Ex01` | Segment verification, metadata extraction |- 🗄️ Processed database discovery (AXIOM parsing; additional detectors)- Report generation (PDF, DOCX, HTML, Markdown; Typst optional)

| L01/Lx01 | `.L01`, `.Lx01` | Logical image parsing |

| Raw Images | `.dd`, `.raw`, `.img`, `.001` | Direct byte access, VFS mounting |- 📊 Report generation (PDF, DOCX, HTML, Markdown)- Extension registry for future parsers, viewers, and export formats

| UFED | `.ufd`, `.ufdr`, `.ufdx` | Mobile extraction parsing |

| Archives | `.zip`, `.7z`, `.rar` | Metadata + ZIP extraction |- 🔌 Extension registry for custom parsers, viewers, and exporters



### Universal File Viewers## Format Support



| Category | Formats |## Format Support

| -------- | ------- |

| Documents | PDF, DOCX, HTML, Markdown, Text |### Fully Parsed Containers

| Images | PNG, JPEG, GIF, WebP, HEIC + EXIF metadata |

| Email | EML, MBOX |### Fully Parsed Containers

| Binaries | PE (EXE/DLL), ELF, Mach-O |

| Data | Plist, JSON, XML, CSV, Excel, SQLite |These formats are parsed for metadata, verification, and browsing:



### Detected For Triage| Format | Extensions | Capabilities |



These formats are detected during scans but have limited parsing:|--------|-----------|--------------|- AD1 (AccessData logical)



- AFF/AFF4, VMDK, VHD, VHDX, QCOW2, ISO, DMG| AD1 | `.ad1`, `.ad2`... | Tree browsing, extraction, hash verification |- E01/Ex01 (Expert Witness Format)

- TAR, GZIP, XZ, BZIP2, ZSTD, LZ4

| E01/Ex01 | `.E01`, `.Ex01` | Segment verification, metadata extraction |- L01/Lx01 (EnCase logical)

## Quick Start

| L01/Lx01 | `.L01`, `.Lx01` | Logical image parsing |- Raw images (.dd, .raw, .img, .001)

```bash

# Install dependencies| Raw Images | `.dd`, `.raw`, `.img`, `.001` | Direct byte access, VFS mounting |- UFED extractions (.ufd, .ufdr)

npm install

| UFED | `.ufd`, `.ufdr`, `.ufdx` | Mobile extraction parsing |- Archives (ZIP/7z/RAR metadata; ZIP extraction supported)

# Run in development mode

npm run tauri dev| Archives | `.zip`, `.7z`, `.rar` | Metadata + ZIP extraction |



# Build for production### Detected For Triage

npm run tauri build

### Universal File Viewers

# Run Rust tests

cd src-tauri && cargo testThese are detected during scans to surface candidate evidence or related files. Metadata support varies by format and may be limited:

```

| Category | Formats |

## Project Structure

|----------|---------|- AFF/AFF4

```text

CORE-FFX/| Documents | PDF, DOCX, HTML, Markdown, Text |- Virtual disks (VMDK, VHD, VHDX, QCOW2)

├── src/                    # Frontend (SolidJS + TypeScript)

│   ├── components/         # UI components| Images | PNG, JPEG, GIF, WebP, HEIC + EXIF metadata |- Optical images (ISO, DMG)

│   ├── hooks/              # State management + Tauri bridge

│   ├── styles/             # CSS design system| Email | EML, MBOX |- Compressed archives (TAR, GZIP, XZ, BZIP2, ZSTD, LZ4)

│   │   └── variables.css   # Design tokens

│   └── types/              # TypeScript definitions| Binaries | PE (EXE/DLL), ELF, Mach-O |

├── src-tauri/              # Backend (Rust + Tauri v2)

│   └── src/| Data | Plist, JSON, XML, CSV, Excel, SQLite |## Quick Start

│       ├── commands/       # Tauri IPC commands

│       ├── containers/     # Container abstraction layer

│       ├── viewer/         # File viewers

│       └── common/         # Shared utilities### Detected For Triage```bash

├── docs/                   # Technical documentation

├── tailwind.config.js      # Tailwind CSS configuration# Install dependencies

└── .github/

    └── copilot-instructions.mdThese formats are detected during scans but have limited parsing:npm install

```

- AFF/AFF4, VMDK, VHD, VHDX, QCOW2, ISO, DMG

## Documentation

- TAR, GZIP, XZ, BZIP2, ZSTD, LZ4# Run in development mode

| Document | Purpose |

| -------- | ------- |npm run tauri dev

| [`CODE_BIBLE.md`](CODE_BIBLE.md) | Authoritative codebase map and glossary |

| [`APP_README.md`](APP_README.md) | Application user guide |## Quick Start

| [`HELP.md`](HELP.md) | Quick help reference |

| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Developer workflow |# Build for production

| [`src/styles/README.md`](src/styles/README.md) | Tailwind CSS styling guide |

| [`src-tauri/src/README.md`](src-tauri/src/README.md) | Backend module reference |```bashnpm run tauri build

| [`src/components/README.md`](src/components/README.md) | Frontend component catalog |

| [`.github/copilot-instructions.md`](.github/copilot-instructions.md) | AI coding agent guidance |# Install dependencies```



## Architecturenpm install



```text## Project Structure

┌─────────────────────────────────────────────────────────────────┐

│                    Frontend (SolidJS + Vite)                    │# Run in development mode

│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │

│  │ Components  │  │   Hooks     │  │   Tailwind + CSS Vars   │ │npm run tauri dev```

│  └──────┬──────┘  └──────┬──────┘  └───────────────────────┬─┘ │

│         │                │                                  │   │AD1-tools/

│         └────────────────┼──────────────────────────────────┘   │

│                          │ invoke()                             │# Build for production|-- src/                       # Frontend (SolidJS + TypeScript)

├──────────────────────────┼──────────────────────────────────────┤

│                          ▼                                      │npm run tauri build|-- src-tauri/                 # Backend (Rust + Tauri v2)

│                    Backend (Rust + Tauri v2)                    │

│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │|-- docs/                      # Technical docs and analyses

│  │  Commands   │  │ Containers  │  │   Viewer/Document       │ │

│  │   (IPC)     │  │ Abstraction │  │   (PDF, Email, etc.)    │ │# Run Rust tests|-- ui-template/               # Reusable UI template (not part of runtime)

│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │

└─────────────────────────────────────────────────────────────────┘cd src-tauri && cargo test|-- CODE_BIBLE.md              # Code directory + glossary

```

```|-- APP_README.md              # Application guide and usage

## Key Principles

|-- HELP.md                    # Quick help reference

- **Read-Only Evidence**: Source files are never modified

- **Hash Verification**: Integrity checks against stored checksums## Project Structure```

- **Progress Events**: Long operations emit streaming progress updates

- **Container Abstraction**: Format-agnostic API over specific parsers

- **Design Tokens**: CSS variables for consistent theming

```## Documentation Index

## License

CORE-FFX/

MIT License - See [`LICENSE`](LICENSE).

├── src/                    # Frontend (SolidJS + TypeScript)- Application Guide: `APP_README.md`

│   ├── components/         # UI components- Quick Help: `HELP.md`

│   ├── hooks/              # State management + Tauri bridge- Code Directory and Glossary: `CODE_BIBLE.md`

│   ├── types/              # TypeScript type definitions- Frontend Modules: `src/components/README.md`, `src/hooks/README.md`, `src/report/README.md`

│   └── report/             # Report generation UI- Backend Modules: `src-tauri/src/README.md`, `src-tauri/src/containers/README.md`

├── src-tauri/              # Backend (Rust + Tauri v2)- Project File Spec: `src-tauri/FFX_PROJECT_FORMAT.md`

│   └── src/- AXIOM Reference: `src-tauri/AXIOM_DATABASE_BIBLE.md`

│       ├── commands/       # Tauri IPC commands

│       ├── containers/     # Container abstraction layer## License

│       ├── viewer/         # File viewers

│       │   ├── document/   # Content viewers (PDF, email, binary, etc.)MIT License - See `LICENSE`.

│       │   └── parsers/    # Format header parsers
│       ├── ad1/, ewf/, ufed/  # Format-specific parsers
│       └── common/         # Shared utilities
├── docs/                   # Technical documentation
│   └── archive/            # Historical docs (dated)
└── .github/
    └── copilot-instructions.md  # AI coding agent instructions
```

## Documentation

| Document | Purpose |
|----------|---------|
| [`CODE_BIBLE.md`](CODE_BIBLE.md) | Authoritative codebase map and glossary |
| [`APP_README.md`](APP_README.md) | Application user guide |
| [`HELP.md`](HELP.md) | Quick help reference |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Developer workflow |
| [`src-tauri/src/README.md`](src-tauri/src/README.md) | Backend module reference |
| [`src/components/README.md`](src/components/README.md) | Frontend component catalog |
| [`src/hooks/README.md`](src/hooks/README.md) | State management hooks |
| [`.github/copilot-instructions.md`](.github/copilot-instructions.md) | AI coding agent guidance |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Frontend (SolidJS + Vite)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │ Components  │  │   Hooks     │  │    Event Listeners      │ │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
│         │                │                      │               │
│         └────────────────┼──────────────────────┘               │
│                          │ invoke()                             │
├──────────────────────────┼──────────────────────────────────────┤
│                          ▼                                      │
│                    Backend (Rust + Tauri v2)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │  Commands   │  │ Containers  │  │   Viewer/Document       │ │
│  │   (IPC)     │  │ Abstraction │  │   (PDF, Email, etc.)    │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │ AD1 Parser  │  │ EWF Parser  │  │   UFED / Archive        │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Key Principles

- **Read-Only Evidence**: Source files are never modified
- **Hash Verification**: Integrity checks against stored checksums
- **Progress Events**: Long operations emit streaming progress updates
- **Container Abstraction**: Format-agnostic API over specific parsers

## License

MIT License - See [`LICENSE`](LICENSE).

---

*Last updated: January 18, 2026*
