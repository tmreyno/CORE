// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, on, Show, lazy, Suspense, batch } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useFileManager, useHashManager, useDatabase, useProject, useProcessedDatabases, useHistoryContext, usePreferenceEffects, useKeyboardHandler, createSearchHandlers, createContextMenuBuilders, createCommandPaletteActions, useAppState, useDatabaseEffects, useCenterPaneTabs, useActivityManager, useEntryNavigation, useActivityLogging, useProjectActions, useMenuActions, useLoadingState, useSearchIndex, useWorkspaceMode, TAB_MODULE_MAP, type DetailViewType } from "./hooks";
import { useAppLifecycle } from "./hooks/useAppLifecycle";
import { useAppHandlers } from "./hooks/useAppHandlers";
import { confirmUnsavedChanges } from "./hooks/useCloseConfirmation";
import { useDualPanelResize } from "./hooks/usePanelResize";
import { Toolbar, StatusBar, DetailPanel, ProgressModal, ContainerEntryViewer, useToast, pathToBreadcrumbs, createContextMenu, useTour, DEFAULT_TOUR_STEPS, useDragDrop, Sidebar, AppModals, RightPanel, CenterPane, LeftPanelContent, ExportPanel } from "./components";
import { LoadingOverlay } from "./components/ui";
import { AppSecondaryModals } from "./components/layout/AppSecondaryModals";
import { HelpPanel } from "./components/HelpPanel";
import { QuickActionsBar } from "./components/QuickActionsBar";
import { AppHeader } from "./components/layout/AppHeader";
import { useWorkspaceProfiles } from "./hooks/useWorkspaceProfiles";
import type { DiscoveredFile } from "./types";
import type { ViewerMetadata } from "./types/viewerMetadata";
import type { SelectedEntry } from "./components/EvidenceTree";
import { createPreferences, getPreference, getRecentProjects } from "./components/preferences";
import { createThemeActions } from "./hooks/useTheme";
import { announce } from "./utils/accessibility";
import { logger } from "./utils/logger";
import { getBasename, getDirname } from "./utils/pathUtils";
import { isAcquireEdition, isFullEdition } from "./utils/edition";
import { isTauri } from "./utils/platform";
import { zoomIn, zoomOut, zoomReset, restoreZoom } from "./utils/zoom";
import type { AcquireView } from "./components/acquire/AcquireLayout";
import AcquireLayout from "./components/acquire/AcquireLayout";
import { getContainerType } from "./constants/ui";
import { usePortableMode } from "./hooks/usePortableMode";
import { useAcquisitionSession } from "./hooks/acquire/useAcquisitionSession";
import type { AcquisitionSessionWriter } from "./hooks/acquire/useAcquisitionRunner";
import StartSessionDialog from "./components/acquire/StartSessionDialog";
import { ProjectCloseModal, type ProjectCloseModalStep, type ProjectCloseModalStepStatus } from "./components/project/ProjectCloseModal";
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

type CloseWorkflowReason = "close-project" | "switch-project" | "new-project" | "window-close";
type ProjectCloseUiStepId =
  | "save-project"
  | "end-session"
  | "flush-db-sync"
  | "checkpoint-db"
  | "close-db"
  | "clear-state";

interface ProjectCloseModalState {
  show: boolean;
  title: string;
  message: string;
  error: string | null;
  steps: ProjectCloseModalStep[];
}

const isCffxProjectPath = (path: string) => path.toLowerCase().endsWith(".cffx");
const isAcquisitionSessionPath = (path: string) => path.toLowerCase().endsWith(".acquisition.json");

// AcquireLayout is eagerly imported (not lazy) because it is the primary view
// in the Acquire edition and is always needed on initial render. Lazy-loading it
// caused a visible "Loading…" flash in production builds.

function App() {
  const t0 = performance.now();
  log.info(`App component initializing (edition: ${isAcquireEdition() ? "Acquire" : "Full"})`);
  
  // ===========================================================================
  // Core Services & Hooks
  // ===========================================================================
  const toast = useToast();
  const history = useHistoryContext();
  const preferences = createPreferences();
  log.debug(`Core providers ready: toast, history, preferences (+${(performance.now() - t0).toFixed(1)}ms)`);
  
  const db = useDatabase();
  const fileManager = useFileManager();
  const hashManager = useHashManager(fileManager);
  log.debug(`Data hooks ready: db, fileManager, hashManager (+${(performance.now() - t0).toFixed(1)}ms)`);
  
  const projectManager = useProject();
  const processedDbManager = useProcessedDatabases();
  const workspaceProfiles = useWorkspaceProfiles();
  const workspaceMode = useWorkspaceMode();
  const globalLoading = useLoadingState();
  log.debug(`Project hooks ready: projectManager, processedDbManager, workspaceProfiles, workspaceMode, globalLoading (+${(performance.now() - t0).toFixed(1)}ms)`);
  
  // Search index lifecycle (Tantivy full-text search)
  useSearchIndex({
    hasProject: () => !!projectManager.hasProject(),
    projectPath: () => projectManager.projectPath(),
    discoveredFilePaths: () => fileManager.discoveredFiles().map(f => f.path),
  });
  log.debug(`Search index hook initialized (+${(performance.now() - t0).toFixed(1)}ms)`);
  
  // Theme actions (uses preferences as single source of truth)
  const themeActions = createThemeActions(
    () => preferences.preferences().theme,
    (theme) => preferences.updatePreference("theme", theme)
  );
  
  // Apply preferences to UI (font size, etc.)
  usePreferenceEffects(preferences.preferences);
  log.debug(`Theme & preference effects applied (+${(performance.now() - t0).toFixed(1)}ms)`);

  // Restore zoom level from localStorage
  restoreZoom();
  
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
  log.debug(`UI state & panels initialized (+${(performance.now() - t0).toFixed(1)}ms)`);
  
  // Destructure for easier access
  const { showCommandPalette, setShowCommandPalette, showShortcutsModal, setShowShortcutsModal, 
          showPerformancePanel, setShowPerformancePanel, showSettingsPanel, setShowSettingsPanel,
          showSearchPanel, setShowSearchPanel, showWelcomeModal, setShowWelcomeModal,
          showReportWizard, setShowReportWizard, showProjectWizard, setShowProjectWizard,
          showUpdateModal, setShowUpdateModal, showMergeWizard, setShowMergeWizard,
          showImportWizard, setShowImportWizard,
          showDedupPanel, setShowDedupPanel,
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
  const [viewerMetadata, setViewerMetadata] = createSignal<ViewerMetadata | null>(null);
  
  // Linked data nodes for right panel (emitted by EvidenceCollectionPanel)
  const [linkedDataNodes, setLinkedDataNodes] = createSignal<import("./components/LinkedDataTree").LinkedDataNode[]>([]);
  
  // Report wizard: optional pre-selected report type from sidebar context menu
  const [initialReportType, setInitialReportType] = createSignal<import("./components/report/types").ReportType | undefined>(undefined);
  
  // Search: initial query from text selection in viewers
  const [searchInitialQuery, setSearchInitialQuery] = createSignal<string | undefined>(undefined);

  const buildProjectCloseSteps = (includeSaveStep: boolean): ProjectCloseModalStep[] => {
    const steps: ProjectCloseModalStep[] = [];

    if (includeSaveStep) {
      steps.push({
        id: "save-project",
        label: "Save project file",
        detail: "Writing the latest .cffx project snapshot to disk.",
        status: "pending",
      });
    }

    return steps.concat([
      {
        id: "end-session",
        label: "Finalize session",
        detail: "Closing the active session and capturing the final activity state.",
        status: "pending",
      },
      {
        id: "flush-db-sync",
        label: "Drain database writes",
        detail: "Waiting for queued .ffxdb writes to finish.",
        status: "pending",
      },
      {
        id: "checkpoint-db",
        label: "Checkpoint project database",
        detail: "Folding WAL data back into the main .ffxdb file.",
        status: "pending",
      },
      {
        id: "close-db",
        label: "Close project database",
        detail: "Closing the project database connection for this window.",
        status: "pending",
      },
      {
        id: "clear-state",
        label: "Clear project state",
        detail: "Resetting project-specific UI and in-memory state.",
        status: "pending",
      },
    ]);
  };

  const [projectCloseModal, setProjectCloseModal] = createSignal<ProjectCloseModalState>({
    show: false,
    title: "",
    message: "",
    error: null,
    steps: [],
  });

  const updateProjectCloseStep = (
    id: ProjectCloseUiStepId,
    status: ProjectCloseModalStepStatus,
    detail?: string,
  ) => {
    setProjectCloseModal((current) => ({
      ...current,
      steps: current.steps.map((step) =>
        step.id === id
          ? {
              ...step,
              status,
              detail: detail ?? step.detail,
            }
          : step,
      ),
    }));
  };

  const startProjectCloseModal = (
    reason: CloseWorkflowReason,
    includeSaveStep: boolean,
  ) => {
    const title = reason === "window-close"
      ? "Saving Before Close"
      : reason === "switch-project"
        ? "Closing Current Project"
        : reason === "new-project"
          ? "Preparing New Project"
          : "Closing Project";

    const message = reason === "window-close"
      ? "Finalizing project state before the window closes."
      : reason === "switch-project"
        ? "Saving and closing the current project before loading the selected project."
        : reason === "new-project"
          ? "Saving and closing the current project before creating the new project."
          : "Saving and closing the current project. The steps below reflect exactly what is being finalized.";

    setProjectCloseModal({
      show: true,
      title,
      message,
      error: null,
      steps: buildProjectCloseSteps(includeSaveStep),
    });
  };

  const finishProjectCloseModal = () => {
    setProjectCloseModal({
      show: false,
      title: "",
      message: "",
      error: null,
      steps: [],
    });
  };
  
  // Acquire edition state
  const [acquireView, setAcquireView] = createSignal<AcquireView>("dashboard");
  const [acquireExportMode, setAcquireExportMode] = createSignal<import("./hooks/export/types").ExportMode>("physical");
  const portableMode = usePortableMode();

  // Acquire session — lightweight .acquisition.json file (replaces .cffx + .ffxdb in Acquire edition)
  const sessionManager = isAcquireEdition() ? useAcquisitionSession() : null;
  const [showAcquireSessionDialog, setShowAcquireSessionDialog] = createSignal(false);

  // Stable reactive accessor: true when the current edition has an active project/session.
  // Must be a createMemo (not an inline arrow) so SolidJS tracks the dependency correctly.
  const acquireHasProject = createMemo(() =>
    isAcquireEdition()
      ? !!sessionManager?.hasSession() || !!projectManager.hasProject()
      : !!projectManager.hasProject()
  );

  log.debug(`Acquire & portable mode state ready (+${(performance.now() - t0).toFixed(1)}ms)`);

  // When Acquire browse mode activates, ensure the sidebar is visible and show evidence tab
  createEffect(on(acquireView, (view) => {
    if (isAcquireEdition() && view === "browse") {
      setLeftCollapsed(false);
      setLeftPanelTab("evidence");
    }
  }));

  // Helper: true when Acquire edition is in browse mode
  const isAcquireBrowse = () => isAcquireEdition() && acquireView() === "browse";
  
  // Pending drive sources — set by DriveSourcePanel, consumed by ExportPanel
  const [pendingDriveSources, setPendingDriveSources] = createSignal<string[]>([]);
  const [pendingExportMode, setPendingExportMode] = createSignal<import("./hooks/export/types").ExportMode | null>(null);
  const [pendingDestination, setPendingDestination] = createSignal<string>("");
  const [pendingRemoveSources, setPendingRemoveSources] = createSignal<string[]>([]);

  // Activity Tracking — lifecycle managed by useActivityManager hook
  const activityManager = useActivityManager();
  const { activities, setActivities } = activityManager;

  // Derived: active triage activity (survives view changes for TriageMode fallback)
  const activeTriageActivity = createMemo(() =>
    activities().find(a => a.type === "triage" && (a.status === "running" || a.status === "pending"))
  );
  
  // ===========================================================================
  // Unified Center Pane Tabs - new unified tab management
  // ===========================================================================
  const centerPaneTabs = useCenterPaneTabs();
  const activeViewerEntryKey = createMemo(() => {
    const tab = centerPaneTabs.activeTab();
    if (tab?.type === "entry" && tab.entry) {
      return `${tab.entry.containerPath}::${tab.entry.entryPath}`;
    }
    if (tab?.type === "document" && tab.documentEntry) {
      return `${tab.documentEntry.containerPath}::${tab.documentEntry.entryPath}`;
    }
    return null;
  });

  const metadataEntryKey = (metadata: ViewerMetadata) => {
    const path = metadata.fileInfo.path;
    const containerPath = metadata.fileInfo.containerPath ?? path;
    return `${containerPath}::${path}`;
  };

  const setActiveViewerMetadata = (metadata: ViewerMetadata | null) => {
    if (!metadata) {
      setViewerMetadata(null);
      return;
    }

    const expectedKey = activeViewerEntryKey();
    if (!expectedKey || metadataEntryKey(metadata) !== expectedKey) {
      log.debug("Ignoring metadata from inactive viewer tab:", metadata.fileInfo.path);
      return;
    }

    setViewerMetadata(metadata);
  };
  
  // Quick Actions Bar visibility (hidden by default, toggled via title bar button)
  const [showQuickActions, setShowQuickActions] = createSignal(false);
  log.debug(`Center pane tabs & quick-actions ready (+${(performance.now() - t0).toFixed(1)}ms)`);

  /** Handler: drives panel requests sources be added to the export panel */
  const handleExportSources = (paths: string[], mode?: import("./hooks/export/types").ExportMode, destination?: string) => {
    batch(() => {
      if (paths.length > 0) {
        setPendingDriveSources(prev => [...prev, ...paths]);
      }
      if (mode) setPendingExportMode(mode);
      if (destination) setPendingDestination(destination);
    });
    openExportWithDrives();
  };

  /** Handler: single source added from drive panel check (auto-send) */
  const handleSourceAdd = (path: string) => {
    setPendingDriveSources(prev => [...prev, path]);
    // Auto-open export tab on first selection
    if (!centerPaneTabs.tabs().some(t => t.type === "export")) {
      openExportWithDrives();
    }
  };

  /** Handler: single source removed from drive panel uncheck (bidirectional sync) */
  const handleSourceRemove = (path: string) => {
    // Remove from pending if not yet consumed
    setPendingDriveSources(prev => prev.filter(p => p !== path));
    // Also add to removal list for already-consumed items in export panel
    setPendingRemoveSources(prev => [...prev, path]);
  };

  /** Opens the export tab and switches the left panel to the drives/sources view */
  const openExportWithDrives = () => {
    centerPaneTabs.openExportTab();
    setLeftCollapsed(false);
    setLeftPanelTab("drives");
  };

  /** Register acquisition output so container metadata pre-populates evidence collection forms */
  const registerAcquisitionOutput = (outputPath: string) => {
    const containerType = getContainerType(outputPath);
    if (containerType === "default") return; // not a recognized container file

    const filename = getBasename(outputPath);
    const file: DiscoveredFile = {
      path: outputPath,
      filename,
      container_type: containerType,
      size: 0,
    };
    fileManager.addDiscoveredFile(file);
    fileManager.loadFileInfo(file).catch(() => {
      // Ignore — output may not be parseable (e.g., directory path)
    });
  };

  // ===========================================================================
  // Acquire Session Helpers (lightweight .acquisition.json — Acquire edition only)
  // ===========================================================================

  /** Adapter bridging sessionManager to the AcquisitionSessionWriter interface */
  const acquireSessionWriter: AcquisitionSessionWriter | undefined = sessionManager ? {
    addAcquisition: (record) => sessionManager.addAcquisition(record),
    updateAcquisition: (id, updates) => sessionManager.updateAcquisition(id, updates),
    addActivity: (entry) => sessionManager.addActivity(entry),
  } : undefined;

  /** Create a new acquisition session from StartSessionDialog opts */
  const handleCreateSession = async (opts: import("./hooks/acquire/useAcquisitionSession").CreateSessionOpts) => {
    if (!sessionManager) return;
    try {
      await sessionManager.create(opts);
      setShowAcquireSessionDialog(false);
      toast.success("Session Created", `Case: ${opts.caseNumber}`);
    } catch (e) {
      toast.error("Failed to Create Session", String(e));
    }
  };
  
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
    const activityItems = active.map(activity => {
      const typeLabel = activity.type === "archive" ? "Archive" : activity.type === "triage" ? "Triage" : activity.type === "memory" ? "Memory" : activity.type === "export" ? "Export" : "Copy";
      let detail = activity.progress?.currentFile ? getBasename(activity.progress.currentFile) : "preparing...";
      if (activity.type === "triage" && activity.progress) {
        const p = activity.progress;
        const counts = p.filesProcessed != null && p.totalFiles ? ` ${p.filesProcessed}/${p.totalFiles}` : "";
        detail = p.currentFile ? `${getBasename(p.currentFile)}${counts}` : "preparing...";
      } else if (activity.type === "memory" && activity.progress) {
        detail = activity.progress.currentFile || "preparing...";
      }
      return {
        id: activity.id,
        label: `${typeLabel}: ${detail}`,
        progress: activity.progress?.percent ?? 0,
        indeterminate: activity.status === "pending",
        onClick: () => {
          // Map activity type to export mode tab
          const targetMode: import("./hooks/export/types").ExportMode =
            activity.type === "triage" ? "triage" :
            activity.type === "memory" ? "memory" : "native";

          if (isAcquireEdition()) {
            // Acquire edition: set mode + switch to export view
            setAcquireExportMode(targetMode);
            setAcquireView("export");
          } else {
            // Full edition: set pending mode + request view switch
            setPendingExportMode(targetMode);
            setRequestViewMode("export");
          }
        },
      };
    });
    
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
    if (!isTauri) return [];
    return getRecentProjects().filter((project) =>
      isAcquireEdition()
        ? isAcquisitionSessionPath(project.path)
        : isCffxProjectPath(project.path),
    ).map(p => ({
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

  // Clear viewer metadata immediately when switching between entry/document tabs.
  createEffect(on(
    () => activeViewerEntryKey(),
    () => setViewerMetadata(null),
  ));
  
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
  const { handleSearch, handleSearchResultSelect } = createSearchHandlers({
    fileManager,
    projectManager,
    onOpenEvidenceFile: centerPaneTabs.openEvidenceFile,
  });

  // ── Text selection actions (from viewer right-click) ──────────────────

  const handleBookmarkSelection = (selectedText: string, entryPath: string, entryName: string) => {
    const truncated = selectedText.length > 60 ? selectedText.slice(0, 60) + "…" : selectedText;
    projectManager.addBookmark({
      target_type: "file",
      target_path: entryPath,
      name: truncated,
      notes: selectedText,
      context: { selectedText, entryName },
    });
    toast.success("Selection bookmarked", truncated);
  };

  const handleNoteFromSelection = (selectedText: string, entryPath: string, entryName: string) => {
    projectManager.addNote({
      target_type: "file",
      target_path: entryPath,
      title: `Selection from ${entryName}`,
      content: selectedText,
    });
    toast.success("Note created from selection", `${selectedText.length} characters`);
  };

  const handleSearchSelection = (selectedText: string) => {
    setSearchInitialQuery(selectedText);
    setShowSearchPanel(true);
  };

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

  const closeCurrentProject = async (reason: CloseWorkflowReason): Promise<boolean> => {
    if (!projectManager.hasProject()) {
      return true;
    }

    const hasUnsavedChanges = projectManager.modified();
    let shouldSave = false;

    if (hasUnsavedChanges) {
      const decision = await confirmUnsavedChanges({
        title: "Save Project Before Closing?",
        message: "The current project has unsaved changes. Save it before closing or switching projects?",
      });

      if (decision === "cancel") {
        return false;
      }

      shouldSave = decision === "save";
    }

    startProjectCloseModal(reason, hasUnsavedChanges);

    try {
      if (hasUnsavedChanges) {
        if (shouldSave) {
          updateProjectCloseStep("save-project", "running", "Saving the latest .cffx project snapshot.");

          const saveOptions = getSaveOptions();
          if (!saveOptions) {
            updateProjectCloseStep("save-project", "failed", "The current project could not be serialized for save.");
            setProjectCloseModal((current) => ({
              ...current,
              error: "Project close stopped because the current project could not be prepared for saving.",
            }));
            return false;
          }

          const saveResult = await projectManager.saveProject(
            saveOptions,
            projectManager.projectPath() || undefined,
          );

          if (!saveResult.success) {
            const saveError = saveResult.error || "Project save failed.";
            updateProjectCloseStep("save-project", "failed", saveError);
            setProjectCloseModal((current) => ({
              ...current,
              error: saveError === "Save cancelled"
                ? "Project close was cancelled because the save dialog was cancelled."
                : saveError,
            }));
            return false;
          }

          updateProjectCloseStep("save-project", "completed", `Saved to ${getBasename(saveResult.path || projectManager.projectPath() || "project.cffx")}.`);
        } else {
          updateProjectCloseStep("save-project", "skipped", "Unsaved .cffx changes were discarded before closing the project.");
        }
      }

      const closeResult = await projectManager.clearProject({
        onProgress: ({ step, status, detail }) => {
          updateProjectCloseStep(step as ProjectCloseUiStepId, status, detail);
        },
      });

      if (!closeResult.success) {
        setProjectCloseModal((current) => ({
          ...current,
          error: closeResult.error || "Project close failed.",
        }));
        return false;
      }

      if (closeResult.flushTimedOut) {
        toast.warning(
          "Project Closed With Pending Writes",
          "Background project DB writes were still finishing when the project close completed.",
        );
      } else if (reason === "close-project") {
        toast.success("Project Closed", "The current project was saved and closed.");
      }

      finishProjectCloseModal();
      return true;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setProjectCloseModal((current) => ({
        ...current,
        error: errorMsg,
      }));
      return false;
    }
  };
  
  // Loading-wrapped versions of slow project operations
  let projectLoadInProgress = false;
  let projectSetupInProgress = false;
  const projectActionBusy = () => projectLoadInProgress || projectSetupInProgress;
  const notifyProjectActionBusy = () => {
    toast.info(
      "Project Action In Progress",
      "Finish or cancel the current project action before starting another.",
    );
  };

  const handleLoadProject = async (path?: string) => {
    if (projectActionBusy()) {
      notifyProjectActionBusy();
      return;
    }
    projectLoadInProgress = true;
    try {
      if (!path) {
        if (!projectManager.hasProject() || !isTauri) {
          await _handleLoadProject();
          return;
        }

        const selected = await open({
          filters: [{ name: "CORE-FFX Project", extensions: ["cffx"] }],
          title: "Open Project",
          multiple: false,
        });

        if (!selected) {
          return;
        }

        path = selected as string;
      }

      const canProceed = await closeCurrentProject("switch-project");
      if (!canProceed) return;
      await globalLoading.run("Loading project…", () => _handleLoadProject(path));
    } finally {
      projectLoadInProgress = false;
    }
  };
  const handleOpenRecentProject = async (path: string) => {
    if (isAcquireEdition()) {
      if (!sessionManager) return;
      if (!isAcquisitionSessionPath(path)) {
        toast.error("Cannot Open Recent Item", "Acquire can only open .acquisition.json session files.");
        return;
      }
      try {
        await sessionManager.load(path);
        toast.success("Session Loaded", `Loaded ${getBasename(path)}`);
      } catch (e) {
        toast.error("Failed to Load Session", String(e));
      }
      return;
    }

    if (!isCffxProjectPath(path)) {
      toast.error("Cannot Open Recent Item", "Full Suite can only open .cffx project files.");
      return;
    }
    await handleLoadProject(path);
  };
  const handleSaveProject = () =>
    globalLoading.run("Saving project…", () => _handleSaveProject());
  const handleSaveProjectAs = () =>
    globalLoading.run("Saving project…", () => _handleSaveProjectAs());
  const handleProjectSetupComplete = async (locations: import("./components").ProjectLocations) => {
    if (projectActionBusy()) {
      notifyProjectActionBusy();
      return;
    }
    projectSetupInProgress = true;
    try {
      const canProceed = await closeCurrentProject("new-project");
      if (!canProceed) return;
      await globalLoading.run("Setting up project…", () => _handleProjectSetupComplete(locations));
    } finally {
      projectSetupInProgress = false;
    }
  };
  const handleNewProject = () => {
    if (projectActionBusy()) {
      notifyProjectActionBusy();
      return;
    }

    if (isAcquireEdition()) {
      setShowAcquireSessionDialog(true);
      return;
    }

    if (!isTauri) {
      void handleProjectSetupComplete({
        projectName: "Browser Preview Project",
        projectRoot: "browser-preview-project",
        evidencePath: "browser-preview-project/Evidence",
        processedDbPath: "browser-preview-project/Processed",
        caseDocumentsPath: "browser-preview-project/Case.Documents",
        exportsPath: "browser-preview-project/Exports",
        discoveredEvidence: [],
        discoveredDatabases: [],
        loadStoredHashes: false,
      });
      return;
    }

    setShowProjectWizard(true);
  };
  const handleScanEvidence = () =>
    globalLoading.run("Scanning for evidence…", () => fileManager.scanForFiles());
  const handleCloseProject = () => closeCurrentProject("close-project");
  
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
    setShowDedupPanel,
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
    onOpenEvidenceFile: centerPaneTabs.openEvidenceFile,
  });

  const activeHashEntry = (): SelectedEntry | null => {
    const tab = centerPaneTabs.activeTab();
    if (tab?.type === "entry" && tab.entry) return tab.entry;
    if (tab?.type === "document" && tab.documentEntry) return tab.documentEntry;
    if (tab) return null;
    return selectedContainerEntry();
  };

  const parentFileForEntry = (entry: SelectedEntry): DiscoveredFile | null => {
    const active = fileManager.activeFile();
    if (active?.path === entry.containerPath || (entry.isDiskFile && active?.path === entry.entryPath)) {
      return active;
    }

    return (
      fileManager
        .discoveredFiles()
        .find((file) => file.path === entry.containerPath || (entry.isDiskFile && file.path === entry.entryPath)) ??
      null
    );
  };

  const handleHashActive = () => {
    const entry = activeHashEntry();
    if (entry && !entry.isDir) {
      void hashManager.hashEntry(entry, parentFileForEntry(entry));
      return;
    }

    const active = fileManager.activeFile();
    if (active) void hashManager.hashSingleFile(active);
  };
  
  // ===========================================================================
  // Keyboard Handler Hook - manages global shortcuts
  // ===========================================================================
  log.debug("Registering keyboard handler...");
  useKeyboardHandler({
    setShowCommandPalette,
    setShowSettingsPanel,
    setShowSearchPanel,
    setShowPerformancePanel,
    setShowShortcutsModal,
    setShowProjectWizard,
    setShowReportWizard,
    onLoadProject: handleLoadProject,
    onNewProject: handleNewProject,
    onOpenDirectory: handleOpenDirectory,
    showCommandPalette,
    showShortcutsModal,
    history,
    toast,
    projectManager,
    buildSaveOptions: getSaveOptions,
    acquireView,
    setAcquireView,
    isAcquireEdition,
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
    onOpenProject: handleLoadProject,
    onNewProject: handleNewProject,
    onCloseProject: () => { void handleCloseProject(); },
    onOpenHelp: () => centerPaneTabs.openHelpTab(),
    onOpenExport: () => openExportWithDrives(),
    onToggleQuickActions: () => setShowQuickActions((prev) => !prev),
    onHashActive: handleHashActive,
    hasHashTarget: () => !!activeHashEntry() || !!fileManager.activeFile(),
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
    onDeduplication: () => setShowDedupPanel(true),
    onShowPerformance: () => setShowPerformancePanel(true),
    setShowMergeWizard,
    setShowImportWizard,
  });

  // Database synchronization effects
  useDatabaseEffects({ db, fileManager });
  log.debug(`DB effects, keyboard, and command palette ready (+${(performance.now() - t0).toFixed(1)}ms)`);

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
  log.debug(`Lifecycle hook ready (+${(performance.now() - t0).toFixed(1)}ms)`);

  // ===========================================================================
  // Native Menu Actions — handles events from macOS/Windows menu bar
  // ===========================================================================
  useMenuActions({
    onOpenProject: handleLoadProject,
    onOpenDirectory: handleOpenDirectory,
    onSaveProject: handleSaveProject,
    onSaveProjectAs: handleSaveProjectAs,
    onCloseProject: () => { void handleCloseProject(); },
    onToggleSidebar: () => setLeftCollapsed((prev) => !prev),
    onToggleRightPanel: () => setRightCollapsed((prev) => !prev),
    onKeyboardShortcuts: () => setShowShortcutsModal(true),
    onCommandPalette: () => setShowCommandPalette(true),
    onNewProject: handleNewProject,
    onExport: () => { if (projectManager.hasProject()) openExportWithDrives(); },
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
      const enabled = !projectManager.autoSaveEnabled();
      projectManager.setAutoSaveEnabled(enabled);
      if (enabled) {
        projectManager.startAutoSave();
      } else {
        projectManager.stopAutoSave();
      }
      toast.info(enabled ? "Auto-save enabled" : "Auto-save disabled");
    },
    onHashSelected: () => hashManager.hashSelectedFiles(),
    onHashActive: handleHashActive,
    onStartTour: () => tour.start(),
    onShowDashboard: () => { setLeftCollapsed(false); setLeftPanelTab("dashboard"); },
    onShowActivity: () => { setLeftCollapsed(false); setLeftPanelTab("activity"); },
    onShowBookmarks: () => { setLeftCollapsed(false); setLeftPanelTab("bookmarks"); },
    onViewInfo: () => setCurrentViewMode("info"),
    onViewHex: () => setCurrentViewMode("hex"),
    onViewText: () => setCurrentViewMode("text"),
    onCycleTheme: () => themeActions.cycleTheme(),
    onSelectAllEvidence: () => fileManager.toggleSelectAll(),
    onDeduplication: () => setShowDedupPanel(true),
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
    onImportAcquisitions: () => setShowImportWizard(true),
    onProjectRecovery: () => { if (projectManager.hasProject()) setShowRecoveryModal(true); },
    onCollectLogs: async () => {
      if (!isTauri) {
        toast.error(
          "Log Collection Unavailable",
          "Support log collection is available in the desktop app.",
        );
        return;
      }

      try {
        const datePart = new Date().toISOString().slice(0, 10);
        const path = await save({
          title: "Save Support Logs",
          defaultPath: `core-ffx-logs-${datePart}.zip`,
          filters: [{ name: "ZIP Archive", extensions: ["zip"] }],
        });
        if (!path) return;
        const result = await invoke<string>("collect_support_logs", { destPath: path });
        toast.success("Logs Collected", result);
      } catch (err) {
        toast.error("Log Collection Failed", err instanceof Error ? err.message : String(err));
      }
    },
    onZoomIn: zoomIn,
    onZoomOut: zoomOut,
    onZoomReset: zoomReset,
  });
  log.info(`App component initialization complete (+${(performance.now() - t0).toFixed(1)}ms)`);

  // Defer portable mode check until a project is loaded
  createEffect(on(
    () => !!projectManager.hasProject(),
    (hasProject, prevHasProject) => {
      if (hasProject && !prevHasProject) {
        portableMode.check();
      }
    }
  ));

  // Auto-switch sidebar tab when the active tab's module becomes disabled
  createEffect(on(
    () => workspaceMode.enabledModules(),
    (mods) => {
      const currentTab = leftPanelTab();
      const requiredModule = TAB_MODULE_MAP[currentTab];
      // Tabs without a required module (bookmarks, search) are always valid
      if (requiredModule && !mods.includes(requiredModule)) {
        setLeftPanelTab(workspaceMode.getFirstEnabledTab());
      }
    }
  ));

  // Show user confirmation modal when a project is opened or created (full edition only)
  createEffect(on(
    () => !!projectManager.hasProject(),
    (hasProject, prevHasProject) => {
      // Only trigger on the transition false → true
      if (hasProject && !prevHasProject && isFullEdition()) {
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
  const activeCenterEvidenceFile = createMemo(() => {
    const tab = centerPaneTabs.activeTab();
    return tab?.type === "evidence" ? tab.file ?? null : null;
  });

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
        searchInitialQuery={searchInitialQuery}
        onSearchInitialQueryConsumed={() => setSearchInitialQuery(undefined)}
        showDedupPanel={showDedupPanel}
        setShowDedupPanel={setShowDedupPanel}
        fileContextMenu={fileContextMenu}
        saveContextMenu={saveContextMenu}
        showWelcomeModal={showWelcomeModal}
        setShowWelcomeModal={setShowWelcomeModal}
        onNewProject={handleNewProject}
        onOpenProject={handleLoadProject}
        recentProjects={welcomeModalRecentProjects}
        onSelectRecentProject={handleOpenRecentProject}
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
        showImportWizard={showImportWizard}
        setShowImportWizard={setShowImportWizard}
        showRecoveryModal={showRecoveryModal}
        setShowRecoveryModal={setShowRecoveryModal}
      />

      {/* Acquire Session Dialog — lightweight session creation for Acquire edition */}
      <Show when={isAcquireEdition() && showAcquireSessionDialog()}>
        <StartSessionDialog
          isOpen={true}
          onClose={() => setShowAcquireSessionDialog(false)}
          onCreate={handleCreateSession}
          defaultExaminer={sessionManager?.examiner() || undefined}
        />
      </Show>
      
      {/* Header / Title Bar — hidden in Acquire mode */}
      <Show when={!isAcquireEdition()}>
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
      </Show>

      {/* Toolbar — hidden in Acquire mode */}
      <Show when={!isAcquireEdition()}>
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
        workspaceModeId={workspaceMode.activeMode().id}
        onWorkspaceModeChange={(modeId) => workspaceMode.setMode(modeId)}
        onOpenWorkspaceSettings={() => setShowSettingsPanel(true)}
        isModuleEnabled={(m) => workspaceMode.isModuleEnabled(m as import("./components/preferences").FeatureModule)}
      />
      </Show>
      
      {/* Quick Actions Bar - hidden by default, toggled via title bar ⚡ button */}
      <Show when={showQuickActions() && !isAcquireEdition()}>
        <QuickActionsBar
          actions={workspaceProfiles.currentProfile()?.quick_actions}
          compact={isCompact()}
          onAction={handleQuickAction}
          isModuleEnabled={(m) => workspaceMode.isModuleEnabled(m as import("./components/preferences").FeatureModule)}
        />
      </Show>

      {/* Acquire browse mode: back to dashboard bar + panel toggles */}
      <Show when={isAcquireEdition() && acquireView() === "browse"}>
        <div class="flex items-center px-3 py-1 border-b border-border bg-bg-secondary">
          <button class="btn btn-ghost gap-1 text-xs py-1 px-2" onClick={() => setAcquireView("dashboard")}>
            ← Dashboard
          </button>
          <span class="text-xs text-txt-muted ml-2">Evidence Browser</span>

          {/* Selection summary + acquisition actions */}
          <div class="flex items-center gap-2 ml-4">
            <Show when={fileManager.selectedFiles().size > 0}>
              <span class="text-xs text-accent font-medium">
                {fileManager.selectedFiles().size} selected
              </span>
              <button
                class="btn-sm btn-primary gap-1 text-xs py-0.5 px-2"
                onClick={() => {
                  setAcquireExportMode("native");
                  setAcquireView("export");
                }}
              >
                Acquire Selected →
              </button>
              <button
                class="btn-sm btn-ghost text-xs py-0.5 px-1.5"
                onClick={() => {
                  // Deselect all
                  fileManager.discoveredFiles().forEach(f => {
                    if (fileManager.selectedFiles().has(f.path)) {
                      fileManager.toggleFileSelection(f.path);
                    }
                  });
                }}
              >
                Deselect All
              </button>
            </Show>
            <Show when={fileManager.selectedFiles().size === 0 && fileManager.discoveredFiles().length > 0}>
              <button
                class="btn-sm btn-ghost gap-1 text-xs py-0.5 px-2 text-txt-muted"
                onClick={() => fileManager.toggleSelectAll()}
              >
                Select All
              </button>
            </Show>
          </div>

          {/* Panel layout toggle — same 3-rect SVG as AppHeader */}
          <div class="ml-auto flex items-center gap-0.5">
            <div class="flex items-center justify-center p-1.5 rounded-md text-txt-muted">
              <svg class="w-7 h-4" viewBox="0 0 30 20" fill="none" xmlns="http://www.w3.org/2000/svg">
                <rect x="1" y="3" width="6" height="14" rx="1"
                  fill={leftCollapsed() ? "none" : "currentColor"}
                  stroke="currentColor" stroke-width="1.2"
                  opacity={leftCollapsed() ? "0.4" : "1"}
                  pointer-events="all"
                  class="cursor-pointer transition-all duration-150"
                  style={{ color: leftCollapsed() ? "var(--color-txt-muted)" : "var(--color-accent)" }}
                  onClick={() => setLeftCollapsed((prev) => !prev)}
                >
                  <title>{leftCollapsed() ? "Show Left Panel" : "Hide Left Panel"}</title>
                </rect>
                <rect x="9" y="3" width="12" height="14"
                  fill="currentColor"
                  stroke="currentColor" stroke-width="1.2" opacity="0.5"
                  pointer-events="all"
                  class="cursor-pointer transition-all duration-150"
                  onClick={() => {
                    const bothVisible = !leftCollapsed() && !rightCollapsed();
                    setLeftCollapsed(bothVisible);
                    setRightCollapsed(bothVisible);
                  }}
                >
                  <title>{!leftCollapsed() && !rightCollapsed() ? "Hide Both Panels" : "Show Both Panels"}</title>
                </rect>
                <rect x="23" y="3" width="6" height="14" rx="1"
                  fill={rightCollapsed() ? "none" : "currentColor"}
                  stroke="currentColor" stroke-width="1.2"
                  opacity={rightCollapsed() ? "0.4" : "1"}
                  pointer-events="all"
                  class="cursor-pointer transition-all duration-150"
                  style={{ color: rightCollapsed() ? "var(--color-txt-muted)" : "var(--color-accent)" }}
                  onClick={() => setRightCollapsed((prev) => !prev)}
                >
                  <title>{rightCollapsed() ? "Show Right Panel" : "Hide Right Panel"}</title>
                </rect>
              </svg>
            </div>
          </div>
        </div>
      </Show>

      {/* Main Content Area — Acquire edition replaces three-panel layout */}
      <Show when={!isAcquireEdition() || acquireView() === "browse"} fallback={
        <main class="app-main">
            <AcquireLayout
              onSettings={() => setShowSettingsPanel(true)}
              onHelp={() => centerPaneTabs.openHelpTab()}
              onCommandPalette={() => setShowCommandPalette(true)}
              onOpenProject={handleLoadProject}
              onOpenRecentProject={handleOpenRecentProject}
              onNewProject={handleNewProject}
              projectName={() => (
                isAcquireEdition()
                  ? sessionManager?.projectName() || projectManager.projectName()
                  : projectManager.projectName()
              ) || undefined}
              hasProject={acquireHasProject}
              evidenceCount={() => fileManager.discoveredFiles().length}
              initialSources={() => fileManager.discoveredFiles()
                .filter(f => fileManager.selectedFiles().has(f.path))
                .map(f => f.path)
              }
              initialExaminerName={() => (
                isAcquireEdition()
                  ? sessionManager?.examiner()
                      || projectManager.project()?.owner_name
                      || projectManager.project()?.current_user
                  : projectManager.project()?.owner_name || projectManager.project()?.current_user
              ) || undefined}
              onExportComplete={(outputPath) => {
                toast.success("Export Complete", `Files exported to: ${outputPath}`);
                registerAcquisitionOutput(outputPath);
              }}
              initialDestination={
                isAcquireEdition()
                  ? sessionManager?.outputFolder() || projectManager.projectLocations()?.exports_path || ""
                  : projectManager.projectLocations()?.exports_path || ""
              }
              onActivityCreate={(activity) => {
                setActivities(list => [...list, activity]);
              }}
              onActivityUpdate={(id, updates) => {
                setActivities(list =>
                  list.map(a => a.id === id ? { ...a, ...updates } : a)
                );
              }}
              caseNumber={() => (
                isAcquireEdition()
                  ? sessionManager?.caseNumber() || projectManager.caseNumber()
                  : projectManager.caseNumber()
              ) || undefined}
              discoveredFiles={fileManager.discoveredFiles}
              fileInfoMap={fileManager.fileInfoMap}
              onVerifyHashes={() => {
                hashManager.hashAllFiles();
              }}
              acquireView={acquireView}
              setAcquireView={setAcquireView}
              initialExportMode={acquireExportMode}
              setInitialExportMode={setAcquireExportMode}
              isPortable={portableMode.isPortable}
              portableConfig={portableMode.config}
              activeTriageActivity={activeTriageActivity}
              evidenceBasePath={
                isAcquireEdition()
                  ? sessionManager?.evidenceFolder() || projectManager.projectLocations()?.evidence_path || ""
                  : projectManager.projectLocations()?.evidence_path || ""
              }
              currentUsername={
                isAcquireEdition()
                  ? sessionManager?.examiner() || projectManager.project()?.current_user || undefined
                  : projectManager.project()?.current_user || undefined
              }
              sessionWriter={acquireSessionWriter}
            />
        </main>
      }>
      <main class="app-main">
        {/* Left Panel */}
        <Show when={!leftCollapsed()}>
          <aside class="left-panel flex flex-row" style={{ width: `${leftWidth()}px` }}>
            {/* Vertical Icon Sidebar — hidden in Acquire browse mode */}
            <Show when={!isAcquireBrowse()}>
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
              noteCount={projectManager.noteCount}
              isModuleEnabled={workspaceMode.isModuleEnabled}
              onExport={() => openExportWithDrives()}
              onReport={() => { if (projectManager.hasProject()) { setInitialReportType(undefined); setShowReportWizard(true); } }}
              onReportType={(type: string) => { if (projectManager.hasProject()) { setInitialReportType(type as import("./components/report/types").ReportType); setShowReportWizard(true); } }}
              onExportSelected={() => openExportWithDrives()}
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
                if (!isTauri) {
                  toast.error(
                    "Bookmark Export Unavailable",
                    "Bookmark export file saving is available in the desktop app.",
                  );
                  return;
                }

                try {
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
            </Show>
            
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
                const items = getFileContextMenuItems(fileManager.activeFile);
                // In Acquire browse mode, add acquisition selection items
                if (isAcquireBrowse()) {
                  const isSelected = fileManager.selectedFiles().has(file.path);
                  items.push(
                    { id: "sep-acq", label: "", separator: true },
                    isSelected
                      ? { id: "remove-acquisition", label: "Remove from Acquisition", icon: "❌", onSelect: () => fileManager.toggleFileSelection(file.path) }
                      : { id: "add-acquisition", label: "Add to Acquisition", icon: "📦", onSelect: () => fileManager.toggleFileSelection(file.path) },
                  );
                }
                fileContextMenu.open(e, items);
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
              onExport={() => openExportWithDrives()}
              onReport={() => { if (projectManager.hasProject()) { setInitialReportType(undefined); setShowReportWizard(true); } }}
              onExportSources={handleExportSources}
              onSourceAdd={handleSourceAdd}
              onSourceRemove={handleSourceRemove}
              onOpenProject={handleLoadProject}
              onNewProject={handleNewProject}
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
              if (tab?.type === "evidence" && tab.file) {
                fileManager.setActiveFile(tab.file);
              } else if (tab?.type === "entry" && tab.entry) {
                const parentFile = fileManager
                  .discoveredFiles()
                  .find((file) => file.path === tab.entry?.containerPath);
                if (parentFile) fileManager.setActiveFile(parentFile);
              }
              centerPaneTabs.setActiveTabId(tabId);
            }}
            onTabClose={centerPaneTabs.closeTab}
            onTabsChange={centerPaneTabs.setTabs}
            viewMode={centerPaneTabs.viewMode}
            onViewModeChange={centerPaneTabs.setViewMode}
            onOpenProject={handleLoadProject}
            onNewProject={handleNewProject}
            projectName={projectManager.projectName}
            projectRoot={projectManager.rootPath}
            evidenceCount={() => fileManager.discoveredFiles().length}
          >
            {/* Content based on active tab type and view mode */}
            <Show when={centerPaneTabs.activeTab()}>
              {(tab) => (
                <>
                  {/* Evidence file tabs - show DetailPanel (handles all view modes internally) */}
                  <Show keyed when={activeCenterEvidenceFile()}>
                    {(activeEvidenceFile) => (
                      <DetailPanel {...sharedDetailPanelProps(activeEvidenceFile)} />
                    )}
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
                      onMetadata={setActiveViewerMetadata}
                      onBookmarkSelection={handleBookmarkSelection}
                      onNoteFromSelection={handleNoteFromSelection}
                      onSearchSelection={handleSearchSelection}
                    />
                  </Show>
                  
                  {/* Container entry tabs - show ContainerEntryViewer */}
                  <Show when={tab().type === "entry" && tab().entry}>
                    <ContainerEntryViewer
                      entry={tab().entry!}
                      viewMode={entryContentViewMode()}
                      onBack={() => centerPaneTabs.closeTab(tab().id)}
                      onViewModeChange={setEntryContentViewMode}
                      onMetadata={setActiveViewerMetadata}
                      onBookmarkSelection={handleBookmarkSelection}
                      onNoteFromSelection={handleNoteFromSelection}
                      onSearchSelection={handleSearchSelection}
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
                      caseNumber={projectManager.caseNumber() || undefined}
                      projectName={projectManager.projectName() || undefined}
                      initialDestination={projectManager.projectLocations()?.exports_path || ""}
                      pendingDriveSources={pendingDriveSources}
                      pendingExportMode={pendingExportMode}
                      pendingDestination={pendingDestination}
                      pendingRemoveSources={pendingRemoveSources}
                      onPendingSourcesConsumed={() => { setPendingDriveSources([]); setPendingExportMode(null); setPendingDestination(""); }}
                      onPendingRemoveConsumed={() => setPendingRemoveSources([])}
                      onComplete={(outputPath) => {
                        toast.success("Export Complete", `Files exported to: ${outputPath}`);
                        registerAcquisitionOutput(outputPath);
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
                      activeTriageActivity={activeTriageActivity}
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
      </Show>

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
          const enabled = !projectManager.autoSaveEnabled();
          projectManager.setAutoSaveEnabled(enabled);
          if (enabled) {
            projectManager.startAutoSave();
          } else {
            projectManager.stopAutoSave();
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

      <ProjectCloseModal
        show={projectCloseModal().show}
        title={projectCloseModal().title}
        message={projectCloseModal().message}
        steps={projectCloseModal().steps}
        error={projectCloseModal().error}
        onDismiss={projectCloseModal().error ? finishProjectCloseModal : undefined}
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
