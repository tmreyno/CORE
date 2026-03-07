# CORE-FFX Code Bible

The Code Bible is the authoritative map of the repository. It defines directory layout, module responsibilities, runtime flows, and shared terms.

## Scope

- Directory structure and entry points
- Module responsibilities (frontend and backend)
- Runtime flows and IPC contracts
- Shared glossary and invariants

## Architecture Overview

- **Frontend**: SolidJS + TypeScript (Vite) in `src/`
- **Backend**: Rust + Tauri v2 in `src-tauri/`
- **IPC**: `tauri::command` (Rust) + `invoke()` (TS) + event emission
- **Storage**: SQLite case DB + `.cffx` session files + `.ffxdb` per-project database

## Directory Map

```text
CORE-FFX/
├── README.md                    # Project overview
├── HELP.md                      # Quick help
├── CODE_BIBLE.md                # This document
├── CHANGELOG.md                 # Version history
├── CONTRIBUTING.md              # Developer workflow
├── SECURITY.md                  # Security policy
├── docs/                        # Technical documentation
│   ├── archive/                 # Historical docs (dated)
│   ├── AD1_ARCHITECTURE_DIAGRAMS.md
│   ├── AD1_FILE_STRUCTURE_COMPLETE.md
│   ├── LIBEWF_ANALYSIS.md
│   ├── SEVENZIP_FFI_API_REFERENCE.md
│   └── UI_CONSTANTS_REFERENCE.md
├── src/                         # Frontend (SolidJS + TypeScript)
├── src-tauri/                   # Backend (Rust + Tauri)
├── libewf-ffi/                  # Safe Rust FFI bindings for libewf 20251220 (EWF read/write)
└── sevenzip-ffi/                # C library + Rust FFI for 7z archive creation (LZMA SDK 24.09)
```

## Frontend (`src/`)

```text
src/
├── index.tsx                    # App bootstrap
├── index.css                    # Base styles + Tailwind directives
├── App.tsx                      # Main layout and state wiring
├── App.css                      # App-specific styles
├── AppRouter.tsx                # App shell and routing
├── assets/                      # Branding assets
├── styles/                      # Design system
│   └── variables.css            # CSS custom properties (design tokens)
├── components/                  # UI components
│   ├── EvidenceTree.tsx         # Unified evidence tree
│   ├── HexViewer.tsx            # Hex dump viewer
│   ├── DetailPanel.tsx          # Tabbed detail panel
│   ├── report/                  # Report wizard UI
│   └── export/                  # Export UI
│       ├── PhysicalImageMode.tsx # E01 disk image creation panel
│       ├── LogicalImageMode.tsx  # L01 logical evidence container panel
│       ├── NativeExportMode.tsx  # 7z archive + file export with forensic presets
│       ├── ToolsMode.tsx         # Archive test/repair/validate tools
│       ├── SplitSizeSelector.tsx # Shared split/segment size dropdown (9 presets + Custom)
│       ├── CaseMetadataSection.tsx # Shared collapsible case metadata inputs
│       └── DriveSelector.tsx     # Modal drive picker with read-only mount toggle
├── hooks/                       # State and Tauri integration
│   ├── useFileManager.ts        # Evidence file management
│   ├── useHashManager.ts        # Hash computation
│   ├── useProject.ts            # Project persistence
│   ├── useMenuActions.ts        # Native menu bar event bridge
│   └── project/                 # Project sub-hooks (IO, state, bookmarks)
├── api/                         # Backend API wrappers
│   ├── ewfExport.ts             # E01/EWF creation + read metadata API
│   ├── l01Export.ts             # L01 logical evidence creation API
│   ├── drives.ts               # Drive enumeration + read-only mount API
│   ├── lzmaApi.ts               # LZMA/LZMA2 compress/decompress API
│   └── archiveCreate.ts         # 7z archive creation API
├── constants/                   # Application constants
├── extensions/                  # Extension registry and types
├── report/                      # Report API + types
├── types/                       # Domain types and registries
└── utils/                       # Utility helpers
    ├── telemetry.ts             # Usage telemetry (version from build)
    ├── performance.ts           # Performance monitoring
    ├── operationProfiler.ts     # Operation profiling
    ├── logger.ts                # Structured logging
    ├── accessibility.ts         # Accessibility utilities
    └── platform.ts              # Platform detection
```

## Styling Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                    CSS Architecture                              │
├─────────────────────────────────────────────────────────────────┤
│  1. variables.css    → Design tokens (CSS custom properties)     │
│  2. tailwind.config  → Tailwind theme extension (uses tokens)    │
│  3. index.css        → Base styles + @tailwind directives        │
│  4. App.css          → App-specific styles + fonts               │
└─────────────────────────────────────────────────────────────────┘
```

**Key files:**
- `src/styles/variables.css` - Single source of truth for colors, spacing, sizing
- `tailwind.config.js` - Extends Tailwind with CSS variable references
- `src/index.css` - Base layer, components layer, utilities
- `postcss.config.js` - PostCSS plugins (Tailwind, Autoprefixer)

**Color system:** Use semantic color classes (`bg-bg`, `text-txt`, `border-border`) that reference CSS variables for theme support.

**Container type colors:** `type-ad1`, `type-e01`, `type-l01`, `type-raw`, `type-ufed`, `type-archive`

## Backend (`src-tauri/src/`)

```text
src-tauri/src/
├── main.rs                      # Tauri app entry
├── lib.rs                       # Tauri commands (IPC surface)
├── menu.rs                      # Native menu bar (File, Edit, View, Tools, Window, Help)
├── formats.rs                   # Format registry and detection
├── logging.rs                   # Logging/tracing setup
├── database.rs                  # SQLite case DB
├── project/                     # Project management
│   ├── mod.rs                   # .cffx persistence and project types
│   ├── merge.rs                 # Project merge logic (analyze, execute, database merge)
│   ├── io.rs                    # Project I/O operations
│   ├── migration.rs             # Project format migration
│   └── types.rs                 # Project types
├── project_comparison.rs        # Project diff/comparison
├── project_db/                  # Per-project .ffxdb database
│   ├── mod.rs                   # Database open/close, WAL checkpoint
│   ├── schema.rs                # Schema definitions and migrations
│   ├── types.rs                 # Database record types
│   ├── bookmarks.rs             # Bookmarks and notes CRUD
│   ├── forensic.rs              # COC items, amendments, audit log
│   ├── collections.rs           # Evidence collections and collected items
│   ├── evidence.rs              # Evidence files and hashes
│   ├── activity.rs              # Activity log and sessions
│   ├── search.rs                # Saved/recent searches, FTS
│   ├── processed.rs             # Processed database records
│   ├── workflow.rs              # Reports, exports, case documents
│   └── utilities.rs             # Tags, UI state, forms
├── project_recovery.rs          # Project recovery utilities
├── project_templates.rs         # Project templates
├── raw.rs                       # Raw image support
├── activity_timeline.rs         # Activity trend analysis
├── workspace_profiles.rs        # Workspace profile management
│
├── commands/                    # Tauri command handlers
│   ├── mod.rs                   # Command module exports
│   ├── container.rs             # Container operations (V1 + V2)
│   ├── lazy_loading.rs          # Unified lazy loading
│   ├── archive/                 # Archive tree/extraction
│   │   ├── mod.rs               # Archive module exports
│   │   ├── metadata.rs          # Archive metadata listing
│   │   ├── extraction.rs        # Archive extraction
│   │   ├── nested.rs            # Nested container support (VFS/AD1/Archive parents)
│   │   └── tools.rs             # Archive tools (test/repair/validate + LZMA compress/decompress)
│   ├── archive_create.rs        # Archive creation (7z)
│   ├── ufed.rs                  # UFED operations
│   ├── ewf.rs                   # EWF/E01 operations
│   ├── ewf_export.rs            # EWF image creation (via libewf-ffi)
│   ├── l01_export.rs            # L01 logical evidence creation (pure-Rust)
│   ├── raw.rs                   # Raw disk operations
│   ├── vfs.rs                   # Virtual filesystem
│   ├── hash.rs                  # Batch hashing
│   ├── system.rs                # System monitoring, drives, read-only mount
│   ├── analysis.rs              # File byte reading
│   ├── database.rs              # SQLite persistence
│   ├── project.rs               # Project file handling
│   ├── project_advanced.rs      # Advanced project features
│   ├── project_db/              # Per-project .ffxdb commands (80+ IPC)
│   │   ├── mod.rs, bookmarks.rs, forensic.rs, collections.rs
│   │   ├── evidence.rs, activity.rs, search.rs, processed.rs
│   │   └── utilities.rs, workflow.rs
│   ├── project_extended.rs      # Extended project commands
│   ├── viewer.rs                # File viewer commands
│   ├── discovery.rs             # Path discovery
│   └── export.rs                # File export
│
├── common/                      # Shared utilities
│   ├── hash.rs                  # Hash algorithms
│   ├── binary.rs                # Binary parsing helpers
│   ├── segments.rs              # Multi-segment handling
│   ├── segment_hash.rs          # Segment-level hashing
│   ├── path_security.rs         # Path traversal protection
│   ├── io_adaptive.rs           # Adaptive I/O buffer sizing
│   ├── io_pool.rs               # I/O thread pool
│   ├── hash_cache.rs            # Hash result caching
│   ├── lazy_loading.rs          # Lazy loading utilities
│   ├── vfs.rs                   # Virtual filesystem helpers
│   ├── magic.rs                 # File magic detection
│   ├── container_detect.rs      # Container format detection
│   ├── hex.rs                   # Hex formatting
│   ├── datetime.rs              # Date/time utilities
│   ├── progress.rs              # Progress tracking
│   ├── audit.rs                 # Audit logging
│   ├── retry.rs                 # Retry logic
│   ├── health.rs                # Health monitoring
│   ├── metrics.rs               # Metrics collection
│   └── filesystem/              # Filesystem drivers
│       ├── traits.rs            # FilesystemDriver trait
│       ├── partition.rs         # GPT/MBR parsing
│       ├── apfs_driver.rs       # APFS (dir + extent read)
│       ├── hfsplus_driver.rs    # HFS+ driver
│       ├── ntfs_driver.rs       # NTFS driver
│       ├── ext_driver.rs        # ext2/3/4 driver
│       ├── fat.rs               # FAT12/16/32 driver
│       └── dmg_driver.rs        # DMG container
│
├── containers/                  # Container abstraction layer
│   ├── mod.rs                   # Public API
│   ├── types.rs                 # ContainerInfo, ContainerKind
│   ├── operations.rs            # Info/verify/extract dispatch
│   ├── scanning.rs              # Directory scanning
│   ├── traits.rs                # Evidence container traits
│   ├── impls.rs                 # Trait implementations
│   ├── segments.rs              # Multi-segment helpers
│   ├── companion.rs             # Companion log detection
│   ├── case_documents.rs        # Case document scanning
│   └── unified.rs               # Unified container operations
│
├── viewer/                      # File viewing subsystem
│   ├── mod.rs                   # Hex/text viewing + exports
│   ├── types.rs                 # FileChunk, FileTypeInfo
│   ├── parsers/                 # Format header parsers
│   │   ├── ad1.rs, ewf.rs       # Container headers
│   │   ├── archives.rs          # Archive headers
│   │   └── disk_images.rs       # Disk image headers
│   └── document/                # Content viewers
│       ├── mod.rs               # Document service
│       ├── universal.rs         # Universal format detection
│       ├── commands.rs          # Tauri viewer commands
│       ├── types.rs             # Document types
│       ├── error.rs             # Error handling
│       ├── pdf.rs               # PDF viewing
│       ├── docx.rs              # Word documents
│       ├── html.rs              # HTML viewing
│       ├── markdown.rs          # Markdown viewing
│       ├── exif.rs              # EXIF metadata extraction
│       ├── email.rs             # EML/MBOX parsing
│       ├── binary.rs            # PE/ELF/Mach-O analysis
│       ├── plist_viewer.rs      # Apple plist parsing
│       └── spreadsheet.rs       # Excel/CSV/ODS
│
├── ad1/                         # AD1 parser and VFS
├── ewf/                         # E01/Ex01/L01 parser (pure-Rust, read-only)
├── l01_writer/                  # Pure-Rust L01 logical evidence writer
│   ├── mod.rs                   # L01Writer (add_file, add_directory, write)
│   ├── types.rs                 # L01WriterConfig, LefFileEntry, LefSource, L01WriteResult
│   ├── chunks.rs                # zlib chunk compression (32 KB chunks)
│   ├── sections.rs              # EWF v1 section writers (header, volume, sectors, table, ltree, etc.)
│   ├── segment.rs               # Multi-segment file support
│   └── ltree.rs                 # UTF-16LE ltree text builder (5 categories)
├── ufed/                        # UFED extraction parsing
├── archive/                     # Archive metadata + ZIP extraction
├── processed/                   # Processed DB detection + AXIOM
└── report/                      # Report generation pipeline
```

### libewf-ffi (Workspace Crate)

```text
libewf-ffi/                      # Safe Rust FFI bindings for libewf 20251220
├── src/
│   ├── lib.rs                   # Crate root, re-exports
│   ├── ffi.rs                   # Raw FFI bindings to libewf C functions
│   ├── reader.rs                # EwfReader — read/verify EWF images
│   ├── writer.rs                # EwfWriter — create EWF images
│   └── error.rs                 # EwfError type
├── tests/
│   ├── read_test.rs             # Reader tests (22 tests)
│   └── write_test.rs            # Writer tests (16 tests)
└── Cargo.toml
```

**Two EWF modules exist — do NOT confuse:**
- `libewf-ffi/` — C FFI wrapper for EWF **creation** (write) and **metadata reading**
- `src-tauri/src/ewf/` — Pure-Rust EWF **parser** for container **browsing, verify, VFS**

### sevenzip-ffi (Workspace Crate)

```text
sevenzip-ffi/                    # C library + Rust FFI for 7z archive creation (LZMA SDK 24.09)
├── src/
│   ├── lib.rs                   # Rust FFI wrapper (SevenZip, CompressOptions, StreamOptions)
│   ├── archive_create.c         # Single-volume 7z archive creation
│   ├── archive_create_multivolume.c  # Split/multi-volume 7z archive creation
│   ├── utf8_utf16.h             # UTF-8 → UTF-16LE filename encoding
│   └── ffi_interface.c          # FFI entry points
├── include/
│   └── 7z_ffi.h                 # Public C API header
├── lzma/C/                      # LZMA SDK 24.09 C source files (86 files)
├── build/
│   └── lib7z_ffi.a              # Pre-built static library (macOS arm64)
├── tests/                       # Rust integration tests
├── build.rs                     # Cargo build script
├── CMakeLists.txt               # C library build configuration
└── Cargo.toml
```

**Critical invariants:**
- UTF-16LE filenames via `utf8_to_utf16le()` — NEVER use ASCII-only loop
- Dictionary sizes must match SDK 24.09 defaults
- Dynamic header allocation (no fixed buffers)
- LZMA SDK version 24.09 — do NOT downgrade

## Runtime Flows

### Evidence Scan

1. UI selects a directory
2. `scan_directory*` commands discover containers
3. Backend emits `scan-file-found` events
4. UI builds discovered file list and loads `ContainerInfo`

### Container Browsing

1. UI calls `logical_info` or `logical_info_fast`
2. Backend detects format and extracts metadata
3. AD1 tree is lazy-loaded via V2 APIs (`container_get_root_children_v2`, `container_get_children_at_addr_v2`)

### Hash Verification

1. UI selects algorithm
2. Backend hashes container (format-specific commands)
3. Backend emits `verify-progress` or `batch-progress`

### File Viewing

1. UI requests file info via `universal_get_info`
2. Backend detects format and returns viewer hint
3. Frontend renders with appropriate viewer component
4. Content fetched via `universal_read_text`, `universal_read_data_url`, etc.

### Reports

1. UI builds a report model
2. Backend generates PDF/DOCX/HTML/Markdown
3. Optional Typst and AI features are gated by build flags

## Type Alignment (Frontend ↔ Backend)

Keep TypeScript and Rust types synchronized:

| Frontend | Backend |
|----------|---------|
| `src/types.ts` | `src-tauri/src/formats.rs`, `src-tauri/src/containers/types.rs` |
| `src/types/processed.ts` | `src-tauri/src/processed/types.rs` |
| `src/types/project.ts` | `src-tauri/src/project/types.rs` |
| `src/types/projectDb.ts` | `src-tauri/src/project_db/types.rs`, `src-tauri/src/commands/project_db/` |
| `src/types/database.ts` | `src-tauri/src/database.rs` |
| `src/report/types.ts` | `src-tauri/src/report/types.rs` |
| `src/api/ewfExport.ts` | `src-tauri/src/commands/ewf_export.rs` |
| `src/api/l01Export.ts` | `src-tauri/src/commands/l01_export.rs`, `src-tauri/src/l01_writer/types.rs` |
| `src/api/drives.ts` | `src-tauri/src/commands/system.rs` (DriveInfo, MountResult) |
| `src/api/lzmaApi.ts` | `src-tauri/src/commands/archive/tools.rs` |
| `src/api/archiveCreate.ts` | `src-tauri/src/commands/archive_create.rs` |
| `src/api/projectMerge.ts` | `src-tauri/src/project/merge.rs`, `src-tauri/src/commands/project_merge.rs` |

## Glossary

| Term | Definition |
|------|------------|
| **Artifact** | A record extracted from a processed database |
| **Container** | A forensic evidence file (AD1, E01, L01, Raw, UFED, Archive) |
| **ContainerInfo** | Unified metadata for a container |
| **ContainerKind** | Enum identifying the container type |
| **DiscoveredFile** | A container found during directory scanning |
| **DriveInfo** | OS disk/volume metadata (device, mount, size, read-only, system disk) |
| **Evidence tree** | Lazy-loaded AD1 file tree (addresses for hex navigation) |
| **Extension** | A plugin with a manifest and category (viewer, parser, exporter) |
| **Project file** | `.cffx` session state file |
| **Segment** | One file in a multi-part container (.E01, .E02, .ad2) |
| **SplitSizeSelector** | Shared UI for split/segment size (values in MB, backend expects bytes) |
| **UniversalFormat** | Detected file format for viewer routing |
| **ViewerType** | Category of viewer (Text, Image, Pdf, Binary, etc.) |

## Invariants

- **Read-only evidence**: Source files are never modified
- **Path traversal sanitization**: Backend utilities validate all paths
- **Hash verification**: Prefer stored hashes when present
- **Progress events**: Long operations emit events for UI responsiveness
- **Export defaults**: All compression = "none"/Store, all splits = 2048 MB (2 GB)
- **Split size units**: Frontend stores MB, backend expects bytes — always convert with `* 1024 * 1024`
- **Drive mount restoration**: Always restore drive mount state in `.finally()` after imaging
- **Library name**: Use `ffx_check_lib` for test imports

## Documentation References

### Frontend

- `src/components/README.md` - Component catalog
- `src/hooks/README.md` - State management hooks
- `src/styles/README.md` - Tailwind CSS styling guide
- `src/report/README.md` - Report API
- `src/extensions/README.md` - Extension system
- `src/utils/README.md` - Utility helpers

### Backend

- `src-tauri/src/README.md` - Backend module reference
- `src-tauri/src/containers/README.md` - Container abstraction
- `src-tauri/src/report/README.md` - Report generation
- `docs/formats/FFX_PROJECT_FORMAT.md` - Project file schema
- `docs/formats/AXIOM_DATABASE_BIBLE.md` - AXIOM reference

### AI Coding

- `.github/copilot-instructions.md` - AI agent guidance

---

*Last updated: March 6, 2026*
