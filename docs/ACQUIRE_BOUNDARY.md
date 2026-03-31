# CORE Acquire Edition — Boundary Manifest

> **Purpose:** Defines the explicit API surface for the Acquire edition, enabling clean separation into CORE-ACQ. This document is the source of truth for what belongs in the Acquire build vs. the Review/Full build.

---

## Rust Backend Boundary

The Rust backend is already cleanly separated via Cargo feature flags in `src-tauri/src/lib.rs`:

- **`flavor-acquire`** — marker feature for acquire commands
- **`flavor-review`** — gates review-only modules and commands
- **`full`** = `flavor-review` + `flavor-acquire` + `ai-assistant` + `unrar` (default)
- **`acquire`** = `flavor-acquire` only

### Always-Compiled Modules (Shared)

These modules compile in ALL editions:

```
ad1, archive, aff4, commands, common, containers, database, ewf, formats,
l01_writer, logging, menu, project, project_db, raw, ufed, viewer (hex/text only)
```

### Review-Only Modules (`#[cfg(feature = "flavor-review")]`)

```
activity_timeline, dedup, processed, project_comparison, project_recovery,
project_templates, report, search, workspace_profiles, workspace_profile_types,
workspace_profile_defaults, viewer::document (sub-module only)
```

### Command Registration

| Category | `run_acquire()` | `run_full()` |
|----------|:---:|:---:|
| Container ops (V1, V2, lazy loading) | ✅ | ✅ |
| Archive inspection (metadata, extraction, nested) | ✅ | ✅ |
| UFED, EWF/RAW verify, VFS | ✅ | ✅ |
| Hash (batch, queue) | ✅ | ✅ |
| System, portable, device, memory, triage | ✅ | ✅ |
| Companion files | ✅ | ✅ |
| Discovery (evidence files, streaming scan) | ✅ | ✅ |
| Discovery (processed DBs, case docs, COC forms) | ❌ | ✅ |
| Database (app-level) | ✅ | ✅ |
| Project DB (core: lifecycle, evidence, hashes, COC, collections, forms, exports, activity, ui_state) | ✅ | ✅ |
| Project DB (review: bookmarks, notes, tags, searches, case docs, processed DBs, FTS, annotations) | ❌ | ✅ |
| Project commands (create, load, save, backup) | ✅ | ✅ |
| Export (file copy) | ✅ | ✅ |
| Window management (new_window, labels) | ✅ | ✅ |
| **Archive creation (7z)** | ✅ | ✅ |
| **EWF/E01 export (libewf-ffi)** | ✅ | ✅ |
| **Raw disk image export** | ✅ | ✅ |
| **L01 export (pure-Rust)** | ✅ | ✅ |
| **AFF4 export** | ✅ | ✅ |
| Search (Tantivy) | ❌ | ✅ |
| Deduplication | ❌ | ✅ |
| Project merge | ❌ | ✅ |
| Document viewers | ❌ | ✅ |
| Reports + AI | ❌ | ✅ |
| Processed DBs (AXIOM, Cellebrite, Autopsy) | ❌ | ✅ |
| Workspace profiles, templates, timeline, comparison | ❌ | ✅ |

---

## Frontend Boundary

### Edition Detection (`src/utils/edition.ts`)

```typescript
const APP_EDITION = __APP_EDITION__;  // "full" | "acquire" (Vite define)
isAcquireEdition()  // true when edition === "acquire"
isFullEdition()     // true when edition !== "acquire"
```

Build command: `VITE_EDITION=acquire npm run tauri build -- --config src-tauri/tauri.acquire.conf.json --features acquire -- --no-default-features`

### Acquire-Only Frontend Files

These files exist ONLY for the Acquire edition:

**Components (`src/components/acquire/`):**
- `AcquireCollectionSummary.tsx`
- `AcquireCollectionView.tsx`
- `AcquireDashboard.tsx`
- `AcquireExportView.tsx`
- `AcquireIdentifyView.tsx`
- `AcquireLayout.tsx`
- `AcquireProcessShell.tsx`
- `AcquireTriageView.tsx`
- `AcquireVerifyView.tsx`
- `StartSessionDialog.tsx`
- `SystemInfoPanel.tsx`
- `acquire.css`

**Hooks (`src/hooks/acquire/`):**
- `types.ts`
- `useAcquisitionRunner.ts`
- `useAcquisitionSession.ts`

### Acquire-Eligible API Files

These API files invoke commands available in `run_acquire()`:

| File | Commands | Category |
|------|----------|----------|
| `aff4Export.ts` | `aff4_create_image`, `aff4_cancel_export` | Acquire |
| `archiveCreate.ts` | `create_7z_archive`, `test_7z_archive`, `estimate_archive_size`, `cancel_archive_creation`, `repair_7z_archive`, `validate_7z_archive`, `extract_split_7z_archive`, `encrypt_data_native`, `decrypt_data_native`, `get_last_archive_error`, `clear_last_archive_error` | Acquire |
| `commands.ts` | Container ops, lazy loading, hash queue, DB ops, opener | Shared |
| `companion.ts` | `write_companion_file`, `read_companion_file`, `find_companion_file` | Shared |
| `device.ts` | `check_privilege`, `get_device_size`, `list_physical_disks`, `request_elevation`, `read_raw_device` | Acquire |
| `drives.ts` | `list_drives`, `check_path_writable`, `remount_read_only`, `restore_mount` | Shared |
| `ewfExport.ts` | `ewf_create_image`, `ewf_cancel_export`, `ewf_read_image_info`, `ewf_get_version` | Acquire |
| `exportHistory.ts` | `project_db_get_exports`, `project_db_delete_export` | Shared |
| `fda.ts` | `check_full_disk_access`, `open_full_disk_access_settings` | Shared |
| `fileExport.ts` | `export_files`, `cancel_export` | Shared |
| `l01Export.ts` | `l01_create_image`, `l01_cancel_export`, `l01_estimate_size` | Acquire |
| `memory.ts` | `memory_capture_info`, `memory_capture`, `memory_capture_cancel` | Acquire |
| `portable.ts` | `portable_get_status`, `portable_ensure_dirs` | Shared |
| `rawExport.ts` | `raw_create_image`, `raw_cancel_export` | Acquire |
| `segmentHash.ts` | `hash_container_segments` | Acquire |
| `triage.ts` | `triage_get_profiles`, `triage_collect`, `triage_cancel` | Acquire |

### Review-Only API Files (NOT in Acquire)

| File | Commands | Category |
|------|----------|----------|
| `dedup.ts` | `dedup_analyze`, `dedup_enrich_hashes`, `dedup_export_csv` | Review-only |
| `projectMerge.ts` | `project_merge_analyze`, `project_merge_execute` | Review-only |
| `search.ts` | `search_open_index`, `search_close_index`, `search_delete_index`, `search_get_stats`, `search_index_container`, `search_index_all`, `search_rebuild_index`, `search_query` | Review-only |

### Ambiguous / Utility API Files

| File | Notes |
|------|-------|
| `importAcquisitions.ts` | Calls `scan_for_acquisitions` — registered in both editions |
| `lzmaApi.ts` | Calls `compress_to_lzma*`, `decompress_lzma*` — utility, may not be registered in acquire |
| `index.ts` | Barrel export only |

### Shared Export Hooks (`src/hooks/export/`)

These hooks are used by BOTH editions (they manage acquisition workflows):

- `types.ts` — shared type definitions
- `companionHelper.ts` — companion file + evidence collection auto-creation
- `useAff4ExportState.ts` — AFF4 export state
- `useEwfExportState.ts` — E01/EWF export state
- `useExportCommon.ts` — shared source/destination/drive state
- `useL01ExportState.ts` — L01 export state
- `useMemoryDumpState.ts` — memory capture state
- `useNativeExportState.ts` — 7z archive + file copy export state
- `useRawExportState.ts` — raw disk image export state
- `useTriageState.ts` — triage collection state

### Edition-Aware Files (22 files with `isAcquireEdition()` checks)

These files contain runtime edition guards:

```
App.tsx, QuickActionsBar.tsx, SettingsPanel.tsx, AppHeader.tsx,
AppModals.tsx, AppSecondaryModals.tsx, Sidebar.tsx, WelcomeModal.tsx,
ProjectSetupWizard.tsx, useWizardState.ts, BehaviorTab.tsx, DefaultsTab.tsx,
PerformanceTab.tsx, useAcquisitionRunner.ts, companionHelper.ts,
projectSetup.ts, useAppLifecycle.ts, useCommandPalette.tsx,
useKeyboardHandler.ts, useMenuActions.ts, edition.ts, vite-env.d.ts
```

---

## Cross-Boundary Imports (to resolve)

Acquire components currently import from non-Acquire paths. These must be resolved before separation:

### Shared UI Primitives (candidates for core-shared)

| Import | Used By | Resolution |
|--------|---------|------------|
| `../viewerMetadata/shared` (`CollapsibleGroup`, `MetadataRow`, `OptionalMetadataRow`) | `SystemInfoPanel` | Move to `@core-suite/components` or local `shared/` |
| `../Toast` (`useToast`) | `AcquireIdentifyView`, `AcquireTriageView` | Move to `@core-suite/components` |

### Shared Components (needed by Acquire)

| Import | Used By | Resolution |
|--------|---------|------------|
| `../export-panel/DriveTreeBrowser` | `AcquireDashboard`, `AcquireExportView`, `AcquireIdentifyView` | Keep as shared export component |
| `../EvidenceCollectionPanel` | `AcquireCollectionView` | Keep as shared component |
| `../EvidenceTree/containerDetection` (`getContainerType`) | `AcquireVerifyView` | Extract utility function |
| `../RecentProjectsList` | `AcquireDashboard` | Keep as shared component |
| `../preferences` (`getPreference`) | `AcquireDashboard` | Keep as shared utility |

### Icon Imports (already resolved)

| Import | Status |
|--------|--------|
| `@core-suite/icons` | ✅ Already in core-shared |
| `../icons` (local barrel) | Needs redirect to `@core-suite/icons` or keep local |

---

## CI Build Matrix

The CI validates both editions compile independently:

| Job | Cargo Features | Vite Edition | Purpose |
|-----|----------------|--------------|---------|
| `test-backend` | `--all-features` (default) | — | Full Rust compilation + tests |
| `build` | default (full) | — | Full application build |
| **`build-acquire`** | `--no-default-features --features acquire` | `acquire` | Acquire edition build + boundary lint |

The `build-acquire` job runs:
1. **Boundary lint** (`scripts/lint-acquire-boundary.sh`) — validates no review-only imports in acquire files (Linux only)
2. **`cargo check`** with acquire features — validates Rust compiles cleanly
3. **`cargo clippy`** with acquire features — lint warnings as errors
4. **`npm run tauri build`** with acquire config — full Tauri build with signing

---

## Separation Readiness Checklist

- [x] Cargo feature flags cleanly separate acquire vs review commands
- [x] `run_acquire()` has explicit, minimal command set
- [x] Edition detection works at build time (Vite) and runtime
- [x] Acquire components are in dedicated `src/components/acquire/` directory
- [x] Acquire hooks are in dedicated `src/hooks/acquire/` directory
- [x] CI validates Acquire edition builds independently (`build-acquire` job in tests.yml)
- [x] Import boundary lint prevents new cross-boundary imports (`scripts/lint-acquire-boundary.sh`)
- [ ] Cross-boundary UI imports resolved (Toast, CollapsibleGroup, etc.)
- [ ] All shared types available via `@core-suite/types`
