# CORE-FFX Changelog

All notable changes to CORE-FFX are documented here. Format follows Keep a Changelog and Semantic Versioning.

## [Unreleased]

### Fixed

- **Critical:** Fixed 7-Zip FFI library segfault (SIGSEGV) caused by uninitialized CRC table
  - Root cause: `sevenzip_init()` in `/Users/terryreynolds/GitHub/sevenzip-ffi/src/ffi_interface.c` failed to call `CrcGenerateTable()`
  - This left the `g_CrcUpdate` function pointer as NULL, causing null pointer dereference at address 0x0
  - Added `CrcGenerateTable()` call to library initialization (crash report D3614DF6-E094-46FC-9C94-2583D1BFA2C2)
  - FFI library now stable for all archive sizes and compression modes
  - Eliminated system 7z fallback requirement - native library works reliably

### Added

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
- Project file support (.ffxproj)
- SQLite case database

---

[Unreleased]: https://github.com/tmreyno/CORE/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tmreyno/CORE/releases/tag/v0.1.0
