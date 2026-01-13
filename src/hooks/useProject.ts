// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  FFXProject,
  ProjectSaveResult,
  ProjectLoadResult,
  ProjectTab,
  ProjectHashHistory,
  ActivityLogEntry,
  ActivityCategory,
  ProcessedDatabaseState,
  ProjectUIState,
  ProjectSession,
  ProjectBookmark,
  ProjectNote,
  RecentSearch,
  ProcessedDbIntegrity,
  FilterState,
} from "../types/project";
import {
  PROJECT_FILE_VERSION,
  AUTO_SAVE_INTERVAL_MS,
  generateId,
  nowISO,
  createEmptyProject,
  createActivityEntry,
  createDefaultUIState,
  createDefaultFilterState,
  createDefaultSettings,
} from "../types/project";
import type { DiscoveredFile, HashHistoryEntry } from "../types";
import type { ProcessedDatabase } from "../types/processed";

// Re-export types for consumers
export type {
  FFXProject,
  ProjectSaveResult,
  ProjectLoadResult,
  ActivityLogEntry,
  ActivityCategory,
};

/** Options for building project state */
export interface BuildProjectOptions {
  rootPath: string;
  openTabs: Array<{ file: DiscoveredFile; id: string }>;
  activeTabPath: string | null;
  hashHistory: Map<string, HashHistoryEntry[]>;
  processedDatabases?: ProcessedDatabase[];
  selectedProcessedDb?: ProcessedDatabase | null;
  uiState?: Partial<ProjectUIState>;
  filterState?: FilterState;
}

/** Get current username from environment */
async function getCurrentUsername(): Promise<string> {
  try {
    const username = await invoke<string>("get_current_username");
    return username;
  } catch {
    return "unknown";
  }
}

/** Get current app version */
async function getAppVersion(): Promise<string> {
  try {
    const version = await invoke<string>("get_app_version");
    return version;
  } catch {
    return "0.1.0";
  }
}

/**
 * Hook for managing FFX project files (.ffxproj)
 * Handles saving/loading complete project state including:
 * - Open tabs and file selections
 * - Hash history and verifications
 * - Processed database state and integrity
 * - User activity log
 * - UI state (panel sizes, expanded nodes, etc.)
 * - Bookmarks, notes, and tags
 * - Search history
 */
export function useProject() {
  // === Core State ===
  const [project, setProject] = createSignal<FFXProject | null>(null);
  const [projectPath, setProjectPath] = createSignal<string | null>(null);
  const [modified, setModified] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);

  // === User/Session State ===
  const [currentUser, setCurrentUser] = createSignal<string>("unknown");
  const [currentSessionId, setCurrentSessionId] = createSignal<string | null>(null);

  // === Auto-save State ===
  const [autoSaveEnabled, setAutoSaveEnabled] = createSignal(true);
  const [lastAutoSave, setLastAutoSave] = createSignal<Date | null>(null);
  let autoSaveTimer: ReturnType<typeof setInterval> | null = null;
  let autoSaveCallback: (() => Promise<void>) | null = null;

  // Initialize current user
  getCurrentUsername().then(setCurrentUser);

  // =========================================================================
  // PROJECT LIFECYCLE
  // =========================================================================

  /**
   * Check if a project exists for the given root directory
   */
  const checkProjectExists = async (rootPath: string): Promise<string | null> => {
    try {
      return await invoke<string | null>("project_check_exists", { rootPath });
    } catch (e) {
      console.warn("Failed to check project:", e);
      return null;
    }
  };
  
  /**
   * Get the default project path for a root directory
   */
  const getDefaultProjectPath = async (rootPath: string): Promise<string> => {
    return await invoke<string>("project_get_default_path", { rootPath });
  };
  
  /**
   * Create a new project for the given root directory
   */
  const createProject = async (rootPath: string): Promise<FFXProject> => {
    const username = currentUser();
    const appVersion = await getAppVersion();
    const proj = createEmptyProject(rootPath, username, appVersion);
    
    setProject(proj);
    setProjectPath(null); // Not saved yet
    setCurrentSessionId(proj.current_session_id || null);
    setModified(true);
    
    // Start auto-save if enabled
    startAutoSave();
    
    return proj;
  };

  /**
   * End the current session
   */
  const endCurrentSession = () => {
    const proj = project();
    const sessionId = currentSessionId();
    if (!proj || !sessionId) return;

    const now = nowISO();
    const sessions = proj.sessions.map(s => {
      if (s.session_id === sessionId && !s.ended_at) {
        const startTime = new Date(s.started_at).getTime();
        const endTime = new Date(now).getTime();
        return {
          ...s,
          ended_at: now,
          duration_seconds: Math.floor((endTime - startTime) / 1000),
        };
      }
      return s;
    });

    setProject({ ...proj, sessions });
    setCurrentSessionId(null);
  };

  /**
   * Start a new session
   */
  const startNewSession = async () => {
    const proj = project();
    if (!proj) return;

    const sessionId = generateId();
    const username = currentUser();
    const appVersion = await getAppVersion();
    const now = nowISO();

    const newSession: ProjectSession = {
      session_id: sessionId,
      user: username,
      started_at: now,
      ended_at: null,
      app_version: appVersion,
    };

    setProject({
      ...proj,
      sessions: [...proj.sessions, newSession],
      current_session_id: sessionId,
    });
    setCurrentSessionId(sessionId);
    markModified();
  };
  
  /**
   * Build project from current app state (extended version)
   */
  const buildProjectFromState = async (options: BuildProjectOptions): Promise<FFXProject> => {
    const {
      rootPath,
      openTabs,
      activeTabPath,
      hashHistory,
      processedDatabases = [],
      selectedProcessedDb = null,
      uiState = {},
      filterState: inputFilterState,
    } = options;

    const existingProject = project();
    const now = nowISO();
    const username = currentUser();
    const appVersion = await getAppVersion();
    const name = rootPath.split('/').pop() || 'Untitled';

    // Convert tabs to project format
    const tabs: ProjectTab[] = openTabs.map((tab, index) => ({
      file_path: tab.file.path,
      name: tab.file.filename,
      order: index,
      container_type: tab.file.container_type,
      last_viewed: now,
    }));

    // Convert hash history to project format
    const hashHistoryObj: ProjectHashHistory = { files: {} };
    hashHistory.forEach((entries, filePath) => {
      hashHistoryObj.files[filePath] = entries.map(entry => ({
        algorithm: entry.algorithm,
        hash_value: entry.hash,
        computed_at: entry.timestamp instanceof Date
          ? entry.timestamp.toISOString()
          : String(entry.timestamp),
        verification: entry.verified && entry.verified_against ? {
          result: "match" as const,
          verified_against: entry.verified_against,
          verified_at: entry.timestamp instanceof Date
            ? entry.timestamp.toISOString()
            : String(entry.timestamp),
        } : undefined,
      }));
    });

    // Build processed database state
    const processedDbState: ProcessedDatabaseState = {
      loaded_paths: processedDatabases.map(db => db.path),
      selected_path: selectedProcessedDb?.path || null,
      detail_view_type: null,
      integrity: existingProject?.processed_databases?.integrity || {},
      cached_metadata: Object.fromEntries(
        processedDatabases.map(db => [db.path, {
          db_type: db.db_type,
          name: db.name,
          case_name: db.case_name,
          case_number: db.case_number,
          examiner: db.examiner,
        }])
      ),
    };

    // Merge UI state
    const mergedUIState: ProjectUIState = {
      ...createDefaultUIState(),
      ...(existingProject?.ui_state || {}),
      ...uiState,
    };

    // Build the project
    const proj: FFXProject = {
      // Metadata
      version: PROJECT_FILE_VERSION,
      project_id: existingProject?.project_id || generateId(),
      name: existingProject?.name || name,
      description: existingProject?.description,
      root_path: rootPath,
      created_at: existingProject?.created_at || now,
      saved_at: now,
      created_by_version: existingProject?.created_by_version || appVersion,
      saved_by_version: appVersion,

      // Users & Sessions
      users: existingProject?.users || [{
        username,
        first_access: now,
        last_access: now,
      }],
      current_user: username,
      sessions: existingProject?.sessions || [],
      current_session_id: currentSessionId() || undefined,
      activity_log: existingProject?.activity_log || [],
      activity_log_limit: existingProject?.activity_log_limit || 1000,

      // Evidence State
      open_directories: existingProject?.open_directories || [{
        path: rootPath,
        opened_at: now,
        recursive: true,
        file_count: openTabs.length,
        total_size: 0,
        last_scanned: now,
      }],
      recent_directories: existingProject?.recent_directories || [],
      tabs,
      active_tab_path: activeTabPath,
      file_selection: {
        selected_paths: openTabs.map(t => t.file.path),
        active_path: activeTabPath,
        timestamp: now,
      },
      hash_history: hashHistoryObj,

      // Processed Databases
      processed_databases: processedDbState,

      // Bookmarks & Notes
      bookmarks: existingProject?.bookmarks || [],
      notes: existingProject?.notes || [],
      tags: existingProject?.tags || [],

      // Reports
      reports: existingProject?.reports || [],

      // Searches
      saved_searches: existingProject?.saved_searches || [],
      recent_searches: existingProject?.recent_searches || [],
      filter_state: inputFilterState || existingProject?.filter_state || createDefaultFilterState(),

      // UI State
      ui_state: mergedUIState,

      // Settings
      settings: existingProject?.settings || createDefaultSettings(),

      // Custom data
      custom_data: existingProject?.custom_data,
    };

    return proj;
  };

  // =========================================================================
  // ACTIVITY LOGGING
  // =========================================================================

  /**
   * Log an activity to the project
   */
  const logActivity = (
    category: ActivityCategory,
    action: string,
    description: string,
    filePath?: string,
    details?: Record<string, unknown>
  ) => {
    const proj = project();
    if (!proj || !proj.settings?.track_activity) return;

    const entry = createActivityEntry(
      currentUser(),
      category,
      action,
      description,
      filePath,
      details
    );

    // Add entry, respecting limit
    const limit = proj.activity_log_limit || 1000;
    let log = [entry, ...proj.activity_log];
    if (log.length > limit) {
      log = log.slice(0, limit);
    }

    setProject({ ...proj, activity_log: log });
    markModified();
  };
  
  /**
   * Save current project state
   */
  const saveProject = async (
    options: BuildProjectOptions,
    customPath?: string
  ): Promise<ProjectSaveResult> => {
    try {
      setLoading(true);
      
      // Build project from current state
      const proj = await buildProjectFromState(options);
      
      // Determine save path
      let savePath = customPath || projectPath();
      
      // If no path, ask for one
      if (!savePath) {
        const defaultPath = await getDefaultProjectPath(options.rootPath);
        const selected = await save({
          defaultPath,
          filters: [{ name: "FFX Project", extensions: ["ffxproj"] }],
          title: "Save Project",
        });
        
        if (!selected) {
          return { success: false, error: "Save cancelled" };
        }
        savePath = selected;
      }
      
      // Save via Tauri
      const result = await invoke<ProjectSaveResult>("project_save", {
        project: proj,
        path: savePath,
      });
      
      if (result.success) {
        setProject(proj);
        setProjectPath(result.path || savePath);
        setModified(false);
        setError(null);
        setLastAutoSave(new Date());
        
        // Log the save
        logActivity('project', 'save', `Project saved to ${savePath}`, savePath);
        
        console.log(`Project saved to: ${result.path}`);
      } else {
        setError(result.error || "Failed to save project");
      }
      
      return result;
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setError(errorMsg);
      return { success: false, error: errorMsg };
    } finally {
      setLoading(false);
    }
  };
  
  /**
   * Save project to a new location (Save As)
   */
  const saveProjectAs = async (options: BuildProjectOptions): Promise<ProjectSaveResult> => {
    const defaultPath = await getDefaultProjectPath(options.rootPath);
    const selected = await save({
      defaultPath,
      filters: [{ name: "FFX Project", extensions: ["ffxproj"] }],
      title: "Save Project As",
    });
    
    if (!selected) {
      return { success: false, error: "Save cancelled" };
    }
    
    return saveProject(options, selected);
  };
  
  /**
   * Load a project from file
   */
  const loadProject = async (
    customPath?: string
  ): Promise<{ project: FFXProject | null; error?: string; warnings?: string[] }> => {
    try {
      setLoading(true);
      let loadPath = customPath;
      
      // If no path provided, show file picker
      if (!loadPath) {
        const selected = await open({
          filters: [{ name: "FFX Project", extensions: ["ffxproj"] }],
          title: "Open Project",
          multiple: false,
        });
        
        if (!selected) {
          return { project: null, error: "Open cancelled" };
        }
        loadPath = selected as string;
      }
      
      // Load via Tauri
      const result = await invoke<ProjectLoadResult>("project_load", {
        path: loadPath,
      });
      
      if (result.success && result.project) {
        // End any existing session
        endCurrentSession();
        
        setProject(result.project);
        setProjectPath(loadPath);
        setModified(false);
        setError(null);
        
        // Start a new session for this user
        await startNewSession();
        
        // Start auto-save
        startAutoSave();
        
        // Log the load
        logActivity('project', 'load', `Project loaded: ${result.project.name}`, loadPath);
        
        console.log(`Project loaded: ${result.project.name}`);
        return { project: result.project, warnings: result.warnings };
      } else {
        const errorMsg = result.error || "Failed to load project";
        setError(errorMsg);
        return { project: null, error: errorMsg };
      }
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setError(errorMsg);
      return { project: null, error: errorMsg };
    } finally {
      setLoading(false);
    }
  };

  // =========================================================================
  // AUTO-SAVE
  // =========================================================================

  /** Start auto-save timer */
  const startAutoSave = () => {
    if (autoSaveTimer) {
      clearInterval(autoSaveTimer);
    }
    
    const settings = project()?.settings;
    if (!settings?.auto_save || !autoSaveEnabled()) return;
    
    const interval = settings.auto_save_interval || AUTO_SAVE_INTERVAL_MS;
    
    autoSaveTimer = setInterval(async () => {
      if (modified() && projectPath() && autoSaveCallback) {
        console.log("Auto-saving project...");
        try {
          await autoSaveCallback();
          console.log("Auto-save complete");
        } catch (e) {
          console.warn("Auto-save failed:", e);
        }
      }
    }, interval);
  };

  /** Stop auto-save timer */
  const stopAutoSave = () => {
    if (autoSaveTimer) {
      clearInterval(autoSaveTimer);
      autoSaveTimer = null;
    }
  };

  /** Set the auto-save callback (called from App.tsx with current state) */
  const setAutoSaveCallback = (callback: () => Promise<void>) => {
    autoSaveCallback = callback;
  };

  // =========================================================================
  // STATE MANAGEMENT HELPERS
  // =========================================================================
  
  /**
   * Mark project as modified (call when state changes)
   */
  const markModified = () => {
    if (project() || projectPath()) {
      setModified(true);
    }
  };
  
  /**
   * Clear current project
   */
  const clearProject = () => {
    endCurrentSession();
    stopAutoSave();
    setProject(null);
    setProjectPath(null);
    setModified(false);
    setError(null);
    setCurrentSessionId(null);
  };

  /**
   * Update UI state in project
   */
  const updateUIState = (updates: Partial<ProjectUIState>) => {
    const proj = project();
    if (!proj) return;
    
    setProject({
      ...proj,
      ui_state: {
        ...proj.ui_state,
        ...updates,
      },
    });
    markModified();
  };

  /**
   * Add a bookmark
   */
  const addBookmark = (bookmark: Omit<ProjectBookmark, 'id' | 'created_by' | 'created_at'>) => {
    const proj = project();
    if (!proj) return;

    const newBookmark: ProjectBookmark = {
      ...bookmark,
      id: generateId(),
      created_by: currentUser(),
      created_at: nowISO(),
    };

    setProject({
      ...proj,
      bookmarks: [...proj.bookmarks, newBookmark],
    });
    
    logActivity('bookmark', 'add', `Added bookmark: ${bookmark.name}`, bookmark.target_path);
    markModified();
  };

  /**
   * Remove a bookmark
   */
  const removeBookmark = (bookmarkId: string) => {
    const proj = project();
    if (!proj) return;

    const bookmark = proj.bookmarks.find(b => b.id === bookmarkId);
    setProject({
      ...proj,
      bookmarks: proj.bookmarks.filter(b => b.id !== bookmarkId),
    });
    
    if (bookmark) {
      logActivity('bookmark', 'remove', `Removed bookmark: ${bookmark.name}`, bookmark.target_path);
    }
    markModified();
  };

  /**
   * Add a note
   */
  const addNote = (note: Omit<ProjectNote, 'id' | 'created_by' | 'created_at' | 'modified_at'>) => {
    const proj = project();
    if (!proj) return;

    const now = nowISO();
    const newNote: ProjectNote = {
      ...note,
      id: generateId(),
      created_by: currentUser(),
      created_at: now,
      modified_at: now,
    };

    setProject({
      ...proj,
      notes: [...proj.notes, newNote],
    });
    
    logActivity('note', 'add', `Added note: ${note.title}`, note.target_path);
    markModified();
  };

  /**
   * Update a note
   */
  const updateNote = (noteId: string, updates: Partial<Pick<ProjectNote, 'title' | 'content' | 'tags' | 'priority'>>) => {
    const proj = project();
    if (!proj) return;

    setProject({
      ...proj,
      notes: proj.notes.map(n =>
        n.id === noteId
          ? { ...n, ...updates, modified_at: nowISO() }
          : n
      ),
    });
    
    logActivity('note', 'update', `Updated note: ${noteId}`);
    markModified();
  };

  /**
   * Add a recent search
   */
  const addRecentSearch = (query: string, resultCount: number) => {
    const proj = project();
    if (!proj) return;

    const entry: RecentSearch = {
      query,
      timestamp: nowISO(),
      result_count: resultCount,
    };

    const maxRecent = proj.settings?.max_recent_items || 50;
    let searches = [entry, ...proj.recent_searches.filter(s => s.query !== query)];
    if (searches.length > maxRecent) {
      searches = searches.slice(0, maxRecent);
    }

    setProject({
      ...proj,
      recent_searches: searches,
    });
    
    logActivity('search', 'perform', `Searched: "${query}" (${resultCount} results)`);
    markModified();
  };

  /**
   * Update processed database integrity
   */
  const updateProcessedDbIntegrity = (path: string, integrity: ProcessedDbIntegrity) => {
    const proj = project();
    if (!proj) return;

    setProject({
      ...proj,
      processed_databases: {
        ...proj.processed_databases,
        integrity: {
          ...proj.processed_databases.integrity,
          [path]: integrity,
        },
      },
    });
    
    logActivity('database', 'verify', `Updated integrity for: ${path}`, path, {
      status: integrity.status,
      changes: integrity.changes,
    });
    markModified();
  };

  // =========================================================================
  // RETURN PUBLIC API
  // =========================================================================
  
  return {
    // === State (read-only signals) ===
    project,
    projectPath,
    modified,
    error,
    loading,
    currentUser,
    currentSessionId,
    autoSaveEnabled,
    lastAutoSave,

    // === Lifecycle ===
    checkProjectExists,
    getDefaultProjectPath,
    createProject,
    clearProject,

    // === Save/Load ===
    saveProject,
    saveProjectAs,
    loadProject,

    // === State Updates ===
    markModified,
    updateUIState,
    logActivity,

    // === Bookmarks ===
    addBookmark,
    removeBookmark,

    // === Notes ===
    addNote,
    updateNote,

    // === Search ===
    addRecentSearch,

    // === Processed Databases ===
    updateProcessedDbIntegrity,

    // === Auto-save ===
    setAutoSaveEnabled,
    setAutoSaveCallback,
    startAutoSave,
    stopAutoSave,

    // === Build helper (for external save calls) ===
    buildProjectFromState,
  };
}
