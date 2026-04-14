# CORE-FFX AI Coding Instructions

CORE-FFX is a forensic file explorer built with **Tauri v2 (Rust backend) + SolidJS (TypeScript frontend)**. It handles forensic evidence containers (AD1, E01, UFED, etc.) with strict read-only operations.

Frontend observability wrappers live in `src/utils/logger.ts`, `src/utils/telemetry.ts`, and `src/utils/performance.ts`, but the reusable implementation now lives in `@core-suite/logging` and `@core-suite/logging/performance`. Keep the CORE-FFX files thin and only use them for FFX-specific audit storage keys or bootstrap behavior.

---

## Critical Invariants

- **NEVER modify source evidence files** - all operations are read-only
- **Path traversal sanitization** - always use `common/` utilities for file paths
- **Hash verification** - prefer stored hashes when available; emit progress events for long operations
- **Library name is `ffx_check_lib`** - use this for test imports: `use ffx_check_lib::module::*`
- **CORE-FFX owns its runtime namespaces** - non-portable app state must live under `com.core-ffx.desktop`, `core-ffx/logs`, and `core-ffx-*` temp dirs. Global audit logs and support bundles should keep FFX-owned names such as `ffx-audit.YYYY-MM-DD.log` and `core-ffx-logs-YYYY-MM-DD.zip`. Do not reuse legacy `com.ffxcheck.app` or CORE-ACQ runtime paths in CORE.

---

## Architecture Overview

```text
src/                    # Frontend: SolidJS + TypeScript (Vite)
src-tauri/src/          # Backend: Rust + Tauri v2
  ├── lib.rs            # Tauri command registration + macOS keep-alive (.build().run())
  ├── menu.rs           # Native menu bar + multi-window (File, Edit, View, Tools, Window, Help)
  ├── commands/         # Tauri commands organized by feature
  ├── containers/       # Unified container abstraction layer (re-exports from ffx-containers)
  ├── viewer/           # File viewers (hex, document, universal)
  │   └── document/     # Content viewers (PDF, email, plist, binary, etc.)
  ├── ad1/, ewf/, ufed/ # Re-export shims → crates/ffx-ad1, ffx-ewf, ffx-ufed
  ├── archive/, raw/    # Re-export shims → crates/ffx-archive, ffx-raw
  ├── common/           # Re-export shim → crates/ffx-common
  ├── l01_writer/       # Pure-Rust L01 logical evidence writer
  └── aff4/             # Re-export shim → crates/ffx-aff4
crates/                 # Extracted workspace crates (standalone, testable)
  ├── ffx-ad1/          # AD1 forensic container parser
  ├── ffx-aff4/         # AFF4 forensic container (pure Rust)
  ├── ffx-archive/      # Archive formats (ZIP, 7z, TAR, RAR, ISO, etc.)
  ├── ffx-common/       # Shared utilities (hash, binary, VFS, segments)
  ├── ffx-containers/   # Container traits, error types, operations
  ├── ffx-errors/       # Shared error types across crates
  ├── ffx-ewf/          # EWF/E01/L01 parser
  ├── ffx-formats/      # Format detection and file type utilities
  ├── ffx-l01-writer/   # Pure-Rust L01 logical evidence writer (crate)
  ├── ffx-raw/          # Raw disk image support
  └── ffx-ufed/         # UFED forensic container parser
libewf-ffi/             # Safe Rust FFI bindings for libewf 20251220 (EWF read/write)
sevenzip-ffi/           # C library + Rust FFI for 7z archive creation (LZMA SDK 24.09)
```

---

## Modular Feature System (Cargo Features)

CORE-FFX uses **Cargo feature flags** to produce separate binaries for different editions from the same codebase. Features are composable — you can mix modules to create custom editions.

### Edition Features (select one)

| Feature | Includes | Binary Size (macOS) | Use Case |
|---------|----------|-------------------|----------|
| `full` (default) | All modules + AI + unrar | ~49 MB | Full forensic suite |
| `acquire` | `flavor-acquire` only | ~27 MB | Field acquisition tool (45% smaller) |
| `review` | `flavor-review` only | ~48 MB | Analysis/review workstation |

### Capability Modules (composable)

| Module | Dependencies | Controls |
|--------|-------------|----------|
| `flavor-acquire` | (none — marker) | Acquisition commands (E01/L01/7z creation, hashing, drives, portable) |
| `flavor-review` | `mod-viewers`, `mod-search`, `mod-reports`, `mod-processed` | All review/analysis capabilities |
| `mod-viewers` | calamine, cfb, mail-parser, msg_parser, outlook-pst, goblin, notatin, pdf-extract, lopdf, image, kamadak-exif, csv, rust_xlsxwriter | Document viewers (PDF, Office, Email, Spreadsheet, Binary, Registry, etc.) |
| `mod-search` | tantivy | Full-text search engine + file deduplication |
| `mod-reports` | genpdf, docx-rs, tera | Forensic report generation (PDF, DOCX, HTML) |
| `mod-processed` | (none — code only) | Processed DB parsers (AXIOM, Cellebrite, Autopsy) |
| `ai-assistant` | async-openai, reqwest, url | AI-powered report narrative generation |

Report narrative generation should prefer OpenAI GPT-5 as the default writing model when the AI assistant feature is enabled. The frontend may pass a typed key from the report wizard, but it must also allow the backend to rely on an existing `OPENAI_API_KEY` environment variable instead of enforcing a frontend-only hard block.
The report wizard persists the selected AI provider/model in `ffx-preferences`, but typed API keys remain session-only and must not be stored in preferences.

### Custom Edition Examples

```bash
# Review-only (no acquisition commands)
cargo build --no-default-features --features review

# Full suite (default)
cargo build
```

### Build Command (via Tauri CLI)

```bash
# Full edition (default)
npm run tauri build
```

Standalone CORE-Acquisition builds now belong in the CORE-ACQ repo. Do not build or release the Acquire app from CORE.

### Architecture: `lib.rs` Dispatch

```rust
pub fn run() {
    #[cfg(feature = "flavor-review")]
    run_full();           // All commands (viewers, search, reports, etc.)

    #[cfg(not(feature = "flavor-review"))]
    run_acquire();        // Shared + acquire commands only
}

// Always compiled — can be called directly by any edition
pub fn run_acquire() { ... }

// Only compiled with flavor-review feature
#[cfg(feature = "flavor-review")]
fn run_full() { ... }
```

`run_acquire()` is always compiled (unconditionally `pub`) so it's available regardless of which features are enabled. `run_full()` is only compiled when `flavor-review` is active because it references cfg-gated modules (search, dedup, viewer::document, report, processed, etc.).

### Module → Feature Gate Mapping

**Rust modules gated by `#[cfg(feature = "flavor-review")]`:**
- `activity_timeline`, `dedup`, `processed`, `project_comparison`, `project_recovery`, `project_templates`, `report`, `search`, `workspace_profiles`, `workspace_profile_types`, `workspace_profile_defaults`
- `viewer::document` (submodule only — hex/text viewer always available)
- `commands::dedup`, `commands::project_advanced`, `commands::project_extended`, `commands::project_merge`, `commands::search`, `commands::viewer`

**Always-compiled modules (shared by all editions):**
- `ad1`, `archive`, `commands` (core), `common`, `containers`, `database`, `ewf`, `formats`, `l01_writer`, `logging`, `menu`, `project`, `project_db`, `raw`, `ufed`, `viewer` (hex/text only)

### Do NOT

- Remove `pub` from `run_acquire()` — it must be callable from both the default `run()` dispatch and potential future external entry points
- Add `#[cfg]` gate back to `run_acquire()` — it must compile unconditionally
- Reference cfg-gated modules (search, viewer::document, report, processed, dedup) from `run_acquire()` — those are only available in `run_full()`
- Reintroduce CORE-Acquisition build or release instructions in this repo — standalone Acquire builds now belong in CORE-ACQ
- Change `default = ["full"]` — the default dev/debug build must be the full suite

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
| **Icons** | `--icon-size-micro` (14px), `--icon-size-small` (18px), `--icon-size-base` (22px), `--icon-size-lg` (26px) | `w-icon-sm h-icon-sm`, `w-icon-base h-icon-base` |
| **Spacing** | `--gap-compact` (4px), `--gap-small` (6px), `--gap-base` (8px) | `gap-compact`, `gap-small`, `gap-base` |

### Typography Scale (Standardized)

All text sizes use Tailwind tokens from a single standardized scale. **Never use arbitrary `text-[Xpx]` values.** The scale is defined in `tailwind.config.js` and backed by CSS variables in `variables.css`.

**Font Size Preference (`fontSize` in `AppPreferences`, default 14):**

The user can adjust the base font size (12–18px) via Settings → Appearance → Font Size. The preference drives `--app-font-size-*` CSS custom properties set by `usePreferenceEffects.ts`. App.css applies these via `!important` overrides on `.text-2xs` through `.text-2xl`, ensuring ALL Tailwind text tokens scale uniformly when the slider moves.

| Preference Value | text-2xs | text-compact | text-xs | text-sm | text-base | text-lg | text-xl | text-2xl |
|-----------------|----------|-------------|---------|---------|-----------|---------|---------|----------|
| **12** (min) | 8px | 9px | 10px | 11px | 12px | 14px | 16px | 20px |
| **14** (default) | 10px | 11px | 12px | 13px | 14px | 16px | 18px | 22px |
| **16** | 12px | 13px | 14px | 15px | 16px | 18px | 20px | 24px |
| **18** (max) | 14px | 15px | 16px | 17px | 18px | 20px | 22px | 26px |

Offsets from base: `2xs = base-4`, `compact = base-3`, `xs = base-2`, `sm = base-1`, `base = base`, `lg = base+2`, `xl = base+4`, `2xl = base+8`.

At default (14), the minimum readable text size (`text-xs`) is 12px. Micro tokens (`text-2xs`, `text-compact`) go below 12px at default but are only used for decorative elements (hex bytes, badge counts, section headers).

**Key files:** `src/hooks/usePreferenceEffects.ts` (sets CSS vars), `src/App.css` (defaults + `!important` overrides), `src/components/settings/AppearanceTab.tsx` (slider UI), `src/components/preferences.ts` (`fontSize` field, default 14).

**Do NOT:**
- Remove the `!important` overrides in App.css — they are the mechanism that makes the font size preference actually work
- Use different offset formulas — the offsets MUST match the Tailwind scale spacing (see table above)
- Add `--app-font-size-*` variables without corresponding `!important` class overrides — both are required
- Set `--app-font-size-xs` to `base - 4` — that was the old bug; it must be `base - 2`

| Token | Size | Line Height | Role | Usage Examples |
|-------|------|-------------|------|----------------|
| `text-2xs` | 10px | 14px | **Micro** — tiny indicators, hex bytes | Badge counts, checkmarks, hex viewer bytes, section header text |
| `text-compact` | 11px | 16px | **Mono** — fixed-width data values | Hash values, font-mono metadata, tree constants |
| `text-xs` | 12px | 16px | **Caption** — secondary information | Metadata labels, timestamps, stat values, tab text |
| `text-sm` | 13px | 18px | **Body-sm** — dense content areas | Panel content, tree items, table cells, list items |
| `text-base` | 14px | 20px | **Body** — primary readable content | Form inputs, buttons, general paragraphs |
| `text-lg` | 16px | 22px | **Heading** — section/panel titles | Panel headers, modal section titles, card titles |
| `text-xl` | 18px | 26px | **Title** — prominent headings | Modal titles, page headings |
| `text-2xl` | 22px | 30px | **Display** — large callouts | Dashboard stat numbers, hero values |

**Role-based rules (which size for what):**

| UI Element | Token | Notes |
|------------|-------|-------|
| App body (default) | `text-base` | Set on `<body>` via index.css |
| Form inputs, buttons | `text-sm` / `text-base` | Small inputs use `text-sm`, standard use `text-base` |
| Panel/card body text | `text-sm` | Dense information areas |
| Metadata key-value labels | `text-xs` | Right panel, file info |
| Metadata mono values | `text-compact` | Hash strings, paths in mono |
| Tab bar labels | `text-xs` | Primary and secondary tab bars |
| Section headers (uppercase) | `text-2xs` | `text-2xs uppercase tracking-wider font-medium text-txt-muted` |
| Badge text | `text-2xs` | Status badges, count badges |
| Modal titles | `text-xl` | Modal header `<h2>` or similar |
| Panel header titles | `text-xs` / `text-lg` | Small panels use `text-xs`, large panels `text-lg` |
| Stat card values | `text-2xl` | Dashboard large numbers |
| Hex viewer bytes | `text-2xs` | Monospace grid display |
| Tree node text | `text-sm` | Evidence tree, file tree |
| Toast/notification text | `text-sm` | Toast messages |
| Empty state message | `text-sm` | Centered muted text |
| Toolbar text | `text-xs` / `text-sm` | Labels and buttons |
| Breadcrumbs | `text-2xs` | Path segments |

```tsx
// ✅ GOOD — Use standardized tokens
<span className="text-xs text-txt-muted">Label</span>
<span className="text-sm text-txt">Content</span>
<span className="font-mono text-compact">SHA-256: abc123...</span>
<h2 className="text-lg font-medium">Panel Title</h2>
<span className="text-2xs uppercase tracking-wider">SECTION</span>

// ❌ NEVER — Arbitrary pixel values
<span className="text-[10px]">...</span>
<span className="text-[11px]">...</span>
<span className="text-[9px]">...</span>
```

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
<Icon className="w-icon-sm h-icon-sm" />   // 18px
<Icon className="w-icon-base h-icon-base" /> // 22px
<Icon className="w-icon-lg h-icon-lg" />   // 26px

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

## Icon System

All icons use **Heroicons Outline** (`HiOutline*`) from `solid-icons/hi`, centralized through `src/components/icons/index.tsx`. No `HiSolid*` or `HiMini*` variants are used anywhere.

### Import Rule

```tsx
// ✅ CORRECT — import from centralized icons module
import { HiOutlineArchiveBox, HiOutlineFingerPrint } from "./icons";
// or from relative path:
import { HiOutlineArchiveBox } from "../components/icons";

// ❌ NEVER — import directly from solid-icons
import { HiOutlineArchiveBox } from "solid-icons/hi";
```

### Canonical Feature → Icon Mapping

Each feature uses **one canonical outline icon** everywhere (sidebar buttons, command palette, quick actions bar, modal headers). Context menus use emoji strings (because `ContextMenuItem.icon` is typed as `string`).

| Feature | Icon Component | QuickActions Key | Context Menu Emoji |
|---------|---------------|------------------|-------------------|
| Evidence Containers | `HiOutlineArchiveBox` | — | — |
| Evidence Collection | `HiOutlineArchiveBoxArrowDown` | `evidence` | 📦 |
| Generate Report | `HiOutlineClipboardDocumentList` | `report` | 📝 |
| Open Project | `HiOutlineDocumentCheck` | — | — |
| Open Directory | `HiOutlineFolderOpen` | `folder` | — |
| Compute Hash | `HiOutlineFingerPrint` | `fingerprint` | — |
| Search | `HiOutlineMagnifyingGlass` | `search` | — |
| Export | `HiOutlineArrowUpTray` | `export` | 📤 |
| Verify | `HiOutlineCheckBadge` | `verify` | — |
| Settings | `HiOutlineCog6Tooth` | `settings` | ⚙️ |
| Bookmarks | `HiOutlineBookmark` | `bookmark` | 📑 |
| Activity Timeline | `HiOutlineClock` | — | — |
| Case Documents | `HiOutlineClipboardDocumentList` | — | — |
| Processed DBs | `HiOutlineChartBar` | `chart` | — |
| Command Palette | `HiOutlineCommandLine` | — | 🔧 |
| Deduplication | `HiOutlineDocumentDuplicate` | `duplicate` | — |
| Performance | `HiOutlineBolt` | `bolt` | — |
| Lock/Encryption | `HiOutlineLockClosed` | — | — |
| Dashboard | `HiOutlineRectangleGroup` | — | — |
| Help / User Guide | `HiOutlineQuestionMarkCircle` | — | — |

**Do NOT:**
- Import icons directly from `solid-icons/hi` — always go through `icons/index.tsx`
- Use `HiSolid*` or `HiMini*` icon variants
- Use `HiOutlineLockClosed` for hash/fingerprint operations — use `HiOutlineFingerPrint`
- Use `HiOutlineDocumentText` for report or evidence collection features — that icon is for text/document content
- Use the same icon for different features (e.g., both "Generate Report" and "Evidence Collection")

---

## Help Panel (Center-Pane Tab)

The Help Panel is a **comprehensive in-app documentation system** that opens as a center-pane tab. It provides 15 searchable documentation sections covering all major features.

### Entry Points

- **Help → User Guide** (native menu bar)
- **Command Palette** (`Cmd+K` → "User Guide")
- `centerPaneTabs.openHelpTab()`

### CenterPane Tab Integration

- `CenterTabType` includes `"help"`
- Tab ID: `"__help__"`, type: `"help"`, title: "Help & Documentation"
- `useCenterPaneTabs` exposes: `openHelpTab()`

### Sections (15 total)

Getting Started, Evidence Containers, File Viewers, Hash Verification, Search & Deduplication, Export Formats, Reports, Chain of Custody, Evidence Collection, Processed Databases, Project Management, Filesystem Drivers, Bookmarks & Notes, Keyboard Shortcuts, About CORE-FFX

**Key files:**

| File | Purpose |
|------|----------||
| `src/components/HelpPanel.tsx` | Main help panel component (15 sections, sidebar nav, search) |
| `src/hooks/useCenterPaneTabs.ts` | `openHelpTab()` method |
| `src/hooks/useMenuActions.ts` | `onUserGuide` handler |
| `src/hooks/useCommandPalette.tsx` | "User Guide" command palette action |
| `src-tauri/src/menu.rs` | "User Guide" menu item in Help submenu |

### Do NOT

- Open help as a modal — it is a center-pane tab
- Remove the "User Guide" menu item from the Help submenu — it is the primary entry point
- Add the help tab to `PROJECT_DEPENDENT_IDS` — help should be available without a project loaded

---

## Workspace Modes (Feature Module System)

Workspace Modes allow users to show/hide UI features based on their current workflow. CORE-FFX is the main platform; individual feature areas are treated as modules that can be toggled on or off. The system is **preference-driven** (no dynamic loading or plugin architecture).

### Architecture

```text
AppPreferences (localStorage)
  ├── workspaceMode: string         — active preset ID (default "full")
  └── customEnabledModules: string[] — per-module toggle list for "custom" mode

useWorkspaceMode() hook
  ├── activeMode()       — resolved WorkspaceModePreset
  ├── enabledModules()   — FeatureModule[] currently enabled
  ├── isModuleEnabled(m) — boolean check
  ├── setMode(id)        — switch preset
  ├── toggleModule(m)    — toggle one module (auto-switches to custom)
  ├── setCustomModules() — bulk-set custom modules
  └── getFirstEnabledTab() — fallback tab when active tab is disabled
```

### Feature Modules (6)

| Module ID | Name | Controls |
|-----------|------|----------|
| `forensicExplorer` | Forensic Explorer | Evidence tree, hash section, scan, container ops, hex/text viewers |
| `evidenceCollection` | Evidence Collection | Collection forms, COC management, linked data |
| `documentReview` | Document Review | Case documents panel, document viewers |
| `searchAnalysis` | Search & Analysis | File deduplication, processed database parsers (AXIOM, Cellebrite, Autopsy) |
| `reportExport` | Report, Acquire & Export | Report wizard, forensic acquisition (E01/L01), file export (7z/copy), merge projects |
| `caseManagement` | Case Management | Dashboard, activity timeline, project management |

**Always-available features (not gated by any module):**
- **Search** — Full-text search across evidence containers and case documents (universal investigation tool)
- **Bookmarks & Notes** — Universal annotation tools for all workflows
- **Settings, Help, Command Palette, Theme** — Core app utilities

### Workspace Presets (7)

| Preset ID | Name | Enabled Modules |
|-----------|------|-----------------|
| `full` | Full Suite (default) | All 6 modules |
| `forensic` | Forensic Explorer | forensicExplorer, searchAnalysis |
| `collection` | Evidence Collection & COC | evidenceCollection, forensicExplorer, caseManagement |
| `review` | Document Review | documentReview, searchAnalysis |
| `analysis` | Search & Analysis | searchAnalysis, forensicExplorer |
| `reporting` | Report, Acquire & Export | reportExport, forensicExplorer, evidenceCollection |
| `custom` | Custom | User-selected via per-module toggles |

### UI Entry Points

- **Toolbar dropdown** (`WorkspaceModeSelector`): Quick mode switching (left-most toolbar item)
- **Settings tab** (`WorkspaceModeTab`): First tab in Settings panel — preset cards and per-module toggles
- Switching to a preset seeds `customEnabledModules` from that preset's modules

### Module → UI Element Mapping

**Sidebar navigation tabs** (`TAB_MODULE_MAP` in `useWorkspaceMode.ts`):
- dashboard → `caseManagement`
- evidence → `forensicExplorer`
- processed → `searchAnalysis`
- casedocs → `documentReview`
- activity → `caseManagement`
- bookmarks → always visible (universal annotation tool)

**Sidebar tool buttons:**
- search → always visible (universal investigation tool)
- deduplication → `searchAnalysis`
- export/report → `reportExport`
- command palette, settings, help, theme → always visible

**Toolbar sections:**
- Hash section (algorithm selector, hash button, load metadata) → `forensicExplorer`
- Save section, location selector → always visible

**Quick action buttons** (`ACTION_MODULE_MAP` in `useWorkspaceMode.ts`):
- hash/verify → `forensicExplorer`
- search → always visible (universal investigation tool)
- dedup → `searchAnalysis`
- export/report → `reportExport`
- evidence → `evidenceCollection`
- bookmarks → always visible (universal annotation tool)
- settings/command → always visible

### Auto-Tab-Switch Effect

A `createEffect` in App.tsx watches `workspaceMode.enabledModules()`. When the active sidebar tab's required module is disabled, it auto-switches to `getFirstEnabledTab()` (ordered: dashboard → evidence → processed → casedocs → activity → bookmarks). Tabs without a required module (bookmarks) are always considered valid.

### Key Files

| File | Purpose |
|------|---------|
| `src/components/preferences.ts` | `FeatureModule` type, `FEATURE_MODULES`, `WorkspaceModePreset`, `WORKSPACE_PRESETS`, `getWorkspacePreset()` |
| `src/hooks/useWorkspaceMode.ts` | Hook: `activeMode`, `enabledModules`, `isModuleEnabled`, `setMode`, `toggleModule`, `getFirstEnabledTab`, `TAB_MODULE_MAP`, `ACTION_MODULE_MAP` |
| `src/components/settings/WorkspaceModeTab.tsx` | Settings tab — preset grid + per-module toggle list |
| `src/components/toolbar/WorkspaceModeSelector.tsx` | Toolbar dropdown for quick mode switching |
| `src/components/layout/sidebar/Sidebar.tsx` | Wraps navigation buttons in `<Show when={mod(...)}>` |
| `src/components/layout/sidebar/types.ts` | `SidebarProps.isModuleEnabled` optional prop |
| `src/components/Toolbar.tsx` | Wraps hash section in `<Show when={mod("forensicExplorer")}>` |
| `src/components/QuickActionsBar.tsx` | Filters actions via `ACTION_MODULE_MAP` + `isModuleEnabled` |
| `src/App.tsx` | Wires `useWorkspaceMode` to Sidebar, Toolbar, QuickActionsBar; auto-tab-switch effect |

### Do NOT

- Remove the "full" preset or change its default — it enables all modules and is the initial state
- Remove `isModuleEnabled` from `SidebarProps` — it gates all navigation tabs
- Remove the auto-tab-switch `createEffect` — without it, disabled tabs stay selected and show blank panels
- Store workspace mode in `.ffxdb` — it's a user preference (localStorage), not project data
- Gate settings, help, command palette, or theme switching by any module — they must always be accessible
- Gate search, bookmarks, or notes by any module — they are universal tools needed by all workflows (document review, forensic analysis, case management)
- Remove `TAB_MODULE_MAP` or `ACTION_MODULE_MAP` — they are the source of truth for module → UI element mapping
- Add new sidebar tabs without adding them to `TAB_MODULE_MAP` — they won't be gated by workspace mode

---

## Evidence Collection (Center-Pane Tab)

Evidence collection is a **standalone on-site acquisition form**, completely separate from the Report Wizard. It is NOT a report type. It renders as a **center-pane tab** (not a modal), with its linked data tree displayed in the **right panel**.

### Architecture

```text
EvidenceCollectionPanel.tsx          # Center-pane tab form
  ├── useFormTemplate({ templateId: "evidence_collection" })  # JSON schema
  │     └── autoFillContext: { examiner: { name }, project: { case_number, name } }
  ├── SchemaFormRenderer                                       # Renders form
  ├── useFormPersistence                                       # Auto-save (debounced)
  ├── cocDbSync.ts                                             # Awaitable save to .ffxdb (via direct invoke)
  └── onLinkedNodesChange → App.tsx → RightPanel               # Emits linked data to right panel

LinkedDataTree.tsx                   # Reusable tree component (shared)
LinkedDataPanel.tsx                  # Right-panel wrapper (tabs: Linked Data, Summary)

EvidenceCollectionListPanel.tsx      # Browse/list all collections (center-pane tab)
```

### Key Files

| File | Purpose |
|------|---------|
| `src/components/EvidenceCollectionPanel.tsx` | Tab-based form (schema-driven, no WizardContext) |
| `src/components/EvidenceCollectionListPanel.tsx` | Browse/list all evidence collections; user-facing package import/export entry point |
| `src/components/report/wizard/cocExport.ts` | Save-dialog helper for PDF/XLSX/CSV/HTML export plus canonical JSON package import/export |
| `src/components/LinkedDataTree.tsx` | Reusable tree: `LinkedDataNode` type + `LinkedDataTree` component |
| `src/components/LinkedDataPanel.tsx` | Right-panel wrapper with Linked Data & Summary tabs |
| `src/templates/forms/evidence_collection.json` | JSON schema template (v1.3.0 — 3 sections, reordered forensic workflow, `evidence_container` field, conditional `show_when` device fields) |
| `src/components/report/wizard/cocDbSync.ts` | DB persistence (shared with COC). **Awaitable** — uses direct `invoke()`, NOT fire-and-forget `dbSync` |
| `src/components/report/types.ts` | `EvidenceCollectionData`, `CollectedItem` types |
| `src/components/evidence-collection/evidenceAutoFill.ts` | Maps container metadata (E01/AD1/UFED/L01) to ~30 form fields; includes L01 source metadata enrichment, stored intake hashes, examiner names, acquisition duration, AD1 filesystem/OS info |
| `src/components/evidence-collection/formDataConversion.ts` | Bidirectional `EvidenceCollectionData` ↔ `FormData` conversion (all ~30 fields + photo_refs) |
| `src-tauri/src/report/commands.rs` | Backend evidence collection report/export commands for PDF/XLSX/CSV/HTML and legacy form-driven JSON export |
| `src-tauri/src/commands/project_db/collections.rs` | Canonical `.ffxdb`-driven package import/export via `project_db_export_evidence_collection_package` and `project_db_import_evidence_collection_package` |

### Entry Points

- **Sidebar**: Right-click report button → "Evidence Collection…" context menu item → opens tab via `centerPaneTabs.openEvidenceCollection()`
- **Command Palette**: `Cmd+K` → "Evidence Collection" or "Evidence Collection List"
- **App.tsx**: `centerPaneTabs.openEvidenceCollection(id?, readOnly?)` or `centerPaneTabs.openEvidenceCollectionList()`
- **Collection list header**: `EvidenceCollectionListPanel.tsx` exposes `Import Package` for portable JSON package import and the export dropdown for PDF/XLSX/CSV/HTML/JSON export

### Evidence Collection Export Formats

- `EvidenceCollectionListPanel.tsx` export dropdown is the user-facing entry point for collection export.
- Supported formats are `pdf`, `xlsx`, `csv`, `html`, and `json`.
- The list-panel `json` path calls `project_db_export_evidence_collection_package`, which builds the canonical portable evidence collection package defined in `core-shared/crates/core-types/src/mobile.rs` as `MobileEvidenceCollectionPackage` from persisted `.ffxdb` rows plus current `.cffx` project metadata.
- Do NOT route list-panel `json` export back through `export_evidence_collection` in `src-tauri/src/report/commands.rs` — that path is form-driven and can drop desktop-only links such as `coc_item_id` and `evidence_file_id`.

### Evidence Collection Package Import

- `EvidenceCollectionListPanel.tsx` `Import Package` button is the user-facing entry point for portable collection-package import.
- The list-panel import path calls `project_db_import_evidence_collection_package`, which reads `MobileEvidenceCollectionPackage` JSON from disk and imports collections, collected items, and linked COC bundles transactionally into the current `.ffxdb`.
- Imported linked COC records receive fresh local IDs; collected-item `coc_item_id` links are remapped to those new local IDs during import.
- Imported `evidence_file_id` links are preserved only when the referenced evidence-file ID already exists in the current `.ffxdb`; otherwise the link is dropped and reported back to the frontend import summary.
- Do NOT reimplement package import as a frontend loop over form-only persistence when `project_db_import_evidence_collection_package` already exists — the backend command owns transactional remapping and FK-safe link preservation.

### Linked Data Tree — Right Panel

The linked data tree shows relationships between collected items, COC records, and evidence files. It renders in the **right panel** (alongside file info, EXIF, metadata) when a collection tab is active.

**Data flow:**
1. `EvidenceCollectionPanel` builds linked nodes from DB (FK relationships between `collected_items`, `coc_items`, evidence files)
2. Emits nodes via `onLinkedNodesChange` callback → `App.tsx` `linkedDataNodes` signal
3. `RightPanel` detects `activeTabType === "collection"` → renders `LinkedDataPanel`
4. `LinkedDataPanel` has two tabs: **Linked Data** (tree view) and **Summary** (counts by type)

**`LinkedDataNode` type** (in `LinkedDataTree.tsx`):
```tsx
interface LinkedDataNode {
  id: string;
  label: string;
  sublabel?: string;
  type: "collection" | "collected-item" | "coc" | "evidence-file";
  children?: LinkedDataNode[];
  linkedId?: string; // FK reference
}
```

### CenterPane Tab Integration

- `CenterTabType` includes `"collection"` — used for both form and list views
- `CenterTab` has: `collectionId?: string`, `collectionReadOnly?: boolean`, `collectionListView?: boolean`
- `useCenterPaneTabs` exposes: `openEvidenceCollection(id?, readOnly?)`, `openEvidenceCollectionList()`
- `ProjectTabType` includes `"collection"` and `"help"` — required for tab save/restore across project sessions
- `ProjectTab` has: `collection_id?: string`, `collection_read_only?: boolean`, `collection_list_view?: boolean` (snake_case for .cffx)
- `CenterTabForSave` has: `collectionId?: string`, `collectionReadOnly?: boolean`, `collectionListView?: boolean` (camelCase bridge)
- `projectSaveOptions.ts` serializes collection fields in `buildSaveOptions()` → `CenterTabForSave`
- `useProjectIO.ts` converts `CenterTabForSave.collectionId/ReadOnly/ListView` → `ProjectTab.collection_id/read_only/list_view`
- `projectLoader.ts` `restoreCenterTabs()` handles `case "collection":` to restore collection tabs on project load

### Collection List Filter Behavior

- `EvidenceCollectionListPanel` receives an **optional** `caseNumber` prop
- When `caseNumber` is `undefined`, `loadAllEvidenceCollections()` passes `null` to the backend → returns ALL collections in the project's `.ffxdb`
- Since `.ffxdb` is per-project, all collections in it belong to the current project — no additional filtering is needed
- **Do NOT** pass `projectManager.projectName()` as `caseNumber` to `EvidenceCollectionListPanel` — it filters by SQL `WHERE case_number = ?` which excludes collections with different/empty case numbers
- `EvidenceCollectionPanel` (the form) may still receive `caseNumber` as a default value for pre-filling the form — this is fine since it's used for initial form data, not for filtering

### Auto-Enrichment from Container Metadata

When an evidence collection panel loads AND container metadata is available (`discoveredFiles.length > 0` AND `fileInfoMap.size > 0`), a `createEffect` in `EvidenceCollectionPanel.tsx` **automatically enriches** the form with container data — no manual "From Evidence" button click required.

**Behavior by collection state:**

| State | Behavior |
|-------|----------|
| **New collection** (no saved items) | Calls `handleAutoFillFromEvidence()` → populates header fields + creates collected items for each evidence file |
| **Existing collection** (items loaded from DB) | Calls `enrichExistingItemsFromEvidence()` → fills **only empty fields** from container metadata, never overwrites user-entered data. Also enriches empty header fields. |
| **Read-only / locked** | Skipped entirely |

**Matching strategy** (for existing items → evidence files):
1. `evidence_file_id` FK — explicit file path match
2. Item `description` exactly equals filename (case-insensitive)
3. Item `description` contains the filename (e.g., `"PC-MUS-001.E01 - Hard Drive"` matches `PC-MUS-001.E01`)

**Enrichable fields** (16 total): `brand`, `make`, `model`, `serial_number`, `imei`, `other_identifiers`, `image_format`, `acquisition_method`, `connection_method`, `storage_notes`, `item_collection_datetime`, `item_system_datetime`, `item_collecting_officer`, `device_type`, `notes`, `building`

**Guard:** The `enriched` signal (initially `false`) prevents the effect from re-running. Set to `true` after the first enrichment pass, regardless of whether any fields were changed.

**Key functions:**
- `enrichExistingItemsFromEvidence(items, files, infoMap, caseNumber)` in `evidenceAutoFill.ts` — returns `EnrichmentResult { enrichedCount, fieldsFilled, updatedItems, changed }`
- `extractItemFieldsFromEvidence(file, info?, caseNumber?)` — extracts ~30 fields from a single container's metadata (examiner, stored hashes, format version, segment count, UFED OS/device name/duration, AD1 filesystem/OS/source path/companion hashes, companion log tool identity/duration)
- `extractHeaderFieldsFromEvidence(files, infoMap)` — extracts header-level fields (total items, organization)

### Do NOT

- Add evidence collection back into `ReportType` union or `REPORT_TYPES` array
- Import or use `WizardContext` inside `EvidenceCollectionPanel`
- Add evidence collection signals back to `WizardContext`
- Re-add `EvidenceCollectionFormSection.tsx` or `EvidenceCollectionSchemaSection.tsx` (deleted)
- Put the linked data tree back inside `EvidenceCollectionPanel` as an inline sidebar — it belongs in the right panel via `LinkedDataPanel`
- Use modal signals (`showEvidenceCollection`, `showEvidenceCollectionList`) for evidence collection — use `centerPaneTabs.openEvidenceCollection()` instead
- Pass `projectManager.projectName()` as `caseNumber` to `EvidenceCollectionListPanel` — this filters out collections and shows an empty list
- Remove collection fields (`collectionId`, `collectionReadOnly`, `collectionListView`) from `CenterTabForSave` or `projectSaveOptions.ts` — collection tabs won't persist across project saves
- Remove `case "collection":` from `restoreCenterTabs()` in `projectLoader.ts` — collection tabs won't restore on project load
- Remove device identification fields (brand, make, model, serial_number, imei, other_identifiers) from `evidence_collection.json` — they are auto-filled from E01/AD1/UFED container headers and must be visible/editable
- Remove forensic acquisition fields (image_format, acquisition_method, storage_notes) from `evidence_collection.json` — they are auto-filled from container metadata and must be visible/editable
- Remove per-item collection fields (item_collection_datetime, item_system_datetime, item_collecting_officer, item_authorization) from `evidence_collection.json` — they are auto-filled from container headers
- Remove photo_refs from `evidence_collection.json` — it maps to `DbCollectedItem.photoRefsJson` for photo documentation
- Remove the auto-enrichment `createEffect` from `EvidenceCollectionPanel.tsx` — it fills empty form fields from container metadata when evidence files are available
- Make container metadata auto-fill manual-only again (button-click required) — the `createEffect` ensures forms are always enriched when container info is available
- Remove `enrichExistingItemsFromEvidence()` or `ENRICHABLE_FIELDS` from `evidenceAutoFill.ts` — they power the silent enrichment of existing collections
- Reintroduce a repo-local evidence collection package schema in `cocExport.ts` or `src-tauri/src/report/commands.rs` — the JSON export must stay aligned with `core-types::mobile::MobileEvidenceCollectionPackage`

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

### useMenuActions

```tsx
import { useMenuActions } from "./hooks";

// Called in App.tsx — bridges native menu bar events to frontend handlers.
// Listens for "menu-action" events emitted by menu.rs → handle_menu_event().
// This is the only frontend listener for native menu events.
useMenuActions({
  onOpenProject: () => handleLoadProject(),
  onOpenDirectory: handleOpenDirectory,
  onSaveProject: handleSaveProject,
  onSaveProjectAs: handleSaveProjectAs,
  onToggleSidebar: () => setLeftCollapsed((prev) => !prev),
  onCommandPalette: () => setShowCommandPalette(true),
  onNewProject: () => setShowProjectWizard(true),
  onExport: () => centerPaneTabs.openExportTab(),
  onGenerateReport: () => setShowReportWizard(true),
  onScanEvidence: () => fileManager.scanForFiles(),
  // ... 30+ action handlers (see UseMenuActionsDeps interface)
});
```

**Project-dependent menu state:** Many menu items start disabled and are enabled when a project is loaded. App.tsx syncs this automatically:
```tsx
invoke("set_project_menu_state", { hasProject: true }).catch(() => {});  // create/load
invoke("set_project_menu_state", { hasProject: false }).catch(() => {}); // clear/close
```

### useFileManager

```tsx
const fileManager = useFileManager();

await fileManager.browseScanDir();              // Open directory picker + auto-scan
const files = fileManager.discoveredFiles();    // Get all discovered evidence files
const filtered = fileManager.filteredFiles();   // Get type-filtered files
fileManager.setTypeFilter("ad1");               // Filter by container type (null = all)
fileManager.toggleTypeFilter("ad1");            // Toggle type filter on/off
fileManager.setActiveFile(file);                // Select active file
await fileManager.selectAndViewFile(file);      // Set active + load info
await fileManager.scanForFiles(dir);            // Scan directory for evidence files
await fileManager.loadFileInfo(file);            // Load container info for one file
await fileManager.loadAllInfo();                 // Load full details for all files
fileManager.loadStoredHashesInBackground();      // Start background hash loading (fire-and-forget)
fileManager.toggleFileSelection(path);           // Toggle single file selection
fileManager.toggleSelectAll();                   // Toggle select/deselect all
fileManager.addDiscoveredFile(file);             // Add a single file (deduped)
fileManager.clearAll();                          // Reset all state
```

**Project setup scan performance:**
- `scanForFiles` uses streaming scan (`scan_directory_streaming`) with batched signal updates (80ms interval) to avoid O(n²) array copies
- `flushScanBuffer()` uses `dbSync.batchUpsertEvidenceFiles()` for a single IPC call + SQL transaction instead of N individual `upsertEvidenceFile` calls
- During `handleProjectSetupComplete`, `scanForFiles` is called with `skipHashLoading = true` to prevent `loadStoredHashesInBackground()` from saturating the Tauri thread pool while `saveProject` and `project_db_open` are still pending
- `loadStoredHashesInBackground()` is deferred to AFTER project setup completes (after save + DB open + toast), outside the `globalLoading.run("Setting up project…")` wrapper
- `containers::info_fast()` correctly routes Archive containers to `archive::info_fast()` (1 I/O op — format detection + file size) and UFED containers to `ufed::info_fast()` — NOT the slow `archive::info()` / `ufed::info()` which do segment discovery, central directory parsing, and UFED-in-ZIP detection (4+ I/O ops per file)

### useHashManager

```tsx
const hashManager = useHashManager();

hashManager.setSelectedHashAlgorithm("SHA-256"); // Set hash algorithm
await hashManager.hashSingleFile(...);            // Hash one file
await hashManager.hashSelectedFiles(...);         // Hash all selected files
await hashManager.hashAllFiles(...);              // Hash all discovered files
hashManager.clearAll();                           // Reset hash state
```

Note: Hash verification is handled by the backend (`e01_v3_verify`, `raw_verify`, etc.), not via the `useHashManager` hook.

**Batch hash architecture (`commands/hash.rs` → `useHashComputation.ts`):**
- Backend uses **storage-aware scheduling** with per-drive semaphores. Each file's path is resolved to its mount point via `sysinfo::Disks`, classified by `StorageClass` (7 variants), and assigned a per-drive concurrency limit:
  - **NVMe/PCIe SSD** (`NvmePcie`): 8 concurrent (NVMe drives handle deep I/O queues natively)
  - **Internal SSD** (`InternalSsd`): 4 concurrent (SATA SSDs benefit from moderate parallelism)
  - **RAID Array** (`RaidArray`): 6 concurrent (striped arrays distribute I/O across spindles)
  - **Internal HDD** (`InternalHdd`): 2 concurrent (seek-limited; concurrent reads cause thrashing)
  - **Removable (USB/Thunderbolt)** (`Removable`): 1 concurrent (bus-limited regardless of media type)
  - **Network Share** (`NetworkShare`): 3 concurrent (limited by LAN/WAN throughput, not local I/O)
  - **Unknown**: dynamic — `(CPU cores / 4).clamp(2, 8)` (e.g., 7 on 28-core, 2 on 4-core)
- **Storage detection heuristics** (`classify_storage()`):
  1. **UNC/network paths** (`\\server\share` or `//server/share`) → `NetworkShare`
  2. **macOS `/Volumes/` with network filesystem** (smbfs, nfs, afpfs, webdav, cifs) → `NetworkShare`
  3. **Linux filesystem type** (cifs, smb, nfs, nfs4, fuse.sshfs, 9p) → `NetworkShare` (via `/proc/mounts`)
  4. **NVMe device** (Linux `/dev/nvme*`, macOS `nvme` in disk name) → `NvmePcie`
  5. **RAID device** (Linux `/dev/md*`, macOS `AppleRAID`/`CoreStorage`) → `RaidArray`
  6. **Disk kind** (`SSD` → `InternalSsd`, `HDD` → `InternalHdd`)
  7. **Removable flag** → `Removable`
  8. **Unknown** (fallback)
- **Windows path matching**: `classify_storage()` uses **case-insensitive** comparison on Windows (`#[cfg(target_os = "windows")]` block lowercases both file path and mount point). Without this, `I:\path` won't match `i:\` from sysinfo.
- **Drive letter fallback**: When `sysinfo::Disks` returns 0 disks (observed on Windows 11 build 26200), `extract_drive_letter_mount()` extracts the drive letter (e.g., `I:\`) from the file path itself. This ensures files on the same drive still share a semaphore instead of all grouping under a single `"unknown"` mount with one shared semaphore.
- **Diagnostic logging**: `classify_storage()` logs sysinfo disk count at startup. Emits `warn!` when disk_count=0 explaining that drive-letter fallback heuristics are in use.
- **User-configurable concurrency** (`AppPreferences.hashConcurrency*`): Users can override the default concurrency for each storage class via Settings → Performance → Hash I/O Concurrency. Value `0` means "auto-detect" (use the default). Non-zero values override the detected default. Frontend builds a `concurrencyOverrides: Record<string, number>` map (keys: `nvme`, `ssd`, `raid`, `hdd`, `removable`, `network`) and passes it to `batch_hash`. Backend uses `resolve_concurrency()` helper: checks overrides map by `StorageClass::key()`, falls back to `StorageClass::default_concurrency()`.
- Files on **different drives** hash in parallel independently (separate semaphores per mount point). Files on the **same drive** share a semaphore with the drive-appropriate concurrency limit.
- Backend emits a `"batch-drive-info"` event with `BatchDriveInfo` payload (drives array with mount_point, storage_class, concurrency, file_count) before spawning hash tasks. Frontend listens for this to display drive detection results.
- `BatchHashResult` includes `drive_kind: Option<String>` so the frontend knows each file's storage classification.
- Each task runs a `spawn_blocking` closure that calls `spawn_progress_reporter()` helper + routes to the container-specific hash function via 3-arm routing (`is_ewf_type` → `ewf::verify_with_progress`, `is_ad1_type` → `ad1::hash_segments_with_progress`, everything else → `raw::verify_with_progress`)
- A shared `progress_cb` closure is defined once per task and passed by `&mut` reference to avoid duplicating the progress callback across routing branches
- `spawn_progress_reporter()` helper emits an **immediate 0% event** before the loop starts, then polls every **250ms** with 0.5% granularity and a **3-second heartbeat** (1-second heartbeat during startup while the file handle is opening)
- Frontend **buffers progress events** and flushes them to the UI at most every **200ms** via `pendingProgress` Map + `setTimeout` to avoid creating a new reactive Map per event when many files hash concurrently
- Frontend tracks terminal events (`"completed"` / `"error"`) per file; after `invoke` returns, any files missing terminal events are marked as errors (safety net)
- Frontend uses shared helpers `handleHashCompleted()` (verify + audit + persist) and `persistHashToDb()` (DB write) for both single-file and batch completion paths — no code duplication between the two modes
- `collectStoredHashes()` and `determineVerification()` in `hashUtils.ts` are the single source of truth for stored hash collection and verification logic
- `determineVerification()` must only set `verified` when a stored/container/companion hash exists for the same algorithm. If a newly computed hash only matches prior local history, it returns `verified: null`, `comparisonSource: "history"`, and `verifiedAgainst` for context — the UI renders this as a repeat match, not forensic verification.
- `FileHashInfo` and cached `computed_hashes` entries preserve `computedAt`, `verifiedAgainst`, and `comparisonSource` so the frontend can show computed dates, stored reference dates, and repeat-match state after reload.
- `useProjectIO` hash-history serialization must preserve both matches and mismatches by checking `entry.verified !== undefined` instead of truthiness; otherwise mismatch results disappear on save/load.
- `hashSelectedFiles` ensures all files have `evidence_files` records in `.ffxdb` (via `dbSync.upsertEvidenceFile`) **before** invoking `batch_hash` — this prevents `FOREIGN KEY constraint failed` errors when `persistHashToDb` inserts into the `hashes` table (which has FK `file_id → evidence_files(id)`)
- `hashSelectedFiles` does **NOT** fire parallel `loadFileInfo` calls — each `loadFileInfo` invokes `logical_info` which opens and parses the full container (E01 segment discovery, header parsing). Firing many of these in parallel on USB saturates Tauri's thread pool and I/O, blocking `batch_hash` from starting for minutes with zero UI feedback
- `restoreDiscoveredFiles` in `useFileManager` upserts all restored files to `.ffxdb` — the seed in `useProjectDbRead` only runs when `totalEvidenceFiles === 0`, so restored files would be missing from `.ffxdb` if even one file already existed
- Resource budget examples: at 2 concurrent on USB HDD: 32 file descriptors (2 × 16), ~128 MB buffers, 6 threads. At 6 concurrent on internal SSD: 96 FDs, ~384 MB buffers, 18 threads.

**Do NOT:**
- Use `?` (early return) on `spawn_blocking().await` in `batch_hash` — errors must emit `"batch-progress"` error events before returning
- Remove the frontend `terminatedFiles` safety net in `hashSelectedFiles` — it catches silent backend failures
- Remove the heartbeat in `spawn_progress_reporter()` — it proves the operation is alive during slow I/O or startup delays
- Change the progress thread to use integer percent (1% steps) — for large containers, 0.5% granularity prevents apparent stalls
- Remove the immediate 0% emit before the progress loop — it ensures the UI shows activity within milliseconds of task start
- Remove the `progressFlushTimer` cleanup in the `finally` block of `hashSelectedFiles` — dangling timers and unflushed progress will result
- Change the progress poll interval back to 500ms — 250ms provides noticeably smoother progress for fast containers
- Duplicate the stored hash collection or verification logic inline — use `collectStoredHashes()` and `determineVerification()` from `hashUtils.ts`
- Treat history-vs-history matches as `verified` in badges, row helpers, or detail panels — only stored/container/companion comparisons count as verification; history-only matches must remain `comparisonSource: "history"`
- Add new container type branches to `batch_hash` routing — use the 3-arm pattern (EWF / AD1 / raw fallback)
- Replace per-drive semaphores with a single global semaphore — this loses cross-drive parallelism (e.g., USB + internal SSD should hash independently)
- Change `StorageClass::Removable` concurrency above 2 — removable media (especially USB HDDs) thrash severely with high concurrency
- Remove the `"batch-drive-info"` event emission — the frontend uses it to display storage detection results
- Remove `drive_kind` from `BatchHashResult` — the frontend uses it to show which drive each file was hashed from
- Remove the `concurrency_overrides` parameter from `batch_hash` — it enables user-configurable per-class concurrency
- Remove the `hashConcurrency*` fields from `AppPreferences` — they drive the Settings → Performance → Hash I/O Concurrency sliders
- Remove `StorageClass::key()` method — it maps storage classes to override map keys (nvme, ssd, raid, hdd, removable, network)
- Collapse `NvmePcie` into `InternalSsd` or `RaidArray` into `InternalHdd` — they have genuinely different I/O characteristics and concurrency needs
- Remove the `#[cfg(target_os = "windows")]` case-insensitive path matching in `classify_storage()` — without it, Windows drive letters like `I:\` vs `i:\` won't match sysinfo mounts
- Remove `extract_drive_letter_mount()` — it's the critical fallback when `sysinfo::Disks` returns 0 disks (observed on Windows 11 build 26200); without it, all files share one semaphore
- Change `Unknown` concurrency back to a hardcoded constant — it must scale with CPU cores via `(cores / 4).clamp(2, 8)` to avoid starving high-core machines
- Re-add parallel `loadFileInfo` calls to `hashSelectedFiles` — they saturate the Tauri thread pool and USB I/O, blocking `batch_hash` from starting and creating a minutes-long UI dead zone with no feedback
- Remove the `dbSync.upsertEvidenceFile` calls from `hashSelectedFiles` — without them, `persistHashToDb` fails with FK constraint errors when evidence_file records are missing from `.ffxdb`
- Remove the `dbSync.upsertEvidenceFile` calls from `restoreDiscoveredFiles` — restored files won't be in `.ffxdb` if the DB already had some evidence files (seed skipped)
- Re-add auto-hash on file selection without a visible warning — when `autoVerifyHashes` is enabled, the first auto-hash in a session MUST show a toast warning that auto-hashing slows down evidence viewing
- Remove the `autoVerifyHashes` preference toggle from BehaviorTab.tsx — it is intentionally available but off by default, with a warning description
- Change `containers::info_fast()` Archive/UFED arms to call `archive::info()` or `ufed::info()` — these are the SLOW full-metadata versions; `info_fast` MUST use `archive::info_fast()` and `ufed::info_fast()` which skip segment discovery, ZIP parsing, and UFED detection
- Allow `loadStoredHashesInBackground()` to start DURING `handleProjectSetupComplete` — it saturates the Tauri thread pool with `logical_info_fast` calls, blocking `saveProject` and `project_db_open` from completing
- Replace `dbSync.batchUpsertEvidenceFiles()` in `flushScanBuffer` with individual `dbSync.upsertEvidenceFile()` calls — 400+ individual IPC calls flood the channel and create contention during project setup

### useProject

```tsx
const projectManager = useProject();

await projectManager.createProject(path, name);   // Create .cffx
await projectManager.loadProject(projectPath);    // Load existing
await projectManager.saveProject();               // Save current
const proj = projectManager.project();             // Get FFXProject | null
const name = projectManager.projectName();         // Get project name
const isOpen = projectManager.hasProject();        // Whether a project is loaded
const modified = projectManager.modified();        // Whether unsaved changes exist
const locs = projectManager.projectLocations();    // Get ProjectLocations
const caseNum = projectManager.caseNumber();       // Get case number (from FFXProject.case_number)
const caseNam = projectManager.caseName();         // Get case name (from FFXProject.case_name)
```

### useProjectDbSync (Write-Through to .ffxdb)

```tsx
import { dbSync } from "./hooks/project/useProjectDbSync";

// Fire-and-forget sync via syncInvoke() — errors logged but not awaitable.
// For forensic-critical saves (COC, evidence collections), use direct
// invoke() calls in cocDbSync.ts instead (awaitable).

// Bookmarks & Notes
dbSync.upsertBookmark(bookmark);           // Bookmark create/update
dbSync.deleteBookmark(bookmarkId);         // Bookmark delete
dbSync.upsertNote(note);                   // Note create/update
dbSync.deleteNote(noteId);                 // Note delete

// Activity & Sessions
dbSync.insertActivity(entry);             // Activity log entry
dbSync.upsertSession(session);            // Session create/update
dbSync.endSession(sessionId, summary?);   // End a session
dbSync.upsertUser(user);                  // User record upsert

// Tags
dbSync.upsertTag(tag);                     // Tag create/update
dbSync.deleteTag(tagId);                   // Tag delete
dbSync.assignTag(tagId, targetType, targetId, assignedBy);  // Tag assignment (4 params)
dbSync.removeTag(tagId, targetType, targetId);              // Remove tag assignment

// Evidence & Hashes
dbSync.upsertEvidenceFile(file);           // Evidence file upsert (single)
dbSync.batchUpsertEvidenceFiles(files);    // Batch upsert in single transaction (awaitable)
dbSync.insertHash(hash);                   // Hash record
dbSync.insertVerification(verification);   // Hash verification record

// Reports & Documents
dbSync.insertReport(report);              // Report record
dbSync.upsertCaseDocument(doc);           // Case document upsert

// Searches
dbSync.upsertSavedSearch(search);          // Saved search
dbSync.insertRecentSearch(query, count);   // Recent search

// UI State
dbSync.setUiState(key, value);             // UI state persistence

// Processed Databases
dbSync.upsertProcessedDatabase(db);        // Processed database record
dbSync.upsertAxiomCaseInfo(info);          // AXIOM case info

// COC Items (v5 immutability — fire-and-forget, use cocDbSync.ts for awaitable)
dbSync.insertCocItem(record);             // INSERT only (rejects duplicates)
dbSync.upsertCocItem(record);             // UPDATE draft items only
dbSync.lockCocItem(id, lockedBy);         // Lock a COC item
dbSync.deleteCocItem(id, voidedBy, reason); // Soft-delete (void)
dbSync.insertCocAuditEntry(entry);        // COC audit entry
dbSync.upsertCocTransfer(record);         // COC transfer upsert
dbSync.deleteCocTransfer(id);             // COC transfer delete

// Evidence Collections & Collected Items
dbSync.upsertEvidenceCollection(record);  // Evidence collection upsert
dbSync.deleteEvidenceCollection(id);      // Evidence collection delete
dbSync.upsertCollectedItem(record);       // Collected item upsert
dbSync.deleteCollectedItem(id);           // Collected item delete

// Form Submissions (schema v6)
dbSync.upsertFormSubmission(submission);  // Form submission upsert
dbSync.deleteFormSubmission(id);          // Form submission delete

// Export History
dbSync.insertExport(record);             // Export record create (on export start)
dbSync.updateExport(record);             // Export record update (on completion/failure)
```

> **WARNING:** `dbSync.*` methods are fire-and-forget (errors logged via `log.warn`, not surfaced to callers). For COC and evidence collection persistence where the caller needs confirmation that the save succeeded (e.g., before status transitions), use the awaitable functions in `cocDbSync.ts` instead: `persistCocItemsToDb()` and `persistEvidenceCollectionToDb()`.

### useProjectDbRead (Seed .ffxdb from .cffx)

```tsx
import { seedDatabaseFromProject } from "./hooks/project/useProjectDbRead";

// Called once during project load — seeds empty .ffxdb tables
// from the loaded .cffx project state (idempotent)
await seedDatabaseFromProject(project);
```

### useExaminerProfile (Examiner Auto-Fill)

```tsx
import {
  useExaminerProfile,
  loadExaminerProfile,
  saveExaminerProfile,
  examinerProfileToContext,
} from "./hooks/project";

// Reactive hook — manages profile signal + save
const examinerProfile = useExaminerProfile();
await examinerProfile.refresh();                   // Reload from DB after project load
examinerProfile.save({ name: "Jane", title: "Forensic Analyst" }); // Save partial update
const ctx = examinerProfile.autoFillContext();      // For useFormTemplate autoFillContext

// Standalone functions (no hook needed)
const profile = await loadExaminerProfile();        // Read from ui_state
saveExaminerProfile(profile);                       // Write to ui_state (fire-and-forget)
const ctx = examinerProfileToContext(profile);      // Convert to autoFillContext record
```

The examiner profile is stored in the `.ffxdb` `ui_state` table (key: `"examiner_profile"`, value: JSON) — no schema migration required. It auto-fills examiner fields in all schema-driven forms via the `autoFillContext` mechanism in `useFormTemplate`.

**Key file:** `src/hooks/project/useExaminerProfile.ts`

### User Profiles (App-Level Preferences)

User profiles are stored in **localStorage** (not per-project `.ffxdb`) because they represent the examiner/user identity, not project-specific data. They live alongside other `AppPreferences` in the `ffx-preferences` key.

**Key types (`src/components/preferences.ts`):**

```tsx
interface UserProfile {
  id: string;                    // e.g., "profile-1719842300000-abc123"
  name: string;                  // Full name
  title: string;                 // Job title
  organization: string;          // Organization/agency name
  badgeNumber: string;           // Badge/employee ID
  email: string;                 // Contact email
  phone: string;                 // Contact phone
  certifications: string[];      // Professional certifications
  agency: string;                // Agency name
  logoPath: string;              // Path to organization logo
  caseNumberPrefix: string;      // Default case number prefix
  defaultReportPreset?: string;  // Preferred report template
}
```

**AppPreferences fields:**
- `userProfiles: UserProfile[]` — array of all profiles (default: `[]`)
- `defaultUserProfileId: string` — ID of active profile (default: `""`)
- `confirmUserOnProjectOpen: boolean` — show confirm modal on project open (default: `true`)

**Helper functions:**
- `generateProfileId()` — returns `"profile-{timestamp}-{random}"`
- `createEmptyProfile(name?)` — returns blank `UserProfile` with generated ID
- `getActiveUserProfile()` — reads from localStorage, returns matching profile or `undefined`
- `applyProfileToPreferences(profile, updatePreference)` — syncs 11 examiner/branding fields from profile to flat preferences

**UI Components:**

| Component | Location | Purpose |
|-----------|----------|---------|
| `UserProfilesSettings` | `src/components/settings/UserProfilesTab.tsx` | Full CRUD tab in SettingsPanel (10th tab, "Users & Profiles") |
| `UserConfirmModal` | `src/components/project/UserConfirmModal.tsx` | Profile confirmation on project open/create |

**Entry points:**
- **Settings**: SettingsPanel → "Users & Profiles" tab (between Reports and Keyboard Shortcuts)
- **Project open/create**: Auto-shown via `createEffect` in App.tsx when `confirmUserOnProjectOpen` is true and profiles exist
- **Reports tab**: Profile indicator banner shows active profile link in Branding section

**Do NOT:**
- Store user profiles in the `.ffxdb` database — they are app-level (localStorage), not project-level
- Remove the `confirmUserOnProjectOpen` preference — it gates the modal shown on project open/create
- Confuse `UserProfile` (app-level, localStorage) with `ExaminerProfile` (per-project, `.ffxdb` `ui_state`)

### useFormTemplate Auto-Fill Context

```tsx
const form = useFormTemplate({
  templateId: "evidence_collection",
  autoFillContext: {
    examiner: { name: "Jane", title: "Forensic Analyst" },
    project: { case_number: "2024-001" },
  },
});
```

The `autoFillContext` option is a `Record<string, Record<string, FormValue>>` keyed by source name (matching `AutoFillSource.source` in template JSON). Field `auto_fill.path` last segment is used as the lookup key. Resolves in both `buildDefaults()` (initial form load) and `addRepeatableItem()` (new repeatable items).

### useLoadingState (Global Loading Indicator)

```tsx
import { useLoadingState } from "./hooks";
import { LoadingOverlay } from "./components/ui";

const globalLoading = useLoadingState();

// Wrap slow operations
await globalLoading.run("Loading project…", () => loadProject(path));
await globalLoading.run("Scanning for evidence…", () => scanForFiles());

// Manual control for operations managing their own lifecycle
globalLoading.setLoading(true, "Preparing…");
// ... do work ...
globalLoading.setLoading(false);

// Render the indicator
<LoadingOverlay
  isLoading={globalLoading.isLoading}
  message={globalLoading.message}
  error={globalLoading.error}
  position="bottom-right"
/>
```

The `LoadingOverlay` component shows a small toast-style indicator in a fixed position (bottom-right by default) with a spinner + message for loading state, or an error message that auto-dismisses after 5 seconds. In App.tsx, `globalLoading` wraps: `handleLoadProject`, `handleSaveProject`, `handleSaveProjectAs`, `handleProjectSetupComplete`, and `handleScanEvidence`.

**Key files:** `src/hooks/useLoadingState.ts`, `src/components/ui/LoadingOverlay.tsx`

### useProgressTracker (EMA-Smoothed Speed + ETA)

```tsx
import { createProgressTracker } from "./hooks/useProgressTracker";

const tracker = createProgressTracker();

// In your progress event handler:
tracker.update({ bytesProcessed: 5_000_000, bytesTotal: 10_000_000, percent: 50 });

// In your JSX — all values are reactive:
<span>{tracker.stats().speedFormatted}</span>   // e.g. "125.3 MB/s"
<span>{tracker.stats().etaFormatted}</span>      // e.g. "2m 15s"
<span>{tracker.stats().elapsedFormatted}</span>  // e.g. "1m 30s"

// On reset (new operation):
tracker.reset();
```

Uses **Exponential Moving Average** (α = 0.3) to smooth instantaneous speed samples taken at ≥500ms intervals. Speed and ETA are suppressed for the first 2 seconds (warm-up period) to avoid misleading initial values. A 1-second timer ticks internally to keep elapsed time reactive even when no progress events arrive.

**Key type:** `SmoothedStats { speedBps, etaMs, elapsedMs, speedFormatted, etaFormatted, elapsedFormatted }`

**Key file:** `src/hooks/useProgressTracker.ts`

**Do NOT:**
- Re-implement raw ETA calculation (bytesRemaining / instantSpeed) in acquisition views — use `createProgressTracker()` for stable ETA
- Change `ALPHA` without testing — 0.3 balances responsiveness vs. stability for forensic I/O patterns
- Remove the 2-second warm-up — premature speed values confuse users when I/O hasn't stabilized

### useAppLifecycle (Deferred Startup Orchestration)

```tsx
import { useAppLifecycle } from "./hooks/useAppLifecycle";

const lifecycle = useAppLifecycle({
  fileManager,
  projectManager,
  workspaceProfiles: { listProfiles, getActiveProfile },
  db: { restoreLastSession },
  tour: { hasCompleted },
  preferences: { preferences },
  getSaveOptions,
  setShowWelcomeModal,
});

// Returns: { windowWidth, isCompact }
```

Extracted from App.tsx to keep the root component focused on composition. Handles:
- System stats listener setup and resize tracking
- Workspace profile loading
- Auto-save callback registration
- Welcome modal first-run detection
- Last-session restoration (guarded by `!projectManager.hasProject()`)
- Cleanup (clipboard clear, auto-save)
- Preview-cache cleanup is backend-owned during deferred startup so frontend reload/unmount does not invoke `cleanup_preview_cache`
- Window title and close-confirmation wiring

The `isCompact()` signal is derived from `windowWidth() < 900` and is passed to layout components for responsive behavior.

**Key file:** `src/hooks/useAppLifecycle.ts`

**Do NOT:**
- Add a second `menu-action` listener in `useAppLifecycle` or elsewhere in App.tsx — `useMenuActions` is the single frontend bridge for native menu events

### FDA Advisory (macOS Full Disk Access)

On macOS, many forensic-relevant directories (Mail, Messages, Safari, etc.) are protected by TCC (Transparency, Consent, and Control). Without Full Disk Access, triage and acquisition operations silently skip these paths.

```tsx
import { checkFullDiskAccess, openFullDiskAccessSettings } from "../api/fda";

const status = await checkFullDiskAccess();
// status.hasFullDiskAccess: boolean
// status.blockedPaths: string[] — TCC-protected paths that were inaccessible
// status.message: string

if (!status.hasFullDiskAccess) {
  await openFullDiskAccessSettings(); // Opens System Settings → Privacy → FDA
}
```

The FDA check runs as a deferred startup check (after 500ms). If the app lacks FDA, a non-blocking advisory toast is shown. The check is only performed on macOS.

**Key files:** `src/api/fda.ts`, `src-tauri/src/commands/system.rs` (`check_full_disk_access`, `open_full_disk_access_settings`)

**Do NOT:**
- Make the FDA advisory blocking — it's informational, not a hard requirement
- Run the FDA check on non-macOS platforms — TCC is macOS-only

### WAL Checkpoint Lifecycle

`.ffxdb` databases use WAL (Write-Ahead Logging) mode for concurrent read performance. Without periodic checkpoints, data accumulates in the `.ffxdb-wal` file and the main `.ffxdb` may remain nearly empty. This causes problems when:
- The project is copied/moved without the WAL file
- External volumes are ejected before the WAL is flushed
- Other tools (e.g., merge analyze) open the DB read-only and can't see WAL data

**Checkpoint triggers (automatic):**

| Trigger | Where | Mechanism |
|---------|-------|-----------|
| **Project DB open** | `ProjectDatabase::open()` | Sets `wal_autocheckpoint=256` pages, `journal_size_limit=8 MiB`, and truncates an existing WAL larger than 32 KB |
| **Active session** | `useAppLifecycle()` | Runs `project_db_wal_checkpoint` with `mode="passive"` every 60 seconds while a project is open |
| **Project save** | `useProjectIO.saveProject()` | Calls `project_db_wal_checkpoint` after successful `.cffx` save (fire-and-forget) |
| **Project close** | `project_db_close` (Rust backend) | `PRAGMA wal_checkpoint(TRUNCATE)` before dropping the connection |
| **SQLite auto** | SQLite internal | Auto-checkpoints every 256 pages (about 1 MiB at 4 KB pages) unless active readers pin the WAL |

**Manual checkpoint:** `invoke("project_db_wal_checkpoint")` returns `(log_size, frames_checkpointed)`. Pass `mode: "passive"` for a non-blocking active-session checkpoint, or omit it for the default `TRUNCATE` checkpoint.

A non-empty `.ffxdb-wal` during active use is still normal. Long-lived readers, another open window, or an external SQLite tool can pin WAL frames and prevent `TRUNCATE` from fully resetting the file even when checkpoints run. The Rust checkpoint helper logs this blocked state when the checkpoint reports `busy > 0` or leaves pages uncheckpointed.

`project_db_get_stats` now includes current WAL size and the last checkpoint result so the dashboard can surface whether the WAL is clear, pending, or blocked by active readers.

**Do NOT:**
- Remove the checkpoint from `project_db_close` — external volumes need WAL flushed before eject
- Remove the periodic passive checkpoint from `useAppLifecycle()` — long-lived sessions should not rely only on save/close to fold WAL frames
- Make the save-time checkpoint blocking (`await`) — it's fire-and-forget to avoid slowing saves
- Remove `project_db_wal_checkpoint` from `lib.rs` registration — it's called by the frontend
- Remove `wal_autocheckpoint` or `journal_size_limit` from `ProjectDatabase::open()` — they are the active-use guardrails that keep WAL growth bounded when readers are not pinning frames
- Treat a non-empty `.ffxdb-wal` during active use as a bug by itself — check for pinned readers or another open process before assuming the checkpoint path is broken

### Multi-Window Project Database Isolation

Each Tauri window can have its own project open independently. The project database (`PROJECT_DBS`) is a per-window `HashMap<String, ProjectDatabase>` keyed by window label, not a global singleton.

**Architecture:**

```text
Window "main"         → PROJECT_DBS["main"]         → /path/to/case-A.ffxdb
Window "main-17198…"  → PROJECT_DBS["main-17198…"]  → /path/to/case-B.ffxdb
Window "main-17199…"  → (no entry — no project open in this window)
```

**How it works:**
1. `project_db_open(window, cffx_path)` inserts the database into `PROJECT_DBS` keyed by `window.label()`
2. `project_db_close(window)` removes the database for that window (after WAL checkpoint)
3. ALL 118 project_db commands receive `window: tauri::Window` as a parameter (auto-injected by Tauri — **zero frontend changes needed**)
4. Each command calls `with_project_db(window.label(), |db| ...)` to resolve the correct database
5. Frontend `invoke()` calls are unchanged — Tauri automatically injects the calling window

**Global singletons that are NOT per-window (and why):**

| Singleton | Location | Why Global is OK |
|-----------|----------|-----------------|
| `DB` | `database.rs` | App-level (recent projects, settings) |
| `SYSTEM` | `system.rs` | Shared sysinfo resource |
| `ORIGINAL_MOUNT_STATE` | `system.rs` | Drive mount state (system-wide) |
| `SESSION_CACHE` / `FILE_CACHE` / `INDEX_CACHE` | Various | Keyed by file path, not window |
| `NESTED_CONTAINER_CACHE` | `archive/nested.rs` | Keyed by file path |
| `*_CANCEL_FLAGS` | Various | Keyed by operation ID |
| `METRICS_REGISTRY` | `common/metrics.rs` | App-level metrics |

**Key files:**

| File | Purpose |
|------|---------|
| `src-tauri/src/commands/project_db/mod.rs` | `PROJECT_DBS` storage, `with_project_db(label, f)` helper, lifecycle commands |
| `src-tauri/src/menu.rs` | `new_window` command creates windows with `main-{timestamp}` labels; `create_new_window_from_app()` for dock reopen |
| `src-tauri/src/lib.rs` | `on_window_event(Destroyed)` → `cleanup_window_project_db()` safety net; `.build().run()` for macOS keep-alive |

**Window lifecycle cleanup:**
- Frontend close: `clearProject({ onProgress })` flushes activity, awaits `dbSync.flushPendingWrites()`, checkpoints WAL, then `invoke("project_db_close")` (normal path)
- Backend safety net: `on_window_event(WindowEvent::Destroyed)` → `cleanup_window_project_db(label)` in `lib.rs` — checkpoints WAL and removes the DB entry if the frontend didn't close it (force-quit, crash)

### Project Close Workflow

Project teardown is an explicit async workflow coordinated by `useProjectIO.clearProject()` and surfaced in `App.tsx` through `ProjectCloseModal`.

**Close sequence:**
1. Persist `.cffx` state when needed before teardown begins
2. Finalize session/activity state via `useActivityLog.flushActivity()`
3. Drain pending fire-and-forget `.ffxdb` writes via `dbSync.flushPendingWrites()`
4. Run `project_db_wal_checkpoint` so eligible WAL frames fold into the main DB
5. Call `project_db_close` and clear project/UI state

**UI/entry points:**
- `App.tsx` owns `closeCurrentProject(reason)` and renders `src/components/project/ProjectCloseModal.tsx`
- File menu and command palette both expose **Close Project** and route to the same workflow
- Project switching (`loadProject`) and replacement (`createProject`) reuse the same close path instead of tearing down state ad hoc

**Key files:**
- `src/hooks/project/useProjectIO.ts` — ordered `clearProject()` workflow
- `src/hooks/project/useProjectDbSync.ts` — pending invoke tracking + `flushPendingWrites()`
- `src/hooks/project/useActivityLog.ts` — `flushActivity()` before close
- `src/components/project/ProjectCloseModal.tsx` — step-by-step modal UI
- `src/App.tsx` — close orchestration + modal state

**Do NOT:**
- Change `PROJECT_DBS` back to a global singleton (`OnceLock<Mutex<Option<…>>>`) — multiple windows need independent databases
- Call `project_db_close` directly from random UI code — route close through `clearProject()`
- Start a new/open project flow over an active project without awaiting the close workflow
- Remove `flushPendingWrites()` from the close path — queued `.ffxdb` sync calls would be lost during teardown
- Remove `window: tauri::Window` from any project_db command — the window label is required for DB lookup
- Add `window: tauri::Window` to non-project_db commands unless they also need per-window state
- Use a fixed window label like `"main"` — dynamically created windows get unique labels like `main-{timestamp}`
- Make `with_project_db` public outside the `project_db` module — it's `pub(super)` for internal use only

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

## Native Menu Bar (`menu.rs`)

The native menu bar is built in `src-tauri/src/menu.rs` and registered via `.menu(|app| menu::build_menu(app))` in `lib.rs`. Event handling is wired via `.on_menu_event(|app, event| menu::handle_menu_event(app, event))`.

**Submenus (6 on macOS, 5 on Windows/Linux):**

| Submenu | Key Items | Platform |
|---------|-----------|----------|
| **CORE-FFX** (app) | About, Hide, Quit | macOS only |
| **File** | New Project, New Window, Open Project, Open Directory, Save, Save As, Close Project, Acquire & Export, Scan Evidence, Close Tab/All, Toggle Auto-Save | All |
| **Edit** | Undo, Redo, Cut, Copy, Paste, Select All, Select All Evidence | All |
| **View** | Toggle Sidebar, Toggle Right Panel, Toggle Quick Actions, Dashboard, Evidence, Case Docs, Processed DBs, Activity, Bookmarks, Info/Hex/Text Views, Cycle Theme, Fullscreen | All |
| **Tools** | Generate Report, Evidence Collection, Search, Hash (All/Selected/Active), Deduplication, Load All Info, Clean Cache, Merge Projects, Import Acquisitions, Settings, Performance | All |
| **Window** | Minimize, Maximize, Close | All |
| **Help** | User Guide, Welcome Screen, Start Tour, Keyboard Shortcuts, Command Palette, Check for Updates, About (non-macOS) | All |

**Event flow:** `menu.rs` → `handle_menu_event()` → `emit_to_focused_window(app, "menu-action")` → frontend `useMenuActions` hook dispatches to handlers.

**Project-dependent items:** `set_project_menu_state` Tauri command enables/disables ~28 menu items based on whether a project is loaded. It is invoked from the actual project transition points in `useProjectIO.ts` (`createProject`, `loadProject`, `clearProject`) rather than from a reactive App.tsx effect.

**Multi-window:** `new_window` command creates additional windows with `WebviewWindowBuilder`. `get_window_labels` lists open windows. `create_new_window_from_app()` is a public function used by the `lib.rs` Reopen handler to create windows from `&AppHandle` (unlike `new_window` which takes `tauri::Window`).

**macOS keep-alive:** `lib.rs` uses `.build().run()` (not `.run()`) to install a `RunEvent` handler:
- `ExitRequested` → `api.prevent_exit()` keeps the app running when all windows are closed (standard macOS behavior)
- `Reopen` (macOS only, `#[cfg(target_os = "macos")]`) → creates a fresh window via `menu::create_new_window_from_app()` when the dock icon is clicked with no visible windows
- The `_app_handle` parameter is prefixed with underscore because it's only used in the macOS-specific `Reopen` arm — without this, Linux/Windows builds fail with unused-variable warnings under `-D warnings`

**Key files:**

| File | Purpose |
|------|---------|
| `src-tauri/src/menu.rs` | `build_menu()`, `handle_menu_event()`, `set_project_menu_state`, `new_window`, `get_window_labels`, `create_new_window_from_app()` |
| `src-tauri/src/lib.rs` | `.menu()` + `.on_menu_event()` + `.build().run()` (macOS keep-alive: `ExitRequested` + `Reopen`) |
| `src/hooks/useMenuActions.ts` | Frontend listener — `UseMenuActionsDeps` interface (30+ handlers) |
| `src/App.tsx` | Wires `useMenuActions` with concrete handlers, syncs project menu state |

**Do NOT:**
- Build menus inline in `lib.rs` `.setup()` — the menu is built by `menu::build_menu()` via `.menu()`. Any `app.set_menu()` call in `.setup()` will **override** the real menu.
- Add menu items without a matching entry in `handle_menu_event()` — unmatched IDs are silently ignored
- Add handlers to `useMenuActions` without adding the action string to the `switch` block
- Forget to add project-dependent item IDs to `PROJECT_DEPENDENT_IDS` in `menu.rs`
- Use `app_handle` (without underscore) in the `.run()` callback — it must be `_app_handle` because it's only used in a macOS-only `#[cfg]` block; using it without underscore causes unused-variable errors on Linux/Windows CI

---

## Auto-Updater (Check for Updates)

CORE-FFX uses `tauri-plugin-updater` (Rust) + `@tauri-apps/plugin-updater` (JS) + `tauri-plugin-process` (for `relaunch()`) to check for, download, and install updates from GitHub Releases.

### Architecture

```text
Help → "Check for Updates…"
  → menu.rs emits "check-updates" event
  → useMenuActions dispatches onCheckForUpdates
  → App.tsx sets showUpdateModal(true)
  → src/components/UpdateModal.tsx (lazy-loaded wrapper)
    └── @core-suite/components/updater UpdateModal
        └── useUpdater() from core-shared updater package
            ├── check() from @tauri-apps/plugin-updater → hits latest.json endpoint
            ├── update.downloadAndInstall() → downloads + installs update
            └── relaunch() from @tauri-apps/plugin-process → restarts app
```

### Key Files

| File | Purpose |
|------|---------|
| `src/components/UpdateModal.tsx` | Thin CORE-FFX wrapper that passes updater config into the shared modal |
| `../core-shared/packages/components/src/updater/UpdateModal.tsx` | Shared modal UI: checking → available → downloading → ready states |
| `../core-shared/packages/components/src/updater/useUpdater.ts` | Shared updater lifecycle hook (check, download, relaunch, auth headers) |
| `src/components/help/sections/About.tsx` | About panel version info; uses a timeout-guarded updater check so stalled requests degrade to `unable to check` |
| `src-tauri/tauri.conf.json` | `plugins.updater` config: endpoint URL + Ed25519 public key (CORE-FFX) |
| `src-tauri/capabilities/default.json` | `updater:default` + `process:default` permissions |
| `src-tauri/src/lib.rs` | Plugin registration: `tauri_plugin_updater`, `tauri_plugin_process` |
| `src-tauri/src/menu.rs` | "Check for Updates…" menu item (`check-updates` ID) |
| `.github/workflows/release.yml` | Signs artifacts + generates `latest.json` manifest for CORE-FFX |

### Configuration

- **FFX Endpoint:** `https://github.com/tmreyno/CORE/releases/latest/download/latest.json`
- **Signing keys:** Ed25519 keypair at `~/.tauri/core-ffx.key` (private) and `.pub` (public)
- **GitHub Secrets required:** `TAURI_SIGNING_PRIVATE_KEY` (contents of `~/.tauri/core-ffx.key`), optional `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- **`createUpdaterArtifacts: true`** (boolean, NOT `"v1Compatible"`) — Tauri v2 native updater format

### Tauri v2 Updater Artifacts

With `createUpdaterArtifacts: true` (boolean), Tauri v2 produces these updater artifacts per platform:

| Platform | Updater Bundle | Signature | Notes |
|----------|---------------|-----------|-------|
| **macOS** | `CORE-FFX.app.tar.gz` | `CORE-FFX.app.tar.gz.sig` | .app directory is tarred for transport |
| **Windows** | `CORE-FFX_<ver>_x64-setup.exe` | `CORE-FFX_<ver>_x64-setup.exe.sig` | Raw NSIS installer (NOT `.nsis.zip`) |
| **Linux** | `CORE-FFX_<ver>_amd64.AppImage` | `CORE-FFX_<ver>_amd64.AppImage.sig` | Raw AppImage (NOT `.AppImage.tar.gz`) |

The `"v1Compatible"` string value would produce `.nsis.zip` and `.AppImage.tar.gz` wrapped bundles — we do NOT use that mode. The `publish-release` job's download patterns and manifest globs must match the v2 format above.

### Manifest Generation (`latest.json`)

The `publish-release` job generates the CORE-FFX updater manifest only. CORE-Acquisition updater artifacts are owned by the standalone CORE-ACQ repo.

1. **Determine version** — extracts tag/version
2. **Checkout code** — sparse checkout for `CHANGELOG.md` (release notes)
3. **Download artifacts** — `gh release download` fetches updater bundles + sigs into `artifacts/`
4. **Generate manifest** — `generate_manifest "CORE-FFX" "latest.json"`

The manifest includes `darwin-aarch64`, `darwin-x86_64` (both use the same universal macOS bundle), `windows-x86_64`, and `linux-x86_64` platform entries. The function filters artifacts by the product name prefix to find the correct `.sig` and updater bundle files.

### Private Repo Auth

While the repo is private, GitHub returns 404 for unauthenticated release asset downloads. The updater uses a build-time GitHub PAT to authenticate:

1. **Secret:** `GH_UPDATE_TOKEN` — PAT with `contents:read` on the CORE repo (GitHub disallows `GITHUB_` prefix for secrets)
2. **Build-time injection:** Release workflow sets `VITE_GITHUB_UPDATE_TOKEN=${{ secrets.GH_UPDATE_TOKEN }}` on all 3 platform build steps
3. **Vite define:** `vite.config.ts` exposes it as `__GITHUB_UPDATE_TOKEN__`
4. **Runtime:** `src/components/UpdateModal.tsx` passes `authToken` into the shared updater config, and `useUpdater()` in `../core-shared/packages/components/src/updater/useUpdater.ts` builds `{ Authorization: "token <PAT>" }` headers and passes them to BOTH `check({ headers })` AND `downloadAndInstall(onEvent, { headers })`. The same headers MUST be passed to both calls — `check()` uses them for the manifest fetch, and `downloadAndInstall()` uses them for the binary download. Without headers on the download, GitHub returns 404/HTML for private repo assets and the signature verification fails against garbage data.
5. **Graceful fallback:** If the token is empty (repo made public, secret not set), the updater works without auth
6. **Release workflow handling:** Optional release secrets such as `GH_UPDATE_TOKEN`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, and `APPLE_API_*` must only be masked or exported when they are non-empty. The workflow must treat whitespace-only placeholders as blank, and it must only export `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_CONTENT`, and `APPLE_API_KEY_PATH` when the full App Store Connect credential set is present.

`vite.config.ts` must use `loadEnv(mode, process.cwd(), "")` when defining `__GITHUB_UPDATE_TOKEN__`. Reading only `process.env.VITE_GITHUB_UPDATE_TOKEN` does not load repo-local `.env` files during config evaluation, so local builds silently embed an empty updater token and all private-release update checks fail.

The shared updater modal does not auto-check while hidden. It starts a timeout-guarded check when the modal opens so a stalled `@tauri-apps/plugin-updater` request cannot leave the UI in `checking` forever. The About panel's direct version check must follow the same timeout-guarded pattern.

### Do NOT

- Remove `tauri-plugin-process` — required for `relaunch()` after update install
- Set `TAURI_SIGNING_PRIVATE_KEY` to empty string in production — updates won't be signed and will fail verification
- Add `check-updates` to `PROJECT_DEPENDENT_IDS` — checking for updates should work without a project loaded
- Add `merge-projects` to `PROJECT_DEPENDENT_IDS` — merging projects should work without a project loaded
- Remove `VITE_GITHUB_UPDATE_TOKEN` from the release workflow build steps — private repo updates will break
- Expose the `GH_UPDATE_TOKEN` PAT in logs or committed config files — use build-time env var injection only
- Move the "Download release artifacts" step before "Checkout code for changelog" in `publish-release` — the checkout wipes the working directory and destroys downloaded artifacts
- Use v1-compatible patterns (`*.nsis.zip`, `*.AppImage.tar.gz`) in download or manifest steps — Tauri v2 produces `*-setup.exe` and `*.AppImage` directly
- Change `createUpdaterArtifacts` from `true` to `"v1Compatible"` without updating the manifest generation globs in `release.yml`
- Inline the updater implementation back into `src/components/UpdateModal.tsx` — keep CORE-FFX as a thin config wrapper over the shared updater package
- Remove the `{ headers }` option from `downloadAndInstall()` in `../core-shared/packages/components/src/updater/useUpdater.ts` — without auth headers on the binary download, private repo updates fail with "signature verification failed"
- Pass auth headers to `check()` only — BOTH `check()` and `downloadAndInstall()` need them for private repos
- Re-enable hidden-modal auto-checks or remove the updater timeout guard — a stalled private-repo/auth request can otherwise pin the modal spinner until the app restarts

---

## Backend Command Groups

Commands are organized in `src-tauri/src/commands/`:

| Module | Purpose | Example Commands |
|--------|---------|------------------|
| `container.rs` | AD1/container operations | `logical_info`, `logical_info_fast`, `container_get_root_children_v2`, `container_get_children_at_addr_v2`, `container_extract_entry_to_temp` |
| `archive/` | Archive browsing & extraction | Archive `metadata.rs`, `extraction.rs`, `nested.rs`, `tools.rs` |
| `archive_create/` | Archive creation (modular: `mod.rs`, `manifest.rs`) | `create_7z_archive`, `estimate_archive_size`, `cancel_archive_creation` |
| `companion.rs` | Acquisition companion files | `write_companion_file`, `read_companion_file`, `find_companion_file`, `scan_for_acquisitions` |
| `ewf.rs` | E01/EWF operations | `e01_v3_verify` |
| `ewf_export.rs` | EWF image creation (via libewf-ffi) | `ewf_create_image`, `ewf_estimate_size`, `ewf_cancel_create` |
| `ewf_read.rs` | EWF image info reading (via libewf-ffi) | `ewf_read_image_info` |
| `ewf_helpers.rs` | Shared EWF export helpers | — (internal: progress, cancel flags) |
| `ewf_export_types.rs` | EWF export type definitions | — (internal: shared structs) |
| `aff4_export.rs` | AFF4 forensic container creation (pure-Rust ffx-aff4) | `aff4_create_image`, `aff4_cancel_export` |
| `raw_export.rs` | Raw disk imaging (.dd/.img) with segmentation & hashing | `raw_create_image`, `raw_cancel_export` |
| `l01_export.rs` | L01 logical evidence creation (pure-Rust) | `l01_create_image`, `l01_estimate_size`, `l01_cancel_export` |
| `hash.rs` | Batch hashing & queue | `batch_hash`, `hash_queue_pause`, `hash_queue_resume`, `hash_queue_clear_completed` |
| `viewer.rs` | File viewing | `viewer_read_chunk`, `viewer_detect_type`, `viewer_parse_header`, `viewer_read_text` |
| `analysis.rs` | File byte reading | `read_file_bytes` |
| `database.rs` | SQLite ops (15 commands) | `db_get_or_create_session`, `db_upsert_file`, `db_insert_hash`, `db_get_hashes_for_file` |
| `project.rs` | .cffx project files | `project_save`, `project_load`, `project_create`, `project_check_exists` |
| `project_advanced.rs` | Backup/versioning/recovery | `project_create_backup`, `project_create_version`, `project_check_recovery`, `project_recover_autosave` |
| `project_merge.rs` | Project merge/combine | `project_merge_analyze`, `project_merge_execute` |
| `project_extended.rs` | Workspace profiles | `profile_list`, `profile_get`, `profile_set_active`, `profile_add`, `profile_update`, `profile_delete` |
| `discovery.rs` | File/directory scanning | `path_exists`, `discover_evidence_files`, `scan_directory_streaming`, `find_case_documents` |
| `export.rs` | File export | `export_files`, `cancel_export` |
| `lazy_loading.rs` | Lazy tree loading | `lazy_get_container_summary`, `lazy_get_root_children`, `lazy_get_children`, `lazy_get_settings` |
| `raw.rs` | Raw image verification | `raw_verify` |
| `system.rs` | System stats, drives & mount control (10s TTL physical disk cache) | `get_system_stats`, `cleanup_preview_cache`, `write_text_file`, `read_text_file`, `get_audit_log_path`, `list_drives`, `remount_read_only`, `restore_mount`, `get_current_username`, `get_app_version`, `check_path_writable` |
| `device.rs` | Raw device access, privilege detection, physical disk ops (debug-level logging) | `check_privilege`, `get_device_size`, `list_physical_disks`, `request_elevation`, `read_raw_device` |
| `memory_capture.rs` | Live RAM capture (Linux `/proc/kcore`, Windows WinPmem, macOS unsupported) | `memory_capture_info`, `memory_capture`, `memory_capture_cancel` |
| `triage.rs` | Forensic triage collection + credential/secret scanning | `triage_get_profiles`, `triage_collect`, `triage_cancel` |
| `vfs.rs` | Virtual filesystem (with handle pool: max 32 cached VFS handles, LRU eviction, per-handle dir/attr caches) | `vfs_mount_image`, `vfs_list_dir`, `vfs_read_file`, `vfs_close_container` |
| `ufed.rs` | UFED container operations | `ufed_info`, `ufed_info_fast`, `ufed_verify`, `ufed_get_stats`, `ufed_extract` |
| `search.rs` | Tantivy full-text search | `search_open_index`, `search_close_index`, `search_delete_index`, `search_get_stats`, `search_index_container`, `search_index_all`, `search_rebuild_index`, `search_query` |
| `dedup.rs` | File deduplication analysis | `dedup_analyze`, `dedup_enrich_hashes`, `dedup_export_csv` |
| `segment_verify.rs` | Post-acquisition segment verification | `hash_container_segments` |
| `portable.rs` | Portable mode detection & path management | `portable_get_status`, `portable_ensure_dirs` |
| `project_db/` | Per-window .ffxdb (120+ cmds) — modular directory with `mod.rs`, `activity.rs`, `bookmarks.rs`, `collections.rs`, `evidence.rs`, `forensic.rs`, `processed.rs`, `search.rs`, `utilities.rs`, `workflow.rs`. **All commands receive `window: tauri::Window` (auto-injected by Tauri)** to resolve the per-window database. | `project_db_open`, `project_db_close` (checkpoints WAL), `project_db_wal_checkpoint`, `project_db_get_stats`, `project_db_upsert_bookmark`, `project_db_batch_upsert_evidence_files`, `project_db_export_evidence_collection_package`, `project_db_import_evidence_collection_package`, `project_db_search_fts`, `project_db_get_activity_log` |

**Processed database parsers** (`src-tauri/src/processed/`):

| Module | Purpose | Tauri Commands |
|--------|---------|----------------|
| `detection.rs` | Auto-detect processed DB tool type | — (internal API) |
| `axiom.rs` | Magnet AXIOM case parser | — (via processed commands) |
| `cellebrite.rs` | Cellebrite Physical Analyzer parser | `get_cellebrite_case_info`, `get_cellebrite_artifact_categories` |
| `autopsy.rs` | Autopsy case parser (.aut + autopsy.db) | `get_autopsy_case_info`, `get_autopsy_artifact_categories` |
| `commands.rs` | Tauri command wrappers | All processed DB commands |

---

## Tantivy Full-Text Search Engine

CORE-FFX includes a **Tantivy-powered full-text search engine** that indexes filenames, paths, metadata, and optionally file content across all evidence containers. It replaces the previous broken `search_all_containers` approach and supplements the existing FTS5 search (which remains for notes/bookmarks/activity_log).

### Architecture

```text
Frontend (SearchFilters.tsx / useAppActions.ts)
  → src/api/search.ts (typed invoke wrappers + event listeners)
  → src-tauri/src/commands/search.rs (8 Tauri commands)
  → src-tauri/src/search/ (core engine)
      ├── mod.rs      — Schema (12 fields), SearchIndex lifecycle, global registry
      ├── indexer.rs  — Container crawlers + content extractors (PDF, DOCX, EML, plist)
      └── query.rs    — BM25 search with filters, snippets, facet counts
```

### Index Schema (12 fields)

| Field | Type | Options | Purpose |
|-------|------|---------|--------|
| `doc_id` | STRING | STORED | Unique ID: `container_path:entry_path` |
| `container_path` | STRING | STORED | Source container file path |
| `container_type` | STRING | STORED | `ad1`, `e01`, `l01`, `archive`, `raw`, `disk` |
| `entry_path` | TEXT | STORED | Full path within container |
| `filename` | TEXT | STORED, tokenized | Filename (boosted 3x in queries) |
| `extension` | STRING | STORED | Lowercase extension |
| `content` | TEXT | Positions | Extracted text content |
| `size` | u64 | INDEXED, STORED, FAST | File size in bytes |
| `modified` | i64 | INDEXED, STORED, FAST | Last modified timestamp (unix) |
| `is_dir` | u64 | INDEXED, STORED | 0=file, 1=directory |
| `file_category` | STRING | STORED | `document`, `email`, `code`, `image`, etc. |

### Index Lifecycle (Per-Window)

- `SEARCH_INDEXES`: Global `HashMap<String, Arc<SearchIndex>>` keyed by window label
- Index stored at `<project>.ffxdb-index/` alongside the `.ffxdb` file
- Opened on project load, closed on project close (via `search_open_index`/`search_close_index`)
- Auto-indexed on project load via `useSearchIndex` hook in App.tsx

### Container Crawlers (indexer.rs)

| Container | Crawler | Method |
|-----------|---------|--------|
| AD1 | `crawl_ad1` | `ad1::get_root_children_v2` + recursive `get_children_at_addr_v2` |
| L01 | `crawl_l01` | `ewf::parse_l01_file_tree` → iterate entries |
| Archive | `crawl_archive` | `archive::libarchive_list_all` |
| E01/EWF | `crawl_vfs_ewf` | `EwfVfs::open` + recursive `readdir`/`getattr` |
| Raw | `crawl_vfs_raw` | `RawVfs::open_filesystem` + recursive `readdir`/`getattr` |
| Disk | `crawl_disk_files` | `std::fs` recursive walk |

### Content Extraction

Text extraction is optional (controlled by `index_content` flag). Extractors:
- **Plain text** (txt, log, md, json, xml, code files, etc.): UTF-8 lossy
- **PDF**: `pdf_extract::extract_text_from_mem`
- **DOCX**: ZIP → `word/document.xml` → `<w:t>` tag extraction
- **EML/MBOX**: `mail_parser` → subject + from + body
- **Plist**: `plist::from_bytes` → debug format
- **RTF**: Simple control word stripping
- Max content: 256 KB per file. Files > 10 MB skipped.

### Query Engine (query.rs)

- **BM25 scoring** with field boosting: filename (3x), entry_path (1.5x), content (1x)
- **Fuzzy search**: 1 edit distance on filename field
- **Filters**: container type, extension, category, size range, specific container, include/exclude directories
- **Snippets**: Content and filename snippet generators with `<b>` highlighting
- **Facet counts**: Category and container type aggregation from results

### Search Options

```typescript
interface SearchOptions {
  query: string;            // User search string
  limit?: number;           // Max results (default 100)
  containerTypes?: string[];// Filter by container type
  extensions?: string[];    // Filter by extension
  categories?: string[];    // Filter by file category
  minSize?: number;         // Min file size (bytes)
  maxSize?: number;         // Max file size (bytes)
  includeDirs?: boolean;    // Include directories
  searchContent?: boolean;  // Search file content (default true)
  containerPath?: string;   // Filter to specific container
}
```

### Frontend Integration

- **`src/api/search.ts`**: 8 typed invoke wrappers + `listenIndexProgress` event listener
- **`src/hooks/useSearchIndex.ts`**: Auto-opens index on project load, auto-indexes all containers, cleans up on project close
- **`src/hooks/useAppActions.ts`**: 2-tier search — Tantivy query first, falls back to in-memory filename filter
- **`src/components/search/SearchFilters.tsx`**: "Search contents" toggle (maps to `searchContent` option)
- **`src/components/search/SearchResultItem.tsx`**: Renders content snippets with `<mark>` highlighting, content match badge

### Key Files

| File | Purpose |
|------|--------|
| `src-tauri/src/search/mod.rs` | Schema, `SearchIndex` struct, global registry, `classify_extension()`, `is_text_eligible()` |
| `src-tauri/src/search/indexer.rs` | Container crawlers, content extractors, `index_container()`, `rebuild_index()` |
| `src-tauri/src/search/query.rs` | `search()`, `SearchOptions`, `SearchResults`, `SearchHit`, BM25 + filters |
| `src-tauri/src/commands/search.rs` | 8 Tauri commands: open/close/delete/stats/index/query |
| `src/api/search.ts` | Frontend API wrappers + types |
| `src/hooks/useSearchIndex.ts` | Auto-index lifecycle hook |
| `src/hooks/useAppActions.ts` | Search handler (Tantivy → fallback) |

### Do NOT

- Remove the FTS5 search in `project_db` — it handles notes/bookmarks/activity_log (different data than Tantivy)
- Remove `useSearchIndex` from App.tsx — it manages the index lifecycle (open/close/auto-index)
- Delete a Tantivy index directory by dropping `SearchIndex` alone — `SearchIndex::destroy()` must take the writer out, call `wait_merging_threads()`, then drop the reader/index before removing the directory or macOS CI can fail with `Directory not empty`
- Use `Index::exists(&Path)` — Tantivy requires `Index::exists(&dyn Directory)` via `MmapDirectory::open()`
- Use `DirEntry.size` or `DirEntry.modified` in VFS crawlers — `DirEntry` only has `name` and `is_directory`; use `vfs.getattr()` for `FileAttr`
- Use `archive::is_archive_path()` — the function is `archive::is_archive()` and returns `Result<bool>`
- Use `RangeQuery::new_u64(field: Field, range)` — API takes `String` field name: `RangeQuery::new_u64_bounds("size".to_string(), Bound, Bound)`
- Use `searcher.doc(addr)` without turbofish — must be `searcher.doc::<TantivyDocument>(addr)`
- Use `OwnedValue.as_str()` without importing `tantivy::schema::Value` trait
- Call `L01Entry.modified_time` — the field is `modification_time` (i64)
- Pass `AD1 TreeEntry.data_addr` directly — it's `Option<u64>`, use `.unwrap_or(0)`
- Pass `AD1 TreeEntry.child_count` directly — it's `Option<usize>`, use `.unwrap_or(0)`

---

## Project Merge (Merge Projects Wizard)

The Merge Projects feature combines multiple `.cffx` projects and their `.ffxdb` databases into a single project. It includes **examiner identification** — gathering examiner/user names from both `.cffx` and `.ffxdb` sources to help identify which work belongs to which examiner.

The wizard supports **two modes**:
1. **Standard merge**: Select 2+ `.cffx` files → merge into a new output file
2. **Merge into open project**: When a project is already open, the current project is pinned and only 1 additional project is needed. Output defaults to the current project path. Collection data reconciliation is shown when conflicts are detected.

### Architecture

```text
Tools → "Merge Projects" (menu bar)
  → menu.rs emits "merge-projects"
  → useMenuActions dispatches onMergeProjects
  → App.tsx sets showMergeWizard(true)
  → MergeProjectsWizard.tsx (lazy-loaded modal)
    ├── Step 1: Select .cffx files (current project pinned in merge-into-open mode)
    ├── Step 2: Review (examiners, evidence, collections, COC, forms) + configure
    │           + Collection Reconciliation (merge-into-open mode only)
    ├── Step 3: Execute merge (analyzeProjects → executeMerge)
    └── Step 4: Results + "Open Merged Project" / "Reload Project"
```

### Merge-Into-Open Mode

When `currentProjectPath` prop is set (passed from `AppSecondaryModals` via `projectManager.projectPath()`):

| Aspect | Standard Merge | Merge Into Open |
|--------|---------------|-----------------|
| **Minimum projects** | 2+ selected | 1 additional (current pinned) |
| **File dialog** | Multi-select | Single-select |
| **Output path** | User-chosen (save dialog) | Predetermined (current .cffx) |
| **Header title** | "Merge Projects" | "Merge Into Project" |
| **Merge button** | "Merge Projects" | "Merge Into Project" |
| **Complete button** | "Open Merged Project" | "Reload Project" |
| **Collection reconciliation** | Not shown | Shown when conflicts detected |
| **Post-merge action** | Open new project | Reload current project |

### Collection Reconciliation

When merging into an open project, the wizard detects potential collection conflicts between the current and incoming projects. `INSERT OR IGNORE` handles same-ID dedup automatically, but different-ID records representing the same real-world collection event need user reconciliation.

**Conflict detection** (`detectConflicts()` in `CollectionReconciliation.tsx`):
- Same case number (exact match, case-insensitive)
- Same collection date + collecting officer
- Same collection date + collection location
- Same-ID records are skipped (handled by `INSERT OR IGNORE`)

**Reconciliation UI** (shown in review step when conflicts exist):
- Each conflict shows side-by-side cards (current vs. incoming) with radio buttons: "Keep Current" or "Use Incoming"
- Non-conflicting incoming collections have checkboxes to include/exclude
- Default: all conflicts default to "Keep Current"

**Exclude pipeline**: User choices are compiled into a `MergeExclusions` object and passed through:
1. Frontend `handleMerge()` → builds `MergeExclusions` from category toggles, per-item checkboxes, and reconciliation choices
2. `executeMerge()` in `projectMerge.ts` → passes `exclusions: MergeExclusions` to Tauri
3. `project_merge_execute` command → passes to `execute_merge()`
4. `merge_databases()` in `merge_db.rs` → applies category-level skip via `table_category()` + item-level `WHERE id NOT IN (...)` filters on evidence_files (+ hashes/verifications), coc_items (+ amendments/audit_log/transfers), evidence_collections (+ collected_items), and form_submissions

### Two-Phase Pipeline

1. **Analyze** (`project_merge_analyze`): Reads each `.cffx` (JSON) + `.ffxdb` (SQLite, read-only). Returns `ProjectMergeSummary[]` with counts, examiners, collections, COC items, forms, and evidence files.
2. **Execute** (`project_merge_execute`): Loads all `.cffx` → merges data (dedup by ID) → builds provenance → rebases paths → saves merged `.cffx` → ATTACH each `.ffxdb` → INSERT OR IGNORE into merged `.ffxdb`. Optional `exclusions: MergeExclusions` parameter enables category-level skip (11 categories) and item-level filtering (evidence files, COC items, collections, form submissions) during merge.

### Examiner Identification

The analyze phase gathers examiner names from **7 primary sources** plus **6 additional fallback sources** (ordered by priority). All deduplication is **case-insensitive**.

**Primary sources:**

| Source | Data Location | Role Label |
|--------|--------------|------------|
| Project owner | `.cffx` `owner_name` field | `"project owner"` |
| Project users | `.cffx` `users[]` array | `"session user"` |
| DB users table | `.ffxdb` `users` table | `"session user"` |
| Collecting officers | `.ffxdb` `evidence_collections.collecting_officer` | `"collecting officer"` |
| COC submitted_by | `.ffxdb` `coc_items.submitted_by` | `"submitted by (COC)"` |
| COC received_by | `.ffxdb` `coc_items.received_by` | `"received by (COC)"` |
| Processed DB examiner | `.ffxdb` `processed_databases.examiner` + `axiom_case_info.examiner` | `"processed DB examiner"` / `"AXIOM examiner"` |

**Additional fallback clues** (queried via `query_ffxdb_additional_clues` when primary sources yield no owner):

| Source | Data Location | Role Label |
|--------|--------------|------------|
| Session users | `.ffxdb` `sessions.user` | `"session user"` |
| Activity log users | `.ffxdb` `activity_log.user` | `"activity user"` |
| Bookmark authors | `.ffxdb` `bookmarks.created_by` | `"bookmark author"` |
| Note authors | `.ffxdb` `notes.created_by` | `"note author"` |
| Report authors | `.ffxdb` `reports.generated_by` | `"report author"` |
| Export initiators | `.ffxdb` `export_history.initiated_by` | `"export initiator"` |

Examiners are **deduplicated by name** (case-insensitive). The wizard auto-suggests the project owner from this list (prioritizing "project owner" → "session user" → first examiner).

### Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/project/merge.rs` | Core merge logic: `analyze_projects()`, `merge_projects()`, `execute_merge()` + 7 query helpers + `extract_form_details()` |
| `src-tauri/src/project/merge_types.rs` | `MergeExclusions` struct, `ProjectMergeSummary`, all merge-related Rust types |
| `src-tauri/src/project/merge_db.rs` | Database merge: `merge_databases()` with `INSERT OR IGNORE`, WAL handling, `table_category()` mapping (30 tables → 11 categories), category + item-level exclusion filters |
| `src-tauri/src/commands/project_merge.rs` | Tauri command wrappers: `project_merge_analyze`, `project_merge_execute` (with `exclusions: Option<MergeExclusions>`) |
| `src/api/projectMerge.ts` | Frontend types + invoke wrappers: `ProjectMergeSummary`, `MergeExaminerInfo`, `MergeExclusions`, `MergeDataCategory`, `MergeCategoryInfo`, `MERGE_CATEGORIES` (11 categories) |
| `src/components/merge/MergeProjectsWizard.tsx` | Main wizard: dual-mode (standard merge vs. merge-into-open), reconciliation state, category + item selection state |
| `src/components/merge/DataCategorySelector.tsx` | 2-column checkbox grid for toggling 11 data categories, with icons and item counts |
| `src/components/merge/SelectStep.tsx` | Step 1: file picker with pinned current project support |
| `src/components/merge/CollectionReconciliation.tsx` | Conflict detection (`detectConflicts`), reconciliation UI (radio + checkboxes) |
| `src/components/merge/types.ts` | `MergeProjectsWizardProps` (with `currentProjectPath`), `CollectionConflict`, `ReconciliationChoices`, re-exports `MergeExclusions`, `MergeDataCategory` |
| `src/components/merge/ProjectSummaryCard.tsx` | Expandable per-project detail sections with per-item inclusion checkboxes and Include All / Exclude All toggles |
| `src/components/layout/AppSecondaryModals.tsx` | Passes `currentProjectPath` from `projectManager.projectPath()` to wizard |

### Key Types

| Rust | TypeScript |
|------|-----------|
| `MergeExaminerInfo` { name, display_name, source, role } | `MergeExaminerInfo` { name, displayName, source, role } |
| `MergeCollectionSummary` { id, case_number, collection_date, collecting_officer, ... } | `MergeCollectionSummary` { id, caseNumber, collectionDate, collectingOfficer, ... } |
| `MergeCocSummary` { id, coc_number, case_number, evidence_id, ... } | `MergeCocSummary` { id, cocNumber, caseNumber, evidenceId, ... } |
| `MergeFormSummary` { id, template_id, case_number, status, created_at, collecting_officer, collection_location, lead_examiner } | `MergeFormSummary` { id, templateId, caseNumber, status, createdAt, collectingOfficer, collectionLocation, leadExaminer } |
| `MergeEvidenceFileSummary` { id, path, filename, container_type, total_size } | `MergeEvidenceFileSummary` { id, path, filename, containerType, totalSize } |

### Wizard Review Step (Step 2) Detail Sections

Each project card in the review step has 5 expandable sections (chevron toggle):

| Section | Icon | Data Source | Per-Item Selection |
|---------|------|-------------|-------------------|
| Examiners | `HiOutlineUserGroup` | `summary.examiners` — role badges + source label | No (informational only) |
| Evidence Files | `HiOutlineArchiveBox` | `summary.evidenceFiles` — filename, type badge, size | Yes — checkboxes + Include All / Exclude All |
| Collections | `HiOutlineArchiveBoxArrowDown` | `summary.collections` — case #, officer, items, status | Yes — checkboxes + Include All / Exclude All |
| Chain of Custody | `HiOutlineShieldCheck` | `summary.cocItems` — COC #, from/to, status | Yes — checkboxes + Include All / Exclude All |
| Forms & Evidence Collections | `HiOutlineClipboardDocumentList` | `summary.formSubmissions` — friendly template name, case #, officer/examiner, status | Yes — checkboxes + Include All / Exclude All |

Excluded items are visually dimmed (`opacity-50`) and the section header shows an inclusion counter (e.g., "3/5").

The Owner input uses a `<datalist>` auto-complete populated from the examiner list.

### Merge Database Coverage

`merge_databases()` merges **30 tables** via `INSERT OR IGNORE`, including: `users`, `sessions`, `activity_log`, `evidence_files`, `hashes`, `verifications`, `bookmarks`, `notes`, `tags`, `tag_assignments`, `reports`, `saved_searches`, `recent_searches`, `case_documents`, `processed_databases`, `axiom_case_info`, `axiom_evidence_sources`, `axiom_search_results`, `artifact_categories`, `coc_items`, `coc_amendments`, `coc_audit_log`, `coc_transfers`, `evidence_collections`, `collected_items`, `form_submissions`, `export_history`, `annotations`, `processed_db_integrity`, `processed_db_metrics`, `ui_state`. Tables not present in a source DB are safely skipped. FTS tables and `schema_meta` are not merged (they auto-rebuild).

### Granular Merge Selection (Category + Item Level)

The merge wizard supports **two levels of selectivity** via the `MergeExclusions` struct:

**Category-level** (`skip_categories: Vec<String>`): Entire data categories can be toggled off. The 30 tables are mapped to 11 categories by `table_category()` in `merge_db.rs`:

| Category | Tables |
|----------|--------|
| `evidence` | `evidence_files`, `hashes`, `verifications` |
| `bookmarks_notes` | `bookmarks`, `notes`, `annotations` |
| `activity` | `users`, `sessions`, `activity_log` |
| `coc` | `coc_items`, `coc_amendments`, `coc_audit_log`, `coc_transfers` |
| `collections` | `evidence_collections`, `collected_items` |
| `forms` | `form_submissions` |
| `reports` | `reports` |
| `tags` | `tags`, `tag_assignments` |
| `searches` | `saved_searches`, `recent_searches` |
| `documents` | `case_documents` |
| `processed` | `processed_databases`, `axiom_case_info`, `axiom_evidence_sources`, `axiom_search_results`, `artifact_categories`, `processed_db_integrity`, `processed_db_metrics` |

Tables not in any category (`ui_state`) are always merged. The `exports` category covers `export_history` only.

**Item-level** (per-ID exclusion): Individual records can be excluded within an included category:
- `exclude_evidence_file_ids` → filters `evidence_files` + cascades to `hashes`/`verifications` via `WHERE file_id NOT IN (...)`
- `exclude_coc_item_ids` → filters `coc_items` + cascades to `coc_amendments`/`coc_audit_log` via `WHERE coc_item_id NOT IN (...)` and `coc_transfers` via `WHERE coc_id NOT IN (...)`
- `exclude_collection_ids` → filters `evidence_collections` + cascades to `collected_items` via `WHERE collection_id NOT IN (...)`
- `exclude_form_submission_ids` → filters `form_submissions`

**Frontend state:** `MergeProjectsWizard.tsx` manages selection via `skippedCategories`, `excludedEvidenceFileIds`, `excludedCocItemIds`, `excludedFormIds` signals. `DataCategorySelector` renders a 2-column checkbox grid for categories. `ProjectSummaryCard` renders per-item checkboxes in expandable sections.

### WAL File Handling (Critical for Merge)

`.ffxdb` databases use WAL (Write-Ahead Logging) mode. When a database has an active WAL file (`.ffxdb-wal`), the main `.ffxdb` file may be nearly empty (just the header) — ALL data lives in the WAL. Opening with `SQLITE_OPEN_READ_ONLY` prevents WAL replay, causing queries against empty tables.

**Analyze phase** (`open_ffxdb_for_analysis()`): When a `.ffxdb-wal` file exists and is non-empty, the function copies `.ffxdb` + `.ffxdb-wal` + `.ffxdb-shm` to a temp directory, opens read-write to trigger WAL replay, runs `PRAGMA wal_checkpoint(TRUNCATE)`, and queries the temp copy. This preserves forensic integrity (original files untouched). If no WAL exists, opens directly with `SQLITE_OPEN_READ_ONLY`.

**Execute phase** (`merge_databases()`): Same WAL handling when ATTACHing source databases. If a source has an active WAL, it's copied to a temp dir and checkpointed before ATTACH. Temp directories are kept alive until the merge completes.

All query errors are logged via `warn!()` (not silently swallowed).

### Form Data Extraction

`MergeFormSummary` includes fields extracted from `data_json`: `collecting_officer`, `collection_location`, `lead_examiner`. This enables the wizard to show meaningful details for evidence collection forms. Template IDs are mapped to friendly names (e.g., `evidence_collection` → "Evidence Collection", `iar` → "Investigative Activity Report") via `friendlyTemplateName()` in the wizard.

### Do NOT

- Remove the examiner gathering from `analyze_projects()` — it's critical for multi-examiner merge identification
- Remove the `<datalist>` auto-suggest from the Owner input — it helps users pick the correct examiner
- Add `merge-projects` to `PROJECT_DEPENDENT_IDS` — merging should work without a project loaded
- Assume `.ffxdb` always exists — the analyze phase handles missing `.ffxdb` gracefully (empty arrays)
- Open `.ffxdb` files with `SQLITE_OPEN_READ_ONLY` when WAL files may be present — use `open_ffxdb_for_analysis()` which handles WAL replay via temp copy
- Use `if let Ok(...)` for query errors without logging — all query failures must be logged with `warn!()`
- Remove the WAL temp-copy logic from `open_ffxdb_for_analysis()` or `merge_databases()` — databases on external volumes frequently have un-checkpointed WAL files
- Remove the `extract_form_details()` function or data_json extraction — it provides examiner identification from form submissions
- Remove tables from `merge_databases()` `merge_tables` list without confirming they don't exist in the schema
- Remove `currentProjectPath` from `MergeProjectsWizardProps` — it enables merge-into-open mode
- Remove category or item-level exclusion filters from `merge_databases()` — they power both category toggles and per-item selection
- Pass `projectManager.projectName()` as `currentProjectPath` — it must be the full `.cffx` file path from `projectManager.projectPath()`
- Remove `CollectionReconciliation` from the review step `<Show>` wrapper — it prevents the component from appearing in standard merge mode where it's not needed
- Remove the `exclusions` parameter from `project_merge_execute` — it's required for granular merge selection and collection reconciliation
- Remove `DataCategorySelector` from the Review step — it's the only UI for category-level merge control
- Remove per-item selection props from `ProjectSummaryCard` — they enable item-level merge control in expandable sections
- Remove `table_category()` from `merge_db.rs` — it maps 30 tables to 11 categories for category-level exclusion
- Remove `MergeExclusions` struct or any of its fields — all 5 fields are used by the frontend selection UI

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
| `src/types/projectDb.ts` | `src-tauri/src/project_db/types.rs`, `src-tauri/src/commands/project_db/` (modular directory) |
| `src/types/processed.ts` | `src-tauri/src/processed/types.rs` |
| `src/report/types.ts` | `src-tauri/src/report/types/` (modular: `mod.rs`, `case.rs`, `findings.rs`, `records.rs`, `evidence_collection.rs`) |
| `src/types/hash.ts` | `src-tauri/src/containers/types.rs` (StoredHash) |
| `src-tauri/src/archive/types.rs` | `src/types.ts` (ArchiveFormat, etc.) |
| `src/components/OfficeViewer.tsx` (inline types) | `src-tauri/src/viewer/document/office.rs` (OfficeDocumentInfo, OfficeMetadata, etc.) |
| `src/api/ewfExport.ts` (EwfExportOptions) | `src-tauri/src/commands/ewf_export.rs` |
| `src/api/l01Export.ts` (L01ExportOptions, L01ExportProgress, L01ExportResult) | `src-tauri/src/commands/l01_export.rs`, `src-tauri/src/l01_writer/types.rs` |
| `src/api/drives.ts` (DriveInfo, MountResult) | `src-tauri/src/commands/system.rs` |
| `src/api/device.ts` (PrivilegeInfo, PhysicalDisk, DeviceReadProgress) | `src-tauri/src/commands/device.rs` |
| `src/components/report/types.ts` (wrapper over `@core-suite/types/forensic-report`; COCItem: status, locked_at, locked_by) | `src-tauri/src/project_db/types.rs` (DbCocItem) |
| `src/api/projectMerge.ts` (MergeExclusions, ProjectMergeSummary, MergeDataCategory) | `src-tauri/src/project/merge_types.rs` |
| `src/api/search.ts` (SearchOptions, SearchHit, SearchResults, IndexProgress, IndexStats) | `src-tauri/src/search/query.rs`, `src-tauri/src/search/indexer.rs`, `src-tauri/src/search/mod.rs` |
| `src/api/dedup.ts` (DedupOptions, DedupResults, DuplicateGroup, DuplicateFile, DuplicateMatchType, DedupStats) | `src-tauri/src/dedup/types.rs`, `src-tauri/src/dedup/mod.rs` |
| `src/api/portable.ts` (PortableConfig, PortableStatus) | `src-tauri/src/commands/portable.rs` |
| `src/api/triage.ts` (TriageCategory, TriageProfile, TriageOptions, TriageProgress, SecretFinding, TriageResult) | `src-tauri/src/commands/triage.rs` |
| `src/api/companion.ts` (CompanionFileInput, CompanionFile) | `src-tauri/src/commands/companion.rs` |
| `src/api/aff4Export.ts` (Aff4ExportOptions, Aff4ExportProgress) | `src-tauri/src/commands/aff4_export.rs` |
| `src/api/rawExport.ts` (RawExportOptions, RawExportProgress, RawExportResult) | `src-tauri/src/commands/raw_export.rs` |
| `src/api/exportHistory.ts` (ExportRecord) | `src-tauri/src/project_db/types.rs` (DbExportRecord) |
| `src/api/segmentHash.ts` (SegmentHashProgress, SegmentHashResult) | `src-tauri/src/commands/segment_verify.rs` |
| `src/api/fda.ts` (FullDiskAccessStatus) | `src-tauri/src/commands/system.rs` (`check_full_disk_access`, `open_full_disk_access_settings`) |

---

## Report Rendering

The report subsystem (`src-tauri/src/report/`) has two separate flows:

- **Standard forensic reports** use `ReportGenerator` in `mod.rs` and render through `html.rs`, `markdown.rs`, `pdf.rs`, or `docx.rs`
- **Specialized PDF forms** use `pdf_coc_form7.rs` and `pdf_evidence_collection.rs` when `report_type` is `chain_of_custody` or `evidence_collection`

**Preview/export invariant:** `commands.rs` `preview_report` must use the canonical `HtmlGenerator` output via `ReportGenerator::render_preview_html()`. Do not route the default wizard preview back through `template_engine.render_html()` or preview and exported HTML will drift apart.

**Narrative formatting invariant:** `format_helpers.rs` is the shared normalization layer for freeform narrative text (summary, scope, methodology, conclusions, notes, appendix content, signature notes). Use it for multi-paragraph and bullet-list handling instead of raw newline replacement in individual renderers.

**Section coverage invariant:** When data is present, the standard HTML/Markdown/PDF/DOCX renderers should all cover the same major sections: Case Information, Executive Summary, Scope, Methodology, Evidence, Evidence Collection, Chain of Custody, Findings, Timeline, Hash Verification, Tools, Conclusions, Additional Notes, Appendices, and Approvals/Signatures. Layouts can differ, but section coverage should stay aligned.

**Canonical TypeScript contract:** `src/components/report/types.ts` is a compatibility wrapper over `@core-suite/types/forensic-report`. Add new canonical report fields in the shared package first so CORE-FFX and CORE-ACQ stay aligned.

**Standard package export:** The report wizard export step can save a portable `.forensic-report.json` package containing a `ForensicReportPackage` (`schema_version`, `source`, `report`, optional overrides). This package is the handoff format for CORE-RPT customization. Keep the wizard-side package writer aligned with the shared `@core-suite/types/forensic-report` contract rather than introducing a second local package schema. Shared schema/version constants and standard `.forensic-report.json` filename builders belong in that shared package layer, not in CORE-FFX-local helpers.

**Do NOT:**
- Remove `format_helpers.rs` from the standard renderers and fall back to raw newline-to-`<br>` or single-paragraph dumps for narrative sections
- Re-route `preview_report` through `template.rs` for the default report wizard flow
- Merge the standard PDF renderer with `pdf_coc_form7.rs` or `pdf_evidence_collection.rs` — the form layouts are intentionally separate
- Add a new narrative section to only one standard renderer — keep HTML, Markdown, PDF, and DOCX coverage aligned
- Recreate the canonical `ForensicReport` type locally in CORE-FFX — extend `@core-suite/types/forensic-report` and keep the local wrapper thin
- Change the exported standard package structure in the wizard without updating CORE-RPT's package reader/editor flow — the package JSON is the cross-app contract now

## Testing

```bash
cd src-tauri
cargo test                           # Run all tests
cargo test viewer::document::        # Run specific module tests
cargo test --test test_document_formats -- --nocapture  # Integration tests
cargo run --example logical_container_benchmark -- --dataset-mib 128  # Compare 7z Store vs AFF4 Stored vs L01 None
cd ../libewf-ffi && cargo test       # libewf-ffi reader/writer tests (39 total)
```

The logical container benchmark example is the reproducible CLI entry point for acquisition-speed comparisons. It creates 7z Store, AFF4 Stored, and L01 None outputs from the same source dataset, reopens them through the real reader stack, and hashes the output files with MD5, SHA-1, SHA-256, and BLAKE3.

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

### L01 Reader — Ltree Parser

The pure-Rust L01 reader (`src-tauri/src/ewf/l01_reader.rs`) parses ltree sections from L01 logical evidence containers. It supports **two ltree format versions** with auto-detection:

| Version | Format | Hierarchy | Producers |
|---------|--------|-----------|----------|
| **V2** (tab-depth) | Tab-delimited key-value pairs | Depth indicated by leading tab count | EnCase, most tools |
| **V3** (columnar) | 31-column positional, child_count-based | Parent-child via `child_count` field (no tab-depth) | FTK Imager |

**Key types:**
- `L01Entry` — parsed file/directory with identifier, name, size, data_offset, parent_id, path, hashes, timestamps
- `L01RecordSummary` — `total_bytes`, `file_count`, `cluster_size` from `rec` category
- `L01SourceInfo` — `name`, `identifier`, `evidence_number` from `srce` category
- `L01FileTree` — all entries + record_summary + sources + `id_to_index` HashMap

**Multi-segment support:** `discover_l01_segments()` in `common/segments.rs` finds .L01/.L02/.L03/etc. The ltree parser scans segments in **reverse order** (ltree is in the last segment). Uses `compressed_size` fallback logic when `section_size` is omitted.

**L01 metadata enrichment:** `enrich_l01_info()` in `containers/operations/mod.rs` calls `parse_l01_file_tree()` and fills 4 L01-specific `EwfInfo` fields: `l01_source_name`, `l01_source_evidence_number`, `l01_file_count`, `l01_total_bytes`. Called from both `info()` and `info_fast()` L01 branches. The `l01_source_evidence_number` also backfills `evidence_number` if the header didn't have one.

**Evidence collection auto-fill:** `evidenceAutoFill.ts` maps L01 source metadata to form fields — source name → description/brand fallback, evidence number → item_number fallback, file count + total bytes → storage_notes.

### Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/ewf/l01_reader.rs` | L01 ltree parser (V2 tab-depth + V3 columnar, multi-segment, `L01Entry`/`L01FileTree`/`L01RecordSummary`/`L01SourceInfo`) |
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
- Compute EWF viewer metadata section bounds in `ewf/parser/metadata.rs` with raw `section.file_offset + ...` arithmetic — use checked offset math so malformed section headers are skipped safely instead of overflowing region construction
- Compute L01 entry read offsets in `commands/container.rs` with raw `entry.data_offset + offset` arithmetic — use checked addition after the zero-length guard so malformed logical-entry offsets cannot overflow chunk reads
- Compute file byte read lengths in `commands/analysis.rs` with raw `(file_size - offset) as usize` arithmetic — use checked subtraction and bounded `u64 -> usize` conversion so large files cannot truncate the remaining-byte calculation before buffer allocation.
- Compute raw VFS physical read lengths or seek positions in `crates/ffx-raw/src/vfs.rs` or `src-tauri/src/raw/vfs.rs` with raw `(total_size - offset) as usize` arithmetic or saturating seek math — use checked subtraction, bounded `u64 -> usize` conversion, and checked seek-position helpers so large raw images cannot truncate the remaining-byte calculation or silently saturate invalid seek offsets before physical-read buffer allocation.
- Compute shared binary reader buffer sizes in `crates/ffx-common/src/binary.rs` with direct `vec![0; length]` allocation from untrusted lengths — use checked exact-buffer reservation so malformed string or byte lengths fail cleanly instead of attempting impossible allocations.
- Compute portable free-space bytes in `src-tauri/src/commands/portable.rs` with raw `f_bavail * f_frsize` multiplication — use checked multiplication so malformed filesystem metadata degrades safely instead of wrapping the reported byte count.
- Gate `checked_available_bytes()` in `src-tauri/src/commands/portable.rs` behind `#[cfg(unix)]` only — keep the checked-multiplication helper available for `#[cfg(test)]` builds too (for example `#[cfg(any(unix, test))]`) so Windows clippy/test builds still compile even though the `statvfs` caller is Unix-only.

---

## L01 Writer — Pure-Rust Logical Evidence Writer

Custom pure-Rust implementation for creating L01 logical evidence containers (EWF v1 Logical format). Does **not** use libewf — entirely self-contained.

### Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/l01_writer/mod.rs` | `L01Writer` — main writer struct, `add_file()`, `add_directory()`, `add_source_directory()`, `write()` |
| `src-tauri/src/l01_writer/types.rs` | `L01WriterConfig`, `L01CaseInfo`, `CompressionLevel`, `L01HashAlgorithm`, `LefFileEntry`, `LefSource`, `L01WriteResult` |
| `src-tauri/src/l01_writer/chunks.rs` | zlib chunk compression (32 KB chunks) |
| `src-tauri/src/l01_writer/sections.rs` | EWF v1 section writers (header, header2, volume, sectors, table, table2, ltypes, ltree, data, hash, digest, done, next) |
| `src-tauri/src/l01_writer/segment.rs` | Multi-segment file support |
| `src-tauri/src/l01_writer/ltree.rs` | UTF-16LE ltree text builder (5 categories: file, perm, sub, src, entry) |
| `src-tauri/src/commands/l01_export.rs` | Tauri commands: `l01_create_image`, `l01_estimate_size`, `l01_cancel_export` |
| `src/api/l01Export.ts` | Frontend API: `createL01Image()`, `cancelL01Export()`, `estimateL01Size()`, `buildL01ExportOptions()` |
| `src/components/export/CreateMode.tsx` | L01 UI section (image name, compression, hash, segment size, case metadata) |

### L01 Capabilities

- **Compression**: None / Fast / Best (zlib deflate)
- **Hash algorithms**: MD5 or SHA-1 (image-level integrity)
- **Per-file hashes**: MD5 and SHA-1 stored per `LefFileEntry`
- **Multi-segment**: Configurable `segment_size` (default 2 GB, 0 = no splitting)
- **Case metadata**: Case number, evidence number, examiner, description, notes
- **File timestamps**: Creation, modified, access, entry modification, deletion (5 timestamps)
- **Directory hierarchy**: Parent/child tree via `parent_identifier`
- **Permission groups**: `LefPermissionGroup` with SID and permissions bitmask
- **Subjects**: `LefSubject` (user accounts/profiles)
- **Source tracking**: `LefSource` with device GUID, manufacturer, model, serial number
- **Extended attributes**: Generic key-value metadata per file
- **Progress events**: 6 phases (Preparing → WritingData → BuildingTables → WritingLtree → ComputingHash → Finalizing)
- **Cancellation**: `AtomicBool` cancel flag via `l01_cancel_export`
- **EnCase compatibility**: LVF signature, compatible with EnCase, FTK Imager, AXIOM

### When to Use Which Writer

| Need | Writer | Command |
|------|--------|---------|
| **Physical disk image** (.E01) | `libewf-ffi::EwfWriter` | `ewf_create_image` |
| **Logical file collection** (.L01) | `l01_writer::L01Writer` | `l01_create_image` |
| **7z archive** (.7z) | `sevenzip-ffi` | `create_7z_archive` |

### Do NOT

- Use `libewf-ffi` for L01 creation — libewf's C library explicitly rejects logical format constants for writing
- Confuse `l01_writer` (creation) with `ewf/` parser (reading) — they are separate implementations
- Add BZIP2 compression to L01 — the L01 v1 format only supports zlib deflate
- Remove the `LVF_SIGNATURE` (`[0x4C, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00]`) — it identifies L01 files vs E01

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

**Do NOT:**
- Compute the APFS read clamp in `crates/ffx-common/src/filesystem/apfs_driver.rs` or `src-tauri/src/common/filesystem/apfs_driver.rs` with raw `offset + size as u64` arithmetic — use checked or saturating clamping first so oversized chunk requests cannot overflow before the file-boundary guard runs.
- Compute APFS B-tree TOC/key/value offsets or reserve TOC vectors in `crates/ffx-common/src/filesystem/apfs_driver.rs` or `src-tauri/src/common/filesystem/apfs_driver.rs` with raw `toc_offset + ...`, `key_area_offset + ...`, `key_area_offset + kvloc.key_offset`, `block_size - val_offset - val_len`, or direct `Vec::with_capacity(nkeys)` reservation — use checked addition/subtraction plus fallible TOC reservation so malformed catalog, object-tree, or extent-node metadata cannot overflow, underflow, or request oversized upfront allocations before buffer slicing.
- Compute APFS extent read lengths, logical extent ends, physical read offsets, or destination copy ranges in `crates/ffx-common/src/filesystem/apfs_driver.rs` or `src-tauri/src/common/filesystem/apfs_driver.rs` with raw `(read_end - offset) as usize`, raw `extent_logical_offset + extent_length`, raw `self.offset + extent_phys_block * block_size + read_start_in_extent`, raw `total_to_read - dest_offset`, or unchecked `bytes_filled += copy_len` arithmetic — use checked range-length helpers, checked extent-end and physical-offset assembly, and checked destination bookkeeping so malformed APFS extent metadata cannot underflow or wrap file reads.
- Compute the exFAT read slice end in `exfat_driver.rs` with raw `start + size` arithmetic — use checked or saturating slicing so oversized chunk requests degrade safely instead of overflowing before the bounds check runs.
- Compute exFAT cluster offsets, FAT entry offsets, parsed file sizes, contiguous cluster numbers, or result capacities in `crates/ffx-common/src/filesystem/exfat_driver.rs` or `src-tauri/src/common/filesystem/exfat_driver.rs` with raw reserved-cluster subtraction, unchecked absolute-offset addition, lossy `entry.size as usize`, unchecked `entry.start_cluster + i as u32`, or direct metadata-driven `Vec` capacity allocation — use checked arithmetic and fallible reservation so malformed cluster metadata fails cleanly instead of underflowing, wrapping, or attempting impossible allocations.
- Compute HFS+ catalog record or key offsets in the shared or app-side `hfsplus_driver.rs` copies with raw `record_start + key_size` / `data_offset + ...` arithmetic or a 6-byte key-header minimum — use checked offset math and require the full 8-byte catalog-key header before reading `name_length`.
- Allocate HFS+ record-offset or node-name buffers in `crates/ffx-common/src/filesystem/hfsplus_driver.rs` or `src-tauri/src/common/filesystem/hfsplus_driver.rs` with direct `Vec::with_capacity` calls from catalog record or name counts — use checked/clamped reservation so malformed catalog metadata cannot trigger oversized upfront allocations.
- Compute HFS+ catalog-header or fork-read offsets in `crates/ffx-common/src/filesystem/hfsplus_driver.rs` or `src-tauri/src/common/filesystem/hfsplus_driver.rs` with raw `offset + start_block * block_size`, raw `(logical_size - offset) as usize`, raw `extent_logical_start + extent_size`, raw `self.offset + ... + extent_offset`, or raw `extent_size - extent_offset` arithmetic — use checked offset math and bounded read-length helpers so malformed HFS+ extents cannot wrap or underflow during catalog-header reads or file-fork reads.
- Parse ext directory entries in `crates/ffx-common/src/filesystem/ext_driver.rs` or `src-tauri/src/common/filesystem/ext_driver.rs` with raw `offset + 8 + name_len`, advance `offset` without first validating `rec_len`, or allow names to extend past the current record — use checked record bounds so malformed ext directory metadata cannot read past an entry or step outside the buffer.
- Build ext block-group descriptor tables in `crates/ffx-common/src/filesystem/ext_driver.rs` or `src-tauri/src/common/filesystem/ext_driver.rs` with unchecked `num_groups * desc_size`, raw `offset + block_size` arithmetic, direct `vec![0; inode_size as usize]` / descriptor-table buffer allocation, direct `Vec::with_capacity(num_groups)` reservation, or by ignoring the byte count returned from `read_at` — validate the table size against the device, reject undersized inode records, use checked descriptor offsets plus fallible inode/descriptor reservation, and parse only the bytes that were actually read so malformed ext metadata cannot overflow allocations or treat the zero-filled tail of the buffer as real descriptors.
- Compute ext inode-read lengths or data-block offsets in `crates/ffx-common/src/filesystem/ext_driver.rs` or `src-tauri/src/common/filesystem/ext_driver.rs` with raw `file_size - offset`, raw `self.offset + block_num * block_size + offset_in_block`, or unchecked `current_offset += bytes_to_read` arithmetic — use checked subtraction, checked block-offset assembly, and checked rolling offsets so malformed ext inode metadata cannot underflow, wrap, or truncate the remaining-byte calculation before reading file data blocks.
- Compute NTFS read offsets in `crates/ffx-common/src/filesystem/ntfs_driver.rs` or `src-tauri/src/common/filesystem/ntfs_driver.rs` with raw `self.offset + self.position` arithmetic, raw available-byte subtraction like `file_size - offset` or `self.size - self.position`, cast the remaining byte count directly to `usize`, or accept truncated NTFS boot sectors from `read_at` — clamp read lengths with checked subtraction and checked conversion, reject short boot-sector reads before parsing BPB fields, and validate seek/read offsets so malformed NTFS metadata cannot overflow absolute offsets or silently saturate invalid seeks.
- Compute NTFS total filesystem size in `crates/ffx-common/src/filesystem/ntfs_driver.rs` or `src-tauri/src/common/filesystem/ntfs_driver.rs` with raw `total_sectors * bytes_per_sector` arithmetic — use checked multiplication so malformed boot-sector metadata fails cleanly instead of wrapping the reported NTFS size.
- Compute DMG memory block read slices in the shared or app-side `dmg_driver.rs` copies with lossy `u64 -> usize` casts or raw `offset + to_read` / `position + to_read` arithmetic — use checked conversion and bounded slice ends so oversized offsets degrade to EOF safely.
- Compute GPT entry-array sizes, entry offsets, or byte ranges in the shared or app-side `partition.rs` copies with raw multiplication/subtraction (`num_entries * entry_size`, `i * entry_size`, `end_lba - start_lba + 1`) or unchecked LBA-to-byte conversion — use checked arithmetic so malformed headers cannot overflow allocations, entry slices, or partition sizes.
- Parse FAT boot sectors in `crates/ffx-common/src/filesystem/fat.rs` or `src-tauri/src/common/filesystem/fat.rs` without checking how many bytes `read_at` actually returned — use the real byte count for BPB, signature, and label parsing so short reads fail cleanly instead of reading the zero-filled tail of the fixed 512-byte buffer.
- Compute FAT file read lengths, wrapper read offsets, or wrapper seek positions in `crates/ffx-common/src/filesystem/fat.rs` or `src-tauri/src/common/filesystem/fat.rs` with raw `(file_size - offset) as usize` arithmetic, unchecked `self.offset + self.position` addition, or saturating `SeekFrom::Current` / `SeekFrom::End` math — use checked subtraction, bounded `u64 -> usize` conversion, checked absolute-offset math, and checked seek-position helpers so large FAT files cannot truncate the remaining-byte calculation before allocation, overflow the underlying block-device read offset, or silently saturate invalid relative seeks.
- Compute AFF4 bevy chunk ranges in `crates/ffx-aff4/src/reader.rs` with raw `entry.offset as usize..entry.offset as usize + entry.length as usize` slicing, divide by unchecked `chunk_size` / `chunks_per_segment` values, or add map ranges in `crates/ffx-aff4/src/map.rs` with unchecked `mapped_offset + length` / `target_offset + delta` arithmetic — validate the stream layout first and use checked AFF4 range bounds so malformed metadata returns `InvalidContainer`, `InvalidBevyIndex`, or `None` instead of panicking or wrapping.
- Parse AFF4 Turtle string literals in `crates/ffx-aff4/src/rdf.rs` without assuming a closing quote or a complete escape sequence — track the closing quote first and only inspect typed-literal suffix text after it exists so malformed RDF strings cannot step past the token end or slice suffix text from an unterminated literal.
- Compute archive file read slice ends in `crates/ffx-archive/src/vfs.rs` with raw `start + size` arithmetic — use checked or saturating slicing so oversized chunk requests degrade safely instead of overflowing before the bounds check runs.
- Compute ZIP central-directory bounds, buffer sizes, filename slices, or next-entry offsets in `crates/ffx-archive/src/vfs.rs` or `crates/ffx-archive/src/zip_index.rs` with raw `cd_offset + cd_size`, `cd_size as usize`, `pos + 46 + filename_len`, or `46 + filename_len + extra_len + comment_len` arithmetic — use checked bound validation, checked size conversion, and checked range/advance helpers so malformed ZIP metadata cannot overflow central-directory allocation or slice past entry boundaries.
- Compute ZIP local-file scan buffer sizes, decompressed output capacity, or next-entry offsets in `crates/ffx-archive/src/vfs.rs` with raw `compressed_size as usize`, `uncompressed_size as usize`, or `30 + filename_len as u64 + extra_len as u64 + compressed_size` arithmetic — use checked size conversion and checked local-header bound helpers so malformed ZIP local-file metadata cannot overflow allocation size or walk the VFS scan loop past archive bounds.
- Compute native ZIP entry allocation size in `crates/ffx-archive/src/lib.rs` with raw `entry.size() as usize` reservation arithmetic — use checked size conversion and fallible buffer reservation so malformed ZIP metadata cannot truncate allocation size or panic on impossible reservations during native ZIP reads.
- Compute AD1 file read slice ends in `crates/ffx-ad1/src/vfs.rs` with raw `start + size` arithmetic — use checked or saturating slicing so oversized chunk requests degrade safely instead of overflowing before the bounds check runs.
- Compute AD1 zlib chunk-table entry addresses or chunk output end indexes in `crates/ffx-ad1/src/parser.rs` with unchecked addition or lossy `u64 -> usize` conversion — use checked address math, checked chunk-length conversion, and saturating output-index updates so malformed AD1 metadata returns parse errors instead of wrapping reads or output copies.
- Compute AD1 item-name, parent-folder, or metadata field offsets in `crates/ffx-ad1/src/parser.rs` or `crates/ffx-ad1/src/reader_v2.rs` with raw `offset + ITEM_*`, `offset + METADATA_*`, or `offset + ITEM_NAME + name_length` arithmetic, or cast item-name or metadata lengths with `as usize` / `as u64` on either the lazy or eager read path — use checked offset helpers and explicit length conversion so malformed AD1 item and metadata records fail safely instead of wrapping item-name, parent-folder, or metadata-data reads.
- Compute AD1 `reader_v2` decompression chunk-table entry addresses, chunk spans, or decompressed output capacity in `crates/ffx-ad1/src/operations_v2.rs` with raw `item.zlib_metadata_addr + ((index + 1) * ZLIB_CHUNK_ADDR_SIZE)`, `chunk_addrs[i + 1] - chunk_start`, or `item.decompressed_size as usize` arithmetic — use checked address math, checked subtraction, and checked capacity conversion so malformed AD1 compressed metadata returns `InvalidFormat` instead of wrapping chunk-table offsets, chunk lengths, or destination allocation size.
- Compute EWF section walker bounds in `crates/ffx-ewf/src/parser/mod.rs` with raw `offset + SECTION_HEADER_SIZE` or `offset + section_size` arithmetic — use checked addition so malformed section metadata cannot wrap the parser before the next header read.
- Parse split-archive suffixes in `crates/ffx-containers/src/segments.rs` without slicing the last four characters directly — use checked tail extraction so short or malformed filenames cannot panic while ZIP or RAR segment detection and basename grouping run.
- Compute 7z next-header absolute offsets in `crates/ffx-archive/src/sevenz.rs` with raw `32 + next_offset_relative` arithmetic — use checked addition so malformed 7z metadata cannot wrap the seek target before next-header inspection runs.
- Compute 7z multi-file seek targets or segment ends in `crates/ffx-archive/src/sevenz.rs` with raw `(self.total_size as i64 + p) as u64`, `(self.total_pos as i64 + p) as u64`, or `cumulative + size` arithmetic — use checked seek-target resolution and checked segment-end math so malformed or negative split-archive seeks cannot wrap reader positions across 7z segments.

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
cd src-tauri && cargo test  # Run backend tests
```

---

## Pre-built Native Libraries

CORE-FFX depends on three native C libraries that are compiled to static libraries and committed to the repo. CI workflows build them for all platforms; local dev uses the macOS ARM64 prebuilts or system libraries.

### Library Inventory

| Library | Purpose | macOS (local dev) | Linux (CI) | Windows (CI) |
|---------|---------|-------------------|------------|--------------|
| **libarchive** | Archive reading (ZIP, 7z, TAR, RAR, ISO, etc.) | `patches/libarchive2-sys/prebuilt/macos-arm64/libarchive.a` (1.3 MB) | `prebuilt/linux-x64/libarchive.a` (1.9 MB) | `prebuilt/windows-x64-msvc/archive.lib` (2.5 MB) + compression libs |
| **libewf** | EWF forensic image creation via C FFI | `libewf-ffi/prebuilt/macos-arm64/libewf.a` (6.2 MB) or Homebrew pkg-config | `prebuilt/linux-x64/libewf.a` (4.0 MB) | `prebuilt/windows-x64-msvc/ewf.lib` (19.9 MB, merged static) |
| **sevenzip-ffi** | 7z archive creation (LZMA SDK 24.09) | `sevenzip-ffi/prebuilt/macos-arm64/lib7z_ffi.a` (384 KB) or `sevenzip-ffi/build/lib7z_ffi.a` | `prebuilt/linux-x64/lib7z_ffi.a` (580 KB) | `prebuilt/windows-x64-msvc/7z_ffi.lib` (716 KB) |

### Directory Structure

```text
libewf-ffi/prebuilt/
  ├── macos-arm64/libewf.a              # macOS ARM64 static lib
  ├── linux-x64/libewf.a               # Linux x64 static lib
  └── windows-x64-msvc/
      ├── ewf.lib                       # 19 sub-libraries merged via lib.exe
      ├── zlib.lib                      # Static zlib (vcpkg x64-windows-static)
      └── bz2.lib                       # Static bzip2 (vcpkg x64-windows-static)

sevenzip-ffi/
  ├── build/lib7z_ffi.a                 # Local dev build (macOS only)
  └── prebuilt/
      ├── macos-arm64/lib7z_ffi.a
      ├── linux-x64/lib7z_ffi.a
      └── windows-x64-msvc/7z_ffi.lib

patches/libarchive2-sys/prebuilt/
  ├── macos-arm64/libarchive.a
  ├── linux-x64/libarchive.a
  └── windows-x64-msvc/
      ├── archive.lib
      ├── zlib.lib, bz2.lib, lzma.lib, lz4.lib, zstd.lib
```

### Build Script Discovery Order

Each library's `build.rs` follows a priority chain:

**libewf-ffi/build.rs:**
1. `LIBEWF_DIR` env var → link directly (CI sets this to `prebuilt/windows-x64-msvc/`)
2. pkg-config → system-installed libewf (Homebrew on macOS dev)
3. `prebuilt/<platform>/` directory → CI-built static libs
4. Common library paths (`/opt/homebrew/lib`, `/usr/local/lib`)
5. Stub fallback → compiles `stub.c` (EWF C-library features disabled; pure-Rust reader still works)

**sevenzip-ffi/build.rs:**
1. `prebuilt/<platform>/` directory → CI-built static libs
2. `build/lib7z_ffi.a` (local macOS dev build, only when target == host)
3. Stub fallback → compiles `stub.c` (7z features return errors at runtime)

**patches/libarchive2-sys/build.rs:**
1. `prebuilt/<platform>/` directory → CI-built static libs (macOS checks `macos-arm64` then `macos-universal`)
2. Build from source via CMake (requires system libarchive + compression libs)
3. Stub fallback for Windows cross-compilation

**macOS libarchive linking rule:** `patches/libarchive2-sys/build.rs` must prefer static Homebrew archives (`libb2.a`, `liblzma.a`, `libzstd.a`, `liblz4.a`) when they exist. Homebrew's `liblz4.dylib` install name resolves to `@executable_path/../lib/liblz4.1.dylib`, which causes `cargo run` / `npm run tauri dev` to crash at launch if linked dynamically into `target/debug/core-ffx`.

**macOS release workflow rule:** The release workflow must install Homebrew `libb2` before the macOS Tauri build, even when `patches/libarchive2-sys/prebuilt/macos-universal/libarchive.a` is present. The prebuilt libarchive link still resolves `-lb2`, and the GitHub macOS runner does not provide it by default.

### Rebuilding Pre-built Libraries

Use the `prebuild-native-deps.yml` workflow to rebuild libraries:

```bash
# Rebuild all platforms (creates PR with updated .a/.lib files)
gh workflow run prebuild-native-deps.yml \
  -f platform=all \
  -f build_libarchive=true \
  -f build_sevenzip=true \
  -f build_libewf=true \
  -f create_pr=true

# Rebuild Windows only
gh workflow run prebuild-native-deps.yml \
  -f platform=windows \
  -f build_sevenzip=true \
  -f build_libewf=true \
  -f create_pr=true

# Rebuild local sevenzip-ffi (macOS dev)
cd ~/GitHub/sevenzip-ffi
rm -rf build && mkdir build && cd build
cmake -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release ..
make -j$(sysctl -n hw.ncpu)
cp build/lib7z_ffi.a ~/GitHub/CORE-1/sevenzip-ffi/build/lib7z_ffi.a
```

### Windows Build Invariants

- **libewf `ewf.lib`** is a merged static library (19 sub-projects built with `/p:ConfigurationType=StaticLibrary` then merged via `lib.exe`). It must NOT be a DLL import library.
- **`ZLIB_DLL` must be stripped** from all libewf `.vcxproj` files before building. Without this, libewf expects `__declspec(dllimport)` on zlib symbols (`__imp_compress2`, etc.), which fails to link against static `zlib.lib`.
- **sevenzip-ffi** uses `portable_aligned_alloc()` / `portable_aligned_free()` macros instead of C11 `aligned_alloc()` — MSVC does not provide `aligned_alloc`. The macros map to `_aligned_malloc()` / `_aligned_free()` on Windows.
- **Windows compression deps** (zlib, bzip2) come from vcpkg `x64-windows-static` triplet, not DLL versions.

### Do NOT

- Use DLL versions of zlib/bzip2 when building libewf on Windows — `__imp_` link errors will result
- Use C11 `aligned_alloc()` in sevenzip-ffi C code — use `portable_aligned_alloc()` macro from `encryption_aes.c`
- Remove the `ZLIB_DLL` stripping step from `prebuild-native-deps.yml` — it prevents `__imp_compress2` link failures
- Build libewf as a DLL on Windows — the resulting import library is tiny (~72 KB) and won't contain actual code
- Use `sevenzip-ffi/build/lib7z_ffi.a` on Linux CI — it may contain macOS objects (build.rs guards against this)
- Revert `patches/libarchive2-sys/build.rs` to dynamic `lz4` linking on macOS — Homebrew's `liblz4.dylib` install name points at `@executable_path/../lib/liblz4.1.dylib`, which breaks `tauri dev`
- Put `[profile.*]` or duplicate `[patch.crates-io]` sections back into `src-tauri/Cargo.toml` — workspace-level Cargo settings must live in the root `Cargo.toml` or they are ignored with warnings

---

## CI/CD Workflows

### Release Workflow (`.github/workflows/release.yml`)

Triggered by tag push (`v*`) or manual `workflow_dispatch`. Produces signed installers for all 3 platforms for **CORE-FFX only**. CORE-Acquisition builds and releases are owned by the standalone CORE-ACQ repo.

**Jobs (5, sequential dependencies):**

```text
create-release → build-macos ─┐
                 build-linux ──┼→ publish-release
                 build-windows ┘
```

Each platform build job builds the CORE-FFX app once using the default feature set.

| Job | Runner | Outputs |
|-----|--------|---------|
| **Create Release** | `ubuntu-latest` | Draft GitHub Release with changelog |
| **Build macOS** | `macos-latest` (ARM64) | `.dmg` (signed + notarized) |
| **Build Linux** | `ubuntu-22.04` | `.deb`, `.AppImage` |
| **Build Windows** | `windows-latest` | `.exe` (NSIS), `.msi` |
| **Publish Release** | `ubuntu-latest` | Marks release as non-draft, uploads `latest.json` |

**Release artifacts:**

| File | Platform | Edition |
|------|----------|---------|
| `CORE-FFX_<ver>_aarch64.dmg` | macOS ARM64 | FFX |
| `CORE-FFX_<ver>_amd64.deb` | Linux x64 | FFX |
| `CORE-FFX_<ver>_amd64.AppImage` | Linux x64 | FFX |
| `CORE-FFX_<ver>_x64-setup.exe` | Windows x64 (NSIS) | FFX |
| `CORE-FFX_<ver>_x64_en-US.msi` | Windows x64 (MSI) | FFX |
| `latest.json` | Updater manifest | FFX |

Acquire artifacts and manifests are now published only by the standalone CORE-ACQ repo.

**Updater manifest generation:**
- `generate_manifest()` shell function takes a product name prefix and output filename
- Called once: `generate_manifest "CORE-FFX" "latest.json"`
- Each manifest filters downloaded release assets by the product name prefix to find the correct `.sig` and updater bundle files

**Version bump checklist (before tagging):**
1. `src-tauri/tauri.conf.json` → `"version": "X.Y.Z"`
2. `src-tauri/Cargo.toml` → `version = "X.Y.Z"`
3. `package.json` → `"version": "X.Y.Z"`
4. Update `CHANGELOG.md` with release notes (add `## [X.Y.Z] - YYYY-MM-DD` entry)
5. Commit, then `git tag -a vX.Y.Z -m "Release vX.Y.Z"` and `git push origin vX.Y.Z`

**Auto-updated on release (by `publish-release` job):**
- `README.md` version badge → sed replaces `version-X.Y.Z-blue` pattern
- `CHANGELOG.md` `[Unreleased]` comparison link → updated to `compare/vX.Y.Z...HEAD`
- `CHANGELOG.md` version comparison link → added if missing (e.g., `[X.Y.Z]: .../compare/vPREV...vX.Y.Z`)
- Committed as `github-actions[bot]` with `[skip ci]` to avoid re-triggering workflows

### GitHub Secrets Required

| Secret | Purpose | Platform |
|--------|---------|----------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 Developer ID certificate | macOS |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 certificate | macOS |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Your Name (TEAM_ID)` | macOS |
| `APPLE_API_ISSUER` | App Store Connect API issuer UUID | macOS (notarization) |
| `APPLE_API_KEY` | App Store Connect API key ID | macOS (notarization) |
| `APPLE_API_KEY_CONTENT` | Full `.p8` private key file contents | macOS (notarization) |
| `TAURI_SIGNING_PRIVATE_KEY` | Ed25519 private key for update signing | All platforms |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the signing key (optional) | All platforms |
| `GH_UPDATE_TOKEN` | PAT (`contents:read`) for private repo update checks | All platforms |
| `GITHUB_TOKEN` | Auto-provided by GitHub Actions | All platforms |

Blank or whitespace-only optional release secrets must be treated as absent in `release.yml`; do not append them to `GITHUB_ENV` or emit `::add-mask::` for empty values.

### Prebuild Workflow (`.github/workflows/prebuild-native-deps.yml`)

Manual `workflow_dispatch` with inputs for platform selection and individual library toggles. Builds static native libraries and optionally creates a PR to commit them.

**Inputs:**

| Input | Default | Description |
|-------|---------|-------------|
| `platform` | `all` | `all`, `macos`, `linux`, `windows` |
| `build_libarchive` | `true` | Build libarchive static lib |
| `build_sevenzip` | `true` | Build sevenzip-ffi static lib |
| `build_libewf` | `true` | Build libewf static lib |
| `create_pr` | `false` | Create PR with built libraries |

**Jobs:**

| Job | Runner | Libraries Built |
|-----|--------|-----------------|
| `prebuild-macos` | `macos-14` (ARM64) | libarchive, sevenzip-ffi, libewf (from Homebrew source) |
| `prebuild-linux` | `ubuntu-22.04` | libarchive (CMake), sevenzip-ffi, libewf (from source tarball) |
| `prebuild-windows` | `windows-latest` | libarchive (vcpkg), sevenzip-ffi (CMake/VS2022), libewf (MSBuild static + lib.exe merge) |
| `create-pr` | `ubuntu-latest` | Downloads artifacts → commits → opens PR |

### Nightly Release Workflow (`.github/workflows/nightly.yml`)

Automated scheduled workflow that bumps the patch version and triggers a full release build. Runs daily at 3 AM UTC, or manually via `workflow_dispatch`.

**How it works:**
1. Checks for new commits since the last `v*.*.*` release tag (skips if none, unless `force: true`)
2. Bumps version in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`
3. Adds a `[X.Y.Z]` entry to `CHANGELOG.md`
4. Commits, creates `vX.Y.Z` tag, pushes both → triggers `release.yml`

**Manual dispatch inputs:**

| Input | Default | Description |
|-------|---------|-------------|
| `bump_type` | `patch` | `patch` (0.1.55→0.1.56) or `minor` (0.1.55→0.2.0) |
| `force` | `false` | Release even if no new commits since last tag |
| `dry_run` | `false` | Show what would happen without committing |

**Do NOT:**
- Change the tag format from `vX.Y.Z` — the existing `release.yml` triggers on `v*` tags
- Remove the "no new commits" check — it prevents empty releases from wasting CI minutes
- Add build steps to `nightly.yml` — it delegates to `release.yml` via tag push

### Key Workflow Files

| File | Purpose |
|------|---------|
| `.github/workflows/release.yml` | Release pipeline (tag push → build → sign → publish) |
| `.github/workflows/nightly.yml` | Scheduled nightly version bump + tag → triggers release.yml |
| `.github/workflows/prebuild-native-deps.yml` | Build static native libraries for all platforms |
| `.github/workflows/tests.yml` | CI tests (cargo test + cargo clippy) |
| `.github/workflows/performance.yml` | Performance benchmarks |

`tests.yml` must pin `ref: main` on every `actions/checkout` step that pulls `tmreyno/core-shared` as the sibling local dependency. Leaving the ref implicit makes `actions/checkout` query GitHub's default-branch API, which can fail under rate limiting before the frontend/build jobs even start.

### Do NOT

- Build libewf as a solution-level MSBuild — it produces DLL import libraries. Build individual projects with `/p:ConfigurationType=StaticLibrary`.
- Skip the `ZLIB_DLL` patching step when building libewf on Windows
- Remove `TAURI_SIGNING_PRIVATE_KEY` from release.yml — update signing will fail
- Remove `ref: main` from `tests.yml` `core-shared` checkout steps — the default-branch API lookup can fail under GitHub rate limiting and abort the build before dependency installation
- Add `check-updates` to project-dependent menu IDs — updates should work without a project
- Use `workflow_dispatch` for release builds in production — always use tag push (`v*`)
- Forget to delete the old GitHub Release and tag before re-creating after a failed build

---

## Key Documentation

| Document | Purpose |
|----------|---------|
| `.github/copilot-instructions.md` | **Primary source of truth** — architecture, invariants, "Do NOT" rules, guard documentation. **Must be updated with every architectural or behavioral change.** |
| `CODE_BIBLE.md` | Authoritative codebase map and glossary |
| `CRATE_API_NOTES.md` | **Third-party crate API reference — check before using any crate** |
| `FRONTEND_API_NOTES.md` | **SolidJS/TypeScript API reference — check before writing frontend code** |
| `docs/SEVENZIP_FFI_API_REFERENCE.md` | **sevenzip-ffi C API docs — dict sizes, UTF-8, SDK 24.09 details** |
| `src-tauri/src/README.md` | Backend module structure |
| `src/components/README.md` | Frontend component catalog |
| `src/hooks/README.md` | State management hooks reference |
| `src/styles/README.md` | Tailwind CSS styling guide |
| `docs/FORM_TEMPLATE_SYSTEM.md` | **JSON schema form system — templates, options, hooks, persistence** |

> **MANDATORY:** After completing any task that changes behavior, architecture, guards, entry points, types, or component contracts, update the relevant documentation files listed above **before considering the task complete.** See Rule 6 in AI Agent Error Prevention Rules for the full checklist.

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

2. **Folder structure created automatically on new project.** `handleProjectSetupComplete` calls `create_folders_from_template` after `createProject()` to ensure the standard forensic folder structure exists on disk. The template is **edition-aware**: the Acquire edition uses `acquire-folder-template.json` (4 folders: Evidence, Exports, Photos, Case.Notes), while the full edition uses `case-folder-template.json` (7 folders including Processed.Database and Case.Documents). This is idempotent — already-existing directories are not affected. If auto-discovery defaulted paths to the project root (no specific subdirectories found), the paths are updated to the template's role paths (evidence, processedDb, caseDocuments).

3. **Session restore must guard against project load.** `restoreLastSession()` runs async and may resolve after the user opens a project. Always guard with `if (lastSession && !projectManager.hasProject())` to avoid overwriting a freshly-set `scanDir`.

4. **Processed Database path needs a fallback.** The `processedDbPath` accessor in `App.tsx` must fall back to deriving from `processedDbManager.databases()` when `projectLocations()?.processed_db_path` is null/empty — otherwise older projects without a `locations` field won't show the "Processed Database" entry in the toolbar dropdown.

5. **Older projects: derive locations in step 1b.** When loading a `.cffx` without a `locations` field, `handleLoadProject` must derive locations from `cached_databases`, `loaded_paths`, and `case_documents_cache` and call `updateLocations()` so the toolbar dropdown is populated.

Key files: `src/hooks/project/projectSetup.ts` (handleProjectSetupComplete — folder creation + project setup), `src/hooks/project/projectHelpers.ts` (handleLoadProject), `src/App.tsx` (Toolbar props, session restore), `src/components/toolbar/toolbarHelpers.ts` (buildProjectLocations).

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

**User Confirm Modal:**
- `UserConfirmModal` (`src/components/project/UserConfirmModal.tsx`) shows on project open/create when `confirmUserOnProjectOpen` is true and user profiles exist
- Triggered by `createEffect` in App.tsx watching `projectManager.hasProject()` transition false→true
- Allows selecting active profile, shows profile details, and applies profile to preferences on confirm
- "Skip" closes without applying; "Open Settings" navigates to SettingsPanel

**Key files:** `src/App.tsx` (shell layout, signals), `src/components/Toolbar.tsx` (toolbar content), `src/components/StatusBar.tsx` (status bar), `src/components/QuickActionsBar.tsx` (quick actions), `src/components/wizard/ConfigureLocationsStep.tsx` (profile selector), `src/components/project/UserConfirmModal.tsx` (profile confirmation modal), `src/hooks/useFileManager.ts` (recursive scan signal), `src/hooks/useMenuActions.ts` (native menu bridge), `src-tauri/src/menu.rs` (native menu bar).

### Right Panel UI Standard

The right panel (`src/components/layout/RightPanel.tsx`) renders metadata, activity, linked data, and tree views. **All right-panel sub-components MUST follow the shared primitives** defined in `src/components/viewerMetadata/shared.tsx` to ensure visual consistency.

**Shared Primitives (import from `viewerMetadata/shared`):**

| Primitive | Purpose | Key Classes |
|-----------|---------|-------------|
| `CollapsibleGroup` | Collapsible section wrapper | `border-b border-border/30`, `text-2xs uppercase tracking-wider text-txt-muted font-medium`, ChevronDown/Right `w-3 h-3` |
| `MetadataRow` | Key-value row (required value) | `flex items-baseline gap-2 text-xs py-0.5`, label `w-20 text-txt-muted shrink-0` LEFT-aligned |
| `OptionalMetadataRow` | Key-value row (auto-hides when empty) | Same styles as `MetadataRow`, wraps in `<Show when={value}>` |
| `SectionHeader` | Non-collapsible heading | `text-2xs font-medium text-txt-muted uppercase tracking-wider` |
| `SummaryRow` | Icon + label + value stat row | `text-xs` (NOT text-sm), `bg-bg-secondary rounded` |
| `StatusBadge` | Draft/locked/voided badge | `inline-flex items-center px-2.5 py-1 text-sm leading-none font-medium rounded-full border`, status-colored |

**Layout Rules:**

| Element | Standard | Example |
|---------|----------|---------|
| Root container | `flex flex-col h-full bg-bg` | All panels must include `bg-bg` |
| Scrollable body | `flex-1 overflow-y-auto` | Content area below header/tabs |
| Content padding | `p-3 space-y-3` (sections) or `p-2 space-y-2` (lists) | Inside scroll container |
| Panel header | `flex items-center justify-between px-3 py-2 border-b border-border bg-bg-secondary` | Non-tabbed panels |
| Header title | `text-xs font-medium text-txt` (NOT `text-sm`) | Consistent with tab text |
| Tab header | `flex items-center border-b border-border bg-bg-secondary` | Tabbed panels (ViewerMetadata, LinkedData) |
| Active tab | `text-accent border-b-2 border-accent` | Tab button active state |
| Tab button | `px-3 py-2 text-xs font-medium transition-colors` | Both active and inactive |
| Empty state | `flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2` | Icon `w-8 h-8 opacity-30` + text |

**Typography Rules:**

| Element | Font Size | Weight | Letter Spacing | Color |
|---------|-----------|--------|----------------|-------|
| Section headers | `text-2xs` (10px) | `font-medium` | `tracking-wider` (NOT `tracking-wide`) | `text-txt-muted` |
| Key-value labels | `text-xs` (12px) | normal | normal | `text-txt-muted` |
| Key-value values | `text-xs` (12px) | normal | normal | `text-txt` |
| Summary row text | `text-xs` (12px, NOT `text-sm`) | normal / `font-medium` | normal | `text-txt` |
| Mono values | `font-mono text-compact` (11px) | normal | normal | `text-txt` |

**Label Width:** Always `w-20` (5rem / 80px). Labels are **LEFT-aligned** (NOT right-aligned). This applies to `MetadataRow`, `OptionalMetadataRow`, and any custom key-value rows.

**Collapse Icons:** Always use `ChevronDownIcon`/`ChevronRightIcon` from `../icons` (plain SVG), sized `w-3 h-3`. Do NOT use `HiOutlineChevronDown`/`HiOutlineChevronRight` heroicons in collapsible headers — those are for non-collapsible interactive elements.

**Exception:** `MetadataPanel` (hex metadata, `src/components/metadata/`) is exempt from these rules because it uses a unique 3-column grid layout (key / value / offset) with region-click navigation. It may maintain its own styling.

**Components using shared primitives:**
- `viewerMetadata/*Section.tsx` (10 files) — all use `CollapsibleGroup` + `MetadataRow` ✅
- `viewerMetadata/FileInfoTab.tsx` — uses `MetadataRow` ✅
- `LinkedDataPanel.tsx` — uses `CollapsibleGroup` + `OptionalMetadataRow` + `SectionHeader` + `SummaryRow` ✅
- `EvidenceCollectionSummaryPanel.tsx` — uses `OptionalMetadataRow` + `StatusBadge` ✅
- `activity-panel/SimpleActivityPanelComponent.tsx` — follows header/root pattern ✅

**Key files:**
- `src/components/viewerMetadata/shared.tsx` — all shared primitives (source of truth)
- `src/components/viewerMetadata/index.tsx` — barrel re-export
- `src/components/layout/RightPanel.tsx` — main switcher (6 view modes)
- `src/components/ViewerMetadataPanel.tsx` — tabbed viewer metadata

**Do NOT:**
- Define local `DetailSection`, `DetailRow`, `FieldRow`, or `SummaryRow` components in right-panel files — use the shared primitives
- Use `text-sm` (14px) for panel headers or summary rows — use `text-xs` (12px)
- Use `tracking-wide` for section headers — use `tracking-wider`
- Use right-aligned labels (`text-right`) — labels are left-aligned
- Use `w-24` for labels — use `w-20`
- Use arbitrary `text-[Xpx]` values for key-value text — use `text-xs` (12px) via `MetadataRow`/`OptionalMetadataRow`
- Use `HiOutlineChevronDown`/`HiOutlineChevronRight` in collapsible section headers — use `ChevronDownIcon`/`ChevronRightIcon`
- Omit `bg-bg` from root containers — it ensures consistent background
- Omit `bg-bg-secondary` from panel headers — it ensures visual separation
- Create new right-panel components without importing from `viewerMetadata/shared`

---

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

### Bookmarks & Notes UI

Bookmarks and notes are **universal annotation tools** available in all workspace modes. They are created from the file right-click context menu, the **text selection context menu** inside document viewers, and managed in the sidebar's "Bookmarks & Notes" panel.

**Entry points for creating bookmarks/notes:**
- **File right-click context menu**: `getFileContextMenuItems()` in `useAppActions.ts` includes "📑 Bookmark" and "📝 Add Note" items after the separator following Copy Name — these operate at the **file level**
- **Text selection context menu**: Select text inside any document viewer (PDF, Office, Email, Text, etc.), right-click → "📑 Bookmark Selection", "📝 Note from Selection", "🔍 Search for Selection" — these operate at the **text level** and store the selected text
- **Notes panel**: The "+" button in the Notes sub-tab creates standalone notes (not attached to a specific file)

**Text Selection Context Menu (`useTextSelectionMenu` hook):**

When text is selected inside a document viewer and the user right-clicks:
1. `useTextSelectionMenu` checks `window.getSelection()?.toString().trim()`
2. If text is selected → shows custom context menu with 3 actions + Copy
3. If no text is selected → browser default context menu passes through (no interference)

| Action | Behavior |
|--------|----------|
| **📑 Bookmark Selection** | Creates bookmark with truncated text as name (60 chars), full text in `notes` field, `{ selectedText, entryName }` in `context` field |
| **📝 Note from Selection** | Creates note with title "Selection from {filename}", selected text as `content` |
| **🔍 Search for Selection** | Sets `searchInitialQuery` signal → opens SearchPanel pre-filled with the selected text for cross-document search |
| **📋 Copy** | Copies selected text to clipboard via `navigator.clipboard.writeText()` |

**Data flow for text selection actions:**
1. `ContainerEntryViewer.tsx` wraps all sub-viewers in a div with `onContextMenu={selectionMenu.handleContextMenu}`
2. `useTextSelectionMenu` hook manages the context menu state via `createContextMenu()`
3. Action callbacks flow up via props: `ContainerEntryViewer` → `App.tsx` handler functions
4. App.tsx handlers call `projectManager.addBookmark()` / `projectManager.addNote()` / set `searchInitialQuery` signal
5. For search: `searchInitialQuery` signal flows through `AppModals` → `SearchPanel` → `SearchPanelComponent` → `useSearch.setQuery()`

**SearchPanel `initialQuery` support:**
- `SearchPanelProps` has optional `initialQuery?: string` and `onInitialQueryConsumed?: () => void`
- `SearchPanelComponent` has a `createEffect` that sets `search.setQuery(initialQuery)` when the panel opens with an initialQuery
- After consuming, calls `onInitialQueryConsumed()` to clear the signal (prevents re-triggering on panel reopen)

**Sidebar panel (left panel, "bookmarks" tab):**
- The panel has **two sub-tabs**: "Bookmarks" and "Notes", controlled by `bookmarkNotesTab` signal in `LeftPanelContent.tsx`
- The sidebar button badge shows the **combined count** of bookmarks + notes
- The sidebar button title is "Bookmarks & Notes"

**Bookmarks sub-tab** (`BookmarksPanel`):
- Displays all bookmarks with search, filter by type/color, sort
- Actions: navigate to file, edit (color, name, tags), remove
- Empty state directs users to right-click context menu

**Notes sub-tab** (`NotesPanel`):
- Displays all notes with search, filter by target type (file/artifact/database/case/general)
- Full CRUD: create, edit (title, content, priority, tags), remove
- `NoteEditDialog` modal with title, content textarea, priority selector (Low/Normal/High/Critical), comma-separated tags
- Notes are sorted by `modified_at` descending
- Priority colors: Low (green), Normal (blue), High (amber), Critical (red)

**Backend hooks:**
- `useBookmarks.ts` → `createBookmarkManager()` returns `{ addBookmark, updateBookmark, removeBookmark, clearBookmarks }`
- `useNotes.ts` → `createNoteManager()` returns `{ addNote, updateNote, removeNote }`
- Both persist to `.ffxdb` via `dbSync` (fire-and-forget)

**Key files:**

| File | Purpose |
|------|---------|
| `src/hooks/useTextSelectionMenu.ts` | Text selection context menu hook (bookmark/note/search from selected text) |
| `src/hooks/useAppActions.ts` | File-level context menu items (Bookmark + Add Note) in `getFileContextMenuItems()` |
| `src/hooks/project/useBookmarks.ts` | Bookmark CRUD + signal management |
| `src/hooks/project/useNotes.ts` | Note CRUD + signal management |
| `src/components/container-viewer/ContainerEntryViewer.tsx` | Wraps all sub-viewers with `onContextMenu` for text selection |
| `src/components/container-viewer/types.ts` | `onBookmarkSelection`, `onNoteFromSelection`, `onSearchSelection` callback props |
| `src/components/search/types.ts` | `initialQuery` and `onInitialQueryConsumed` on `SearchPanelProps` |
| `src/components/search/SearchPanelComponent.tsx` | Consumes `initialQuery` via `createEffect` |
| `src/components/bookmarks/BookmarksPanel.tsx` | Bookmark list with search/filter/edit/remove |
| `src/components/notes/NotesPanel.tsx` | Notes list with search/filter + create/edit/remove |
| `src/components/notes/NoteItem.tsx` | Individual note display row |
| `src/components/notes/NoteEditDialog.tsx` | Create/edit modal for notes |
| `src/components/notes/helpers.ts` | Icon/label/formatting utilities |
| `src/components/notes/types.ts` | Type definitions + NOTE_PRIORITIES |
| `src/components/layout/LeftPanelContent.tsx` | Sub-tabbed view (bookmarkNotesTab signal) |
| `src/components/layout/sidebar/Sidebar.tsx` | Combined badge count + "Bookmarks & Notes" title |
| `src/components/layout/AppModals.tsx` | Passes `searchInitialQuery` / `onSearchInitialQueryConsumed` to SearchPanel |
| `src/components/help/sections/BookmarksNotes.tsx` | Help documentation |
| `src/components/help/sections/Tutorial.tsx` | Tutorial Step 7: Bookmark & Annotate Findings |

**Do NOT:**
- Remove the "📑 Bookmark" or "📝 Add Note" items from `getFileContextMenuItems()` — they are the primary entry points for file-level annotations
- Remove `onContextMenu={selectionMenu.handleContextMenu}` from the viewer content wrapper in `ContainerEntryViewer.tsx` — it's the only entry point for text-level annotations
- Remove the `useTextSelectionMenu` hook or its "no text selected → pass through" behavior — without it, browser default context menu breaks
- Remove `initialQuery` / `onInitialQueryConsumed` from `SearchPanelProps` — "Search for Selection" depends on them
- Remove `searchInitialQuery` signal from App.tsx — it connects text selection to the search panel
- Remove the sub-tab switcher from LeftPanelContent's "bookmarks" panel — both BookmarksPanel and NotesPanel share this sidebar tab
- Change the sidebar badge back to bookmarks-only count — it must show combined bookmarks + notes
- Import icons directly from `solid-icons/hi` in notes components — use the centralized barrel export from `../icons`
- Gate bookmarks or notes by any workspace mode module — they are universal tools

---

### Archive Container Tree & Viewer Architecture

Archive containers (ZIP, 7z, TAR, GZ, RAR, DMG, ISO, etc.) use a **synthesized directory** approach because many archive formats (especially 7z) don't include explicit directory entries in their file listings.

**DMG and ISO are browsed through the archive interface, not the VFS mount flow.** DMG entry reads route through the HFS+ helper in `read_dmg_entry_bytes()`, and ISO tree listing uses the same archive metadata pipeline as ZIP/7z/RAR.

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
3. Preview mode → `container_extract_entry_to_temp` → `read_archive_entry_bytes()` extracts to temp → shows in the appropriate viewer
4. `read_archive_entry_bytes()` resolves synthetic single-file compressed entries, routes DMG reads through HFS+, and falls back from libarchive to native archive readers when needed

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
- `src-tauri/src/commands/container.rs` — archive-entry preview extraction routes through `read_archive_entry_bytes()`
- `src-tauri/src/archive/libarchive_backend.rs` — `LibarchiveHandler` unified backend for all archive formats

**Do NOT:**
- Bypass `synthesizeDirectories()` when listing archive root entries
- Add manual path filtering (e.g., `!path.includes('/')`) for archive root entries — use `archive.getArchiveRootEntries()`
- Assume archive entries always include explicit directory entries
- Route DMG or ISO through the VFS tree path in `EvidenceTree.tsx` — they are browsed through the archive interface
- Compute archive chunk slice ends with raw `start + size` arithmetic in `archive_read_entry_chunk` — use checked/saturating slicing because chunk sizes come from IPC and can overflow before the bounds check runs
- Use `allArchiveEntries()` directly for file/folder counts — use `allSynthesizedEntries()` which includes virtual directories

---

### Nested Containers Inside All Parent Types (VFS, AD1, Archive, Lazy Trees)

Nested container support allows container files (ZIP, 7z, AD1, E01, etc.) **inside** other containers to be expanded inline in the evidence tree. This works for **all four parent container types**: VFS (E01/Raw), AD1, Archive, and Lazy Tree containers (L01/UFED).

**Critical invariant: All four tree node types — `VfsTreeNode`, `Ad1TreeNode`, `ArchiveTreeNode`, and `LazyTreeNode` — MUST have nested container detection and expansion support.** Each detects container files via `isNestedContainerFile()`, renders expand icons, and delegates to the `useNestedContainers` hook. Removing nested container props from ANY of these tree nodes will break containers-inside-containers for that parent type.

**Backend extraction pipeline (`src-tauri/src/commands/archive/nested.rs`):**

`get_or_create_nested_temp()` extracts a nested file from its parent to a temp directory. It detects the parent container type by **file signature** (not extension) before falling through to archive extraction:

1. **E01 parent** (`ewf::is_ewf()`) → opens `EwfVfs`, reads file via VFS `file_size()` + `read()`, writes to temp
2. **Raw parent** (`raw::is_raw()`) → opens `RawVfs` (filesystem or raw), reads via VFS, writes to temp
3. **AD1 parent** (`ad1::is_ad1()`) → uses `ad1::read_entry_data()` to read entry, writes to temp
4. **L01 parent** (`ewf::is_l01_file()`) → reads the nested container by ltree offset via `EwfHandle.read_at()`, writes to temp
5. **UFED parent** (`ufed::is_ufed()`) → opens `UfedVfs`, reads file contents, writes to temp
6. **Archive parent** (else branch) → routes through `read_archive_entry_bytes()` for zip/7z/rar/tar/dmg/iso/compressed archive entries

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
- Lazy section → `LazyTreeNode` (lines ~365)
- AD1 section → `Ad1TreeNode` (lines ~414)

**Key files:**
- `src/components/EvidenceTree/nodes/VfsTreeNode.tsx` — VFS tree node with nested container support, includes `VfsNestedEntryNode`
- `src/components/EvidenceTree/nodes/Ad1TreeNode.tsx` — AD1 tree node with nested container support, includes `Ad1NestedEntryNode`
- `src/components/EvidenceTree/nodes/ArchiveTreeNode.tsx` — Archive tree node with nested container support, includes `NestedContainerEntryNode`
- `src/components/EvidenceTree/nodes/LazyTreeNode.tsx` — Lazy tree node with nested container support for L01/UFED trees, includes `LazyNestedEntryNode`
- `src/components/EvidenceTree/hooks/useNestedContainers.ts` — shared hook managing nested container state, caching, and IPC
- `src/components/EvidenceTree/containerDetection.ts` — `isNestedContainerFile()`, `getNestedContainerType()`, `NESTED_CONTAINER_EXTENSIONS`
- `src-tauri/src/commands/archive/nested.rs` — `get_or_create_nested_temp()`, `nested_container_get_tree()`, `nested_archive_read_entry_chunk()`
- `src-tauri/src/commands/container.rs` — `parse_nested_archive_path()`, nested archive extraction routing, and entry context classification

**Do NOT:**
- Remove nested container props (`isNestedExpanded`, `onToggleNested`, etc.) from `VfsTreeNode`, `Ad1TreeNode`, `ArchiveTreeNode`, or `LazyTreeNode`
- Remove nested container props from `PartitionNodeProps` — they must pass through to `VfsTreeNode`
- Remove the `isNestedContainer` memo or `isNestedContainerFile()` detection from any tree node
- Remove the E01/Raw/AD1/L01/UFED parent detection in `get_or_create_nested_temp()` — the `is_ewf`/`is_raw`/`is_ad1`/`is_l01`/`is_ufed` checks MUST run before the archive extension match
- Treat malformed nested archive paths with empty outer or inner `::` segments as valid nested extractions in `src-tauri/src/commands/container.rs` — route detection and extraction through `parse_nested_archive_path()` so inputs like `::file.txt` and `outer.zip::` fall back to normal archive handling instead of extracting an empty path component
- Compute nested archive chunk slice ends with raw `start + size` arithmetic in `nested_archive_read_entry_chunk()` — use checked/saturating slicing so oversized chunk requests degrade safely instead of overflowing
- Remove `VfsNestedEntryNode` from `VfsTreeNode.tsx` or `Ad1NestedEntryNode` from `Ad1TreeNode.tsx`
- Skip wiring `tree.nested.*` props when rendering `PartitionNode`, `LazyTreeNode`, or `Ad1TreeNode` in `EvidenceTree.tsx`

---

### Viewer-Inside-Container Pipeline

All file viewers (PDF, Office, Spreadsheet, Email, PST, Image, Database, Plist, Binary, Registry, DocumentViewer) work inside forensic containers (E01, Raw, L01, AD1, UFED, ZIP, TAR, 7z, DMG, ISO, etc.) through a unified extraction-then-render pipeline. This section documents the critical data flow to prevent regressions.

**End-to-end pipeline (user clicks file in tree → viewer renders):**

1. **Tree click** → `EvidenceTree.tsx` calls `props.onSelectEntry({ containerPath, entryPath, name, size, isDir, isVfsEntry, isArchiveEntry, dataAddr, ... })`
2. **Entry navigation** → `useEntryNavigation.handleSelectEntry()` calls:
   - `centerPaneTabs.openContainerEntry(entry)` → creates/focuses entry tab, sets CenterPane `viewMode("document")`
   - `setEntryContentViewMode("auto")` → triggers ContainerEntryViewer's reactive effect
3. **Auto-preview effect** → `ContainerEntryViewer.createEffect` detects `mode === "auto"`:
   - `shouldAttempt = true` (because `canPreview(name) || mode === "auto"`)
   - Calls `handlePreview()` → `setPreviewLoading(true)` → spinner renders
4. **Extraction** → `handlePreview()` invokes `container_extract_entry_to_temp`:
  - **Archive** (`isArchiveEntry: true`): `read_archive_entry_bytes()` → temp file
   - **VFS** (`isVfsEntry: true` or E01/Raw detected): `EwfVfs.read()` / `RawVfs.read()` with `entrySize` → temp file
  - **L01** (`ewf::is_l01_file()`): reads file bytes from ltree offsets via `EwfHandle.read_at()` → temp file
  - **UFED** (`ufed::is_ufed()`): `UfedVfs.read()` with file-size lookup fallback → temp file
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
- `src-tauri/src/commands/archive/extraction.rs` — shared archive entry reads, including DMG/ISO archive-style extraction
- `src-tauri/src/viewer/document/universal.rs` — `UniversalFormat` enum, `from_extension()`, `detect_by_magic()`, `viewer_type()`
- `src-tauri/src/viewer/document/commands.rs` — `detect_content_format` backend command

**Do NOT:**
- Remove any type guard from `canPreview()` — this gates whether auto-preview triggers
- Break the `else if` chain in the metadata `viewerType` assignment — each branch must use `else if`
- Remove `isPst` from `canPreview()` — PST files need it to trigger auto-preview
- Change `handleSelectEntry` to set `entryContentViewMode` to anything other than `"auto"` — `"auto"` is the universal trigger
- Assume `containerPath === entryPath` means "container entry" — it means "disk file" (no extraction needed)
- Force UFED lazy-tree selections to set `isArchiveEntry: true` in `EvidenceTree.tsx` — UFED top-level viewing relies on backend UFED detection, not archive routing
- Route L01 back through the VFS tree or VFS extraction path — L01 browsing is lazy-tree based and file reads use ltree offsets
- Remove the content detection fallback for unknown extensions — it enables magic-byte-based viewer routing

---

### Portable Mode (CORE Acquire Zero-Footprint Operation)

Portable mode enables CORE Acquire (AqX) to run as a **zero-footprint forensic tool** from USB or other removable media. When active, all writes (cache, temp files, logs, projects) are redirected to a `CoreAcquireData/` directory alongside the executable instead of the system's default locations. This is critical for field forensics where the host system must not be altered.

**Detection is runtime, not build-time.** The same Acquire binary works both installed and portable. Portable mode is detected once at startup and cached for the process lifetime.

### Detection Priority

1. **Marker file** (`portable.marker`): A file named `portable.marker` adjacent to the executable (or adjacent to the `.app` bundle on macOS). Highest priority — overrides all other checks.
2. **Removable media**: If the executable resides on a removable volume (USB drive, SD card), detected via `sysinfo::Disks` `is_removable()`.

### Directory Structure

When portable mode is active, all data is stored under `CoreAcquireData/` alongside the executable:

```text
<exe_dir>/
  ├── CORE-Acquisition.exe (or .app bundle)
  ├── portable.marker          (optional — forces portable mode)
  └── CoreAcquireData/
      ├── config/              — Preferences, examiner profiles
      ├── cache/               — WebView2 cache, preview thumbnails
      ├── temp/                — Extraction temp files, viewer cache
      ├── logs/                — Audit logs, session logs
      └── projects/            — Default project output directory
```

### macOS `.app` Bundle Handling

On macOS, the executable lives inside `Contents/MacOS/` within the `.app` bundle. `resolve_exe_dir()` walks up from `Contents/MacOS/` to find the `.app` parent directory, so `CoreAcquireData/` is created **alongside** the `.app` bundle, not inside it.

### Architecture

```text
Backend (Rust):
  src-tauri/src/commands/portable.rs
    ├── PORTABLE_CONFIG: OnceLock<Option<PortableConfig>>  — cached singleton
    ├── detect_portable_mode()    — priority: marker > removable
    ├── init_portable_mode()      — creates dirs, logs status (called in common_setup)
    ├── is_portable()             — public helper
    ├── get_config()              — public helper (returns Option<&PortableConfig>)
    ├── portable_temp_dir()       — returns temp dir path (portable or system)
    ├── portable_cache_dir()      — returns cache dir path (portable or system)
    └── Tauri commands: portable_get_status, portable_ensure_dirs

Frontend (TypeScript/SolidJS):
  src/api/portable.ts            — PortableConfig + PortableStatus interfaces, invoke wrappers
  src/hooks/usePortableMode.ts   — SolidJS hook, queries backend on mount, reactive signals
  src/App.tsx                    — calls usePortableMode(), threads to AcquireLayout
  src/components/acquire/
    ├── AcquireLayout.tsx        — View routing, isPortable + portableConfig + evidence data props
    └── AcquireDashboard.tsx     — 4-phase workflow dashboard (Project → Identify → Acquire & Package → Verify & Document), portable badge (green) + low-space warning banner
```

### Key Types

| Rust | TypeScript |
|------|-----------|
| `PortableConfig` { data_dir, config_dir, cache_dir, temp_dir, log_dir, projects_dir, detection_reason, volume_mount_point, has_sufficient_space, free_space_bytes } | `PortableConfig` { dataDir, configDir, cacheDir, tempDir, logDir, projectsDir, detectionReason, volumeMountPoint, hasSufficientSpace, freeSpaceBytes } |
| `PortableStatus` { is_portable, config } | `PortableStatus` { isPortable, config } |

### Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/commands/portable.rs` | Detection, path management, directory creation, Tauri commands |
| `src-tauri/src/commands/mod.rs` | `pub mod portable;` registration |
| `src-tauri/src/lib.rs` | `init_portable_mode()` in `common_setup()`; commands in both `run_full()` and `run_acquire()` |
| `src/api/portable.ts` | Frontend API — `getPortableStatus()`, `ensurePortableDirs()` |
| `src/hooks/usePortableMode.ts` | SolidJS hook — `isPortable()`, `config()`, `status()`, `ready()` |
| `src/components/acquire/AcquireDashboard.tsx` | Portable badge + low-space warning in dashboard top bar |
| `src/components/acquire/AcquireLayout.tsx` | View routing (dashboard/identify/export/verify/collection) + portable props threading |

### Path Redirection (Zero-Footprint Enforcement)

All temporary file, cache, log, and app database paths are redirected through portable-aware helpers. When portable mode is active, writes go to `CoreAcquireData/` subdirectories instead of system locations.

| Call Site | Original Path | Portable Redirect |
|-----------|--------------|-------------------|
| `cleanup_preview_cache` (system.rs) | `std::env::temp_dir()` | `portable_temp_dir()` |
| `container_extract_entry_to_temp` (container.rs) | `std::env::temp_dir()/core-ffx-preview` | `portable_temp_dir()/core-ffx-preview` |
| `create_thumbnail` (viewer_hint.rs) | `std::env::temp_dir()/core-ffx-thumbnails` | `portable_temp_dir()/core-ffx-thumbnails` |
| `archive_extract_entry` (extraction.rs) | `std::env::temp_dir()/core-ffx-nested` | `portable_temp_dir()/core-ffx-nested` |
| `get_or_create_nested_temp` (nested.rs) | `std::env::temp_dir()/core-ffx-nested` | `portable_temp_dir()/core-ffx-nested` |
| `audit_log_dir` (logging.rs) | `dirs::data_local_dir()/core-ffx/logs` | `portable_config.log_dir` |
| `get_db` (database.rs) | `dirs::data_local_dir()/com.ffxcheck.app/ffx.db` | `portable_config.config_dir/ffx.db` |

### UI Indicators

- **Portable badge**: Green `HiOutlineServer` icon + "Portable" text + free space in GB, shown in `AcquireDashboard` top bar
- **Low-space warning**: Yellow banner below top bar when `hasSufficientSpace === false` (< 100 MB free)

### Do NOT

- Change portable mode to build-time detection — it must be runtime so the same binary works installed and portable
- Store `PORTABLE_CONFIG` as mutable — it's `OnceLock` (set once at startup, immutable thereafter)
- Create `CoreAcquireData/` inside the `.app` bundle on macOS — it must be alongside the bundle
- Lower the minimum space threshold below 100 MB — that's the minimum for safe forensic operation
- Remove `init_portable_mode()` from `common_setup()` — path redirection must happen before database init
- Remove `portable_get_status` from either `run_full()` or `run_acquire()` command registration — both editions need it
- Gate portable mode by `isAcquireEdition()` on the backend — the backend detects it regardless of edition; the UI indicator is Acquire-only
- Use `std::env::temp_dir()` for temp files when portable mode is active — use `portable_temp_dir()` which falls back to system temp when not portable
- Use `dirs::data_local_dir()` for logs or app database without checking `get_config()` first — portable mode redirects these to `CoreAcquireData/`
- Add new `std::env::temp_dir()` call sites without routing through `portable_temp_dir()` — all temp paths must be portable-aware

---

### Acquisition Session System (Acquire Edition Only)

The Acquire edition uses a **lightweight `.acquisition.json` session file** instead of the full `.cffx` + `.ffxdb` dual-file project system. This eliminates SQLite overhead, enables zero-footprint field operation, and produces a human-readable JSON audit trail.

**Design principle:** The session file is a single JSON document that tracks all acquisition metadata, evidence collections, and activity within one portable file. Companion files (`.ffx-companion.json` + `.txt` log) remain completely unchanged — they use independent `invoke()` APIs, not DB-based writes.

### Architecture

```text
App.tsx
  ├── useAcquisitionSession()           # Session lifecycle (create/load/save/close)
  ├── showAcquireSessionDialog signal   # Controls StartSessionDialog visibility
  ├── acquireSessionWriter adapter      # Bridges sessionManager → AcquisitionSessionWriter interface
  ├── handleLoadSession()               # File dialog → sessionManager.load()
  └── handleCreateSession()             # StartSessionDialog → sessionManager.create()

AcquireLayout (sessionWriter prop)
  └── AcquireDashboard (sessionWriter prop)
      └── useAcquisitionRunner({ sessionWriter })  # Records acquisitions to session
```

### Session File Format (v1.0)

```json
{
  "version": "1.0",
  "caseNumber": "2024-001",
  "caseName": "Evidence Collection",
  "examiner": "Jane Analyst",
  "organization": "",
  "outputFolder": "/exports",
  "evidenceFolder": "/evidence",
  "sessionFilePath": "/path/to/2024-001_2025-01-15.acquisition.json",
  "createdAt": "2025-01-15T10:30:00.000Z",
  "modifiedAt": "2025-01-15T11:45:00.000Z",
  "acquisitions": [...],
  "collections": [...],
  "activity": [...],
  "systemInfo": { "hostname": "...", "username": "...", "os": "...", "drives": [...] }
}
```

### Edition Guards

Three locations have `isAcquireEdition()` guards to prevent `.ffxdb` writes in Acquire mode:

| File | Guarded Calls |
|------|---------------|
| `useAcquisitionRunner.ts` | `dbSync.insertExport()`, `dbSync.updateExport()` (2 calls — success + failure) |
| `companionHelper.ts` | `dbSync.upsertEvidenceCollection()`, `dbSync.upsertCollectedItem()` (in `startAcquisitionRecord`), `createEvidenceCollectionRecord()` (in `handleAcquisitionComplete`) |

### Key Types (`src/types/acquisitionSession.ts`)

| Type | Purpose |
|------|---------|
| `AcquisitionSession` | Top-level session with metadata arrays |
| `SessionAcquisitionRecord` | Per-acquisition: id, type, source, output, hashes, timing, case info |
| `SessionCollectionRecord` + `SessionCollectedItem` | Evidence collection tracking |
| `SessionActivityEntry` | Activity log entry (action + timestamp + details) |
| `SessionSystemInfo` + `SessionDriveSnapshot` | System identification data |

### Key Files

| File | Purpose |
|------|---------|
| `src/types/acquisitionSession.ts` | All session types, `ACQUISITION_SESSION_VERSION`, `createEmptySession()` |
| `src/hooks/acquire/useAcquisitionSession.ts` | Session CRUD hook: `create()`, `load()`, `close()`, mutation methods, debounced save (300ms) |
| `src/components/acquire/StartSessionDialog.tsx` | Modal dialog: Case Number, Case Name, Examiner, Output Folder |
| `src/hooks/acquire/useAcquisitionRunner.ts` | `AcquisitionSessionWriter` interface, session recording on acquisition complete/fail |
| `src/hooks/export/companionHelper.ts` | Edition guards on `dbSync` calls |
| `src/App.tsx` | Session hook instantiation, `acquireSessionWriter` adapter, `handleLoadSession()`, `handleCreateSession()`, AcquireLayout prop wiring |
| `src-tauri/src/commands/system.rs` | `read_text_file` command (session file I/O) |

### AcquisitionSessionWriter Interface

```typescript
interface AcquisitionSessionWriter {
  addAcquisition(record: SessionAcquisitionRecord): void;
  updateAcquisition(id: string, updates: Partial<SessionAcquisitionRecord>): void;
  addActivity(entry: { action: string; details?: string }): void;
}
```

### Do NOT

- Use `.ffxdb` or `dbSync` calls in Acquire edition — all persistence goes through the session JSON file
- Remove edition guards from `useAcquisitionRunner.ts` or `companionHelper.ts` — they prevent SQLite writes
- Change companion file behavior (`.ffx-companion.json` + `.txt` log) — they are independent of the session system
- Use `@tauri-apps/plugin-fs` for session file I/O — use `invoke("write_text_file")` and `invoke("read_text_file")` (plugin-fs is not installed)
- Remove `sessionWriter` prop from `AcquireLayoutProps` or `AcquireDashboardProps` — it's the bridge from App.tsx to the acquisition runner
- Open `showProjectWizard` in Acquire edition — use `showAcquireSessionDialog` instead (wired via `onNewProject` prop)
- Call `handleLoadProject()` in Acquire edition — use `handleLoadSession()` instead (wired via `onOpenProject` prop)

---

### CORE Acquire Edition UI Architecture

The Acquire edition (`VITE_EDITION=acquire`) replaces the full CORE-FFX three-panel layout with a streamlined, card-based dashboard. The `AcquireLayout` component manages view routing between 7 views.

**AcquireView type:** `"dashboard" | "identify" | "export" | "browse" | "verify" | "collection" | "triage"`

**View routing:**

| View | Component / Mechanism | Description |
|------|----------------------|-------------|
| `dashboard` | `AcquireDashboard` | 4-phase workflow dashboard: ① Project (new/open), ② Identify (browse, triage, memory + system/drives), ③ Acquire & Package (physical, logical, export), ④ Verify & Document (verify, collection) |
| `identify` | `AcquireIdentifyView` | Dedicated identify-system flow: captures system stats, detected volumes, evidence item folder, exposes the shared drive browser with right-side `SystemInfoPanel`, and provides the explicit handoff into Evidence Collection |
| `export` | `AcquireExportView` → `ExportPanelComponent` | Unified acquire/export workflow with mode tabs and a source browser; uses a flat section-based shell rather than nested panel-inside-panel cards |
| `browse` | App.tsx three-panel FFX layout | **Renders the real three-panel layout** (sidebar + center + right panel) — NOT a placeholder |
| `verify` | `AcquireVerifyView` | Hash verification with batch_hash parallel progress bars, per-file progress, throughput stats, and minimal header chrome |
| `collection` | Lazy `EvidenceCollectionPanel` | Inline evidence collection form (not a center-pane tab) |
| `triage` | `AcquireTriageView` | Standalone quick triage flow with profile selection, live progress, and result summary |

**Browse mode architecture:** When `acquireView === "browse"`, App.tsx renders the full three-panel layout (sidebar, center pane, right panel) instead of the AcquireLayout. The AppHeader, Toolbar, and QuickActionsBar are shown only in browse mode. A "← Dashboard" back bar appears above the layout. A `createEffect` auto-opens the sidebar when entering browse mode.

**AppHeader/Toolbar/QuickActionsBar gating:** These components are wrapped in `<Show when={!isAcquireEdition() || acquireView() === "browse"}>` so they only render in the full FFX edition OR when Acquire browse mode is active.

**Keyboard shortcuts (Acquire edition):** `Cmd+1` through `Cmd+6` switch views: 1=dashboard, 2=identify, 3=export, 4=browse, 5=verify, 6=collection. Handled in `useKeyboardHandler.ts` when `isAcquireEdition()` is true.

**Evidence Collection:** The collection view renders `AcquireCollectionView`, which wraps `EvidenceCollectionPanel` in the shared Acquire process shell and threads through `caseNumber`, `discoveredFiles`, and `fileInfoMap` from the parent. It can open either a fresh collection form or a specific saved collection in review/edit mode from dashboard summaries. It does NOT use `centerPaneTabs.openEvidenceCollection()` — that creates a center-pane tab which doesn't render in Acquire mode.

**Identify handoff:** `AcquireIdentifyView` must provide an explicit next-step transition into the collection workflow once system data has been captured. Do NOT rely on users returning to the dashboard to find the next step after Identify finishes.

**Acquire process-shell rule:** `AcquireExportView`, `AcquireVerifyView`, and `AcquireTriageView` are already rendered inside the Acquire layout shell, so their internal content should use flat sections and lightweight callouts. Do NOT reintroduce old desktop-style nested `card`/toolbar framing around every process step or mode body — the workflow should read as one continuous surface with only the outer Acquire view providing structure.

**Acquire export routing rule:** `AcquireExportView` passes `hideTriageMode` to `ExportPanelComponent` so the export panel does not expose a second triage entry point. In the Acquire edition, triage lives on the dedicated `AcquireTriageView` route; the shared export panel should expose physical, logical, AFF4, native export, memory, and tools only.

**Key files:**

| File | Purpose |
|------|---------|
| `src/App.tsx` | Edition gating, browse mode layout, acquireView/acquireExportMode signals, sidebar auto-open effect |
| `src/components/acquire/AcquireLayout.tsx` | View routing (dashboard/identify/export/verify/collection), props threading, lifted Identify state, `pendingVerifyFiles` signal for quick-hash flow |
| `src/components/acquire/AcquireDashboard.tsx` | 4-phase workflow dashboard launcher (Project → Identify → Acquire & Package → Verify & Document) + portable badge + low-space warning + recent acquisitions history + recent projects list + quick verify button |
| `src/components/acquire/AcquireIdentifyView.tsx` | Dedicated identify-system flow with system scan, evidence-folder creation, shared drive browser, and right-side system info panel |
| `src/components/acquire/AcquireCollectionView.tsx` | Dedicated Acquire collection shell that wraps `EvidenceCollectionPanel` and optional `SystemInfoPanel` |
| `src/components/acquire/AcquireExportView.tsx` | Wraps ExportPanel with back nav + `initialMode` + `pendingExportMode` props; wires DriveTreeBrowser `onAcquireSource`; provides the outer Acquire flow shell |
| `src/components/acquire/AcquireVerifyView.tsx` | Hash verification with batch_hash, per-file progress bars, `initialFiles` prop for quick-hash pre-population, and reduced header chrome |
| `src/components/acquire/AcquireTriageView.tsx` | Standalone quick triage workflow with flat sections and result summary |
| `src/components/acquire/AcquireProcessShell.tsx` | Shared process-shell wrapper (back button, title bar, `headerActions` slot) used by all non-dashboard Acquire views |
| `src/components/acquire/acquire.css` | Acquire-only shell/layout styles including flat process sections and minimal workflow headers |
| `src/utils/edition.ts` | `isAcquireEdition()`, `isFullEdition()` |

**Do NOT:**
- Re-add `AcquireImageWizard.tsx`, `AcquireSourcePanel.tsx`, or `AcquireProgressView.tsx` — these are dead code (deleted), replaced by the unified ExportPanel
- Re-add `"imaging"` or `"progress"` to the `AcquireView` type — the imaging wizard flow was replaced by ExportPanel modes
- Use `centerPaneTabs.openEvidenceCollection()` in Acquire mode — it creates a center-pane tab that doesn't render; use the inline `EvidenceCollectionPanel` instead
- Remove the `createEffect` on `acquireView` that opens the sidebar — browse mode needs the sidebar visible
- Remove the `<Show when={!isAcquireEdition() || acquireView() === "browse"}>` from AppHeader/Toolbar — it prevents redundant UI in Acquire dashboard/export/verify/collection views
- Add a placeholder `<div>` for Browse Evidence — browse mode must render the real three-panel FFX layout
- Pass `browseContent` prop to AcquireLayout — browse is handled by App.tsx directly, not by AcquireLayout
- Reintroduce old desktop-style nested panel chrome inside Acquire process views — use flat sections/callouts inside the Acquire shell instead

**Acquire Project Setup (Simplified):**

The Acquire edition uses a **simplified project setup wizard** that hides fields irrelevant to field acquisition. When `isAcquireEdition()` is true:

| What Changes | Full Edition | Acquire Edition |
|---------|-------------|-----------------|
| **Folder template** | `case-folder-template.json` (7 folders, 3 roles) | `acquire-folder-template.json` (4 folders: Evidence, Exports, Photos, Case.Notes; 1 role: evidence) |
| **ConfigureLocationsStep** | Shows all 6 sections (Project Info, Case ID, Workspace Profile, Evidence, Processed DB, Case Documents) | Hides 3 sections: Workspace Profile, Processed Databases, Case Documents |
| **Auto-discovery scanning** | Scans for evidence + processed DB + case doc directories | Only scans for evidence directories (skips processed DB and case doc scanning) |
| **Processed DB handling** | Discovers and adds to `processedDbManager`, switches to processed tab | Skipped entirely |

Key files:
- `src/templates/project/acquire-folder-template.json` — minimal 4-folder template
- `src/components/wizard/ConfigureLocationsStep.tsx` — `isAcquireMode` prop gates Processed DB, Case Documents, Workspace Profile sections
- `src/components/project-wizard/useWizardState.ts` — skips processed DB/case doc scanning when `isAcquireEdition()`
- `src/components/project-wizard/ProjectSetupWizard.tsx` — passes `isAcquireMode={isAcquireEdition()}` to ConfigureLocationsStep
- `src/hooks/project/projectSetup.ts` — uses `acquireFolderTemplate` when `isAcquireEdition()`

**Do NOT:**
- Remove `isAcquireMode` prop from `ConfigureLocationsStep` — it gates the 3 hidden sections
- Use `acquire-folder-template.json` in the full edition — it only has the evidence role
- Add Processed DB or Case Documents scanning back to Acquire mode — these features are not in the Acquire feature set
- Remove the `acquireFolderTemplate` import from `useWizardState.ts` or `projectSetup.ts` — both need it for edition-aware template selection

### Acquire Edition Boundary Enforcement (CORE-ACQ Separation Prep)

The Acquire edition has moved to the standalone **CORE-ACQ** repository. The remaining acquire-specific code in CORE exists for boundary/reference work only and must not be reintroduced into CORE CI or release builds.

**Boundary manifest:** `docs/ACQUIRE_BOUNDARY.md` — the source of truth for what belongs in the Acquire build. Contains: Rust backend boundary (feature flags, command registration table), frontend file classification (acquire-only, shared, review-only), cross-boundary import catalog, CI build matrix, and separation readiness checklist.

**Boundary lint script:** `scripts/lint-acquire-boundary.sh` — automated validation with 4 checks:
1. **No review-only imports** in acquire files (search, dedup, processed, report, workspace_profile patterns)
2. **All cross-boundary imports registered** in the `ALLOWED_CROSS_BOUNDARY` list (prevents accidental new dependencies)
3. **No review-only hook imports** from acquire hooks
4. **No review-only API imports** (dedup, projectMerge, search) from acquire files

**CI validation:** Standalone Acquire build validation now lives in the CORE-ACQ repository. The CORE repo should not run acquire release or build jobs.

**Allowed cross-boundary imports** (acquire files importing from non-acquire sibling directories):

| Import Path | Reason |
|-------------|--------|
| `../icons` | Shared icon barrel export |
| `../Toast` | Shared toast notifications (candidate for `@core-suite/components`) |
| `../viewerMetadata/shared` | Shared metadata primitives (candidate for `@core-suite/components`) |
| `../EvidenceCollectionPanel` | Shared evidence collection form |
| `../EvidenceTree/containerDetection` | Container type detection utilities |
| `../RecentProjectsList` | Shared recent projects UI |
| `../export-panel/DriveTreeBrowser` | Shared drive browser |
| `../preferences` | App preferences utilities |
| `../export/companionHelper` | Acquisition companion file helper |
| `../project/useProjectDbSync` | Database sync utilities |

**Key files:**

| File | Purpose |
|------|---------|
| `docs/ACQUIRE_BOUNDARY.md` | Boundary manifest — source of truth for acquire API surface |
| `scripts/lint-acquire-boundary.sh` | Automated boundary lint (run in CI and locally) |
| `src/utils/edition.ts` | `isAcquireEdition()`, `isFullEdition()`, `APP_NAME`, `APP_SHORT` |

**Do NOT:**
- Import review-only modules (`search/`, `dedup/`, `processed/`, `report/`, `workspace_profile*`) from any file under `src/components/acquire/` or `src/hooks/acquire/`
- Add new cross-boundary imports from acquire files without adding them to `ALLOWED_CROSS_BOUNDARY` in `scripts/lint-acquire-boundary.sh`
- Reintroduce CORE-Acquisition build or release jobs to `.github/workflows/tests.yml` or `.github/workflows/release.yml` — standalone acquire builds belong in CORE-ACQ
- Import review-only APIs (`dedup`, `projectMerge`, `search`) from acquire hooks or components
- Modify `run_acquire()` in `lib.rs` to include review-only commands (search, dedup, viewer::document, report, processed)

---

### Acquire & Export Panel Architecture (Unified)

The Export Panel (`src/components/export-panel/ExportPanelComponent.tsx`) is a **unified acquisition and export panel** shared by both the full CORE-FFX edition and the CORE Acquire edition. It provides eight forensic modes:

| Mode | Label | Component | Output Format | Backend |
|------|-------|-----------|---------------|---------|
| `"physical"` | Physical Image | `PhysicalImageMode.tsx` | E01 disk image or Raw (.dd) | `ewf_create_image` / `raw_create_image` |
| `"logical"` | Logical Image | `LogicalImageMode.tsx` | L01 logical evidence | `l01_create_image` (pure-Rust l01_writer) |
| `"aff4"` | AFF4 Image | `Aff4ImageMode.tsx` | AFF4 forensic container | `aff4_create_image` (pure-Rust ffx-aff4) |
| `"native"` | Export | `NativeExportMode.tsx` | 7z archive or file copy | `create_7z_archive` / `export_files` |
| `"tools"` | Tools | `ToolsMode.tsx` | — | Test/repair/validate archives |
| `"memory"` | Memory | `MemoryMode.tsx` | Raw `.mem` dump | `memory_capture` (live RAM capture) |
| `"triage"` | Triage | `TriageMode.tsx` | Collected artifacts + secret findings | `triage_collect` (forensic triage + credential scan) |

**Triage result semantics:** `TriageResult.filesSkipped` is the authoritative count for artifacts that could not be collected because they were inaccessible, oversized, or timed out in blocked I/O. Per-category `CategoryResult` must track skipped files separately from failed files so the UI does not label expected macOS permission/TCC skips as hard failures.

**Triage packaging:** `triage.rs` packages collected artifacts with `SevenZip::create_archive_streaming()` rather than `create_archive()` so packaging does not emit the in-memory archive warning for large staging directories.

**PhysicalImageMode format selector:** The Physical Image mode offers a radio-button toggle between E01 (EWF) and Raw (.dd) formats. E01 provides 5 format variants, compression, and embedded hash selection. Raw provides hash algorithm toggles (MD5/SHA-1/SHA-256) and segment size configuration.

The Acquire edition's `AcquireLayout` routes all imaging/export actions through this same panel via `AcquireExportView`, which wraps `ExportPanelComponent` with an `initialMode` prop. Physical and logical acquisition no longer use a separate wizard — they use the unified panel's Physical Image and Logical Image modes.

`AcquireExportView` also manages a `pendingMode` signal wired to `ExportPanel`'s `pendingExportMode` prop. When a user right-clicks a drive or file in the inline `DriveTreeBrowser` and selects an acquisition mode (E01/L01/Export), the `onAcquireSource` callback triggers `setPendingMode(mode)`, which the ExportPanel consumes to switch to the correct mode tab. The pending mode is cleared alongside pending sources in `onPendingSourcesConsumed`.

**Shared sub-components (`src/components/export-panel/` + `src/components/export/`):**

| Component | Purpose |
|-----------|---------|
| `ExportHeader.tsx` | Mode tab selector ("Acquire & Export" header with mode buttons including AFF4) |
| `ExportSourceSection.tsx` | Source file/folder picker + destination selector + inline drive tree |
| `DriveTreeBrowser.tsx` | Reusable inline drive/volume browser with lazy-loaded directory trees, 15-second auto-refresh for drive hot-plug detection, and right-click context menu for acquisition mode selection (E01/L01/Export) |
| `SplitSizeSelector.tsx` | Unified split/segment size dropdown (9 presets + Custom) |
| `CaseMetadataSection.tsx` | Collapsible case info (case number, evidence number, examiner, description, notes) |
| `DriveSelector.tsx` | Modal picker for system drives with read-only mount toggle |

#### Export Default Destination (Project Exports Folder)

When a project is open, the export destination auto-defaults to the project's Exports folder (`ProjectLocations.exports_path`). This is set during project creation from the folder template's `exports` role (mapped via `roleMapping`). The `initialDestination` prop flows through the component chain:

```text
projectManager.projectLocations()?.exports_path
  → App.tsx (initialDestination prop)
  → AcquireLayout → AcquireExportView → ExportPanel (Acquire edition)
  → ExportPanel (full edition, center-pane tab)
  → useExportState → useExportCommon → destination signal initial value
```

Both folder templates define the `exports` role:
- Full edition: `3.Exports.Results` (role: `"exports"`)
- Acquire edition: `2.Exports` (role: `"exports"`)

For older projects without `exports_path`, the destination starts empty (same as before — user must Browse).

#### Export Default Values (Forensic Standard)

All export modes default to **no compression** and **2 GB split size**. These defaults prioritize forensic integrity (bit-for-bit fidelity) and compatibility (FAT32/FTK Imager).

| Setting | E01 (Physical) | L01 (Logical) | 7z (Native) |
|---------|----------------|---------------|-------------|
| **Compression** | `"none"` | `"none"` | `CompressionLevel.Store` (0) |
| **Split/Segment Size** | 2048 MB | 2048 MB | 2048 MB |
| **Hash** | MD5 ✅, SHA1 ❌ | MD5 ✅ | SHA-256 ✅ |
| **Verify after create** | — | — | ✅ |

**AFF4 speed-first defaults:**

- Frontend default compression is `"stored"`
- Backend default compression falls back to `Aff4Compression::Stored` when omitted
- Small logical AFF4 files must honor Stored mode too; do not force deflate for ZIP-segment members when the selected compression is Stored

**NativeExportMode presets** (all use `CompressionLevel.Store`):

| Preset | Split Size | Solid | Notes |
|--------|-----------|-------|-------|
| Standard | 2048 MB | ❌ | General forensic export |
| Court | 4096 MB | ❌ | Court submission (DVD-sized) |
| Transfer | 2048 MB | ❌ | USB/cloud transfer |
| Long-term | 2048 MB | ❌ | Archival |
| Custom | 2048 MB | ❌ | User-configurable |

#### SplitSizeSelector Conventions

`SplitSizeSelector` (`src/components/export/SplitSizeSelector.tsx`) is the **sole UI for split/segment size selection** across all export modes. It replaces raw number inputs.

**`SPLIT_SIZE_OPTIONS` presets (values in MB):**

| Label | Value (MB) | Description |
|-------|-----------|-------------|
| No splitting | 0 | Single output file |
| 100 MB | 100 | Small transfers |
| 650 MB | 650 | CD-ROM |
| 700 MB | 700 | CD-R |
| 1 GB | 1024 | General purpose |
| 2 GB | 2048 | FAT32 / FTK default |
| 4 GB | 4096 | DVD / FAT32 limit |
| 4.7 GB | 4700 | DVD single-layer |
| 25 GB | 25600 | Blu-ray |

**Props:** `valueMb: Accessor<number>`, `setValueMb: Setter<number>`, optional `label` (default: "Split Size").

**MB → bytes conversion:** The frontend stores split values in **MB**. Backend APIs expect **bytes**. Conversion happens in `ExportPanel.tsx` when building options:
```tsx
// E01: ewfSegmentSize() is in MB → backend expects bytes
segment_size: ewfSegmentSize() > 0 ? ewfSegmentSize() * 1024 * 1024 : 0

// L01: l01SegmentSize() is in MB → backend expects Option<u64> bytes
segment_size: l01SegmentSize() > 0 ? l01SegmentSize() * 1024 * 1024 : undefined

// 7z: splitSizeMb() is in MB → backend expects bytes
split_size: splitSizeMb() > 0 ? splitSizeMb() * 1024 * 1024 : 0
```

#### Drive Source Selection & Read-Only Mounting

Users can select system drives as export sources (for physical/logical imaging). The `DriveSelector` modal enumerates drives via `list_drives` and offers an optional **read-only remount** toggle for forensic integrity.

**Safety invariants (enforced in backend `system.rs`):**
- **Write probe first (ground truth)**: `check_path_writable` MUST attempt a write probe (`File::create` + `remove_file`) BEFORE consulting `sysinfo::Disks` mount metadata. On macOS Catalina+, `/Users` is a firmlink to `/System/Volumes/Data/Users`. `sysinfo::Disks` reports `/` as read-only (sealed system volume) and `/System/Volumes/Data` as writable. Paths like `/Users/terryreynolds/...` start with `/` but NOT `/System/Volumes/Data`, so prefix matching picks the wrong mount and incorrectly reports read-only. The write probe bypasses this — if the probe file can be created, the path is writable regardless of what sysinfo reports. Mount metadata is ONLY used for descriptive error messages when the probe fails.
- **Virtual drives filtered out**: `/dev`, `devfs`, `tmpfs`, etc. are excluded from `list_drives`
- **System disk marked**: `isSystemDisk: true` for boot volumes — UI shows warning
- **Boot volume protection**: `remount_read_only` refuses to remount `/` (macOS boot volume)
- **Overlap checks**: Backend validates no source path overlaps with destination
- **Space checks**: Backend validates destination has sufficient free space (via `libc::statvfs`)
- **Mount state tracking**: `ORIGINAL_MOUNT_STATE` (LazyLock HashMap) records pre-imaging mount state
- **Guaranteed restoration**: `restoreAllDriveMounts()` in `ExportPanel.handleStart().finally()` restores all drives

**Frontend drive state signals (in `ExportPanel.tsx`):**
```tsx
const [driveSources, setDriveSources] = createSignal<Set<string>>(new Set());
const [mountDrivesReadOnly, setMountDrivesReadOnly] = createSignal(false);
```

#### Sidebar Drive Panel — Bidirectional Sync with Export

The **DriveSourcePanel** (`src/components/drives/DriveSourcePanel.tsx`) is a left sidebar panel (tab: `"drives"`, gated by `reportExport` module) that lets users browse drives/volumes and select sources for export. It uses **live bidirectional sync** — checking/unchecking items in the drive panel instantly adds/removes them from the Export Panel's source list.

**Data flow (add):**
1. User checks a file/folder in DriveSourcePanel → `toggleSelect()` calls `props.onSourceAdd(path)`
2. `App.tsx` `handleSourceAdd(path)` appends path to `pendingDriveSources` signal, auto-opens the Export tab on first add
3. `ExportPanelComponent.tsx` `createEffect` watches `pendingDriveSources`, calls `state.handleAddDriveSource(path)` in `untrack()`, then `onPendingDriveConsumed()`

**Data flow (remove):**
1. User unchecks a file/folder in DriveSourcePanel → `toggleSelect()` calls `props.onSourceRemove(path)`
2. `App.tsx` `handleSourceRemove(path)` filters from `pendingDriveSources` + appends to `pendingRemoveSources` signal
3. `ExportPanelComponent.tsx` second `createEffect` watches `pendingRemoveSources`, calls `state.removeSourceByPath(path)` in `untrack()`, then `onPendingRemoveConsumed()`

**Key behaviors:**
- **Auto-send**: Selecting files/folders via file dialogs in DriveSourcePanel automatically calls `onSourceAdd` for each new path — no Export button click required
- **Clear All**: The "Clear" action iterates `props.onSourceRemove` for each selected path before clearing internal selection state
- **View button**: The selection bar shows "N in export" with a "View" button that focuses the Export tab (via `props.onExportSources([])`)
- **Toast suppression**: Single auto-adds suppress the toast notification; toasts only fire for 2+ items added at once

**Props on DriveSourcePanel:**
```tsx
onSourceAdd?: (path: string) => void;    // Called on check / file dialog add
onSourceRemove?: (path: string) => void; // Called on uncheck / clear
onExportSources: (paths: string[]) => void; // Focus export tab
```

**Signals in App.tsx:**
```tsx
const [pendingDriveSources, setPendingDriveSources] = createSignal<string[]>([]);
const [pendingExportMode, setPendingExportMode] = createSignal<...>(null);
const [pendingRemoveSources, setPendingRemoveSources] = createSignal<string[]>([]);
```

**Key files:**
- `src/components/drives/DriveSourcePanel.tsx` — sidebar drive/volume browser with live sync callbacks
- `src/components/export-panel/ExportPanelComponent.tsx` — main composition component (unified panel)
- `src/components/export-panel/ExportHeader.tsx` — mode tab selector with "Acquire & Export" header
- `src/components/export-panel/ExportSourceSection.tsx` — source file/folder picker + inline DriveTreeBrowser
- `src/components/export-panel/DriveTreeBrowser.tsx` — reusable inline drive/volume browser with lazy-loaded directory trees, 15-second auto-refresh polling for drive hot-plug detection, and right-click context menu (`onAcquireSource` prop) for selecting acquisition mode (E01 Physical / L01 Logical / Export 7z)
- `src/components/export/SplitSizeSelector.tsx` — shared split size dropdown
- `src/components/export/CaseMetadataSection.tsx` — shared case metadata inputs
- `src/components/export/DriveSelector.tsx` — modal picker for system drives with read-only mount toggle
- `src/components/export/PhysicalImageMode.tsx` — E01 creation UI
- `src/components/export/LogicalImageMode.tsx` — L01 creation UI
- `src/components/export/Aff4ImageMode.tsx` — AFF4 creation UI (image name, compression, multi-select hash, case metadata)
- `src/components/export/NativeExportMode.tsx` — 7z/file export UI with forensic presets
- `src/components/export/ToolsMode.tsx` — archive test/repair/validate UI
- `src/components/export/MemoryMode.tsx` — live RAM capture UI (info, options, progress, results)
- `src/components/ExportPanel.tsx` — orchestrator (state, conversion, IPC)
- `src-tauri/examples/logical_container_benchmark.rs` — terminal-runnable logical container benchmark (7z Store, AFF4 Stored, L01 None + reopen + output hashing)
- `src/components/acquire/AcquireExportView.tsx` — Acquire edition wrapper (passes `initialMode` + `pendingExportMode` to ExportPanel); wires DriveTreeBrowser's `onAcquireSource` to set `pendingMode` signal
- `src/components/acquire/AcquireLayout.tsx` — Acquire edition root layout (routes physical/logical to unified panel)
- `src/hooks/export/useNativeExportState.ts` — native file export + 7z archive handlers with DB tracking
- `src/hooks/export/useL01ExportState.ts` — L01 logical evidence handler with DB tracking
- `src/hooks/export/useAff4ExportState.ts` — AFF4 export state management with progress, companion files, DB tracking
- `src/hooks/export/useRawExportState.ts` — Raw export state management with hash toggles, segment size, companion files
- `src/hooks/export/useExportCommon.ts` — shared export state (sources, destinations, drive handling, `removeSourceByPath`)
- `src/hooks/export/useMemoryDumpState.ts` — memory capture state/handler hook with DB tracking
- `src/api/drives.ts` — DriveInfo/MountResult types, listDrives(), remountReadOnly(), restoreMount()
- `src/api/ewfExport.ts` — E01 export API
- `src/api/l01Export.ts` — L01 export API
- `src/api/aff4Export.ts` — AFF4 export API (createAff4Image, cancelAff4Export, progress listener)
- `src/api/rawExport.ts` — Raw export API (createRawImage, cancelRawExport, buildRawExportOptions)
- `src/api/exportHistory.ts` — Export history CRUD (getExportHistory, deleteExportRecord)
- `src/api/fileExport.ts` — CopyResult (includes `operationId`), CopyProgress, ExportOptions
- `src/api/memory.ts` — MemoryCaptureInfo/Progress/Result types, captureMemory(), cancelMemoryCapture()
- `src/api/triage.ts` — TriageCategory/Profile/Options/Progress/SecretFinding/Result types, triageCollect(), triageCancel(), listenTriageProgress()
- `src/hooks/export/useTriageState.ts` — Triage collection state/handler hook with DB tracking
- `src/components/export/TriageMode.tsx` — Triage mode UI (profile selection, category toggles, secrets scan, results)
- `src-tauri/src/commands/triage.rs` — Forensic triage collection + credential/secret scanning (platform-specific artifacts, 30+ secret patterns)
- `src-tauri/src/commands/system.rs` — list_drives, remount_read_only, restore_mount
- `src-tauri/src/commands/ewf_export.rs` — ewf_create_image (+ walk_dir_files for folder support)
- `src-tauri/src/commands/aff4_export.rs` — aff4_create_image, aff4_cancel_export (pure-Rust ffx-aff4, Deflate/LZ4/Snappy/Stored, 5 hash algos, RDF metadata)
- `src-tauri/src/commands/raw_export.rs` — raw_create_image, raw_cancel_export (byte-for-byte copy, segmentation, concurrent MD5/SHA-1/SHA-256, boot volume safety)
- `src-tauri/src/commands/l01_export.rs` — l01_create_image (+ walk_dir_into_writer for folder structure)
- `src-tauri/src/commands/export.rs` — export_files, cancel_export with unique operation_id manifest naming, per-operation AtomicBool cancel flags, conditional hashing, destination free space check

#### Export DB Tracking

All export operations (E01, L01, AFF4, Raw, 7z archive, native file copy) are tracked in the `export_history` table via `dbSync.insertExport()` (on start) and `dbSync.updateExport()` (on completion/failure). Each export gets a unique ID (e.g., `l01-1719842300000`, `aff4-1719842300000`, `raw-1719842300000`, `archive-1719842300000`, `file-export-1719842300000`).

**Type alignment:** `DbExportRecord` in `src/types/projectDb.ts` ↔ `DbExportRecord` in `src-tauri/src/project_db/types.rs`.

**Tauri commands:** `project_db_insert_export`, `project_db_update_export`, `project_db_get_exports`, `project_db_delete_export` (all registered in `lib.rs`).

**Sync layer:** `dbSync.insertExport(record)` and `dbSync.updateExport(record)` in `useProjectDbSync.ts`.

#### Unique Export Manifest Naming

Forensic manifests and reports include the `operation_id` in the filename to prevent overwrites across multiple exports:
- JSON manifest: `{export_name}_{operation_id}_manifest.json`  (e.g., `forensic_export_export-1719842300000_manifest.json`)
- TXT report: `{export_name}_{operation_id}_report.txt`

The `operation_id` is also included inside the JSON manifest body and TXT report header. The `CopyResult` struct returns `operationId` to the frontend for DB correlation.

#### L01 Writer Invariants

- **Single-segment** and **multi-segment** both use `segment::segment_path()` to derive the output path with `.L01`/`.L02` extension. The frontend passes the base path (without extension) and the backend appends it.
- **Directory sources** preserve the selected folder name: the backend creates a directory entry for the folder via `writer.add_directory(dir_name, 0)`, then walks contents under that entry via `walk_dir_into_writer()`. The folder appears as the root entry in the L01 tree.

#### Native Export Folder Preservation

When a directory is selected as a source, `collect_files()` uses `path.parent()` as the base for `strip_prefix`, so the folder name is preserved in relative paths. e.g., selecting `/path/to/Evidence/` produces `Evidence/file1.txt` (not just `file1.txt`).

**Do NOT:**
- Change default compression from `"none"` / `CompressionLevel.Store` — forensic standard requires bit-for-bit fidelity
- Reintroduce unconditional deflate for AFF4 small logical files when the selected compression is Stored — the speed-first AFF4 path depends on small ZIP-segment members honoring Stored mode too
- Change default split size from 2048 MB — this is the FAT32/FTK Imager standard
- Use raw `<input type="number">` for split/segment sizes — always use `SplitSizeSelector`
- Pass MB values directly to backend APIs — always multiply by `1024 * 1024` for bytes conversion
- Skip mount state restoration after imaging — always use `.finally()` to call `restoreAllDriveMounts()`
- Remove the `isSystemDisk` flag from `DriveInfo` — it gates the boot volume warning in `DriveSelector`
- Remove the `walk_dir_files()` helper from `ewf_export.rs` — it enables "Add Folder" support for E01 imaging
- Remove the `walk_dir_into_writer()` helper from `l01_export.rs` — it preserves folder structure in L01 images
- Re-add compression to NativeExportMode presets — all presets intentionally use `CompressionLevel.Store`
- Use `File::create(output_path)` in `write_single_segment` — use `segment::segment_path(output_path, 1)` to ensure `.L01` extension
- Use `path_obj` as base in `collect_files` for directories — use `path_obj.parent()` to preserve the folder name
- Remove `operationId` from `CopyResult` — it's used for unique manifest naming and DB tracking correlation
- Remove `dbSync.insertExport`/`dbSync.updateExport` from export hooks — exports won't be tracked in the database
- Remove `EXPORT_CANCEL_FLAGS` or the `cancel_export` command from `export.rs` — file export cancellation depends on them
- Remove `activeExportOperationId` signal from `useNativeExportState` — the cancel button visibility depends on it
- Remove the `get_available_space()` free space check from `export.rs` — exports to near-full destinations will silently fail mid-copy
- Collapse triage per-category skipped artifacts into `filesFailed` — macOS permission/TCC/time-out skips must remain visible as skipped, not hard failures
- Switch `triage.rs` back to `create_archive()` for packaging — use `create_archive_streaming()` to avoid the known in-memory packaging warning
- Replace the `shared_category_details` mutex recovery helpers in `triage.rs` with `lock().unwrap()` or `into_inner().unwrap_or_default()` — poisoned triage workers must recover category stats instead of crashing or silently dropping them during final aggregation
- Use `open()` dialog for repair output path — use `save()` dialog (the output is a new file being created, not an existing file being selected)
- Move the sysinfo `mounted_ro` check BEFORE the write probe in `check_path_writable` — on macOS firmlinked paths (`/Users`, `/Library`), sysinfo incorrectly matches the read-only system volume `/` instead of the writable data volume. The write probe MUST run first as ground truth.
- Remove `onSourceAdd`/`onSourceRemove` props from `DriveSourcePanel` — they power the live bidirectional sync between the drive panel and export panel
- Remove the second `createEffect` (watching `pendingRemoveSources`) from `ExportPanelComponent` — unchecking items in the drive panel won't remove them from the export source list
- Remove `pendingRemoveSources` signal from `App.tsx` — bidirectional removal sync will break
- Remove `removeSourceByPath` from `useExportCommon` — the drive panel removal path depends on it
- Remove `handleSourceAdd`/`handleSourceRemove` from `App.tsx` — they connect the sidebar drive panel to the export panel's pending signals
- Remove `initialDestination` from `ExportPanelProps`, `UseExportStateOptions`, or `UseExportCommonOptions` — it pre-sets the export destination to the project's Exports folder
- Remove `exports` role from folder templates or `exports_path` from `ProjectLocations` — the export destination auto-default depends on them

---

### Acquisition Companion Files & Evidence Collection Auto-Creation

Every successful acquisition (E01, L01, 7z archive, file copy, memory capture, triage collection) automatically:
1. Writes an `.ffx-companion.json` sidecar file alongside the output
2. Creates an `evidence_collections` + `collected_items` record in the project `.ffxdb`

Both operations are **fire-and-forget** — errors are logged via `console.warn` but never fail the acquisition UI.

**Companion timing invariant:** `CompanionTiming.durationMs` is serialized as integer milliseconds on the Rust side (`u64`). Frontend acquisition flows must normalize any floating-point durations (for example `durationSecs * 1000`) to a rounded integer before calling `write_companion_file`.

#### Companion File Format (v1.0)

```json
{
  "version": "1.0",
  "tool": "CORE-FFX",
  "toolVersion": "0.1.53",
  "createdAt": "2025-01-15T10:30:00.000Z",
  "acquisition": {
    "type": "e01",
    "sources": ["/dev/disk2"],
    "output": { "path": "/exports/image.E01", "format": "e01" },
    "hashes": { "md5": "abc...", "sha1": "def...", "sha256": "ghi..." },
    "timing": { "startedAt": "...", "completedAt": "...", "durationMs": 12345 },
    "caseInfo": { "caseNumber": "2024-001", "evidenceNumber": "EV-01", "examiner": "Jane" },
    "stats": { "totalBytes": 1073741824, "totalFiles": 1500 }
  }
}
```

#### Companion File Naming

| Output Type | Companion Path |
|---|---|
| Single file (E01, L01, 7z, .mem) | `<output_file>.ffx-companion.json` |
| Directory (triage, file copy) | `<output_dir>/ffx-companion.json` |

#### Evidence Collection Record

On acquisition completion, `companionHelper.ts` creates:
- `DbEvidenceCollection` — ID `ec-{timestamp}-{random}`, status `"draft"`, linked to case number
- `DbCollectedItem` — ID `ci-{timestamp}-{random}`, linked to collection, with acquisition method, format, file count, size, hashes, and duration

#### Hook Wiring

| Hook | File | Acquisition Type |
|------|------|-----------------|
| E01 (physical) | `useEwfExportState.ts` | `"e01"` |
| L01 (logical) | `useL01ExportState.ts` | `"l01"` |
| 7z archive | `useNativeExportState.ts` | `"archive"` |
| File copy | `useNativeExportState.ts` | `"file_copy"` |
| Memory capture | `useMemoryDumpState.ts` | `"memory"` |
| Triage collection | `useTriageState.ts` | `"triage"` |

All hooks capture source paths **before** `clearAllSources()` runs (via `[...common.sources()]`), to avoid reading an empty signal after cleanup.

#### Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/commands/companion.rs` | Rust backend: `write_companion_file`, `read_companion_file`, `find_companion_file`, `scan_for_acquisitions` |
| `src/api/companion.ts` | TypeScript API types + invoke wrappers |
| `src/api/importAcquisitions.ts` | `DiscoveredAcquisition`, `ImportResult` types + `scanForAcquisitions()` invoke wrapper |
| `src/hooks/export/companionHelper.ts` | `handleAcquisitionComplete()` — writes companion + creates evidence collection record |
| `src/hooks/useImportAcquisitions.ts` | Import acquisitions hook — scan, select, import logic + dbSync integration |
| `src/components/import/ImportAcquisitionsWizard.tsx` | 3-step modal wizard: Scan → Review → Results |

### Import Acquisitions (Companion File Discovery)

The Import Acquisitions feature scans a directory for `.ffx-companion.json` sidecar files and imports the referenced acquisitions into the current project.

**Entry Points:**
- **Tools → Import Acquisitions** (native menu bar, project-dependent)
- **Command Palette** (`Cmd+K` → "Import Acquisitions", project-dependent)

**Architecture:**

```text
Tools → "Import Acquisitions"
  → menu.rs emits "import-acquisitions"
  → useMenuActions dispatches onImportAcquisitions
  → App.tsx sets showImportWizard(true)
  → ImportAcquisitionsWizard.tsx (lazy-loaded modal)
    ├── Step 1: Scan — directory picker + scan button
    │   → invoke("scan_for_acquisitions", { dirPath })
    │   → recursive walk (MAX_DEPTH=10, skips hidden dirs)
    │   → returns DiscoveredAcquisition[] with companion data + output existence/size
    ├── Step 2: Review — acquisition cards with checkboxes
    │   → type badges, hash indicators, paths, warnings (already imported / file missing)
    │   → select all / deselect all / individual toggle
    └── Step 3: Results — success/skip/error summary
```

**What gets imported per acquisition:**
1. `DbEvidenceFile` — upserted via `dbSync.upsertEvidenceFile()` (path, container type from companion `acquisition.type`)
2. `DbProjectHash` records — one per hash in `companion.acquisition.hashes` with `source: "imported"`
3. `DbEvidenceCollection` — status `"draft"`, links to case number
4. `DbCollectedItem` — links to collection + evidence file, includes method/format/timing/hashes
5. `DbExportRecord` — inserted via `dbSync.insertExport()` for acquisition history tracking

**Deduplication:** Acquisitions whose output path is already in `knownPaths` (derived from `fileManager.discoveredFiles()`) are shown with an "Already imported" warning and deselected by default.

**Key Files:**

| File | Purpose |
|------|---------||
| `src-tauri/src/commands/companion.rs` | `scan_for_acquisitions` — recursive dir walk, parses `.ffx-companion.json` files |
| `src/api/importAcquisitions.ts` | `DiscoveredAcquisition`, `ImportResult` types + invoke wrapper |
| `src/hooks/useImportAcquisitions.ts` | Hook: `scan`, `toggleSelect`, `selectAll`, `deselectAll`, `importSelected`, `reset` |
| `src/components/import/ImportAcquisitionsWizard.tsx` | 3-step modal wizard with `AcquisitionCard` sub-component |

#### Do NOT

- Make companion writes blocking — they must be fire-and-forget to avoid slowing acquisition completion
- Remove the source capture (`[...common.sources()]`) before async operations — `clearAllSources()` runs synchronously after `invoke()` starts
- Remove `handleAcquisitionComplete` imports from any export hook — companion creation will silently stop for that mode
- Change the companion file naming convention — `.ffx-companion.json` suffix is used by `find_companion_file` and `scan_for_acquisitions` for discovery
- Remove companion commands from `run_acquire()` in `lib.rs` — the Acquire edition needs them too
- Remove `scan_for_acquisitions` from either `run_full()` or `run_acquire()` command registration — both editions need import capability
- Remove `import-acquisitions` from `PROJECT_DEPENDENT_IDS` in `menu.rs` — importing requires an open project to write to `.ffxdb`
- Use `dbSync.insertHash` for imported hashes — use `dbSync.insertHash` with `source: "imported"` to distinguish from computed hashes

---

### COC Immutability Model (Schema v9 — Extended COC Fields)

Chain of Custody records use an **append-only immutability model** enforced at both the Rust backend and the SolidJS frontend. This ensures forensic integrity and a complete audit trail for all evidence handling.

The COC data model and UI follow a standardized chain of custody form structure with fields for owner identification, collection method, transfer records, and final disposition.

**COC Field Mapping (schema v9, 15 new coc\_items columns + 2 coc\_transfers columns):**

| COC Section | Fields | DB Columns |
|---|---|---|
| Header | Case Title, Office, COC# | `case_title`, `office` |
| Owner / Source / Contact | Owner Name/Address/Phone, Source, Other Contact Name/Relation/Phone | `owner_name`, `owner_address`, `owner_phone`, `source`, `other_contact_name`, `other_contact_relation`, `other_contact_phone` |
| Collection Method | Search Warrant, Grand Jury Subpoena, Consent Seizure, Abandoned, Digital/Electronic Capture, Voluntary Submission, Other | `collection_method`, `collection_method_other` |
| Collected By | Date Collected | `collected_date` |
| Final Disposition | Disposition By, Returned To, Destruction Date | `disposition_by`, `returned_to`, `destruction_date` |
| Transfer Rows | Storage Location, Date Entered Storage | `storage_location` (on coc\_transfers), `storage_date` (on coc\_transfers) |

**COCItemRow UI structure** (8 numbered sections):
1. Case Information (Case Title, Office, Case#, COC#, Evidence ID)
2. Owner / Source / Contact
3. Collection Method (radio buttons: 7 options from `COC_COLLECTION_METHODS`)
4. Item Details (Description, Type, Make, Model, Serial, Capacity, Condition)
5. Collection & Custody (Collected By, Date, Received By, Acquisition/Custody Dates, Storage Location, Intake Hashes)
6. Remarks
7. Transfer Records (Relinquished By, Received By, Date, Purpose, Storage Location, Date Entered, Method)
8. Final Disposition (Disposition, By, Returned To, Destruction Date, Notes)

**COC Prefill** (`src/components/report/wizard/utils/cocPrefill.ts`):
- `prefillCocFromContainer(group, info, caseNum, examiner, caseTitle)` — maps E01/AD1/UFED container metadata to COCItem fields
- `overlayCocFromCollection(item, collection, collectedItem)` — enriches COCItem with evidence collection form data
- Auto-populate in `COCFormSection` now uses `prefillCocFromContainer` + `fileInfoMap` + `fileHashMap` to richly populate COC items instead of just filename + "HardDrive"

**Status Lifecycle:**

```text
  draft  ──(lock)──▸  locked  ──(void)──▸  voided
    │                    │
    │ (free edit)        │ (amend w/ initials + reason)
    ▼                    ▼
  update               amendment record created
```

| Status | Behavior | UI |
|--------|----------|-----|
| `draft` | Freely editable, removable | Green "Draft" badge, all inputs active |
| `locked` | Immutable. Edits require initials + reason → creates `DbCocAmendment` | Yellow "🔒 Locked" badge, `readOnly` inputs, amendment modal on edit attempt |
| `voided` | Soft-deleted. Record persists for audit trail, hidden from active views | Red "Voided" badge, `opacity-50`, `line-through`, form collapsed |

**Database Tables (`project_db/schema.rs`, schema v9):**

| Table | Purpose |
|-------|---------|
| `coc_items` | COC records. Columns: `status TEXT DEFAULT 'draft'`, `locked_at TEXT`, `locked_by TEXT` |
| `coc_amendments` | Per-field amendment records. FK → `coc_items(id)` with `ON DELETE RESTRICT` |
| `coc_audit_log` | Action log (insert, update, lock, amend, void, transfer). FK → `coc_items(id)` with `ON DELETE RESTRICT` |

**Tauri Commands (COC-specific):**

| Command | Purpose |
|---------|---------|
| `project_db_insert_coc_item` | INSERT only (rejects duplicates), creates audit entry |
| `project_db_upsert_coc_item` | UPDATE only if status = `draft`; rejects locked/voided |
| `project_db_lock_coc_item` | Sets status = `locked`, records `locked_by` + timestamp, creates audit entry |
| `project_db_amend_coc_item` | Validates field name against 24-field allowlist, creates `DbCocAmendment`, updates field, creates audit entry |
| `project_db_delete_coc_item` | Soft-delete: sets status = `voided`, requires `voided_by` + `reason`, creates audit entry |
| `project_db_get_coc_amendments` | Returns amendments for a COC item, ordered by `amended_at` |
| `project_db_get_coc_audit_log` | Returns audit entries for a COC item (or all if id is null) |
| `project_db_insert_coc_audit_entry` | Manual audit entry insertion |

**Frontend Sync (useProjectDbSync.ts):**

| Sync Function | Maps To |
|---------------|---------|
| `dbSync.insertCocItem(record)` | `project_db_insert_coc_item` |
| `dbSync.upsertCocItem(record)` | `project_db_upsert_coc_item` |
| `dbSync.lockCocItem(id, lockedBy)` | `project_db_lock_coc_item` |
| `dbSync.deleteCocItem(id, voidedBy, reason)` | `project_db_delete_coc_item` |
| `dbSync.insertCocAuditEntry(entry)` | `project_db_insert_coc_audit_entry` |
| `dbSync.upsertCocTransfer(record)` | `project_db_upsert_coc_transfer` |
| `dbSync.deleteCocTransfer(id)` | `project_db_delete_coc_transfer` |

> **Note:** `dbSync.*` COC methods are fire-and-forget. For awaitable COC persistence (e.g., before closing the wizard), use `persistCocItemsToDb()` from `cocDbSync.ts`.

**Key Types:**

| Rust | TypeScript |
|------|-----------|
| `DbCocItem` (+ `status`, `locked_at`, `locked_by`) | `DbCocItem` in `projectDb.ts` |
| `DbCocAmendment` | `DbCocAmendment` in `projectDb.ts` |
| `DbCocAuditEntry` | `DbCocAuditEntry` in `projectDb.ts` |
| `COCItem` (report type, + `status`, `locked_at`, `locked_by`) | `COCItem` in `report/types.ts` |

**Key Files:**

| File | Purpose |
|------|---------|
| `src-tauri/src/project_db/` | Module directory: `schema.rs` (v9), `forensic.rs` (COC CRUD), `collections.rs` (evidence collections), `types.rs` (all DB types) |
| `src-tauri/src/commands/project_db/` | Module directory: `forensic.rs` (COC commands), `collections.rs` (evidence collection commands) |
| `src/types/projectDb.ts` | `DbCocItem`, `DbCocAmendment`, `DbCocAuditEntry` TS interfaces |
| `src/hooks/project/useProjectDbSync.ts` | Fire-and-forget sync functions (use `cocDbSync.ts` for awaitable COC/collection saves) |
| `src/components/report/types.ts` | `COCItem` with `status`, `locked_at`, `locked_by` fields |
| `src/components/report/constants.ts` | `COC_COLLECTION_METHODS` — COC collection method options |
| `src/components/report/wizard/utils/cocPrefill.ts` | `prefillCocFromContainer()`, `overlayCocFromCollection()` — maps container/collection data to COCItem |
| `src/components/report/wizard/steps/reportdata/COCFormSection.tsx` | UI with lock/amend/void modals, auto-populate from container metadata |
| `src/components/report/wizard/steps/reportdata/COCItemRow.tsx` | Structured COC layout (8 numbered sections) |

**Do NOT:**
- Allow direct UPDATE of locked COC items — all edits must go through `amend_coc_item` with initials + reason
- Hard-delete COC items that have audit history — `ON DELETE RESTRICT` on FKs prevents this
- Remove the `status` field or default from `DbCocItem` — all new items must start as `"draft"`
- Bypass the 24-field allowlist in `amend_coc_item` — SQL injection via field name is prevented by this whitelist
- Remove the amendment/lock/void modals from `COCFormSection.tsx` — they enforce the initials requirement
- Change `readOnly` to `disabled` on locked text inputs — `readOnly` preserves the visual appearance while preventing edits
- Allow `select` dropdowns on locked items without `disabled` — selects need `disabled` (not `readOnly`)
- Remove `ON DELETE RESTRICT` from `coc_amendments` or `coc_audit_log` FKs — this prevents data loss

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
| `src-tauri/src/report/types/` (module dir) | `src/report/types.ts` |
| `src-tauri/src/viewer/document/types.rs` | `src/types/viewer.ts` |
| `src-tauri/src/archive/types.rs` | `src/types.ts` (`ArchiveFormat`, etc.) |
| `src-tauri/src/commands/lazy_loading.rs` | `src/types/lazy-loading.ts` |
| `src-tauri/src/database.rs` | `src/types/database.ts` |
| `src-tauri/src/project_db/types.rs` | `src/types/projectDb.ts` |
| `src-tauri/src/processed/types.rs` | `src/types/processed.ts` |
| `src-tauri/src/commands/ewf_export.rs` | `src/api/ewfExport.ts` (EwfExportOptions) |
| `src-tauri/src/commands/l01_export.rs` | `src/api/l01Export.ts` (L01ExportOptions) |
| `src-tauri/src/l01_writer/types.rs` | `src/api/l01Export.ts` (L01ExportProgress, L01WritePhase, L01ExportResult) |
| `src-tauri/src/commands/system.rs` (DriveInfo, MountResult) | `src/api/drives.ts` |
| `src-tauri/src/commands/device.rs` (PrivilegeInfo, PhysicalDisk, DeviceReadProgress) | `src/api/device.ts` |

**Workflow when changing a Rust struct:**
1. Make the Rust change
2. Search for the TypeScript interface: `grep -rn "interface MyType" src/`
3. Add/rename/remove the corresponding TypeScript field (use `camelCase`)
4. If adding a new enum variant in Rust, add the same string to the TypeScript union type

### Rule 4: Never Send `undefined` for NOT NULL / Non-Optional Fields (Prevents Silent Save Failures)

**When building objects to send to the Rust backend via `invoke()`, NEVER use `|| undefined` for fields that map to `String` (non-`Option`) Rust types or `NOT NULL` SQLite columns.**

Why this matters:
- `JSON.stringify()` strips `undefined` fields entirely
- Rust serde expects a `String` field to be present in the JSON → deserialization fails when the field is missing
- `syncInvoke()` in `useProjectDbSync.ts` catches all errors silently → the INSERT/UPDATE never reaches SQLite and the caller doesn't know

**Pattern to avoid:**
```typescript
// ❌ BAD — if someValue is "", || undefined removes the field from JSON
const record = { myField: someValue || undefined };
```

**Correct pattern:**
```typescript
// ✅ GOOD — empty string is a valid String in Rust and satisfies NOT NULL
const record = { myField: someValue || "" };
```

**Affected conversion functions in `cocDbSync.ts`:**
- `cocItemToDb()` — all NOT NULL fields use `|| ""`
- `evidenceCollectionToDb()` — all NOT NULL fields use `|| ""`
- `collectedItemToDb()` — all NOT NULL fields use `|| ""`

**TypeScript interface alignment:** Fields that are `NOT NULL` in the Rust struct / SQLite schema MUST be non-optional (`string`, not `string?`) in the TypeScript interface. Key interfaces: `DbCocItem`, `DbEvidenceCollection`, `DbCollectedItem` in `src/types/projectDb.ts`.

### Rule 5: Run `cargo check` Early and Often

After every non-trivial code change, run `cargo check` before moving to the next file. This catches API mismatches immediately instead of accumulating them.

### Rule 6: Always Update Documentation and Instructions After Code Changes

**Every code change that modifies architecture, adds/removes features, changes entry points, alters guard logic, or introduces new patterns MUST be accompanied by documentation updates.** Undocumented changes create knowledge drift, which causes future agents and developers to reintroduce bugs, bypass guards, or violate invariants.

**What to update and when:**

| Change Type | Files to Update |
|---|---|
| New Tauri command added | `copilot-instructions.md` (Backend Command Groups table), `CODE_BIBLE.md` (command listing), `lib.rs` registration |
| New hook or hook API change | `copilot-instructions.md` (Hooks API Reference), `FRONTEND_API_NOTES.md`, `src/hooks/README.md` |
| New component or component removal | `src/components/README.md`, `copilot-instructions.md` (relevant architecture section) |
| New file viewer/parser | `copilot-instructions.md` (Adding a New Viewer/Parser, Viewer-Inside-Container Pipeline) |
| Feature guard added/removed | `copilot-instructions.md` (document which entry points are guarded and the guard mechanism) |
| Menu item added/removed | `copilot-instructions.md` (Native Menu Bar section, `PROJECT_DEPENDENT_IDS` if project-dependent) |
| New "Do NOT" invariant discovered | `copilot-instructions.md` (add to the relevant section's "Do NOT" list) |
| Rust struct with `serde(rename_all)` changed | Corresponding TypeScript interface (see Type Alignment table), `copilot-instructions.md` if it adds a new mapping |
| New crate dependency or API usage | `CRATE_API_NOTES.md` (document actual method signatures and gotchas) |
| UI layout change (toolbar, sidebar, title bar, status bar) | `copilot-instructions.md` (UI Layout Invariants section) |
| New export format or export default change | `copilot-instructions.md` (Export Panel Architecture section) |
| Database schema migration | `copilot-instructions.md` (COC Immutability Model or relevant DB section), schema version note |
| New keyboard shortcut | `copilot-instructions.md` (document in relevant hook section) |
| CSS design token or component class added | `copilot-instructions.md` (CSS Architecture section), `src/styles/README.md` |

**Documentation update checklist (run mentally after every task):**

1. **Was a feature added or removed?** → Update `copilot-instructions.md` architecture sections and `CODE_BIBLE.md`
2. **Was a guard or safety check added?** → Document what is guarded, at which entry points, and the guard mechanism
3. **Was a "Do NOT" lesson learned?** → Add it to the closest relevant section's "Do NOT" list to prevent regression
4. **Was frontend ↔ backend contract changed?** → Update the Type Alignment table and both sides' types
5. **Was a third-party API used for the first time?** → Document actual signatures in `CRATE_API_NOTES.md`
6. **Were props added/removed from a component?** → Update component docs and any "Do NOT re-add" lists if the removal was intentional

**Why this matters for code protection:**
- `copilot-instructions.md` is the **primary defense** against AI agents and developers undoing intentional decisions
- Every undocumented removal gets re-added; every undocumented guard gets bypassed
- The "Do NOT" lists exist because those exact mistakes were made before — they are the project's institutional memory
- `CRATE_API_NOTES.md` prevents the #1 source of compilation errors (guessed API signatures)

**Failure to update documentation is a bug** — treat it with the same priority as a missing test or a broken build.
