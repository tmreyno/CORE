// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { onMount, onCleanup, createSignal, createEffect, createMemo, on, Show, lazy } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { useFileManager, useHashManager, useDatabase, useProject, useProcessedDatabases, useHistoryContext, usePreferenceEffects, useTransferEvents, useKeyboardHandler, createSearchHandlers, createContextMenuBuilders, createCommandPaletteActions, useAppState, useDatabaseEffects, buildSaveOptions, handleLoadProject as loadProjectHandler, handleOpenDirectory as openDirectoryHandler, handleProjectSetupComplete as projectSetupHandler, useCenterPaneTabs } from "./hooks";
import { useDualPanelResize } from "./hooks/usePanelResize";
import { Toolbar, StatusBar, DetailPanel, ProgressModal, EvidenceTree, ContainerEntryViewer, useToast, pathToBreadcrumbs, createContextMenu, useTour, DEFAULT_TOUR_STEPS, useDragDrop, CaseDocumentsPanel, Sidebar, AppModals, RightPanel, CenterPane, CollapsiblePanelContent } from "./components";
import { ActivityPanel } from "./components/ActivityPanel";
import type { ProjectLocations, SelectedEntry } from "./components";
import type { DiscoveredFile, CaseDocument } from "./types";
import { createPreferences, getPreference } from "./components/preferences";
import { createThemeActions } from "./hooks/useTheme";
import { announce } from "./utils/accessibility";
import ffxLogo from "./assets/branding/core-logo-48.png";
import "./App.css";

// ============================================================================
// Lazy-loaded Components (Code Splitting)
// These are heavy components that aren't needed on initial render
// ============================================================================
const ReportWizard = lazy(() => import("./components/report/wizard/ReportWizard").then(m => ({ default: m.ReportWizard })));
const ProcessedDatabasePanel = lazy(() => import("./components/ProcessedDatabasePanel"));
const ProcessedDetailPanel = lazy(() => import("./components/ProcessedDetailPanel"));

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
    right: { initialWidth: 280, minWidth: 150, maxWidth: 500, startCollapsed: true },
  });
  // Panel aliases for cleaner template usage
  const { width: leftWidth, collapsed: leftCollapsed, setWidth: setLeftWidth, setCollapsed: setLeftCollapsed } = panels.left;
  const { width: rightWidth, collapsed: rightCollapsed, setWidth: setRightWidth, setCollapsed: setRightCollapsed } = panels.right;
  
  // ===========================================================================
  // UI State - consolidated from useAppState hook
  // ===========================================================================
  const appState = useAppState();
  const { modals, views, project, transfer, leftPanel, centerPanel } = appState;
  
  // Destructure for easier access
  const { showCommandPalette, setShowCommandPalette, showShortcutsModal, setShowShortcutsModal, 
          showPerformancePanel, setShowPerformancePanel, showSettingsPanel, setShowSettingsPanel,
          showSearchPanel, setShowSearchPanel, showWelcomeModal, setShowWelcomeModal,
          showReportWizard, setShowReportWizard, showProjectWizard, setShowProjectWizard } = modals;
  
  const { openTabs, setOpenTabs, currentViewMode, setCurrentViewMode, hexMetadata, setHexMetadata,
          selectedContainerEntry, setSelectedContainerEntry, entryContentViewMode, setEntryContentViewMode,
          requestViewMode, setRequestViewMode, hexNavigator, setHexNavigator, 
          treeExpansionState, setTreeExpansionState } = views;
  
  const { pendingProjectRoot, setPendingProjectRoot, caseDocumentsPath, setCaseDocumentsPath,
          caseDocuments, setCaseDocuments } = project;
  
  const { transferJobs, setTransferJobs } = transfer;
  const { leftPanelTab, setLeftPanelTab, leftPanelMode, setLeftPanelMode } = leftPanel;
  // Note: Old centerPanel state is deprecated, using unified centerPaneTabs instead
  void centerPanel; // Suppress warning for now - will remove centerPanel from useAppState later
  
  // ===========================================================================
  // Unified Center Pane Tabs - new unified tab management
  // ===========================================================================
  const centerPaneTabs = useCenterPaneTabs();
  
  // Window size tracking (not in useAppState as it's window-specific)
  const [windowWidth, setWindowWidth] = createSignal(window.innerWidth);
  const isCompact = () => windowWidth() < 900;
  
  // Transfer events listener
  useTransferEvents(setTransferJobs);
  
  // ===========================================================================
  // Derived State & Computed Values
  // ===========================================================================
  const transferProgressItems = (): import("./components").ProgressItem[] => {
    const jobs = transferJobs().filter(j => j.status === "running" || j.status === "pending");
    return jobs.map(job => ({
      id: job.id,
      label: `Export: ${job.progress?.current_file?.split("/").pop() || "preparing..."}`,
      progress: job.progress?.overall_percent ?? 0,
      indeterminate: job.status === "pending",
      onClick: () => setRequestViewMode("export"),
    }));
  };
  
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
  
  // Auto-verify hashes when a file becomes active (if preference enabled)
  const autoVerifiedFiles = new Set<string>();
  createEffect(on(
    () => fileManager.activeFile(),
    (active) => {
      if (!active || !getPreference("autoVerifyHashes")) return;
      
      // Only auto-verify once per file per session
      if (autoVerifiedFiles.has(active.path)) return;
      
      autoVerifiedFiles.add(active.path);
      console.log("[autoVerifyHashes] Auto-verifying:", active.path);
      hashManager.verifySegments(active);
    },
    { defer: true }
  ));
  
  // Sync export request to currentViewMode
  createEffect(() => {
    const requested = requestViewMode();
    if (requested === "export") {
      setCurrentViewMode("export");
      setRequestViewMode(null);
    }
  });
  
  // ===========================================================================
  // Handler Functions
  // ===========================================================================
  
  const handleHexNavigatorReady = (nav: (offset: number, size?: number) => void) => {
    setHexNavigator(() => nav);
  };
  
  // Search handlers from useAppActions
  const { handleSearch, handleSearchResultSelect } = createSearchHandlers({ fileManager });

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
          // Set scan directory to the parent of the first dropped file
          const firstPath = paths[0];
          const dirPath = firstPath.substring(0, firstPath.lastIndexOf("/"));
          if (dirPath) {
            fileManager.setScanDir(dirPath);
            await fileManager.scanForFiles();
            toast.success(`Loaded ${paths.length} file(s) from dropped location`);
            announce(`Loaded ${paths.length} files`);
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
  // Helper Functions - Using extracted helpers from hooks/project
  // =========================================================================
  
  /** Build save options using the extracted helper */
  const getSaveOptions = () => buildSaveOptions({
    fileManager,
    hashManager,
    processedDbManager,
    // New center pane tabs system
    centerTabs: centerPaneTabs.tabs,
    activeTabId: centerPaneTabs.activeTabId,
    viewMode: centerPaneTabs.viewMode,
    // Legacy tabs (deprecated but kept for backwards compatibility)
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
  });
  
  /** Handle loading a project using the extracted helper */
  const handleLoadProject = async () => {
    await loadProjectHandler({
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
      // New center pane tabs system
      setCenterTabs: centerPaneTabs.setTabs,
      setActiveTabId: centerPaneTabs.setActiveTabId,
      setCenterViewMode: centerPaneTabs.setViewMode,
      toast,
    });
  };
  
  /** Handle saving the project */
  const handleSaveProject = async () => {
    const options = getSaveOptions();
    if (options) {
      try {
        const result = await projectManager.saveProject(options);
        if (result.success) {
          toast.success("Project Saved", "Your project has been saved");
        } else if (result.error && result.error !== "Save cancelled") {
          toast.error("Save Failed", result.error);
        }
      } catch (err) {
        toast.error("Save Failed", "Could not save the project");
      }
    } else {
      toast.error("No Evidence", "Open an evidence directory first");
    }
  };
  
  /** Handle selecting a container entry - opens in center pane tab */
  const handleSelectEntry = (entry: SelectedEntry) => {
    // Set the entry for legacy views
    setSelectedContainerEntry(entry);
    
    // Open in unified center pane tab
    centerPaneTabs.openContainerEntry(entry);
    
    // Determine best view mode based on file type
    const ext = entry.name.toLowerCase().split('.').pop() || '';
    const previewableExtensions = [
      'pdf', 'docx', 'doc', 'txt', 'md', 'html', 'htm', 'rtf',
      'xlsx', 'xls', 'csv', 'ods',
      'jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg',
    ];
    
    if (previewableExtensions.includes(ext)) {
      setEntryContentViewMode("document");
    } else {
      setEntryContentViewMode("hex");
    }
    
    console.log('Selected entry:', entry.entryPath, 'from', entry.containerPath);
  };
  
  /** Handle selecting an evidence file - opens in center pane tab */
  const handleSelectEvidenceFile = (file: DiscoveredFile) => {
    fileManager.selectAndViewFile(file);
    centerPaneTabs.openEvidenceFile(file);
  };
  
  /** Handle opening a nested container */
  const handleOpenNestedContainer = (tempPath: string, originalName: string, containerType: string, parentPath: string) => {
    const nestedFile: DiscoveredFile = {
      path: tempPath,
      filename: `📦 ${originalName} (from ${parentPath.split('/').pop() || parentPath})`,
      container_type: containerType.toUpperCase(),
      size: 0,
      segment_count: 1,
    };
    fileManager.addDiscoveredFile(nestedFile);
    handleSelectEvidenceFile(nestedFile);
    toast.success("Nested Container", `Opened ${originalName}`);
  };
  
  /** Handle selecting a processed database */
  const handleSelectProcessedDb = (db: Parameters<typeof processedDbManager.selectDatabase>[0]) => {
    processedDbManager.selectDatabase(db);
    fileManager.setActiveFile(null);
    // Open in unified center pane
    if (db) {
      centerPaneTabs.openProcessedDatabase(db);
    }
  };
  
  /** Handle selecting a case document - opens in a document tab */
  const handleCaseDocumentSelect = (doc: CaseDocument) => {
    // Use the unified center pane tabs API - entry is stored in the tab
    centerPaneTabs.openCaseDocument(doc);
    
    const ext = doc.path.toLowerCase().split('.').pop() || '';
    
    // Check for previewable document types
    const previewableExtensions = [
      // Documents
      'pdf', 'docx', 'doc', 'txt', 'md', 'html', 'htm', 'rtf',
      // Spreadsheets
      'xlsx', 'xls', 'csv', 'ods',
      // Images
      'jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg',
    ];
    
    if (previewableExtensions.includes(ext)) {
      if (ext === 'pdf') {
        // PDFs use the dedicated PDF viewer through file manager
        const pdfFile: DiscoveredFile = {
          path: doc.path,
          filename: doc.filename,
          container_type: 'pdf',
          size: doc.size,
          created: doc.modified ?? undefined,
          modified: doc.modified ?? undefined,
        };
        fileManager.setActiveFile(pdfFile);
        setRequestViewMode("pdf");
      } else {
        // Other documents use the ContainerEntryViewer in document mode
        setEntryContentViewMode("document");
      }
    } else {
      // Default to hex for unknown/binary files
      setEntryContentViewMode("hex");
    }
  };
  
  /** Handle viewing case document as hex */
  const handleCaseDocViewHex = (doc: CaseDocument) => {
    centerPaneTabs.openCaseDocument(doc);
    setEntryContentViewMode("hex");
  };
  
  /** Handle viewing case document as text */
  const handleCaseDocViewText = (doc: CaseDocument) => {
    centerPaneTabs.openCaseDocument(doc);
    setEntryContentViewMode("text");
  };
  
  // Context menu builders from useAppActions
  const { getFileContextMenuItems, getSaveContextMenuItems } = createContextMenuBuilders({
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
  });

  // Database synchronization effects
  useDatabaseEffects({ db, fileManager });

  // ===========================================================================
  // Lifecycle - Mount & Cleanup
  // ===========================================================================
  
  let cleanupSystemStats: (() => void) | undefined;
  const handleResize = () => setWindowWidth(window.innerWidth);

  onMount(async () => {
    // System stats listener
    const unlisten = await fileManager.setupSystemStatsListener();
    cleanupSystemStats = unlisten;
    
    // Window resize handling - makeEventListener auto-cleans up
    makeEventListener(window, 'resize', handleResize);
    
    // Auto-save callback
    projectManager.setAutoSaveCallback(async () => {
      const options = getSaveOptions();
      if (options) await projectManager.saveProject(options);
    });
    
    // Welcome modal for first-time users
    const hasSeenWelcome = localStorage.getItem("ffx-welcome-seen");
    if (!hasSeenWelcome && !tour.hasCompleted()) {
      setTimeout(() => setShowWelcomeModal(true), 500);
    }
    
    // Restore last session (non-blocking)
    db.restoreLastSession()
      .then((lastSession) => {
        if (lastSession) {
          fileManager.setScanDir(lastSession.root_path);
          console.log(`Restored session: ${lastSession.name} (${lastSession.root_path})`);
        }
      })
      .catch((e) => console.warn("Failed to restore last session:", e));
  });

  onCleanup(() => {
    cleanupSystemStats?.();
    projectManager.stopAutoSave();
    
    // Clear clipboard on close if preference is set (security feature)
    if (preferences.preferences().clearClipboardOnClose) {
      navigator.clipboard.writeText("").catch(() => {});
    }
  });

  // Wrapper for opening a project directory - uses extracted handler
  const handleOpenDirectory = () => openDirectoryHandler({
    setPendingProjectRoot,
    setShowProjectWizard,
    toast,
  });

  // Wrapper for project setup completion - uses extracted handler
  const handleProjectSetupComplete = (locations: ProjectLocations) => projectSetupHandler(
    {
      fileManager,
      hashManager,
      processedDbManager,
      projectManager,
      setShowProjectWizard,
      setCaseDocumentsPath,
      setLeftPanelTab,
      setPendingProjectRoot,
      toast,
    },
    locations
  );

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
        tour={tour}
        showProjectWizard={showProjectWizard}
        setShowProjectWizard={setShowProjectWizard}
        pendingProjectRoot={pendingProjectRoot}
        setPendingProjectRoot={setPendingProjectRoot}
        onProjectSetupComplete={handleProjectSetupComplete}
      />
      
      {/* Header */}
      <header class="app-header">
        <div class="brand">
          <img src={ffxLogo} alt="CORE-FFX Logo" class="brand-logo" />
          <span class="brand-name">CORE</span>
          <span class="brand-tag">Forensic File Xplorer</span>
        </div>
        <div class="header-status">
          <span class={`status-dot ${fileManager.statusKind()}`} />
          <span class="status-text">{fileManager.statusMessage()}</span>
        </div>
      </header>

      {/* Toolbar */}
      <Toolbar
        scanDir={fileManager.scanDir()}
        onScanDirChange={(dir) => fileManager.setScanDir(dir)}
        recursiveScan={fileManager.recursiveScan()}
        onRecursiveScanChange={(recursive) => fileManager.setRecursiveScan(recursive)}
        selectedHashAlgorithm={hashManager.selectedHashAlgorithm()}
        onHashAlgorithmChange={(alg) => hashManager.setSelectedHashAlgorithm(alg)}
        selectedCount={fileManager.selectedCount()}
        discoveredCount={fileManager.discoveredFiles().length}
        busy={fileManager.busy()}
        onBrowse={handleOpenDirectory}
        onScan={() => fileManager.scanForFiles()}
        onHashSelected={() => hashManager.hashSelectedFiles()}
        onLoadAll={() => fileManager.loadAllInfo()}
        compact={isCompact()}
      />

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
              projectModified={projectManager.modified}
              transferJobs={transferJobs}
              onSave={handleSaveProject}
              onSaveContextMenu={(e) => {
                e.preventDefault();
                saveContextMenu.open(e, getSaveContextMenuItems());
              }}
              onLoad={handleLoadProject}
              onExport={() => setRequestViewMode("export")}
              onReport={() => setShowReportWizard(true)}
              onSearch={() => setShowSearchPanel(true)}
              onSettings={() => setShowSettingsPanel(true)}
              theme={themeActions.theme}
              resolvedTheme={themeActions.resolvedTheme}
              cycleTheme={themeActions.cycleTheme}
            />
            
            {/* Panel Content Area */}
            <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
            {/* Tab-based Content */}
            <Show when={leftPanelMode() === "tabs"}>
            <div class={`flex-1 flex flex-col min-w-0 overflow-hidden ${leftPanelTab() === "evidence" ? "" : "hidden"}`}>
              {/* Unified Evidence Tree with selection and hashing */}
              <EvidenceTree
                discoveredFiles={fileManager.discoveredFiles()}
                activeFile={fileManager.activeFile()}
                busy={fileManager.busy()}
                onSelectContainer={handleSelectEvidenceFile}
                onSelectEntry={handleSelectEntry}
                typeFilter={fileManager.typeFilter()}
                onToggleTypeFilter={(type) => fileManager.toggleTypeFilter(type)}
                onClearTypeFilter={() => fileManager.setTypeFilter(null)}
                containerStats={fileManager.containerStats()}
                onOpenNestedContainer={handleOpenNestedContainer}
                // Tree expansion state persistence
                initialExpansionState={treeExpansionState() || undefined}
                onExpansionStateChange={setTreeExpansionState}
                // Selection & Hashing props (merged from FilePanel)
                selectedFiles={fileManager.selectedFiles()}
                fileHashMap={hashManager.fileHashMap()}
                hashHistory={hashManager.hashHistory()}
                fileStatusMap={fileManager.fileStatusMap()}
                fileInfoMap={fileManager.fileInfoMap()}
                onToggleFileSelection={(path) => fileManager.toggleFileSelection(path)}
                onHashFile={(file) => hashManager.hashSingleFile(file)}
                onContextMenu={(file, e) => {
                  fileManager.setActiveFile(file);
                  fileContextMenu.open(e, getFileContextMenuItems(fileManager.activeFile));
                }}
                allFilesSelected={fileManager.allFilesSelected()}
                onToggleSelectAll={() => fileManager.toggleSelectAll()}
                totalSize={fileManager.totalSize()}
              />
            </div>

            {/* Processed Databases Tab */}
            <div class={`flex-1 flex flex-col min-w-0 overflow-hidden ${leftPanelTab() === "processed" ? "" : "hidden"}`}>
              <ProcessedDatabasePanel 
                manager={processedDbManager}
                onSelectDatabase={handleSelectProcessedDb}
                onSelectArtifact={(db, artifact) => console.log('Selected artifact:', artifact.name, 'from', db.path)}
              />
            </div>

            {/* Case Documents Tab */}
            <Show when={leftPanelTab() === "casedocs"}>
              <CaseDocumentsPanel 
                evidencePath={stableCaseDocsPath() ?? undefined}
                onDocumentSelect={handleCaseDocumentSelect}
                onViewHex={handleCaseDocViewHex}
                onViewText={handleCaseDocViewText}
                cachedDocuments={caseDocuments() ?? undefined}
                onDocumentsLoaded={(docs, _searchPath) => setCaseDocuments(docs)}
              />
            </Show>
            
            {/* Activity Panel */}
            <Show when={leftPanelTab() === "activity"}>
              <ActivityPanel project={projectManager.project()} />
            </Show>
            </Show>
            
            {/* Unified/Collapsible View */}
            <Show when={leftPanelMode() === "unified"}>
              <CollapsiblePanelContent
                discoveredFiles={fileManager.discoveredFiles}
                activeFile={fileManager.activeFile}
                busy={fileManager.busy}
                onSelectContainer={handleSelectEvidenceFile}
                onSelectEntry={handleSelectEntry}
                typeFilter={fileManager.typeFilter}
                onToggleTypeFilter={(type) => fileManager.toggleTypeFilter(type)}
                onClearTypeFilter={() => fileManager.setTypeFilter(null)}
                containerStats={fileManager.containerStats}
                onOpenNestedContainer={handleOpenNestedContainer}
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
                totalSize={fileManager.totalSize}
                processedDbManager={processedDbManager}
                onSelectProcessedDb={(db) => {
                  processedDbManager.selectDatabase(db);
                  fileManager.setActiveFile(null);
                }}
                setActiveFile={fileManager.setActiveFile}
                caseDocumentsPath={caseDocumentsPath}
                evidencePath={stableCaseDocsPath() ?? undefined}
                projectLocations={projectManager.projectLocations() ?? undefined}
                caseDocuments={caseDocuments}
                onCaseDocumentsLoaded={(docs, _searchPath) => setCaseDocuments(docs)}
                onDocumentSelect={handleCaseDocumentSelect}
                onViewHex={handleCaseDocViewHex}
                onViewText={handleCaseDocViewText}
                project={projectManager.project}
                toast={toast}
              />
            </Show>
            </div>
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
            onTabSelect={(tabId) => centerPaneTabs.setActiveTabId(tabId)}
            onTabClose={centerPaneTabs.closeTab}
            onTabsChange={centerPaneTabs.setTabs}
            viewMode={centerPaneTabs.viewMode}
            onViewModeChange={centerPaneTabs.setViewMode}
          >
            {/* Content based on active tab type and view mode */}
            <Show when={centerPaneTabs.activeTab()}>
              {(tab) => (
                <>
                  {/* Evidence file tabs - show DetailPanel (handles all view modes internally) */}
                  <Show when={tab().type === "evidence" && tab().file}>
                    <DetailPanel
                      activeFile={tab().file!}
                      fileInfoMap={fileManager.fileInfoMap}
                      fileStatusMap={fileManager.fileStatusMap}
                      fileHashMap={hashManager.fileHashMap}
                      hashHistory={hashManager.hashHistory}
                      segmentResults={hashManager.segmentResults}
                      tree={fileManager.tree()}
                      filteredTree={fileManager.filteredTree()}
                      treeFilter={fileManager.treeFilter()}
                      onTreeFilterChange={(filter: string) => fileManager.setTreeFilter(filter)}
                      selectedHashAlgorithm={hashManager.selectedHashAlgorithm()}
                      segmentVerifyProgress={hashManager.segmentVerifyProgress()}
                      storedHashesGetter={hashManager.getAllStoredHashesSorted}
                      busy={fileManager.busy()}
                      onVerifySegments={(file) => hashManager.verifySegments(file)}
                      onLoadInfo={(file) => fileManager.loadFileInfo(file, true)}
                      formatHashDate={hashManager.formatHashDate}
                      onTabSelect={(file) => file && centerPaneTabs.openEvidenceFile(file)}
                      onTabsChange={(tabs) => setOpenTabs(tabs)}
                      onMetadataLoaded={setHexMetadata}
                      onViewModeChange={setCurrentViewMode}
                      onHexNavigatorReady={handleHexNavigatorReady}
                      requestViewMode={requestViewMode()}
                      onViewModeRequestHandled={() => setRequestViewMode(null)}
                      breadcrumbItems={breadcrumbItems()}
                      onBreadcrumbNavigate={(path) => {
                        console.log("Navigate to:", path);
                      }}
                      scanDir={fileManager.scanDir()}
                      selectedFiles={fileManager.discoveredFiles().filter(f => 
                        fileManager.selectedFiles().has(f.path)
                      )}
                      onHashComputed={(entries) => {
                        hashManager.addTransferHashesToHistory(entries);
                      }}
                      onTransferProgressUpdate={setTransferJobs}
                      transferJobs={transferJobs()}
                      onTransferJobsChange={setTransferJobs}
                    />
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
                    />
                  </Show>
                  
                  {/* Container entry tabs - show ContainerEntryViewer */}
                  <Show when={tab().type === "entry" && tab().entry}>
                    <ContainerEntryViewer
                      entry={tab().entry!}
                      viewMode={entryContentViewMode()}
                      onBack={() => centerPaneTabs.closeTab(tab().id)}
                      onViewModeChange={setEntryContentViewMode}
                    />
                  </Show>
                  
                  {/* Processed database tabs */}
                  <Show when={tab().type === "processed" && tab().processedDb}>
                    <ProcessedDetailPanel
                      database={tab().processedDb!}
                      caseInfo={processedDbManager.selectedCaseInfo()}
                      categories={processedDbManager.selectedCategories()}
                      loading={processedDbManager.isSelectedLoading()}
                      detailView={processedDbManager.detailView()}
                      onDetailViewChange={(view) => processedDbManager.setDetailView(view)}
                    />
                  </Show>
                  
                  {/* Export tab */}
                  <Show when={tab().type === "export"}>
                    <div class="flex flex-col h-full p-4">
                      <h2 class="text-lg font-semibold mb-4">Export Panel</h2>
                      <p class="text-txt-muted">Select files to export from the Evidence panel.</p>
                    </div>
                  </Show>
                </>
              )}
            </Show>
            
            {/* Fallback when no tabs or no active tab - show empty state or legacy DetailPanel */}
            <Show when={!centerPaneTabs.activeTab() && fileManager.activeFile()}>
              <DetailPanel
                activeFile={fileManager.activeFile()}
                fileInfoMap={fileManager.fileInfoMap}
                fileStatusMap={fileManager.fileStatusMap}
                fileHashMap={hashManager.fileHashMap}
                hashHistory={hashManager.hashHistory}
                segmentResults={hashManager.segmentResults}
                tree={fileManager.tree()}
                filteredTree={fileManager.filteredTree()}
                treeFilter={fileManager.treeFilter()}
                onTreeFilterChange={(filter: string) => fileManager.setTreeFilter(filter)}
                selectedHashAlgorithm={hashManager.selectedHashAlgorithm()}
                segmentVerifyProgress={hashManager.segmentVerifyProgress()}
                storedHashesGetter={hashManager.getAllStoredHashesSorted}
                busy={fileManager.busy()}
                onVerifySegments={(file) => hashManager.verifySegments(file)}
                onLoadInfo={(file) => fileManager.loadFileInfo(file, true)}
                formatHashDate={hashManager.formatHashDate}
                onTabSelect={(file) => file && centerPaneTabs.openEvidenceFile(file)}
                onTabsChange={(tabs) => setOpenTabs(tabs)}
                onMetadataLoaded={setHexMetadata}
                onViewModeChange={setCurrentViewMode}
                onHexNavigatorReady={handleHexNavigatorReady}
                requestViewMode={requestViewMode()}
                onViewModeRequestHandled={() => setRequestViewMode(null)}
                breadcrumbItems={breadcrumbItems()}
                onBreadcrumbNavigate={(path) => {
                  console.log("Navigate to:", path);
                }}
                scanDir={fileManager.scanDir()}
                selectedFiles={fileManager.discoveredFiles().filter(f => 
                  fileManager.selectedFiles().has(f.path)
                )}
                onHashComputed={(entries) => {
                  hashManager.addTransferHashesToHistory(entries);
                }}
                onTransferProgressUpdate={setTransferJobs}
                transferJobs={transferJobs()}
                onTransferJobsChange={setTransferJobs}
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
          transferJobs={transferJobs}
          setTransferJobs={setTransferJobs}
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
        progressItems={transferProgressItems()}
        autoSaveStatus={autoSaveStatus()}
        autoSaveEnabled={projectManager.autoSaveEnabled()}
        lastAutoSave={projectManager.lastAutoSave()}
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
        <ReportWizard
          files={fileManager.discoveredFiles()}
          fileInfoMap={fileManager.fileInfoMap()}
          fileHashMap={hashManager.fileHashMap()}
          onClose={() => setShowReportWizard(false)}
          onGenerated={(path: string, format: string) => {
            console.log(`Report generated: ${path} (${format})`);
            fileManager.setOk(`Report saved to ${path}`);
          }}
        />
      </Show>
    </div>
  );
}

export default App;
