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
- **Storage**: SQLite case DB + `.cffx` session files

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
│   └── UI_CONSTANTS_REFERENCE.md
├── src/                         # Frontend (SolidJS + TypeScript)
└── src-tauri/                   # Backend (Rust + Tauri)
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
│   └── report/                  # Report wizard UI
├── hooks/                       # State and Tauri integration
│   ├── useFileManager.ts        # Evidence file management
│   ├── useHashManager.ts        # Hash computation
│   ├── useProject.ts            # Project persistence
│   └── project/                 # Project sub-hooks (IO, state, bookmarks)
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
├── formats.rs                   # Format registry and detection
├── logging.rs                   # Logging/tracing setup
├── database.rs                  # SQLite case DB
├── project.rs                   # .cffx persistence
├── project_comparison.rs        # Project diff/comparison
├── project_recovery.rs          # Project recovery utilities
├── project_statistics.rs        # Project statistics
├── project_templates.rs         # Project templates
├── raw.rs                       # Raw image support
├── activity_timeline.rs         # Activity trend analysis
├── session_analytics.rs         # Session analytics
├── workspace_profiles.rs        # Workspace profile management
│
├── commands/                    # Tauri command handlers
│   ├── mod.rs                   # Command module exports
│   ├── container.rs             # Container operations (V1 + V2)
│   ├── lazy_loading.rs          # Unified lazy loading
│   ├── archive/                 # Archive tree/extraction
│   ├── archive_create.rs        # Archive creation (7z)
│   ├── ufed.rs                  # UFED operations
│   ├── ewf.rs                   # EWF/E01 operations
│   ├── raw.rs                   # Raw disk operations
│   ├── vfs.rs                   # Virtual filesystem
│   ├── hash.rs                  # Batch hashing
│   ├── system.rs                # System monitoring
│   ├── analysis.rs              # Hex dump, entropy
│   ├── database.rs              # SQLite persistence
│   ├── project.rs               # Project file handling
│   ├── project_advanced.rs      # Advanced project features
│   ├── project_extended.rs      # Extended project commands
│   ├── viewer.rs                # File viewer commands
│   ├── discovery.rs             # Path discovery
│   ├── unified.rs               # Unified container API
│   ├── export.rs                # File export
│   ├── search.rs                # Search operations
│   ├── index.rs                 # Index cache commands
│   ├── streaming_extract.rs     # Streaming extraction
│   ├── parallel_extract.rs      # Parallel extraction
│   ├── deduplication.rs         # File deduplication
│   ├── recovery.rs              # Recovery operations
│   ├── regression.rs            # Regression testing commands
│   ├── mmap_hex.rs              # Memory-mapped hex viewer
│   ├── profiler.rs              # CPU profiling commands
│   ├── memory_profiler.rs       # Memory profiling commands
│   └── observability.rs         # Metrics/health commands
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
│   ├── hash_queue.rs            # Priority hash queue
│   ├── index_cache.rs           # Container index cache
│   ├── index_worker.rs          # Background indexing
│   ├── lazy_loading.rs          # Lazy loading utilities
│   ├── vfs.rs                   # Virtual filesystem helpers
│   ├── magic.rs                 # File magic detection
│   ├── container_detect.rs      # Container format detection
│   ├── entropy.rs               # Entropy calculation
│   ├── hex.rs                   # Hex formatting
│   ├── datetime.rs              # Date/time utilities
│   ├── progress.rs              # Progress tracking
│   ├── notifications.rs         # Notification helpers
│   ├── audit.rs                 # Audit logging
│   ├── retry.rs                 # Retry logic
│   ├── recovery.rs              # Recovery utilities
│   ├── health.rs                # Health monitoring
│   ├── metrics.rs               # Metrics collection
│   ├── profiler.rs              # CPU profiler
│   ├── memory_profiler.rs       # Memory profiler
│   ├── regression.rs            # Regression detection
│   ├── tracing_setup.rs         # Tracing configuration
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
├── ewf/                         # E01/Ex01/L01 parser
├── ufed/                        # UFED extraction parsing
├── archive/                     # Archive metadata + ZIP extraction
├── processed/                   # Processed DB detection + AXIOM
└── report/                      # Report generation pipeline
```

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
| `src/types/project.ts` | `src-tauri/src/project.rs` |
| `src/report/types.ts` | `src-tauri/src/report/types.rs` |

## Glossary

| Term | Definition |
|------|------------|
| **Artifact** | A record extracted from a processed database |
| **Container** | A forensic evidence file (AD1, E01, L01, Raw, UFED, Archive) |
| **ContainerInfo** | Unified metadata for a container |
| **ContainerKind** | Enum identifying the container type |
| **DiscoveredFile** | A container found during directory scanning |
| **Evidence tree** | Lazy-loaded AD1 file tree (addresses for hex navigation) |
| **Extension** | A plugin with a manifest and category (viewer, parser, exporter) |
| **Project file** | `.cffx` session state file |
| **Segment** | One file in a multi-part container (.E01, .E02, .ad2) |
| **UniversalFormat** | Detected file format for viewer routing |
| **ViewerType** | Category of viewer (Text, Image, Pdf, Binary, etc.) |

## Invariants

- **Read-only evidence**: Source files are never modified
- **Path traversal sanitization**: Backend utilities validate all paths
- **Hash verification**: Prefer stored hashes when present
- **Progress events**: Long operations emit events for UI responsiveness
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

*Last updated: February 11, 2026*
