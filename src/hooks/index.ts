// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview
 * CORE-FFX Hooks - Custom SolidJS hooks for the Forensic File Explorer.
 * 
 * This module provides a collection of reactive hooks for managing:
 * - File operations and system statistics
 * - Hash computation and verification
 * - Database connections and queries
 * - Project lifecycle management
 * - Theme and accessibility preferences
 * - Keyboard shortcuts and navigation
 * - Undo/redo command history
 * 
 * @module hooks
 */

// ============================================================================
// File Management
// ============================================================================

/**
 * Hook for managing evidence files, scanning directories, and system stats.
 * @see {@link useFileManager}
 */
export { useFileManager } from "./useFileManager";
export type { FileManager, SystemStats, FileStatus } from "./useFileManager";

/**
 * Utilities for reading data from various evidence sources.
 * Unified interface for disk files, AD1 entries, VFS entries, and archive entries.
 * @see {@link readBytesFromSource}
 * @see {@link readTextFromSource}
 */
export { 
  readBytesFromSource, 
  readTextFromSource, 
  getSourceKey, 
  getSourceFilename 
} from "./useEntrySource";
export type { ByteReadResult, TextReadResult } from "./useEntrySource";

/**
 * Hook for computing and managing file hashes (MD5, SHA1, SHA256).
 * @see {@link useHashManager}
 */
export { useHashManager } from "./useHashManager";
export type { HashManager, FileHashInfo } from "./useHashManager";

/**
 * Hook for AD1 Container V2 operations (50x faster than OLD implementation).
 * Provides lazy loading, hash verification, extraction, and container info.
 * @see {@link useAd1ContainerV2}
 */
export { useAd1ContainerV2 } from "./useAd1ContainerV2";
export type { 
  TreeEntryV2, 
  ItemVerifyResult, 
  ExtractionResult, 
  Ad1InfoV2, 
  TreeItem 
} from "./useAd1ContainerV2";

/**
 * Hook for unified lazy loading across all container types.
 * Provides a consistent API for lazy loading data from AD1, E01, UFED, ZIP, etc.
 * @see {@link useLazyLoading}
 * 
 * @example
 * ```tsx
 * const { summary, rootChildren, loadChildren } = useLazyLoading(
 *   () => containerPath,
 *   { autoLoad: true }
 * );
 * ```
 */
export { 
  useLazyLoading, 
  getContainerSummary,
  getRootChildren,
  getChildren,
  getLazyLoadSettings,
  updateLazyLoadSettings,
} from "./useLazyLoading";
export type { 
  UseLazyLoadingOptions, 
  UseLazyLoadingReturn 
} from "./useLazyLoading";

/**
 * Hook for unified container access (RECOMMENDED).
 * Single API for ALL container types with automatic type detection.
 * Replaces fragmented per-container approaches.
 * @see {@link useUnifiedContainer}
 * 
 * @example
 * ```tsx
 * const { summary, rootChildren, loadChildren } = useUnifiedContainer(
 *   () => containerPath,
 *   { autoLoadSummary: true }
 * );
 * ```
 */
export {
  useUnifiedContainer,
  getContainerSummary as getUnifiedSummary,
  getRootChildren as getUnifiedRootChildren,
  getChildren as getUnifiedChildren,
} from "./useUnifiedContainer";
export type {
  UseUnifiedContainerOptions,
  UseUnifiedContainerReturn,
} from "./useUnifiedContainer";

// ============================================================================
// Data Management
// ============================================================================

/**
 * Hook for SQLite database operations on processed evidence.
 * @see {@link useDatabase}
 */
export { useDatabase } from "./useDatabase";
export * from "./useDatabase";

/**
 * Hook for managing preview file cache to avoid re-extraction.
 * @see {@link usePreviewCache}
 */
export { usePreviewCache, createCacheKey } from "./usePreviewCache";
export type { PreviewCacheManager } from "./usePreviewCache";

/**
 * Hook for managing forensic project lifecycle (create, open, save, close).
 * @see {@link useProject}
 */
export { useProject } from "./useProject";
export { 
  buildSaveOptions, 
  handleLoadProject, 
  createDocumentEntry,
  handleOpenDirectory,
  handleProjectSetupComplete,
  type BuildSaveOptionsParams,
  type HandleLoadProjectParams,
  type HandleOpenDirectoryParams,
  type HandleProjectSetupCompleteParams,
  type BuildProjectOptions,
  type CenterTabForSave,
} from "./project";

/**
 * Hook for managing processed databases from forensic tools.
 * @see {@link useProcessedDatabases}
 */
export { useProcessedDatabases } from "./useProcessedDatabases";
export type { ProcessedDatabasesManager, DetailViewType } from "./useProcessedDatabases";

// ============================================================================
// UI Hooks
// ============================================================================

/**
 * Hook for defining and handling keyboard shortcuts.
 * Supports platform-aware modifier keys (Cmd on Mac, Ctrl on Windows/Linux).
 * @see {@link useKeyboardShortcuts}
 * 
 * @example
 * ```tsx
 * const shortcuts = useKeyboardShortcuts([
 *   { id: "save", keys: "cmd+s", description: "Save", handler: handleSave },
 *   { id: "open", keys: "cmd+o", description: "Open", handler: handleOpen },
 * ]);
 * ```
 */
export { useKeyboardShortcuts, formatShortcutKeys, commonShortcuts } from "./useKeyboardShortcuts";
export type { KeyboardShortcut } from "./useKeyboardShortcuts";

/**
 * Theme utilities and types for managing application theme.
 * Theme is managed through preferences system - use createThemeActions with preferences.
 * @see {@link createThemeActions}
 * 
 * @example
 * ```tsx
 * const themeActions = createThemeActions(
 *   () => preferences.preferences().theme,
 *   (theme) => preferences.updatePreference("theme", theme)
 * );
 * ```
 */
export { 
  getThemeIcon, 
  getThemeLabel, 
  getSystemTheme, 
  resolveTheme, 
  applyTheme, 
  getNextTheme,
  createThemeActions 
} from "./useTheme";
export type { Theme, ResolvedTheme, ThemeActions } from "./useTheme";

/**
 * Hook for trapping focus within a container (modals, dialogs).
 * Ensures keyboard navigation stays within the active overlay.
 * @see {@link useFocusTrap}
 * 
 * @example
 * ```tsx
 * const trapRef = useFocusTrap(() => containerRef(), () => isOpen());
 * ```
 */
export { useFocusTrap, createFocusTrap } from "./useFocusTrap";

// ============================================================================
// Undo/Redo History
// ============================================================================

/**
 * Hook for managing undo/redo command history.
 * Implements the Command pattern for reversible operations.
 * @see {@link useHistory}
 * 
 * @example
 * ```tsx
 * const [state, actions] = useHistory({ maxHistory: 50 });
 * 
 * // Execute a command
 * actions.execute({
 *   type: "update",
 *   description: "Update file name",
 *   previousState: oldName,
 *   newState: newName,
 *   execute: () => setName(newName),
 *   undo: () => setName(oldName),
 * });
 * 
 * // Undo/Redo
 * actions.undo();
 * actions.redo();
 * ```
 */
export { 
  useHistory, 
  useHistoryContext,
  HistoryProvider,
  createStateCommand,
  createBatchCommand
} from "./useHistory";
export type { Command, HistoryState, HistoryActions, UseHistoryOptions } from "./useHistory";

/**
 * Hook for managing resizable panels.
 * Handles width constraints, collapse/expand, and mouse drag.
 * @see {@link usePanelResize}
 * @see {@link useDualPanelResize}
 * 
 * @example
 * ```tsx
 * const panel = usePanelResize({ 
 *   initialWidth: 320, 
 *   minWidth: 150, 
 *   maxWidth: 600, 
 *   side: "left" 
 * });
 * 
 * <div style={{ width: panel.collapsed() ? "0" : `${panel.width()}px` }}>
 *   ...
 * </div>
 * <div onMouseDown={panel.startDrag}>Resize</div>
 * ```
 */
export { usePanelResize, useDualPanelResize } from "./usePanelResize";
export type { PanelResizeOptions, UsePanelResizeReturn, DualPanelResizeOptions, UseDualPanelResizeReturn } from "./usePanelResize";

// ============================================================================
// Async State Management
// ============================================================================

/**
 * Generic hooks for async operation state management.
 * Provides loading, error, and data state tracking.
 * @see {@link useAsyncState}
 * @see {@link useAsyncSetState}
 * @see {@link useCachedAsyncState}
 * 
 * @example
 * ```tsx
 * // Single async operation
 * const fileState = useAsyncState<FileData>();
 * await fileState.execute(() => loadFile(path));
 * 
 * // Multiple items loading
 * const loading = useAsyncSetState<string>();
 * await loading.execute(path, () => loadFile(path));
 * 
 * // Cached async state
 * const cache = useCachedAsyncState<string, FileInfo>();
 * const info = await cache.fetch(path, () => getInfo(path));
 * ```
 */
export { 
  useAsyncState, 
  useAsyncSetState, 
  useCachedAsyncState 
} from "./useAsyncState";
export type { 
  AsyncStatus, 
  AsyncState, 
  ExecuteOptions,
  AsyncSetState,
  CachedAsyncState,
} from "./useAsyncState";

// ============================================================================
// Application-Level Hooks
// ============================================================================

/**
 * Hook to apply user preference settings to the DOM.
 * Handles theme, accent color, font size, animations, density, and sidebar position.
 * @see {@link usePreferenceEffects}
 */
export { usePreferenceEffects } from "./usePreferenceEffects";

/**
 * Hook to manage application UI state (modals, views, panels, etc.).
 * @see {@link useAppState}
 */
export { useAppState } from "./useAppState";
export type { 
  ModalState, 
  ViewState, 
  ProjectState, 
  LeftPanelState, 
  LeftPanelMode,
  CenterPanelState,
  CenterPaneTab,
  OpenDocumentTab,
  AppState 
} from "./useAppState";

/**
 * Hook to manage unified center pane tabs.
 * @see {@link useCenterPaneTabs}
 */
export { useCenterPaneTabs } from "./useCenterPaneTabs";
export type { 
  CenterPaneTabsState,
  CenterTab,
  CenterTabType,
  CenterPaneViewMode 
} from "./useCenterPaneTabs";

/**
 * Hook to manage global keyboard shortcuts.
 * @see {@link useKeyboardHandler}
 */
export { useKeyboardHandler } from "./useKeyboardHandler";
export type { KeyboardHandlerDeps } from "./useKeyboardHandler";

/**
 * Hook to manage window title with project name and unsaved indicator.
 * @see {@link useWindowTitle}
 */
export { useWindowTitle, setWindowTitle } from "./useWindowTitle";
export type { UseWindowTitleOptions } from "./useWindowTitle";

/**
 * Hook to confirm before closing window with unsaved changes.
 * @see {@link useCloseConfirmation}
 */
export { useCloseConfirmation, confirmUnsavedChanges } from "./useCloseConfirmation";
export type { UseCloseConfirmationOptions } from "./useCloseConfirmation";

/**
 * Hook to manage context menus (file operations, save operations).
 * @see {@link useContextMenus}
 */
export { useContextMenus } from "./useContextMenus";
export type { ContextMenusDeps, ContextMenusResult } from "./useContextMenus";

/**
 * Helper functions for search and context menus.
 * @see {@link createSearchHandlers}
 * @see {@link createContextMenuBuilders}
 */
export { createSearchHandlers, createContextMenuBuilders } from "./useAppActions";
export type { AppActionsDeps } from "./useAppActions";

/**
 * Factory for command palette actions.
 * @see {@link createCommandPaletteActions}
 */
export { createCommandPaletteActions } from "./useCommandPalette";
export type { CommandPaletteConfig, CommandPaletteViewMode } from "./useCommandPalette";

/**
 * Hook for database synchronization effects.
 * @see {@link useDatabaseEffects}
 */
export { useDatabaseEffects } from "./useDatabaseEffects";
export type { UseDatabaseEffectsOptions } from "./useDatabaseEffects";

// ============================================================================
// Store-based State Management
// ============================================================================

/**
 * Store utilities for complex nested state using solid-js/store.
 * Use these for efficient updates to arrays and nested objects.
 * @see {@link createTransferStore}
 * @see {@link createTabStore}
 * @see {@link createSelectionStore}
 * 
 * @example
 * ```tsx
 * // Transfer jobs with efficient updates
 * const [transferStore, transferActions] = createTransferStore();
 * transferActions.addJob({ id: "1", ... });
 * transferActions.updateJobProgress("1", 50);
 * 
 * // Multi-select with range support
 * const [selectionStore, selectionActions] = createSelectionStore(() => itemIds);
 * selectionActions.select(id, e.shiftKey ? "range" : "single");
 * ```
 */

// ============================================================================
// Performance Toolkit (Phases 13-16)
// ============================================================================

/**
 * Hook for Phase 13: Advanced Observability & Telemetry
 * Provides metrics, health monitoring, and distributed tracing.
 * @see {@link useObservability}
 */
export { useObservability } from "./useObservability";
export type {
  MetricValue,
  MetricEntry,
  HealthStatus,
  ComponentHealth,
  SystemHealth,
  HealthThresholdsInput,
  SystemStatus,
  LogLevel,
} from "./useObservability";

/**
 * Hook for Phase 14: Advanced CPU Profiling
 * Provides CPU profiling with flamegraph generation using pprof.
 * @see {@link useCPUProfiler}
 */
export { useCPUProfiler } from "./useCPUProfiler";
export type {
  ProfileReport,
  FunctionSample,
  ProfileComparison,
  FunctionDiff,
  ProfileSummary,
  ActiveProfile
} from "./useCPUProfiler";

/**
 * Hook for Phase 15: Advanced Memory Profiling
 * Provides memory profiling with leak detection and allocation tracking.
 * @see {@link useMemoryProfiler}
 */
export { useMemoryProfiler } from "./useMemoryProfiler";
export type {
  MemoryReport,
  MemorySnapshot,
  LeakAnalysis,
  LeakCandidate,
  MemoryTimeline,
  SnapshotComparison
} from "./useMemoryProfiler";

/**
 * Hook for Phase 16: Automated Performance Regression Testing
 * Provides statistical regression detection with baseline management.
 * @see {@link useRegressionTesting}
 */
export { useRegressionTesting } from "./useRegressionTesting";
export type {
  PerformanceBaseline,
  PerformanceStatistics,
  RegressionReport,
  TrendAnalysis,
  TestHistory,
  PerformanceMeasurement,
  RegressionSummary,
  ThresholdConfig
} from "./useRegressionTesting";

// ============================================================================
// Project Enhancement Hooks (Phase 17)
// ============================================================================

/**
 * Hook for project recovery, backups, and health monitoring
 * @see {@link useProjectRecovery}
 */
export { useProjectRecovery } from "./useProjectRecovery";
export type {
  BackupFile,
  BackupMetadata,
  BackupType,
  ProjectHealth,
  ProjectHealthStatus,
  HealthIssue,
  IssueSeverity,
  IssueCategory,
  RecoveryInfo
} from "./useProjectRecovery";

/**
 * Hook for workspace profiles management
 * @see {@link useWorkspaceProfiles}
 */
export { useWorkspaceProfiles } from "./useWorkspaceProfiles";
export type {
  WorkspaceProfile,
  ProfileSummary as WorkspaceProfileSummary,
  ProfileType as WorkspaceProfileType,
  LayoutConfig,
  ToolConfig,
  FilterPreset,
  ViewSettings,
  QuickAction,
  CenterLayout
} from "./useWorkspaceProfiles";

/**
 * Hook for project templates
 * @see {@link useProjectTemplates}
 */
export { useProjectTemplates } from "./useProjectTemplates";
export type {
  ProjectTemplate,
  TemplateSummary,
  TemplateCategory,
  BookmarkTemplate,
  NoteTemplate,
  TabTemplate,
  ChecklistItem,
  MetadataField
} from "./useProjectTemplates";

/**
 * Hook for activity timeline visualization
 * @see {@link useActivityTimeline}
 */
export { useActivityTimeline } from "./useActivityTimeline";
export type {
  TimelineVisualization,
  TimelineSummary,
  ActivityHeatmap,
  DailyBreakdown,
  TypeDistribution,
  UserActivity,
  ActivityTrends,
  TimelineExport,
  ExportMetadata,
  ActivityExportEntry,
  FFXProject
} from "./useActivityTimeline";

/**
 * Hook for project comparison and merging
 * @see {@link useProjectComparison}
 */
export { useProjectComparison } from "./useProjectComparison";
export type {
  ProjectComparison,
  ComparisonSummary,
  BookmarkDiff,
  NoteDiff,
  EvidenceDiff,
  ActivityDiff,
  MergeConflict,
  ConflictType,
  MergeResult,
  MergeStrategy
} from "./useProjectComparison";
