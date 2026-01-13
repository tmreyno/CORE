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

Project file (.ffxproj) management.

- `.ffxproj` create/load/save
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

### useKeyboardShortcuts

Global keyboard shortcut management.

- Register/unregister shortcuts
- Modifier key detection (Cmd/Ctrl, Shift, Alt)
- Shortcut conflict detection
- Platform-aware key display

```tsx
import { useKeyboardShortcuts, formatShortcutKeys } from "./hooks";

const shortcuts = useKeyboardShortcuts([
  { 
    id: "save", 
    keys: "cmd+s", 
    description: "Save project", 
    handler: handleSave 
  },
  { 
    id: "open", 
    keys: "cmd+o", 
    description: "Open file", 
    handler: handleOpen 
  },
]);

// Enable/disable a shortcut
shortcuts.setEnabled("save", false);

// Format for display
const display = formatShortcutKeys("cmd+shift+s"); // "⌘⇧S"
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

```
hooks/
├── index.ts              # Barrel exports
├── README.md             # This file
├── useDatabase.ts        # SQLite persistence
├── useFileManager.ts     # File discovery & management
├── useFocusTrap.ts       # Modal focus trapping
├── useHashManager.ts     # Hash computation
├── useHistory.tsx        # Undo/redo history
├── useKeyboardShortcuts.ts # Keyboard shortcuts
├── useProcessedDatabases.ts # Processed DB parsing
├── useProject.ts         # Project management
└── useTheme.ts           # Theme management
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
  useKeyboardShortcuts,
  useTheme,
  useFocusTrap,
  useHistory,
  
  // Utilities
  formatShortcutKeys,
  createStateCommand,
  getLatestHash,
} from "./hooks";
```

## Conventions

- All hooks follow SolidJS reactive patterns
- Tauri commands are wrapped with error handling
- Event listeners are properly cleaned up with `onCleanup`
- TypeScript types are exported for prop definitions

## Tauri Integration

Hooks invoke backend commands with `invoke()` and listen to events for progress and system stats.
