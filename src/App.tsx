// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, on, Show, lazy, Suspense } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useFileManager, useHashManager, useDatabase, useProject, useProcessedDatabases, useHistoryContext, usePreferenceEffects, useKeyboardHandler, createSearchHandlers, createContextMenuBuilders, createCommandPaletteActions, useAppState, useDatabaseEffects, useCenterPaneTabs, useActivityManager, useEntryNavigation, useActivityLogging, useProjectActions, type DetailViewType } from "./hooks";
import { useAppLifecycle } from "./hooks/useAppLifecycle";
import { useDualPanelResize } from "./hooks/usePanelResize";
import { Toolbar, StatusBar, DetailPanel, ProgressModal, ContainerEntryViewer, useToast, pathToBreadcrumbs, createContextMenu, useTour, DEFAULT_TOUR_STEPS, useDragDrop, Sidebar, AppModals, RightPanel, CenterPane, LeftPanelContent, ExportPanel } from "./components";
import { QuickActionsBar } from "./components/QuickActionsBar";
import { HiOutlineBolt } from "./components/icons";
import { useWorkspaceProfiles } from "./hooks/useWorkspaceProfiles";
import type { DiscoveredFile } from "./types";
import { createPreferences, getPreference, getRecentProjects } from "./components/preferences";
import { createThemeActions } from "./hooks/useTheme";
import { announce } from "./utils/accessibility";
import { logger } from "./utils/logger";
import ffxLogo from "./assets/branding/core-logo-48.png";
import "./App.css";

// Dev-only: Performance test runner (available in console as window.__runPerfTests)
if (import.meta.env.DEV) {
  import("./utils/perfTestRunner");
}

const log = logger.scope("App");

// ============================================================================
// Lazy-loaded Components (Code Splitting)
// These are heavy components that aren't needed on initial render
// ============================================================================
const ReportWizard = lazy(() => import("./components/report/wizard/ReportWizard").then(m => ({ default: m.ReportWizard })));
const ProcessedDetailPanel = lazy(() => import("./components/ProcessedDetailPanel").then(m => ({ default: m.ProcessedDetailPanel })));
const EvidenceCollectionModal = lazy(() => import("./components/EvidenceCollectionModal").then(m => ({ default: m.EvidenceCollectionModal })));

function App() {
  // ===========================================================================
  // Core Services & Hooks
  // ===========================================================================
  const toast = useToast();
  const history = useHistoryContext();
  const preferences = createPreferences();
  const db = useDatabase();
  const fileManager = useFileManager();
  const hashManager = useHashManager(fileManager);
  const projectManager = useProject();
  const processedDbManager = useProcessedDatabases();
  const workspaceProfiles = useWorkspaceProfiles();
  
  // Theme actions (uses preferences as single source of truth)
  const themeActions = createThemeActions(
    () => preferences.preferences().theme,
    (theme) => preferences.updatePreference("theme", theme)
  );
  
  // Apply preferences to UI (font size, etc.)
  usePreferenceEffects(preferences.preferences);
  
  // ===========================================================================
  // UI State - Panels & Layout
  // ===========================================================================
  const panels = useDualPanelResize({
    left: { initialWidth: 320, minWidth: 150, maxWidth: 600, startCollapsed: false },
    right: { initialWidth: 320, minWidth: 150, maxWidth: 500, startCollapsed: false },
  });
  // Panel aliases for cleaner template usage
  const { width: leftWidth, collapsed: leftCollapsed, setWidth: setLeftWidth, setCollapsed: setLeftCollapsed } = panels.left;
  const { width: rightWidth, collapsed: rightCollapsed, setWidth: setRightWidth, setCollapsed: setRightCollapsed } = panels.right;
  
  // ===========================================================================
  // UI State - consolidated from useAppState hook
  // ===========================================================================
  const appState = useAppState();
  const { modals, views, project, leftPanel } = appState;
  
  // Destructure for easier access
  const { showCommandPalette, setShowCommandPalette, showShortcutsModal, setShowShortcutsModal, 
          showPerformancePanel, setShowPerformancePanel, showSettingsPanel, setShowSettingsPanel,
          showSearchPanel, setShowSearchPanel, showWelcomeModal, setShowWelcomeModal,
          showReportWizard, setShowReportWizard, showProjectWizard, setShowProjectWizard,
          showEvidenceCollection, setShowEvidenceCollection } = modals;
  
  const { openTabs, setOpenTabs, currentViewMode, setCurrentViewMode, hexMetadata, setHexMetadata,
          selectedContainerEntry, setSelectedContainerEntry, entryContentViewMode, setEntryContentViewMode,
          requestViewMode, setRequestViewMode, hexNavigator, setHexNavigator, 
          treeExpansionState, setTreeExpansionState } = views;
  
  const { pendingProjectRoot, setPendingProjectRoot, caseDocumentsPath, setCaseDocumentsPath,
          caseDocuments, setCaseDocuments } = project;
  
  const { leftPanelTab, setLeftPanelTab, leftPanelMode, setLeftPanelMode } = leftPanel;
  
  // Viewer metadata for right panel (emitted by ContainerEntryViewer)
  const [viewerMetadata, setViewerMetadata] = createSignal<import("./types/viewerMetadata").ViewerMetadata | null>(null);
  
  // Report wizard: optional pre-selected report type from sidebar context menu
  const [initialReportType, setInitialReportType] = createSignal<import("./components/report/types").ReportType | undefined>(undefined);
  
  // Activity Tracking — lifecycle managed by useActivityManager hook
  const activityManager = useActivityManager();
  const { activities, setActivities } = activityManager;
  
  // ===========================================================================
  // Unified Center Pane Tabs - new unified tab management
  // ===========================================================================
  const centerPaneTabs = useCenterPaneTabs();
  
  // Quick Actions Bar visibility (hidden by default, toggled via title bar button)
  const [showQuickActions, setShowQuickActions] = createSignal(false);
  
  // ===========================================================================
  // Derived State & Computed Values
  // ===========================================================================
  
  const breadcrumbItems = () => {
    const activeFile = fileManager.activeFile();
    if (!activeFile) return [];
    return pathToBreadcrumbs(activeFile.path);
  };
  
  const activeFileInfo = () => {
    const active = fileManager.activeFile();
    if (!active) return undefined;
    return fileManager.fileInfoMap().get(active.path);
  };
  
  // Activity progress items for status bar
  const activityProgressItems = (): import("./components").ProgressItem[] => {
    const active = activities().filter(a => a.status === "running" || a.status === "pending" || a.status === "paused");
    return active.map(activity => ({
      id: activity.id,
      label: `${activity.type === "archive" ? "Archive" : activity.type === "export" ? "Export" : "Copy"}: ${activity.progress?.currentFile?.split("/").pop() || "preparing..."}`,
      progress: activity.progress?.percent ?? 0,
      indeterminate: activity.status === "pending",
      onClick: () => setRequestViewMode("export"),
    }));
  };

  // Stable case documents path - only changes when explicit case documents path changes
  // Don't use activeFile here to avoid loops when selecting documents
  const stableCaseDocsPath = createMemo(() => {
    const locations = projectManager.projectLocations();
    return caseDocumentsPath() || 
           locations?.case_documents_path || 
           locations?.evidence_path ||
           null;
  });
  
  // Autosave status for StatusBar indicator
  const autoSaveStatus = createMemo((): import("./components/StatusBar").AutoSaveStatus => {
    if (fileManager.busy()) return "saving"; // Rough proxy for saving state
    if (projectManager.modified()) return "modified";
    if (projectManager.lastAutoSave()) return "saved";
    return "idle";
  });
  
  // Recent projects for welcome modal (convert to RecentProjectInfo format)
  const welcomeModalRecentProjects = createMemo(() => {
    // Re-read on showWelcomeModal change to ensure freshness
    void showWelcomeModal();
    return getRecentProjects().map(p => ({
      path: p.path,
      name: p.name,
      lastOpened: p.lastOpened,
    }));
  });
  
  // ===========================================================================
  // Effects - View State Synchronization
  // ===========================================================================
  
  // Clear metadata when active file changes
  createEffect(() => {
    void fileManager.activeFile();
    setHexMetadata(null);
    setCurrentViewMode("info");
    setSelectedContainerEntry(null);
  });
  
  // Clear viewer metadata when switching away from entry/document tabs
  createEffect(() => {
    const tabType = centerPaneTabs.activeTabType();
    if (tabType !== "entry" && tabType !== "document") {
      setViewerMetadata(null);
    }
  });
  
  // Auto-verify hashes when a file becomes active (if preference enabled)
  const autoVerifiedFiles = new Set<string>();
  createEffect(on(
    () => fileManager.activeFile(),
    (active) => {
      if (!active || !getPreference("autoVerifyHashes")) return;
      
      // Only auto-verify once per file per session
      if (autoVerifiedFiles.has(active.path)) return;
      
      autoVerifiedFiles.add(active.path);
      log.debug(`Auto-verifying: ${active.path}`);
      hashManager.hashSingleFile(active);
    },
    { defer: true }
  ));
  
  // Note: Export view mode is handled by DetailPanel via requestViewMode prop
  // DetailPanel will call onViewModeRequestHandled() when it processes the request
  // Do NOT clear requestViewMode here - it creates a race condition

  // ===========================================================================
  // Activity Logging Effects (extracted to useActivityLogging hook)
  // ===========================================================================
  
  useActivityLogging({ fileManager, hashManager, projectManager, activities, tabs: centerPaneTabs.tabs });
  
  // ===========================================================================
  // Handler Functions
  // ===========================================================================
  
  const handleHexNavigatorReady = (nav: (offset: number, size?: number) => void) => {
    setHexNavigator(() => nav);
  };
  
  // Search handlers from useAppActions
  const { handleSearch, handleSearchResultSelect } = createSearchHandlers({ fileManager, projectManager });

  // Tour hook for guided onboarding
  const tour = useTour({
    steps: DEFAULT_TOUR_STEPS,
    storageKey: "ffx-tour-completed",
    autoStart: false,
    onComplete: () => {
      toast.success("Tour completed! Press ? for keyboard shortcuts.");
    },
    onSkip: () => {
      toast.info("Tour skipped. Press ? for help anytime.");
    }
  });
  
  // Drag and drop for file import
  let appContainerRef: HTMLDivElement | undefined;
  const dragDrop = useDragDrop(
    () => appContainerRef,
    {
      accept: [".e01", ".E01", ".ad1", ".AD1", ".l01", ".L01", ".raw", ".img", ".dd", ".zip", ".7z", ".tar", ".gz"],
      multiple: true,
      allowDirectories: true,
      onDrop: async (_files, paths) => {
        if (paths && paths.length > 0) {
          // Determine the directory from the first dropped file
          const firstPath = paths[0];
          const dirPath = firstPath.substring(0, firstPath.lastIndexOf("/"));
          if (dirPath) {
            // Go through the project wizard for a unified flow
            setPendingProjectRoot(dirPath);
            setShowProjectWizard(true);
            announce(`Opening project wizard for dropped files`);
          }
        } else {
          toast.info("File paths not available - please use the browse button");
        }
      }
    }
  );
  
  // Context menu state
  const fileContextMenu = createContextMenu();
  const saveContextMenu = createContextMenu();
  
  // =========================================================================
  // Project Actions (extracted to useProjectActions hook)
  // =========================================================================
  
  const projectActions = useProjectActions({
    fileManager,
    hashManager,
    projectManager,
    processedDbManager,
    centerPaneTabs,
    toast,
    openTabs,
    selectedContainerEntry,
    leftWidth,
    rightWidth,
    leftCollapsed,
    rightCollapsed,
    leftPanelTab,
    currentViewMode,
    entryContentViewMode,
    caseDocumentsPath,
    treeExpansionState,
    caseDocuments,
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
    setPendingProjectRoot,
    setShowProjectWizard,
  });
  
  // Destructure for convenience
  const { getSaveOptions, handleSaveProject, handleSaveProjectAs, handleLoadProject, handleOpenDirectory, handleProjectSetupComplete } = projectActions;
  
  // ===========================================================================
  // Location-aware selection handler
  // ===========================================================================
  
  /**
   * Handle location selection from the toolbar dropdown.
   * Routes to the appropriate scan based on location type:
   * - "evidence" → scan for evidence files (default behavior)
   * - "processed" → scan for processed databases and sync to ffxdb
   * - "documents" → just set the scan directory
   */
  const handleLocationSelect = async (path: string, locationId: string) => {
    if (locationId === "processed") {
      // Scan for processed databases and sync to ffxdb
      log.info(`Scanning for processed databases at: ${path}`);
      try {
        const results = await invoke<import("./types/processed").ProcessedDatabase[]>(
          "scan_processed_databases",
          { path, recursive: true }
        );
        
        if (results.length > 0) {
          // Add to in-memory manager
          processedDbManager.addDatabases(results);
          
          // Sync each to ffxdb
          const { dbSync } = await import("./hooks/project/useProjectDbSync");
          for (const db of results) {
            dbSync.upsertProcessedDatabase(db);
          }
          
          // Select the first one if none selected
          if (!processedDbManager.selectedDatabase()) {
            await processedDbManager.selectDatabase(results[0]);
          }
          
          // Switch to processed tab in left panel
          setLeftPanelTab("processed");
          
          toast.success(
            "Databases Found",
            `Discovered ${results.length} processed database${results.length !== 1 ? "s" : ""}`
          );
          log.info(`Synced ${results.length} processed databases to ffxdb`);
        } else {
          toast.info("No Databases", "No processed databases found in this directory");
        }
      } catch (err) {
        log.error("Failed to scan for processed databases:", err);
        toast.error("Scan Failed", err instanceof Error ? err.message : String(err));
      }
    } else {
      // Default: set scan dir and scan for evidence files
      fileManager.setScanDir(path);
      fileManager.scanForFiles();
    }
  };
  
  // ===========================================================================
  // Entry Navigation (extracted to useEntryNavigation hook)
  // ===========================================================================
  
  const entryNav = useEntryNavigation({
    fileManager,
    centerPaneTabs,
    processedDbManager,
    setSelectedContainerEntry,
    setEntryContentViewMode,
    toast,
    logActivity: projectManager.logActivity,
  });
  
  // Context menu builders from useAppActions
  const { getFileContextMenuItems } = createContextMenuBuilders({
    fileManager,
    hashManager,
    projectManager,
    toast,
    buildSaveOptions: getSaveOptions,
  });
  
  // ===========================================================================
  // Keyboard Handler Hook - manages global shortcuts
  // ===========================================================================
  useKeyboardHandler({
    setShowCommandPalette,
    setShowSettingsPanel,
    setShowSearchPanel,
    setShowPerformancePanel,
    setShowShortcutsModal,
    setShowProjectWizard,
    setShowReportWizard,
    onLoadProject: () => handleLoadProject(),
    onOpenDirectory: handleOpenDirectory,
    showCommandPalette,
    showShortcutsModal,
    history,
    toast,
    projectManager,
    buildSaveOptions: getSaveOptions,
  });
  
  // Command palette actions - uses extracted factory
  const commandPaletteActions = createCommandPaletteActions({
    fileManager,
    hashManager,
    setCurrentViewMode,
    setLeftCollapsed,
    setRightCollapsed,
    setShowReportWizard,
    setShowSettingsPanel,
    setShowShortcutsModal,
    setShowProjectWizard,
    setShowSearchPanel,
    setShowPerformancePanel,
    setShowEvidenceCollection,
    onOpenDirectory: handleOpenDirectory,
    onOpenProject: () => handleLoadProject(),
  });

  // Database synchronization effects
  useDatabaseEffects({ db, fileManager });

  // ===========================================================================
  // Lifecycle — Window Title, Close Confirmation, Mount & Cleanup
  // (extracted to useAppLifecycle hook)
  // ===========================================================================
  const { isCompact } = useAppLifecycle({
    fileManager,
    projectManager,
    workspaceProfiles,
    db,
    tour,
    preferences,
    getSaveOptions,
    setShowWelcomeModal,
  });

  // Shared DetailPanel props builder — avoids duplicating ~25 props across tab and fallback views
  const sharedDetailPanelProps = (activeFile: DiscoveredFile) => ({
    activeFile,
    fileInfoMap: fileManager.fileInfoMap,
    fileStatusMap: fileManager.fileStatusMap,
    fileHashMap: hashManager.fileHashMap,
    hashHistory: hashManager.hashHistory,
    tree: fileManager.tree(),
    filteredTree: fileManager.filteredTree(),
    treeFilter: fileManager.treeFilter(),
    onTreeFilterChange: (filter: string) => fileManager.setTreeFilter(filter),
    selectedHashAlgorithm: hashManager.selectedHashAlgorithm(),
    storedHashesGetter: hashManager.getAllStoredHashesSorted,
    busy: fileManager.busy(),
    onLoadInfo: (file: DiscoveredFile) => fileManager.loadFileInfo(file, true),
    formatHashDate: hashManager.formatHashDate,
    onTabSelect: (file: DiscoveredFile | null) => file && centerPaneTabs.openEvidenceFile(file),
    onTabsChange: (tabs: import("./components").OpenTab[]) => setOpenTabs(tabs),
    onMetadataLoaded: setHexMetadata,
    onViewModeChange: setCurrentViewMode,
    onHexNavigatorReady: handleHexNavigatorReady,
    requestViewMode: requestViewMode(),
    onViewModeRequestHandled: () => setRequestViewMode(null),
    breadcrumbItems: breadcrumbItems(),
    onBreadcrumbNavigate: (path: string) => {
      log.debug(`Breadcrumb navigate to: ${path}`);
      const matchingFile = fileManager.discoveredFiles().find(f => 
        path.startsWith(f.path) || f.path.includes(path)
      );
      if (matchingFile) {
        entryNav.handleSelectEvidenceFile(matchingFile);
      }
    },
    scanDir: fileManager.scanDir(),
    selectedFiles: fileManager.discoveredFiles().filter(f => 
      fileManager.selectedFiles().has(f.path)
    ),
    onHashComputed: (entries: import("./types").HashHistoryEntry[]) => {
      hashManager.addTransferHashesToHistory(entries);
    },
  });

  return (
    <div ref={appContainerRef} class="app-root" classList={{ 'is-resizing': panels.isDragging() }}>
      {/* Drag overlay */}
      <Show when={dragDrop.isDragging()}>
        <div class="fixed inset-0 z-[1000] bg-bg/90 flex items-center justify-center pointer-events-none">
          <div class={`p-12 rounded-2xl border-2 border-dashed transition-all ${dragDrop.isOver() ? "border-accent bg-accent/20 scale-105" : "border-border-subtle bg-bg-panel/50"}`}>
            <div class="text-6xl mb-4 text-center">📂</div>
            <div class="text-xl font-semibold text-txt text-center">
              {dragDrop.isOver() ? "Release to import" : "Drop evidence files here"}
            </div>
            <div class="text-sm text-txt-secondary text-center mt-2">
              E01, AD1, L01, Raw images, or archives
            </div>
          </div>
        </div>
      </Show>
      
      {/* Skip link for accessibility */}
      <a href="#main-content" class="skip-link">
        Skip to main content
      </a>
      
      {/* All Modals and Overlays */}
      <AppModals
        commandPaletteActions={commandPaletteActions()}
        showCommandPalette={showCommandPalette}
        setShowCommandPalette={setShowCommandPalette}
        showShortcutsModal={showShortcutsModal}
        setShowShortcutsModal={setShowShortcutsModal}
        showPerformancePanel={showPerformancePanel}
        setShowPerformancePanel={setShowPerformancePanel}
        showSettingsPanel={showSettingsPanel}
        setShowSettingsPanel={setShowSettingsPanel}
        preferences={preferences.preferences()}
        onUpdatePreference={(key, value) => preferences.updatePreference(key, value)}
        onUpdateShortcut={(action, shortcut) => preferences.updateShortcut(action, shortcut)}
        onResetToDefaults={preferences.resetToDefaults}
        showSearchPanel={showSearchPanel}
        setShowSearchPanel={setShowSearchPanel}
        onSearch={handleSearch}
        onSelectSearchResult={handleSearchResultSelect}
        fileContextMenu={fileContextMenu}
        saveContextMenu={saveContextMenu}
        showWelcomeModal={showWelcomeModal}
        setShowWelcomeModal={setShowWelcomeModal}
        onNewProject={() => setShowProjectWizard(true)}
        onOpenProject={() => handleLoadProject()}
        recentProjects={welcomeModalRecentProjects}
        onSelectRecentProject={handleLoadProject}
        tour={tour}
        showProjectWizard={showProjectWizard}
        setShowProjectWizard={setShowProjectWizard}
        pendingProjectRoot={pendingProjectRoot}
        setPendingProjectRoot={setPendingProjectRoot}
        onProjectSetupComplete={handleProjectSetupComplete}
      />
      
      {/* Header / Title Bar */}
      <header class="app-header">
        <div class="brand">
          <img src={ffxLogo} alt="CORE-FFX Logo" class="brand-logo" />
        </div>
        
        {/* Project Badge (moved from toolbar) */}
        <Show when={projectManager.projectName()}>
          <div 
            class="flex items-center gap-1.5 px-2.5 py-1 text-sm font-medium text-accent bg-accent/10 rounded-md border border-accent/20 truncate max-w-[220px]"
            title={`Project: ${projectManager.projectName()!}`}
          >
            <span class="truncate">{projectManager.projectName()!}</span>
            <Show when={projectManager.modified()}>
              <span class="w-1.5 h-1.5 rounded-full bg-warning shrink-0" title="Unsaved changes" />
            </Show>
          </div>
        </Show>
        
        {/* Quick Actions Bar Toggle */}
        <div class="ml-auto mr-2">
          <button
            class={`flex items-center justify-center p-1.5 rounded-md transition-all duration-150 ${showQuickActions() ? 'bg-accent/20 text-accent' : 'text-txt-muted hover:text-txt hover:bg-bg-hover'}`}
            onClick={() => setShowQuickActions(!showQuickActions())}
            title={showQuickActions() ? "Hide Quick Actions" : "Show Quick Actions"}
            aria-label={showQuickActions() ? "Hide quick actions bar" : "Show quick actions bar"}
            aria-pressed={showQuickActions()}
          >
            <HiOutlineBolt class="w-4 h-4" />
          </button>
        </div>
      </header>

      {/* Toolbar */}
      <Toolbar
        scanDir={fileManager.scanDir()}
        onScanDirChange={(dir) => fileManager.setScanDir(dir)}
        selectedHashAlgorithm={hashManager.selectedHashAlgorithm()}
        onHashAlgorithmChange={(alg) => hashManager.setSelectedHashAlgorithm(alg)}
        selectedCount={fileManager.selectedCount()}
        discoveredCount={fileManager.discoveredFiles().length}
        busy={fileManager.busy()}
        onSave={handleSaveProject}
        onSaveAs={handleSaveProjectAs}
        autoSaveEnabled={projectManager.autoSaveEnabled}
        onAutoSaveToggle={() => {
          const newEnabled = !projectManager.autoSaveEnabled();
          projectManager.setAutoSaveEnabled(newEnabled);
          if (newEnabled) {
            projectManager.startAutoSave();
          } else {
            projectManager.stopAutoSave();
          }
        }}
        projectModified={projectManager.modified}
        onScan={() => fileManager.scanForFiles()}
        onHashSelected={() => hashManager.hashSelectedFiles()}
        onLoadAll={() => fileManager.loadAllInfo()}
        compact={isCompact()}
        evidencePath={() => projectManager.projectLocations()?.evidence_path ?? (projectManager.hasProject() ? (fileManager.scanDir() || null) : null)}
        processedDbPath={() => {
          const loc = projectManager.projectLocations()?.processed_db_path;
          if (loc) return loc;
          // Fallback: derive from loaded processed databases when project
          // locations haven't been set yet (older projects, first load)
          if (projectManager.hasProject()) {
            const dbs = processedDbManager.databases();
            if (dbs.length > 0) {
              const firstPath = dbs[0].path;
              const lastSlash = firstPath.lastIndexOf("/");
              return lastSlash > 0 ? firstPath.substring(0, lastSlash) : firstPath;
            }
          }
          return null;
        }}
        caseDocumentsPath={() => projectManager.projectLocations()?.case_documents_path ?? (projectManager.hasProject() ? (caseDocumentsPath() ?? null) : null)}
        projectName={projectManager.projectName}
        onLocationSelect={handleLocationSelect}
      />
      
      {/* Quick Actions Bar - hidden by default, toggled via title bar ⚡ button */}
      <Show when={showQuickActions()}>
        <QuickActionsBar
          actions={workspaceProfiles.currentProfile()?.quick_actions}
          compact={isCompact()}
          onAction={(action) => {
            // Handle quick actions
            switch (action.command) {
              case "hash_selected":
                hashManager.hashSelectedFiles();
                break;
              case "open_search":
                setShowSearchPanel(true);
                break;
              case "export_selected":
                setRequestViewMode("export");
                break;
              case "verify_hashes":
                hashManager.hashAllFiles();
                break;
              case "generate_report":
                setShowReportWizard(true);
                break;
              case "evidence_collection":
                setShowEvidenceCollection(true);
                break;
              default:
                toast.info("Action", action.name);
            }
          }}
        />
      </Show>

      {/* Main Content Area */}
      <main class="app-main">
        {/* Left Panel */}
        <Show when={!leftCollapsed()}>
          <aside class="left-panel flex flex-row" style={{ width: `${leftWidth()}px` }}>
            {/* Vertical Icon Sidebar */}
            <Sidebar
              activeTab={leftPanelTab}
              onTabChange={setLeftPanelTab}
              viewMode={leftPanelMode}
              onViewModeChange={setLeftPanelMode}
              busy={fileManager.busy}
              hasEvidence={() => !!fileManager.scanDir()}
              hasDiscoveredFiles={() => fileManager.discoveredFiles().length > 0}
              bookmarkCount={projectManager.bookmarkCount}
              onExport={() => centerPaneTabs.openExportTab()}
              onReport={() => { setInitialReportType(undefined); setShowReportWizard(true); }}
              onReportType={(type) => { setInitialReportType(type); setShowReportWizard(true); }}
              onExportSelected={() => centerPaneTabs.openExportTab()}
              onClearBookmarks={() => projectManager.clearBookmarks?.()}
              onExportBookmarks={() => projectManager.exportBookmarks?.()}
              onSearch={() => setShowSearchPanel(true)}
              onSettings={() => setShowSettingsPanel(true)}
              onPerformance={() => setShowPerformancePanel(true)}
              onCommandPalette={() => setShowCommandPalette(true)}
              onHelp={() => setShowShortcutsModal(true)}
              onEvidenceCollection={() => setShowEvidenceCollection(true)}
              theme={themeActions.theme}
              resolvedTheme={themeActions.resolvedTheme}
              cycleTheme={themeActions.cycleTheme}
            />
            
            {/* Panel Content Area — extracted to LeftPanelContent component */}
            <LeftPanelContent
              leftPanelMode={leftPanelMode}
              leftPanelTab={leftPanelTab}
              discoveredFiles={fileManager.discoveredFiles}
              activeFile={fileManager.activeFile}
              busy={fileManager.busy}
              onSelectContainer={entryNav.handleSelectEvidenceFile}
              onSelectEntry={entryNav.handleSelectEntry}
              typeFilter={fileManager.typeFilter}
              onToggleTypeFilter={(type) => fileManager.toggleTypeFilter(type)}
              onClearTypeFilter={() => fileManager.setTypeFilter(null)}
              containerStats={fileManager.containerStats}
              onOpenNestedContainer={entryNav.handleOpenNestedContainer}
              treeExpansionState={treeExpansionState}
              onTreeExpansionStateChange={setTreeExpansionState}
              selectedFiles={fileManager.selectedFiles}
              fileHashMap={hashManager.fileHashMap}
              hashHistory={hashManager.hashHistory}
              fileStatusMap={fileManager.fileStatusMap}
              fileInfoMap={fileManager.fileInfoMap}
              onToggleFileSelection={(path) => fileManager.toggleFileSelection(path)}
              onHashFile={(file) => hashManager.hashSingleFile(file)}
              onContextMenu={(file, e) => {
                fileManager.setActiveFile(file);
                fileContextMenu.open(e, getFileContextMenuItems(fileManager.activeFile));
              }}
              allFilesSelected={fileManager.allFilesSelected}
              onToggleSelectAll={() => fileManager.toggleSelectAll()}
              setActiveFile={fileManager.setActiveFile}
              processedDbManager={processedDbManager}
              onSelectProcessedDb={entryNav.handleSelectProcessedDb}
              onOpenProcessedDatabase={(db) => centerPaneTabs.openProcessedDatabase(db)}
              caseDocumentsPath={caseDocumentsPath}
              stableCaseDocsPath={stableCaseDocsPath}
              caseDocuments={caseDocuments}
              setCaseDocuments={setCaseDocuments}
              onDocumentSelect={entryNav.handleCaseDocumentSelect}
              projectManager={projectManager}
              toast={toast}
            />
          </aside>
        </Show>

        {/* Left Resize Handle */}
        <div 
          class="resize-handle" 
          classList={{ collapsed: leftCollapsed() }}
          onMouseDown={panels.left.startDrag}
          onClick={() => leftCollapsed() && setLeftCollapsed(false)}
          onDblClick={panels.left.toggleCollapsed}
        >
          <Show when={leftCollapsed()}>
            <span class="expand-icon">›</span>
          </Show>
        </div>

        {/* Center Panel - Unified tabbed interface */}
        <section class="center-panel" id="main-content">
          <CenterPane
            tabs={centerPaneTabs.tabs}
            activeTabId={centerPaneTabs.activeTabId}
            onTabSelect={(tabId) => {
              // Reset entryContentViewMode to "auto" when switching to an entry/document tab
              // so ContainerEntryViewer triggers auto-preview for the newly active tab
              const tab = centerPaneTabs.tabs().find(t => t.id === tabId);
              if (tab && (tab.type === "entry" || tab.type === "document")) {
                setEntryContentViewMode("auto");
              }
              centerPaneTabs.setActiveTabId(tabId);
            }}
            onTabClose={centerPaneTabs.closeTab}
            onTabsChange={centerPaneTabs.setTabs}
            viewMode={centerPaneTabs.viewMode}
            onViewModeChange={centerPaneTabs.setViewMode}
            onOpenProject={handleLoadProject}
            onNewProject={() => setShowProjectWizard(true)}
            projectName={projectManager.projectName}
            projectRoot={projectManager.rootPath}
            evidenceCount={() => fileManager.discoveredFiles().length}
          >
            {/* Content based on active tab type and view mode */}
            <Show when={centerPaneTabs.activeTab()}>
              {(tab) => (
                <>
                  {/* Evidence file tabs - show DetailPanel (handles all view modes internally) */}
                  <Show when={tab().type === "evidence" && tab().file}>
                    <DetailPanel {...sharedDetailPanelProps(tab().file!)} />
                  </Show>
                  
                  {/* Case document tabs - show ContainerEntryViewer using stored entry */}
                  <Show when={tab().type === "document" && tab().documentEntry}>
                    <ContainerEntryViewer
                      entry={tab().documentEntry!}
                      viewMode={entryContentViewMode()}
                      onBack={() => {
                        centerPaneTabs.closeTab(tab().id);
                      }}
                      onViewModeChange={setEntryContentViewMode}
                      onMetadata={setViewerMetadata}
                    />
                  </Show>
                  
                  {/* Container entry tabs - show ContainerEntryViewer */}
                  <Show when={tab().type === "entry" && tab().entry}>
                    <ContainerEntryViewer
                      entry={tab().entry!}
                      viewMode={entryContentViewMode()}
                      onBack={() => centerPaneTabs.closeTab(tab().id)}
                      onViewModeChange={setEntryContentViewMode}
                      onMetadata={setViewerMetadata}
                    />
                  </Show>
                  
                  {/* Processed database tabs */}
                  <Show when={tab().type === "processed" && tab().processedDb}>
                    <Suspense>
                      <ProcessedDetailPanel
                        database={tab().processedDb!}
                        caseInfo={processedDbManager.selectedCaseInfo()}
                        categories={processedDbManager.selectedCategories()}
                        loading={processedDbManager.isSelectedLoading()}
                        detailView={processedDbManager.detailView()}
                        onDetailViewChange={(view: DetailViewType) => processedDbManager.setDetailView(view)}
                      />
                    </Suspense>
                  </Show>
                  
                  {/* Export tab */}
                  <Show when={tab().type === "export"}>
                    <ExportPanel
                      initialSources={fileManager.discoveredFiles()
                        .filter(f => fileManager.selectedFiles().has(f.path))
                        .map(f => f.path)
                      }
                      onComplete={(destination) => {
                        toast.success("Export Complete", `Files exported to: ${destination}`);
                      }}
                      onActivityCreate={(activity) => {
                        setActivities(list => [...list, activity]);
                        // Open right panel to show activity
                        setRightCollapsed(false);
                        // Directly set view mode to export
                        setCurrentViewMode("export");
                        setRequestViewMode("export");
                      }}
                      onActivityUpdate={(id, updates) => {
                        setActivities(list =>
                          list.map(a => a.id === id ? { ...a, ...updates } : a)
                        );
                      }}
                    />
                  </Show>
                </>
              )}
            </Show>
            
            {/* Fallback when no tabs or no active tab - show empty state or legacy DetailPanel */}
            <Show when={!centerPaneTabs.activeTab() && fileManager.activeFile()}>
              <DetailPanel
                {...sharedDetailPanelProps(fileManager.activeFile()!)}
                onTransferStart={() => {
                  // Open right panel when transfer starts to show progress
                  setRightCollapsed(false);
                }}
              />
            </Show>
          </CenterPane>
        </section>

        {/* Right Resize Handle */}
        <div 
          class="resize-handle" 
          classList={{ collapsed: rightCollapsed() }}
          onMouseDown={panels.right.startDrag}
          onClick={() => rightCollapsed() && setRightCollapsed(false)}
          onDblClick={panels.right.toggleCollapsed}
        >
          <Show when={rightCollapsed()}>
            <span class="expand-icon">‹</span>
          </Show>
        </div>

        {/* Right Panel - switches based on view mode */}
        <RightPanel
          collapsed={rightCollapsed}
          width={rightWidth}
          currentViewMode={currentViewMode}
          setRequestViewMode={setRequestViewMode}
          hexMetadata={hexMetadata}
          hexNavigator={hexNavigator}
          activeFile={fileManager.activeFile}
          activeFileInfo={activeFileInfo}
          selectedEntry={selectedContainerEntry}
          viewerMetadata={viewerMetadata}
          activeTabType={centerPaneTabs.activeTabType}
          activities={activities}
          onCancelActivity={activityManager.cancel}
          onClearActivity={activityManager.clear}
          onPauseActivity={activityManager.pause}
          onResumeActivity={activityManager.resume}
        />
      </main>

      {/* Status bar */}
      <StatusBar
        statusKind={fileManager.statusKind()}
        statusMessage={fileManager.statusMessage()}
        discoveredCount={fileManager.discoveredFiles().length}
        totalSize={fileManager.totalSize()}
        selectedCount={fileManager.selectedCount()}
        systemStats={fileManager.systemStats()}
        progressItems={activityProgressItems()}
        autoSaveStatus={autoSaveStatus()}
        autoSaveEnabled={projectManager.autoSaveEnabled()}
        lastAutoSave={projectManager.lastAutoSave()}
        activityCount={projectManager.project()?.activity_log?.length ?? 0}
        bookmarkCount={projectManager.bookmarkCount()}
        noteCount={projectManager.noteCount()}
        onAutoSaveToggle={() => {
          if (projectManager.modified()) {
            handleSaveProject();
          } else {
            projectManager.setAutoSaveEnabled(!projectManager.autoSaveEnabled());
          }
        }}
      />
      
      {/* Progress Modal */}
      <ProgressModal
        show={fileManager.loadProgress().show}
        title={fileManager.loadProgress().title}
        message={fileManager.loadProgress().message}
        current={fileManager.loadProgress().current}
        total={fileManager.loadProgress().total}
        onCancel={fileManager.cancelLoading}
      />
      
      {/* Report Wizard Modal */}
      <Show when={showReportWizard()}>
        <Suspense>
          <ReportWizard
            files={fileManager.discoveredFiles()}
            fileInfoMap={fileManager.fileInfoMap()}
            fileHashMap={hashManager.fileHashMap()}
            activityLog={projectManager.project()?.activity_log}
            sessions={projectManager.project()?.sessions}
            projectName={projectManager.projectName() || undefined}
            projectDescription={projectManager.project()?.description}
            caseDocumentsCache={projectManager.project()?.case_documents_cache?.documents}
            bookmarks={projectManager.project()?.bookmarks}
            notes={projectManager.project()?.notes}
            initialReportType={initialReportType()}
            onClose={() => { setShowReportWizard(false); setInitialReportType(undefined); }}
            onGenerated={(path: string, format: string) => {
            log.info(`Report generated: ${path} (${format})`);
            fileManager.setOk(`Report saved to ${path}`);
            projectManager.logActivity(
              'export',
              'report',
              `Report generated: ${path.split('/').pop() || path} (${format})`,
              path,
              { format },
            );
          }}
        />
        </Suspense>
      </Show>

      {/* Evidence Collection Modal (standalone) */}
      <Show when={showEvidenceCollection()}>
        <Suspense>
          <EvidenceCollectionModal
            caseNumber={projectManager.project()?.case_info?.case_number || undefined}
            projectName={projectManager.projectName() || undefined}
            onClose={() => setShowEvidenceCollection(false)}
          />
        </Suspense>
      </Show>
    </div>
  );
}

export default App;
