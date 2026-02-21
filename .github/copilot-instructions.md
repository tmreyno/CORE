# CORE-FFX AI Coding Instructions

CORE-FFX is a forensic file explorer built with **Tauri v2 (Rust backend) + SolidJS (TypeScript frontend)**. It handles forensic evidence containers (AD1, E01, UFED, etc.) with strict read-only operations.

---

## Critical Invariants

- **NEVER modify source evidence files** - all operations are read-only
- **Path traversal sanitization** - always use `common/` utilities for file paths
- **Hash verification** - prefer stored hashes when available; emit progress events for long operations
- **Library name is `ffx_check_lib`** - use this for test imports: `use ffx_check_lib::module::*`

---

## Architecture Overview

```text
src/                    # Frontend: SolidJS + TypeScript (Vite)
src-tauri/src/          # Backend: Rust + Tauri v2
  ├── lib.rs            # Tauri command registration (IPC surface)
  ├── commands/         # Tauri commands organized by feature
  ├── containers/       # Unified container abstraction layer
  ├── viewer/           # File viewers (hex, document, universal)
  │   └── document/     # Content viewers (PDF, email, plist, binary, etc.)
  ├── ad1/, ewf/, ufed/ # Format-specific parsers
  └── common/           # Shared utilities (hash, binary, segments)
libewf-ffi/             # Safe Rust FFI bindings for libewf 20251220 (EWF read/write)
sevenzip-ffi/           # C library + Rust FFI for 7z archive creation (LZMA SDK 24.09)
```

---

## CSS Architecture

### Style Pipeline

```text
┌─────────────────────────────────────────────────────────────────┐
│  1. variables.css    → Design tokens (CSS custom properties)     │
│  2. tailwind.config  → Tailwind theme extension (uses tokens)    │
│  3. index.css        → Base styles + @tailwind directives        │
│  4. App.css          → App-specific styles + fonts               │
└─────────────────────────────────────────────────────────────────┘
```

### Design Token Categories (`src/styles/variables.css`)

| Category | Variables | Example Usage |
|----------|-----------|---------------|
| **Background** | `--color-bg`, `--color-bg-secondary`, `--color-bg-panel`, `--color-bg-hover` | `bg-bg`, `bg-bg-secondary` |
| **Text** | `--color-txt`, `--color-txt-secondary`, `--color-txt-muted` | `text-txt`, `text-txt-muted` |
| **Accent** | `--color-accent`, `--color-accent-hover` | `text-accent`, `hover:text-accent-hover` |
| **Status** | `--color-success`, `--color-warning`, `--color-error`, `--color-info` | `text-success`, `text-error` |
| **Container Types** | `--color-type-ad1`, `--color-type-e01`, `--color-type-l01`, `--color-type-raw`, `--color-type-ufed`, `--color-type-archive` | `text-type-ad1`, `text-type-e01` |
| **Icons** | `--icon-size-micro` (12px), `--icon-size-small` (16px), `--icon-size-base` (20px), `--icon-size-lg` (24px) | `w-icon-sm h-icon-sm`, `w-icon-base h-icon-base` |
| **Spacing** | `--gap-compact` (4px), `--gap-small` (6px), `--gap-base` (8px) | `gap-compact`, `gap-small`, `gap-base` |

### Styling Patterns

```tsx
// ✅ GOOD - Semantic color classes
<div className="bg-bg text-txt border-border" />
<div className="bg-bg-secondary text-txt-secondary" />
<div className="text-accent hover:text-accent-hover" />

// ✅ GOOD - Container type colors
<span className="text-type-ad1">AD1</span>
<span className="text-type-e01">E01</span>
<span className="text-type-ufed">UFED</span>

// ✅ GOOD - Status colors
<span className="text-success">Verified</span>
<span className="text-warning">Pending</span>
<span className="text-error">Failed</span>

// ✅ GOOD - Icon sizing
<Icon className="w-icon-sm h-icon-sm" />   // 16px
<Icon className="w-icon-base h-icon-base" /> // 20px
<Icon className="w-icon-lg h-icon-lg" />   // 24px

// ❌ AVOID - Hardcoded colors
<div className="bg-zinc-900 text-zinc-100 border-zinc-700" />
```

### Z-Index Scale

```tsx
className="z-dropdown"       // Dropdowns
className="z-sticky"         // Sticky elements
className="z-modal-backdrop" // Modal backdrop
className="z-modal"          // Modal content
className="z-tooltip"        // Tooltips
className="z-notification"   // Toast notifications
```

### Animations

```tsx
<div className="animate-fade-in" />      // Fade in
<div className="animate-slide-up" />     // Slide up + fade
<div className="animate-slide-in" />     // Slide in from right
<div className="animate-pulse-slow" />   // Slow pulse
```

---

## UI Component Patterns (CSS Classes)

Use CSS component classes from `index.css` for consistency. **Prefer these over inline Tailwind utilities.**

### Buttons

```tsx
// Primary actions
<button class="btn btn-primary">Save</button>

// Secondary actions
<button class="btn btn-secondary">Cancel</button>

// Ghost/text buttons
<button class="btn btn-ghost">Skip</button>
<button class="btn-text">Learn more</button>
<button class="btn-text-danger">Delete</button>

// Small action buttons
<button class="btn-sm">Small</button>
<button class="btn-action-primary"><Icon /> Export</button>

// Icon-only buttons
<button class="icon-btn"><Icon class="w-5 h-5" /></button>
<button class="icon-btn-sm"><Icon class="w-4 h-4" /></button>
```

### Inputs & Forms

```tsx
<input class="input" placeholder="Standard input" />
<input class="input-sm" placeholder="Small input" />
<input class="input-inline" placeholder="Inline input (for horizontal layouts)" />
<input class="input-xs w-12" placeholder="Compact input (toolbars)" />
<textarea class="textarea" rows="4" />

<div class="form-group">
  <label class="label">Field Label</label>
  <input class="input" />
</div>

{/* Settings row pattern */}
<div class="flex items-center gap-2">
  <input class="input-inline" placeholder="File path..." />
  <button class="btn-sm">Browse</button>
</div>
```

### Cards & Panels

```tsx
<div class="card">Standard card</div>
<div class="card-interactive">Clickable card</div>
<div class="info-card">
  <div class="info-card-title"><Icon /> Title</div>
  Content
</div>

{/* Stat card - large centered stats */}
<div class="stat-card">
  <span class="stat-value">42</span>
  <span class="stat-label">Items</span>
</div>

{/* Stat box - compact inline stats for grids/dashboards */}
<div class="grid grid-cols-4 gap-4">
  <div class="stat-box">
    <div class="text-txt-muted text-xs">Label</div>
    <div class="text-xl font-semibold text-txt">Value</div>
  </div>
</div>
```

### Modals

```tsx
<div class="modal-overlay">
  <div class="modal-content w-[500px]">
    <div class="modal-header">
      <h2>Title</h2>
      <button class="icon-btn-sm"><X /></button>
    </div>
    <div class="modal-body">Content</div>
    <div class="modal-footer justify-end">
      <button class="btn btn-secondary">Cancel</button>
      <button class="btn btn-primary">Confirm</button>
    </div>
  </div>
</div>
```

### Badges & Chips

```tsx
<span class="badge badge-success">Verified</span>
<span class="badge badge-warning">Pending</span>
<span class="badge badge-error">Failed</span>

<button class="chip chip-cyan">Active</button>
<button class="chip chip-neutral">Inactive</button>
```

### Layout Helpers

```tsx
<div class="row">Flex row with gap-2</div>
<div class="row-between">Space between</div>
<div class="col">Flex column</div>
<div class="toolbar">Toolbar container</div>
<div class="panel-header">Panel header</div>
```

### UI Standards Reference

| Component | Border Radius | Padding |
|-----------|---------------|---------|
| Buttons | `rounded-lg` | `px-4 py-2` (base), `px-3 py-1.5` (sm) |
| Inputs | `rounded-lg` | `px-3 py-2.5` (base), `px-2.5 py-2` (sm) |
| Cards | `rounded-lg` | `p-4` |
| Modals | `rounded-xl` | Header/Footer: `px-5 py-4`, Body: `p-5` |
| Badges | `rounded` | `px-2 py-0.5` |
| Tooltips | `rounded` | `px-2 py-1` |
| Context Menus | `rounded-lg` | `py-1` |

---

## SolidJS Component Patterns

### Reactive Primitives

```tsx
import { createSignal, createMemo, createEffect, Show, For, onMount, onCleanup } from "solid-js";

// State management
const [value, setValue] = createSignal<string>("");
const [items, setItems] = createSignal<Item[]>([]);
const [loading, setLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

// Derived state
const filteredItems = createMemo(() => 
  items().filter(item => item.name.includes(searchQuery()))
);

// Side effects with explicit dependency tracking
createEffect(on(
  () => props.isOpen,
  (isOpen) => {
    if (isOpen) initializeData();
  }
));

// Lifecycle
onMount(() => {
  const listener = setupListener();
  onCleanup(() => listener.remove());
});
```

### Component Structure

```tsx
import { Component, Show, For } from "solid-js";
import { HiOutlineFolder, HiOutlineDocument } from "./icons";

interface MyComponentProps {
  items: Item[];
  onSelect: (item: Item) => void;
  isLoading?: boolean;
}

const MyComponent: Component<MyComponentProps> = (props) => {
  const [selected, setSelected] = createSignal<string | null>(null);

  return (
    <div className="bg-bg-panel rounded-md border border-border">
      <Show when={!props.isLoading} fallback={<Skeleton />}>
        <For each={props.items}>
          {(item) => (
            <div
              className="flex items-center gap-small p-2 hover:bg-bg-hover cursor-pointer"
              classList={{ "bg-bg-active": selected() === item.id }}
              onClick={() => {
                setSelected(item.id);
                props.onSelect(item);
              }}
            >
              <HiOutlineDocument className="w-icon-sm h-icon-sm text-txt-muted" />
              <span className="text-txt">{item.name}</span>
            </div>
          )}
        </For>
      </Show>
    </div>
  );
};
```

### Context Pattern (Toast example)

```tsx
import { createContext, useContext, type ParentComponent } from "solid-js";

interface ToastContextValue {
  success: (title: string, message?: string) => void;
  error: (title: string, message?: string) => void;
}

const ToastContext = createContext<ToastContextValue>();

export const ToastProvider: ParentComponent = (props) => {
  const [toasts, setToasts] = createSignal<Toast[]>([]);
  
  const value: ToastContextValue = {
    success: (title, message) => addToast({ type: "success", title, message }),
    error: (title, message) => addToast({ type: "error", title, message }),
  };

  return (
    <ToastContext.Provider value={value}>
      {props.children}
    </ToastContext.Provider>
  );
};

export const useToast = () => useContext(ToastContext)!;
```

---

## Hooks API Reference

### useFileManager

```tsx
const fileManager = useFileManager();

await fileManager.selectDirectory();      // Open directory picker
const files = fileManager.files();        // Get discovered files
fileManager.setFilter("ad1");             // Filter by type
fileManager.setActiveFile(file);          // Select active file
```

### useHashManager

```tsx
const hashManager = useHashManager();

hashManager.setAlgorithm("SHA-256");      // Set algorithm
await hashManager.hashActiveFile();        // Hash current file
const valid = await hashManager.verifyHash(file, expected);
```

### useProject

```tsx
const project = useProject();

await project.createProject(path, name);   // Create .cffx
await project.loadProject(projectPath);    // Load existing
await project.saveProject();               // Save current
const info = project.projectInfo();        // Get project metadata
```

### useProjectDbSync (Write-Through to .ffxdb)

```tsx
import { dbSync } from "./hooks/project/useProjectDbSync";

// Fire-and-forget sync — .cffx remains source of truth
dbSync.upsertBookmark(bookmark);           // Bookmark create/update
dbSync.deleteBookmark(bookmarkId);         // Bookmark delete
dbSync.upsertNote(note);                   // Note create/update
dbSync.deleteNote(noteId);                 // Note delete
dbSync.insertActivity(entry);             // Activity log entry
dbSync.upsertTag(tag);                     // Tag create/update
dbSync.assignTag(fileId, tagId, user);     // Tag assignment
dbSync.upsertSession(session);             // Session create/update
dbSync.upsertEvidenceFile(file);           // Evidence file upsert
dbSync.insertHash(hash);                   // Hash record
dbSync.insertVerification(verification);   // Hash verification record
dbSync.insertReport(report);              // Report record
dbSync.upsertSavedSearch(search);          // Saved search
dbSync.setUiState(key, value);             // UI state persistence
```

### useProjectDbRead (Seed .ffxdb from .cffx)

```tsx
import { seedDatabaseFromProject } from "./hooks/project/useProjectDbRead";

// Called once during project load — seeds empty .ffxdb tables
// from the loaded .cffx project state (idempotent)
await seedDatabaseFromProject(project);
```

---

## IPC Pattern: Frontend ↔ Backend

### Frontend (TypeScript)

```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Simple command
const info = await invoke<ContainerInfo>("logical_info", { path: filePath });

// With progress events
const unlisten = await listen<VerifyProgress>("verify-progress", (event) => {
  setProgress(event.payload.percent);
});
try {
  await invoke("verify_container", { path: filePath });
} finally {
  unlisten();
}
```

### Backend (Rust)

```rust
use tauri::command;

// Derive and serde attributes
#  [derive(Clone, serde::Serialize)]
#  [serde(rename_all = "camelCase")]
pub struct VerifyProgress {
    pub path: String,
    pub current: usize,
    pub total: usize,
    pub percent: f64,
}

// Tauri command attribute
#  [tauri::command]
pub async fn verify_container(
    path: String,
    window: tauri::Window,
) -> Result<bool, String> {
    // Emit progress events
    window.emit("verify-progress", VerifyProgress { 
        path: path.clone(), 
        current: 50, 
        total: 100, 
        percent: 50.0 
    }).map_err(|e| e.to_string())?;
    
    Ok(true)
}
```

**Registration:** All commands must be registered in `lib.rs` → `tauri::generate_handler![]`

---

## Backend Command Groups

Commands are organized in `src-tauri/src/commands/`:

| Module | Purpose | Example Commands |
|--------|---------|------------------|
| `container.rs` | AD1/container operations | `logical_info`, `logical_info_fast`, `container_get_root_children_v2`, `container_get_children_at_addr_v2`, `container_extract_entry_to_temp` |
| `archive/` | Archive browsing & extraction | Archive `metadata.rs`, `extraction.rs`, `nested.rs`, `tools.rs` |
| `archive_create.rs` | Archive creation | `create_7z_archive`, `estimate_archive_size`, `cancel_archive_creation` |
| `ewf.rs` | E01/EWF operations | `e01_v3_verify` |
| `ewf_export.rs` | EWF image creation (via libewf-ffi) | `ewf_create_image`, `ewf_estimate_size`, `ewf_cancel_create` |
| `hash.rs` | Batch hashing & queue | `batch_hash`, `hash_queue_pause`, `hash_queue_resume`, `hash_queue_clear_completed` |
| `viewer.rs` | File viewing | `viewer_read_chunk`, `viewer_detect_type`, `viewer_parse_header`, `viewer_read_text` |
| `analysis.rs` | File byte reading | `read_file_bytes` |
| `database.rs` | SQLite ops (15 commands) | `db_get_or_create_session`, `db_upsert_file`, `db_insert_hash`, `db_get_hashes_for_file` |
| `project.rs` | .cffx project files | `project_save`, `project_load`, `project_create`, `project_check_exists` |
| `project_advanced.rs` | Backup/versioning/recovery | `project_create_backup`, `project_create_version`, `project_check_recovery`, `project_recover_autosave` |
| `project_extended.rs` | Workspace profiles | `profile_list`, `profile_get`, `profile_set_active`, `profile_add`, `profile_update`, `profile_delete` |
| `discovery.rs` | File/directory scanning | `path_exists`, `discover_evidence_files`, `scan_directory_streaming`, `find_case_documents` |
| `export.rs` | File export | `export_files` |
| `lazy_loading.rs` | Lazy tree loading | `lazy_get_container_summary`, `lazy_get_root_children`, `lazy_get_children`, `lazy_get_settings` |
| `raw.rs` | Raw image verification | `raw_verify` |
| `system.rs` | System stats & utilities | `get_system_stats`, `cleanup_preview_cache`, `write_text_file`, `get_audit_log_path` |
| `vfs.rs` | Virtual filesystem | `vfs_mount_image`, `vfs_list_dir`, `vfs_read_file` |
| `ufed.rs` | UFED container operations | `ufed_info`, `ufed_info_fast`, `ufed_verify`, `ufed_get_stats`, `ufed_extract` |
| `project_db.rs` | Per-project .ffxdb (80+ cmds) | `project_db_open`, `project_db_get_stats`, `project_db_upsert_bookmark`, `project_db_search_fts`, `project_db_get_activity_log` |

**Processed database parsers** (`src-tauri/src/processed/`):

| Module | Purpose | Tauri Commands |
|--------|---------|----------------|
| `detection.rs` | Auto-detect processed DB tool type | — (internal API) |
| `axiom.rs` | Magnet AXIOM case parser | — (via processed commands) |
| `cellebrite.rs` | Cellebrite Physical Analyzer parser | `get_cellebrite_case_info`, `get_cellebrite_artifact_categories` |
| `autopsy.rs` | Autopsy case parser (.aut + autopsy.db) | `get_autopsy_case_info`, `get_autopsy_artifact_categories` |
| `commands.rs` | Tauri command wrappers | All processed DB commands |

---

## Adding a New Viewer/Parser

1. Create module in `src-tauri/src/viewer/document/` (e.g., `myformat.rs`)
2. Add `pub mod myformat;` to `viewer/document/mod.rs`
3. Extend `UniversalFormat` enum in `universal.rs` with new variant
4. Add Tauri commands to `viewer/document/commands.rs`
5. Register commands in `lib.rs`

**Existing viewer patterns:**
- `viewer/document/exif.rs` - EXIF metadata extraction
- `viewer/document/binary.rs` - PE/ELF/Mach-O analysis
- `viewer/document/email.rs` - EML/MBOX parsing
- `viewer/document/plist_viewer.rs` - Apple plist parsing
- `viewer/document/spreadsheet.rs` - Excel/CSV/ODS
- `viewer/document/office.rs` - DOCX/DOC/PPTX/PPT/ODT/ODP/RTF text extraction (uses `zip` + `quick-xml` + `cfb`)

---

## Type Alignment (Frontend ↔ Backend)

Keep TypeScript and Rust types synchronized:

| Frontend | Backend |
|----------|---------|
| `src/types/container.ts` | `src-tauri/src/containers/types.rs`, `src-tauri/src/formats.rs` |
| `src/types/containerInfo.ts` | `src-tauri/src/containers/types.rs` |
| `src/types/lazy-loading.ts` | `src-tauri/src/commands/lazy_loading.rs`, `src-tauri/src/common/lazy_loading.rs` |
| `src/types/viewer.ts` | `src-tauri/src/viewer/document/types.rs` |
| `src/types/project.ts` | `src-tauri/src/project.rs` |
| `src/types/database.ts` | `src-tauri/src/database.rs` |
| `src/types/projectDb.ts` | `src-tauri/src/project_db.rs`, `src-tauri/src/commands/project_db.rs` |
| `src/types/processed.ts` | `src-tauri/src/processed/types.rs` |
| `src/report/types.ts` | `src-tauri/src/report/types.rs` |
| `src/types/hash.ts` | `src-tauri/src/containers/types.rs` (StoredHash) |
| `src-tauri/src/archive/types.rs` | `src/types.ts` (ArchiveFormat, etc.) |
| `src/components/OfficeViewer.tsx` (inline types) | `src-tauri/src/viewer/document/office.rs` (OfficeDocumentInfo, OfficeMetadata, etc.) |

---

## Testing

```bash
cd src-tauri
cargo test                           # Run all tests
cargo test viewer::document::        # Run specific module tests
cargo test --test test_document_formats -- --nocapture  # Integration tests
cd ../libewf-ffi && cargo test       # libewf-ffi reader/writer tests (39 total)
```

---

## EWF Module Architecture — Two Separate Implementations

CORE-FFX has **two separate EWF implementations**. Do NOT confuse them.

| Module | Location | Purpose | Dependencies |
|--------|----------|---------|--------------|
| **libewf-ffi** | `libewf-ffi/` (workspace crate) | EWF image **creation** (write) and **reading** via C FFI to libewf 20251220 | libewf C library (`/opt/homebrew/Cellar/libewf/20251220/`) |
| **Pure-Rust EWF parser** | `src-tauri/src/ewf/` | E01/Ex01/L01 **read-only parsing** for container browsing, verify, VFS | No external dependencies (pure Rust) |

### When to Use Which

- **Creating EWF images** (export/acquisition) → `libewf-ffi::EwfWriter` via `commands/ewf_export.rs`
- **Reading EWF metadata for export/analysis** → `libewf-ffi::EwfReader` (wraps C library, full format support)
- **Browsing E01 containers in the tree** → `src-tauri/src/ewf/` (pure-Rust parser, `EwfHandle`, `EwfVfs`)
- **Verifying E01 hash integrity** → `commands/ewf.rs` → `ewf::operations` (pure-Rust)

### Key Files

| File | Purpose |
|------|---------|
| `libewf-ffi/src/reader.rs` | `EwfReader` — safe FFI wrapper for reading EWF images |
| `libewf-ffi/src/writer.rs` | `EwfWriter` — safe FFI wrapper for creating EWF images |
| `libewf-ffi/src/ffi.rs` | Raw FFI bindings to libewf C functions |
| `libewf-ffi/src/error.rs` | `EwfError` type |
| `src-tauri/src/ewf/handle.rs` | `EwfHandle` — pure-Rust EWF file reader |
| `src-tauri/src/ewf/parser.rs` | EWF section parsing (headers, volumes, tables) |
| `src-tauri/src/ewf/vfs.rs` | `EwfVfs` — virtual filesystem for E01/L01 content |
| `src-tauri/src/ewf/operations.rs` | Info, verify, extract, hash operations |
| `src-tauri/src/ewf/types.rs` | `EwfInfo`, `StoredImageHash`, `VerifyResult`, etc. |
| `src-tauri/src/commands/ewf_export.rs` | Tauri commands for EWF image creation |
| `src-tauri/src/commands/ewf.rs` | Tauri commands for E01 verification |
| `src/api/ewfExport.ts` | Frontend API for EWF export |

### Do NOT

- Confuse `libewf-ffi::EwfReader` with `ewf::EwfHandle` — they are separate implementations
- Use `libewf-ffi` for container tree browsing — use the pure-Rust `ewf/` module
- Use the pure-Rust `ewf/` module for image creation — use `libewf-ffi::EwfWriter`
- Forget CString null-termination when adding new FFI functions to `libewf-ffi/src/ffi.rs`
- Map `"ex01"` to `EwfFormat::Encase7` — it must map to `EwfFormat::V2Encase7` (see `parse_format()` in `ewf_export.rs`)

---

## sevenzip-ffi — 7z Archive Creation Library

Custom C library with Rust FFI wrapper for creating 7z archives. Uses **LZMA SDK 24.09** (must match or exceed Homebrew's `sevenzip` formula version).

### Key Files

| File | Purpose |
|------|---------|
| `sevenzip-ffi/src/archive_create.c` | Single-volume 7z archive creation |
| `sevenzip-ffi/src/archive_create_multivolume.c` | Split/multi-volume 7z archive creation |
| `sevenzip-ffi/src/utf8_utf16.h` | UTF-8 → UTF-16LE filename encoding (MUST be used for all filenames) |
| `sevenzip-ffi/lzma/C/` | LZMA SDK 24.09 C source files (86 files) |
| `sevenzip-ffi/include/7z_ffi.h` | Public C API header |
| `sevenzip-ffi/src/lib.rs` | Rust FFI wrapper |
| `sevenzip-ffi/build/lib7z_ffi.a` | Pre-built static library (macOS arm64) |
| `sevenzip-ffi/CMakeLists.txt` | Build configuration |

### Critical Invariants

1. **UTF-8 → UTF-16LE filenames**: 7z format stores filenames as UTF-16LE. ALL filename encoding MUST use `utf8_to_utf16le()` and `utf8_to_utf16le_size()` from `src/utf8_utf16.h`. **NEVER** use the ASCII-only loop (`*p++ = (Byte)*name++; *p++ = 0;`) — it corrupts non-ASCII filenames (CJK, emoji, accented chars).

2. **Dictionary sizes must match SDK 24.09 defaults**: When `dict_size = 0`:
   - STORE (level 0): 64KB (`1 << 16`)
   - FASTEST (level 1): 256KB (`1 << 18`)
   - FAST (level 3): 4MB (`1 << 22`)
   - NORMAL (level 5): 32MB (`1 << 25`)
   - MAXIMUM (level 7): 128MB (`1 << 27`)
   - ULTRA (level 9): 256MB (`1 << 28`)
   
   The multivolume creator lets `Lzma2EncProps_Normalize()` set dictionary from level (correct — it uses SDK defaults automatically).

3. **Entropy threshold = 220**: Both `archive_create.c` and `archive_create_multivolume.c` use `unique_bytes < 220` for compressibility detection. Keep them in sync.

4. **Dynamic header allocation**: `build_7z_header()` uses `calc_7z_header_size()` for pre-allocation + `CHECK_SPACE()` macro for safety. The old 256KB fixed buffer caused heap overflow with >625 files. **NEVER replace dynamic allocation with a fixed buffer.**

5. **SDK version**: LZMA SDK files in `lzma/C/` are version **24.09**. SDK 24.09 auto-detects `MY_CPU_ARM64` from `__aarch64__` — do NOT add `MY_CPU_ARM64` to CMake compile definitions (it will cause a redefinition warning).

### Build & Deploy

```bash
# Build
cd /Users/terryreynolds/GitHub/sevenzip-ffi
rm -rf build && mkdir build && cd build
cmake -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release ..
make -j$(sysctl -n hw.ncpu)

# Test
cd .. && cargo test --lib --tests

# Deploy to CORE-1
cp build/lib7z_ffi.a /Users/terryreynolds/GitHub/CORE-1/sevenzip-ffi/build/lib7z_ffi.a
cd /Users/terryreynolds/GitHub/CORE-1/src-tauri && cargo check
```

### Do NOT

- Use the ASCII-only encoding loop for filenames — use `utf8_to_utf16le()` from `utf8_utf16.h`
- Use hardcoded dictionary sizes that don't match SDK 24.09 defaults (see table above)
- Use a fixed-size buffer for `build_7z_header()` — it will overflow with many files
- Add `MY_CPU_ARM64` to CMakeLists.txt — SDK 24.09 detects it automatically
- Change the entropy threshold (220) in one file without updating the other
- Downgrade the LZMA SDK from 24.09 to an older version

---

## Filesystem Drivers (`src-tauri/src/common/filesystem/`)

Read-only filesystem drivers for parsing partitions inside forensic images. All implement the `FilesystemDriver` trait from `traits.rs`.

| Driver | File | Supported Types |
|--------|------|-----------------|
| FAT | `fat.rs` | FAT12, FAT16, FAT32 |
| exFAT | `exfat_driver.rs` | exFAT (64-bit, large files) |
| NTFS | `ntfs_driver.rs` | NTFS |
| HFS+ | `hfsplus_driver.rs` | HFS+, HFSX |
| APFS | `apfs_driver.rs` | APFS |
| ext | `ext_driver.rs` | ext2, ext3, ext4 |
| DMG | `dmg_driver.rs` | Apple DMG container |

**Auto-detection**: `detect_filesystem_type()` in `mod.rs` reads boot sector magic bytes to identify the filesystem.

**Adding a new filesystem driver**: Create a module implementing `FilesystemDriver` (methods: `info()`, `getattr()`, `readdir()`, `read()`), add the match arm in `mount_filesystem()`, and add detection logic to `detect_filesystem_type()`.

---

## Processed Database Parsers (`src-tauri/src/processed/`)

Parsers for forensic tool output databases, enabling CORE-FFX to read case data from third-party tools.

| Parser | File | Input Formats | Key Types |
|--------|------|---------------|-----------|
| AXIOM | `axiom.rs` | Magnet AXIOM case dirs | `AxiomCaseInfo`, `AxiomArtifactCategory` |
| Cellebrite | `cellebrite.rs` | report.xml + SQLite DBs | `CellebriteCaseInfo`, `CellebriteArtifactCategory`, `CellebriteDataSource` |
| Autopsy | `autopsy.rs` | .aut files + autopsy.db | `AutopsyCaseInfo`, `AutopsyDataSource`, `AutopsyArtifactCategory`, `AutopsyTag` |
| Detection | `detection.rs` | Auto-detect tool type | `ProcessedDbType` enum |

**Adding a new parser**: Create module in `processed/`, add `pub mod` to `mod.rs`, add Tauri commands to `commands.rs`, register in `lib.rs`, and add detection logic to `detection.rs`.

---

## Development Commands

```bash
npm install                 # Install frontend dependencies
npm run tauri dev           # Development mode with hot reload
npm run tauri build         # Production build
cargo check                 # Quick Rust compilation check
```

---

## Key Documentation

| Document | Purpose |
|----------|---------|
| `CODE_BIBLE.md` | Authoritative codebase map and glossary |
| `CRATE_API_NOTES.md` | **Third-party crate API reference — check before using any crate** |
| `FRONTEND_API_NOTES.md` | **SolidJS/TypeScript API reference — check before writing frontend code** |
| `docs/SEVENZIP_FFI_API_REFERENCE.md` | **sevenzip-ffi C API docs — dict sizes, UTF-8, SDK 24.09 details** |
| `src-tauri/src/README.md` | Backend module structure |
| `src/components/README.md` | Frontend component catalog |
| `src/hooks/README.md` | State management hooks reference |
| `src/styles/README.md` | Tailwind CSS styling guide |

---

## Common Patterns

### Progress Events

```rust
window.emit("verify-progress", VerifyProgress { path, current, total, percent })?;
```

### Container Abstraction

Use `containers/` traits for format-agnostic operations rather than calling format-specific code directly.

### Error Handling

Return `Result<T, String>` from Tauri commands; avoid `.unwrap()` in production paths.

### File Header License

All source files should include the standard header:

```rust
// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
```

### Toolbar & Project State Timing

The toolbar's `ProjectLocationSelector` is a `<select>` driven by `scanDir` (the selected value) and `buildProjectLocations()` (the options). Several timing invariants must be preserved to avoid stale paths or missing dropdown entries:

1. **setScanDir BEFORE project signal updates.** In `handleProjectSetupComplete`, call `fileManager.setScanDir(locations.evidencePath)` **before** `createProject()` / `updateLocations()`. In `handleLoadProject`, clear with `setScanDir("")` before `loadProject()`. Because SolidJS updates are synchronous within a microtask but `await` creates new microtasks, the `<select>` value must already match the new options when reactivity fires.

2. **Session restore must guard against project load.** `restoreLastSession()` runs async and may resolve after the user opens a project. Always guard with `if (lastSession && !projectManager.hasProject())` to avoid overwriting a freshly-set `scanDir`.

3. **Processed Database path needs a fallback.** The `processedDbPath` accessor in `App.tsx` must fall back to deriving from `processedDbManager.databases()` when `projectLocations()?.processed_db_path` is null/empty — otherwise older projects without a `locations` field won't show the "Processed Database" entry in the toolbar dropdown.

4. **Older projects: derive locations in step 1b.** When loading a `.cffx` without a `locations` field, `handleLoadProject` must derive locations from `cached_databases`, `loaded_paths`, and `case_documents_cache` and call `updateLocations()` so the toolbar dropdown is populated.

Key files: `src/hooks/project/projectHelpers.ts` (handleLoadProject, handleProjectSetupComplete), `src/App.tsx` (Toolbar props, session restore), `src/components/toolbar/toolbarHelpers.ts` (buildProjectLocations).

### UI Layout Invariants (Title Bar, Toolbar, Quick Actions, StatusBar)

The application shell has a strict layout hierarchy. **Do NOT re-add removed elements** — these decisions were intentional.

**Title Bar (`<header class="app-header">` in App.tsx):**
- ✅ Logo only (brand-logo image)
- ✅ Project badge (project name + modified indicator dot) — conditionally shown when a project is loaded
- ✅ Quick Actions toggle button (⚡ `HiOutlineBolt` icon) — toggles `showQuickActions` signal, positioned `ml-auto`
- ❌ Do NOT add app name (`brand-name`), tagline (`brand-tag`), status indicator (`header-status`/`status-dot`), or `ProfileSelector` back to the title bar

**Toolbar (`src/components/Toolbar.tsx`):**
- ✅ Save dropdown (Save, Save As, Auto-save toggle)
- ✅ `ProjectLocationSelector` (evidence path, processed DB, case docs dropdown)
- ✅ Scan button
- ✅ Hash section (algorithm selector, hash button, info button)
- ❌ Do NOT add an Open dropdown (`onBrowse`, `onOpenProject`), project name badge, or recursive scan toggle back to the Toolbar
- ❌ `ToolbarProps` does NOT include `recursiveScan`, `onRecursiveScanChange`, `onBrowse`, `onOpenProject` — do not re-add these props

**Recursive Scan:**
- The `recursiveScan` signal in `useFileManager.ts` defaults to `true` and there is **no UI toggle** for it. Directory scanning is always recursive.
- The signal and its setter still exist in `useFileManager` for API compatibility, but no component sets it to `false`.

**Quick Actions Bar (`src/components/QuickActionsBar.tsx`):**
- Hidden by default — controlled by `showQuickActions` signal (default `false`) in App.tsx
- Toggled via the ⚡ button in the title bar
- Wrapped in `<Show when={showQuickActions()}>` in App.tsx
- ❌ Do NOT make it visible by default or remove the `<Show>` wrapper

**StatusBar (`src/components/StatusBar.tsx`):**
- Shows evidence counts (discovered, selected, total size), system stats, progress items, and auto-save status
- ✅ Also shows project stats: activity events (`HiOutlineClipboardDocumentList`), bookmarks (`HiOutlineBookmark`), notes (`HiOutlineDocumentText`) — conditionally rendered when any count > 0
- `StatusBarProps` includes: `activityCount`, `bookmarkCount`, `noteCount` (all optional numbers)
- Passed from App.tsx: `activityCount={projectManager.project()?.activity_log?.length ?? 0}`, `bookmarkCount={projectManager.bookmarkCount()}`, `noteCount={projectManager.noteCount()}`

**Profile Selector:**
- Lives in the **project setup wizard** (`src/components/wizard/ConfigureLocationsStep.tsx`), NOT in the title bar or toolbar
- Rendered in a "Workspace Profile" section after the "Project Name" field
- `ConfigureLocationsStep` accepts optional `onProfileChange?: (profileId: string) => void` prop
- `ProfileSelector` internally uses `useWorkspaceProfiles` hook — no state threading needed

**Key files:** `src/App.tsx` (shell layout, signals), `src/components/Toolbar.tsx` (toolbar content), `src/components/StatusBar.tsx` (status bar), `src/components/QuickActionsBar.tsx` (quick actions), `src/components/wizard/ConfigureLocationsStep.tsx` (profile selector), `src/hooks/useFileManager.ts` (recursive scan signal).

### Case Documents Tree Design

The case documents tree (left panel, "casedocs" tab) uses a **compact single-line layout** — each row shows only:
- A document icon (`HiOutlineDocumentText`)
- The filename (truncated)
- A small, muted format label to the right (PDF, DOCX, TXT, etc.)

**All file attributes and metadata** (size, modified date, case number, evidence ID, document type) are shown in the **right panel** (`ViewerMetadataPanel` → `FileInfoTab`) when the document is selected and opened as a tab. This keeps the tree clean and scannable.

**Do NOT add size, dates, case numbers, open-external buttons, or viewer buttons (HEX, TXT, etc.) back to `DocumentItem.tsx`.** The metadata flows through this pipeline:
1. `createDocumentEntry()` stores `document_type`, `case_number`, `evidence_id`, `format`, `modified` in `SelectedEntry.metadata`
2. `ContainerEntryViewer` reads `entry.metadata` and populates `ViewerMetadata.fileInfo` (via `FileInfoMetadata` optional fields)
3. `ViewerMetadataPanel` → `FileInfoTab` renders a "Case Info" section with case number, evidence ID, and document type

**Do NOT re-add viewer buttons (HEX, TXT, etc.) to `DocumentItem.tsx` or pass `onViewHex`/`onViewText` props through the component chain.** The prop chain was intentionally removed:
- `DocumentItem.tsx` — compact layout, no metadata props, no `onOpenExternal`, no `onViewHex`/`onViewText`
- `CaseDocumentsPanel.tsx` — no `onViewHex`/`onViewText` props, does not pass `onOpenExternal` to DocumentItem
- `LeftPanelContent.tsx` — no `onViewHex`/`onViewText` props
- `CollapsiblePanelContent.tsx` — no `onViewHex`/`onViewText` props
- `App.tsx` — does not pass `onViewHex`/`onViewText` to `LeftPanelContent`

The `useEntryNavigation` hook still exports `handleCaseDocViewHex`/`handleCaseDocViewText` for programmatic use, but they are not wired into the tree UI.

Key files: `src/components/casedocs/DocumentItem.tsx`, `src/components/CaseDocumentsPanel.tsx`, `src/components/layout/LeftPanelContent.tsx`, `src/components/layout/CollapsiblePanelContent.tsx`, `src/types/viewerMetadata.ts` (FileInfoMetadata), `src/components/ViewerMetadataPanel.tsx` (FileInfoTab), `src/hooks/project/projectHelpers.ts` (createDocumentEntry).

---

### Archive Container Tree & Viewer Architecture

Archive containers (ZIP, 7z, TAR, GZ, RAR, DMG, ISO, etc.) use a **synthesized directory** approach because many archive formats (especially 7z) don't include explicit directory entries in their file listings.

**Critical invariant: `useEvidenceTree.getArchiveRootEntries` MUST delegate to `archive.getArchiveRootEntries(entries)` — NEVER filter entries manually.** The `useArchiveTree.getArchiveRootEntries` calls `synthesizeDirectories()` internally, which creates virtual directory entries from file paths. Without this, archives where all entries are like `folder/file.txt` (no explicit `folder/` entry) will show an **empty tree**.

**Data flow for archive tree expansion:**
1. `useEvidenceTree.toggleContainer()` → calls `archive.loadArchiveTree(path)` → invokes `archive_get_tree` Tauri command
2. Backend (`commands/archive/metadata.rs`) reads archive via `libarchive_backend.rs` → returns `ArchiveTreeEntry[]`
3. `synthesizeDirectories()` scans all entry paths, creates virtual `isDir: true` entries for intermediate directories
4. `getArchiveRootEntries()` returns only entries without `/` in their path (after synthesis)
5. `getArchiveChildren(allEntries, parentPath)` returns direct children of a directory

**Data flow for archive entry viewing (center panel):**
1. User clicks file → creates `SelectedEntry` with `isArchiveEntry: true`
2. `ContainerEntryViewer` → `useEntrySource.readBytesFromSource()` → invokes `archive_read_entry_chunk` for hex data
3. Preview mode → `container_extract_entry_to_temp` → `libarchive_read_file` extracts to temp → shows in appropriate viewer

**Data flow for archive metadata (right panel):**
1. `EvidenceTree.tsx` enriches `SelectedEntry.metadata` with `archiveFormat`, `totalEntries`, `totalFiles`, `totalFolders`, `archiveSize`, `encrypted`, `entryCompressedSize`, `entryCrc32`, `entryModified`
2. `ContainerEntryViewer.tsx` builds `ArchiveMetadataSection` from `entry.metadata` when `isArchiveEntry` is true
3. `ViewerMetadataPanel.tsx` renders `ArchiveSection` with collapsible "Archive Info" and "Entry Details" groups

**Key files:**
- `src/components/EvidenceTree/hooks/useArchiveTree.ts` — archive tree state, `synthesizeDirectories`, `getArchiveRootEntries`, `getArchiveChildren`, `getAllWithSyntheticDirs`
- `src/components/EvidenceTree/hooks/useEvidenceTree.ts` — master hook, `getArchiveRootEntries` MUST delegate to `archive.getArchiveRootEntries`
- `src/components/EvidenceTree.tsx` — renders archive tree, enriches `SelectedEntry.metadata` with archive data
- `src/components/EvidenceTree/nodes/ArchiveTreeNode.tsx` — recursive archive tree node with nested container support
- `src/components/EvidenceTree/nodes/VfsTreeNode.tsx` — VFS tree node with nested container support (ZIP inside E01)
- `src/components/EvidenceTree/nodes/Ad1TreeNode.tsx` — AD1 tree node with nested container support (ZIP inside AD1)
- `src/components/ContainerEntryViewer.tsx` — builds `ArchiveMetadataSection` for right panel
- `src/components/ViewerMetadataPanel.tsx` — `ArchiveSection` component renders archive metadata
- `src/types/viewerMetadata.ts` — `ArchiveMetadataSection` interface
- `src-tauri/src/commands/archive/metadata.rs` — `archive_get_metadata`, `archive_get_tree` backend commands
- `src-tauri/src/commands/archive/extraction.rs` — `archive_extract_entry`, `archive_read_entry_chunk`
- `src-tauri/src/archive/libarchive_backend.rs` — `LibarchiveHandler` unified backend for all archive formats

**Do NOT:**
- Bypass `synthesizeDirectories()` when listing archive root entries
- Add manual path filtering (e.g., `!path.includes('/')`) for archive root entries — use `archive.getArchiveRootEntries()`
- Assume archive entries always include explicit directory entries
- Use `allArchiveEntries()` directly for file/folder counts — use `allSynthesizedEntries()` which includes virtual directories

---

### Nested Containers Inside All Parent Types (VFS, AD1, Archive)

Nested container support allows container files (ZIP, 7z, AD1, E01, etc.) **inside** other containers to be expanded inline in the evidence tree. This works for **all three parent container types**: VFS (E01/Raw), AD1, and Archive.

**Critical invariant: All three tree node types — `VfsTreeNode`, `Ad1TreeNode`, and `ArchiveTreeNode` — MUST have nested container detection and expansion support.** Each detects container files via `isNestedContainerFile()`, renders expand icons, and delegates to the `useNestedContainers` hook. Removing nested container props from ANY of these tree nodes will break containers-inside-containers for that parent type.

**Backend extraction pipeline (`src-tauri/src/commands/archive/nested.rs`):**

`get_or_create_nested_temp()` extracts a nested file from its parent to a temp directory. It detects the parent container type by **file signature** (not extension) before falling through to archive extraction:

1. **E01 parent** (`ewf::is_ewf()`) → opens `EwfVfs`, reads file via VFS `file_size()` + `read()`, writes to temp
2. **Raw parent** (`raw::is_raw()`) → opens `RawVfs` (filesystem or raw), reads via VFS, writes to temp
3. **AD1 parent** (`ad1::is_ad1()`) → uses `ad1::read_entry_data()` to read entry, writes to temp
4. **Archive parent** (else branch) → matches extension (zip, 7z, rar, tar, etc.) and extracts via `libarchive_read_file()` or `SevenZipHandler`

**Frontend nested container pattern (replicated in all 3 tree nodes):**

```tsx
// Props interface includes:
isNestedExpanded?: (parentPath: string, nestedPath: string) => boolean;
isNestedLoading?: (parentPath: string, nestedPath: string) => boolean;
getNestedEntries?: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
onToggleNested?: (parentPath: string, nestedPath: string) => Promise<void>;
onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;

// Component body includes:
const isNestedContainer = createMemo(() => !entry.isDir && isNestedContainerFile(entry.name));
// ... expansion state, root entries filtering, VfsNestedEntryNode/Ad1NestedEntryNode/NestedContainerEntryNode rendering
```

**EvidenceTree.tsx wiring — all three container sections wire `tree.nested.*` methods as props:**
- VFS section → `PartitionNode` → `VfsTreeNode` (lines ~216)
- Archive section → `ArchiveTreeNode` (lines ~299)  
- AD1 section → `Ad1TreeNode` (lines ~414)

**Key files:**
- `src/components/EvidenceTree/nodes/VfsTreeNode.tsx` — VFS tree node with nested container support, includes `VfsNestedEntryNode`
- `src/components/EvidenceTree/nodes/Ad1TreeNode.tsx` — AD1 tree node with nested container support, includes `Ad1NestedEntryNode`
- `src/components/EvidenceTree/nodes/ArchiveTreeNode.tsx` — Archive tree node with nested container support, includes `NestedContainerEntryNode`
- `src/components/EvidenceTree/hooks/useNestedContainers.ts` — shared hook managing nested container state, caching, and IPC
- `src/components/EvidenceTree/containerDetection.ts` — `isNestedContainerFile()`, `getNestedContainerType()`, `NESTED_CONTAINER_EXTENSIONS`
- `src-tauri/src/commands/archive/nested.rs` — `get_or_create_nested_temp()`, `nested_container_get_tree()`, `nested_archive_read_entry_chunk()`

**Do NOT:**
- Remove nested container props (`isNestedExpanded`, `onToggleNested`, etc.) from `VfsTreeNode`, `Ad1TreeNode`, or `ArchiveTreeNode`
- Remove nested container props from `PartitionNodeProps` — they must pass through to `VfsTreeNode`
- Remove the `isNestedContainer` memo or `isNestedContainerFile()` detection from any tree node
- Remove the E01/Raw/AD1 parent detection in `get_or_create_nested_temp()` — the `is_ewf`/`is_raw`/`is_ad1` checks MUST run before the archive extension match
- Remove `VfsNestedEntryNode` from `VfsTreeNode.tsx` or `Ad1NestedEntryNode` from `Ad1TreeNode.tsx`
- Skip wiring `tree.nested.*` props when rendering `PartitionNode` or `Ad1TreeNode` in `EvidenceTree.tsx`

---

### Viewer-Inside-Container Pipeline

All file viewers (PDF, Office, Spreadsheet, Email, PST, Image, Database, Plist, Binary, Registry, DocumentViewer) work inside forensic containers (E01, AD1, ZIP, TAR, 7z, etc.) through a unified extraction-then-render pipeline. This section documents the critical data flow to prevent regressions.

**End-to-end pipeline (user clicks file in tree → viewer renders):**

1. **Tree click** → `EvidenceTree.tsx` calls `props.onSelectEntry({ containerPath, entryPath, name, size, isDir, isVfsEntry, isArchiveEntry, dataAddr, ... })`
2. **Entry navigation** → `useEntryNavigation.handleSelectEntry()` calls:
   - `centerPaneTabs.openContainerEntry(entry)` → creates/focuses entry tab, sets CenterPane `viewMode("document")`
   - `setEntryContentViewMode("auto")` → triggers ContainerEntryViewer's reactive effect
3. **Auto-preview effect** → `ContainerEntryViewer.createEffect` detects `mode === "auto"`:
   - `shouldAttempt = true` (because `canPreview(name) || mode === "auto"`)
   - Calls `handlePreview()` → `setPreviewLoading(true)` → spinner renders
4. **Extraction** → `handlePreview()` invokes `container_extract_entry_to_temp`:
   - **Archive** (`isArchiveEntry: true`): `libarchive_read_file()` → temp file
   - **VFS** (`isVfsEntry: true` or E01/Raw detected): `EwfVfs.read()` / `RawVfs.read()` with `entrySize` → temp file
   - **AD1** (auto-detected via `is_ad1()`): `ad1::read_entry_data_by_addr()` or `ad1::read_entry_data()` → temp file
   - **Disk file** (`isDiskFile: true` or `containerPath === entryPath`): uses path directly, no extraction
5. **Content detection** (unknown extensions only): `detect_content_format` with magic-byte analysis → may set `detectedFormat()`
6. **Viewer routing** → `effectiveMode()` returns `"preview"`, `<Switch>` matches the correct viewer:
   - `fileIsPdf()` → `PdfViewer`
   - `fileIsImage()` → `ImageViewer` + `ExifPanel`
   - `fileIsSpreadsheet()` → `SpreadsheetViewer`
   - `fileIsOffice()` → `OfficeViewer`
   - `fileIsEmail()` → `EmailViewer`
   - `fileIsPst()` → `PstViewer`
   - `fileIsPlist()` → `PlistViewer`
   - `fileIsBinary()` → `BinaryViewer`
   - `fileIsRegistry()` → `RegistryViewer`
   - `fileIsDatabase()` → `DatabaseViewer`
   - `fileIsDetectedText()` → `TextViewer` (with extracted temp file)
   - Fallback → `DocumentViewer`

**Two separate viewMode signals (do NOT conflate):**
- `centerPaneTabs.viewMode` — CenterPane's internal state, set to `"document"` by `openContainerEntry()`
- `entryContentViewMode` — from `useAppState`, passed to ContainerEntryViewer as `viewMode` prop, set to `"auto"` by `handleSelectEntry()`

**`canPreview()` function** (in `ContainerEntryViewer.tsx`) — MUST include ALL previewable type guards:
```tsx
isPdf || isImage || isSpreadsheet || isOffice || isTextDocument || isCode || isConfig || isEmail || isPst || isPlist || isBinaryExecutable || isDatabase || isRegistryHive
```
If you add a new viewer type, you MUST add its type guard to `canPreview()` or it will only work via content detection fallback.

**Key files:**
- `src/components/ContainerEntryViewer.tsx` — main viewer wrapper, extraction, routing, metadata emission
- `src/utils/fileTypeUtils.ts` — centralized file type detection (extension arrays and type guards)
- `src/hooks/useEntryNavigation.ts` — entry selection, sets `entryContentViewMode("auto")`
- `src/hooks/useCenterPaneTabs.ts` — tab management, `openContainerEntry()`
- `src/hooks/useAppState.ts` — `entryContentViewMode` signal
- `src-tauri/src/commands/container.rs` — `container_extract_entry_to_temp` backend extraction
- `src-tauri/src/viewer/document/universal.rs` — `UniversalFormat` enum, `from_extension()`, `detect_by_magic()`, `viewer_type()`
- `src-tauri/src/viewer/document/commands.rs` — `detect_content_format` backend command

**Do NOT:**
- Remove any type guard from `canPreview()` — this gates whether auto-preview triggers
- Break the `else if` chain in the metadata `viewerType` assignment — each branch must use `else if`
- Remove `isPst` from `canPreview()` — PST files need it to trigger auto-preview
- Change `handleSelectEntry` to set `entryContentViewMode` to anything other than `"auto"` — `"auto"` is the universal trigger
- Assume `containerPath === entryPath` means "container entry" — it means "disk file" (no extraction needed)
- Remove the content detection fallback for unknown extensions — it enables magic-byte-based viewer routing

---

## AI Agent Error Prevention Rules

These rules exist because ~90% of historical compilation errors fall into three categories. **Follow them in order before writing any new Rust or TypeScript code.**

### Rule 1: Verify Third-Party Crate APIs (Prevents ~50% of Errors)

**NEVER guess method names, field names, or return types for external crates.**

Before using any crate API:
1. **Check `CRATE_API_NOTES.md`** in the project root — it documents actual signatures for all major crates used in this project.
2. **If the crate isn't documented there**, verify by running:
   ```bash
   grep -rn "pub fn\|pub struct\|pub enum" ~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/ | head -50
   ```
3. Pay special attention to:
   - Whether a method returns `Result<T>`, `Option<T>`, or `T` directly
   - Whether a method takes arguments (e.g., `body_text(0)` not `body_text()`)
   - Whether something returns an iterator vs a `Vec` vs a slice
   - Field name spelling (e.g., `libs` vs `libraries`, `is_dir` vs `is_directory`)

**Common traps already documented:**
- `msg_parser`: No `body_html` field; `bcc` is `String` not `Vec<Person>`
- `mail-parser`: `headers()` returns `&[Header]` (slice); `body_text(pos)` requires an index
- `goblin`: `macho.symbols()` returns iterator (no `?`); MachO field is `libs` not `libraries`
- `notatin`: Use `std::sync::LazyLock`, NOT `once_cell`
- `libewf-ffi`: CString required for all identifier/value passing; hash needs dual binary+UTF-8 API; `EwfFormat::Encase7` → `.E01` NOT `.Ex01`; SHA1/model/serial_number NOT stored in Encase5

### Rule 2: Verify Internal Types Before Use (Prevents ~25% of Errors)

**NEVER assume field or variant names for types defined in other modules.**

Before referencing any internal struct, enum, or trait:
1. **Grep for the definition** in the project:
   ```bash
   grep -rn "pub struct MyType\|pub enum MyType" src-tauri/src/
   ```
2. **Read the actual field list** before using any field name.
3. **Check `CRATE_API_NOTES.md` § Internal Types** for documented gotchas.

**Common traps:**
- `ArchiveEntryInfo.is_dir` (NOT `is_directory`); no `compressed_size` field
- `ArchiveFormat::Rar4` / `Rar5` (NOT `Rar`); `Iso` (NOT `Iso9660`)
- `StoredHash.hash` (NOT `value`)
- Use `std::sync::LazyLock` (NOT `once_cell::sync::Lazy` or `lazy_static!`)

### Rule 3: Keep Frontend ↔ Backend Types in Sync (Prevents ~15% of Errors)

**After modifying any Rust struct with `serde(rename_all = "camelCase")` attribute, update the matching TypeScript interface.**

Type sync map — these files must stay aligned:

| Rust Source | TypeScript Interface |
|-------------|---------------------|
| `src-tauri/src/containers/types.rs` | `src/types/container.ts`, `src/types/containerInfo.ts`, `src/types/hash.ts` |
| `src-tauri/src/formats.rs` | `src/types/container.ts` |
| `src-tauri/src/project.rs` | `src/types/project.ts` |
| `src-tauri/src/report/types.rs` | `src/report/types.ts` |
| `src-tauri/src/viewer/document/types.rs` | `src/types/viewer.ts` |
| `src-tauri/src/archive/types.rs` | `src/types.ts` (`ArchiveFormat`, etc.) |
| `src-tauri/src/commands/lazy_loading.rs` | `src/types/lazy-loading.ts` |
| `src-tauri/src/database.rs` | `src/types/database.ts` |
| `src-tauri/src/project_db.rs` | `src/types/projectDb.ts` |
| `src-tauri/src/processed/types.rs` | `src/types/processed.ts` |
| `src-tauri/src/commands/ewf_export.rs` | `src/api/ewfExport.ts` (EwfExportOptions) |

**Workflow when changing a Rust struct:**
1. Make the Rust change
2. Search for the TypeScript interface: `grep -rn "interface MyType" src/`
3. Add/rename/remove the corresponding TypeScript field (use `camelCase`)
4. If adding a new enum variant in Rust, add the same string to the TypeScript union type

### Rule 4: Run `cargo check` Early and Often

After every non-trivial code change, run `cargo check` before moving to the next file. This catches API mismatches immediately instead of accumulating them.
