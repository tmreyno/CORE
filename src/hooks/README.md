# CORE-FFX Hooks

Custom SolidJS hooks that manage state and bridge to Tauri commands.

## Hooks

### useFileManager

Evidence file management and discovery.

- Evidence directory selection and scanning
- Streaming discovery events (`scan-file-found`)
- File selection, filtering, and active file tracking
- Container info loading and status tracking
- System stats listener (`system-stats`)

```tsx
import { useFileManager } from "./hooks";

const fileManager = useFileManager();

// Select evidence directory
await fileManager.selectDirectory();

// Get discovered files
const files = fileManager.files();

// Filter files
fileManager.setFilter("ad1");

// Select a file
fileManager.setActiveFile(file);
```

### useEntrySource

Utilities for reading data from various evidence sources (disk files, AD1 entries, VFS entries, archive entries).

- `readBytesFromSource(source, offset, length)` — read raw bytes from any source type
- `readTextFromSource(source, maxBytes?)` — read text content from any source type
- `getSourceKey(source)` — get a unique cache key for a source
- `getSourceFilename(source)` — extract the display filename from a source

```tsx
import { readBytesFromSource, readTextFromSource, getSourceKey } from "./hooks";

// Read bytes from an evidence entry
const result = await readBytesFromSource(entrySource, 0, 512);

// Read text content
const text = await readTextFromSource(entrySource, 65536);

// Get cache key for deduplication
const key = getSourceKey(entrySource);
```

### useHashManager

Hash computation and verification.

- Hash algorithm selection (forensic + fast)
- Batch hashing with progress events (`batch-progress`)
- Verification against stored hashes
- Hash history tracking

```tsx
import { useHashManager } from "./hooks";

const hashManager = useHashManager();

// Set algorithm
hashManager.setAlgorithm("SHA-256");

// Hash active file
await hashManager.hashActiveFile();

// Verify hash
const isValid = await hashManager.verifyHash(file, expectedHash);
```

### useDatabase

SQLite-backed settings and session data persistence.

- Per-file persistence for discovery/hashes
- Settings storage
- Session state management

```tsx
import { useDatabase, getLatestHash } from "./hooks";

// Get stored hash
const hash = await getLatestHash(filePath, "SHA-256", "full");
```

### useProject

Project file (.cffx) management.

- `.cffx` create/load/save
- Auto-save coordination (every 5 minutes)
- Activity logging
- Recent projects tracking

```tsx
import { useProject } from "./hooks";

const project = useProject();

// Create new project (with optional case identification)
await project.createProject(path, name, caseNumber?, caseName?);

// Load existing project
await project.loadProject(projectPath);

// Save project
await project.saveProject();

// Access project info
const info = project.projectInfo();
const caseNum = project.caseNumber();   // from FFXProject.case_number
const caseNam = project.caseName();     // from FFXProject.case_name
```

### useProjectDbSync

Write-through sync layer for the per-project `.ffxdb` SQLite database.

- Fire-and-forget: all writes are non-blocking
- `.cffx` remains the source of truth
- Covers bookmarks, notes, activity, tags, sessions, evidence files, hashes, verifications, reports, saved searches, UI state

```tsx
import { dbSync } from "./hooks/project/useProjectDbSync";

dbSync.upsertBookmark(bookmark);
dbSync.insertActivity(entry);
dbSync.upsertEvidenceFile(file);
dbSync.insertHash(hash);
dbSync.setUiState(key, value);
```

### useProjectDbRead

Seeds the `.ffxdb` from the loaded `.cffx` project on first open.

- Called once during project load (from `projectHelpers.ts`)
- Idempotent: checks DB stats before seeding each table
- Seeds bookmarks, notes, activity log, tags, sessions, saved searches, reports, evidence files, cached hashes

```tsx
import { seedDatabaseFromProject } from "./hooks/project/useProjectDbRead";

await seedDatabaseFromProject(project);
```

### useProcessedDatabases

Processed database detection and parsing.

- Detect and list processed databases
- AXIOM parsing for categories/artifacts
- Selection and detail state

### useMenuActions

Bridges native menu bar events to frontend handlers. Called once in `App.tsx`.

- Listens for `"menu-action"` Tauri events emitted by `menu.rs`
- Dispatches to 36 callback functions via `UseMenuActionsDeps` interface
- Auto-cleans up listener via `onCleanup()`

```tsx
import { useMenuActions } from "./hooks";

useMenuActions({
  onOpenProject: () => handleLoadProject(),
  onSaveProject: handleSaveProject,
  onToggleSidebar: () => setLeftCollapsed((prev) => !prev),
  onNewProject: () => setShowProjectWizard(true),
  // ... 32 more handlers (see UseMenuActionsDeps interface)
});
```

**Event flow:** `menu.rs` → `handle_menu_event()` → `emit("menu-action", id)` → `useMenuActions` switch → callback.

```tsx
import { useProcessedDatabases } from "./hooks";

const processedDb = useProcessedDatabases();

// Scan for databases
await processedDb.scanDatabases(directory);

// Get database details
const details = processedDb.selectedDetails();
```

### useProgressTracker

Smoothed progress statistics (speed, ETA, elapsed) for long-running acquisition operations.

- Exponential Moving Average (EMA) with `α = 0.3` for stable speed estimates
- 2-second warmup before showing speed/ETA
- Pre-formatted strings for direct rendering (`speedFormatted`, `etaFormatted`, `elapsedFormatted`)
- Auto-ticking elapsed timer (1 s interval, auto-cleaned up)

```tsx
import { createProgressTracker } from "./hooks/useProgressTracker";

const tracker = createProgressTracker();

// Feed progress snapshots from a Tauri event listener
tracker.update({ bytesProcessed: 5000, bytesTotal: 10000, percent: 50 });

// Read reactive stats in JSX
<span>{tracker.stats().speedFormatted}</span>  // "125.3 MB/s"
<span>{tracker.stats().etaFormatted}</span>    // "2m 15s"
<span>{tracker.stats().elapsedFormatted}</span> // "1m 30s"

// Reset between operations
tracker.reset();
```

**Interfaces:** `ProgressSnapshot { bytesProcessed, bytesTotal, percent }`, `SmoothedStats { speedBps, etaMs, elapsedMs, speedFormatted, etaFormatted, elapsedFormatted }`

### useAppLifecycle

Manages the App component's mount/cleanup lifecycle. Extracted from App.tsx to keep the root component focused on composition.

- System stats listener setup
- Window width tracking + compact-mode detection (`< 900px`)
- Workspace profile loading (full edition only)
- Auto-save callback registration
- Welcome modal first-run detection
- Last-session restoration (full edition only)
- Window title and close-confirmation wiring
- Cleanup: preview cache, clipboard clear, auto-save stop

```tsx
import { useAppLifecycle } from "./hooks/useAppLifecycle";

const { windowWidth, isCompact } = useAppLifecycle({
  fileManager,
  projectManager,
  workspaceProfiles: { listProfiles, getActiveProfile },
  db: { restoreLastSession },
  tour: { hasCompleted },
  preferences: { preferences },
  getSaveOptions: () => buildSaveOptions(...),
  setShowWelcomeModal,
});

// Use reactive window width
<Show when={!isCompact()}>...</Show>
```

### usePortableMode

Detects and exposes portable mode status (zero-footprint operation from USB/removable media).

- Lazy initialization — call `check()` explicitly (no backend call on mount)
- Result cached after first successful query (portable mode can't change at runtime)
- Signals: `isPortable()`, `config()`, `status()`, `ready()`

```tsx
import { usePortableMode } from "./hooks/usePortableMode";

const portable = usePortableMode();
portable.check();  // Query backend (safe to call multiple times)

if (portable.isPortable()) {
  const cfg = portable.config(); // PortableConfig with data/cache/temp/log dirs
}
```

### usePreferenceEffects

Applies `AppPreferences` to the DOM via reactive `createEffect` hooks. Called once in App.tsx.

- 10 DOM effects: theme, accentColor, fontSize (8-token scale with 8px floor), animationsEnabled, treeDensity, sidebarPosition, iconSet, showStatusBar, showLineNumbers, showFileSizes
- System theme change listener (re-resolves when `prefers-color-scheme` changes)

```tsx
import { usePreferenceEffects } from "./hooks/usePreferenceEffects";

usePreferenceEffects(() => preferences());
// No return value — all effects are side-effect only
```

**Font size scale:** `2xs=base-4, compact=base-3, xs=base-2, sm=base-1, base, lg=base+2, xl=base+4, 2xl=base+8` (all clamped to 8px floor)

### useSearchIndex

Manages the Tantivy full-text search index lifecycle, tied to project open/close.

- Opens index when project opens, closes on project close
- Auto-indexes discovered containers (metadata-only by default)
- Listens for `index-progress` events
- Provides indexing state and stats

```tsx
import { useSearchIndex } from "./hooks/useSearchIndex";

const searchIndex = useSearchIndex({
  hasProject: () => !!project(),
  projectPath: () => project()?.path ?? null,
  discoveredFilePaths: () => files().map(f => f.path),
});

// Check index state
searchIndex.indexReady()    // boolean — can accept queries
searchIndex.indexing()      // boolean — indexing in progress
searchIndex.stats()         // IndexStats | null

// Manual operations
await searchIndex.indexSingleContainer(containerPath, includeContent);
await searchIndex.indexAllDiscovered(includeContent);
await searchIndex.rebuildIndex(includeContent);
await searchIndex.refreshStats();
```

### useWorkspaceMode

Reactive hook for workspace mode management (feature module system).

- Resolves active preset from `AppPreferences.workspaceMode`
- Computes enabled `FeatureModule[]` for sidebar, toolbar, and quick action gating
- Exports `TAB_MODULE_MAP` (sidebar tab → module) and `ACTION_MODULE_MAP` (quick action → module)

```tsx
import { useWorkspaceMode, TAB_MODULE_MAP, ACTION_MODULE_MAP } from "./hooks/useWorkspaceMode";

const workspaceMode = useWorkspaceMode();

workspaceMode.activeMode()                // WorkspaceModePreset
workspaceMode.enabledModules()            // FeatureModule[]
workspaceMode.isModuleEnabled("forensicExplorer")  // boolean

workspaceMode.setMode("forensic");        // Switch to preset
workspaceMode.toggleModule("searchAnalysis"); // Toggle module (auto-switches to "custom")
workspaceMode.getFirstEnabledTab()        // LeftPanelTab fallback
```

**Module IDs:** `forensicExplorer`, `evidenceCollection`, `documentReview`, `searchAnalysis`, `reportExport`, `caseManagement`

### useTextSelectionMenu

Context menu for text selected inside document viewers.

- Detects `window.getSelection()` — shows custom menu only when text is selected
- Falls through to browser default context menu when no text is selected
- Four actions: Bookmark Selection, Note from Selection, Search for Selection, Copy

```tsx
import { useTextSelectionMenu } from "./hooks/useTextSelectionMenu";

const selectionMenu = useTextSelectionMenu({
  onBookmarkSelection: (text) => addBookmark(text),
  onNoteFromSelection: (text) => addNote(text),
  onSearchSelection: (text) => setSearchQuery(text),
});

<div onContextMenu={selectionMenu.handleContextMenu}>
  {/* Document viewer content */}
</div>
```

### useTheme

Theme management (light/dark/system).

- Theme persistence
- System preference detection
- CSS class application

```tsx
import { useTheme } from "./hooks";

const { theme, setTheme, toggleTheme, isDark } = useTheme();

// Get current theme
console.log(theme()); // "dark" | "light" | "system"

// Set specific theme
setTheme("light");

// Toggle between light/dark
toggleTheme();

// Check if dark mode is active
if (isDark()) {
  // Dark mode styling
}
```

### useFocusTrap

Focus trapping for modals and dialogs.

- Trap focus within a container
- Return focus on close
- Keyboard navigation (Tab, Shift+Tab)

```tsx
import { useFocusTrap } from "./hooks";

function Modal(props) {
  const modalRef = useFocusTrap(props.isOpen);

  return (
    <div ref={modalRef} class="modal">
      <button>First focusable</button>
      <button>Last focusable</button>
    </div>
  );
}
```

### useHistory

Undo/redo state management.

- Command pattern for state changes
- History stack with configurable limit
- Undo/redo operations

```tsx
import { useHistory, createStateCommand } from "./hooks";

const history = useHistory({ maxHistory: 50 });

// Create a command
const setNameCommand = createStateCommand(
  setName,
  "Change Name",
  oldValue,
  newValue
);

// Execute with history tracking
history.execute(setNameCommand);

// Undo/Redo
history.undo();
history.redo();

// Check state
const canUndo = history.canUndo();
const canRedo = history.canRedo();
```

---

## Acquire Hooks (`acquire/`)

### acquire/types.ts

Type definitions for the acquisition runner system.

- `AcquisitionTaskType`: `"e01" | "l01" | "aff4" | "raw" | "archive" | "file_copy" | "memory" | "triage"`
- `ACQUISITION_PRIORITY`: Record mapping each type to a numeric priority (lower = first). E01 → 1, L01 → 2, AFF4 → 3, Raw → 4, Memory → 5, Archive → 6, Triage → 7, File Copy → 8
- `AcquisitionTaskConfig`: Per-task configuration with type, source, destination, format-specific options
- `defaultConfig()`: Returns sensible defaults for each `AcquisitionTaskType`
- `AcquisitionTask`: Full task record with id, config, status, progress, timing, results, companion ref

### acquire/useAcquisitionRunner.ts

Sequential acquisition orchestrator — runs queued acquisition tasks one at a time, ordered by priority.

- Manages a task queue with add/remove/reorder operations
- Executes tasks sequentially (never parallel) in priority order
- Dispatches to format-specific export hooks (E01, L01, AFF4, Raw, Native, Memory, Triage)
- Creates companion `.ffx-companion.json` files and `evidence_collections` records on completion
- Tracks per-task progress, timing, and results

```tsx
import { useAcquisitionRunner } from "./hooks/acquire/useAcquisitionRunner";

const runner = useAcquisitionRunner({ fileManager, hashManager, projectManager, toast });

runner.addTask({ type: "e01", source: "/dev/disk2", destination: "/exports/" });
runner.addTask({ type: "l01", source: "/evidence/", destination: "/exports/" });

await runner.startAll();  // Runs queued tasks sequentially by priority
runner.cancelCurrent();   // Cancel currently running task
runner.clearCompleted();  // Remove finished tasks from queue
```

---

## Export Hooks (`export/`)

### export/companionHelper.ts

Writes companion sidecar `.ffx-companion.json` files and creates `evidence_collection` + `collected_item` records in `.ffxdb` after every successful acquisition.

- `startAcquisitionRecord(type, sources, output, caseInfo)` → returns `startTime` for later use
- `handleAcquisitionComplete(params)` → writes companion file via `write_companion_file` Tauri command + creates `DbEvidenceCollection` + `DbCollectedItem` records via `dbSync`
- Both operations are fire-and-forget — errors are logged but never fail the acquisition UI
- Works for all acquisition types: E01, L01, AFF4, Raw, 7z, file copy, memory, triage

### export/useExportCommon.ts

Shared export state used by all format-specific export hooks.

- Source file/folder management (add, remove, clear)
- Destination path selection
- Drive source handling (read-only mount toggle)
- `removeSourceByPath(path)` — used by sidebar bidirectional sync

```tsx
import { useExportCommon } from "./hooks/export/useExportCommon";

const common = useExportCommon({ initialDestination: "/exports/" });

common.handleAddSource();        // Opens file picker
common.handleAddFolder();       // Opens folder picker
common.handleAddDriveSource(path); // Adds drive path directly
common.clearAllSources();
```

### export/useEwfExportState.ts

E01 forensic image creation via `ewf_create_image` (libewf-ffi).

### export/useL01ExportState.ts

L01 logical evidence creation via `l01_create_image` (pure-Rust l01_writer).

### export/useAff4ExportState.ts

AFF4 forensic container creation via `aff4_create_image` (pure-Rust ffx-aff4).

### export/useRawExportState.ts

Raw disk image creation via `raw_create_image` with segmentation and concurrent hashing.

### export/useNativeExportState.ts

7z archive creation + native file copy via `create_7z_archive` / `export_files`.

### export/useMemoryDumpState.ts

Live RAM capture via `memory_capture` (Linux `/proc/kcore`, Windows WinPmem).

### export/useTriageState.ts

Forensic triage collection + credential/secret scanning via `triage_collect`.

All export state hooks follow the same pattern:
- Accept `common: UseExportCommonReturn` for shared source/destination state
- Expose `handleStart()`, `handleCancel()`, progress signals, result state
- Call `companionHelper.handleAcquisitionComplete()` on success
- Track exports in `.ffxdb` via `dbSync.insertExport()` / `dbSync.updateExport()`

---

## Architecture

```text
hooks/
├── index.ts                  # Barrel exports
├── README.md                 # This file
├── hashUtils.ts              # Hash utility helpers
│
│ # Core hooks
├── useEntrySource.ts         # Evidence source read utilities
├── useFileManager.ts         # File discovery & management
├── useHashManager.ts         # Hash computation
├── useHashComputation.ts     # Batch hash orchestration
├── useHashHistory.ts         # Hash history tracking
├── useDatabase.ts            # SQLite persistence
├── useProject.ts             # Project management
├── useProcessedDatabases.ts  # Processed DB parsing
├── useAppState.ts            # Global app state
├── useAppActions.ts          # App-wide actions
├── useAppHandlers.ts         # App-level event handlers
├── useAppLifecycle.ts        # Mount/cleanup lifecycle (system stats, resize, session restore)
├── useMenuActions.ts         # Native menu bar event bridge
├── useSearchIndex.ts         # Tantivy search index lifecycle
├── useWorkspaceMode.ts       # Workspace mode / feature module management
├── usePortableMode.ts        # Portable mode detection (zero-footprint USB)
├── useProgressTracker.ts     # Smoothed progress stats (EMA speed/ETA)
├── useLoadingState.ts        # Global loading indicator
├── usePreferenceEffects.ts   # Preference → DOM side effects
├── useImportAcquisitions.ts  # Import acquisitions from companion files
│
│ # Project sub-hooks
├── project/
│   ├── index.ts              # Project hook exports
│   ├── types.ts              # Project types
│   ├── projectHelpers.ts     # Project utility helpers
│   ├── projectSetup.ts       # Project setup logic
│   ├── useProjectState.ts    # Project state management
│   ├── useProjectIO.ts       # Project file I/O
│   ├── useProjectHelpers.ts  # Project helper utilities
│   ├── useAutoSave.ts        # Auto-save functionality
│   ├── useBookmarks.ts       # Evidence bookmarks
│   ├── useNotes.ts           # Case notes
│   ├── useActivityLog.ts     # Activity logging
│   ├── useProjectDbSync.ts   # Write-through sync to .ffxdb
│   ├── useProjectDbRead.ts   # Seed .ffxdb from .cffx on load
│   └── useExaminerProfile.ts # Examiner profile persistence
│
│ # Acquire sub-hooks
├── acquire/
│   ├── types.ts              # AcquisitionTask, AcquisitionTaskType, ACQUISITION_PRIORITY
│   └── useAcquisitionRunner.ts # Sequential acquisition orchestrator
│
│ # Export sub-hooks
├── export/
│   ├── types.ts              # Export-specific types
│   ├── companionHelper.ts    # Companion sidecar + evidence collection records
│   ├── useExportCommon.ts    # Shared export state (sources, destination, drives)
│   ├── useEwfExportState.ts  # E01 export handler
│   ├── useL01ExportState.ts  # L01 logical export handler
│   ├── useAff4ExportState.ts # AFF4 export handler
│   ├── useRawExportState.ts  # Raw disk image handler
│   ├── useNativeExportState.ts # 7z archive + file copy handler
│   ├── useMemoryDumpState.ts # Live RAM capture handler
│   └── useTriageState.ts     # Forensic triage handler
│
│ # UI hooks
├── useTheme.ts               # Theme management
├── useFocusTrap.ts           # Modal focus trapping
├── useHistory.tsx            # Undo/redo history
├── useKeyboardHandler.ts     # Keyboard event handler
├── usePanelResize.ts         # Panel resize logic
├── useCenterPaneTabs.ts      # Center pane tab management
├── useCommandPalette.tsx     # Command palette
├── useCloseConfirmation.ts   # Unsaved changes guard
├── useWindowTitle.ts         # Window title management
├── useTextSelectionMenu.ts   # Text selection context menu (bookmark/note/search)
│
│ # Feature hooks
├── useActivityLogging.ts     # Activity logging side effects
├── useActivityManager.ts     # Running operation tracking
├── useActivityTimeline.ts    # Activity timeline
├── useEntryNavigation.ts     # Evidence entry click-to-open
├── useExportState.ts         # Export panel state orchestrator
├── useLazyLoading.ts         # Lazy loading
├── useProjectActions.ts      # Project save/load action bundle
├── useProjectComparison.ts   # Project comparison
├── useProjectTemplates.ts    # Project templates
├── useProjectRecovery.ts     # Project recovery
├── useWorkspaceProfiles.ts   # Workspace profiles
└── useDatabaseEffects.ts     # Database side effects
```

## Exports

All hooks and utilities are exported from `index.ts`:

```tsx
import {
  // Core hooks
  useFileManager,
  useHashManager,
  useDatabase,
  useProject,
  useProcessedDatabases,
  
  // UI hooks
  useTheme,
  useFocusTrap,
  useHistory,
  
  // Utilities
  createStateCommand,
  getLatestHash,
} from "./hooks";
```

---

## Conventions

- All hooks follow SolidJS reactive patterns
- Tauri commands are wrapped with error handling
- Event listeners are properly cleaned up with `onCleanup`
- TypeScript types are exported for prop definitions

## Tauri Integration

Hooks invoke backend commands with `invoke()` and listen to events for progress and system stats.
