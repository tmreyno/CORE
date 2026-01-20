// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Hook for managing FFX project files (.cffx)
 * 
 * Handles saving/loading complete project state including:
 * - Open tabs and file selections
 * - Hash history and verifications
 * - Processed database state and integrity
 * - User activity log
 * - UI state (panel sizes, expanded nodes, etc.)
 * - Bookmarks, notes, and tags
 * - Search history
 * 
 * This is a composition of focused sub-hooks:
 * - useProjectState: Core signal management
 * - useActivityLog: Activity logging
 * - useBookmarks: Bookmark management
 * - useNotes: Note management
 * - useAutoSave: Auto-save timer
 * - useProjectIO: Save/load operations
 * - useProjectHelpers: Search, UI state, and processed DB management
 */

// Re-export types for consumers
export type {
  FFXProject,
  ProjectSaveResult,
  ProjectLoadResult,
  ActivityLogEntry,
  ActivityCategory,
  BuildProjectOptions,
} from "./types";

import { createProjectState, createMarkModified } from "./useProjectState";
import { createActivityLogger } from "./useActivityLog";
import { createBookmarkManager } from "./useBookmarks";
import { createNoteManager } from "./useNotes";
import { createAutoSaveManager } from "./useAutoSave";
import { createProjectIO } from "./useProjectIO";
import {
  createSearchHistoryManager,
  createUIStateManager,
  createProcessedDbManager,
} from "./useProjectHelpers";

/**
 * Main hook for project management
 * Composes all sub-hooks into a single API
 */
export function useProject() {
  // Create core state signals and setters
  const { signals, setters } = createProjectState();

  // Create mark modified helper
  const markModified = createMarkModified(signals, setters);

  // Create activity logger (needed by other managers)
  const { logActivity } = createActivityLogger(signals, setters, markModified);
  const logger = { logActivity };

  // Create auto-save manager
  const autoSaveManager = createAutoSaveManager(signals, setters);

  // Create project I/O (save/load/create)
  const projectIO = createProjectIO(signals, setters, markModified, logger, autoSaveManager);

  // Create bookmark manager
  const bookmarkManager = createBookmarkManager(signals, setters, markModified, logger);

  // Create note manager
  const noteManager = createNoteManager(signals, setters, markModified, logger);

  // Create search history manager
  const searchManager = createSearchHistoryManager(signals, setters, markModified, logger);

  // Create UI state manager
  const uiStateManager = createUIStateManager(signals, setters, markModified);

  // Create processed DB manager
  const processedDbManager = createProcessedDbManager(signals, setters, markModified, logger);

  // Return unified API
  return {
    // === State (read-only signals) ===
    project: signals.project,
    projectPath: signals.projectPath,
    modified: signals.modified,
    error: signals.error,
    loading: signals.loading,
    currentUser: signals.currentUser,
    currentSessionId: signals.currentSessionId,
    autoSaveEnabled: signals.autoSaveEnabled,
    lastAutoSave: signals.lastAutoSave,

    // === Lifecycle ===
    checkProjectExists: projectIO.checkProjectExists,
    getDefaultProjectPath: projectIO.getDefaultProjectPath,
    createProject: projectIO.createProject,
    clearProject: projectIO.clearProject,

    // === Save/Load ===
    saveProject: projectIO.saveProject,
    saveProjectAs: projectIO.saveProjectAs,
    loadProject: projectIO.loadProject,

    // === State Updates ===
    markModified,
    updateUIState: uiStateManager.updateUIState,
    logActivity,

    // === Bookmarks ===
    addBookmark: bookmarkManager.addBookmark,
    removeBookmark: bookmarkManager.removeBookmark,

    // === Notes ===
    addNote: noteManager.addNote,
    updateNote: noteManager.updateNote,

    // === Search ===
    addRecentSearch: searchManager.addRecentSearch,

    // === Processed Databases ===
    updateProcessedDbIntegrity: processedDbManager.updateProcessedDbIntegrity,

    // === Auto-save ===
    setAutoSaveEnabled: autoSaveManager.setAutoSaveEnabled,
    setAutoSaveCallback: autoSaveManager.setAutoSaveCallback,
    startAutoSave: autoSaveManager.startAutoSave,
    stopAutoSave: autoSaveManager.stopAutoSave,
  };
}
