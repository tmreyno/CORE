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
 * Hook for managing application theme (light, dark, system).
 * Persists preference to localStorage and responds to system changes.
 * @see {@link useTheme}
 * 
 * @example
 * ```tsx
 * const { theme, setTheme, toggleTheme, resolvedTheme } = useTheme();
 * ```
 */
export { useTheme, getThemeIcon, getThemeLabel } from "./useTheme";
export type { Theme } from "./useTheme";

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
