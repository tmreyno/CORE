// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { onMount, onCleanup, createSignal, createEffect, Show, lazy } from "solid-js";
import { useFileManager, useHashManager, useDatabase, useProject, useProcessedDatabases, useHistoryContext, usePreferenceEffects, useTransferEvents } from "./hooks";
import { useDualPanelResize } from "./hooks/usePanelResize";
import { Toolbar, StatusBar, DetailPanel, TreePanel, ProgressModal, MetadataPanel, ProjectSetupWizard, EvidenceTree, ContainerEntryViewer, CommandPalette, KeyboardShortcutsModal, DEFAULT_SHORTCUT_GROUPS, useToast, ThemeSwitcher, pathToBreadcrumbs, SearchPanel, ContextMenu, createContextMenu, WelcomeModal, useTour, TourOverlay, DEFAULT_TOUR_STEPS, useDragDrop, CaseDocumentsPanel, TransferProgressPanel } from "./components";
import { ActivityPanel } from "./components/ActivityPanel";
import type { ProjectLocations, SelectedEntry, OpenTab, CommandAction, SearchFilter, SearchResult, ContextMenuItem } from "./components";
import type { DiscoveredFile, CaseDocument } from "./types";
import { createPreferences } from "./components/preferences";
import { createThemeActions } from "./hooks/useTheme";
import type { ParsedMetadata, TabViewMode } from "./components";
import { announce } from "./utils/accessibility";
import { logError, logInfo } from "./utils/telemetry";
import { transferCancel } from "./transfer";
import ffxLogo from "./assets/branding/core-logo-48.png";
import "./App.css";

// ============================================================================
// Lazy-loaded Components (Code Splitting)
// These are heavy components that aren't needed on initial render
// ============================================================================
const ReportWizard = lazy(() => import("./components/report/wizard/ReportWizard").then(m => ({ default: m.ReportWizard })));
const SettingsPanel = lazy(() => import("./components/SettingsPanel"));
const PerformancePanel = lazy(() => import("./components/PerformancePanel"));
const ProcessedDatabasePanel = lazy(() => import("./components/ProcessedDatabasePanel"));
const ProcessedDetailPanel = lazy(() => import("./components/ProcessedDetailPanel"));

import {
  HiOutlineFolderOpen,
  HiOutlineDocumentText,
  HiOutlineInformationCircle,
  HiOutlineCodeBracket,
  HiOutlineDocument,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineLockClosed,
  HiOutlineCheckBadge,
  HiOutlineCog6Tooth,
  HiOutlineArrowPath,
  HiOutlineArchiveBox,
  HiOutlineChartBar,
  HiOutlineCommandLine,
  HiOutlineMagnifyingGlass,
  HiOutlineClipboardDocumentList,
  HiOutlineDocumentArrowDown,
  HiOutlineFolderArrowDown,
  HiOutlineArrowUpTray,
  HiOutlineClock,
} from "./components/icons";

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
  
  const [leftPanelTab, setLeftPanelTab] = createSignal<"evidence" | "processed" | "casedocs" | "activity">("evidence");
  const [windowWidth, setWindowWidth] = createSignal(window.innerWidth);
  const isCompact = () => windowWidth() < 900;
  
  // ===========================================================================
  // UI State - Modals & Overlays
  // ===========================================================================
  const [showCommandPalette, setShowCommandPalette] = createSignal(false);
  const [showShortcutsModal, setShowShortcutsModal] = createSignal(false);
  const [showPerformancePanel, setShowPerformancePanel] = createSignal(false);
  const [showSettingsPanel, setShowSettingsPanel] = createSignal(false);
  const [showSearchPanel, setShowSearchPanel] = createSignal(false);
  const [showWelcomeModal, setShowWelcomeModal] = createSignal(false);
  const [showReportWizard, setShowReportWizard] = createSignal(false);
  const [showProjectWizard, setShowProjectWizard] = createSignal(false);
  
  // ===========================================================================
  // UI State - View & Content
  // ===========================================================================
  const [openTabs, setOpenTabs] = createSignal<OpenTab[]>([]);
  const [currentViewMode, setCurrentViewMode] = createSignal<TabViewMode>("info");
  const [hexMetadata, setHexMetadata] = createSignal<ParsedMetadata | null>(null);
  const [selectedContainerEntry, setSelectedContainerEntry] = createSignal<SelectedEntry | null>(null);
  const [entryContentViewMode, setEntryContentViewMode] = createSignal<"auto" | "hex" | "text" | "document">("hex");
  const [requestViewMode, setRequestViewMode] = createSignal<"info" | "hex" | "text" | "pdf" | "export" | null>(null);
  const [hexNavigator, setHexNavigator] = createSignal<((offset: number, size?: number) => void) | null>(null);
  
  // ===========================================================================
  // UI State - Project & Documents
  // ===========================================================================
  const [pendingProjectRoot, setPendingProjectRoot] = createSignal<string | null>(null);
  const [caseDocumentsPath, setCaseDocumentsPath] = createSignal<string | null>(null);
  const [caseDocuments, setCaseDocuments] = createSignal<CaseDocument[] | null>(null);
  
  // ===========================================================================
  // UI State - Transfer & Progress
  // ===========================================================================
  const [transferJobs, setTransferJobs] = createSignal<import("./components").TransferJob[]>([]);
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
  
  /**
   * Search handler for SearchPanel - searches both file names and container contents.
   */
  const handleSearch = async (query: string, _filters: SearchFilter): Promise<SearchResult[]> => {
    const lowerQuery = query.toLowerCase();
    const results: SearchResult[] = [];
    const files = fileManager.discoveredFiles();
    
    // 1. Search through discovered files (container files themselves)
    for (const file of files) {
      const name = file.path.split("/").pop() || file.path;
      const matchesName = name.toLowerCase().includes(lowerQuery);
      const matchesPath = file.path.toLowerCase().includes(lowerQuery);
      
      if (matchesName || matchesPath) {
        results.push({
          id: file.path,
          path: file.path,
          name,
          size: file.size || 0,
          isDir: false,
          score: matchesName ? 100 : 50,
          matchType: matchesName ? "name" : "path",
        });
      }
    }
    
    // 2. Search INSIDE containers using backend (for queries >= 2 chars)
    if (query.length >= 2) {
      const { invoke } = await import("@tauri-apps/api/core");
      
      // Build list of containers to search
      const containers = files
        .filter(f => ["ad1", "zip", "7z", "rar", "tar", "tgz"].some(
          ext => f.container_type.toLowerCase().includes(ext)
        ))
        .map(f => [f.path, f.container_type.toLowerCase()] as [string, string]);
      
      if (containers.length > 0) {
        try {
          const containerResults = await invoke<Array<{
            containerPath: string;
            containerType: string;
            entryPath: string;
            name: string;
            isDir: boolean;
            size: number;
            score: number;
            matchType: string;
          }>>("search_all_containers", {
            containers,
            query,
            options: { maxResults: 200, includeDirs: false }
          });
          
          // Convert backend results to SearchResult format
          for (const r of containerResults) {
            results.push({
              id: `${r.containerPath}::${r.entryPath}`,
              path: r.entryPath,
              name: r.name,
              size: r.size,
              isDir: r.isDir,
              score: r.score,
              containerPath: r.containerPath,
              containerType: r.containerType,
              matchType: r.matchType,
            });
          }
        } catch (err) {
          console.error("Container search failed:", err);
        }
      }
    }
    
    // Sort by score (highest first) and limit results
    return results.sort((a, b) => b.score - a.score).slice(0, 300);
  };
  
  /**
   * Handle search result selection - navigates to file or container entry.
   */
  const handleSearchResultSelect = (result: SearchResult) => {
    if (result.containerPath) {
      // Result is inside a container - find container and select entry
      const containerFile = fileManager.discoveredFiles().find(f => f.path === result.containerPath);
      if (containerFile) {
        fileManager.setActiveFile(containerFile);
        announce(`Found ${result.name} in ${containerFile.path.split("/").pop()}`);
      }
    } else {
      // Result is a top-level file
      const file = fileManager.discoveredFiles().find(f => f.path === result.path);
      if (file) {
        fileManager.setActiveFile(file);
        announce(`Selected ${result.name}`);
      }
    }
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
  
  // Get context menu items for a file
  const getFileContextMenuItems = (file: typeof fileManager.activeFile): ContextMenuItem[] => {
    if (!file) return [];
    const f = file();
    if (!f) return [];
    
    return [
      { id: "open", label: "Open", icon: "📂", onSelect: () => fileManager.setActiveFile(f) },
      { id: "sep1", label: "", separator: true },
      { id: "hash", label: "Compute Hash", icon: "🔐", shortcut: "cmd+h", onSelect: () => hashManager.hashSingleFile(f) },
      { id: "verify", label: "Verify Segments", icon: "✓", shortcut: "cmd+shift+h", onSelect: () => hashManager.verifySegments(f) },
      { id: "sep2", label: "", separator: true },
      { id: "select", label: fileManager.selectedFiles().has(f.path) ? "Deselect" : "Select", icon: fileManager.selectedFiles().has(f.path) ? "☐" : "☑", onSelect: () => fileManager.toggleFileSelection(f.path) },
      { id: "sep3", label: "", separator: true },
      { id: "copy-path", label: "Copy Path", icon: "📋", onSelect: () => {
        navigator.clipboard.writeText(f.path);
        toast.success("Path copied to clipboard");
      }},
      { id: "copy-name", label: "Copy Name", icon: "📋", onSelect: () => {
        const name = f.path.split("/").pop() || f.path;
        navigator.clipboard.writeText(name);
        toast.success("Name copied to clipboard");
      }},
    ];
  };
  
  // =========================================================================
  // Helper Functions (extracted to reduce duplication)
  // =========================================================================
  
  /**
   * Build the save options object for project save operations.
   * Used by auto-save, Cmd+S, and manual save button.
   */
  const buildSaveOptions = () => {
    const scanDir = fileManager.scanDir();
    if (!scanDir) return null;
    
    return {
      rootPath: scanDir,
      openTabs: openTabs(),
      activeTabPath: fileManager.activeFile()?.path || null,
      hashHistory: hashManager.hashHistory(),
      processedDatabases: processedDbManager.databases(),
      selectedProcessedDb: processedDbManager.selectedDatabase(),
      uiState: {
        left_panel_width: leftWidth(),
        right_panel_width: rightWidth(),
        left_panel_collapsed: leftCollapsed(),
        right_panel_collapsed: rightCollapsed(),
        left_panel_tab: leftPanelTab(),
        detail_view_mode: currentViewMode(),
      },
      evidenceCache: {
        discoveredFiles: fileManager.discoveredFiles(),
        fileInfoMap: fileManager.fileInfoMap(),
        fileHashMap: hashManager.fileHashMap(),
      },
      processedDbCache: {
        databases: processedDbManager.databases(),
        axiomCaseInfo: processedDbManager.axiomCaseInfo(),
        artifactCategories: processedDbManager.artifactCategories(),
      },
      caseDocumentsCache: caseDocuments() ? {
        documents: caseDocuments()!,
        searchPath: fileManager.scanDir() || '',
      } : undefined,
    };
  };
  
  /**
   * Create a SelectedEntry from a CaseDocument for viewing.
   */
  const createDocumentEntry = (doc: CaseDocument, isDiskFile = true): SelectedEntry => ({
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
  });
  
  /**
   * Handle loading a project file and restoring all state.
   */
  const handleLoadProject = async () => {
    try {
      const result = await projectManager.loadProject();
      if (!result.project) return;
      
      const project = result.project;
      
      // 1. Set scan directory
      fileManager.setScanDir(project.root_path);
      
      // 2. Restore evidence cache if available (skip scanning)
      const cache = project.evidence_cache;
      if (cache && cache.valid && cache.discovered_files.length > 0) {
        console.log("[Project Load] Using cached evidence state");
        
        fileManager.restoreDiscoveredFiles(cache.discovered_files as DiscoveredFile[]);
        
        if (cache.file_info && Object.keys(cache.file_info).length > 0) {
          fileManager.restoreFileInfoMap(cache.file_info as Record<string, import("./types").ContainerInfo>);
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
      
      // 3. Restore UI state
      if (project.ui_state) {
        const ui = project.ui_state;
        if (ui.left_panel_width) setLeftWidth(ui.left_panel_width);
        if (ui.right_panel_width) setRightWidth(ui.right_panel_width);
        if (ui.left_panel_collapsed !== undefined) setLeftCollapsed(ui.left_panel_collapsed);
        if (ui.right_panel_collapsed !== undefined) setRightCollapsed(ui.right_panel_collapsed);
        if (ui.left_panel_tab) setLeftPanelTab(ui.left_panel_tab);
        if (ui.detail_view_mode) setCurrentViewMode(ui.detail_view_mode as TabViewMode);
      }
      
      // 4. Restore tabs from discovered files
      if (project.tabs && project.tabs.length > 0) {
        const discoveredFiles = fileManager.discoveredFiles();
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
      
      // 5. Restore hash history
      if (project.hash_history?.files && Object.keys(project.hash_history.files).length > 0) {
        hashManager.restoreHashHistory(project.hash_history.files);
      }
      
      // 6. Restore processed databases
      if (project.processed_databases) {
        const pd = project.processed_databases;
        
        if (pd.cached_databases && pd.cached_databases.length > 0) {
          processedDbManager.restoreFullState(
            pd.cached_databases,
            pd.selected_path,
            pd.cached_axiom_case_info as Record<string, import("./types").AxiomCaseInfo> | undefined,
            pd.cached_artifact_categories as Record<string, import("./types").ArtifactCategorySummary[]> | undefined,
            pd.detail_view_type
          );
          console.log(`  - Restored ${pd.cached_databases.length} processed databases from cache`);
        } else if (pd.loaded_paths && pd.loaded_paths.length > 0) {
          await processedDbManager.restoreFromProject(
            pd.loaded_paths,
            pd.selected_path,
            pd.cached_metadata
          );
        }
      }
      
      // 7. Restore case documents cache if available
      const docsCache = project.case_documents_cache;
      if (docsCache && docsCache.valid && docsCache.documents && docsCache.documents.length > 0) {
        setCaseDocuments(docsCache.documents as CaseDocument[]);
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
  };
  
  // Command palette actions
  const commandPaletteActions = (): CommandAction[] => [
    // File operations
    { id: "browse", label: "Browse Folder", icon: <HiOutlineFolderOpen class="w-4 h-4" />, category: "File", shortcut: "cmd+o", onSelect: () => fileManager.browseScanDir() },
    { id: "scan", label: "Scan for Files", icon: <HiOutlineArrowPath class="w-4 h-4" />, category: "File", shortcut: "cmd+r", onSelect: () => fileManager.scanForFiles() },
    { id: "report", label: "Generate Report", icon: <HiOutlineDocumentText class="w-4 h-4" />, category: "File", shortcut: "cmd+p", onSelect: () => setShowReportWizard(true) },
    
    // View operations  
    { id: "view-info", label: "Show Info View", icon: <HiOutlineInformationCircle class="w-4 h-4" />, category: "View", shortcut: "cmd+1", onSelect: () => setCurrentViewMode("info") },
    { id: "view-hex", label: "Show Hex View", icon: <HiOutlineCodeBracket class="w-4 h-4" />, category: "View", shortcut: "cmd+2", onSelect: () => setCurrentViewMode("hex") },
    { id: "view-text", label: "Show Text View", icon: <HiOutlineDocument class="w-4 h-4" />, category: "View", shortcut: "cmd+3", onSelect: () => setCurrentViewMode("text") },
    { id: "toggle-left", label: "Toggle Left Panel", icon: <HiOutlineChevronLeft class="w-4 h-4" />, category: "View", shortcut: "cmd+b", onSelect: () => setLeftCollapsed(v => !v) },
    { id: "toggle-right", label: "Toggle Right Panel", icon: <HiOutlineChevronRight class="w-4 h-4" />, category: "View", shortcut: "cmd+shift+b", onSelect: () => setRightCollapsed(v => !v) },
    
    // Hash operations
    { id: "hash-compute", label: "Compute Hash", icon: <HiOutlineLockClosed class="w-4 h-4" />, category: "Hash", shortcut: "cmd+h", onSelect: () => {
      const active = fileManager.activeFile();
      if (active) hashManager.hashSingleFile(active);
    }, disabled: !fileManager.activeFile() },
    { id: "hash-verify", label: "Verify Segments", icon: <HiOutlineCheckBadge class="w-4 h-4" />, category: "Hash", shortcut: "cmd+shift+h", onSelect: () => {
      const active = fileManager.activeFile();
      if (active) hashManager.verifySegments(active);
    }, disabled: !fileManager.activeFile() },
    
    // Settings & Help
    { id: "settings", label: "Settings", icon: <HiOutlineCog6Tooth class="w-4 h-4" />, category: "Settings", shortcut: "cmd+,", onSelect: () => setShowSettingsPanel(true) },
    { id: "shortcuts", label: "Keyboard Shortcuts", icon: <HiOutlineCommandLine class="w-4 h-4" />, category: "Help", shortcut: "?", onSelect: () => setShowShortcutsModal(true) },
    { id: "project-setup", label: "Project Setup", icon: <HiOutlineCog6Tooth class="w-4 h-4" />, category: "Project", onSelect: () => setShowProjectWizard(true) },
    { id: "search", label: "Search Files", icon: <HiOutlineMagnifyingGlass class="w-4 h-4" />, category: "Search", shortcut: "cmd+f", onSelect: () => setShowSearchPanel(true) },
    
    // Developer tools
    { id: "dev-performance", label: "Performance Monitor", icon: <HiOutlineChartBar class="w-4 h-4" />, category: "Developer", shortcut: "ctrl+shift+p", onSelect: () => setShowPerformancePanel(v => !v) },
  ];

  // Initialize/update database session when scan directory changes (debounced)
  let sessionInitTimer: ReturnType<typeof setTimeout> | null = null;
  createEffect(() => {
    const scanDir = fileManager.scanDir();
    if (scanDir && scanDir.length > 0) {
      // Debounce to avoid multiple rapid inits
      if (sessionInitTimer) clearTimeout(sessionInitTimer);
      sessionInitTimer = setTimeout(() => {
        db.initSession(scanDir)
          .then(() => console.log(`Database session initialized for: ${scanDir}`))
          .catch((e) => console.warn("Failed to initialize database session:", e));
      }, 500);
    }
  });
  
  // Save discovered files to database when they change (batched, debounced)
  let fileSaveTimer: ReturnType<typeof setTimeout> | null = null;
  createEffect(() => {
    const files = fileManager.discoveredFiles();
    const session = db.session();
    if (!session || files.length === 0) return;
    
    // Debounce to batch multiple rapid file additions
    if (fileSaveTimer) clearTimeout(fileSaveTimer);
    fileSaveTimer = setTimeout(() => {
      // Save files in background - don't await in effect
      Promise.all(
        files.map(file => 
          db.saveFile(file).catch(e => 
            console.warn(`Failed to save file: ${file.path}`, e)
          )
        )
      ).then(() => {
        console.log(`Saved ${files.length} files to database`);
      });
    }, 1000); // Wait 1 second after last file change
  });

  // ===========================================================================
  // Lifecycle - Mount & Cleanup
  // ===========================================================================
  
  let cleanupSystemStats: (() => void) | undefined;
  const handleResize = () => setWindowWidth(window.innerWidth);

  onMount(async () => {
    // System stats listener
    const unlisten = await fileManager.setupSystemStatsListener();
    cleanupSystemStats = unlisten;
    
    // Window resize handling
    window.addEventListener('resize', handleResize);
    
    // Auto-save callback
    projectManager.setAutoSaveCallback(async () => {
      const options = buildSaveOptions();
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
    
    // Global keyboard shortcuts
    window.addEventListener("keydown", handleGlobalKeyDown);
  });

  onCleanup(() => {
    cleanupSystemStats?.();
    if (sessionInitTimer) clearTimeout(sessionInitTimer);
    if (fileSaveTimer) clearTimeout(fileSaveTimer);
    window.removeEventListener('resize', handleResize);
    window.removeEventListener("keydown", handleGlobalKeyDown);
    projectManager.stopAutoSave();
  });

  // Handler for opening a project directory - shows the setup wizard
  const handleOpenDirectory = async () => {
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
        source: "App.handleOpenDirectory" 
      });
      toast.error("Failed to Open", "Could not open directory dialog");
    }
  };

  // Handler for when project setup wizard completes
  const handleProjectSetupComplete = async (locations: ProjectLocations) => {
    setShowProjectWizard(false);
    
    // Set the evidence path and scan for files
    fileManager.setScanDir(locations.evidencePath);
    await fileManager.scanForFiles(locations.evidencePath);
    
    // Store case documents path for CaseDocumentsPanel
    setCaseDocumentsPath(locations.caseDocumentsPath || locations.evidencePath);
    
    // If processed databases were discovered, add them
    if (locations.discoveredDatabases.length > 0) {
      // Switch to processed tab if databases found
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
    logInfo("Project setup complete", { source: "App.handleProjectSetupComplete", context: { locations } });
    toast.success("Project Ready", `Found ${fileManager.discoveredFiles().length} files`);
    announce(`Project setup complete. Found ${fileManager.discoveredFiles().length} evidence files.`);
    
    setPendingProjectRoot(null);
  };
  
  /**
   * Global keyboard shortcut handler.
   */
  const handleGlobalKeyDown = (e: KeyboardEvent) => {
    const meta = e.metaKey || e.ctrlKey;
    const inInput = ["INPUT", "TEXTAREA"].includes((e.target as HTMLElement)?.tagName);
    
    // Cmd+K: Command palette
    if (meta && e.key === "k") {
      e.preventDefault();
      setShowCommandPalette(v => !v);
      return;
    }
    
    // Cmd+,: Settings
    if (meta && e.key === ",") {
      e.preventDefault();
      setShowSettingsPanel(true);
      return;
    }
    
    // Cmd+F: Search
    if (meta && e.key === "f") {
      e.preventDefault();
      setShowSearchPanel(true);
      return;
    }
    
    // Ctrl+Shift+P: Performance panel (dev)
    if (e.ctrlKey && e.shiftKey && e.key === "P") {
      e.preventDefault();
      setShowPerformancePanel(v => !v);
      return;
    }
    
    // ?: Shortcuts help (when not in input)
    if (e.key === "?" && !inInput) {
      e.preventDefault();
      setShowShortcutsModal(true);
      return;
    }
    
    // Cmd+Z: Undo
    if (meta && e.key === "z" && !e.shiftKey) {
      e.preventDefault();
      if (history.state.canUndo()) {
        history.actions.undo();
        const desc = history.state.undoDescription() || "Action undone";
        toast.info("Undo", desc);
        announce(`Undo: ${desc}`);
      }
      return;
    }
    
    // Cmd+Shift+Z or Cmd+Y: Redo
    if (meta && ((e.key === "z" && e.shiftKey) || e.key === "y")) {
      e.preventDefault();
      if (history.state.canRedo()) {
        history.actions.redo();
        const desc = history.state.redoDescription() || "Action redone";
        toast.info("Redo", desc);
        announce(`Redo: ${desc}`);
      }
      return;
    }
    
    // Cmd+S: Save
    if (meta && e.key === "s") {
      e.preventDefault();
      const options = buildSaveOptions();
      if (options) {
        projectManager.saveProject(options).then(() => {
          toast.success("Saved", "Project saved");
          announce("Project saved");
        }).catch((err) => {
          logError(err instanceof Error ? err : new Error("Save failed"), { source: "keyboard.save" });
          toast.error("Save Failed", "Could not save project");
        });
      }
      return;
    }
    
    // Escape: Close modals
    if (e.key === "Escape") {
      if (showCommandPalette()) { setShowCommandPalette(false); return; }
      if (showShortcutsModal()) { setShowShortcutsModal(false); return; }
    }
  };

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
      
      {/* Command Palette */}
      <CommandPalette
        actions={commandPaletteActions()}
        isOpen={showCommandPalette()}
        onClose={() => setShowCommandPalette(false)}
        placeholder="Search commands..."
      />
      
      {/* Keyboard Shortcuts Modal */}
      <KeyboardShortcutsModal
        isOpen={showShortcutsModal()}
        onClose={() => setShowShortcutsModal(false)}
        groups={DEFAULT_SHORTCUT_GROUPS}
      />
      
      {/* Performance Panel (dev mode) */}
      <PerformancePanel
        isOpen={showPerformancePanel()}
        onClose={() => setShowPerformancePanel(false)}
      />
      
      {/* Settings Panel */}
      <SettingsPanel
        isOpen={showSettingsPanel()}
        onClose={() => setShowSettingsPanel(false)}
        preferences={preferences.preferences()}
        onUpdatePreference={(key, value) => preferences.updatePreference(key, value)}
        onUpdateShortcut={(action, shortcut) => preferences.updateShortcut(action, shortcut)}
        onResetToDefaults={preferences.resetToDefaults}
      />
      
      {/* Search Panel */}
      <SearchPanel
        isOpen={showSearchPanel()}
        onClose={() => setShowSearchPanel(false)}
        onSearch={handleSearch}
        onSelectResult={handleSearchResultSelect}
        placeholder="Search files and container contents..."
      />
      
      {/* File Context Menu */}
      <ContextMenu
        items={fileContextMenu.items()}
        position={fileContextMenu.position()}
        onClose={fileContextMenu.close}
      />
      
      {/* Welcome Modal (first-time users) */}
      <WelcomeModal
        isOpen={showWelcomeModal()}
        onClose={() => {
          setShowWelcomeModal(false);
          localStorage.setItem("ffx-welcome-seen", "true");
        }}
        onStartTour={() => {
          setShowWelcomeModal(false);
          localStorage.setItem("ffx-welcome-seen", "true");
          tour.start();
        }}
      />
      
      {/* Tour Overlay (guided onboarding) */}
      <TourOverlay
        isActive={tour.isActive()}
        step={tour.currentStep()}
        stepIndex={tour.currentStepIndex()}
        totalSteps={DEFAULT_TOUR_STEPS.length}
        progress={tour.progress()}
        isFirst={tour.isFirstStep()}
        isLast={tour.isLastStep()}
        onNext={tour.next}
        onPrevious={tour.previous}
        onSkip={tour.skip}
      />
      
      {/* Project Setup Wizard */}
      <ProjectSetupWizard
        projectRoot={pendingProjectRoot() || ''}
        isOpen={showProjectWizard()}
        onClose={() => {
          setShowProjectWizard(false);
          setPendingProjectRoot(null);
        }}
        onComplete={handleProjectSetupComplete}
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
            <div class="flex flex-col items-center gap-1.5 py-2 px-2.5 bg-bg-secondary border-r border-border h-full">
              {/* Project/Data Icons - Top Section */}
              <button 
                class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${leftPanelTab() === "evidence" ? "bg-accent text-white" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
                onClick={() => setLeftPanelTab("evidence")}
                title="Evidence Containers (E01, AD1, L01, etc.)"
              >
                <HiOutlineArchiveBox class="w-4 h-4" />
              </button>
              <button 
                class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${leftPanelTab() === "processed" ? "bg-accent text-white" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
                onClick={() => setLeftPanelTab("processed")}
                title="Processed Databases (AXIOM, Cellebrite PA, etc.)"
              >
                <HiOutlineChartBar class="w-4 h-4" />
              </button>
              <button 
                class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${leftPanelTab() === "casedocs" ? "bg-accent text-white" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
                onClick={() => setLeftPanelTab("casedocs")}
                title="Case Documents (Chain of Custody, Intake Forms, etc.)"
              >
                <HiOutlineClipboardDocumentList class="w-4 h-4" />
              </button>
              <button 
                class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${leftPanelTab() === "activity" ? "bg-accent text-white" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
                onClick={() => setLeftPanelTab("activity")}
                title="Activity Log & Sessions"
              >
                <HiOutlineClock class="w-4 h-4" />
              </button>
              
              {/* Spacer to push action and utility icons to bottom */}
              <div class="flex-1" />
              
              {/* Project Actions - Above Utilities */}
              <div class="flex flex-col items-center gap-1.5 pb-2 border-b border-border/50">
                <button 
                  class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${projectManager.modified() ? "text-warning" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
                  onClick={async () => {
                    const options = buildSaveOptions();
                    if (options) {
                      try {
                        await projectManager.saveProject(options);
                        toast.success("Project Saved", "Your project has been saved");
                      } catch (err) {
                        toast.error("Save Failed", "Could not save the project");
                      }
                    }
                  }}
                  disabled={fileManager.busy() || !fileManager.scanDir()}
                  title={projectManager.modified() ? "Save Project (unsaved changes)" : "Save Project (⌘S)"}
                >
                  <HiOutlineDocumentArrowDown class="w-4 h-4" />
                </button>
                <button 
                  class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover"
                  onClick={handleLoadProject}
                  disabled={fileManager.busy()}
                  title="Load Project (⌘O)"
                >
                  <HiOutlineFolderArrowDown class="w-4 h-4" />
                </button>
                <button 
                  class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover relative"
                  onClick={() => setRequestViewMode("export")}
                  disabled={fileManager.busy()}
                  title="Export/Transfer Files"
                >
                  <HiOutlineArrowUpTray class="w-4 h-4" />
                  <Show when={transferJobs().filter(j => j.status === "running" || j.status === "pending").length > 0}>
                    <span class="absolute -top-1 -right-1 flex items-center justify-center min-w-[14px] h-3.5 px-0.5 text-[9px] leading-tight font-bold text-white bg-accent rounded-full animate-pulse">
                      {transferJobs().filter(j => j.status === "running" || j.status === "pending").length}
                    </span>
                  </Show>
                </button>
                <button 
                  class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover"
                  onClick={() => setShowReportWizard(true)}
                  disabled={fileManager.busy() || fileManager.discoveredFiles().length === 0}
                  title="Generate Report (⌘P)"
                >
                  <HiOutlineDocumentText class="w-4 h-4" />
                </button>
              </div>
              
              {/* Utility Icons - Bottom Section */}
              <div class="flex flex-col items-center gap-1.5 pt-2">
                <button 
                  class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover"
                  onClick={() => setShowSearchPanel(true)}
                  title="Search (⌘F)"
                >
                  <HiOutlineMagnifyingGlass class="w-4 h-4" />
                </button>
                <ThemeSwitcher 
                  compact 
                  theme={themeActions.theme}
                  resolvedTheme={themeActions.resolvedTheme}
                  cycleTheme={themeActions.cycleTheme}
                />
                <button 
                  class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover"
                  onClick={() => setShowSettingsPanel(true)}
                  title="Settings (⌘,)"
                >
                  <HiOutlineCog6Tooth class="w-4 h-4" />
                </button>
              </div>
            </div>
            
            {/* Panel Content Area */}
            <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
            {/* Tab Content - Using CSS visibility to preserve state across tab switches */}
            <div class={`flex-1 flex flex-col min-w-0 overflow-hidden ${leftPanelTab() === "evidence" ? "" : "hidden"}`}>
              {/* Unified Evidence Tree with selection and hashing */}
              <EvidenceTree
                discoveredFiles={fileManager.discoveredFiles()}
                activeFile={fileManager.activeFile()}
                busy={fileManager.busy()}
                onSelectContainer={(file) => fileManager.selectAndViewFile(file)}
                onSelectEntry={(entry) => {
                  setSelectedContainerEntry(entry);
                  // Switch to hex view to show the entry content
                  setCurrentViewMode("hex");
                  console.log('Selected entry:', entry.entryPath, 'from', entry.containerPath);
                }}
                typeFilter={fileManager.typeFilter()}
                onToggleTypeFilter={(type) => fileManager.toggleTypeFilter(type)}
                onClearTypeFilter={() => fileManager.setTypeFilter(null)}
                containerStats={fileManager.containerStats()}
                onOpenNestedContainer={(tempPath, originalName, containerType, parentPath) => {
                  // Create a DiscoveredFile for the nested container
                  const nestedFile: DiscoveredFile = {
                    path: tempPath,
                    filename: `📦 ${originalName} (from ${parentPath.split('/').pop() || parentPath})`,
                    container_type: containerType.toUpperCase(),
                    size: 0, // Size will be determined when opened
                    segment_count: 1,
                  };
                  fileManager.addDiscoveredFile(nestedFile);
                  // Automatically open the nested container
                  fileManager.selectAndViewFile(nestedFile);
                  toast.success("Nested Container", `Opened ${originalName}`);
                }}
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
                onSelectDatabase={(db) => {
                  processedDbManager.selectDatabase(db);
                  // Clear active forensic file when switching to processed view
                  fileManager.setActiveFile(null);
                }}
                onSelectArtifact={(db, artifact) => console.log('Selected artifact:', artifact.name, 'from', db.path)}
              />
            </div>

            {/* Case Documents Tab */}
            <Show when={leftPanelTab() === "casedocs"}>
              <CaseDocumentsPanel 
                evidencePath={caseDocumentsPath() || fileManager.activeFile()?.path || projectManager.project()?.locations?.case_documents_path || projectManager.project()?.locations?.evidence_path}
                onDocumentSelect={(doc) => {
                  setSelectedContainerEntry(createDocumentEntry(doc));
                  
                  // If it's a PDF, also set up for PDF viewing
                  if (doc.path.toLowerCase().endsWith('.pdf')) {
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
                    setEntryContentViewMode("hex");
                  }
                }}
                onViewHex={(doc) => {
                  setSelectedContainerEntry(createDocumentEntry(doc));
                  setEntryContentViewMode("hex");
                }}
                onViewText={(doc) => {
                  setSelectedContainerEntry(createDocumentEntry(doc));
                  setEntryContentViewMode("text");
                }}
                cachedDocuments={caseDocuments() ?? undefined}
                onDocumentsLoaded={(docs, _searchPath) => setCaseDocuments(docs)}
              />
            </Show>
            
            {/* Activity Panel */}
            <Show when={leftPanelTab() === "activity"}>
              <ActivityPanel project={projectManager.project()} />
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

        {/* Center Panel */}
        <section class="center-panel" id="main-content">
          {/* Show ContainerEntryViewer when viewing a file inside a container */}
          <Show when={selectedContainerEntry() && !selectedContainerEntry()!.isDir} fallback={
            /* Show ProcessedDetailPanel when viewing processed databases */
            <Show when={leftPanelTab() === "processed" && processedDbManager.selectedDatabase()} fallback={
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
                onTabSelect={(file) => fileManager.setActiveFile(file)}
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
                  // Add transfer hashes to the hash history
                  hashManager.addTransferHashesToHistory(entries);
                }}
                onTransferProgressUpdate={setTransferJobs}
                transferJobs={transferJobs()}
                onTransferJobsChange={setTransferJobs}
              />
            }>
              <ProcessedDetailPanel
                database={processedDbManager.selectedDatabase()}
                caseInfo={processedDbManager.selectedCaseInfo()}
                categories={processedDbManager.selectedCategories()}
                loading={processedDbManager.isSelectedLoading()}
                detailView={processedDbManager.detailView()}
                onDetailViewChange={(view) => processedDbManager.setDetailView(view)}
              />
            </Show>
          }>
            <ContainerEntryViewer
              entry={selectedContainerEntry()!}
              viewMode={entryContentViewMode()}
              onBack={() => setSelectedContainerEntry(null)}
              onViewModeChange={setEntryContentViewMode}
            />
          </Show>
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
        <Show when={!rightCollapsed()}>
          <aside class="right-panel" style={{ width: `${rightWidth()}px` }}>
            {/* Transfer indicator banner - shows when transfers active but not in export view */}
            <Show when={currentViewMode() !== "export" && transferJobs().some(j => j.status === "running" || j.status === "pending")}>
              <button
                onClick={() => setRequestViewMode("export")}
                class="w-full flex items-center justify-between gap-2 px-3 py-1.5 bg-accent/20 hover:bg-accent/30 border-b border-accent/30 text-accent text-xs cursor-pointer transition-colors"
                title="Click to view transfer details"
              >
                <span class="flex items-center gap-1.5">
                  <span class="w-2 h-2 bg-accent rounded-full animate-pulse" />
                  <span class="font-medium">
                    {transferJobs().filter(j => j.status === "running").length} transfer{transferJobs().filter(j => j.status === "running").length !== 1 ? 's' : ''} running
                  </span>
                </span>
                <span class="text-accent/70">View →</span>
              </button>
            </Show>
            
            <Show when={currentViewMode() === "export"}>
              <TransferProgressPanel 
                jobs={transferJobs()} 
                onCancelJob={async (jobId) => {
                  try {
                    await transferCancel(jobId);
                    setTransferJobs(jobs => jobs.map(job =>
                      job.id === jobId ? { ...job, status: "cancelled" } : job
                    ));
                  } catch (err) {
                    console.error("Failed to cancel transfer:", err);
                  }
                }}
              />
            </Show>
            <Show when={currentViewMode() === "hex"}>
              <MetadataPanel 
                metadata={hexMetadata()}
                containerInfo={activeFileInfo()}
                fileInfo={fileManager.activeFile() ? {
                  path: fileManager.activeFile()!.path,
                  filename: fileManager.activeFile()!.filename,
                  size: fileManager.activeFile()!.size,
                  created: fileManager.activeFile()!.created,
                  modified: fileManager.activeFile()!.modified,
                  container_type: fileManager.activeFile()!.container_type,
                  segment_count: fileManager.activeFile()!.segment_count
                } : null}
                selectedEntry={selectedContainerEntry()}
                onRegionClick={(offset) => {
                  // Request DetailPanel to switch to hex view mode
                  setRequestViewMode("hex");
                  
                  // Retry function to wait for HexViewer to mount
                  const tryNavigate = (attempts: number) => {
                    const nav = hexNavigator();
                    if (nav) {
                      nav(offset);
                    } else if (attempts > 0) {
                      // HexViewer not mounted yet, retry
                      setTimeout(() => tryNavigate(attempts - 1), 100);
                    }
                  };
                  
                  // Start retrying after a small delay for view mode to switch
                  setTimeout(() => tryNavigate(5), 100);
                }}
              />
            </Show>
            <Show when={currentViewMode() !== "hex" && currentViewMode() !== "export"}>
              <TreePanel info={activeFileInfo()} />
            </Show>
          </aside>
        </Show>
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
