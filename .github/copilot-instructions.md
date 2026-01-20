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

await project.createProject(path, name);   // Create .ffxproj
await project.loadProject(projectPath);    // Load existing
await project.saveProject();               // Save current
const info = project.projectInfo();        // Get project metadata
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

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyProgress {
    pub path: String,
    pub current: usize,
    pub total: usize,
    pub percent: f64,
}

#[tauri::command]
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
| `container.rs` | AD1 operations | `logical_info`, `container_get_tree` |
| `lazy_loading.rs` | Unified lazy loading | `container_get_children` |
| `archive.rs` | Archive operations | `archive_list`, `archive_extract` |
| `ufed.rs` | UFED parsing | `ufed_list_associated` |
| `ewf.rs` | E01/EWF operations | `ewf_info`, `ewf_verify` |
| `hash.rs` | Batch hashing | `batch_hash`, `verify_hash` |
| `viewer.rs` | File viewing | `read_file_chunk`, `detect_format` |
| `analysis.rs` | Hex/entropy | `hex_dump`, `calculate_entropy` |
| `database.rs` | SQLite ops | `db_save_hash`, `db_get_settings` |
| `project.rs` | .ffxproj files | `create_project`, `load_project` |

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

---

## Type Alignment (Frontend ↔ Backend)

Keep TypeScript and Rust types synchronized:

| Frontend | Backend |
|----------|---------|
| `src/types/formats.ts` | `src-tauri/src/formats.rs` |
| `src/types/lifecycle.ts` | `src-tauri/src/containers/traits.rs` |
| `src/report/types.ts` | `src-tauri/src/report/types.rs` |

---

## Testing

```bash
cd src-tauri
cargo test                           # Run all tests
cargo test viewer::document::        # Run specific module tests
cargo test --test test_document_formats -- --nocapture  # Integration tests
```

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
