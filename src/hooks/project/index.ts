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
  CenterTabForSave,
} from "./types";

// Re-export helper functions
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
} from "./projectHelpers";

import { createMemo } from "solid-js";
import { getBasename } from "../../utils/pathUtils";
import { createProjectState, createMarkModified } from "./useProjectState";
import { createActivityLogger } from "./useActivityLog";
import { createBookmarkManager } from "./useBookmarks";
import { createNoteManager } from "./useNotes";
import { createTagManager } from "./useTagManager";
import { createAutoSaveManager } from "./useAutoSave";
import { createProjectIO } from "./useProjectIO";
import {
  createSearchHistoryManager,
  createUIStateManager,
  createProcessedDbManager,
  createProjectLocationsManager,
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

  // Create tag manager
  const tagManager = createTagManager(signals, setters, markModified, logger);

  // Create search history manager
  const searchManager = createSearchHistoryManager(signals, setters, markModified, logger);

  // Create UI state manager
  const uiStateManager = createUIStateManager(signals, setters, markModified);

  // Create processed DB manager
  const processedDbManager = createProcessedDbManager(signals, setters, markModified, logger);
  
  // Create project locations manager
  const locationsManager = createProjectLocationsManager(signals, setters, markModified, logger);

  // === Derived/Memoized Values ===
  
  /** Whether a project is currently open */
  const hasProject = createMemo(() => signals.project() !== null);
  
  /** Project name (derived from project or path) */
  const projectName = createMemo(() => {
    const proj = signals.project();
    if (proj?.name) return proj.name;
    const path = signals.projectPath();
    if (path) return getBasename(path)?.replace('.cffx', '') || 'Untitled';
    return null;
  });
  
  /** Whether save is needed (modified and has path) */
  const needsSave = createMemo(() => 
    signals.modified() && signals.projectPath() !== null
  );
  
  /** Bookmark count for UI display */
  const bookmarkCount = createMemo(() => 
    signals.project()?.bookmarks?.length ?? 0
  );
  
  /** Note count for UI display */
  const noteCount = createMemo(() => 
    signals.project()?.notes?.length ?? 0
  );
  
  /** Recent searches (limited to last 10 for quick access) */
  const recentSearches = createMemo(() => 
    signals.project()?.recent_searches?.slice(0, 10) ?? []
  );
  
  /** Project locations (evidence path, case docs path, etc.) */
  const projectLocations = createMemo(() => 
    signals.project()?.locations ?? null
  );
  
  /** Root path from project */
  const rootPath = createMemo(() => 
    signals.project()?.root_path ?? null
  );

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
    
    // === Derived State (memoized) ===
    hasProject,
    projectName,
    needsSave,
    bookmarkCount,
    noteCount,
    recentSearches,
    projectLocations,
    rootPath,

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
    updateLocations: locationsManager.updateLocations,
    logActivity,

    // === Bookmarks ===
    addBookmark: bookmarkManager.addBookmark,
    updateBookmark: bookmarkManager.updateBookmark,
    removeBookmark: bookmarkManager.removeBookmark,
    clearBookmarks: bookmarkManager.clearBookmarks,

    // === Notes ===
    addNote: noteManager.addNote,
    updateNote: noteManager.updateNote,
    removeNote: noteManager.removeNote,

    // === Tags ===
    addTag: tagManager.addTag,
    updateTag: tagManager.updateTag,
    removeTag: tagManager.removeTag,

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
