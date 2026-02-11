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

```text
hooks/
├── index.ts                  # Barrel exports
├── README.md                 # This file
├── hashUtils.ts              # Hash utility helpers
│
│ # Core hooks
├── useFileManager.ts         # File discovery & management
├── useHashManager.ts         # Hash computation
├── useDatabase.ts            # SQLite persistence
├── useProject.ts             # Project management
├── useProcessedDatabases.ts  # Processed DB parsing
├── useUnifiedContainer.ts    # Unified container access
├── useEntrySource.ts         # Container entry resolution
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
├── useKeyboardShortcuts.ts   # Keyboard shortcuts
├── useKeyboardHandler.ts     # Keyboard event handler
├── usePanelResize.ts         # Panel resize logic
├── usePreviewCache.ts        # File preview caching
├── useCenterPaneTabs.ts      # Center pane tab management
├── useContextMenus.ts        # Context menu state
├── useCommandPalette.tsx     # Command palette
├── useNotifications.ts       # Toast notifications
├── useCloseConfirmation.ts   # Unsaved changes guard
├── useWindowTitle.ts         # Window title management
├── usePreferenceEffects.ts   # Preference side effects
│
│ # Feature hooks
├── useActivityTimeline.ts    # Activity timeline
├── useAsyncState.ts          # Async state management
├── useDeduplication.ts       # File deduplication
├── useIndexCache.ts          # Container index cache
├── useLazyLoading.ts         # Lazy loading
├── useParallelExtractor.ts   # Parallel extraction
├── useStreamingExtractor.ts  # Streaming extraction
├── useProjectComparison.ts   # Project comparison
├── useProjectTemplates.ts    # Project templates
├── useProjectRecovery.ts     # Project recovery
├── useRecovery.ts            # General recovery
├── useWorkspaceProfiles.ts   # Workspace profiles
├── useDatabaseEffects.ts     # Database side effects
│
│ # Performance toolkit hooks
├── useObservability.ts       # Metrics and health monitoring
├── useCPUProfiler.ts         # CPU profiling
├── useMemoryProfiler.ts      # Memory profiling
└── useRegressionTesting.ts   # Performance regression testing
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
  
  // Performance toolkit (Phases 13-16)
  useObservability,
  useCPUProfiler,
  useMemoryProfiler,
  useRegressionTesting,
  
  // Utilities
  formatShortcutKeys,
  createStateCommand,
  getLatestHash,
} from "./hooks";
```

---

## Performance Toolkit Hooks (Phases 13-16)

### useObservability

**Phase 13**: Advanced observability and telemetry with metrics, health monitoring, and distributed tracing.

- Metrics collection (counters, gauges, histograms)
- System health monitoring (CPU, memory, disk)
- Distributed tracing with correlation IDs
- Real-time performance tracking

```tsx
import { useObservability } from "./hooks";

const obs = useObservability();

// Increment counter
await obs.incrementCounter("files_processed", 1);

// Set gauge value
await obs.setGauge("memory_usage_mb", 1024);

// Record histogram sample
await obs.recordHistogram("operation_duration_ms", 150);

// Get system health
const health = await obs.getHealthStatus();
console.log(`Status: ${health.status}, CPU: ${health.cpu_percent}%`);

// Start automatic health monitoring (polls every 5s)
obs.startHealthMonitoring(5000);

// Start tracing
const traceId = await obs.startTracing("container_load", { path: "/evidence/file.ad1" });
// ... perform operation ...
await obs.endTracing(traceId, "completed");

// Get all metrics
const metrics = await obs.getAllMetrics();

// Export metrics to JSON
const json = await obs.exportMetrics();
```

---

### useCPUProfiler

**Phase 14**: CPU profiling with flamegraph generation using pprof.

- Sampling-based CPU profiling
- Flamegraph SVG generation
- Profile comparison for regression detection
- Export to protobuf format

```tsx
import { useCPUProfiler } from "./hooks";

const profiler = useCPUProfiler();

// Start profiling (100 Hz sampling)
await profiler.startProfiling("container_load", 100);

// ... perform CPU-intensive operations ...

// Stop and get report
const report = await profiler.stopProfiling("container_load");
console.log(`Samples: ${report.sample_count}, Duration: ${report.duration_ms}ms`);
console.log("Top functions:", report.top_functions);

// Generate flamegraph
const svg = await profiler.generateFlamegraph("container_load");
// Display SVG in UI or save to file

// Compare two profiles
const comparison = await profiler.compareProfiles("baseline", "current");
console.log("Regressed:", comparison.functions_regressed);
console.log("Improved:", comparison.functions_improved);

// Export profile
await profiler.exportProfile("container_load", "/path/to/profile.pb");

// Profile an async function automatically
const { result, report } = await profiler.profileAsync(
  "hash_computation",
  async () => {
    // ... perform operation ...
    return result;
  }
);
```

---

### useMemoryProfiler

**Phase 15**: Memory profiling with leak detection and allocation tracking.

- Real-time memory tracking
- Leak detection algorithms
- Snapshot comparison
- Timeline analysis

```tsx
import { useMemoryProfiler } from "./hooks";

const profiler = useMemoryProfiler();

// Start memory profiling (1s interval)
await profiler.startProfiling("container_process", 1000);

// ... perform memory-intensive operations ...

// Stop and get report
const report = await profiler.stopProfiling("container_process");
console.log(`Peak: ${report.peak_memory_mb}MB, Net change: ${report.net_change_mb}MB`);

// Analyze for memory leaks
const leaks = await profiler.analyzeLeaks("container_process");
if (leaks.potential_leaks.length > 0) {
  console.warn(`Found ${leaks.potential_leaks.length} potential memory leaks`);
  console.warn(`Total leaked: ${leaks.total_leaked_mb}MB`);
}

// Get memory timeline
const timeline = await profiler.getTimeline("container_process");
// Display timeline chart with snapshots

// Compare snapshots
const comparison = await profiler.compareSnapshots("container_process", 0, -1);
console.log(`Growth rate: ${comparison.growth_rate_mb_per_sec} MB/s`);

// Profile an async function automatically
const { result, report, leaks } = await profiler.profileAsync(
  "file_extraction",
  async () => {
    // ... perform operation ...
    return result;
  }
);

// Quick leak check (profile for 30 seconds)
const leakAnalysis = await profiler.quickLeakCheck("test_operation", 30000);
```

---

### useRegressionTesting

**Phase 16**: Automated performance regression testing with statistical analysis.

- Performance baseline management
- Statistical regression detection (mean, stddev, percentiles)
- Linear regression for trend analysis
- Configurable thresholds per test

```tsx
import { useRegressionTesting } from "./hooks";

const regression = useRegressionTesting();

// Record baseline (5 measurements)
await regression.recordBaseline("hash_sha256", [
  100.5, 102.1, 99.8, 101.3, 100.9
], "abc123" // optional git commit hash
);

// Run test and detect regression
const result = await regression.runTest("hash_sha256", 150.0);
if (result.is_regression) {
  console.warn(`Regression detected: ${result.percent_change}% slower`);
  console.warn(`Current: ${result.current_duration_ms}ms vs Baseline: ${result.baseline_mean_ms}ms`);
}

// Get all baselines
const baselines = await regression.getBaselines();

// Analyze trends over 30 days
const trends = await regression.analyzeTrends("hash_sha256", 30);
if (trends.is_degrading) {
  console.warn(`Performance degrading: ${trends.total_change_percent}% over ${trends.period_days} days`);
  console.log(`Slope: ${trends.slope}ms/day`);
}

// Set custom threshold (15% instead of default 10%)
await regression.setThreshold("hash_sha256", 15.0);

// Get test history
const history = await regression.getHistory("hash_sha256");

// Get summary of all tests
const summary = await regression.getSummary();
console.log(`Total tests: ${summary.total_tests}, Regressions: ${summary.regressions_detected}`);

// Export report to JSON
await regression.exportReport("/path/to/report.json");

// Benchmark an async function automatically
const { result, report } = await regression.benchmarkAsync(
  "container_load",
  async () => {
    // ... perform operation ...
    return result;
  }
);

// Quick regression check
const isRegression = await regression.quickCheck("hash_sha256", 150.0);
```

---

## Conventions

- All hooks follow SolidJS reactive patterns
- Tauri commands are wrapped with error handling
- Event listeners are properly cleaned up with `onCleanup`
- TypeScript types are exported for prop definitions

## Tauri Integration

Hooks invoke backend commands with `invoke()` and listen to events for progress and system stats.
