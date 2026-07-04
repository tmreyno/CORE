// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Project I/O operations (save, load, create)
 */

import { batch } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { logAuditAction } from "../../utils/telemetry";
import { logger as appLogger } from "../../utils/logger";
import { addRecentProject } from "../../components/preferences";
import { getBasename } from "../../utils/pathUtils";
import { isTauri } from "../../utils/platform";
import { dbSync } from "./useProjectDbSync";
import { seedDatabaseFromProject } from "./useProjectDbRead";

const log = appLogger.scope("ProjectIO");
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
  ClearProjectOptions,
  ProjectStateSignals,
  ProjectStateSetters,
  ActivityLogger,
  ProjectIO,
  ProjectCloseResult,
} from "./types";

export function uniqueProjectFilePaths(tabs: ProjectTab[]): string[] {
  return Array.from(new Set(tabs.map(tab => tab.file_path).filter(Boolean)));
}

type BrowserProjectFile = {
  path: string;
  project: FFXProject;
};

let activeBrowserProjectPickerCancel: (() => void) | null = null;
const BROWSER_PROJECT_PICKER_CANCEL_GRACE_MS = 1_500;

export function parseBrowserProjectFile(content: string, fallbackPath: string): BrowserProjectFile {
  const parsed = JSON.parse(content) as Partial<FFXProject>;

  if (!parsed || typeof parsed !== "object" || !parsed.name || !parsed.root_path) {
    throw new Error("Selected file is not a valid CORE-FFX project");
  }

  return {
    path: fallbackPath,
    project: {
      ...(parsed as FFXProject),
      version: typeof parsed.version === "number" ? parsed.version : PROJECT_FILE_VERSION,
      project_id: parsed.project_id || generateId(),
      tabs: parsed.tabs || [],
      users: parsed.users || [],
      sessions: parsed.sessions || [],
      activity_log: parsed.activity_log || [],
      bookmarks: parsed.bookmarks || [],
      notes: parsed.notes || [],
      tags: parsed.tags || [],
      reports: parsed.reports || [],
      saved_searches: parsed.saved_searches || [],
      recent_directories: parsed.recent_directories || [],
      open_directories: parsed.open_directories || [],
    },
  };
}

export function pickBrowserProjectFile(): Promise<BrowserProjectFile | null> {
  if (typeof document === "undefined") {
    return Promise.resolve(null);
  }

  activeBrowserProjectPickerCancel?.();

  return new Promise((resolve, reject) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".cffx,application/json";
    input.style.display = "none";
    let settled = false;
    let focusCheck: number | undefined;

    const cleanup = () => {
      window.removeEventListener("focus", handleWindowFocus);
      if (focusCheck !== undefined) {
        window.clearTimeout(focusCheck);
      }
      if (activeBrowserProjectPickerCancel === cancelCurrent) {
        activeBrowserProjectPickerCancel = null;
      }
      input.remove();
    };

    const finish = (selected: BrowserProjectFile | null) => {
      if (settled) return;
      settled = true;
      cleanup();
      resolve(selected);
    };

    const fail = (err: unknown) => {
      if (settled) return;
      settled = true;
      cleanup();
      reject(err);
    };

    const cancelCurrent = () => {
      finish(null);
    };

    function handleWindowFocus() {
      if (focusCheck !== undefined) {
        window.clearTimeout(focusCheck);
      }

      focusCheck = window.setTimeout(() => {
        if (!input.files?.length) {
          finish(null);
        }
      }, BROWSER_PROJECT_PICKER_CANCEL_GRACE_MS);
    }

    activeBrowserProjectPickerCancel = cancelCurrent;

    input.addEventListener("cancel", () => {
      finish(null);
    }, { once: true });

    input.addEventListener("change", async () => {
      try {
        const file = input.files?.[0];
        if (!file) {
          finish(null);
          return;
        }

        const content = await file.text();
        const fallbackPath = file.name.endsWith(".cffx") ? file.name : `${file.name}.cffx`;
        finish(parseBrowserProjectFile(content, fallbackPath));
      } catch (err) {
        fail(err);
      }
    }, { once: true });

    window.addEventListener("focus", handleWindowFocus);
    document.body.appendChild(input);
    input.click();
  });
}

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

    // Write-through: end session in .ffxdb
    const endedSession = sessions.find(s => s.session_id === sessionId);
    dbSync.endSession(sessionId, endedSession?.summary);

    logger.logActivity('system', 'session_end', `Session ended`, undefined, {
      sessionId,
      durationSeconds: sessions.find(s => s.session_id === sessionId)?.duration_seconds,
    });
    logger.flushActivity();
  };

  /**
   * Start a new session
   */
  const startNewSession = async (): Promise<void> => {
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

    // Write-through: user MUST be created before session (FK: sessions.user → users.username)
    dbSync.upsertUser({
      username,
      firstAccess: now,
      lastAccess: now,
    });
    dbSync.upsertSession({
      sessionId,
      user: username,
      startedAt: now,
      appVersion,
    });

    logger.logActivity('system', 'session_start', `Session started`, undefined, {
      sessionId,
      user: username,
      appVersion,
    });
    markModified();
  };

  /**
   * Build project from current app state (extended version)
   */
  const buildProjectFromState = async (options: BuildProjectOptions): Promise<FFXProject> => {
    const {
      rootPath,
      projectName,
      centerTabs = [],
      activeTabId = null,
      viewMode = "info",
      openTabs = [], // legacy fallback
      activeTabPath = null, // legacy fallback
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
    const name = projectName || existingProject?.name || getBasename(rootPath) || 'Untitled';

    // Convert center tabs to project format (new system)
    let tabs: ProjectTab[];
    let computedActiveTabPath: string | null = activeTabPath;
    
    if (centerTabs.length > 0) {
      // Use new center tabs system
      tabs = centerTabs.map((tab, index) => {
        const projectTab: ProjectTab = {
          id: tab.id,
          type: tab.type,
          file_path: tab.file?.path || tab.documentPath || tab.entry?.containerPath || tab.processedDb?.path || "",
          name: tab.title,
          subtitle: tab.subtitle,
          order: index,
          container_type: tab.file?.container_type,
          document_path: tab.documentPath,
          entry_path: tab.entry?.entryPath,
          entry_container_path: tab.entry?.containerPath,
          entry_name: tab.entry?.name,
          processed_db_path: tab.processedDb?.path,
          processed_db_type: tab.processedDb?.db_type,
          collection_id: tab.collectionId,
          collection_read_only: tab.collectionReadOnly,
          collection_list_view: tab.collectionListView,
          last_viewed: now,
        };
        return projectTab;
      });
      
      // Find active tab path for backwards compatibility
      if (activeTabId) {
        const activeTab = centerTabs.find(t => t.id === activeTabId);
        if (activeTab) {
          computedActiveTabPath = activeTab.file?.path || activeTab.documentPath || activeTab.entry?.containerPath || activeTab.processedDb?.path || null;
        }
      }
    } else if (openTabs.length > 0) {
      // Fallback to legacy tab format
      tabs = openTabs.map((tab, index) => ({
        id: `evidence:${tab.file.path}`,
        type: "evidence" as const,
        file_path: tab.file.path,
        name: tab.file.filename,
        order: index,
        container_type: tab.file.container_type,
        last_viewed: now,
      }));
      computedActiveTabPath = activeTabPath;
    } else {
      tabs = [];
    }

    // Convert hash history to project format
    const hashHistoryObj: ProjectHashHistory = { files: {} };
    hashHistory.forEach((entries, filePath) => {
      // Filter out entries with missing/invalid hash values
      const validEntries = entries.filter(entry => 
        entry.hash && typeof entry.hash === 'string' && entry.hash.trim().length > 0
      );
      
      if (validEntries.length === 0) return; // Skip files with no valid hashes
      
      hashHistoryObj.files[filePath] = validEntries.map(entry => {
        // Safely convert timestamp to ISO string, handling invalid dates
        let timestampStr: string;
        if (entry.timestamp instanceof Date && !isNaN(entry.timestamp.getTime())) {
          timestampStr = entry.timestamp.toISOString();
        } else if (typeof entry.timestamp === 'string' && entry.timestamp && entry.timestamp !== 'Invalid Date') {
          timestampStr = entry.timestamp;
        } else {
          timestampStr = now; // Fallback to current time
        }
        
        return {
          algorithm: entry.algorithm || 'UNKNOWN',
          hash_value: entry.hash, // Already validated above
          computed_at: timestampStr,
          verification:
            entry.verified !== undefined && entry.verified_against
              ? {
                  result: entry.verified ? ("match" as const) : ("mismatch" as const),
                  verified_against: entry.verified_against,
                  verified_at: timestampStr,
                }
              : undefined,
        };
      });
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
        segment_count: f.segment_count ?? 1,
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
          computed_at: hashInfo.computedAt || now,
          verified_against: hashInfo.verifiedAgainst,
          comparison_source: hashInfo.comparisonSource,
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
      activity_log_limit: existingProject?.activity_log_limit ?? 1000,

      // Evidence State
      open_directories: existingProject?.open_directories || [{
        path: rootPath,
        opened_at: now,
        recursive: true,
        file_count: tabs.length,
        total_size: 0,
        last_scanned: now,
      }],
      recent_directories: existingProject?.recent_directories || [],
      tabs,
      active_tab_path: computedActiveTabPath,
      // Store center pane state for proper restoration
      center_pane_state: centerTabs.length > 0 ? {
        active_tab_id: activeTabId,
        view_mode: viewMode,
      } : undefined,
      file_selection: {
        selected_paths: uniqueProjectFilePaths(tabs),
        active_path: computedActiveTabPath,
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

      // Preserve fields that are set during creation or load but not rebuilt here
      owner_name: existingProject?.owner_name,
      case_number: existingProject?.case_number,
      case_name: existingProject?.case_name,
      locations: existingProject?.locations,
      db_path: existingProject?.db_path,
      preview_cache: existingProject?.preview_cache,
      merge_sources: existingProject?.merge_sources,
    };

    return proj;
  };

  /**
   * Check if a project exists for the given root directory
   */
  const checkProjectExists = async (rootPath: string): Promise<string | null> => {
    if (!isTauri) {
      void rootPath;
      return null;
    }

    try {
      return await invoke<string | null>("project_check_exists", { rootPath });
    } catch (e) {
      log.warn("Failed to check project:", e);
      return null;
    }
  };

  /**
   * Get the default project path for a root directory
   */
  const getDefaultProjectPath = async (rootPath: string): Promise<string> => {
    if (!isTauri) {
      const trimmed = rootPath.replace(/\/+$/, "");
      const baseName = trimmed.split(/[\\/]/).filter(Boolean).pop() || "CORE-FFX-Project";
      return `${baseName}.cffx`;
    }

    return await invoke<string>("project_get_default_path", { rootPath });
  };

  /**
   * Create a new project for the given root directory
   */
  const createProject = async (
    rootPath: string,
    projectName?: string,
    ownerName?: string,
    caseNumber?: string,
    caseName?: string,
  ): Promise<FFXProject> => {
    log.debug(`createProject called for rootPath=${rootPath}, name=${projectName}, owner=${ownerName}, case=${caseNumber}`);
    const username = signals.currentUser();
    const appVersion = await getAppVersion();
    const proj = createEmptyProject(rootPath, username, appVersion, projectName, caseNumber, caseName);

    // Set owner_name if provided
    if (ownerName) {
      proj.owner_name = ownerName;
    }

    batch(() => {
      setters.setProject(proj);
      setters.setProjectPath(null); // Not saved yet
      setters.setCurrentSessionId(proj.current_session_id || null);
      setters.setModified(true);
    });
    if (isTauri) {
      invoke("set_project_menu_state", { hasProject: true }).catch(() => {});
    }
    log.debug(`createProject: Project created, modified=true, name=${proj.name}`);

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
    log.debug(`saveProject called with customPath=${customPath}, rootPath=${options.rootPath}`);
    try {
      setters.setLoading(true);

      // Build project from current state
      const proj = await buildProjectFromState(options);
      log.debug(`saveProject: Built project name=${proj.name}, version=${proj.version}`);

      // Sanitize null fields that Rust expects as non-optional (bool, u32, etc.)
      // Serde treats explicit JSON null differently from missing keys —
      // missing keys fall back to #[serde(default)], but null causes a type error.
      // Removing the key lets serde use its default.
      const sanitizeNullFields = (obj: Record<string, unknown>, path = ''): void => {
        for (const [key, value] of Object.entries(obj)) {
          const currentPath = path ? `${path}.${key}` : key;
          if (value === null) {
            // Known numeric keys that Rust expects as u32/u64 (non-optional)
            const numericKeys = ['version', 'order', 'activity_log_limit', 'size', 'segment_count', 
                                'file_count', 'total_size', 'duration_seconds', 'result_count',
                                'left_panel_width', 'right_panel_width', 'width', 'height', 'font_size',
                                'auto_save_interval', 'max_recent_items'];
            // Known boolean keys that Rust expects as bool (non-optional)
            const booleanKeys = ['auto_save', 'verify_hashes_on_load', 'track_activity',
                                'left_panel_collapsed', 'right_panel_collapsed', 'expanded',
                                'show_hidden_files', 'confirm_on_close', 'valid', 'verified',
                                'is_regex', 'case_sensitive', 'auto_discovered',
                                'load_stored_hashes', 'recursive', 'success'];
            if (numericKeys.some(k => key.includes(k) || key === k)) {
              log.warn(`Null numeric field at ${currentPath}, removing from object`);
              delete obj[key];
            } else if (booleanKeys.some(k => key === k)) {
              log.warn(`Null boolean field at ${currentPath}, removing from object`);
              delete obj[key];
            }
          } else if (value && typeof value === 'object' && !Array.isArray(value)) {
            sanitizeNullFields(value as Record<string, unknown>, currentPath);
          } else if (Array.isArray(value)) {
            value.forEach((item, idx) => {
              if (item && typeof item === 'object') {
                sanitizeNullFields(item as Record<string, unknown>, `${currentPath}[${idx}]`);
              }
            });
          }
        }
      };

      // Sanitize the project object
      sanitizeNullFields(proj as unknown as Record<string, unknown>);

      // Validate critical fields before sending to backend
      if (typeof proj.version !== 'number' || proj.version === null || proj.version === undefined) {
        log.error("Invalid project version:", proj.version, "Resetting to", PROJECT_FILE_VERSION);
        proj.version = PROJECT_FILE_VERSION;
      }
      if (!proj.project_id) {
        proj.project_id = generateId();
      }
      if (!proj.root_path) {
        return { success: false, error: "No root path specified" };
      }

      if (!isTauri) {
        const errorMsg = "Project saving is available in the desktop app. Browser preview can load .cffx files but cannot write them back to disk.";
        setters.setError(errorMsg);
        return { success: false, error: errorMsg };
      }

      // Deep check for nulls in numeric fields - log any found
      const findNullNumericFields = (obj: unknown, path = ''): string[] => {
        const nullPaths: string[] = [];
        if (obj === null) return [path];
        if (typeof obj !== 'object' || obj === null) return [];
        
        for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
          const currentPath = path ? `${path}.${key}` : key;
          if (value === null) {
            nullPaths.push(currentPath);
          } else if (typeof value === 'object') {
            if (Array.isArray(value)) {
              value.forEach((item, idx) => {
                nullPaths.push(...findNullNumericFields(item, `${currentPath}[${idx}]`));
              });
            } else {
              nullPaths.push(...findNullNumericFields(value, currentPath));
            }
          }
        }
        return nullPaths;
      };
      
      const nullFields = findNullNumericFields(proj);
      if (nullFields.length > 0 && import.meta.env.DEV) {
        // Debug only - these nulls are expected for optional fields
        log.debug(`Optional null fields: ${nullFields.length}`);
      }

      // Determine save path - always show dialog unless customPath provided
      let savePath = customPath;

      if (!savePath) {
        // Use existing project path as default, or generate one from root
        const existingPath = signals.projectPath();
        const defaultPath = existingPath || await getDefaultProjectPath(options.rootPath);
        
        const selected = await save({
          defaultPath,
          filters: [{ name: "CORE-FFX Project", extensions: ["cffx"] }],
          title: "Save Project",
        });

        if (!selected) {
          log.debug("saveProject: User cancelled save dialog");
          return { success: false, error: "Save cancelled" };
        }
        savePath = selected;
        log.debug(`saveProject: User selected save path: ${savePath}`);
      }

      // Save via Tauri
      log.debug(`saveProject: Invoking project_save for ${savePath}`);
      const result = await invoke<ProjectSaveResult>("project_save", {
        project: proj,
        path: savePath,
      });

      log.debug(`saveProject: Result success=${result.success}, path=${result.path}`);
      if (result.success) {
        setters.setProject(proj);
        setters.setProjectPath(result.path || savePath);
        setters.setModified(false);
        setters.setError(null);
        setters.setLastAutoSave(new Date());
        log.debug("saveProject: State updated, modified=false");

        // Log the save
        logger.logActivity('project', 'save', `Project saved to ${savePath}`, savePath);
        
        // Audit log project saved
        logAuditAction("project_saved", {
          path: savePath,
          projectName: proj.name,
          version: proj.version,
        });

        // Checkpoint WAL to flush .ffxdb data to main file (best-effort)
        // This ensures the .ffxdb is self-contained (no data only in WAL)
        // which is critical for external volumes and project portability.
        invoke("project_db_wal_checkpoint").catch((e) => {
          log.debug(`WAL checkpoint after save (non-fatal): ${e}`);
        });

        log.info(`saveProject: SUCCESS - saved to ${result.path}`);
      } else {
        log.warn(`saveProject: FAILED - ${result.error}`);
        setters.setError(result.error || "Failed to save project");
      }

      return result;
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      log.error(`saveProject: EXCEPTION - ${errorMsg}`);
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
    if (!isTauri) {
      return {
        success: false,
        error: "Project Save As is available in the desktop app.",
      };
    }

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
    log.debug(`loadProject called with customPath=${customPath}`);
    try {
      let loadPath = customPath;

      // If no path provided, show file picker
      if (!loadPath) {
        log.debug("loadProject: No path provided, showing file picker");
        if (!isTauri) {
          const selected = await pickBrowserProjectFile();

          if (!selected) {
            log.debug("loadProject: User cancelled browser file picker");
            return { project: null, error: "Open cancelled" };
          }

          loadPath = selected.path;
          const project = selected.project;
          const browserProjectPath = selected.path;
          log.debug(`loadProject: Browser selected ${loadPath}`);
          setters.setLoading(true);

          batch(() => {
            setters.setProject(project);
            setters.setProjectPath(browserProjectPath);
            setters.setModified(false);
            setters.setError(null);
          });

          await startNewSession();
          autoSave.startAutoSave();
          addRecentProject(loadPath, project.name);
          logger.logActivity('project', 'load', `Project loaded: ${project.name}`, loadPath);

          return {
            project,
            warnings: ["Loaded in browser preview mode; desktop project database features are unavailable."],
          };
        }

        const selected = await open({
          filters: [{ name: "CORE-FFX Project", extensions: ["cffx"] }],
          title: "Open Project",
          multiple: false,
        });

        if (!selected) {
          log.debug("loadProject: User cancelled file picker");
          return { project: null, error: "Open cancelled" };
        }
        loadPath = selected as string;
        log.debug(`loadProject: User selected ${loadPath}`);
      }

      setters.setLoading(true);

      if (!isTauri) {
        const errorMsg = "Browser preview can only open project files through the Open Project file picker.";
        setters.setError(errorMsg);
        return { project: null, error: errorMsg };
      }

      // Load via Tauri
      log.debug(`loadProject: Invoking project_load for ${loadPath}`);
      const tLoad = performance.now();
      const result = await invoke<ProjectLoadResult>("project_load", {
        path: loadPath,
      });
      log.info(`loadProject: project_load invoke took ${(performance.now() - tLoad).toFixed(1)}ms`);

      log.debug(`loadProject: Result success=${result.success}, hasProject=${!!result.project}`);
      if (result.success && result.project) {
        // End any existing session (fire-and-forget — no await needed)
        endCurrentSession();

        // CRITICAL ORDER: Open the project database (.ffxdb) FIRST, before
        // setting project signals. When setProject() fires, reactive effects
        // throughout the app immediately query the DB (collections, menu state,
        // etc.). If the DB isn't open yet, those queries fail and retry with
        // 500ms delays, causing a visible load stall.
        try {
          const t1 = performance.now();
          const dbMsg = await invoke<string>("project_db_open", {
            cffxPath: loadPath,
          });
          log.info(`loadProject: project_db_open took ${(performance.now() - t1).toFixed(1)}ms — ${dbMsg}`);

          // Seed the .ffxdb from .cffx data if tables are empty (non-blocking)
          seedDatabaseFromProject(result.project).catch((err) => {
            log.warn("DB seeding failed (non-fatal):", err);
          });
        } catch (dbErr) {
          log.warn("Could not open project database:", dbErr);
          // Non-fatal: project still loads without the DB
        }

        // Now set all project signals in a batch — this ensures reactive effects
        // fire only ONCE (after all 4 signals are set) instead of 4 separate
        // times. The DB is already open, so effects that query it will succeed.
        const t0 = performance.now();
        batch(() => {
          setters.setProject(result.project!);
          setters.setProjectPath(loadPath);
          setters.setModified(false);
          setters.setError(null);
        });
        if (isTauri) {
          invoke("set_project_menu_state", { hasProject: true }).catch(() => {});
        }
        log.info(`loadProject: batch signal setters took ${(performance.now() - t0).toFixed(1)}ms`);

        // Start a new session for this user
        const t2 = performance.now();
        await startNewSession();
        log.info(`loadProject: startNewSession took ${(performance.now() - t2).toFixed(1)}ms`);

        // Update current_user on the in-memory project to match the real OS username
        const currentProj = signals.project();
        if (currentProj && currentProj.current_user !== signals.currentUser()) {
          setters.setProject({ ...currentProj, current_user: signals.currentUser() } as FFXProject);
        }

        // Start auto-save
        autoSave.startAutoSave();
        log.debug("loadProject: AutoSave started");

        // Add to recent projects list
        addRecentProject(loadPath, result.project.name);
        log.debug("loadProject: Added to recent projects");

        // Log the load
        logger.logActivity('project', 'load', `Project loaded: ${result.project.name}`, loadPath);
        
        // Audit log project loaded
        logAuditAction("project_loaded", {
          path: loadPath,
          projectName: result.project.name,
          version: result.project.version,
        });

        log.info(`loadProject: SUCCESS - ${result.project.name}`);
        return { project: result.project, warnings: result.warnings };
      } else {
        const errorMsg = result.error || "Failed to load project";
        log.warn(`loadProject: FAILED - ${errorMsg}`);
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
  const clearProject = async (
    options: ClearProjectOptions = {},
  ): Promise<ProjectCloseResult> => {
    const emitProgress = (
      step: "end-session" | "flush-db-sync" | "checkpoint-db" | "close-db" | "clear-state",
      status: "running" | "completed" | "warning" | "failed",
      detail?: string,
    ) => {
      options.onProgress?.({ step, status, detail });
    };

    try {
      emitProgress("end-session", "running", "Finalizing the active examiner session.");
      endCurrentSession();
      emitProgress("end-session", "completed", "Session history captured for the project close.");

      autoSave.stopAutoSave();

      emitProgress("flush-db-sync", "running", "Waiting for pending .ffxdb writes to finish.");
      const flushResult = await dbSync.flushPendingWrites();
      if (flushResult.timedOut) {
        emitProgress(
          "flush-db-sync",
          "warning",
          `${flushResult.pendingCount} background write${flushResult.pendingCount === 1 ? "" : "s"} still pending when close continued.`,
        );
      } else {
        emitProgress("flush-db-sync", "completed", "All pending .ffxdb writes completed.");
      }

      emitProgress("checkpoint-db", "running", "Checkpointing the project database WAL.");
      if (!isTauri) {
        emitProgress("checkpoint-db", "completed", "Project database checkpoint skipped in browser preview.");
      } else {
        try {
          await invoke("project_db_wal_checkpoint");
          emitProgress("checkpoint-db", "completed", "Project database checkpoint completed.");
        } catch (err) {
          emitProgress(
            "checkpoint-db",
            "warning",
            err instanceof Error ? err.message : String(err),
          );
        }
      }

      emitProgress("close-db", "running", "Closing the project database connection.");
      if (!isTauri) {
        emitProgress("close-db", "completed", "Project database close skipped in browser preview.");
      } else {
        try {
          await invoke("project_db_close");
          emitProgress("close-db", "completed", "Project database closed.");
        } catch (err) {
          emitProgress(
            "close-db",
            "warning",
            err instanceof Error ? err.message : String(err),
          );
        }
      }

      emitProgress("clear-state", "running", "Clearing in-memory project state and UI bindings.");
      batch(() => {
        setters.setProject(null);
        setters.setProjectPath(null);
        setters.setModified(false);
        setters.setError(null);
        setters.setCurrentSessionId(null);
      });
      if (isTauri) {
        await invoke("set_project_menu_state", { hasProject: false }).catch(() => {});
      }
      emitProgress("clear-state", "completed", "Project state cleared.");

      return {
        success: true,
        flushTimedOut: flushResult.timedOut,
      };
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      emitProgress("clear-state", "failed", errorMsg);
      return {
        success: false,
        error: errorMsg,
      };
    }
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
