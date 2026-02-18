# FRONTEND_API_NOTES.md — SolidJS + TypeScript Reference for AI Agents

> **Purpose:** Prevent wrong assumptions about frontend APIs, hook signatures,
> component patterns, type shapes, and import paths. Consult this BEFORE writing
> any new frontend code.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Provider & Context Hierarchy](#provider--context-hierarchy)
3. [AppContext — Central State Access](#appcontext--central-state-access)
4. [Hooks API Reference](#hooks-api-reference)
   - [useFileManager](#usefilemanager)
   - [useHashManager](#usehashmanager)
   - [useProject](#useproject)
   - [useDatabase](#usedatabase)
   - [useAppState](#useappstate)
   - [useCenterPaneTabs](#usecenterpantabs)
   - [useLazyLoading](#uselazyloading)
   - [useEntrySource](#useentrysource)
   - [useEntryNavigation](#useentrynavigation)
   - [useActivityManager](#useactivitymanager)
   - [useProjectActions](#useprojectactions)
   - [useProjectDbSync](#useprojectdbsync-write-through-to-ffxdb)
   - [useProjectDbRead](#useprojectdbread-seed-ffxdb-from-cffx)
   - [usePanelResize](#usepanelresize)
   - [useHistory](#usehistory)
   - [useTheme](#usetheme)
   - [useFocusTrap](#usefocustrap)
   - [useProcessedDatabases](#useprocesseddatabases)
   - [useKeyboardHandler](#usekeyboardhandler)
   - [useWindowTitle](#usewindowtitle)
   - [useCloseConfirmation](#usecloseconfirmation)
   - [useAppActions](#useappactions-factory-functions)
   - [useCommandPalette](#usecommandpalette)
   - [useDatabaseEffects](#usedatabaseeffects)
   - [usePreferenceEffects](#usepreferenceeffects)
   - [useActivityLogging](#useactivitylogging)
   - [useProjectRecovery](#useprojectrecovery)
   - [useWorkspaceProfiles](#useworkspaceprofiles)
   - [useProjectTemplates](#useprojecttemplates)
   - [useActivityTimeline](#useactivitytimeline)
   - [useProjectComparison](#useprojectcomparison)
   - [hashUtils (Pure Functions)](#hashutils-pure-functions)
5. [API Layer (Tauri Commands)](#api-layer-tauri-commands)
6. [Type System](#type-system)
   - [Core Container Types](#core-container-types)
   - [Lazy Loading Types](#lazy-loading-types)
   - [Hash Types](#hash-types)
   - [Type Sync Map](#type-sync-map)
7. [Utility Functions](#utility-functions)
   - [Formatting Utilities (src/utils.ts)](#formatting-utilities)
   - [Path Utilities (src/utils/pathUtils.ts)](#path-utilities)
   - [File Type Detection (src/utils/fileTypeUtils.ts)](#file-type-detection)
   - [Metadata Utilities (src/utils/metadata.ts)](#metadata-utilities)
   - [Platform Utilities (src/utils/platform.ts)](#platform-utilities)
   - [Error Utilities](#error-utilities)
   - [Logger](#logger)
   - [Accessibility Utilities](#accessibility-utilities)
   - [Performance Utilities](#performance-utilities)
   - [Telemetry Utilities](#telemetry-utilities)
   - [Processed Database Utilities](#processed-database-utilities)
8. [Component Patterns](#component-patterns)
   - [UI Component Library](#ui-component-library)
   - [Icons](#icons)
   - [Toast Notifications](#toast-notifications)
   - [Context Menus](#context-menus)
   - [Command Palette](#command-palette)
   - [Error Boundaries](#error-boundaries)
   - [VirtualList](#virtuallist-performance-critical)
   - [Preferences API](#preferences-api)
   - [Drag & Drop](#drag--drop)
   - [Search Panel](#search-panel)
   - [Export Panel](#export-panel)
9. [Constants Reference](#constants-reference)
10. [Import Path Conventions](#import-path-conventions)
11. [Common Gotchas & Anti-Patterns](#common-gotchas--anti-patterns)

---

## Architecture Overview

```text
src/
├── index.tsx              # Entry point → renders AppRouter
├── AppRouter.tsx           # Provider tree: ToastProvider > HistoryProvider > App
├── App.tsx                 # Main layout, initializes all hooks
├── api/                    # Tauri command wrappers (type-safe invoke calls)
│   ├── commands.ts         # Namespace: commands.container / .hash / .database / .system
│   ├── fileExport.ts       # Export operations with progress
│   └── archiveCreate.ts    # Archive creation API
├── components/             # UI components (see Component Patterns section)
│   ├── icons/index.tsx     # Centralized Heroicon re-exports + FileIcon helper
│   ├── ui/                 # Primitives: Button, Input, Modal, Badge, Card, etc.
│   ├── layout/             # CenterPane, layout structure
│   ├── tree/               # Evidence tree components
│   └── [Feature].tsx       # Feature components (HexViewer, PdfViewer, etc.)
├── constants/
│   └── ui.ts               # Numeric constants, type mappings, keyboard shortcuts
├── context/
│   ├── AppContext.tsx       # Central context: useAppContext(), useServices(), useManagers()
│   └── index.ts            # Re-exports
├── hooks/                  # All reactive hooks (see Hooks API Reference)
│   ├── index.ts            # Barrel export — import everything from here
│   ├── project/            # useProject sub-modules (modular architecture)
│   └── [useXxx].ts         # Individual hook files
├── styles/
│   └── variables.css       # Design tokens (CSS custom properties)
├── types/                  # TypeScript type definitions
│   ├── types.ts            # Barrel re-export
│   ├── container.ts        # TreeEntry, ArchiveTreeEntry, DiscoveredFile
│   ├── containerInfo.ts    # Ad1Info, EwfInfo, ContainerInfo
│   ├── hash.ts             # HASH_ALGORITHMS, HashError, StoredHashEntry
│   ├── lazy-loading.ts     # LazyTreeEntry, LazyLoadResult, ContainerSummary
│   ├── viewer.ts           # FileChunk, ParsedMetadata, FileTypeInfo
│   ├── project.ts          # Project file types (.cffx)
│   ├── database.ts         # DbSession, DbFileRecord, DbHashRecord
│   ├── vfs.ts              # VfsEntry, VfsPartitionInfo
│   ├── processed.ts        # ProcessedDatabase, ProcessedDbEntry
│   └── caseDocument.ts     # CaseDocument, CaseDocumentType
├── utils/                  # Pure utility functions
│   ├── index.ts            # Barrel export
│   ├── pathUtils.ts        # getExtension, getBasename, joinPath, normalizePath
│   ├── fileTypeUtils.ts    # isImage, isEmail, isPdf, detectFileType, etc.
│   ├── metadata.ts         # formatDate, truncateHash, formatNumber, etc.
│   ├── platform.ts         # isMac, isWindows, isPrimaryModifier, formatShortcut
│   ├── performance.ts      # useRenderTracker, getMemoryMetrics, useFPSMonitor
│   ├── accessibility.ts    # announce (screen reader), getFocusableElements
│   ├── errorUtils.ts       # getErrorMessage (safe error extraction)
│   ├── logger.ts           # logger.info/warn/error/debug
│   └── telemetry.ts        # logError, initGlobalErrorHandlers
└── utils.ts                # Root-level formatters (formatBytes, debounce, etc.)
```

---

## Provider & Context Hierarchy

```text
<ToastProvider>                    ← Toast notification system
  <HistoryProvider>                ← Undo/redo command history
    <App>                          ← Main application
      <AppProvider>                ← Central context (all managers)
        {children}
      </AppProvider>
    </App>
  </HistoryProvider>
</ToastProvider>
```

**Key rule:** `AppProvider` is inside `App` — it wraps the main content but is
itself inside `ToastProvider` and `HistoryProvider`. Access toast via
`useToast()` (from `components`), history via `useHistoryContext()` (from
`hooks`).

---

## AppContext — Central State Access

```typescript
import { useAppContext, useServices, useManagers } from "../context";

// Full context
const ctx = useAppContext();
ctx.toast          // ToastProvider return
ctx.history        // HistoryProvider return (undo/redo)
ctx.preferences    // createPreferences() return
ctx.themeActions   // Theme toggle/cycle
ctx.db             // useDatabase() return
ctx.fileManager    // useFileManager() return
ctx.hashManager    // useHashManager() return
ctx.projectManager // useProject() return
ctx.processedDbManager // useProcessedDatabases() return

// Convenience shortcuts
const { toast, history, preferences, themeActions } = useServices();
const { db, fileManager, hashManager, projectManager, processedDbManager } = useManagers();
```

**⚠️ Gotcha:** `useAppContext()` throws if used outside `<AppProvider>`. Always
check component is inside the provider tree.

---

## Hooks API Reference

All hooks are exported from `src/hooks/index.ts`. Import from `"../hooks"`.

### useFileManager

**Returns:** `FileManager` (type alias for return type)

```typescript
const fm = useFileManager();

// === State (Signals) ===
fm.scanDir()                    // Accessor<string> — current scan directory
fm.setScanDir(dir)              // Setter<string>
fm.recursiveScan()              // Accessor<boolean>
fm.setRecursiveScan(val)        // Setter<boolean>
fm.discoveredFiles()            // Accessor<DiscoveredFile[]>
fm.selectedFiles()              // Accessor<Set<string>> — selected file paths
fm.setSelectedFiles(set)        // Setter<Set<string>>
fm.activeFile()                 // Accessor<DiscoveredFile | null>
fm.setActiveFile(file)          // Setter
fm.hoveredFile()                // Accessor<string | null> — hovered file PATH (not object)
fm.setHoveredFile(path)         // Setter<string | null>
fm.fileInfoMap()                // Accessor<Map<string, ContainerInfo>>
fm.setFileInfoMap(map)          // Setter<Map<string, ContainerInfo>>
fm.fileStatusMap()              // Accessor (per-file status tracking)
fm.tree()                       // Accessor<TreeEntry[]>
fm.treeFilter()                 // Accessor<string>
fm.setTreeFilter(filter)        // Setter<string>
fm.typeFilter()                 // Accessor<string | null> — single type filter (NOT Set)
fm.setTypeFilter(type)          // Setter<string | null>
fm.focusedFileIndex()           // Accessor (keyboard navigation index)
fm.setFocusedFileIndex(idx)     // Setter
fm.busy()                       // Accessor<boolean>
fm.statusMessage()              // Accessor<string>
fm.statusKind()                 // Accessor<"idle" | "working" | "ok" | "error"> — includes "idle"
fm.systemStats()                // Accessor<SystemStats | null>
fm.loadProgress()               // Accessor<{ show: boolean; title: string; message: string; current: number; total: number; cancelled: boolean }>

// === Computed (Memos) ===
fm.filteredFiles()              // Accessor<DiscoveredFile[]>
fm.allFilesSelected()           // Accessor<boolean>
fm.selectedCount()              // Accessor<number>
fm.filteredTree()               // Accessor<TreeEntry[]>
fm.totalSize()                  // Accessor<number>
fm.containerStats()             // Accessor<{ ad1: number, e01: number, ... }>

// === Actions ===
fm.browseScanDir()              // Open directory picker dialog
fm.scanForFiles()               // Scan scanDir for evidence files
fm.loadFileInfo(file)           // Load container info for one file
fm.loadAllInfo()                // Load info for all discovered files
fm.selectAndViewFile(file)      // Select + activate + load info
fm.toggleTypeFilter(type)       // Toggle type in filter set
fm.toggleFileSelection(path)    // Toggle file in selection set
fm.toggleSelectAll()            // Select/deselect all
fm.cancelLoading()              // Cancel in-progress loading
fm.handleFileListKeyDown(e)     // Keyboard navigation handler
fm.setupSystemStatsListener()   // Start system stats event listener
fm.addDiscoveredFile(file)      // Manually add file (e.g., nested)
fm.clearAll()                   // Reset all state
fm.restoreDiscoveredFiles(files)  // For project restore
fm.restoreFileInfoMap(map)        // For project restore
fm.setWorking(msg)              // Set status to working
fm.setOk(msg)                   // Set status to ok
fm.setError(msg)                // Set status to error
fm.updateFileStatus(path, status) // Update per-file status
```

### useHashManager

**Requires:** `useHashManager(fileManager: FileManager)`

```typescript
const hm = useHashManager(fileManager);

// === State ===
hm.selectedHashAlgorithm()     // Accessor<HashAlgorithmName>
hm.setSelectedHashAlgorithm(a) // Setter
hm.fileHashMap()                // Accessor<Map<string, FileHashInfo>>
hm.setFileHashMap(map)          // Setter<Map<string, FileHashInfo>>
hm.hashHistory()                // Accessor<Map<string, HashHistoryEntry[]>>

// === Actions ===
hm.hashSingleFile(path)        // Hash one file with selected algorithm
hm.hashSelectedFiles()         // Hash all selected files
hm.hashAllFiles()              // Hash all discovered files
hm.importStoredHashesToHistory(path, hashes)  // Import container-stored hashes
hm.importPreloadedStoredHashes(path, hashes)  // Import from project load
hm.addTransferHashesToHistory(path, hashes)   // Import from file transfer
hm.restoreHashHistory(history)   // For project restore
hm.restoreFileHashMap(map)       // For project restore
hm.clearAll()                    // Reset all state

// === Helpers ===
hm.getAllStoredHashesSorted(path)  // Get sorted hash history for a file
hm.formatHashDate(timestamp)      // Format hash timestamp
hm.recordHashToHistory(path, entry) // Record individual hash entry
```

### useProject

**Returns:** Project lifecycle manager with sub-module composition.

```typescript
const pm = useProject();

// === State ===
pm.project()                   // Accessor<FFXProject | null>
pm.projectPath()               // Accessor<string | null>
pm.modified()                  // Accessor<boolean>
pm.error()                     // Accessor<string | null> 
pm.loading()                   // Accessor<boolean>
pm.currentUser()               // Accessor<string>
pm.currentSessionId()          // Accessor<string>
pm.autoSaveEnabled()           // Accessor<boolean>
pm.lastAutoSave()              // Accessor<Date | null>

// === Derived ===
pm.hasProject()                // Accessor<boolean>
pm.projectName()               // Accessor<string>
pm.needsSave()                 // Accessor<boolean>
pm.bookmarkCount()             // Accessor<number>
pm.noteCount()                 // Accessor<number>
pm.recentSearches()            // Accessor<string[]>
pm.projectLocations()          // Accessor<ProjectLocations | null>
pm.rootPath()                  // Accessor<string | null>

// === Lifecycle ===
pm.checkProjectExists(dirPath)   // Check for .cffx file
pm.getDefaultProjectPath(dir)    // Suggest project path
pm.createProject(path, name)     // Create new .cffx project
pm.clearProject()                // Close current project

// === Save/Load ===
pm.saveProject(options)          // Save to current path
pm.saveProjectAs(path, options)  // Save to new path
pm.loadProject(path)             // Load .cffx project

// === State Updates ===
pm.markModified()                // Flag project as unsaved
pm.updateUIState(state)          // Update UI state in project
pm.updateLocations(locs)         // Update project directory locations
pm.logActivity(entry)            // Add activity log entry

// === Bookmarks ===
pm.addBookmark(bookmark)
pm.updateBookmark(id, updates)
pm.removeBookmark(id)

// === Notes ===
pm.addNote(note)
pm.updateNote(id, updates)
pm.removeNote(id)

// === Search ===
pm.addRecentSearch(query)

// === Processed Databases ===
pm.updateProcessedDbIntegrity(...)  // Update processed DB integrity state

// === Auto-save ===
pm.setAutoSaveEnabled(enabled)
pm.setAutoSaveCallback(fn)
pm.startAutoSave()
pm.stopAutoSave()
```

### useDatabase

```typescript
const db = useDatabase();

// === State ===
db.session()                    // Accessor<DbSession | null>
db.files()                      // Accessor<Map<string, DbFileRecord>>
db.isLoading()                  // Accessor<boolean>
db.error()                      // Accessor<string | null>

// === Session ===
db.initSession(rootPath)        // Create/load session for directory
db.restoreLastSession()         // Restore previous session

// === File ops ===
db.saveFile(discoveredFile)     // Upsert file record
db.getFileRecord(path)          // Get cached file record

// === Hash ops ===
db.saveHash(filePath, algorithm, hashValue, source?, segmentIndex?, segmentName?)
db.getFileHashes(filePath)      // Get hash history

// === Verification ===
db.saveVerification(hashId, result, expected, actual)
db.getFileVerifications(filePath)

// === Tabs ===
db.saveTabs(filePaths, activeFilePath?)
db.loadTabs()

// === Settings ===
db.setSetting(key, value)
db.getSetting(key)
db.getSettingWithDefault(key, defaultValue)
```

### useAppState

```typescript
const state = useAppState();

state.modals.showCommandPalette()     // Accessor<boolean> + setter
state.modals.showSettingsPanel()
state.modals.showSearchPanel()
state.modals.showReportWizard()
state.modals.showProjectWizard()
state.modals.showWelcomeModal()
state.modals.showShortcutsModal()
state.modals.showPerformancePanel()

state.views.openTabs()                // Accessor<OpenTab[]>
state.views.currentViewMode()         // Accessor<TabViewMode>
state.views.hexMetadata()             // Accessor<ParsedMetadata | null>
state.views.selectedContainerEntry()  // Accessor<SelectedEntry | null>
state.views.entryContentViewMode()    // Accessor<"auto" | "hex" | "text" | "document">
state.views.requestViewMode()         // Accessor<"info" | "hex" | "text" | "pdf" | "export" | null>
state.views.hexNavigator()            // Accessor<((offset, size?) => void) | null>
state.views.treeExpansionState()      // Accessor<TreeExpansionState | null>

state.project.pendingProjectRoot()    // Accessor<string | null>
state.project.caseDocumentsPath()     // Accessor<string | null>
state.project.caseDocuments()         // Accessor<CaseDocument[] | null>

state.leftPanel.leftPanelTab()        // Accessor<"dashboard" | "evidence" | "processed" | "casedocs" | "activity" | "bookmarks">
state.leftPanel.leftPanelMode()       // Accessor<"tabs" | "unified">
```

### useCenterPaneTabs

```typescript
const tabs = useCenterPaneTabs();

// === State ===
tabs.tabs()                           // Accessor<CenterTab[]>
tabs.activeTabId()                    // Accessor<string | null>
tabs.viewMode()                       // Accessor<CenterPaneViewMode>
tabs.activeTab()                      // Accessor<CenterTab | null>
tabs.activeTabType()                  // Accessor<CenterTabType | null>
tabs.recentlyClosed()                 // Accessor<Set<string>>

// === Tab Types ===
// CenterTabType = "evidence" | "casedoc" | "entry" | "processed" | "export"
// CenterPaneViewMode = "info" | "hex" | "text" | "pdf" | "export" | "document"

// === Actions ===
tabs.openEvidenceFile(file: DiscoveredFile)
tabs.openCaseDocument(doc: CaseDocument)
tabs.openContainerEntry(entry: SelectedEntry)
tabs.openProcessedDatabase(db: ProcessedDatabase)
tabs.openExportTab()
tabs.closeTab(tabId)
tabs.closeAllTabs()
tabs.clearRecentlyClosed()
```

### useLazyLoading

```typescript
const ll = useLazyLoading(() => containerPath, { autoLoad: true });

// === State ===
ll.summary()                    // Accessor<ContainerSummary | null>
ll.rootChildren()               // Accessor<LazyTreeEntry[]>
ll.rootTotalCount()             // Accessor<number>
ll.isLoadingRoot()              // Accessor<boolean>
ll.isLoading()                  // Accessor<boolean>
ll.error()                      // Accessor<Error | null>
ll.config()                     // Accessor<LazyLoadConfig>
ll.childrenCache()              // Accessor<Map<string, LazyTreeEntry[]>>
ll.hasMoreRoot()                // Accessor<boolean>

// === Actions ===
ll.loadSummary()
ll.loadRootChildren(offset?, limit?)
ll.loadMoreRoot()               // Pagination: load next batch
ll.loadChildren(parentPath, offset?, limit?)
ll.getCachedChildren(parentPath)
ll.hasChildren(parentPath)
ll.clearCache()
ll.updateSettings(settings)
ll.refreshSettings()
```

### useEntrySource

**Pure functions** (not a hook — no signals):

```typescript
import { readBytesFromSource, readTextFromSource, getSourceKey, getSourceFilename } from "../hooks";

// Read bytes from any source type
const result: ByteReadResult = await readBytesFromSource(
  source: "disk" | "ad1" | "vfs" | "archive" | "nested",
  containerPath: string,     // Path to container
  entryPath: string,         // Path within container
  offset?: number,           // Byte offset
  length?: number            // Bytes to read
);
// result.bytes: number[] | null
// result.totalSize: number

// Read text from any source type
const text: TextReadResult = await readTextFromSource(source, containerPath, entryPath);
// text.content: string | null
// text.encoding: string

// Utility: create unique key for an entry
const key = getSourceKey(source, containerPath, entryPath); // string
const name = getSourceFilename(source, containerPath, entryPath); // string
```

### useEntryNavigation

```typescript
const nav = useEntryNavigation(deps: UseEntryNavigationDeps);
// Handles click-to-open for evidence files, container entries,
// case documents, processed databases, nested containers
```

### useActivityManager

```typescript
const am = useActivityManager();
// Tracks running backend operations, supports cancel/pause/resume/clear
```

### useProjectActions

```typescript
const pa = useProjectActions(deps: UseProjectActionsDeps);
// Bundles save, saveAs, load, openDirectory, projectSetupComplete
```

### useProjectDbSync (Write-Through to .ffxdb)

```typescript
import { dbSync } from "./hooks/project/useProjectDbSync";

// Fire-and-forget sync — .cffx remains source of truth
// All methods are non-blocking; failures logged but never thrown
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

```typescript
import { seedDatabaseFromProject } from "./hooks/project/useProjectDbRead";

// Called once during project load (in projectHelpers.ts)
// Seeds empty .ffxdb tables from loaded .cffx project state
// Idempotent: checks DB stats before seeding each table
await seedDatabaseFromProject(project);
```

### usePanelResize

```typescript
const panel = usePanelResize({
  initialWidth: 320,
  minWidth: 150,
  maxWidth: 600,
  side: "left" | "right",
});

panel.width()         // Accessor<number>
panel.collapsed()     // Accessor<boolean>
panel.startDrag(e)    // MouseDown handler
panel.toggleCollapse()

// For dual panels:
const dual = useDualPanelResize(options);
```

### useHistory

```typescript
const [state, actions] = useHistory({ maxHistory: 50 });

actions.execute(command: Command)   // Execute & record a command
actions.undo()                      // Undo last command
actions.redo()                      // Redo undone command
state.canUndo                       // boolean
state.canRedo                       // boolean
state.history                       // Command[]
state.position                      // number
```

### useTheme

```typescript
const themeActions = createThemeActions(
  () => preferences.preferences().theme,
  (theme) => preferences.updatePreference("theme", theme)
);

themeActions.theme()           // Accessor<Theme> — "light" | "dark" | "system"
themeActions.resolvedTheme()   // Accessor<ResolvedTheme> — "light" | "dark"
themeActions.toggle()          // Cycle through themes
themeActions.setTheme(theme)

// Helper functions (not hook-based):
getThemeIcon(theme)   // Returns icon component
getThemeLabel(theme)  // Returns display label
getSystemTheme()      // Returns current OS theme
resolveTheme(theme)   // Resolve "system" to actual theme
applyTheme(theme)     // Apply to DOM
getNextTheme(theme)   // Cycle: light → dark → system → light
```

### useFocusTrap

```typescript
// Traps focus within a container (for modals, dialogs)
const trapRef = useFocusTrap(() => containerRef(), () => isOpen());
// Alternative:
const cleanup = createFocusTrap(element);
```

### useProcessedDatabases

```typescript
const pdm = useProcessedDatabases();
// Manages AXIOM, PA, etc. processed database loading and display
```

### useKeyboardHandler

```typescript
useKeyboardHandler(deps: KeyboardHandlerDeps);
// Registers global keyboard shortcuts (Ctrl+S, Ctrl+Z, etc.)
```

### useWindowTitle

```typescript
const wt = useWindowTitle({
  projectName: () => "My Project",    // Accessor<string | null>
  modified: () => false,               // Accessor<boolean>
  projectPath: () => "/path/to/proj",  // Accessor<string | null> (optional)
});
// Title formats: "CORE-FFX", "My Project — CORE-FFX", "● My Project — CORE-FFX" (modified)

// Standalone function (no hook needed)
await setWindowTitle("Custom Title");
```

### useCloseConfirmation

```typescript
const close = useCloseConfirmation({
  hasUnsavedChanges: () => pm.modified(),  // Accessor<boolean>
  onSave: async () => { await save(); return true; },
  onClose: () => cleanup(),
  dialogTitle: "Unsaved Changes",
});

// Standalone confirmation dialog
const proceed = await confirmUnsavedChanges({
  title: "Close project?",
  message: "You have unsaved changes.",
});
```

### useAppActions (Factory Functions)

```typescript
import { createSearchHandlers, createContextMenuBuilders } from "../hooks";

// Creates search action handlers
const searchHandlers = createSearchHandlers({ fileManager });

// Creates context menu item builders for files/entries
const menuBuilders = createContextMenuBuilders({
  fileManager, hashManager, projectManager, toast,
  buildSaveOptions: () => ({...}),
});
```

### useCommandPalette

```typescript
import { createCommandPaletteActions, type CommandPaletteConfig } from "../hooks";

const getActions = createCommandPaletteActions({
  fileManager, hashManager, projectManager, toast,
  /* ... view mode setters, panel toggles */
});
// Returns () => CommandAction[] — call inside component to get action list
```

### useDatabaseEffects

```typescript
useDatabaseEffects({
  db,                    // useDatabase() return
  fileManager,           // useFileManager() return
  hashManager,           // useHashManager() return
  projectManager,        // useProject() return
  enabled: () => true,   // Accessor<boolean>
});
// Auto-persists state changes to SQLite — no return value, pure side-effect hook
```

### usePreferenceEffects

```typescript
usePreferenceEffects(preferences: Accessor<AppPreferences>);
// Applies preferences to DOM (font-size, animations, etc.) — pure side-effect
```

### useActivityLogging

```typescript
useActivityLogging({
  fileManager,
  hashManager,
  projectManager,
});
// Logs user actions to project activity timeline — pure side-effect
```

### useProjectRecovery

```typescript
const recovery = useProjectRecovery();

// State
recovery.backups()        // Accessor<BackupFile[]>
recovery.health()         // Accessor<ProjectHealth | null>
recovery.versions()       // Accessor<BackupMetadata[]>
recovery.recoveryInfo()   // Accessor<RecoveryInfo | null>
recovery.loading()        // Accessor<boolean>
recovery.error()          // Accessor<string | null>

// Actions
recovery.createBackup(projectPath, type: BackupType)
recovery.createVersionBackup(projectPath, description)
recovery.listVersions(projectPath)
recovery.checkRecovery(projectPath)
recovery.recoverFromAutosave(projectPath)
recovery.clearAutosave(projectPath)
recovery.checkHealth(projectPath)

// Helpers
recovery.getHealthStatusColor(status: ProjectHealthStatus)  // CSS class
recovery.getSeverityColor(severity: IssueSeverity)           // CSS class

// Types
type BackupType = "ManualSave" | "AutoSave" | "ManualBackup" | "PreOperation";
type ProjectHealthStatus = "Healthy" | "Warning" | "Critical";
```

### useWorkspaceProfiles

```typescript
const wp = useWorkspaceProfiles();

// State
wp.profiles()        // Accessor<ProfileSummary[]>
wp.currentProfile()  // Accessor<WorkspaceProfile | null>
wp.loading()         // Accessor<boolean>
wp.error()           // Accessor<string | null>

// Actions
wp.listProfiles()
wp.getProfile(name: string)
wp.getActiveProfile()
wp.setActiveProfile(name: string)
wp.addProfile(profile: WorkspaceProfile)
wp.updateProfile(name: string, updates: Partial<WorkspaceProfile>)
wp.deleteProfile(name: string)
wp.cloneProfile(source: string, newName: string)
wp.exportProfile(name: string)

// Types
type ProfileType = "Investigation" | "Review" | "Triage" | "Analysis" | "Custom";
type CenterLayout = "Single" | "SplitVertical" | "SplitHorizontal" | "Grid";
```

### useProjectTemplates

```typescript
const pt = useProjectTemplates();

// State
pt.templates()        // Accessor<TemplateSummary[]>
pt.currentTemplate()  // Accessor<ProjectTemplate | null>
pt.loading()          // Accessor<boolean>

// Actions
pt.listTemplates()
pt.getTemplate(id: string)
pt.applyTemplate(templateId, projectPath)
pt.createFromProject(projectPath, name, category)
pt.deleteTemplate(id)
pt.exportTemplate(id)
pt.importTemplate(data)

// Utilities
pt.getByCategory(category: TemplateCategory)
pt.isBuiltinTemplate(id)
pt.getBuiltinTemplates()
pt.getCustomTemplates()

type TemplateCategory = "DFIR" | "Mobile" | "Network" | "Memory" | "Malware" | "Custom";
```

### useActivityTimeline

```typescript
const at = useActivityTimeline();

// State
at.visualization()  // Accessor<TimelineVisualization | null>
at.loading()        // Accessor<boolean>

// Actions
at.computeVisualization(activities)
at.exportTimeline(format)
at.exportTimelineJson()

// Utilities
at.getHeatmapData()
at.getMostActivePeriods()
at.getTypeBreakdown()
at.getUserSummary()
at.getTrends()
```

### useProjectComparison

```typescript
const pc = useProjectComparison();
// Compares two .cffx project files, shows diffs in bookmarks, notes, evidence
// Returns: ProjectComparison with BookmarkDiff[], NoteDiff[], EvidenceDiff[]
```

### hashUtils (Pure Functions)

**Import from:** `"../hooks"` (re-exported via barrel)

```typescript
// Extract stored hashes from any container type
await extractStoredHashes(path: string, extension: string): Promise<StoredHashEntry[]>
// Dispatches to: extractE01StoredHashes, extractL01StoredHashes,
//   extractAd1StoredHashes, extractUfedStoredHashes

// Hash any container type with selected algorithm
await hashContainer(path: string, extension: string, algorithm: HashAlgorithmName): Promise<string>
// Dispatches to: hashE01Container, hashL01Container, hashAd1Container,
//   hashUfedContainer, hashRawContainer

// Progress event setup
await setupProgressListener(callback: (progress: ProgressPayload) => void): Promise<UnlistenFn>

// Comparison utilities
compareHashes(hash1, hash2, algorithm1, algorithm2): CompareResult
findMatchingStoredHash(computed, storedHashes): StoredHashEntry | null
hasStoredHashForAlgorithm(algorithm, storedHashes): boolean
deduplicateStoredHashes(hashes: StoredHashEntry[]): StoredHashEntry[]
groupHashesByAlgorithm(hashes): Map<string, StoredHashEntry[]>
```

---

## API Layer (Tauri Commands)

Use `src/api/commands.ts` for type-safe backend calls:

```typescript
import { commands } from "../api/commands";

// Container operations
const summary = await commands.container.getSummary(path);
const root = await commands.container.getRootChildren(path);
const children = await commands.container.getChildrenAtAddr(path, addr, parentPath);
const settings = await commands.container.getSettings();
await commands.container.updateSettings(config);

// Hash operations
await commands.hash.queue.resume();
await commands.hash.queue.pause();
await commands.hash.queue.clearCompleted();

// Database operations
await commands.database.upsertFile(fileRecord);
await commands.database.insertHash(hashRecord);
await commands.database.insertVerification(verificationRecord);
await commands.database.saveOpenTabs(sessionId, tabs);
await commands.database.setSetting(key, value);

// System
await commands.system.openPath(path);
```

**⚠️ For raw Tauri calls** (commands not in the wrapper):

```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Direct invoke
const info = await invoke<ContainerInfo>("logical_info", { path });

// Listen for progress events
const unlisten = await listen<ProgressPayload>("verify-progress", (event) => {
  setProgress(event.payload.percent);
});
// ALWAYS clean up listeners:
onCleanup(() => unlisten());
```

---

## Type System

### Core Container Types

**Import from:** `"../types"` (barrel re-export)

```typescript
// AD1-specific entry (mirrors Rust ad1::TreeEntry)
interface TreeEntry {
  path: string;
  name: string;
  is_dir: boolean;                    // ⚠️ snake_case (from Rust serde)
  size: number;
  item_type: number;
  first_child_addr?: number | null;   // For lazy loading
  data_addr?: number | null;          // For reading content
  item_addr?: number | null;          // Hex location
  md5_hash?: string | null;
  sha1_hash?: string | null;
  created?: string | null;            // ISO 8601
  accessed?: string | null;
  modified?: string | null;
  child_count?: number | null;
}

// Archive entry (ZIP, 7z, etc.)
interface ArchiveTreeEntry {
  path: string;
  name: string;
  isDir: boolean;                     // ⚠️ camelCase (from Rust #[serde(rename_all = "camelCase")])
  size: number;
  compressedSize: number;
  crc32: number;
  modified: string;
}

// File discovery result
interface DiscoveredFile {
  path: string;
  filename: string;
  container_type: string;             // ⚠️ snake_case
  size: number;
  segment_count?: number;
  created?: string;
  modified?: string;
}
```

**⚠️ CRITICAL:** Some types use `snake_case` (matching Rust's default serde)
and some use `camelCase` (from `#[serde(rename_all = "camelCase")]`).
Check the Rust struct's serde attribute before assuming field names.

### Lazy Loading Types

**Import from:** `"../types/lazy-loading"`

```typescript
interface LazyTreeEntry {
  id: string;                 // Unique within container
  name: string;
  path: string;
  is_dir: boolean;            // ⚠️ snake_case
  size: number;
  entry_type: string;
  child_count: number;        // -1 if unknown
  children_loaded: boolean;
  hash: string | null;
  modified: string | null;
  metadata: string | null;    // JSON string
}

interface LazyLoadResult {
  entries: LazyTreeEntry[];
  total_count: number;
  has_more: boolean;
  next_offset: number;
  lazy_loaded: boolean;
  config: LazyLoadConfig;
}

interface ContainerSummary {
  path: string;
  container_type: string;
  total_size: number;
  entry_count: number;
  root_entry_count: number;
  lazy_loading_recommended: boolean;
  estimated_load_time_ms: number | null;
}

interface LazyLoadConfig {
  enabled: boolean;
  batch_size: number;                  // Default: 100
  auto_expand_threshold: number;       // Default: 50
  large_container_threshold: number;   // Default: 10,000
  pagination_threshold: number;        // Default: 500
  show_entry_count: boolean;
  count_timeout_ms: number;            // Default: 5,000
  load_timeout_ms: number;             // Default: 30,000
}
```

### Hash Types

**Import from:** `"../types/hash"` or `"../types"`

```typescript
// Canonical algorithm names — use these constants, NOT raw strings
const HASH_ALGORITHMS = {
  MD5: 'MD5',
  SHA1: 'SHA-1',      // ⚠️ Hyphenated
  SHA256: 'SHA-256',   // ⚠️ Hyphenated
  SHA512: 'SHA-512',
  BLAKE3: 'BLAKE3',
  BLAKE2: 'BLAKE2b',  // ⚠️ Lowercase 'b'
  XXH3: 'XXH3',
  XXH64: 'XXH64',
  CRC32: 'CRC32',
} as const;

type HashAlgorithmName = typeof HASH_ALGORITHMS[keyof typeof HASH_ALGORITHMS];

// Stored hash entry (from container metadata or computed)
interface StoredHashEntry {
  algorithm: string;
  hash: string;                    // Lowercase hex
  timestamp?: string | null;
  source: 'container' | 'companion' | 'computed';
  filename?: string | null;
  verified?: boolean | null;
}

// File hash info (current computed hash)
interface FileHashInfo {
  algorithm: string;
  hash: string;
  verified?: boolean | null;
}

// Hash error class with error codes
class HashError extends Error {
  code: HashErrorCode;
  context?: Record<string, unknown>;
  is(code: HashErrorCode): boolean;
  getUserMessage(): string;
}

type HashErrorCode =
  | 'SEGMENT_MISSING'
  | 'VERIFICATION_FAILED'
  | 'UNSUPPORTED_FORMAT'
  | 'FILE_READ_ERROR'
  | 'PROGRESS_TIMEOUT'
  | 'COMPUTATION_ERROR'
  | 'INVALID_ALGORITHM';
```

### Type Sync Map

These TypeScript files must stay aligned with their Rust counterparts:

| TypeScript File | Rust Source |
|-----------------|------------|
| `src/types/container.ts` | `src-tauri/src/containers/types.rs`, `src-tauri/src/formats.rs` |
| `src/types/containerInfo.ts` | `src-tauri/src/containers/types.rs` |
| `src/types/lazy-loading.ts` | `src-tauri/src/commands/lazy_loading.rs`, `src-tauri/src/common/lazy_loading.rs` |
| `src/types/viewer.ts` | `src-tauri/src/viewer/document/types.rs` |
| `src/types/project.ts` | `src-tauri/src/project.rs` |
| `src/types/database.ts` | `src-tauri/src/database.rs` |
| `src/types/projectDb.ts` | `src-tauri/src/project_db.rs`, `src-tauri/src/commands/project_db.rs` |
| `src/types/processed.ts` | `src-tauri/src/processed/types.rs` |
| `src/report/types.ts` | `src-tauri/src/report/types.rs` |

---

## Utility Functions

### Formatting Utilities

**Import from:** `"../utils"` (root utils.ts)

```typescript
formatBytes(value: number): string           // 1024 → "1.00 KB"
normalizeError(err: unknown): string         // Safe error-to-string
typeClass(type: string): string              // Container type → CSS class
parseTimestamp(timestamp: string): Date | null
formatHashDate(timestamp: string): string
formatDateByPreference(date, includeTime?): string
formatDuration(seconds: number): string      // 125 → "2m 5s"
debounce<T>(fn: T, wait: number): T
formatOffset(offset: number, ...): string    // Hex offset formatting
byteToHex(byte: number): string              // 255 → "FF"
byteToAscii(byte: number): string            // Printable or "."
```

### Path Utilities

**Import from:** `"../utils/pathUtils"` or `"../utils"`

```typescript
getExtension(filename: string): string             // "file.txt" → "txt"
hasExtension(filename, ext): boolean
hasAnyExtension(filename, extensions): boolean
getBasename(path: string): string                  // "/foo/bar.txt" → "bar.txt"
getBasenameWithoutExt(path: string): string         // "/foo/bar.txt" → "bar"
getDirname(path: string): string                    // "/foo/bar.txt" → "/foo"
joinPath(...parts: string[]): string
normalizePath(path: string): string                 // Normalize separators
isAbsolutePath(path: string): boolean
isHiddenFile(path: string): boolean                 // Starts with "."
```

### File Type Detection

**Import from:** `"../utils/fileTypeUtils"` or `"../utils"`

```typescript
isImage(filename): boolean      // jpg, png, gif, bmp, webp, svg, ico, tiff
isVideo(filename): boolean
isAudio(filename): boolean
isDocument(filename): boolean   // pdf, doc, docx, xls, xlsx, ppt, etc.
isSpreadsheet(filename): boolean
isTextDocument(filename): boolean
isCode(filename): boolean
isDatabase(filename): boolean   // sqlite, db, mdb
isRegistryHive(filename): boolean
isArchive(filename): boolean    // zip, 7z, rar, tar, gz
isEmail(filename): boolean      // eml, msg, mbox
isPlist(filename): boolean      // plist
isBinaryExecutable(filename): boolean  // exe, dll, so, dylib
isConfig(filename): boolean
isPdf(filename): boolean

detectFileType(filename): FileTypeCategory
// FileTypeCategory = 'image' | 'video' | 'audio' | 'document' | 'spreadsheet'
//   | 'textDocument' | 'code' | 'database' | 'registryHive' | 'archive'
//   | 'email' | 'plist' | 'binaryExecutable' | 'config' | 'pdf' | 'text' | 'unknown'
```

### Metadata Utilities

**Import from:** `"../utils/metadata"` or `"../utils"`

```typescript
formatDate(dateStr: string | null | undefined): string
formatTimestamp(timestamp: number | null | undefined): string
formatNumber(n: number, options?): string
formatCount(count: number, singular, plural?): string
formatOffset(offset, prefix?): string
formatDecimalOffset(offset, prefix?): string
truncateHash(hash: string, length?: number): string
formatAlgorithm(algorithm: string): string
getVerificationStatus(verified): VerificationStatus
getVerificationIcon(status): string
getVerificationClass(status): string
filterEmptyFields(fields: DisplayField[]): DisplayField[]
groupFieldsByCategory(fields): Map<string, DisplayField[]>
sortFieldsByLabel(fields): DisplayField[]

// Reactive memo creators (for SolidJS)
createHashDisplayMemo(hashAccessor: Accessor<HashInfo | undefined>)
createDateDisplayMemo(dateAccessor)
createTimestampDisplayMemo(timestampAccessor)
```

### Platform Utilities

**Import from:** `"../utils/platform"` or `"../utils"`

```typescript
const platform: "mac" | "windows" | "linux" | "ios" | "android" | "unknown"
isMac: boolean
isWindows: boolean
isLinux: boolean
isTauri: boolean
isMobile: boolean
isDesktop: boolean
isPrimaryModifier(e: KeyboardEvent): boolean   // Cmd on Mac, Ctrl on Win/Linux
formatShortcut(shortcut: string): string        // "mod+s" → "⌘S" (Mac) or "Ctrl+S"
```

### Error Utilities

**Import from:** `"../utils/errorUtils"` or `"../utils"`

```typescript
getErrorMessage(error: unknown): string
// Safe extraction: handles Error objects, strings, objects with .message, null/undefined
// Always returns a string — never throws
```

### Logger

**Import from:** `"../utils/logger"` or `"../utils"`

```typescript
import { logger } from "../utils/logger";

logger.debug("Detailed info", context?);
logger.info("Operation completed", context?);
logger.warn("Potential issue", context?);
logger.error("Something failed", context?);
// context is optional Record<string, unknown>
// Output: structured console logging with timestamp + level
```

### Accessibility Utilities

**Import from:** `"../utils/accessibility"` or `"../utils"`

```typescript
initAnnouncer(): void                        // Initialize ARIA live region (call once at startup)
announce(message: string, priority?: AriaLive): void  // Screen reader announcement
getFocusableElements(container: HTMLElement): HTMLElement[]  // Find all focusable children

type AriaLive = "polite" | "assertive" | "off";
```

### Performance Utilities

**Import from:** `"../utils/performance"` or `"../utils"`

```typescript
setPerformanceMonitoringEnabled(enabled: boolean): void
isPerformanceMonitoringEnabled(): boolean

// SolidJS hooks
useRenderTracker(componentName: string)     // Track component re-renders
trackEffect<T>(name: string, fn: () => T): T    // Wrap createEffect
trackMemo<T>(name: string, fn: () => T): T      // Wrap createMemo
trackAction<T>(name: string, fn: () => Promise<T>): Promise<T>  // Track async actions

getMemoryMetrics(): MemoryMetrics | null    // { usedJSHeapSize, totalJSHeapSize, jsHeapSizeLimit }
```

### Telemetry Utilities

**Import from:** `"../utils/telemetry"` or `"../utils"`

```typescript
logError(error: Error, context?: Record<string, unknown>): void
logInfo(message: string, context?: Record<string, unknown>): void
initGlobalErrorHandlers(): void   // Set up window.onerror + unhandledrejection

type ErrorSeverity = "debug" | "info" | "warning" | "error" | "fatal";
type ErrorCategory = "network" | "runtime" | "user" | "system" | "unknown";
```

### Processed Database Utilities

**Import from:** `"../utils/processed"`

```typescript
ellipsePath(path: string, maxLen?: number): string      // Truncate long paths with "..."
getDbTypeName(dbType: ProcessedDbType): string           // "axiom" → "Axiom"
getDbTypeIcon(dbType: ProcessedDbType): string           // Returns icon identifier
getCategoryIcon(category: string): string                // Returns icon for category
```

---

## Component Patterns

### UI Component Library

**Import from:** `"../components/ui"`

```tsx
import { Button, IconButton, Spinner, Modal } from "../components/ui";
import { Input, Textarea, Select } from "../components/ui";
import { FormField, Card, Badge, SectionHeader, Divider } from "../components/ui";
import { Checkbox, Toggle, Slider } from "../components/ui";

// Button variants
<Button variant="primary" onClick={...}>Save</Button>
<Button variant="secondary">Cancel</Button>
<Button variant="ghost">Skip</Button>
<IconButton onClick={...}><XMark /></IconButton>
<Spinner />  // Loading spinner

// Modal
<Modal isOpen={showModal()} onClose={() => setShowModal(false)} title="Confirm">
  <p>Are you sure?</p>
</Modal>
```

**⚠️ Also available via CSS classes** (see copilot-instructions.md):
`btn btn-primary`, `btn btn-secondary`, `btn btn-ghost`, `input`, `card`, etc.

### Icons

**Import from:** `"../components/icons"`

```tsx
import {
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineMagnifyingGlass,
  HiOutlineXMark,
  HiOutlineCheck,
  HiOutlineArrowPath,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  // ... 100+ icons re-exported from solid-icons/hi
  FileIcon,    // Smart icon based on extension
  getFileIconName,  // Get icon name for extension
} from "../components/icons";

// Usage
<HiOutlineFolder class="w-icon-sm h-icon-sm text-txt-muted" />
<FileIcon filename="report.pdf" class="w-icon-base h-icon-base" />
```

**⚠️ Use icon size utilities from CSS variables:**
- `w-icon-sm h-icon-sm` = 16px
- `w-icon-base h-icon-base` = 20px
- `w-icon-lg h-icon-lg` = 24px

### Toast Notifications

```tsx
import { useToast } from "../components";

const toast = useToast();

toast.success("Saved", "Project saved successfully");
toast.error("Error", "Failed to load container");
toast.warning("Warning", "Missing segment files");
toast.info("Info", "Scan complete");
```

### Context Menus

```tsx
import { createContextMenu, type ContextMenuItem } from "../components";

const { showMenu } = createContextMenu();

const items: ContextMenuItem[] = [
  { label: "Open", icon: <HiOutlineFolderOpen />, action: () => {} },
  { type: "separator" },
  { label: "Delete", icon: <HiOutlineTrash />, action: () => {}, danger: true },
];

<div onContextMenu={(e) => showMenu(e, items)}>Right-click me</div>
```

### Command Palette

```tsx
import { createCommandPalette, type CommandAction } from "../components";

const actions: CommandAction[] = [
  { id: "save", label: "Save Project", shortcut: "⌘S", action: () => save() },
  { id: "open", label: "Open Project", shortcut: "⌘O", action: () => open() },
];
```

### Error Boundaries

```tsx
import { ErrorBoundary, CompactErrorBoundary } from "../components";

<ErrorBoundary>
  <RiskyComponent />
</ErrorBoundary>

// Compact variant for inline use
<CompactErrorBoundary>
  <SmallWidget />
</CompactErrorBoundary>
```

### VirtualList (Performance-Critical)

**Import from:** `"../components"` or `"../components/VirtualList"`

```tsx
import { VirtualList, VirtualTree, flattenTree, useVirtualList } from "../components";

// Virtualized list — only renders visible items
<VirtualList
  items={entries()}              // T[]
  itemHeight={28}                // Fixed height per item (required)
  height={500}                   // Container height (number or CSS string)
  overscan={5}                   // Items above/below viewport to pre-render
  onScroll={(top) => {}}          // Scroll position callback
>
  {(item, index) => <div>{item.name}</div>}
</VirtualList>

// Virtualized tree — for hierarchical data
<VirtualTree
  rootItems={rootEntries()}      // T[]
  getChildren={(item) => item.children}
  isExpanded={(item) => expandedSet.has(item.id)}
  itemHeight={28}
  height="100%"
>
  {(item, index) => <TreeRow item={item} />}
</VirtualTree>

// Low-level hook for custom implementations
const vl = useVirtualList({
  items: () => entries(),
  itemHeight: 28,
  containerHeight: 500,
  overscan: 5,
});
// vl.visibleItems, vl.totalHeight, vl.scrollOffset, etc.
```

### Preferences API

**Import from:** `"../components"` or `"../components/preferences"`

```typescript
import {
  createPreferences, DEFAULT_PREFERENCES,
  getPreference, getLastPath, setLastPath,
  getRecentProjects, addRecentProject, removeRecentProject,
  type AppPreferences, type Theme, type TreeDensity
} from "../components";

// Create reactive preferences (used once in AppProvider)
const prefs = createPreferences();
prefs.preferences()                          // Accessor<AppPreferences>
prefs.isDirty()                              // Accessor<boolean>
prefs.updatePreference("theme", "dark")      // Type-safe key+value
prefs.updateShortcut("save", "mod+s")        // Update keybinding
prefs.resetToDefaults()                      // Reset all prefs

// Key preference fields:
// theme, accentColor, treeDensity, showLineNumbers, fontSize,
// defaultHashAlgorithm, defaultExportFormat, defaultViewMode,
// autoSaveProject, autoSaveIntervalMs, dateFormat,
// examinerName, organizationName, caseNumberPrefix,
// confirmBeforeDelete, confirmBeforeExport, confirmBeforeHash

// Standalone helpers (localStorage-based)
getLastPath(): string | null
setLastPath(path: string): void
getRecentProjects(): RecentProject[]
addRecentProject(project: RecentProject): void
```

### Drag & Drop

**Import from:** `"../components"` or `"../components/DragDrop"`

```tsx
import { DropZone, useDragDrop } from "../components";

<DropZone
  onDrop={(files: File[]) => handleFiles(files)}
  accept={[".ad1", ".e01"]}
  class="w-full h-48"
>
  <p>Drop evidence files here</p>
</DropZone>

const dragDrop = useDragDrop({
  onDrop: (files) => processFiles(files),
  accept: [".ad1", ".e01", ".l01"],
});
// dragDrop.isDragging, dragDrop.isOver, dragDrop.dragProps
```

### Search Panel

**Import from:** `"../components"` or `"../components/SearchPanel"`

```tsx
import { SearchPanel, useSearch, type SearchFilter, type SearchResult } from "../components";
// Full-featured search with filters, saved searches, result navigation
```

### Export Panel

**Import from:** `"../components"` or `"../components/ExportPanel"`

```tsx
import { ExportPanel, type ExportMode } from "../components";
// type ExportMode = "files" | "report" | "hashes"
```

---

## Constants Reference

**Import from:** `"../constants/ui"`

```typescript
// Numeric layout values
BAR_HEIGHT_SMALL = 24;     BAR_HEIGHT_BASE = 32;     BAR_HEIGHT_LG = 40;
SIDEBAR_WIDTH = 280;       SIDEBAR_MIN_WIDTH = 200;  SIDEBAR_MAX_WIDTH = 400;
SETTINGS_PANEL_WIDTH = 640;
ICON_SIZE_MICRO = 12;  ICON_SIZE_COMPACT = 14;  ICON_SIZE_SMALL = 16;
ICON_SIZE_BASE = 20;   ICON_SIZE_LG = 24;

// Z-index scale (use CSS classes when possible: z-modal, z-tooltip, etc.)
Z_INDEX = { base: 0, dropdown: 10, sticky: 20, fixed: 30,
  modalBackdrop: 40, modal: 50, popover: 60, tooltip: 70, notification: 80 };

// Container type detection
getContainerType(type: string): ContainerType     // "ad1" | "e01" | "l01" | "raw" | "ufed" | "archive"
getContainerTextColor(type: string): string        // "text-type-ad1" etc.
getContainerBadgeClass(type: string): string       // "badge-ad1" etc.

// File category detection
getFileCategory(filename): FileCategory  // "image" | "video" | "audio" | ...

// Keyboard shortcuts
DEFAULT_SHORTCUTS = { toggleSidebar: 'mod+b', search: 'mod+f', settings: 'mod+,', ... }

// Tree density presets
TREE_DENSITY_PRESETS = { compact: {...}, default: {...}, comfortable: {...} }
applyTreeDensity(density: TreeDensity): void
```

---

## Import Path Conventions

```typescript
// ✅ CORRECT import patterns
import { useFileManager, useHashManager } from "../hooks";
import { useAppContext, useManagers } from "../context";
import { HiOutlineFolder, FileIcon } from "../components/icons";
import { Button, Input, Modal } from "../components/ui";
import { useToast, ErrorBoundary, Tooltip } from "../components";
import { commands } from "../api/commands";
import type { TreeEntry, DiscoveredFile, ContainerInfo } from "../types";
import type { LazyTreeEntry, ContainerSummary } from "../types/lazy-loading";
import { formatBytes, debounce, normalizeError } from "../utils";
import { getExtension, getBasename, joinPath } from "../utils/pathUtils";
import { isImage, isEmail, detectFileType } from "../utils/fileTypeUtils";
import { formatDate, truncateHash } from "../utils/metadata";
import { isMac, formatShortcut } from "../utils/platform";
import { getContainerType, Z_INDEX, DEFAULT_SHORTCUTS } from "../constants/ui";

// ❌ WRONG — don't import from specific hook files
import { useFileManager } from "../hooks/useFileManager";     // Use ../hooks
import { getExtension } from "../utils/pathUtils/getExtension"; // Use ../utils/pathUtils
```

---

## Common Gotchas & Anti-Patterns

### SolidJS-Specific

| What You Might Assume | Actual Behavior |
|---|---|
| `props.items` is a plain array | **It's a getter.** In SolidJS, props are lazy. Access inside JSX or reactive contexts only. Never destructure props: `const { items } = props` ❌ |
| `createEffect` runs once | **Runs whenever its tracked dependencies change.** Use `on()` for explicit dependency tracking. |
| `createMemo` is like React `useMemo` | **Similar but auto-tracks.** No dependency array needed. Don't pass `[]`. |
| `signal()` in event handlers is fine | **Yes.** But `signal()` in component body (outside JSX) creates a subscription. Be intentional. |
| `setSignal(newVal)` causes re-render | **SolidJS doesn't re-render.** It updates the specific DOM nodes that read the signal. |
| `<Show>` and `<For>` work like React conditionals | **They're components.** `<Show when={signal()}>` — `when` must be a function/signal, not a static value. `<For each={items()}>` — `each` takes an accessor. |
| Cleanup in `createEffect` | **Use `onCleanup` inside the effect**, not return a function. |
| `classList` takes a class string | **Takes an object:** `classList={{ active: isActive(), disabled: !enabled() }}` |
| `className` vs `class` | **Use `class` or `className`** — both work in SolidJS. The codebase uses `className` predominantly. |

### Type Naming

| Trap | Correct |
|---|---|
| `TreeEntryInfo` is the main tree type | **`TreeEntry`** is AD1-specific, **`LazyTreeEntry`** is the unified type for all containers. `TreeEntryInfo` is in `types/lifecycle.ts` for the trait-based layer. |
| All types use camelCase field names | **Some use snake_case** (`TreeEntry.is_dir`, `LazyTreeEntry.child_count`) because their Rust source doesn't use `rename_all = "camelCase"`. |
| `DiscoveredFile.containerType` | **`container_type`** — snake_case (from Rust). |
| Hash algorithm is `"SHA256"` | **`"SHA-256"`** with hyphen. Use `HASH_ALGORITHMS.SHA256` constant. |

### Hook Dependencies

| Trap | Correct |
|---|---|
| `useHashManager()` takes no args | **Requires `fileManager`:** `useHashManager(fileManager)` |
| Hooks can be called anywhere | **Only call in component body or `createRoot`.** Never in event handlers or async callbacks. |
| `useAppContext()` works everywhere | **Only inside `<AppProvider>`.** Components rendered before the provider will throw. |

### Event Handling

| Trap | Correct |
|---|---|
| `listen()` auto-cleans up | **Must manually `unlisten()`:** `const unlisten = await listen(...); onCleanup(() => unlisten());` |
| `invoke()` errors are `Error` | **Errors from Rust are strings.** Use `normalizeError(e)` or `String(e)`. |

### Styling

| Trap | Correct |
|---|---|
| Use `bg-zinc-900` | **Use semantic tokens:** `bg-bg`, `bg-bg-secondary`, `bg-bg-panel` |
| Use `text-gray-400` | **Use:** `text-txt`, `text-txt-secondary`, `text-txt-muted` |
| Hardcode colors for container types | **Use:** `text-type-ad1`, `text-type-e01`, `text-type-ufed` (from CSS variables) |
| Use inline `z-index: 50` | **Use scale:** `z-modal`, `z-tooltip`, `z-dropdown` (from Tailwind config) |

---

*Last updated: 2026-02-16*
