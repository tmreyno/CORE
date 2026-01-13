# CORE-FFX Code Bible (AD1-tools)

The Code Bible is the authoritative map of the repository. It defines directory layout, module responsibilities, runtime flows, and shared terms.

## Scope

- Directory structure and entry points
- Module responsibilities (frontend and backend)
- Runtime flows and IPC contracts
- Shared glossary and invariants

## Architecture At A Glance

- Frontend: SolidJS + TypeScript (Vite) in `src/`
- Backend: Rust + Tauri v2 in `src-tauri/`
- IPC: `tauri::command` (Rust) + `invoke()` (TS) + event emission
- Storage: SQLite case DB + `.ffxproj` session files

## Directory Map

AD1-tools/
|-- README.md                    - Project overview
|-- APP_README.md                - Application guide
|-- HELP.md                      - Quick help
|-- CODE_BIBLE.md                - This document
|-- CHANGELOG.md                 - Version history
|-- CONTRIBUTING.md              - Contributor/developer workflow
|-- SECURITY.md                  - Security policy
|-- docs/                        - Technical analyses and references
|-- ui-template/                 - UI template package (not runtime)
|-- src/                         - Frontend (SolidJS + TypeScript)
|-- src-tauri/                   - Backend (Rust + Tauri)

### Frontend (`src/`)

src/
|-- index.tsx                    - App bootstrap
|-- App.tsx                      - Main layout and state wiring
|-- AppRouter.tsx                - App shell
|-- assets/                      - Branding assets
|-- components/                  - UI components
|-- hooks/                       - State and Tauri integration
|-- extensions/                  - Extension registry and types
|-- report/                      - Report API + types (frontend)
|-- types/                       - Domain types and registries
|-- utils/                       - Utility helpers

### Backend (`src-tauri/`)

src-tauri/
|-- Cargo.toml, Cargo.lock       - Rust dependencies
|-- tauri.conf.json              - Tauri app config
|-- src/                         - Rust source
|-- test_output/                 - Example report output
|-- AXIOM_DATABASE_BIBLE.md      - AXIOM reference (parser uses subset)
|-- FFX_PROJECT_FORMAT.md        - Project file schema
|-- FFX_REPORT_TEMPLATES.md      - Report template reference
|-- Archive_Reference.md         - Archive notes (filename includes trailing space)

#### Rust source (`src-tauri/src/`)

src-tauri/src/
|-- main.rs                      - Tauri app entry
|-- lib.rs                       - Tauri commands (IPC surface)
|-- formats.rs                   - Format registry and detection
|-- logging.rs                   - Logging/tracing setup
|-- database.rs                  - SQLite case DB
|-- project.rs                   - `.ffxproj` persistence (v1 on-disk)
|-- raw.rs                       - Raw image support
|-- viewer.rs                    - File viewer utilities
|-- common/                      - Hashing, path security, IO pools
|-- containers/                  - Container abstraction layer
|-- ad1/                         - AD1 parser and VFS
|-- ewf/                         - E01/Ex01/L01 parser
|-- ufed/                        - UFED extraction parsing
|-- archive/                     - Archive metadata and ZIP extraction
|-- processed/                   - Processed DB detection + AXIOM parsing
|-- report/                      - Report generation pipeline
|-- report/templates/            - HTML/Markdown templates

## Runtime Flows (High Level)

Evidence scan:
1) UI selects a directory
2) `scan_directory*` commands discover containers
3) Backend emits `scan-file-found` events
4) UI builds discovered file list and loads `ContainerInfo`

Container info and browsing:
1) UI calls `logical_info` or `logical_info_fast`
2) Backend detects format and extracts metadata
3) AD1 tree is lazy-loaded via `container_get_tree` / `container_get_children`

Hash verification:
1) UI selects algorithm
2) Backend hashes container (format-specific commands)
3) Backend emits `verify-progress` or `batch-progress`

Reports:
1) UI builds a report model
2) Backend generates PDF/DOCX/HTML/Markdown
3) Optional Typst and AI features are gated by build flags

Processed databases:
1) Detector scans for AXIOM/PA/X-Ways/etc.
2) AXIOM parsing is implemented; others are detection-only
3) UI renders metadata and artifact categories

## Format Support Notes

- Full parsing: AD1, E01/Ex01, L01/Lx01, Raw, UFED, ZIP/7z/RAR metadata
- ZIP extraction is implemented; other archive formats are metadata only
- Additional formats are detected during scans (AFF/AFF4, VMDK/VHD/VHDX, QCOW2, ISO, DMG, TAR/GZIP/XZ/BZIP2/ZSTD/LZ4)

## Contracts And Shared Types

TypeScript definitions live in `src/types/` and `src/types.ts`. Rust equivalents live in `src-tauri/src/`.

Alignment points:
- `src/types/formats.ts` <-> `src-tauri/src/formats.rs`
- `src/types/lifecycle.ts` <-> `src-tauri/src/containers/traits.rs`
- `src/types/processed.ts` <-> `src-tauri/src/processed/types.rs`
- `src/report/types.ts` <-> `src-tauri/src/report/types.rs`

Project files:
- Frontend uses an extended project model (v2)
- Rust persists a v1 subset in `src-tauri/src/project.rs`

## Glossary

- Artifact: A record extracted from a processed database.
- Container: A forensic evidence file (AD1, E01, L01, Raw, UFED, Archive).
- ContainerInfo: Unified metadata for a container.
- ContainerKind: Enum identifying the container type.
- DiscoveredFile: A container found during directory scanning.
- Evidence tree: Lazy-loaded AD1 file tree (addresses for hex navigation).
- Extension: A plugin with a manifest and category (viewer, parser, exporter, tool).
- Project file: `.ffxproj` session state file.
- ReportConfig/ForensicReport: Report metadata and output specification.
- Segment: One file in a multi-part container (.E01, .E02, .ad2).

## Invariants

- Evidence files are treated as read-only.
- Path traversal is sanitized in backend utilities.
- Hash verification prefers stored hashes when present.
- Long operations emit progress events for UI responsiveness.

## References

Frontend docs:
- `src/components/README.md`
- `src/hooks/README.md`
- `src/report/README.md`
- `src/extensions/README.md`
- `src/types/README.md`
- `src/utils/README.md`

Backend docs:
- `src-tauri/src/README.md`
- `src-tauri/src/containers/README.md`
- `src-tauri/src/report/README.md`
- `src-tauri/FFX_PROJECT_FORMAT.md`
- `src-tauri/AXIOM_DATABASE_BIBLE.md`
