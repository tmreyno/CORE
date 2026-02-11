# CLAUDE.md - Project Instructions for Claude

This file provides guidance to Claude Code (claude.ai/code) when working with this codebase.

## Project Overview

CORE-FFX is a forensic file explorer built with **Tauri v2 (Rust backend) + SolidJS (TypeScript frontend)**. It handles forensic evidence containers (AD1, E01, UFED, etc.) with strict read-only operations.

## Critical Invariants

- **NEVER modify source evidence files** - all operations are read-only
- **Path traversal sanitization** - always use `common/` utilities for file paths
- **Hash verification** - prefer stored hashes when available; emit progress events for long operations
- **Library name is `ffx_check_lib`** - use this for test imports: `use ffx_check_lib::module::*`

## Build & Development Commands

```bash
# Frontend
npm install                 # Install frontend dependencies
npm run tauri dev           # Development mode with hot reload
npm run tauri build         # Production build

# Backend
cd src-tauri
cargo check                 # Quick Rust compilation check
cargo test                  # Run all tests
cargo test viewer::document::  # Run specific module tests
cargo test --test test_document_formats -- --nocapture  # Integration tests
```

## Architecture

```
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

## CSS Architecture

### Style Pipeline

```
variables.css → tailwind.config → index.css → App.css
(tokens)        (theme extension)  (base+directives)  (app styles)
```

### Styling Rules

**DO use semantic Tailwind classes:**
```tsx
<div className="bg-bg text-txt border-border" />
<div className="bg-bg-secondary text-txt-secondary" />
<span className="text-type-ad1">AD1</span>
<span className="text-success">Verified</span>
<Icon className="w-icon-sm h-icon-sm" />
```

**DON'T use hardcoded colors:**
```tsx
// ❌ AVOID
<div className="bg-zinc-900 text-zinc-100" />
```

### Design Token Reference

| Category | Classes |
|----------|---------|
| Background | `bg-bg`, `bg-bg-secondary`, `bg-bg-panel`, `bg-bg-hover` |
| Text | `text-txt`, `text-txt-secondary`, `text-txt-muted` |
| Accent | `text-accent`, `hover:text-accent-hover` |
| Status | `text-success`, `text-warning`, `text-error`, `text-info` |
| Container Types | `text-type-ad1`, `text-type-e01`, `text-type-l01`, `text-type-raw`, `text-type-ufed`, `text-type-archive` |
| Icon Sizes | `w-icon-sm h-icon-sm` (16px), `w-icon-base h-icon-base` (20px), `w-icon-lg h-icon-lg` (24px) |
| Spacing | `gap-compact` (4px), `gap-small` (6px), `gap-base` (8px) |
| Z-Index | `z-dropdown`, `z-sticky`, `z-modal-backdrop`, `z-modal`, `z-tooltip`, `z-notification` |
| Animations | `animate-fade-in`, `animate-slide-up`, `animate-slide-in`, `animate-pulse-slow` |

## SolidJS Patterns

### State Management

```tsx
import { createSignal, createMemo, createEffect, Show, For, onMount, onCleanup } from "solid-js";

const [value, setValue] = createSignal<string>("");
const [items, setItems] = createSignal<Item[]>([]);

// Derived state
const filteredItems = createMemo(() => 
  items().filter(item => item.name.includes(searchQuery()))
);

// Side effects with explicit dependency tracking
createEffect(on(
  () => props.isOpen,
  (isOpen) => { if (isOpen) initializeData(); }
));
```

### Component Template

```tsx
import { Component, Show, For } from "solid-js";
import { HiOutlineDocument } from "./icons";

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
              onClick={() => props.onSelect(item)}
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

## Hooks Reference

```tsx
// File management
const fileManager = useFileManager();
await fileManager.selectDirectory();
const files = fileManager.files();
fileManager.setFilter("ad1");

// Hash operations
const hashManager = useHashManager();
hashManager.setAlgorithm("SHA-256");
await hashManager.hashActiveFile();

// Project persistence
const project = useProject();
await project.createProject(path, name);
await project.loadProject(projectPath);
```

## IPC Pattern (Frontend ↔ Backend)

### Frontend

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

### Backend

```rust
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
    window.emit("verify-progress", VerifyProgress { 
        path: path.clone(), 
        current: 50, 
        total: 100, 
        percent: 50.0 
    }).map_err(|e| e.to_string())?;
    
    Ok(true)
}
```

**Note:** All commands must be registered in `lib.rs` → `tauri::generate_handler![]`

## Backend Command Groups

| Module | Purpose |
|--------|---------|
| `commands/container.rs` | AD1 operations |
| `commands/lazy_loading.rs` | Unified lazy loading |
| `commands/archive.rs` | Archive operations |
| `commands/ufed.rs` | UFED parsing |
| `commands/ewf.rs` | E01/EWF operations |
| `commands/hash.rs` | Batch hashing |
| `commands/viewer.rs` | File viewing |
| `commands/analysis.rs` | Hex/entropy |
| `commands/database.rs` | SQLite ops |
| `commands/project.rs` | .ffxproj files |

## Adding a New Viewer/Parser

1. Create module in `src-tauri/src/viewer/document/` (e.g., `myformat.rs`)
2. Add `pub mod myformat;` to `viewer/document/mod.rs`
3. Extend `UniversalFormat` enum in `universal.rs`
4. Add Tauri commands to `viewer/document/commands.rs`
5. Register commands in `lib.rs`

**Reference implementations:**
- `viewer/document/exif.rs` - EXIF metadata
- `viewer/document/binary.rs` - PE/ELF/Mach-O
- `viewer/document/email.rs` - EML/MBOX
- `viewer/document/plist_viewer.rs` - Apple plist
- `viewer/document/spreadsheet.rs` - Excel/CSV/ODS

## Type Alignment

| Frontend | Backend |
|----------|---------|
| `src/types/formats.ts` | `src-tauri/src/formats.rs` |
| `src/types/lifecycle.ts` | `src-tauri/src/containers/traits.rs` |
| `src/report/types.ts` | `src-tauri/src/report/types.rs` |

## Error Handling

- Return `Result<T, String>` from Tauri commands
- Avoid `.unwrap()` in production paths
- Emit progress events for long operations

## File Header

All source files should include:

```rust
// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
```

## Key Documentation

- `CODE_BIBLE.md` - Authoritative codebase map and glossary
- `src-tauri/src/README.md` - Backend module structure
- `src/components/README.md` - Frontend component catalog
- `src/hooks/README.md` - State management hooks
- `src/styles/README.md` - Tailwind CSS styling guide
