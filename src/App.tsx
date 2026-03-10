// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, on, Show, lazy, Suspense } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useFileManager, useHashManager, useDatabase, useProject, useProcessedDatabases, useHistoryContext, usePreferenceEffects, useKeyboardHandler, createSearchHandlers, createContextMenuBuilders, createCommandPaletteActions, useAppState, useDatabaseEffects, useCenterPaneTabs, useActivityManager, useEntryNavigation, useActivityLogging, useProjectActions, useMenuActions, useLoadingState, type DetailViewType } from "./hooks";
import { useAppLifecycle } from "./hooks/useAppLifecycle";
import { useAppHandlers } from "./hooks/useAppHandlers";
import { useDualPanelResize } from "./hooks/usePanelResize";
import { Toolbar, StatusBar, DetailPanel, ProgressModal, ContainerEntryViewer, useToast, pathToBreadcrumbs, createContextMenu, useTour, DEFAULT_TOUR_STEPS, useDragDrop, Sidebar, AppModals, RightPanel, CenterPane, LeftPanelContent, ExportPanel } from "./components";
import { LoadingOverlay } from "./components/ui";
import { AppSecondaryModals } from "./components/layout/AppSecondaryModals";
import { HelpPanel } from "./components/HelpPanel";
import { QuickActionsBar } from "./components/QuickActionsBar";
import { AppHeader } from "./components/layout/AppHeader";
import { useWorkspaceProfiles } from "./hooks/useWorkspaceProfiles";
import type { DiscoveredFile } from "./types";
import { createPreferences, getPreference, getRecentProjects } from "./components/preferences";
import { createThemeActions } from "./hooks/useTheme";
import { announce } from "./utils/accessibility";
import { logger } from "./utils/logger";
import { getBasename, getDirname } from "./utils/pathUtils";
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
const ProcessedDetailPanel = lazy(() => import("./components/ProcessedDetailPanel").then(m => ({ default: m.ProcessedDetailPanel })));
const EvidenceCollectionPanel = lazy(() => import("./components/EvidenceCollectionPanel").then(m => ({ default: m.EvidenceCollectionPanel })));
const EvidenceCollectionListPanel = lazy(() => import("./components/EvidenceCollectionListPanel").then(m => ({ default: m.EvidenceCollectionListPanel })));

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
  const globalLoading = useLoadingState();
  
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
          showUpdateModal, setShowUpdateModal, showMergeWizard, setShowMergeWizard,
          showRecoveryModal, setShowRecoveryModal,
          showUserConfirmModal, setShowUserConfirmModal,
          userConfirmAction, setUserConfirmAction,
          userConfirmProjectName, setUserConfirmProjectName } = modals;
  
  const { openTabs, setOpenTabs, currentViewMode, setCurrentViewMode, hexMetadata, setHexMetadata,
          selectedContainerEntry, setSelectedContainerEntry, entryContentViewMode, setEntryContentViewMode,
          requestViewMode, setRequestViewMode, hexNavigator, setHexNavigator, 
          treeExpansionState, setTreeExpansionState } = views;
  
  const { pendingProjectRoot, setPendingProjectRoot, caseDocumentsPath, setCaseDocumentsPath,
          caseDocuments, setCaseDocuments } = project;
  
  const { leftPanelTab, setLeftPanelTab, leftPanelMode, setLeftPanelMode } = leftPanel;
  
  // Viewer metadata for right panel (emitted by ContainerEntryViewer)
  const [viewerMetadata, setViewerMetadata] = createSignal<import("./types/viewerMetadata").ViewerMetadata | null>(null);
  
  // Linked data nodes for right panel (emitted by EvidenceCollectionPanel)
  const [linkedDataNodes, setLinkedDataNodes] = createSignal<import("./components/LinkedDataTree").LinkedDataNode[]>([]);
  
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
    const activityItems = active.map(activity => ({
      id: activity.id,
      label: `${activity.type === "archive" ? "Archive" : activity.type === "export" ? "Export" : "Copy"}: ${activity.progress?.currentFile ? getBasename(activity.progress.currentFile) : "preparing..."}`,
      progress: activity.progress?.percent ?? 0,
      indeterminate: activity.status === "pending",
      onClick: () => setRequestViewMode("export"),
    }));
    
    // Hash batch progress items
    const batches = hashManager.activeBatches();
    const hashItems = batches.filter(b => !b.done).map(batch => ({
      id: batch.id,
      label: `# Hash ${batch.completedFiles}/${batch.totalFiles}${batch.paused ? " ⏸" : ""}`,
      progress: batch.percent,
      indeterminate: batch.percent === 0 && !batch.paused,
      isPausable: true,
      isPaused: batch.paused,
      onCancel: batch.paused
        ? () => hashManager.resumeHashQueue()
        : () => hashManager.pauseHashQueue(),
      cancelTitle: batch.paused ? "Resume hashing" : "Pause hashing",
    }));
    
    return [...hashItems, ...activityItems];
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
    if (projectManager.modified()) return "modified";
    if (projectManager.lastAutoSave()) return "saved";
    return "idle";
  });
  
  // Recent projects for welcome modal (convert to RecentProjectInfo format)
  // Coerce null → undefined for AppHeader's expected type
  const headerProjectName = createMemo(() => projectManager.projectName() ?? undefined);

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
    if (tabType !== "collection") {
      setLinkedDataNodes([]);
    }
  });
  
  // Auto-verify hashes when a file becomes active (if preference enabled)
  const autoVerifiedFiles = new Set<string>();
  let autoHashWarningShown = false;
  createEffect(on(
    () => fileManager.activeFile(),
    (active) => {
      if (!active || !getPreference("autoVerifyHashes")) return;
      
      // Show a one-time warning per session that auto-hashing is on
      if (!autoHashWarningShown) {
        autoHashWarningShown = true;
        toast.warning(
          "Auto-hash enabled",
          "Files are hashed automatically on selection. This can slow down evidence viewing. Disable in Settings \u2192 Behavior."
        );
      }
      
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
          const dirPath = getDirname(firstPath);
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
  const { getSaveOptions, handleSaveProject: _handleSaveProject, handleSaveProjectAs: _handleSaveProjectAs, handleLoadProject: _handleLoadProject, handleOpenDirectory, handleProjectSetupComplete: _handleProjectSetupComplete } = projectActions;
  
  // Loading-wrapped versions of slow project operations
  const handleLoadProject = (path?: string) =>
    globalLoading.run("Loading project…", () => _handleLoadProject(path));
  const handleSaveProject = () =>
    globalLoading.run("Saving project…", () => _handleSaveProject());
  const handleSaveProjectAs = () =>
    globalLoading.run("Saving project…", () => _handleSaveProjectAs());
  const handleProjectSetupComplete = (locations: import("./components").ProjectLocations) =>
    globalLoading.run("Setting up project…", () => _handleProjectSetupComplete(locations));
  const handleScanEvidence = () =>
    globalLoading.run("Scanning for evidence…", () => fileManager.scanForFiles());
  
  // ===========================================================================
  // App Handlers — location selection & quick actions (extracted to useAppHandlers)
  // ===========================================================================
  
  const { handleLocationSelect, handleQuickAction } = useAppHandlers({
    processedDbManager,
    fileManager,
    hashManager,
    projectManager,
    centerPaneTabs,
    toast,
    setLeftPanelTab,
    setLeftCollapsed,
    handleScanEvidence,
    setShowSearchPanel,
    setShowReportWizard,
    setShowSettingsPanel,
    setShowCommandPalette,
  });
  
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
    hasProject: () => !!projectManager.hasProject(),
    onOpenEvidenceCollection: () => centerPaneTabs.openEvidenceCollection(),
    onOpenEvidenceCollectionList: () => centerPaneTabs.openEvidenceCollectionList(),
    onOpenDirectory: handleOpenDirectory,
    onOpenProject: () => handleLoadProject(),
    onOpenHelp: () => centerPaneTabs.openHelpTab(),
    onOpenExport: () => centerPaneTabs.openExportTab(),
    onToggleQuickActions: () => setShowQuickActions((prev) => !prev),
    onCycleTheme: () => themeActions.cycleTheme(),
    onShowDashboard: () => { setLeftCollapsed(false); setLeftPanelTab("dashboard"); },
    onShowEvidence: () => { setLeftCollapsed(false); setLeftPanelTab("evidence"); },
    onShowProcessed: () => { setLeftCollapsed(false); setLeftPanelTab("processed"); },
    onShowCaseDocs: () => { setLeftCollapsed(false); setLeftPanelTab("casedocs"); },
    onShowActivity: () => { setLeftCollapsed(false); setLeftPanelTab("activity"); },
    onShowBookmarks: () => { setLeftCollapsed(false); setLeftPanelTab("bookmarks"); },
    onCloseActiveTab: () => {
      const tabId = centerPaneTabs.activeTabId();
      if (tabId) centerPaneTabs.closeTab(tabId);
    },
    onCloseAllTabs: () => centerPaneTabs.closeAllTabs(),
    onDeduplication: () => toast.info("Deduplication", "Feature coming soon"),
    onShowPerformance: () => setShowPerformancePanel(true),
    setShowMergeWizard,
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
    menuActions: {
      onOpenProject: () => handleLoadProject(),
      onOpenDirectory: handleOpenDirectory,
      onSave: handleSaveProject,
      onSaveAs: handleSaveProjectAs,
      onCommandPalette: () => setShowCommandPalette(true),
    },
  });

  // ===========================================================================
  // Native Menu Actions — handles events from macOS/Windows menu bar
  // ===========================================================================
  useMenuActions({
    onOpenProject: () => handleLoadProject(),
    onOpenDirectory: handleOpenDirectory,
    onSaveProject: handleSaveProject,
    onSaveProjectAs: handleSaveProjectAs,
    onToggleSidebar: () => setLeftCollapsed((prev) => !prev),
    onToggleRightPanel: () => setRightCollapsed((prev) => !prev),
    onKeyboardShortcuts: () => setShowShortcutsModal(true),
    onCommandPalette: () => setShowCommandPalette(true),
    onNewProject: () => setShowProjectWizard(true),
    onExport: () => { if (projectManager.hasProject()) centerPaneTabs.openExportTab(); },
    onGenerateReport: () => { if (projectManager.hasProject()) setShowReportWizard(true); },
    onScanEvidence: () => handleScanEvidence(),
    onToggleQuickActions: () => setShowQuickActions((prev) => !prev),
    onShowEvidence: () => { setLeftCollapsed(false); setLeftPanelTab("evidence"); },
    onShowCaseDocs: () => { setLeftCollapsed(false); setLeftPanelTab("casedocs"); },
    onShowProcessed: () => { setLeftCollapsed(false); setLeftPanelTab("processed"); },
    onEvidenceCollection: () => { if (projectManager.hasProject()) centerPaneTabs.openEvidenceCollection(); },
    onSearchEvidence: () => setShowSearchPanel(true),
    onSettings: () => setShowSettingsPanel(true),
    onCloseAllTabs: () => centerPaneTabs.closeAllTabs(),
    onHashAll: () => hashManager.hashAllFiles(),
    onEvidenceCollectionList: () => { if (projectManager.hasProject()) centerPaneTabs.openEvidenceCollectionList(); },
    onUserGuide: () => centerPaneTabs.openHelpTab(),
    onWelcomeScreen: () => setShowWelcomeModal(true),
    onCloseActiveTab: () => {
      const tabId = centerPaneTabs.activeTabId();
      if (tabId) centerPaneTabs.closeTab(tabId);
    },
    onToggleAutoSave: () => {
      const current = projectManager.autoSaveEnabled();
      projectManager.setAutoSaveEnabled(!current);
      toast.info(current ? "Auto-save disabled" : "Auto-save enabled");
    },
    onHashSelected: () => hashManager.hashSelectedFiles(),
    onHashActive: () => {
      const active = fileManager.activeFile();
      if (active) hashManager.hashSingleFile(active);
    },
    onStartTour: () => tour.start(),
    onShowDashboard: () => { setLeftCollapsed(false); setLeftPanelTab("dashboard"); },
    onShowActivity: () => { setLeftCollapsed(false); setLeftPanelTab("activity"); },
    onShowBookmarks: () => { setLeftCollapsed(false); setLeftPanelTab("bookmarks"); },
    onViewInfo: () => setCurrentViewMode("info"),
    onViewHex: () => setCurrentViewMode("hex"),
    onViewText: () => setCurrentViewMode("text"),
    onCycleTheme: () => themeActions.cycleTheme(),
    onSelectAllEvidence: () => fileManager.toggleSelectAll(),
    onDeduplication: () => toast.info("Deduplication", "Feature coming soon"),
    onLoadAllInfo: () => fileManager.loadAllInfo(),
    onCleanCache: async () => {
      try {
        await invoke("cleanup_preview_cache");
        toast.success("Cache cleaned", "Preview cache cleared successfully");
      } catch (err) {
        toast.error("Failed to clean cache", String(err));
      }
    },
    onCheckForUpdates: () => setShowUpdateModal(true),
    onMergeProjects: () => setShowMergeWizard(true),
    onProjectRecovery: () => { if (projectManager.hasProject()) setShowRecoveryModal(true); },
  });

  // Sync native menu enabled state with project lifecycle
  createEffect(on(
    () => !!projectManager.hasProject(),
    (hasProject) => {
      invoke("set_project_menu_state", { hasProject }).catch(() => {});
    }
  ));

  // Show user confirmation modal when a project is opened or created
  createEffect(on(
    () => !!projectManager.hasProject(),
    (hasProject, prevHasProject) => {
      // Only trigger on the transition false → true
      if (hasProject && !prevHasProject) {
        const shouldConfirm = getPreference("confirmUserOnProjectOpen");
        const profiles = preferences.preferences().userProfiles || [];
        if (shouldConfirm && profiles.length > 0) {
          setUserConfirmAction("open");
          setUserConfirmProjectName(projectManager.projectName() || "");
          setShowUserConfirmModal(true);
        }
      }
    }
  ));

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
      
      {/* Secondary Modals — UserConfirm, Report, Update, Merge, Recovery */}
      <AppSecondaryModals
        fileManager={fileManager}
        hashManager={hashManager}
        projectManager={projectManager}
        showUserConfirmModal={showUserConfirmModal}
        setShowUserConfirmModal={setShowUserConfirmModal}
        userConfirmAction={userConfirmAction}
        userConfirmProjectName={userConfirmProjectName}
        userProfiles={preferences.preferences().userProfiles || []}
        defaultUserProfileId={preferences.preferences().defaultUserProfileId || ""}
        onUpdatePreference={(key, value) => preferences.updatePreference(key, value)}
        setShowSettingsPanel={setShowSettingsPanel}
        showReportWizard={showReportWizard}
        setShowReportWizard={setShowReportWizard}
        initialReportType={initialReportType}
        setInitialReportType={setInitialReportType}
        showUpdateModal={showUpdateModal}
        setShowUpdateModal={setShowUpdateModal}
        showMergeWizard={showMergeWizard}
        setShowMergeWizard={setShowMergeWizard}
        onLoadProject={handleLoadProject}
        showRecoveryModal={showRecoveryModal}
        setShowRecoveryModal={setShowRecoveryModal}
      />
      
      {/* Header / Title Bar */}
      <AppHeader
        projectName={headerProjectName}
        projectModified={projectManager.modified}
        leftCollapsed={leftCollapsed}
        setLeftCollapsed={setLeftCollapsed}
        rightCollapsed={rightCollapsed}
        setRightCollapsed={setRightCollapsed}
        showQuickActions={showQuickActions}
        setShowQuickActions={setShowQuickActions}
      />

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
        onScan={() => handleScanEvidence()}
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
              const dir = getDirname(firstPath);
              return dir || firstPath;
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
          onAction={handleQuickAction}
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
              hasProject={() => !!projectManager.hasProject()}
              bookmarkCount={projectManager.bookmarkCount}
              onExport={() => centerPaneTabs.openExportTab()}
              onReport={() => { if (projectManager.hasProject()) { setInitialReportType(undefined); setShowReportWizard(true); } }}
              onReportType={(type: string) => { if (projectManager.hasProject()) { setInitialReportType(type as import("./components/report/types").ReportType); setShowReportWizard(true); } }}
              onExportSelected={() => centerPaneTabs.openExportTab()}
              onClearBookmarks={() => {
                const count = projectManager.bookmarkCount();
                if (count === 0) return;
                if (confirm(`Remove all ${count} bookmarks? This cannot be undone.`)) {
                  projectManager.clearBookmarks();
                  toast.success("Bookmarks Cleared", `Removed ${count} bookmarks`);
                }
              }}
              onExportBookmarks={async () => {
                const proj = projectManager.project();
                if (!proj?.bookmarks?.length) return;
                try {
                  const { save } = await import("@tauri-apps/plugin-dialog");
                  const path = await save({
                    title: "Export Bookmarks",
                    defaultPath: `bookmarks.json`,
                    filters: [{ name: "JSON", extensions: ["json"] }],
                  });
                  if (!path) return;
                  const content = JSON.stringify(proj.bookmarks, null, 2);
                  await invoke("write_text_file", { path, content });
                  toast.success("Bookmarks Exported", `${proj.bookmarks.length} bookmarks saved`);
                } catch (err) {
                  toast.error("Export Failed", err instanceof Error ? err.message : String(err));
                }
              }}
              onSearch={() => setShowSearchPanel(true)}
              onSettings={() => setShowSettingsPanel(true)}
              onCommandPalette={() => setShowCommandPalette(true)}
              onHelp={() => setShowShortcutsModal(true)}
              onEvidenceCollection={() => {
                if (projectManager.hasProject()) centerPaneTabs.openEvidenceCollection();
              }}
              onEvidenceCollectionList={() => { if (projectManager.hasProject()) centerPaneTabs.openEvidenceCollectionList(); }}
              onScanEvidence={() => handleScanEvidence()}
              onSelectAllEvidence={() => fileManager.toggleSelectAll()}
              onLoadAllInfo={() => fileManager.loadAllInfo()}
              onRefreshProcessed={() => { /* Refresh processed databases */ }}
              onRefreshCaseDocs={() => { /* Refresh case documents */ }}
              onToggleSidebar={() => setLeftCollapsed((prev) => !prev)}
              onToggleRightPanel={() => setRightCollapsed((prev) => !prev)}
              onToggleQuickActions={() => setShowQuickActions((prev) => !prev)}
              onOpenHelpTab={() => centerPaneTabs.openHelpTab()}
              onShowPerformance={() => setShowPerformancePanel(true)}
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
              onNavigateTab={(tab) => setLeftPanelTab(tab as import("./components/layout/sidebar/types").LeftPanelTab)}
              onExport={() => centerPaneTabs.openExportTab()}
              onReport={() => { if (projectManager.hasProject()) { setInitialReportType(undefined); setShowReportWizard(true); } }}
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
                    <Suspense fallback={<div class="flex items-center justify-center h-full text-txt-muted text-sm">Loading database viewer…</div>}>
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
                      initialExaminerName={projectManager.project()?.owner_name || projectManager.project()?.current_user || undefined}
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
                  
                  {/* Evidence collection tabs */}
                  <Show when={tab().type === "collection"}>
                    <Suspense fallback={<div class="flex items-center justify-center h-full text-txt-muted text-sm">Loading evidence collection…</div>}>
                      <Show when={tab().collectionListView} fallback={
                        <EvidenceCollectionPanel
                          caseNumber={projectManager.caseNumber() || undefined}
                          projectName={projectManager.projectName() || undefined}
                          examinerName={projectManager.project()?.owner_name || projectManager.project()?.current_user || undefined}
                          collectionId={tab().collectionId}
                          readOnly={tab().collectionReadOnly}
                          discoveredFiles={fileManager.discoveredFiles()}
                          fileInfoMap={fileManager.fileInfoMap()}
                          onClose={() => centerPaneTabs.closeTab(tab().id)}
                          onOpenCollection={(id, ro) => centerPaneTabs.openEvidenceCollection(id, ro)}
                          onLinkedNodesChange={setLinkedDataNodes}
                        />
                      }>
                        <EvidenceCollectionListPanel
                          projectName={projectManager.projectName() || undefined}
                          onOpenCollection={(id, ro) => centerPaneTabs.openEvidenceCollection(id, ro)}
                          onNewCollection={() => centerPaneTabs.openEvidenceCollection()}
                          onExport={async (id, format) => {
                            const { exportEvidenceCollection } = await import("./components/report/wizard/cocDbSync");
                            const path = await exportEvidenceCollection(id, format, projectManager.caseNumber() || undefined);
                            if (path) {
                              toast.success(`${format.toUpperCase()} Exported`, `Saved to ${getBasename(path)}`);
                            }
                          }}
                        />
                      </Show>
                    </Suspense>
                  </Show>
                  
                  {/* Help & documentation tab */}
                  <Show when={tab().type === "help"}>
                    <HelpPanel />
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
          linkedDataNodes={linkedDataNodes}
          hasProject={() => !!projectManager.hasProject()}
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
        onEvidenceClick={() => { setLeftCollapsed(false); setLeftPanelTab("evidence"); }}
        onBookmarkClick={() => { setLeftCollapsed(false); setLeftPanelTab("bookmarks"); }}
        onActivityClick={() => { setLeftCollapsed(false); setLeftPanelTab("activity"); }}
        onPerformanceClick={() => setShowPerformancePanel(true)}
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

      {/* Global Loading Indicator */}
      <LoadingOverlay
        isLoading={globalLoading.isLoading}
        message={globalLoading.message}
        error={globalLoading.error}
        position="bottom-right"
      />
    </div>
  );
}

export default App;
