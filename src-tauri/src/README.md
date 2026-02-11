# CORE-FFX Rust Backend

Tauri v2 backend for the CORE-FFX forensic file explorer.

## Module Structure

```text
src/
├── main.rs           # Tauri app entry point
├── lib.rs            # Tauri commands registration (IPC surface)
├── formats.rs        # Format registry and detection
├── database.rs       # SQLite case DB
├── logging.rs        # Tracing configuration
├── project.rs        # .cffx persistence
├── raw.rs            # Raw disk image support
│
├── commands/         # Tauri command handlers (organized by feature)
│   ├── container.rs  # AD1 container operations (V1 + V2)
│   ├── lazy_loading.rs # Unified lazy loading
│   ├── archive/      # Archive tree listing and extraction
│   ├── archive_create.rs # Archive creation (7z)
│   ├── ufed.rs       # UFED container operations
│   ├── ewf.rs        # EWF/E01 format operations
│   ├── raw.rs        # Raw disk image operations
│   ├── vfs.rs        # Virtual filesystem mounting
│   ├── hash.rs       # Batch hashing operations
│   ├── system.rs     # System stats and monitoring
│   ├── analysis.rs   # Data viewing, hex dump, entropy
│   ├── database.rs   # SQLite persistence
│   ├── project.rs    # Project file handling
│   ├── viewer.rs     # File viewer operations
│   ├── discovery.rs  # Path and evidence discovery
│   ├── unified.rs    # Unified container API
│   ├── export.rs     # File export
│   ├── search.rs     # Search operations
│   ├── index.rs      # Index cache commands
│   └── ...           # Plus profiling, recovery, extraction modules
│
├── common/           # Shared utilities
│   ├── hash.rs       # Hash algorithms (MD5, SHA-1, SHA-256, etc.)
│   ├── binary.rs     # Binary parsing helpers
│   ├── segments.rs   # Multi-segment file handling
│   ├── path_security.rs # Path traversal protection
│   ├── io_adaptive.rs # Adaptive I/O buffer sizing
│   ├── hash_queue.rs # Priority hash queue
│   ├── index_cache.rs # Container index cache
│   ├── vfs.rs        # Virtual filesystem helpers
│   └── ...           # Plus metrics, profiling, health modules
│
├── containers/       # Unified container abstraction
│   ├── types.rs      # ContainerInfo, ContainerKind, errors
│   ├── operations.rs # Info/verify/extract dispatch
│   ├── scanning.rs   # Directory scanning
│   ├── traits.rs     # Evidence container traits
│   ├── impls.rs      # Trait implementations
│   ├── segments.rs   # Multi-segment helpers
│   ├── companion.rs  # Companion log detection
│   ├── case_documents.rs # Case document scanning
│   └── unified.rs    # Unified container operations
│
├── viewer/           # File viewing subsystem
│   ├── mod.rs        # Hex/text viewing core
│   ├── types.rs      # FileChunk, FileTypeInfo
│   ├── parsers/      # Format header parsers
│   └── document/     # Content viewers (see below)
│
├── ad1/              # AD1 parser
├── ewf/              # E01/Ex01/L01 parser
├── ufed/             # UFED parsing
├── archive/          # Archive metadata + ZIP extraction
├── processed/        # Processed DB detection + AXIOM parsing
└── report/           # Report generation
```

## Viewer/Document Module

The `viewer/document/` module provides universal file viewing capabilities:

```text
viewer/document/
├── mod.rs            # Document service and exports
├── universal.rs      # UniversalFormat detection and ViewerType hints
├── commands.rs       # Tauri viewer commands
├── types.rs          # Document types (DocumentContent, metadata)
├── error.rs          # Error handling
├── pdf.rs            # PDF viewing (lopdf)
├── docx.rs           # Word documents (docx-rs)
├── html.rs           # HTML viewing
├── markdown.rs       # Markdown viewing
├── exif.rs           # EXIF metadata extraction (kamadak-exif)
├── email.rs          # EML/MBOX parsing (mail-parser)
├── binary.rs         # PE/ELF/Mach-O analysis (goblin)
├── plist_viewer.rs   # Apple plist parsing (plist)
└── spreadsheet.rs    # Excel/CSV/ODS (calamine)
```

## Supported Formats

### Containers (Full Parsing)

| Format | Extensions | Module |
|--------|------------|--------|
| AD1 | `.ad1`, `.ad2`... | `ad1/` |
| E01/Ex01 | `.E01`, `.Ex01` | `ewf/` |
| L01/Lx01 | `.L01`, `.Lx01` | `ewf/` |
| Raw | `.dd`, `.raw`, `.img`, `.001` | `raw.rs` |
| UFED | `.ufd`, `.ufdr`, `.ufdx` | `ufed/` |
| Archives | `.zip`, `.7z`, `.rar` | `archive/` |

### Universal File Viewers

| Category | Formats | Crate |
|----------|---------|-------|
| Documents | PDF, DOCX, HTML, MD | lopdf, docx-rs |
| Images | PNG, JPEG, GIF, WebP, HEIC | image, kamadak-exif |
| Email | EML, MBOX | mail-parser |
| Binaries | PE, ELF, Mach-O | goblin |
| Data | Plist, JSON, XML | plist, serde_json |
| Spreadsheets | XLSX, CSV, ODS | calamine |

## Tauri Commands

Commands are exposed via `#[tauri::command]` and registered in `lib.rs`. Major groups:

| Group | Commands | Purpose |
|-------|----------|---------|
| Container | `logical_info`, `container_get_tree` | Container metadata and browsing |
| Hashing | `raw_verify`, `ad1_hash_segments` | Hash verification |
| Viewer | `universal_get_info`, `universal_read_text` | File viewing |
| Document | `document_read`, `document_render_html` | Document rendering |
| Discovery | `scan_directory` | Evidence scanning |
| Report | `generate_report`, `preview_report` | Report generation |

## Report Outputs

| Format | Crate | Notes |
|--------|-------|-------|
| PDF | genpdf | Primary output |
| DOCX | docx-rs | Microsoft Word |
| HTML | native | Web viewing |
| Markdown | native | Plain text |
| Typst | typst | Optional feature: `typst-reports` |

AI features are gated by the `ai-assistant` feature flag.

## Build

```bash
cd src-tauri
cargo build --release
```

## Test

```bash
cd src-tauri
cargo test                              # All tests
cargo test viewer::document::           # Document module tests
cargo test --test test_document_formats # Integration tests
```

## Library Name

The crate is named `ffx_check_lib`. Use this for test imports:

```rust
use ffx_check_lib::viewer::document::universal::UniversalFormat;
```

## Key Principles

1. **Read-only**: Never modify source evidence files
2. **Path safety**: Use `common/path_security.rs` for all path operations
3. **Progress events**: Emit `*-progress` events for long operations
4. **Error handling**: Return `Result<T, String>` from commands, avoid `.unwrap()`

---

*Last updated: February 11, 2026*
