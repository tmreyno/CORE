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

// Create new project
await project.createProject(path, name);

// Load existing project
await project.loadProject(projectPath);

// Save project
await project.saveProject();

// Access project info
const info = project.projectInfo();
```

### useProcessedDatabases

Processed database detection and parsing.

- Detect and list processed databases
- AXIOM parsing for categories/artifacts
- Selection and detail state

```tsx
import { useProcessedDatabases } from "./hooks";

const processedDb = useProcessedDatabases();

// Scan for databases
await processedDb.scanDatabases(directory);

// Get database details
const details = processedDb.selectedDetails();
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
├── useDatabase.ts            # SQLite persistence
├── useProject.ts             # Project management
├── useProcessedDatabases.ts  # Processed DB parsing
├── useAppState.ts            # Global app state
├── useAppActions.ts          # App-wide actions
│
│ # Project sub-hooks
├── project/
│   ├── index.ts              # Project hook exports
│   ├── types.ts              # Project types
│   ├── projectHelpers.ts     # Project utility helpers
│   ├── useProjectState.ts    # Project state management
│   ├── useProjectIO.ts       # Project file I/O
│   ├── useProjectHelpers.ts  # Project helper utilities
│   ├── useAutoSave.ts        # Auto-save functionality
│   ├── useBookmarks.ts       # Evidence bookmarks
│   ├── useNotes.ts           # Case notes
│   └── useActivityLog.ts     # Activity logging
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
├── usePreferenceEffects.ts   # Preference side effects
│
│ # Feature hooks
├── useActivityTimeline.ts    # Activity timeline
├── useLazyLoading.ts         # Lazy loading
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
