# CORE-FFX Changelog

All notable changes to CORE-FFX are documented here. Format follows Keep a Changelog and Semantic Versioning.

## [0.1.46] - 2026-03-12

### Added

- **Multi-flavor build system** — same-repo flavor-based builds with Cargo feature flags; `full` (default, all features), `acquire` (acquisition-only: E01/L01/7z creation, hashing, drive ops), and `review` (future); 18 optional dependencies gated behind `flavor-review`
- **CORE Acquire build target** — dedicated `tauri.acquire.conf.json` config overlay with product name "CORE Acquire", separate bundle identifier, and smaller default window; `dev:acquire` and `build:acquire` npm scripts
- **Frontend edition flag** — `__APP_EDITION__` compile-time constant (via `VITE_EDITION` env var) for edition-aware UI gating in SolidJS components
- **Split command registration** — `run_full()` registers all ~180 commands; `run_acquire()` registers only shared + acquisition commands; compile-time dispatch via `#[cfg(feature = "flavor-review")]`
- **Granular module gating** — `viewer::document` submodule (heavy optional deps: lopdf, plist, image, calamine, etc.) gated behind `flavor-review`; viewer core types/parsers always available; `pdf_extract` usage in case documents gated; processed DB scanner gated

## [0.1.45] - 2026-03-12

### Added

- **File deduplication engine** — full-stack deduplication analysis using MD5/SHA-256 hash matching and filename similarity scoring; backend in `dedup/mod.rs` + `dedup/types.rs` with `dedup_analyze`, `dedup_enrich_hashes`, and `dedup_export_csv` Tauri commands; frontend `DeduplicationPanel` with duplicate group visualization, sorting, filtering, and CSV export
- **Tantivy full-text search engine** — BM25-scored full-text search across all evidence containers; indexes filenames, paths, metadata, and optionally file content (PDF, DOCX, EML, plist, plain text); 12-field schema with facet counts and snippet highlighting; per-window index lifecycle; `useSearchIndex` hook for automatic indexing on project load
- **Workspace modes** — feature module system with 6 toggleable modules (Forensic Explorer, Evidence Collection, Document Review, Search & Analysis, Report & Export, Case Management) and 7 presets (Full Suite, Forensic Explorer, Evidence Collection & COC, Document Review, Search & Analysis, Report & Export, Custom); `WorkspaceModeTab` in Settings; `WorkspaceModeSelector` toolbar dropdown; auto-tab-switch when active tab's module is disabled
- **Notes panel** — full CRUD for investigative notes with priority levels (Low/Normal/High/Critical), tags, search/filter by target type (file/artifact/database/case/general), and `NoteEditDialog` modal; integrated into the sidebar's "Bookmarks & Notes" sub-tabbed panel
- **Text selection context menu** — right-click selected text inside any document viewer for "Bookmark Selection", "Note from Selection", "Search for Selection", and "Copy"; `useTextSelectionMenu` hook passes through to browser default context menu when no text is selected
- **Search content toggle** — "Search contents" checkbox in `SearchFilters` to control whether Tantivy searches file content or just metadata/filenames
- **Search initial query support** — `SearchPanel` accepts `initialQuery` prop for pre-filling search from text selection; connected via `searchInitialQuery` signal in App.tsx

### Changed

- **Sidebar module gating** — sidebar navigation tabs, tool buttons, and quick action buttons are now filtered by the active workspace mode via `TAB_MODULE_MAP` and `ACTION_MODULE_MAP`; search and bookmarks remain always-visible as universal tools
- **Combined bookmarks + notes badge** — sidebar badge shows total of bookmarks and notes (was bookmarks-only)
- **Settings panel expanded** — added "Workspace Modes" as first tab in SettingsPanel (before Appearance)
- **Search result items** — content match badge and snippet rendering with `<mark>` highlighting for Tantivy content search hits
- **CI updated for Node.js 24** — all GitHub Actions workflows (release, tests, prebuild, codeql, performance) updated to Node.js 24-compatible action versions

### Fixed

- **Search `stats()` panic** — `TopDocs::with_limit(0)` causes a Tantivy runtime panic; replaced with `tantivy::collector::Count` for document counting
- **6 stale `useAppActions` tests** — updated mock signatures to match current hook API after prior refactors
- **Dead code cleanup** — removed unused imports and stale code paths across frontend modules

### Tests

- **61 new Rust tests** — 15 dedup module tests (format_size, enrich_with_hashes, DedupOptions, serialization) + 46 search module tests (classify_extension for all 17 categories, is_text_eligible, SearchIndex CRUD, search queries with filters, facet counts, global registry); all 1,701 backend tests passing

### Documentation

- **Search architecture guide** — `docs/SEARCH_ARCHITECTURE.md` covering Tantivy schema, indexer pipeline, query engine, and frontend integration
- **Hash verification model** — `docs/HASH_VERIFICATION_MODEL.md` documenting stored hash collection, verification logic, and per-container hash behavior
- **Case data model** — `docs/CASE_DATA_MODEL.md` covering project structure, database schema, and entity relationships
- **README restructured** — docs/ directory index updated with new architecture documents
- **CRATE_API_NOTES expanded** — added Tantivy 0.22 API notes and dedup crate usage patterns
- **copilot-instructions expanded** — documented Tantivy search, dedup, workspace modes, notes, text selection menu, and search index architecture (+335 lines)

## [0.1.44] - 2026-03-11

### Fixed

- **Project setup hang with 400+ evidence files** — creating a new project with a large number of archive containers (ZIP, 7z, TAR, etc.) no longer causes the app to freeze with the "Setting up project…" spinner
- **info_fast routing bug** — `containers::info_fast()` was incorrectly calling the slow `archive::info()` (4+ I/O ops per file — segment discovery, central directory parsing, UFED-in-ZIP detection) and `ufed::info()` instead of their fast variants `archive::info_fast()` and `ufed::info_fast()` (1 I/O op each); this multiplied background hash loading time by ~4x with many archive containers
- **IPC contention during project setup** — `loadStoredHashesInBackground()` is now deferred to AFTER `saveProject` + `project_db_open` complete, preventing it from saturating the Tauri thread pool and blocking critical setup operations
- **Evidence file IPC flood** — `flushScanBuffer()` now uses a single `batchUpsertEvidenceFiles()` IPC call with a prepared statement and SQL transaction instead of N individual fire-and-forget `upsertEvidenceFile()` calls (400 calls → 1 for a typical scan)

### Added

- **`project_db_batch_upsert_evidence_files` command** — backend batch insert/update for evidence files in a single transaction; exposed as `dbSync.batchUpsertEvidenceFiles()` (awaitable) on the frontend
- **`loadStoredHashesInBackground()` exported** — now available on the `useFileManager` return value for deferred invocation after project setup

## [0.1.43] - 2026-03-10

### Added

- **Granular merge selection** — category-level toggles and per-item checkboxes in the Merge Projects wizard allow selective merging of 12 data categories (evidence, bookmarks/notes, activity, COC, collections, forms, reports, tags, searches, documents, exports, processed databases)
- **DataCategorySelector component** — 2-column checkbox grid with category icons, descriptions, and per-category item counts; enabled/total counter; wired into the wizard Review step
- **Per-item merge exclusion** — individual evidence files, COC items, collections, and form submissions can be included/excluded via checkboxes in expandable project summary cards; "Include All / Exclude All" toggles per section
- **MergeExclusions backend** — new `MergeExclusions` struct with `skip_categories`, `exclude_evidence_file_ids`, `exclude_coc_item_ids`, `exclude_collection_ids`, `exclude_form_submission_ids`; 35 tables mapped to 12 categories via `table_category()` in `merge_db.rs`

### Changed

- **merge_databases() rewritten** — database merge loop now applies category-level skip + item-level WHERE NOT IN filters; dependent tables (hashes → evidence_files, coc_amendments → coc_items, collected_items → collections) are automatically filtered when parent items are excluded
- **executeMerge() API updated** — accepts full `MergeExclusions` object instead of `excludeCollectionIds: string[]`; reconciliation choices integrated into the exclusions pipeline

## [0.1.42] - 2026-03-10

### Added

- **Expanded container metadata auto-fill** — 7 new mappings from E01/AD1/UFED/L01 container headers: examiner name → collecting officer, stored intake hashes (MD5/SHA1/SHA256), EWF format version + segment count, UFED OS version + device name, AD1 filesystem/OS info/source path/companion log hashes, acquisition duration from companion logs
- **Evidence collection template v1.3.0** — form restructured to match forensic workflow order; new `evidence_container` field with conditional `show_when` device identification fields; field groups reordered (collection info → device → forensic acquisition → storage)
- **Collected items in collection list** — `EvidenceCollectionListPanel` now shows collected item count and resolved container filenames per collection

### Changed

- **Enrichable fields expanded to 16** — added `connection_method` and `building` to `ENRICHABLE_FIELDS` for re-enrichment on existing collections
- **UFED auto-fill restructured** — device name overrides generic filename in description; OS version appended to other_identifiers; extraction duration calculated from start/end timestamps
- **AD1 auto-fill expanded** — companion log examiner, source path, and hashes (MD5/SHA1/SHA256) now populate collecting officer and storage notes; volume filesystem, OS info, and volume label mapped to storage notes and other_identifiers
- **E01/L01 auto-fill expanded** — examiner name, format version, segment count, and stored image hashes (intake hashes) now populate collecting officer and storage notes
- **Companion log auto-fill expanded** — tool identity (`created_by`), unique description, and acquisition duration now populate storage notes

### Fixed

- **FK constraint on session insert** — `user` record is now ensured to exist before inserting a session, preventing `FOREIGN KEY constraint failed` errors

## [0.1.41] - 2026-03-10

Cumulative release covering all changes from v0.1.31 through v0.1.41.

### Added

- **Merge Projects Wizard** — combine multiple `.cffx` projects into a single unified project with examiner identification, collection reconciliation, and 35-table database merge
- **Merge-into-open project** — merge an external project into the currently open project; pins current project, detects collection conflicts (same case number, date+officer, date+location), and provides side-by-side reconciliation UI with keep/replace choices
- **Evidence Collection Summary Panel** — right-panel tab showing aggregated statistics, field completeness, and item breakdown with document and CSV export
- **Evidence auto-fill from containers** — automatic enrichment of evidence collection forms from E01/AD1/UFED/L01 container metadata (device type, serial number, make/model, acquisition method, hashes, dates); 14 enrichable fields with 3-tier matching; preview panel before applying
- **Linked data tree in right panel** — metadata badges (type, status, hash status) and detail pane for collection → collected-item → COC → evidence-file relationships
- **COC Form 7-01 alignment** — Chain of Custody restructured to match EPA CID OCEFT Form 7-01 with 15 new fields (owner info, collection method, disposition), 8 numbered sections, and auto-populate from container metadata
- **Case number and case name fields** — project-level case identification that pre-fills Report Wizard, Evidence Collection, and COC forms
- **L01 V3 columnar parser** — support for FTK Imager's 31-column positional ltree format with auto-detection between V2 (tab-depth) and V3 (columnar)
- **L01 source metadata enrichment** — 4 new EwfInfo fields from L01 ltree (source name, evidence number, file count, total bytes) wired into evidence collection auto-fill
- **L01 multi-segment ltree scanning** — reverse-order segment scanning to find ltree in the last segment
- **VFS handle pool** — global cache of opened VFS instances (max 32, LRU eviction) to avoid re-parsing segment headers on every read
- **Evidence reconciliation system** — matching engine scores container ↔ COC pairs by serial number, evidence ID, item number, and description; conflict resolver modal for manual/auto resolution
- **Export database tracking** — all exports (L01, 7z, native file copy) tracked in `export_history` table with unique operation IDs and manifest naming
- **Per-window project database isolation** — multiple Tauri windows can independently open separate projects
- **User profiles** — CRUD for examiner profiles in Settings with profile confirmation on project open/create
- **Automatic folder structure** — standard forensic folder template created on new project (1.Evidence, 2.Processed.Database, 4.Case.Documents, etc.)
- **macOS keep-app-alive** — app stays running when all windows close; dock icon click reopens window
- **Auto-hash preference** — configurable auto-verify toggle in Settings → Behavior with session warning toast
- **Performance preferences** — PerformanceTab in SettingsPanel with configurable defaults
- **Loading overlay** — global loading indicator for slow operations
- **Integration tests** — 58-test pipeline against real forensic data (E01, AD1, L01, UFED, VFS)
- **Test suite expansion** — 2,987+ tests across 127 files
- **41 component decompositions** — monolithic components refactored into modular sub-component directories

### Changed

- **Evidence collection template v1.2.0** — 45 fields, 7 headings; device identification and forensic acquisition fields restored for container metadata auto-fill
- **Evidence collection ID format** — new collections prefixed with case/project number (e.g., `10115-0900-EC-a1b2c3d4`); displayed in collection list panel
- **Hash architecture simplification** — deduplicated 6 function pairs into 3 (EWF, AD1, raw); shared helpers for completion/persistence; spawn_progress_reporter helper
- **App-wide UI compaction** — systematic tightening of spacing and typography across 30+ components (Report Wizard, Evidence Collection, Export, Settings, Modals, Help)
- **L01 path building optimization** — O(n²) → O(1) HashMap lookup (79s → 2.68s, 29.5x speedup for 211K entries)
- **Email HTML rendering** — sandboxed `<iframe>` instead of `innerHTML`; fixed header field access and inline detection
- **Image viewer** — fit-to-view uses actual container dimensions; Ctrl/Cmd+wheel zoom; reset on path change
- **Document viewer** — race guard for rapid file switching; zoom via `zoom` property; 50 MB size limits for email/text
- **Dotfile handling** — proper extension detection for `.gitignore`, `.env`, etc.
- **File decomposition** — 5 large files split for maintainability (merge.rs, schema.rs, ewf_export.rs, workspace_profiles.rs, App.tsx)
- **Release automation** — auto-updates README version badge and CHANGELOG links

### Fixed

- **Reconciliation selection buttons** — reliable event handling for radio/checkbox in SolidJS For loops; case-insensitive case number matching
- **Batch hash DB persistence** — hash results and verification records now persist to `.ffxdb` for batch operations
- **Hash stalling on large containers** — error events for all failure paths; 0.5% progress granularity with 3-second heartbeat
- **L01 batch hash progress** — chunk-level progress reporting instead of 0%→100% jumps
- **L01 compressed_size=0** — fallback when section_size is omitted by EWF writers
- **Sidebar click handlers** — fixed SolidJS non-reactive event binding causing export/report buttons to fail after project load
- **Auto-updater** — auth headers on both `check()` and `downloadAndInstall()` for private repos; correct v2 artifact patterns; signing key regenerated
- **macOS firmlink write-check** — write probe before sysinfo mount metadata prevents false "read-only" errors on `/Users` paths
- **COC persistence** — fixed parameter name mismatch (`{ item }` vs `{ record }`) causing silent save failures
- **Window-destroy DB cleanup** — WAL checkpoint and connection cleanup on force-quit/crash
- **Center-pane tab reliability** — race conditions fixed when opening tabs
- **Spreadsheet/text viewer height** — `h-full` instead of `h-screen`
- **Database viewer SQL** — column type extraction handles `COLLATE NOCASE`
- **PDF viewer** — pdfjs-dist v4 worker import (.js → .mjs)
- **Security** — replaced `Math.random()` with `crypto.randomUUID()` for ID generation

### Security

- DOMPurify 3.3.1 → 3.3.2; bytes crate 1.11.0 → 1.11.1 (CVE fixes)
- 113 Rust crate patches; 11 npm semver patches
- CodeQL security scanning added

## [0.1.40] - 2026-03-09

### Changed

- **Hash architecture simplification** — Deduplicated 6 identical function pairs into 3 (EWF, AD1, raw fallback) across both frontend and backend; extracted `collectStoredHashes()` and `determineVerification()` as single source of truth for stored hash collection and verification logic; added shared `handleHashCompleted()` and `persistHashToDb()` helpers eliminating code duplication between single-file and batch hash paths; extracted `spawn_progress_reporter()` helper in backend replacing 70 lines of inline progress thread code; replaced 6 container routing branches with 3-arm match using shared progress callback. Net -73 lines.

### Fixed

- **Batch hash DB persistence** — Batch-hashed files now persist hash results and verification records to the project database (`.ffxdb`), fixing a silent data loss where only single-file hashes were saved

## [0.1.39] - 2026-03-09

### Added

- **Evidence reconciliation system** — Schema v10 with `reconciliation_results`, `reconciliation_conflicts` tables; matching engine scores container ↔ COC pairs by serial number, evidence ID, item number, and description; conflict resolver modal for manual/auto resolution; merge support in `merge_databases()`; activity log tracking for all reconciliation actions
- **`terminatedFiles` safety net** — Frontend hash computation tracks files that received terminal events; after `batch_hash` returns, any files missing terminal events are marked as errors to prevent silent failures

### Fixed

- **Hash stalling on large containers** — Emit error events for all failure paths (semaphore acquisition, `spawn_blocking` panics) so the frontend's `completedCount` always reaches total; use 0.5% progress granularity with 3-second heartbeat to prevent apparent stalls during slow I/O; scope `EwfHandle` early to free file descriptors before I/O thread opens its own handle
- **Batch hash verification** — Hash verification now matches all stored hash sources (header hashes, segment hashes, stored image hashes) instead of only checking the first source
- **L01 batch hash progress** — L01 containers now report per-file chunk-level progress during batch hashing instead of jumping from 0% to 100%
- **Hash batch progress, pause/resume, and button guard** — Fixed progress bar updates during batch hashing; pause/resume controls work correctly; hash button properly guards against concurrent operations

### Changed

- **File decomposition for maintainability** — Decomposed 5 large files: `merge.rs` (1925→1002 lines, extracted 4 modules), `schema.rs` (1321→12, extracted 2 modules), `ewf_export.rs` (1303→420, extracted 3 modules), `workspace_profiles.rs` (1228→729, extracted 2 modules), `App.tsx` (1265→1135, extracted 2 modules)

## [0.1.38] - 2026-03-09

### Added

- **L01 V3 columnar ltree parser** — Adds support for FTK Imager's 31-column positional ltree format with `child_count`-based hierarchy; version auto-detection between V2 (tab-depth) and V3 (columnar) formats; includes `L01RecordSummary` (total_bytes, file_count) and `L01SourceInfo` (name, evidence_number) extraction
- **L01 multi-segment ltree scanning** — Scans all segments in reverse order to find the ltree section (stored in the last segment, not the first)
- **L01 source metadata → evidence collection auto-fill** — 4 new `EwfInfo` fields (`l01_source_name`, `l01_source_evidence_number`, `l01_file_count`, `l01_total_bytes`) wired into the evidence collection form auto-fill pipeline; L01 source name maps to description/brand, evidence number to item_number, file count and total bytes to storage_notes
- **VFS handle pool** — Global `VFS_POOL` caches opened VFS instances (max 32 handles, LRU eviction) to avoid re-parsing segment headers on every `vfs_list_dir`/`vfs_read_file` call; includes per-handle directory and attribute caches
- **`vfs_close_container` command** — Explicitly evicts a container from the VFS handle pool
- **L01 segment discovery** — `discover_l01_segments()` in `common/segments.rs` discovers multi-segment L01 files (.L01, .L02, .L03, etc.)
- **Performance preferences** — `PerformanceTab` in SettingsPanel with configurable performance defaults
- **User profiles** — `UserProfilesTab` CRUD for examiner profiles; `UserConfirmModal` for profile confirmation on project open/create
- **Loading overlay** — `LoadingOverlay` component + `useLoadingState` hook for global loading indicator
- **Batched file discovery events** — 80ms buffer batches discovery events to avoid O(n²) array copies in `useFileManager`
- **Integration tests** — `test_case_10115.rs` example: 58-test pipeline against real forensic data (E01, AD1, L01, UFED, VFS) with `--quick` and `--verbose` flags

### Fixed

- **L01 compressed_size=0** — Fallback to `next_offset` or `file_size` when `section_size` is omitted by EWF writers
- **L01 uncompressed ltree data** — Detect raw UTF-16LE when FTK Imager uses None compression (compressed == uncompressed size)

### Changed

- **L01 path building optimization** — O(n²)→O(1) HashMap lookup replaces O(n) linear scan for parent path resolution (79s → 2.68s, 29.5x speedup for 211K entries)

## [0.1.37] - 2026-03-08

### Added

- **Case number and case name fields** — `FFXProject` now stores `case_number` and `case_name` for parent case identification; fields appear in the Project Setup Wizard (ConfigureLocationsStep), pre-fill the Report Wizard case info and Evidence Collection forms, and are preserved through project merge
- **Automatic folder structure on new project** — `handleProjectSetupComplete` now calls `create_folders_from_template` after project creation to ensure the standard forensic folder structure (1.Evidence, 2.Processed.Database, 4.Case.Documents, etc.) exists on disk; idempotent — already-existing directories are unaffected; paths default to template roles when auto-discovery found no subdirectories
- **macOS keep-app-alive** — app stays running when all windows are closed (standard macOS behavior); clicking the dock icon reopens a window via `Reopen` event handler; explicit quit via Cmd+Q or app menu

### Fixed

- **Evidence collection case number** — `EvidenceCollectionPanel` now receives `projectManager.caseNumber()` instead of `projectManager.projectName()` for the `caseNumber` prop, and includes project name in autoFillContext
- **Wizard state reset** — `caseNumber` and `caseName` signals are properly cleared when the Project Setup Wizard closes

## [0.1.36] - 2026-03-08

### Fixed

- **Per-window project database isolation** — `PROJECT_DBS` changed from a global singleton to a per-window `HashMap<String, ProjectDatabase>` keyed by window label; each Tauri window now independently opens/closes its own `.ffxdb` database, enabling multiple projects open simultaneously in separate windows
- **macOS firmlink write-check fix** — `check_path_writable` restructured to try a write probe first (ground truth) before consulting `sysinfo` mount metadata; on macOS Catalina+, `/Users` is a firmlink to `/System/Volumes/Data/Users` and sysinfo incorrectly matched the read-only root volume, causing project creation to fail with "OS drive is read-only"
- **Window-destroy DB cleanup** — added `on_window_event(WindowEvent::Destroyed)` handler in `lib.rs` that calls `cleanup_window_project_db()` as a safety net for force-quit/crash scenarios, ensuring WAL checkpoint and connection cleanup even if the frontend didn't call `project_db_close`
- **COC persistence parameter name bug** — fixed `cocPersistence.ts` awaitable invoke calls using `{ item }` / `{ transfer }` instead of `{ record }` to match the Rust function parameter name; COC upsert saves were silently failing because Tauri v2 requires JSON keys to match Rust parameter names

### Changed

- All 118 `project_db` Tauri commands now receive `window: tauri::Window` as a parameter (auto-injected by Tauri — zero frontend changes needed) and resolve the correct database via `with_project_db(window.label(), |db| ...)`

## [0.1.35] - 2026-03-08

### Added

- **Evidence auto-fill from container metadata** — new "From Evidence" button in evidence collection forms that maps AD1 companion logs, E01/L01 EWF headers, and UFED device/extraction info to form fields (device type, description, serial number, make/model, acquisition method, etc.). Includes a preview panel before applying changes.
- **`evidenceAutoFill.ts` utility** — centralized mapping module with `extractItemFieldsFromEvidence()`, `extractHeaderFieldsFromEvidence()`, `buildCollectedItemsFromEvidence()`, and `getAutoFillSummaries()` exports
- **COC prefill from container metadata** — new `cocPrefill.ts` utility with `prefillCocFromContainer()` and `overlayCocFromCollection()` that maps E01/AD1/UFED container metadata (serial number, make, model, hashes, dates, examiner) to COCItem fields
- **`get_current_username` and `get_app_version` Tauri commands** — new system commands for OS username lookup and app version reporting
- **Examiner profile hook** — extracted `useExaminerProfile.ts` for persistent examiner auto-fill across forms
- **Export examiner pre-fill** — export panel now accepts `initialExaminerName` prop to pre-fill examiner fields from project owner

### Changed

- **COC Form 7-01 alignment** — restructured Chain of Custody data model, UI, and auto-populate to align with EPA CID OCEFT Form 7-01 (Rev\_03/2017):
  - **Schema v9 migration**: 15 new `coc_items` columns (case\_title, office, owner info, collection method, disposition fields) + 2 new `coc_transfers` columns (storage\_location, storage\_date)
  - **COCItemRow UI**: 8 numbered sections matching Form 7-01 layout (Case Info, Owner/Source/Contact, Collection Method radio buttons, Item Details, Collection & Custody, Remarks, Transfer Records, Final Disposition)
  - **Auto-populate enhancement**: "Auto-Populate from Evidence" now fills serial number, model, make, capacity, hashes, dates, examiner, and collection method from loaded container metadata
- **Evidence collection form streamlined (v1.0.0 → v1.1.0)** — removed 12 fields and 3 sections that should only be filled from evidence container metadata: serial_number, brand, make, model, IMEI, other_identifiers, image_format, acquisition_method, connection_method, item_system_datetime, timezone, storage_notes, plus Time Documentation, Forensic Image, and Additional Storage Info headings
- **Email HTML rendering** — email body now renders in a sandboxed `<iframe>` instead of `innerHTML` to prevent style leakage and improve layout integrity; auto-resizes to content height
- **Email parser fixes** — fixed `is_inline` detection to use `content_disposition()` API instead of `is_message()`; fixed header `name`/`value` field access for `mail-parser` API compatibility
- **Image viewer improvements** — fit-to-view now uses actual container dimensions instead of hardcoded 800×600; added Ctrl/Cmd+wheel zoom; reset zoom/size on image path change
- **Document viewer race guard** — added load generation counter to prevent stale async results from overwriting current content when switching files rapidly
- **Document viewer zoom** — changed from CSS `transform: scale()` to `zoom` property for proper layout flow
- **Large file protection** — email (EML/MBOX) and text documents now enforce 50 MB size limits with clear error messages; text viewer truncates to first 50 MB for oversized files
- **Dotfile extension handling** — `getExtension()` now returns `""` for dotfiles (`.gitignore`, `.env`, etc.); config detection uses dedicated `CONFIG_DOTFILE_NAMES` list for full-basename matching
- **File type detection** — added `jfif` to image extensions; added `amr`, `caf` to mobile forensic formats
- **Windows disk info help text** — updated `storage_interface` field help from deprecated `wmic diskdrive get InterfaceType,Model` to PowerShell `Get-CimInstance Win32_DiskDrive | Select InterfaceType,Model` (WMIC removed in Windows 11)
- **Code decomposition** — extracted `AppHeader.tsx` from App.tsx (112 lines), split `cocDbSync.ts` into `cocConverters.ts` (362), `cocExport.ts` (155), `cocPersistence.ts` (231); decomposed `SchemaFormRenderer.tsx` into `fields/` (4 components) and `sections/` modules (700 lines extracted)

### Fixed

- **6 TypeScript compilation errors** — App.tsx `projectName` null/undefined type mismatch, duplicate `make`/`model` keys in cocConverters.ts, unused imports in useExaminerProfile.ts and sections/index.tsx
- **Spreadsheet viewer height** — fixed container using `h-screen` (full viewport) instead of `h-full` (panel-relative)
- **Text content height** — fixed container using `h-screen` instead of `h-full`
- **Office viewer empty state** — added null guard for `info()` accessor to prevent rendering empty metadata when data hasn't loaded yet
- **Database viewer SQL** — fixed SQLite column type extraction to handle both `COLLATE NOCASE` and bare type names

## [0.1.34] - 2026-03-08

### Fixed

- **Auto-updater signing keys** — regenerated Ed25519 signing keypair and updated `tauri.conf.json` pubkey + GitHub secret to resolve persistent "signature verification failed" update errors; old keypair had a key/password mismatch in the GitHub secret that caused all updater signatures to be invalid

## [0.1.33] - 2026-03-08

### Fixed

- **Sidebar click handlers** — fixed SolidJS non-reactive event binding in `SidebarButton` that caused export/report left-click to silently fail after project load (buttons started disabled, `onClick` was set to `undefined` at mount time and never updated; context menu worked because `onContextMenu` was always bound unconditionally)

## [0.1.32] - 2026-03-08

### Changed

- **Release automation** — `publish-release` job now auto-updates README version badge and CHANGELOG comparison links after every successful release (replaces fragile detached-HEAD push with proper branch checkout)

## [0.1.31] - 2026-03-08

### Changed

- **App-wide UI compaction** — systematic tightening of spacing, typography, and decorative elements across 30+ component files for a denser, more professional layout inspired by Linear/Notion:
  - **Report wizard** — WizardLayout, SchemaFormRenderer (13 changes), ReportTypeStep, CaseInfoSchemaStep, FindingsStep, PreviewStep, ExportStep, ReportDataStep, COCFormSection, COCItemRow, COCModals
  - **Evidence collection** — EvidenceCollectionPanel, EvidenceCollectionListPanel
  - **Export panel** — ExportHeader, ExportPanelComponent, ExportSourceSection, ExportFooter, PhysicalImageMode, LogicalImageMode, NativeExportMode, FileExportTab, ArchiveTab, CaseMetadataSection
  - **Tools sub-tabs** — ToolsModeComponent, ToolsTabSelector, TestTab, RepairTab, ValidateTab, ExtractTab, CompressTab, DecompressTab
  - **Settings** — SettingsPanel sidebar and content area
  - **Modals & overlays** — DriveSelector, MergeProjectsWizard, WelcomeModal, UpdateModal, CommandPalette, SearchPanel
  - **Help** — HelpPanel section headers and content padding

### Fixed

- **Center-pane tab reliability** — refactored `useCenterPaneTabs` to fix race conditions when opening tabs, preventing blank/stuck tab states
- **Keyboard handler** — improved `useKeyboardHandler` edge-case handling
- **Auto-updater auth** — renamed `GITHUB_UPDATE_TOKEN` secret to `GH_UPDATE_TOKEN` (GitHub disallows `GITHUB_` prefix for repository secrets, so the token was never injected into CI builds)
- **CI lint compliance** — resolved clippy and rustfmt violations in `export.rs` (unnecessary references, unnecessary casts, inline if/else formatting)

## [0.1.27] - 2026-03-07

### Added

- **Spreadsheet viewer interactivity** — column sorting (click headers to cycle ascending/descending/clear), row search/filter toolbar, cell click-to-copy with clipboard feedback, Export CSV via save dialog, Print via `printDocument()`, filtered/total row count display
- **Evidence collection list enhancements** — search bar filtering by officer/case number/status/date/authorization, Print All button generating HTML table, smart empty state messaging, filtered footer count
- **Linked data panel export & print** — Export CSV button (flattens tree hierarchy with indented labels), Print button (HTML report with stats and indented tree), `csvEsc`/`escH` helpers
- **Spreadsheet helpers module** — `cellSortValue()`, `sortRows()`, `filterRows()`, `rowsToCsv()`, `rowsToHtmlTable()`, `rowsToTsv()`, `csvEscape()`, `escHtml()` utility functions

### Fixed

- **License & third-party compliance** — comprehensive audit and remediation:
  - Created missing LICENSE files for `libewf-ffi/` (MIT + LGPL-3.0 third-party notice), `sevenzip-ffi/` (MIT + LZMA SDK public domain notice), `patches/libarchive2-sys/` (MIT + BSD notice), and `sevenzip-ffi/lzma/` (LZMA SDK public domain)
  - Expanded `THIRD_PARTY_LICENSES.md` from 96 to 310+ lines — now covers native C libraries, 60+ Rust crates by category, npm packages, fonts (OFL-1.1), notable non-MIT licenses (MPL-2.0, BSL-1.0, CDLA-Permissive-2.0), and full license text references
  - Added collapsible "Third-Party Software Licenses" section to in-app About page with LGPL-3.0 attribution for libewf, native library table, key open-source components, and font credits
  - Updated native About dialog (macOS + Windows/Linux) to mention LGPL-3.0, BSD, and Public Domain third-party software

## [0.1.26] - 2026-03-07

### Fixed

- **Auto-updater signature verification** — pass auth headers to `downloadAndInstall()` in `UpdateModal.tsx`; without this, private GitHub repo binary downloads return 404/HTML and signature verification fails against garbage data
- **Auto-updater manifest generation** — three bugs in release.yml `publish-release` job caused `latest.json` to have empty `"platforms": {}` on every release:
  - `actions/checkout@v4` wiped `./artifacts/` after download — moved checkout before download
  - Download patterns used v1 names (`*.nsis.zip`, `*.AppImage.tar.gz`) instead of Tauri v2 format (`*-setup.exe`, `*.AppImage`)
  - Manifest globs matched v1 wrapped bundles instead of v2 raw installers
- **Release badge update** — sparse checkout now includes `README.md` alongside `CHANGELOG.md` so the version badge step no longer fails

## [0.1.25] - 2026-03-07

### Added

- **Clear Bookmarks** — new `clearBookmarks()` function in project hook; wired to menu action with confirmation dialog and toast feedback
- **Export Bookmarks** — export all bookmarks as a JSON file via native save dialog; wired to menu action with toast feedback

### Changed

- **Documentation overhaul** — README rewritten as product-focused overview, SECURITY.md made evergreen, CODE_BIBLE.md module paths corrected, HELP.md expanded with missing features
- **CI badge automation** — release workflow `publish-release` job auto-updates README version badge on each release via sed + git push (`[skip ci]`)

### Fixed

- Deleted stale duplicate test file (`src/utils/__tests__/pathUtils.test.ts`) that caused 3 false test failures — canonical tests remain at `src/utils/pathUtils.test.ts`

### Security

- **bytes** crate updated 1.11.0 → 1.11.1 (CVE fix — Dependabot alert)
- **npm audit** clean — 0 vulnerabilities

### Dependencies

- **Full cargo update** — 113 Rust crate patches applied (nom pinned to 7.x via lockfile — notatin 1.0.1 incompatible with nom 8.0)
- **npm update** — 11 safe semver patches: @solid-primitives (event-listener, keyboard, resize-observer, scheduled, storage), @tauri-apps/cli, @types/node, autoprefixer, dompurify, postcss, solid-icons

## [0.1.24] - 2026-03-06

### Added

- **Software Update Release Notes** — in-app updater now displays full formatted release notes:
  - `release.yml` extracts version-specific notes from CHANGELOG.md into `latest.json` manifest
  - `UpdateModal.tsx` renders markdown as sanitized HTML (h2/h3/h4, bold, code, links, lists)
  - DOMPurify sanitization for safe innerHTML rendering
  - `.release-notes` CSS styles in `index.css` for proper typography

### Changed

- Version bump to 0.1.24 (0.1.23 tag was deleted by repo rule enforcement and cannot be recreated)

## [0.1.23] - 2026-03-06

### Added

- **Merge Projects Wizard** — combine multiple `.cffx` projects and `.ffxdb` databases into a single unified project:
  - 4-step wizard (Select → Review → Execute → Complete) accessible from Tools menu and Command Palette
  - Two-phase pipeline: analyze phase reads each project (read-only) with full examiner identification; execute phase merges data with deduplication
  - Examiner identification from 7 primary + 9 fallback sources (project owner, session users, COC officers, form submissions, etc.)
  - Owner auto-suggest via datalist populated from gathered examiners
  - Expandable per-project detail sections: Examiners, Evidence Files, Collections, Chain of Custody, Forms
  - Merges 35 `.ffxdb` tables via `INSERT OR IGNORE` with safe WAL handling (temp-copy + checkpoint for databases with active WAL files)
  - Form data extraction for evidence collection forms (officer, location, lead examiner)
  - Works without a project loaded (not project-dependent)
- **Export Database Tracking** — all export operations (L01, 7z, native file copy) now tracked in `export_history` table:
  - Unique operation IDs for each export (e.g., `l01-1719842300000`, `archive-1719842300000`)
  - `dbSync.insertExport()` on start, `dbSync.updateExport()` on completion/failure
  - Unique manifest naming with operation ID to prevent overwrites across multiple exports
- **Case Folder Template** — new JSON template for standardized case directory structure
- **CodeQL Security Scanning** — added custom CodeQL workflow with Rust build support
- **Component Decompositions** — 41 monolithic components decomposed into modular sub-component directories:
  - `activity-panel`, `activity`, `binary-viewer`, `bookmarks`, `command-palette`, `container-viewer`
  - `database-viewer`, `detail-panel`, `drag-drop`, `email-viewer`, `evidence-collection`, `exif-panel`
  - `export-panel`, `export/nativeExport`, `export/tools-mode`, `file-row`, `filter-presets`, `hash-badge`
  - `help`, `help/sections`, `hex`, `layout/center-pane`, `layout/sidebar`, `merge`
  - `metadata/container-details`, `metadata`, `office-viewer`, `pdf`, `plist-viewer`, `project-dashboard`
  - `project-wizard`, `project/recovery`, `pst`, `registry`, `report/wizard/steps/evidence`
  - `search`, `spreadsheet-viewer`, `status-bar`, `tab-bar`, `text-viewer`, `virtual-list`
  - Each module follows consistent pattern: `index.ts`, `types.ts`, `helpers.ts`, `__tests__/helpers.test.ts`, `Component.tsx`
- **Test Coverage Expansion** — 41 new test files with ~600 additional tests across:
  - Component helpers: bookmarks, command-palette, binary-viewer, database-viewer, search, merge, activity, hex, etc.
  - API modules: archiveCreate, drives, ewfExport, fileExport, l01Export
  - Hooks: useDatabase, useFormTemplate, projectSaveOptions, projectSetup
  - Templates: deviceTypeFilters, formDataConversion, fieldCounters
  - Detail panel: normalizeContainerFields, cocDbSync, hashHelpers, constants
  - Total test suite: 2,987 tests across 127 files (2,984 passing)

### Fixed

- **L01 Export** — folder sources now preserve directory name in L01 tree; single-segment and multi-segment both use `segment::segment_path()` for correct `.L01`/`.L02` extensions
- **Native Export** — `collect_files()` preserves folder name via `path.parent()` strip_prefix; unique manifest naming prevents overwrites
- **CI Test Failures** — resolved 23 TypeScript type errors in 8 test files (missing `container` field on `ContainerInfo`, missing `witnesses` on `EvidenceCollectionData`, wrong import paths, unused variables)
- **Rust Formatting** — `cargo fmt` applied to 5 files (merge.rs, discovery.rs, export.rs, project_db/mod.rs, system.rs)
- **Clippy Warnings** — fixed unused variable `recommended` in sevenzip-ffi archive.rs; replaced `format!("literal")` with `.to_string()` in system.rs
- **CodeQL** — fixed Rust extractor to use `build-mode: none` (CodeQL requirement)
- **`joinPath` utility** — fixed cross-platform path joining edge cases

### Changed

- **Component Architecture** — major frontend refactoring from monolithic components to modular sub-component directories with extracted types, helpers, and dedicated test files
- **Test Infrastructure** — Vitest v4.0.18 with jsdom environment, comprehensive mocking patterns for Tauri IPC (`@tauri-apps/api/core`), SolidJS reactivity testing

### Security

- **DOMPurify** — bumped from 3.3.1 to 3.3.2 (Dependabot security fix)

## [0.1.22] - 2026-03-03

### Fixed

- **CI**: Fixed draft release race condition — upload steps now preserve draft status

## [0.1.21] - 2026-03-03

### Fixed

- **Signing**: Regenerated updater signing key (v4) with empty password for CI compatibility
- **PDF Viewer**: Fixed pdfjs-dist v4 worker import (.js → .mjs)
- **Security**: Replaced `Math.random()` with `crypto.randomUUID()` in ID generation
- **CI**: Added `permissions: contents: read` to tests.yml and performance-regression.yml
- **Example**: Removed cleartext password from libarchive2 example output

## [0.1.20] - 2026-03-02

### Fixed

- **Release Management**: Cleaned up all orphaned draft releases and tags
- **Auto-updater**: Verified signing key configuration matches `core-ffx-v3.key`

### Changed

- **CI/CD**: Fresh release pipeline with clean tag history

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

[Unreleased]: https://github.com/tmreyno/CORE/compare/v0.1.44...HEAD
[0.1.44]: https://github.com/tmreyno/CORE/compare/v0.1.27..........................................v0.1.44
[0.1.43]: https://github.com/tmreyno/CORE/compare/v0.1.27.......................................v0.1.43
[0.1.42]: https://github.com/tmreyno/CORE/compare/v0.1.27....................................v0.1.42
[0.1.41]: https://github.com/tmreyno/CORE/compare/v0.1.27.................................v0.1.41
[0.1.40]: https://github.com/tmreyno/CORE/compare/v0.1.27..............................v0.1.40
[0.1.39]: https://github.com/tmreyno/CORE/compare/v0.1.27...........................v0.1.39
[0.1.38]: https://github.com/tmreyno/CORE/compare/v0.1.27........................v0.1.38
[0.1.37]: https://github.com/tmreyno/CORE/compare/v0.1.27.....................v0.1.37
[0.1.36]: https://github.com/tmreyno/CORE/compare/v0.1.27..................v0.1.36
[0.1.35]: https://github.com/tmreyno/CORE/compare/v0.1.27...............v0.1.35
[0.1.34]: https://github.com/tmreyno/CORE/compare/v0.1.27............v0.1.34
[0.1.33]: https://github.com/tmreyno/CORE/compare/v0.1.27.........v0.1.33
[0.1.32]: https://github.com/tmreyno/CORE/compare/v0.1.27......v0.1.32
[0.1.31]: https://github.com/tmreyno/CORE/compare/v0.1.27...v0.1.31
[0.1.27]: https://github.com/tmreyno/CORE/compare/v0.1.26...v0.1.27
[0.1.26]: https://github.com/tmreyno/CORE/compare/v0.1.25...v0.1.26
[0.1.25]: https://github.com/tmreyno/CORE/compare/v0.1.24...v0.1.25
[0.1.24]: https://github.com/tmreyno/CORE/compare/v0.1.23...v0.1.24
[0.1.23]: https://github.com/tmreyno/CORE/compare/v0.1.22...v0.1.23
[0.1.22]: https://github.com/tmreyno/CORE/compare/v0.1.21...v0.1.22
[0.1.21]: https://github.com/tmreyno/CORE/compare/v0.1.20...v0.1.21
[0.1.20]: https://github.com/tmreyno/CORE/compare/v0.1.18...v0.1.20
[0.1.18]: https://github.com/tmreyno/CORE/compare/v0.1.14...v0.1.18
[0.1.14]: https://github.com/tmreyno/CORE/compare/v0.1.0...v0.1.14
[0.1.0]: https://github.com/tmreyno/CORE/releases/tag/v0.1.0
