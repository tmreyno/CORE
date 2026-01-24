// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Accessor, Setter } from "solid-js";
import type { OpenTab, TabViewMode, SelectedEntry, TreeExpansionState } from "../../components";
import type { CenterTab, CenterPaneViewMode } from "../../components/layout/CenterPane";
import type { DiscoveredFile, CaseDocument, ContainerInfo, AxiomCaseInfo, ArtifactCategorySummary } from "../../types";
import type { ProjectTab, ProjectTabType } from "../../types/project";
import type { useFileManager, useHashManager, useProject, useProcessedDatabases } from "../../hooks";
import type { LeftPanelTab } from "../../components";
import type { CenterTabForSave } from "./types";

export interface BuildSaveOptionsParams {
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  /** @deprecated Use centerTabs instead */
  openTabs?: Accessor<OpenTab[]>;
  /** New: Center pane tabs (unified system) */
  centerTabs?: Accessor<CenterTab[]>;
  /** New: Active tab ID */
  activeTabId?: Accessor<string | null>;
  /** New: View mode */
  viewMode?: Accessor<CenterPaneViewMode>;
  selectedContainerEntry: Accessor<SelectedEntry | null>;
  leftWidth: Accessor<number>;
  rightWidth: Accessor<number>;
  leftCollapsed: Accessor<boolean>;
  rightCollapsed: Accessor<boolean>;
  leftPanelTab: Accessor<LeftPanelTab>;
  currentViewMode: Accessor<TabViewMode>;
  entryContentViewMode: Accessor<"auto" | "hex" | "text" | "document">;
  caseDocumentsPath: Accessor<string | null>;
  treeExpansionState: Accessor<TreeExpansionState | null>;
  caseDocuments: Accessor<CaseDocument[] | null>;
}

/**
 * Build the save options object for project save operations.
 * Used by auto-save, Cmd+S, and manual save button.
 * 
 * Saves the following state:
 * - Root path (scan directory)
 * - Open tabs (evidence, documents, entries, processed databases)
 * - Active tab ID and view mode
 * - Hash history for all files
 * - Processed databases state (including detail view type)
 * - UI state:
 *   - Panel dimensions and collapse states
 *   - Active left panel tab
 *   - Detail view mode
 *   - Selected container entry (for resuming work)
 *   - Entry content view mode (auto/hex/text/document)
 *   - Case documents path
 *   - Tree expansion state (which containers/folders are expanded)
 * - Filter state (type filter for evidence tree)
 * - Evidence cache (discovered files, container info, computed hashes)
 * - Processed database cache (full database objects, AXIOM case info, artifacts)
 * - Case documents cache (discovered documents and search path)
 */
export function buildSaveOptions(params: BuildSaveOptionsParams) {
  const scanDir = params.fileManager.scanDir();
  if (!scanDir) return null;
  
  // Capture selected entry for restoration
  const entry = params.selectedContainerEntry();
  const selectedEntryData = entry ? {
    containerPath: entry.containerPath,
    entryPath: entry.entryPath,
    name: entry.name,
  } : null;
  
  // Convert CenterTabs to serializable format
  const centerTabs: CenterTabForSave[] = params.centerTabs?.() 
    ? params.centerTabs().map(tab => ({
        id: tab.id,
        type: tab.type as ProjectTabType,
        title: tab.title,
        subtitle: tab.subtitle,
        file: tab.file,
        documentPath: tab.documentPath,
        entry: tab.entry,
        processedDb: tab.processedDb,
      }))
    : [];
  
  // Also include legacy openTabs for backwards compatibility if no centerTabs
  const legacyOpenTabs = params.openTabs?.() || [];
  
  return {
    rootPath: scanDir,
    // New center tabs system
    centerTabs: centerTabs.length > 0 ? centerTabs : undefined,
    activeTabId: params.activeTabId?.() || null,
    viewMode: params.viewMode?.() || "info",
    // Legacy tabs for backwards compatibility
    openTabs: centerTabs.length === 0 ? legacyOpenTabs : [],
    activeTabPath: params.fileManager.activeFile()?.path || null,
    hashHistory: params.hashManager.hashHistory(),
    processedDatabases: params.processedDbManager.databases(),
    selectedProcessedDb: params.processedDbManager.selectedDatabase(),
    uiState: {
      left_panel_width: params.leftWidth(),
      right_panel_width: params.rightWidth(),
      left_panel_collapsed: params.leftCollapsed(),
      right_panel_collapsed: params.rightCollapsed(),
      left_panel_tab: params.leftPanelTab(),
      detail_view_mode: params.currentViewMode(),
      // New fields for improved restoration
      selected_entry: selectedEntryData,
      entry_content_view_mode: params.entryContentViewMode(),
      case_documents_path: params.caseDocumentsPath() || undefined,
      // Tree expansion state for restoring which containers/folders are expanded
      tree_expansion_state: params.treeExpansionState() || undefined,
    },
    // Save filter state for evidence tree
    filterState: {
      type_filter: params.fileManager.typeFilter(),
      status_filter: null,
      search_query: null,
      sort_by: 'name',
      sort_direction: 'asc' as const,
    },
    evidenceCache: {
      discoveredFiles: params.fileManager.discoveredFiles(),
      fileInfoMap: params.fileManager.fileInfoMap(),
      fileHashMap: params.hashManager.fileHashMap(),
    },
    processedDbCache: {
      databases: params.processedDbManager.databases(),
      axiomCaseInfo: params.processedDbManager.axiomCaseInfo(),
      artifactCategories: params.processedDbManager.artifactCategories(),
      detailViewType: params.processedDbManager.detailView()?.type || null,
    },
    caseDocumentsCache: params.caseDocuments() ? {
      documents: params.caseDocuments()!,
      searchPath: params.caseDocumentsPath() || scanDir,
    } : undefined,
  };
}

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
  };
}

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
        const matchedFile = discoveredFiles.find(f => f.path === savedTab.file_path);
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
        const matchedDoc = caseDocuments.find(d => d.path === docPath);
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
        const matchedDb = processedDatabases.find(db => db.path === savedTab.processed_db_path);
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
    }
  }
  
  return restoredTabs;
}

/**
 * Handle loading a project file and restoring all state.
 * 
 * Restoration steps:
 * 1. Set scan directory (root path)
 * 2. Restore evidence cache (discovered files, container info, computed hashes)
 * 3. Restore UI state (panel sizes, collapse states, view modes)
 * 4. Restore filter state (type filter for evidence tree)
 * 5. Restore tabs (evidence, documents, entries, processed databases)
 * 6. Restore selected container entry
 * 7. Restore hash history
 * 8. Restore processed databases state
 * 9. Restore case documents cache
 */
export async function handleLoadProject(params: HandleLoadProjectParams) {
  const { 
    fileManager, hashManager, projectManager, processedDbManager,
    setLeftWidth, setRightWidth, setLeftCollapsed, setRightCollapsed,
    setLeftPanelTab, setCurrentViewMode, setEntryContentViewMode,
    setCaseDocumentsPath, setTreeExpansionState, setSelectedContainerEntry,
    setOpenTabs, setCaseDocuments, setCenterTabs, setActiveTabId, setCenterViewMode, toast
  } = params;
  
  try {
    const result = await projectManager.loadProject();
    if (!result.project) return;
    
    const project = result.project;
    
    // ===========================================================================
    // STEP 1: Set scan directory (root path)
    // ===========================================================================
    fileManager.setScanDir(project.root_path);
    
    // ===========================================================================
    // STEP 2: Restore evidence cache (discovered files, container info, hashes)
    // ===========================================================================
    const cache = project.evidence_cache;
    if (cache && cache.valid && cache.discovered_files.length > 0) {
      console.log("[Project Load] Using cached evidence state");
      
      fileManager.restoreDiscoveredFiles(cache.discovered_files as DiscoveredFile[]);
      
      if (cache.file_info && Object.keys(cache.file_info).length > 0) {
        fileManager.restoreFileInfoMap(cache.file_info as Record<string, ContainerInfo>);
      }
      
      if (cache.computed_hashes && Object.keys(cache.computed_hashes).length > 0) {
        hashManager.restoreFileHashMap(cache.computed_hashes);
      }
      
      console.log(`  - Restored ${cache.discovered_files.length} files from cache`);
      console.log(`  - Restored ${Object.keys(cache.file_info || {}).length} file info entries`);
      console.log(`  - Restored ${Object.keys(cache.computed_hashes || {}).length} computed hashes`);
    } else {
      console.log("[Project Load] No evidence cache, scanning directory...");
      await fileManager.scanForFiles(project.root_path);
    }
    
    // ===========================================================================
    // STEP 3: Restore UI state (panel sizes, view modes, selected entry, etc.)
    // ===========================================================================
    if (project.ui_state) {
      const ui = project.ui_state;
      
      // Panel dimensions and collapse states
      if (ui.left_panel_width) setLeftWidth(ui.left_panel_width);
      if (ui.right_panel_width) setRightWidth(ui.right_panel_width);
      if (ui.left_panel_collapsed !== undefined) setLeftCollapsed(ui.left_panel_collapsed);
      if (ui.right_panel_collapsed !== undefined) setRightCollapsed(ui.right_panel_collapsed);
      
      // Active panels and view modes
      if (ui.left_panel_tab) setLeftPanelTab(ui.left_panel_tab);
      if (ui.detail_view_mode) setCurrentViewMode(ui.detail_view_mode as TabViewMode);
      
      // Entry content view mode (how to display files inside containers)
      if (ui.entry_content_view_mode) {
        setEntryContentViewMode(ui.entry_content_view_mode);
        console.log(`  - Restored entry content view mode: ${ui.entry_content_view_mode}`);
      }
      
      // Case documents path (where to look for case documents)
      if (ui.case_documents_path) {
        setCaseDocumentsPath(ui.case_documents_path);
        console.log(`  - Restored case documents path: ${ui.case_documents_path}`);
      }
      
      // Tree expansion state (which containers/folders are expanded in the tree)
      if (ui.tree_expansion_state) {
        setTreeExpansionState(ui.tree_expansion_state as TreeExpansionState);
        console.log(`  - Restored tree expansion state`);
      }
    }
    
    // ===========================================================================
    // STEP 4: Restore filter state (type filter for evidence tree)
    // ===========================================================================
    if (project.filter_state?.type_filter) {
      fileManager.setTypeFilter(project.filter_state.type_filter);
      console.log(`  - Restored type filter: ${project.filter_state.type_filter}`);
    }
    
    // ===========================================================================
    // STEP 5: Restore tabs (evidence, documents, entries, processed databases)
    // ===========================================================================
    if (project.tabs && project.tabs.length > 0) {
      const discoveredFiles = fileManager.discoveredFiles();
      const processedDatabases = processedDbManager.databases();
      const caseDocsList: CaseDocument[] = [];
      
      // Check if we have new-style tabs (with type field) and setCenterTabs available
      const hasNewStyleTabs = project.tabs.some(t => t.type && t.type !== "evidence");
      
      if (setCenterTabs && (hasNewStyleTabs || project.center_pane_state)) {
        // Use new CenterTabs system
        const restoredCenterTabs = restoreCenterTabs(
          project.tabs,
          discoveredFiles,
          processedDatabases,
          caseDocsList
        );
        
        if (restoredCenterTabs.length > 0) {
          setCenterTabs(restoredCenterTabs);
          console.log(`  - Restored ${restoredCenterTabs.length} center pane tabs`);
          
          // Restore active tab and view mode
          if (project.center_pane_state?.active_tab_id && setActiveTabId) {
            setActiveTabId(project.center_pane_state.active_tab_id);
          } else if (restoredCenterTabs.length > 0 && setActiveTabId) {
            setActiveTabId(restoredCenterTabs[0].id);
          }
          
          if (project.center_pane_state?.view_mode && setCenterViewMode) {
            setCenterViewMode(project.center_pane_state.view_mode as CenterPaneViewMode);
          }
          
          // Set active file for evidence tabs
          const activeTab = restoredCenterTabs.find(t => 
            t.id === project.center_pane_state?.active_tab_id
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
          
          const matchedFile = discoveredFiles.find(f => f.path === savedTab.file_path);
          if (matchedFile) {
            restoredTabs.push({
              file: matchedFile,
              id: savedTab.file_path,
              viewMode: (savedTab.container_type === "export" ? "export" : undefined) as TabViewMode | undefined,
            });
          }
        }
        
        if (restoredTabs.length > 0) {
          setOpenTabs(restoredTabs);
          const activeTab = project.active_tab_path 
            ? restoredTabs.find(t => t.file.path === project.active_tab_path)
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
      console.log(`  - Restored selected entry: ${savedEntry.name}`);
    }
    
    // ===========================================================================
    // STEP 7: Restore hash history
    // ===========================================================================
    if (project.hash_history?.files && Object.keys(project.hash_history.files).length > 0) {
      hashManager.restoreHashHistory(project.hash_history.files);
      console.log(`  - Restored hash history for ${Object.keys(project.hash_history.files).length} files`);
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
          pd.cached_axiom_case_info as Record<string, AxiomCaseInfo> | undefined,
          pd.cached_artifact_categories as Record<string, ArtifactCategorySummary[]> | undefined,
          pd.detail_view_type // Restore the detail view type (e.g., 'artifacts', 'timeline')
        );
        console.log(`  - Restored ${pd.cached_databases.length} processed databases from cache`);
        if (pd.detail_view_type) {
          console.log(`  - Restored detail view type: ${pd.detail_view_type}`);
        }
      } else if (pd.loaded_paths && pd.loaded_paths.length > 0) {
        await processedDbManager.restoreFromProject(
          pd.loaded_paths,
          pd.selected_path,
          pd.cached_metadata
        );
      }
    }
    
    // ===========================================================================
    // STEP 9: Restore case documents cache
    // ===========================================================================
    const docsCache = project.case_documents_cache;
    if (docsCache && docsCache.valid && docsCache.documents && docsCache.documents.length > 0) {
      setCaseDocuments(docsCache.documents as CaseDocument[]);
      // Also restore the search path if we have documents
      if (docsCache.search_path) {
        setCaseDocumentsPath(docsCache.search_path);
      }
      console.log(`  - Restored ${docsCache.documents.length} case documents from cache`);
    }
    
    // Log restoration summary
    console.log(`Project restored: ${project.name}`);
    console.log(`  - Sessions: ${project.sessions?.length || 0}`);
    console.log(`  - Activity log entries: ${project.activity_log?.length || 0}`);
    console.log(`  - Bookmarks: ${project.bookmarks?.length || 0}`);
    console.log(`  - Notes: ${project.notes?.length || 0}`);
    console.log(`  - Saved searches: ${project.saved_searches?.length || 0}`);
    
    toast.success("Project Loaded", `Opened: ${project.name}`);
  } catch (err) {
    console.error("Load project error:", err);
    toast.error("Load Failed", "Could not load the project");
  }
}

/**
 * Create a SelectedEntry from a CaseDocument for viewing.
 */
export function createDocumentEntry(doc: CaseDocument, isDiskFile = true): SelectedEntry {
  return {
    containerPath: doc.path,
    entryPath: doc.path,
    name: doc.filename,
    size: doc.size,
    isDir: false,
    isDiskFile,
    containerType: doc.format || 'file',
    metadata: {
      document_type: doc.document_type,
      case_number: doc.case_number,
      format: doc.format,
    },
  };
}

// =============================================================================
// Project Setup Handlers
// =============================================================================

import type { ProjectLocations } from "../../components";
import { logError, logInfo } from "../../utils/telemetry";
import { announce } from "../../utils/accessibility";

export interface HandleOpenDirectoryParams {
  setPendingProjectRoot: Setter<string | null>;
  setShowProjectWizard: Setter<boolean>;
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
}

/**
 * Handle opening a project directory - shows the setup wizard.
 */
export async function handleOpenDirectory(params: HandleOpenDirectoryParams) {
  const { setPendingProjectRoot, setShowProjectWizard, toast } = params;
  const { open } = await import('@tauri-apps/plugin-dialog');
  
  try {
    const selected = await open({ 
      title: "Select Project Directory", 
      multiple: false, 
      directory: true 
    });
    if (selected) {
      setPendingProjectRoot(selected);
      setShowProjectWizard(true);
    }
  } catch (err) {
    console.error("Failed to open directory:", err);
    logError(err instanceof Error ? err : new Error("Failed to open directory dialog"), { 
      category: "ui", 
      source: "handleOpenDirectory" 
    });
    toast.error("Failed to Open", "Could not open directory dialog");
  }
}

export interface HandleProjectSetupCompleteParams {
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  projectManager: ReturnType<typeof useProject>;
  setShowProjectWizard: Setter<boolean>;
  setCaseDocumentsPath: Setter<string | null>;
  setLeftPanelTab: Setter<LeftPanelTab>;
  setPendingProjectRoot: Setter<string | null>;
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
}

/**
 * Handle when project setup wizard completes.
 */
export async function handleProjectSetupComplete(
  params: HandleProjectSetupCompleteParams,
  locations: ProjectLocations
) {
  const { 
    fileManager, 
    hashManager,
    processedDbManager, 
    projectManager, 
    setShowProjectWizard, 
    setCaseDocumentsPath, 
    setLeftPanelTab, 
    setPendingProjectRoot,
    toast 
  } = params;
  
  setShowProjectWizard(false);
  
  // Set the evidence path and scan for files
  // Don't auto-load hashes since we may have pre-loaded them in wizard
  fileManager.setScanDir(locations.evidencePath);
  
  // If we have pre-loaded stored hashes from wizard step 2, import them to hash manager
  if (locations.loadedStoredHashes && locations.loadedStoredHashes.size > 0) {
    hashManager.importPreloadedStoredHashes(locations.loadedStoredHashes);
    // Scan without auto-loading hashes (we already have them)
    await fileManager.scanForFiles(locations.evidencePath, undefined, true);
  } else {
    // No pre-loaded hashes - let scanForFiles auto-load in background
    await fileManager.scanForFiles(locations.evidencePath);
  }
  
  // Store case documents path for CaseDocumentsPanel
  setCaseDocumentsPath(locations.caseDocumentsPath || locations.evidencePath);
  
  // If processed databases were discovered, add them
  if (locations.discoveredDatabases.length > 0) {
    // Add discovered processed databases to the manager
    processedDbManager.addDatabases(locations.discoveredDatabases);
    // Select the first discovered database to show details
    await processedDbManager.selectDatabase(locations.discoveredDatabases[0]);
    // Switch to processed tab
    setLeftPanelTab("processed");
    console.log(`Found ${locations.discoveredDatabases.length} processed databases in: ${locations.processedDbPath}`);
  }
  
  // Log the project setup and notify user
  projectManager.logActivity('project', 'setup', 
    `Project setup complete: Evidence=${locations.evidencePath}, Processed=${locations.processedDbPath}, CaseDocs=${locations.caseDocumentsPath}`);
  logInfo("Project setup complete", { source: "handleProjectSetupComplete", context: { locations } });
  toast.success("Project Ready", `Found ${fileManager.discoveredFiles().length} files`);
  announce(`Project setup complete. Found ${fileManager.discoveredFiles().length} evidence files.`);
  
  setPendingProjectRoot(null);
}
