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
 * Hook for managing forensic project lifecycle (create, open, save, close).
 * @see {@link useProject}
 */
export { useProject } from "./useProject";

/**
 * Hook for managing processed databases from forensic tools.
 * @see {@link useProcessedDatabases}
 */
export { useProcessedDatabases } from "./useProcessedDatabases";
export type { ProcessedDatabasesManager } from "./useProcessedDatabases";

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
 * Hook to manage transfer progress and completion event listeners.
 * Sets up global event listeners that persist across tab switches.
 * @see {@link useTransferEvents}
 */
export { useTransferEvents } from "./useTransferEvents";

