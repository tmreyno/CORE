# CORE-FFX Rust Backend

Tauri v2 backend for the CORE-FFX forensic file explorer.

## Module Structure

```
src/
|-- main.rs        # Tauri app entry point
|-- lib.rs         # Tauri commands and IPC surface
|-- formats.rs     # Format registry and detection
|-- database.rs    # SQLite case DB
|-- logging.rs     # Tracing configuration
|-- project.rs     # .ffxproj persistence (v1 on-disk)
|-- raw.rs         # Raw disk image support
|-- viewer.rs      # File viewing utilities
|-- common/        # Hashing, IO pooling, path security
|-- containers/    # Unified container API
|-- ad1/           # AD1 parser
|-- ewf/           # E01/Ex01/L01 parser
|-- ufed/          # UFED parsing
|-- archive/       # Archive metadata + ZIP extraction
|-- processed/     # Processed DB detection + AXIOM parsing
|-- report/        # Report generation
```

## Supported Containers (Full Parsing)

- AD1
- E01/Ex01
- L01/Lx01
- Raw images
- UFED extractions
- Archives (ZIP/7z/RAR metadata; ZIP extraction supported)

Additional formats are detected during scans via `formats.rs` but may be metadata-only.

## Tauri Commands

Commands are exposed via `#[tauri::command]` in `lib.rs`. Major groups:

- Evidence scanning and metadata (`scan_directory*`, `logical_info*`)
- Hashing and verification (`raw_verify`, `e01_v3_verify`, `ad1_hash_segments`)
- AD1 tree operations (`container_get_tree`, `container_get_children`)
- Processed database discovery and AXIOM queries
- Report generation and preview

## Report Outputs

- PDF (genpdf)
- DOCX (docx-rs)
- HTML
- Markdown
- Typst (optional feature: `typst-reports`)

AI features are optional and gated by the `ai-assistant` feature.

## Build

```bash
cd src-tauri
cargo build --release
```

## Test

```bash
cd src-tauri
cargo test
```
