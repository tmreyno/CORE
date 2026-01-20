// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Project I/O operations (save, load, create)
 */

import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  FFXProject,
  ProjectSaveResult,
  ProjectLoadResult,
  ProjectTab,
  ProjectHashHistory,
  ProcessedDatabaseState,
  ProjectUIState,
  ProjectSession,
  FilterState,
  EvidenceCache,
  CachedDiscoveredFile,
  CachedContainerInfo,
  CachedFileHash,
  CaseDocumentsCache,
} from "../../types/project";
import {
  PROJECT_FILE_VERSION,
  generateId,
  nowISO,
  createEmptyProject,
  createDefaultUIState,
  createDefaultFilterState,
  createDefaultSettings,
} from "../../types/project";
import { getAppVersion } from "./useProjectState";
import type {
  BuildProjectOptions,
  ProjectStateSignals,
  ProjectStateSetters,
  ActivityLogger,
  ProjectIO,
} from "./types";

/**
 * Create project I/O functions
 */
export function createProjectIO(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void,
  logger: ActivityLogger,
  autoSave: { startAutoSave: () => void; stopAutoSave: () => void }
): ProjectIO {
  /**
   * End the current session
   */
  const endCurrentSession = () => {
    const proj = signals.project();
    const sessionId = signals.currentSessionId();
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

    setters.setProject({ ...proj, sessions } as FFXProject);
    setters.setCurrentSessionId(null);
  };

  /**
   * Start a new session
   */
  const startNewSession = async () => {
    const proj = signals.project();
    if (!proj) return;

    const sessionId = generateId();
    const username = signals.currentUser();
    const appVersion = await getAppVersion();
    const now = nowISO();

    const newSession: ProjectSession = {
      session_id: sessionId,
      user: username,
      started_at: now,
      ended_at: null,
      app_version: appVersion,
    };

    setters.setProject({
      ...proj,
      sessions: [...proj.sessions, newSession],
      current_session_id: sessionId,
    } as FFXProject);
    setters.setCurrentSessionId(sessionId);
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
      evidenceCache: inputEvidenceCache,
      processedDbCache: inputProcessedDbCache,
      caseDocumentsCache: inputCaseDocsCache,
    } = options;

    const existingProject = signals.project();
    const now = nowISO();
    const username = signals.currentUser();
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

    // Build processed database state with full caching
    const processedDbState: ProcessedDatabaseState = {
      loaded_paths: processedDatabases.map(db => db.path),
      selected_path: selectedProcessedDb?.path || null,
      detail_view_type: inputProcessedDbCache?.detailViewType || null,
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
      // Include full database objects for complete restoration
      cached_databases: inputProcessedDbCache?.databases || processedDatabases,
      // Include AXIOM-specific cached data
      cached_axiom_case_info: inputProcessedDbCache?.axiomCaseInfo,
      cached_artifact_categories: inputProcessedDbCache?.artifactCategories,
    };

    // Build case documents cache (if provided)
    let caseDocumentsCache: CaseDocumentsCache | undefined;
    if (inputCaseDocsCache && inputCaseDocsCache.documents.length > 0) {
      caseDocumentsCache = {
        documents: inputCaseDocsCache.documents.map(doc => ({
          path: doc.path,
          filename: doc.filename,
          document_type: doc.document_type,
          size: doc.size,
          format: doc.format,
          case_number: doc.case_number,
          evidence_id: doc.evidence_id,
          modified: doc.modified,
        })),
        search_path: inputCaseDocsCache.searchPath,
        cached_at: now,
        valid: true,
      };
    }

    // Build evidence cache from current state (if provided)
    let evidenceCache: EvidenceCache | undefined;
    if (inputEvidenceCache) {
      const { discoveredFiles, fileInfoMap, fileHashMap } = inputEvidenceCache;
      
      // Convert discovered files to cacheable format
      const cachedFiles: CachedDiscoveredFile[] = discoveredFiles.map(f => ({
        path: f.path,
        filename: f.filename,
        container_type: f.container_type,
        size: f.size,
        segment_count: f.segment_count,
        created: f.created,
        modified: f.modified,
      }));

      // Convert file info map to cacheable format (serialize unknown types)
      const cachedFileInfo: Record<string, CachedContainerInfo> = {};
      fileInfoMap.forEach((info, path) => {
        cachedFileInfo[path] = {
          container: info.container,
          ad1: info.ad1 ?? undefined,
          e01: info.e01 ?? undefined,
          l01: info.l01 ?? undefined,
          raw: info.raw ?? undefined,
          archive: info.archive ?? undefined,
          ufed: info.ufed ?? undefined,
          note: info.note ?? undefined,
          companion_log: info.companion_log ?? undefined,
        };
      });

      // Convert file hash map to cacheable format
      const cachedHashes: Record<string, CachedFileHash> = {};
      fileHashMap.forEach((hashInfo, path) => {
        cachedHashes[path] = {
          algorithm: hashInfo.algorithm,
          hash: hashInfo.hash,
          verified: hashInfo.verified,
          computed_at: now,
        };
      });

      evidenceCache = {
        discovered_files: cachedFiles,
        file_info: cachedFileInfo,
        computed_hashes: cachedHashes,
        cached_at: now,
        valid: true,
      };
    }

    // Merge UI state
    const mergedUIState: ProjectUIState = {
      ...createDefaultUIState(),
      ...(existingProject?.ui_state || {}),
      ...uiState,
    };

    // Use existing filter state or provided, or create default
    const filterState: FilterState = inputFilterState || existingProject?.filter_state || createDefaultFilterState();

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
      current_session_id: signals.currentSessionId() || undefined,
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
      evidence_cache: evidenceCache,

      // Processed Databases
      processed_databases: processedDbState,

      // Case Documents
      case_documents_cache: caseDocumentsCache,

      // Bookmarks & Notes
      bookmarks: existingProject?.bookmarks || [],
      notes: existingProject?.notes || [],
      tags: existingProject?.tags || [],

      // Reports
      reports: existingProject?.reports || [],

      // Searches
      saved_searches: existingProject?.saved_searches || [],
      recent_searches: existingProject?.recent_searches || [],
      filter_state: filterState,

      // UI State
      ui_state: mergedUIState,

      // Settings
      settings: existingProject?.settings || createDefaultSettings(),

      // Custom data
      custom_data: existingProject?.custom_data,
    };

    return proj;
  };

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
    const username = signals.currentUser();
    const appVersion = await getAppVersion();
    const proj = createEmptyProject(rootPath, username, appVersion);

    setters.setProject(proj);
    setters.setProjectPath(null); // Not saved yet
    setters.setCurrentSessionId(proj.current_session_id || null);
    setters.setModified(true);

    // Start auto-save if enabled
    autoSave.startAutoSave();

    return proj;
  };

  /**
   * Save current project state
   */
  const saveProject = async (
    options: BuildProjectOptions,
    customPath?: string
  ): Promise<ProjectSaveResult> => {
    try {
      setters.setLoading(true);

      // Build project from current state
      const proj = await buildProjectFromState(options);

      // Determine save path
      let savePath = customPath || signals.projectPath();

      // If no path, ask for one
      if (!savePath) {
        const defaultPath = await getDefaultProjectPath(options.rootPath);
        const selected = await save({
          defaultPath,
          filters: [{ name: "CORE-FFX Project", extensions: ["cffx"] }],
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
        setters.setProject(proj);
        setters.setProjectPath(result.path || savePath);
        setters.setModified(false);
        setters.setError(null);
        setters.setLastAutoSave(new Date());

        // Log the save
        logger.logActivity('project', 'save', `Project saved to ${savePath}`, savePath);

        console.log(`Project saved to: ${result.path}`);
      } else {
        setters.setError(result.error || "Failed to save project");
      }

      return result;
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setters.setError(errorMsg);
      return { success: false, error: errorMsg };
    } finally {
      setters.setLoading(false);
    }
  };

  /**
   * Save project to a new location (Save As)
   */
  const saveProjectAs = async (options: BuildProjectOptions): Promise<ProjectSaveResult> => {
    const defaultPath = await getDefaultProjectPath(options.rootPath);
    const selected = await save({
      defaultPath,
      filters: [{ name: "CORE-FFX Project", extensions: ["cffx"] }],
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
      setters.setLoading(true);
      let loadPath = customPath;

      // If no path provided, show file picker
      if (!loadPath) {
        const selected = await open({
          filters: [{ name: "CORE-FFX Project", extensions: ["cffx"] }],
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

        setters.setProject(result.project);
        setters.setProjectPath(loadPath);
        setters.setModified(false);
        setters.setError(null);

        // Start a new session for this user
        await startNewSession();

        // Start auto-save
        autoSave.startAutoSave();

        // Log the load
        logger.logActivity('project', 'load', `Project loaded: ${result.project.name}`, loadPath);

        console.log(`Project loaded: ${result.project.name}`);
        return { project: result.project, warnings: result.warnings };
      } else {
        const errorMsg = result.error || "Failed to load project";
        setters.setError(errorMsg);
        return { project: null, error: errorMsg };
      }
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setters.setError(errorMsg);
      return { project: null, error: errorMsg };
    } finally {
      setters.setLoading(false);
    }
  };

  /**
   * Clear current project
   */
  const clearProject = () => {
    endCurrentSession();
    autoSave.stopAutoSave();
    setters.setProject(null);
    setters.setProjectPath(null);
    setters.setModified(false);
    setters.setError(null);
    setters.setCurrentSessionId(null);
  };

  return {
    checkProjectExists,
    getDefaultProjectPath,
    createProject,
    saveProject,
    saveProjectAs,
    loadProject,
    clearProject,
  };
}

/**
 * Export buildProjectFromState for external use
 */
export { getAppVersion };
