# CORE-FFX Changelog

All notable changes to CORE-FFX are documented here. Format follows Keep a Changelog and Semantic Versioning.

## [Unreleased]

*No unreleased changes.*

## [0.1.18] - 2026-03-01

### Fixed

- **Auto-updater**: Enable updater bundle generation via `createUpdaterArtifacts: v1Compatible` in bundle config
- **Auto-updater**: Fix double-base64 encoded signing pubkey in tauri.conf.json
- **Auto-updater**: Disable `strip` entirely (`strip = false`) to preserve `__TAURI_BUNDLE_TYPE` symbol for updater
- **Auto-updater**: Regenerate signing keys without password for simpler CI integration
- **Auto-updater**: Remove `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from CI workflow — empty env var was treated as wrong password
- **Auto-updater**: Add GitHub PAT auth header support for update checks (graceful fallback when public)

### Changed

- **Documentation**: Comprehensive audit of all repository docs (README, SECURITY, CONTRIBUTING, CHANGELOG, THIRD_PARTY_LICENSES, HELP, CODE_BIBLE)

## [0.1.14] - 2026-03-01

### Added

- **E01 Export UI** — new `EwfExportMode.tsx` component and full integration
  into `ExportPanel.tsx` as a fourth export mode:
  - EnCase 5/6/7 and V2 (Ex01/Lx01) format selection via grouped dropdown
  - Compression level (none/fast/best) and method (Deflate/BZIP2 with V2 guard)
  - MD5/SHA1 hash checkboxes for forensic verification
  - Case metadata section (case number, evidence number, examiner, description, notes)
  - Advanced options (segment file size for multi-part E01 output)
  - Full activity tracking and progress events via `ewf-export-progress`
- **EWF Image Info Reader** — new `ewf_read_image_info` Tauri command using
  libewf-ffi `EwfReader` for extracting detailed E01/Ex01/L01 metadata:
  - Format detection (19 EWF format variants with name, extension, V2/logical flags)
  - Media parameters (size, bytes/sector, sectors/chunk, compression level/method)
  - Case metadata (case number, evidence number, examiner, acquisition info)
  - Stored MD5/SHA1 hashes, corruption/encryption flags, segment file version
  - Frontend API: `readEwfImageInfo()` in `ewfExport.ts` with full TypeScript types
- **LZMA/LZMA2 Raw Compression** — enabled 4 new Tauri commands from sevenzip-ffi:
  - `compress_to_lzma` / `decompress_lzma` — standalone .lzma file compression
  - `compress_to_lzma2` / `decompress_lzma2` — standalone .xz file compression
  - All use `tokio::task::spawn_blocking` for non-blocking async operation
- **Encrypted 7z Fallback** — added sevenzip-ffi as middle fallback in
  `archive/sevenz.rs` `list_entries()`:
  - Chain: libarchive → sevenzip-ffi → sevenz-rust
  - Enables listing encrypted 7z archives that libarchive cannot handle
  - Converts sevenzip-ffi `ArchiveEntry` to internal `ArchiveEntry` with timestamp formatting

- **E01/EWF Image Creation** — new `ewf_export.rs` command module for creating
  forensic disk images via libewf-ffi:
  - EnCase 5/6/7 and V2 (Ex01/Lx01) format support
  - Deflate and BZIP2 compression methods
  - Full case metadata (case number, evidence number, examiner, notes)
  - Streaming MD5/SHA1 hash computation during write
  - Progress events and cancellation support
  - 47 unit tests covering format parsing, compression parsing, serialization,
    and critical format-extension invariants
- **7z Archive Creation** — new `archive_create.rs` command module for creating
  forensic-grade 7z archives via sevenzip-ffi:
  - AES-256 encryption, multi-threading, split volumes
  - Streaming compression for large archives (>1GB auto-streaming, >8GB auto-split)
  - Forensic JSON manifest with per-file SHA-256/MD5/SHA-1 hashes
  - Chain-of-custody metadata (examiner, case number, evidence description)
  - Post-creation archive integrity verification
  - Progress events and cancellation support
  - 42 unit tests covering defaults, file collection, compression mapping,
    serialization, and cancel flag logic
- **Frontend Export APIs** — `src/api/ewfExport.ts` and `src/api/archiveCreate.ts`
  TypeScript wrappers for E01 and 7z creation commands
- **Enhanced ArchiveMode UI** — `src/components/export/ArchiveMode.tsx` expanded
  with compression level selector, encryption, split archives, and forensic
  manifest options

### Changed

- **LZMA SDK 24.09** — upgraded sevenzip-ffi from LZMA SDK 23.01 to 24.09:
  - Matches Homebrew's `sevenzip` formula version
  - Auto-detects `MY_CPU_ARM64` from `__aarch64__` (no manual CMake define needed)
  - Updated dictionary size defaults to match SDK 24.09 specifications
- **sevenzip-ffi Flat Layout** — restructured `sevenzip-ffi/` workspace crate
  from nested `sevenzip-ffi/rust/` subdirectory to flat layout matching
  `libewf-ffi/` pattern (Cargo.toml, build.rs, src/ at crate root)

### Fixed

- **Cellebrite parser warning** — removed unused `std::io::Write` import in
  `cellebrite.rs` (only `fs::write` was used, not the `Write` trait); zero
  cargo warnings confirmed

### Added

- **Activity Tracking Coverage** — closed remaining activity-tracking gaps:
  system sessions, file/view, file/close, db/open, note/delete, export/start
- **Test Suite Expansion** — 272 new tests across parsers, viewers, and adapters:
  - 145 tests for viewer/parsers (AD1, ZIP, 7z, RAR, GZIP, raw, VMDK, VHDx, QCOW2, EWF) and hash types
  - 127 tests for adapters, pdf.rs, docx.rs; fixed DOCX heading parsing bug
- **Document/Binary Viewers & Toolbar** — new viewers and container operation
  improvements
- **Per-Project .ffxdb Database** — SQLite database per project (schema v3)
  for bookmarks, notes, tags, sessions, activity logs, saved searches, and
  FTS5 full-text search
- **Write-Through Sync** — `useProjectDbSync` hook mirrors .cffx state to
  .ffxdb in real time (bookmarks, notes, tags, sessions, activity, evidence
  files, hashes, verifications, reports, saved searches, UI state)
- **Database Seeding** — `seedDatabaseFromProject()` populates .ffxdb tables
  from loaded .cffx project state on first open (idempotent)
- **Evidence & Hash Wiring** — evidence file discovery, hash computation, and
  cache seeding automatically flow into .ffxdb
- **dbSync Integration** — write-through wired into report export, case
  documents, tags, and search operations
- **Report Auto-Population** — report wizard fields pre-filled from project
  data; expanded preferences for examiner defaults
- **Project Dashboard** — new "Dashboard" tab in left panel with project
  overview, stats, recent activity, and quick actions
- **Cross-Entity FTS5 Search** — unified full-text search across bookmarks,
  notes, tags, activity logs, evidence files, and hashes via .ffxdb FTS5
  virtual tables
- **Forensic Timeline Export** — comprehensive timeline export to CSV, JSON,
  and HTML formats with filtering, date-range selection, and activity-type
  grouping
- **L01 Logical VFS** — `open_logical()` constructor and auto-detect for
  L01/Lx01 extensions; exposes embedded data stream as single virtual file
- **Lazy Loading Full Coverage** — memory dump containers return single virtual
  file entry; unknown formats return empty result instead of error
- **PDF Real Font Loading** — `with_font()` loads custom fonts from disk via
  `genpdf::fonts::from_files()` instead of stubbing
- **Archive VFS Descriptive Errors** — unsupported formats now return specific
  error messages (single-stream compression, disk images, no-directory formats)
- **RecentProjectsList Show All** — "Show all" button toggles between truncated
  and full recent projects list

### Changed

- **FRONTEND_API_NOTES** — added `"dashboard"` to `leftPanelTab` accessor type
- **FAT Driver** — updated module doc comment to reflect fully-implemented
  status (was incorrectly marked as placeholder)

### Documentation

- Added .ffxdb / project_db coverage across all project documentation
- Comprehensive CHANGELOG entries for 14 previously undocumented commits

### Fixed

- **Compile errors:** Fixed `test_archive()` signature across
  9 test/example files in both CORE-1 and standalone sevenzip-ffi
  repos (added missing progress callback argument)
- **Critical:** Fixed 7-Zip FFI library segfault (SIGSEGV) caused by uninitialized CRC table
  - Root cause: `sevenzip_init()` in `sevenzip-ffi/src/ffi_interface.c` failed to call `CrcGenerateTable()`
  - This left the `g_CrcUpdate` function pointer as NULL, causing null pointer dereference at address 0x0
  - Added `CrcGenerateTable()` call to library initialization (crash report D3614DF6-E094-46FC-9C94-2583D1BFA2C2)
  - FFI library now stable for all archive sizes and compression modes
  - Eliminated system 7z fallback requirement - native library works reliably

### Added

- **Export hash verification** — `database.rs` gains
  `lookup_known_hash_by_path()` (SQL JOIN across files→hashes);
  `export.rs` verifies exported file hashes against known DB values
- **Encrypted DMG detection** — `DmgDriver::is_encrypted()` checks
  for `encrcdsa` magic bytes; metadata returns early with
  `encrypted: true` instead of crashing on AES-256 DMGs
- **Health queue monitoring** — `QueueMetricsRegistry` with
  lock-free atomics; `batch_hash_smart` reports real queue
  metrics; alerts for high depth (>100) and stalled queues
- **Health error type tracking** — `collect_error_metrics()` now
  parses Prometheus-format metric keys to extract per-type
  error breakdowns (top 10 by count)
- **Activity timeline trends** — per-type trend analysis computing
  first-half vs second-half averages with increasing/decreasing/
  stable classification
- **App cancel logic** — operation cleanup registry with
  `registerOperationCleanup`/`unregisterOperationCleanup` helpers
- **Telemetry app version** — `__APP_VERSION__` injected from
  `package.json` at build time via Vite `define`, replacing
  hardcoded `"0.1.0"`; added `@types/node` dev dependency
- **APFS file extent reading** — `find_file_extents()` traverses
  catalog B-tree for `J_FILE_EXTENT_TYPE` records; `read()`
  maps logical offsets to physical blocks for data extraction
- Report wizard with preview and export (PDF, DOCX, HTML, Markdown; Typst optional)
- Expanded hash algorithm options (SHA-1/256/512, MD5, BLAKE3, BLAKE2b, XXH3, XXH64, CRC32)
- Processed database discovery for multiple tools (AXIOM parsing implemented)
- Archive metadata parsing and ZIP extraction utilities
- **Adaptive I/O Buffer Sizing** - Intelligent buffer management for forensic operations:
  - Dynamic buffer sizing (512KB → 32MB) based on file size and operation type
  - Operation-specific strategies: Hash (max throughput), Read (balance latency/throughput), Stream (min latency)
  - Adaptive progress chunk sizing (10-100 updates) for responsive UI without performance overhead
  - Integrated into hash operations (common/hash.rs, AD1 multi-segment hashing)
  - Performance: Expected 20-30% improvement for large file hashing
  - Added `AdaptiveBuffer` utility module with comprehensive test suite (8/8 passing)
- **Smart Hash Queue** - Priority-based job scheduling for parallel batch hashing:
  - Priority scheduling: Quick (< 100MB) → Normal (100MB-1GB) → Low (1-10GB) → Background (> 10GB)
  - Small files processed first for immediate user feedback ("quick wins")
  - Adaptive concurrency: 1.5x CPU cores for I/O-bound workloads
  - Real-time throughput tracking (MB/s) and ETA prediction
  - Worker management with dynamic scaling based on queue depth
  - Queue statistics: completion tracking, failure rates, aggregate performance
  - Added `HashQueue` module (478 lines) with 5/5 tests passing
  - New `batch_hash_smart` command using priority queue (registered in lib.rs)
- **LRU Cache Dependency** - Added `lru = "0.12"` for future memory-mapped viewer enhancements
- **Virtual Scrolling Library** - Installed @tanstack/solid-virtual@^3.10.0 for future FilePanel optimization
- **Incremental Container Indexing** - SQLite-backed metadata cache for fast repeated access:
  - Persistent cache of container file lists with automatic mtime-based invalidation
  - Lightweight summary API (total files/dirs/size) without loading full index
  - Connection-per-operation pattern ensures thread-safe SQLite access in async context
  - Batch index storage with transaction support for efficiency
  - Cache management: invalidation, statistics (DB size, entry counts), full clear
  - Added `IndexCache` module with 2/2 tests passing
  - New Tauri commands: `index_cache_init`, `index_cache_has_index`, `index_cache_get_summary`, `index_cache_store`, `index_cache_load`, `index_cache_invalidate`, `index_cache_stats`, `index_cache_clear`
  - Future: Background workers for automatic index building on container open

### Changed

- Documentation sweep and consolidation
- Clarified format support (full parsing vs detection-only)
- **I/O Performance**: Replaced fixed 16MB BUFFER_SIZE with adaptive sizing in hash operations
- LZMA commands: clarified as blocked on upstream Rust bindings
  (not a TODO — tracked externally)
- Replaced stale `@ts-expect-error` in `vite.config.ts`
  (no longer needed after `@types/node` install)
- `tsconfig.node.json`: added `"types": ["node"]`

### Documentation

- **common/README.md**: comprehensive rewrite covering all 30+
  modules — core utilities, I/O performance, monitoring,
  resilience, and filesystem drivers
- **CODE_BIBLE.md**: added missing backend files
  (`activity_timeline.rs`, `session_analytics.rs`,
  `workspace_profiles.rs`, `project_*.rs`), expanded
  filesystem driver sub-tree, added frontend utils
- **CHANGELOG.md**: documented all TODO resolutions

### Improved

- **Project Save/Restore System** - Enhanced state persistence for better session continuity:
  - Save and restore selected container entry (resume viewing files inside containers)
  - Save and restore entry content view mode (auto/hex/text/document preference)
  - Save and restore case documents path (maintain document search location)
  - Save and restore filter state (type filter for evidence tree)
  - Save and restore processed database detail view type (artifacts/timeline/etc)
  - Save and restore tree expansion state (remember which containers/folders were expanded)
  - Added comprehensive console logging for debugging restoration
  - Renumbered restoration steps (1-9) with clear section headers for maintainability
- **Hash Performance**: Adaptive buffer sizing provides ~20-30% faster hashing for large forensic containers
  - Small files (< 10MB): 512KB-2MB buffers (reduced memory footprint)
  - Medium files (10MB-100MB): 2MB-8MB buffers (balanced performance)
  - Large files (100MB-1GB): 8MB-16MB buffers (increased throughput)
  - Huge files (> 1GB): 32MB buffers (maximum throughput)
  - Progress updates scale from 10 chunks (small) to 100 chunks (huge) for smooth UI
- **Smart Queue Performance**: Priority scheduling improves user experience for batch operations
  - Small files complete quickly providing immediate feedback
  - Large files queued with lower priority to avoid blocking
  - Adaptive worker scaling prevents system overload
  - Throughput tracking enables accurate ETA predictions
  - Queue depth monitoring for system health visibility

## [0.1.0] - 2024-01-01

### Initial Features

- Initial release
- AD1 logical evidence container support
- E01/Ex01 Expert Witness Format support
- L01/Lx01 EnCase logical format support
- Raw disk image support (DD, IMG, RAW)
- UFED extraction support
- Hash verification (MD5, SHA1, SHA256)
- Hex and text viewers
- Project file support (.cffx)
- SQLite case database

---

[Unreleased]: https://github.com/tmreyno/CORE/compare/v0.1.18...HEAD
[0.1.18]: https://github.com/tmreyno/CORE/compare/v0.1.14...v0.1.18
[0.1.14]: https://github.com/tmreyno/CORE/compare/v0.1.0...v0.1.14
[0.1.0]: https://github.com/tmreyno/CORE/releases/tag/v0.1.0
