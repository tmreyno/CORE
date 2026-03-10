// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Project loading & state restoration from .cffx files.
 *
 * Handles all 10 restoration steps: scan directory, evidence cache, UI state,
 * filter state, tabs, selected entry, hash history, processed databases,
 * case documents, and per-project database (.ffxdb).
 */

import type { Setter } from "solid-js";
import type { OpenTab, TabViewMode, SelectedEntry, TreeExpansionState } from "../../components";
import type { CenterTab, CenterPaneViewMode } from "../../components/layout/CenterPane";
import type { DiscoveredFile, CaseDocument, ContainerInfo, AxiomCaseInfo, ArtifactCategorySummary } from "../../types";
import type { ProjectTab } from "../../types/project";
import type { useFileManager, useHashManager, useProject, useProcessedDatabases } from "../../hooks";
import { getDirname } from "../../utils/pathUtils";
import type { LeftPanelTab } from "../../components";
import { logger } from "../../utils/logger";

// Create a scoped logger for project operations
const log = logger.scope("Project");

// ─── Params Interface ────────────────────────────────────────────────────────

export interface HandleLoadProjectParams {
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  projectManager: ReturnType<typeof useProject>;
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  setLeftWidth: (width: number) => void;
  setRightWidth: (width: number) => void;
  setLeftCollapsed: (collapsed: boolean) => void;
  setRightCollapsed: (collapsed: boolean) => void;
  setLeftPanelTab: Setter<LeftPanelTab>;
  setCurrentViewMode: Setter<TabViewMode>;
  setEntryContentViewMode: Setter<"auto" | "hex" | "text" | "document">;
  setCaseDocumentsPath: Setter<string | null>;
  setTreeExpansionState: Setter<TreeExpansionState | null>;
  setSelectedContainerEntry: Setter<SelectedEntry | null>;
  /** @deprecated Use setCenterTabs instead */
  setOpenTabs?: Setter<OpenTab[]>;
  setCaseDocuments: Setter<CaseDocument[] | null>;
  /** New: Set center pane tabs */
  setCenterTabs?: Setter<CenterTab[]>;
  /** New: Set active tab ID */
  setActiveTabId?: Setter<string | null>;
  /** New: Set view mode */
  setCenterViewMode?: Setter<CenterPaneViewMode>;
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    warning: (title: string, message?: string) => void;
  };
  /** Optional: Path to load (if not provided, shows file picker) */
  projectPath?: string;
}

// ─── Private Helper: Restore CenterTabs ──────────────────────────────────────

/**
 * Helper to restore CenterTabs from project tabs
 */
function restoreCenterTabs(
  projectTabs: ProjectTab[],
  discoveredFiles: DiscoveredFile[],
  processedDatabases: import("../../types/processed").ProcessedDatabase[],
  caseDocuments: CaseDocument[],
): CenterTab[] {
  const restoredTabs: CenterTab[] = [];

  for (const savedTab of projectTabs) {
    // Skip export tab - handled separately
    if (savedTab.id === "__export__" || savedTab.file_path === "__export__") continue;

    switch (savedTab.type) {
      case "evidence": {
        const matchedFile = discoveredFiles.find((f) => f.path === savedTab.file_path);
        if (matchedFile) {
          restoredTabs.push({
            id: savedTab.id || `evidence:${savedTab.file_path}`,
            type: "evidence",
            title: savedTab.name,
            subtitle: savedTab.container_type,
            file: matchedFile,
            closable: true,
          });
        }
        break;
      }
      case "document": {
        const docPath = savedTab.document_path || savedTab.file_path;
        const matchedDoc = caseDocuments.find((d) => d.path === docPath);
        if (matchedDoc || docPath) {
          restoredTabs.push({
            id: savedTab.id || `document:${docPath}`,
            type: "document",
            title: savedTab.name,
            documentPath: docPath,
            closable: true,
          });
        }
        break;
      }
      case "entry": {
        if (savedTab.entry_path && savedTab.entry_container_path) {
          restoredTabs.push({
            id: savedTab.id || `entry:${savedTab.entry_path}`,
            type: "entry",
            title: savedTab.entry_name || savedTab.name,
            entry: {
              containerPath: savedTab.entry_container_path,
              entryPath: savedTab.entry_path,
              name: savedTab.entry_name || savedTab.name,
              size: 0,
              isDir: false,
            },
            closable: true,
          });
        }
        break;
      }
      case "processed": {
        const matchedDb = processedDatabases.find(
          (db) => db.path === savedTab.processed_db_path,
        );
        if (matchedDb) {
          restoredTabs.push({
            id: savedTab.id || `processed:${savedTab.processed_db_path}`,
            type: "processed",
            title: savedTab.name,
            subtitle: savedTab.processed_db_type,
            processedDb: matchedDb,
            closable: true,
          });
        }
        break;
      }
      case "collection": {
        restoredTabs.push({
          id: savedTab.id || (savedTab.collection_list_view
            ? "__collection_list__"
            : savedTab.collection_id
              ? `collection:${savedTab.collection_id}`
              : "__collection_new__"),
          type: "collection",
          title: savedTab.name || (savedTab.collection_list_view ? "Evidence Collections" : "Evidence Collection"),
          collectionId: savedTab.collection_id,
          collectionReadOnly: savedTab.collection_read_only,
          collectionListView: savedTab.collection_list_view,
          closable: true,
        });
        break;
      }
      case "help": {
        restoredTabs.push({
          id: savedTab.id || "__help__",
          type: "help",
          title: savedTab.name || "Help & Documentation",
          closable: true,
        });
        break;
      }
      default: {
        // Handle legacy tabs without a type field - infer type from available data
        // Legacy tabs typically have file_path pointing to evidence containers
        if (savedTab.file_path) {
          const matchedFile = discoveredFiles.find(
            (f) => f.path === savedTab.file_path,
          );
          if (matchedFile) {
            restoredTabs.push({
              id: savedTab.id || `evidence:${savedTab.file_path}`,
              type: "evidence",
              title: savedTab.name,
              subtitle: savedTab.container_type,
              file: matchedFile,
              closable: true,
            });
          }
        }
        break;
      }
    }
  }

  return restoredTabs;
}

// ─── Main Load Handler ───────────────────────────────────────────────────────

/**
 * Handle loading a project file and restoring all state.
 *
 * Restoration steps:
 * 1. Set scan directory (root path)
 * 2. Restore evidence cache (discovered files, container info, computed hashes)
 * 3. Restore UI state (panel sizes, view modes, selected entry, etc.)
 * 4. Restore filter state (type filter for evidence tree)
 * 5. Restore tabs (evidence, documents, entries, processed databases)
 * 6. Restore selected container entry
 * 7. Restore hash history
 * 8. Restore processed databases state
 * 9. Restore case documents cache
 *
 * NOTE: project_db_open + DB seeding is handled inside loadProject()
 * (useProjectIO.ts) before startNewSession(), so dbSync calls have an
 * open database.
 */
export async function handleLoadProject(params: HandleLoadProjectParams) {
  const {
    fileManager,
    hashManager,
    projectManager,
    processedDbManager,
    setLeftWidth,
    setRightWidth,
    setLeftCollapsed,
    setRightCollapsed,
    setLeftPanelTab,
    setCurrentViewMode,
    setEntryContentViewMode,
    setCaseDocumentsPath,
    setTreeExpansionState,
    setSelectedContainerEntry,
    setOpenTabs,
    setCaseDocuments,
    setCenterTabs,
    setActiveTabId,
    setCenterViewMode,
    toast,
    projectPath,
  } = params;

  try {
    // Save current scanDir so we can restore it if the user cancels.
    const previousScanDir = fileManager.scanDir();

    // Clear scanDir BEFORE loading so the toolbar doesn't flash a stale path
    // while the new project's locations are being set by loadProject().
    // loadProject() internally calls setProject() which triggers the
    // projectLocations memo — if scanDir still holds the old path, the
    // <select> value won't match any new option.
    fileManager.setScanDir("");

    // Load project, optionally from a specific path
    const result = await projectManager.loadProject(projectPath);
    if (!result.project) {
      // Restore previous scanDir on cancel / failure
      fileManager.setScanDir(previousScanDir);
      if (result.error && result.error !== "Open cancelled") {
        toast.error("Load Failed", result.error);
      }
      return;
    }

    // Display any warnings (e.g., version migration)
    if (result.warnings && result.warnings.length > 0) {
      for (const warning of result.warnings) {
        toast.warning("Project Notice", warning);
      }
    }

    const project = result.project;

    // ===========================================================================
    // STEP 1: Set scan directory
    // ===========================================================================
    // Use the project's evidence_path when available so the toolbar dropdown
    // <select> value matches the evidence option. Fall back to root_path for
    // older projects that lack explicit locations.
    const initialScanDir =
      project.locations?.evidence_path || project.root_path;
    fileManager.setScanDir(initialScanDir);

    // ===========================================================================
    // STEP 1b: Ensure project locations are populated for toolbar dropdown
    // ===========================================================================
    // If the project already has locations (created via wizard), they are already
    // on the project object. For older projects that lack a `locations` field,
    // derive them from available project data so the toolbar dropdown works.
    if (!project.locations?.evidence_path) {
      // Derive processed DB folder from cached database paths
      let processedDbPath = "";
      const pd = project.processed_databases;
      if (pd?.cached_databases && pd.cached_databases.length > 0) {
        const firstDbPath = (
          pd.cached_databases[0] as { path?: string }
        )?.path;
        if (firstDbPath) {
          processedDbPath = getDirname(firstDbPath) || "";
        }
      } else if (pd?.loaded_paths && pd.loaded_paths.length > 0) {
        const firstPath = pd.loaded_paths[0];
        processedDbPath = getDirname(firstPath) || "";
      }

      // Derive case documents path from UI state or docs cache
      const caseDocsPath =
        project.ui_state?.case_documents_path ||
        project.case_documents_cache?.search_path ||
        "";

      projectManager.updateLocations({
        project_root: project.root_path,
        evidence_path: project.root_path,
        processed_db_path: processedDbPath,
        case_documents_path: caseDocsPath || undefined,
        auto_discovered: true,
        configured_at: new Date().toISOString(),
      });
      log.debug("Derived project locations from loaded project data");
    }

    // ===========================================================================
    // STEP 2: Restore evidence cache (discovered files, container info, hashes)
    // ===========================================================================
    const cache = project.evidence_cache;
    if (cache && cache.valid && cache.discovered_files.length > 0) {
      log.debug("Using cached evidence state");

      fileManager.restoreDiscoveredFiles(
        cache.discovered_files as DiscoveredFile[],
      );

      if (cache.file_info && Object.keys(cache.file_info).length > 0) {
        fileManager.restoreFileInfoMap(
          cache.file_info as Record<string, ContainerInfo>,
        );
      }

      if (
        cache.computed_hashes &&
        Object.keys(cache.computed_hashes).length > 0
      ) {
        hashManager.restoreFileHashMap(cache.computed_hashes);
      }

      log.debug(
        `Restored ${cache.discovered_files.length} files from cache`,
      );
      log.debug(
        `Restored ${Object.keys(cache.file_info || {}).length} file info entries`,
      );
      log.debug(
        `Restored ${Object.keys(cache.computed_hashes || {}).length} computed hashes`,
      );
    } else {
      log.debug("No evidence cache, scanning directory...");
      await fileManager.scanForFiles(initialScanDir);
    }

    // ===========================================================================
    // STEP 3: Restore UI state (panel sizes, view modes, selected entry, etc.)
    // ===========================================================================
    if (project.ui_state) {
      const ui = project.ui_state;

      // Panel dimensions and collapse states
      if (ui.left_panel_width) setLeftWidth(ui.left_panel_width);
      if (ui.right_panel_width) setRightWidth(ui.right_panel_width);
      if (ui.left_panel_collapsed !== undefined)
        setLeftCollapsed(ui.left_panel_collapsed);
      if (ui.right_panel_collapsed !== undefined)
        setRightCollapsed(ui.right_panel_collapsed);

      // Active panels and view modes
      if (ui.left_panel_tab) setLeftPanelTab(ui.left_panel_tab);
      if (ui.detail_view_mode)
        setCurrentViewMode(ui.detail_view_mode as TabViewMode);

      // Entry content view mode (how to display files inside containers)
      if (ui.entry_content_view_mode) {
        setEntryContentViewMode(ui.entry_content_view_mode);
        log.debug(
          `Restored entry content view mode: ${ui.entry_content_view_mode}`,
        );
      }

      // Case documents path (where to look for case documents)
      if (ui.case_documents_path) {
        setCaseDocumentsPath(ui.case_documents_path);
        log.debug(
          `Restored case documents path: ${ui.case_documents_path}`,
        );
      }

      // Tree expansion state (which containers/folders are expanded in the tree)
      if (ui.tree_expansion_state) {
        setTreeExpansionState(
          ui.tree_expansion_state as TreeExpansionState,
        );
        log.debug("Restored tree expansion state");
      }
    }

    // ===========================================================================
    // STEP 4: Restore filter state (type filter for evidence tree)
    // ===========================================================================
    if (project.filter_state?.type_filter) {
      fileManager.setTypeFilter(project.filter_state.type_filter);
      log.debug(
        `Restored type filter: ${project.filter_state.type_filter}`,
      );
    }

    // ===========================================================================
    // STEP 5: Restore tabs (evidence, documents, entries, processed databases)
    // ===========================================================================
    if (project.tabs && project.tabs.length > 0) {
      const discoveredFiles = fileManager.discoveredFiles();
      const processedDatabases = processedDbManager.databases();
      const caseDocsList: CaseDocument[] = [];

      // Check if we have new-style tabs (with type field) and setCenterTabs available
      const hasNewStyleTabs = project.tabs.some(
        (t) => t.type && t.type !== "evidence",
      );

      if (setCenterTabs && (hasNewStyleTabs || project.center_pane_state)) {
        // Use new CenterTabs system
        const restoredCenterTabs = restoreCenterTabs(
          project.tabs,
          discoveredFiles,
          processedDatabases,
          caseDocsList,
        );

        if (restoredCenterTabs.length > 0) {
          setCenterTabs(restoredCenterTabs);
          log.debug(
            `Restored ${restoredCenterTabs.length} center pane tabs`,
          );

          // Restore active tab and view mode
          if (
            project.center_pane_state?.active_tab_id &&
            setActiveTabId
          ) {
            setActiveTabId(project.center_pane_state.active_tab_id);
          } else if (
            restoredCenterTabs.length > 0 &&
            setActiveTabId
          ) {
            setActiveTabId(restoredCenterTabs[0].id);
          }

          if (
            project.center_pane_state?.view_mode &&
            setCenterViewMode
          ) {
            setCenterViewMode(
              project.center_pane_state
                .view_mode as CenterPaneViewMode,
            );
          }

          // Set active file for evidence tabs
          const activeTab =
            restoredCenterTabs.find(
              (t) =>
                t.id === project.center_pane_state?.active_tab_id,
            ) || restoredCenterTabs[0];
          if (activeTab?.file) {
            fileManager.setActiveFile(activeTab.file);
          }
        }
      } else if (setOpenTabs) {
        // Fallback to legacy OpenTabs
        const restoredTabs: OpenTab[] = [];

        for (const savedTab of project.tabs) {
          if (savedTab.file_path === "__export__") continue;

          const matchedFile = discoveredFiles.find(
            (f) => f.path === savedTab.file_path,
          );
          if (matchedFile) {
            restoredTabs.push({
              file: matchedFile,
              id: savedTab.file_path,
              viewMode: (
                savedTab.container_type === "export"
                  ? "export"
                  : undefined
              ) as TabViewMode | undefined,
            });
          }
        }

        if (restoredTabs.length > 0) {
          setOpenTabs(restoredTabs);
          const activeTab = project.active_tab_path
            ? restoredTabs.find(
                (t) => t.file.path === project.active_tab_path,
              )
            : restoredTabs[0];
          if (activeTab) {
            fileManager.setActiveFile(activeTab.file);
          }
        }
      }
    }

    // ===========================================================================
    // STEP 6: Restore selected container entry (file inside container being viewed)
    // ===========================================================================
    if (project.ui_state?.selected_entry) {
      const savedEntry = project.ui_state.selected_entry;
      setSelectedContainerEntry({
        containerPath: savedEntry.containerPath,
        entryPath: savedEntry.entryPath,
        name: savedEntry.name,
        size: 0, // Will be populated when entry is accessed
        isDir: false,
      });
      log.debug(`Restored selected entry: ${savedEntry.name}`);
    }

    // ===========================================================================
    // STEP 7: Restore hash history
    // ===========================================================================
    if (
      project.hash_history?.files &&
      Object.keys(project.hash_history.files).length > 0
    ) {
      hashManager.restoreHashHistory(project.hash_history.files);
      log.debug(
        `Restored hash history for ${Object.keys(project.hash_history.files).length} files`,
      );
    }

    // ===========================================================================
    // STEP 8: Restore processed databases state (includes detail view type)
    // ===========================================================================
    if (project.processed_databases) {
      const pd = project.processed_databases;

      if (pd.cached_databases && pd.cached_databases.length > 0) {
        processedDbManager.restoreFullState(
          pd.cached_databases,
          pd.selected_path,
          pd.cached_axiom_case_info as
            | Record<string, AxiomCaseInfo>
            | undefined,
          pd.cached_artifact_categories as
            | Record<string, ArtifactCategorySummary[]>
            | undefined,
          pd.detail_view_type, // Restore the detail view type (e.g., 'artifacts', 'timeline')
        );
        log.debug(
          `Restored ${pd.cached_databases.length} processed databases from cache`,
        );
        if (pd.detail_view_type) {
          log.debug(
            `Restored detail view type: ${pd.detail_view_type}`,
          );
        }
      } else if (pd.loaded_paths && pd.loaded_paths.length > 0) {
        await processedDbManager.restoreFromProject(
          pd.loaded_paths,
          pd.selected_path,
          pd.cached_metadata,
        );
      }
    }

    // ===========================================================================
    // STEP 9: Restore case documents cache
    // ===========================================================================
    const docsCache = project.case_documents_cache;
    if (
      docsCache &&
      docsCache.valid &&
      docsCache.documents &&
      docsCache.documents.length > 0
    ) {
      setCaseDocuments(docsCache.documents as CaseDocument[]);
      // Also restore the search path if we have documents
      if (docsCache.search_path) {
        setCaseDocumentsPath(docsCache.search_path);
      }
      log.debug(
        `Restored ${docsCache.documents.length} case documents from cache`,
      );
    }

    // NOTE: project_db_open is now called inside loadProject() (useProjectIO.ts)
    // BEFORE startNewSession(), so dbSync calls in session/user/activity logging
    // have an open database. Seeding is also handled there.

    // Log restoration summary
    log.info(`Project restored: ${project.name}`);
    log.debug(`Sessions: ${project.sessions?.length || 0}`);
    log.debug(
      `Activity log entries: ${project.activity_log?.length || 0}`,
    );
    log.debug(`Bookmarks: ${project.bookmarks?.length || 0}`);
    log.debug(`Notes: ${project.notes?.length || 0}`);
    log.debug(
      `Saved searches: ${project.saved_searches?.length || 0}`,
    );

    toast.success("Project Loaded", `Opened: ${project.name}`);
  } catch (err) {
    log.error("Load project error:", err);
    toast.error("Load Failed", "Could not load the project");
  }
}
