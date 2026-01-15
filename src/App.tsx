// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { onMount, onCleanup, createSignal, createEffect, Show } from "solid-js";
import { useFileManager, useHashManager, useDatabase, useProject, useProcessedDatabases, useHistoryContext } from "./hooks";
import { useDualPanelResize } from "./hooks/usePanelResize";
import { Toolbar, StatusBar, FilePanel, DetailPanel, TreePanel, ProgressModal, MetadataPanel, ReportWizard, ProjectSetupWizard, EvidenceTree, ContainerEntryViewer, CommandPalette, KeyboardShortcutsModal, DEFAULT_SHORTCUT_GROUPS, useToast, ThemeSwitcher, pathToBreadcrumbs, SearchPanel, ContextMenu, createContextMenu, WelcomeModal, useTour, TourOverlay, DEFAULT_TOUR_STEPS, useDragDrop, CaseDocumentsPanel } from "./components";
import type { ProjectLocations, SelectedEntry, OpenTab, CommandAction, SearchFilter, SearchResult, ContextMenuItem } from "./components";
import type { DiscoveredFile } from "./types";
import ProcessedDatabasePanel from "./components/ProcessedDatabasePanel";
import ProcessedDetailPanel from "./components/ProcessedDetailPanel";
import PerformancePanel from "./components/PerformancePanel";
import SettingsPanel, { createPreferences } from "./components/SettingsPanel";
import type { ParsedMetadata, TabViewMode } from "./components";
import { announce } from "./utils/accessibility";
import { logError, logInfo } from "./utils/telemetry";
import ffxLogo from "./assets/branding/ffx-logo-48.png";
import "./App.css";
import {
  PANEL_TABS_CLASSES,
  PANEL_TAB_BASE,
  PANEL_TAB_ACTIVE,
  PANEL_TAB_INACTIVE,
  UI_ICON_COMPACT,
} from "./components/ui/constants";
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
  HiOutlineListBullet,
  HiOutlineRectangleStack,
  HiOutlineMagnifyingGlass,
  HiOutlineClipboardDocumentList,
} from "solid-icons/hi";

function App() {
  // Initialize toast notifications
  const toast = useToast();
  
  // Initialize undo/redo history
  const history = useHistoryContext();
  
  // Initialize preferences/settings
  const preferences = createPreferences();
  
  // Initialize database hook
  const db = useDatabase();
  
  // Initialize file manager hook
  const fileManager = useFileManager();
  
  // Initialize hash manager hook (depends on file manager)
  const hashManager = useHashManager(fileManager);
  
  // Initialize project management hook
  const projectManager = useProject();
  
  // Initialize processed databases hook (for AXIOM, Cellebrite, etc.)
  const processedDbManager = useProcessedDatabases();
  
  // Track open tabs (from DetailPanel) for project save
  const [openTabs, setOpenTabs] = createSignal<OpenTab[]>([]);
  
  // Track current view mode for right panel switching
  const [currentViewMode, setCurrentViewMode] = createSignal<TabViewMode>("info");
  
  // Track hex viewer metadata for MetadataPanel
  const [hexMetadata, setHexMetadata] = createSignal<ParsedMetadata | null>(null);
  
  // Track selected container entry for viewing content from within containers
  const [selectedContainerEntry, setSelectedContainerEntry] = createSignal<SelectedEntry | null>(null);
  
  // Evidence view mode: "list" for flat file list, "tree" for hierarchical tree
  const [evidenceViewMode, setEvidenceViewMode] = createSignal<"list" | "tree">("tree");
  
  // Container entry content view mode: "hex" or "text"
  const [entryContentViewMode, setEntryContentViewMode] = createSignal<"hex" | "text">("hex");
  
  // Clear metadata when active file changes
  createEffect(() => {
    void fileManager.activeFile(); // Track active file
    // Clear hex metadata when file changes - new file needs fresh parsing
    setHexMetadata(null);
    // Also reset view mode to info when switching files
    setCurrentViewMode("info");
    // Clear selected container entry when switching files
    setSelectedContainerEntry(null);
  });
  
  // Store hex viewer navigation function
  const [hexNavigator, setHexNavigator] = createSignal<((offset: number, size?: number) => void) | null>(null);
  
  // Wrapper to set navigator
  const handleHexNavigatorReady = (nav: (offset: number, size?: number) => void) => {
    setHexNavigator(() => nav);
  };
  
  // Request view mode change (for MetadataPanel navigation and PDF viewing)
  const [requestViewMode, setRequestViewMode] = createSignal<"info" | "hex" | "text" | "pdf" | null>(null);
  
  // Report wizard state
  const [showReportWizard, setShowReportWizard] = createSignal(false);
  
  // Project Setup Wizard state
  const [showProjectWizard, setShowProjectWizard] = createSignal(false);
  const [pendingProjectRoot, setPendingProjectRoot] = createSignal<string | null>(null);
  
  // Left panel tab state: "evidence", "processed", or "casedocs"
  const [leftPanelTab, setLeftPanelTab] = createSignal<"evidence" | "processed" | "casedocs">("evidence");
  
  // Resizable panel state - use the dual panel resize hook
  const panels = useDualPanelResize({
    left: { initialWidth: 320, minWidth: 150, maxWidth: 600, startCollapsed: false },
    right: { initialWidth: 280, minWidth: 150, maxWidth: 500, startCollapsed: true },
  });
  // Aliases for convenience
  const leftWidth = panels.left.width;
  const rightWidth = panels.right.width;
  const leftCollapsed = panels.left.collapsed;
  const rightCollapsed = panels.right.collapsed;
  const setLeftWidth = panels.left.setWidth;
  const setRightWidth = panels.right.setWidth;
  const setLeftCollapsed = panels.left.setCollapsed;
  const setRightCollapsed = panels.right.setCollapsed;
  
  // Responsive: track window width for compact toolbar
  const [windowWidth, setWindowWidth] = createSignal(window.innerWidth);
  const isCompact = () => windowWidth() < 900;
  
  // Command Palette state
  const [showCommandPalette, setShowCommandPalette] = createSignal(false);
  
  // Keyboard Shortcuts Modal state
  const [showShortcutsModal, setShowShortcutsModal] = createSignal(false);
  
  // Performance Panel state (dev mode only)
  const [showPerformancePanel, setShowPerformancePanel] = createSignal(false);
  
  // Settings Panel state
  const [showSettingsPanel, setShowSettingsPanel] = createSignal(false);
  
  // Search Panel state
  const [showSearchPanel, setShowSearchPanel] = createSignal(false);
  
  // Welcome/Onboarding state
  const [showWelcomeModal, setShowWelcomeModal] = createSignal(false);
  
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
  
  // Check if first launch and show welcome modal
  onMount(() => {
    const hasSeenWelcome = localStorage.getItem("ffx-welcome-seen");
    if (!hasSeenWelcome && !tour.hasCompleted()) {
      // Delay slightly to let the UI settle
      setTimeout(() => setShowWelcomeModal(true), 500);
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
  
  // Breadcrumb items for current path
  const breadcrumbItems = () => {
    const activeFile = fileManager.activeFile();
    if (!activeFile) return [];
    return pathToBreadcrumbs(activeFile.path);
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

  // Store cleanup function reference
  let cleanupSystemStats: (() => void) | undefined;
  
  // Handle window resize for responsive toolbar
  const handleResize = () => setWindowWidth(window.innerWidth);

  onMount(async () => {
    const unlisten = await fileManager.setupSystemStatsListener();
    cleanupSystemStats = unlisten;
    window.addEventListener('resize', handleResize);
    
    // Set up auto-save callback with current state
    projectManager.setAutoSaveCallback(async () => {
      const scanDir = fileManager.scanDir();
      if (!scanDir) return;
      
      await projectManager.saveProject({
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
      });
    });
    
    // Try to restore last session (non-blocking)
    db.restoreLastSession()
      .then((lastSession) => {
        if (lastSession) {
          // Only restore scan directory, don't trigger a scan
          fileManager.setScanDir(lastSession.root_path);
          console.log(`Restored session: ${lastSession.name} (${lastSession.root_path})`);
        }
      })
      .catch((e) => console.warn("Failed to restore last session:", e));
  });

  onCleanup(() => {
    cleanupSystemStats?.();
    if (sessionInitTimer) clearTimeout(sessionInitTimer);
    if (fileSaveTimer) clearTimeout(fileSaveTimer);
    window.removeEventListener('resize', handleResize);
    
    // Stop auto-save on cleanup
    projectManager.stopAutoSave();
  });

  // Helper for TreePanel - gets info for active file
  const activeFileInfo = () => {
    const active = fileManager.activeFile();
    if (!active) return undefined;
    return fileManager.fileInfoMap().get(active.path);
  };

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
      `Project setup complete: Evidence=${locations.evidencePath}, Processed=${locations.processedDbPath}`);
    logInfo("Project setup complete", { source: "App.handleProjectSetupComplete", context: { locations } });
    toast.success("Project Ready", `Found ${fileManager.discoveredFiles().length} files`);
    announce(`Project setup complete. Found ${fileManager.discoveredFiles().length} evidence files.`);
    
    setPendingProjectRoot(null);
  };
  
  // Global keyboard shortcuts
  onMount(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      // Cmd+K: Open command palette
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setShowCommandPalette(v => !v);
        return;
      }
      
      // Cmd+,: Open settings
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        setShowSettingsPanel(true);
        return;
      }
      
      // Cmd+F: Open search panel
      if ((e.metaKey || e.ctrlKey) && e.key === "f") {
        e.preventDefault();
        setShowSearchPanel(true);
        return;
      }
      
      // Ctrl+Shift+P: Toggle performance panel (dev mode)
      if ((e.ctrlKey) && e.shiftKey && e.key === "P") {
        e.preventDefault();
        setShowPerformancePanel(v => !v);
        return;
      }
      
      // ?: Show keyboard shortcuts (when not in input)
      if (e.key === "?" && !["INPUT", "TEXTAREA"].includes((e.target as HTMLElement)?.tagName)) {
        e.preventDefault();
        setShowShortcutsModal(true);
        return;
      }
      
      // Cmd+Z: Undo
      if ((e.metaKey || e.ctrlKey) && e.key === "z" && !e.shiftKey) {
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
      if ((e.metaKey || e.ctrlKey) && ((e.key === "z" && e.shiftKey) || e.key === "y")) {
        e.preventDefault();
        if (history.state.canRedo()) {
          history.actions.redo();
          const desc = history.state.redoDescription() || "Action redone";
          toast.info("Redo", desc);
          announce(`Redo: ${desc}`);
        }
        return;
      }
      
      // Cmd+S: Save project
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        const scanDir = fileManager.scanDir();
        if (scanDir) {
          projectManager.saveProject({
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
          }).then(() => {
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
        if (showCommandPalette()) {
          setShowCommandPalette(false);
          return;
        }
        if (showShortcutsModal()) {
          setShowShortcutsModal(false);
          return;
        }
      }
    };
    
    window.addEventListener("keydown", handleGlobalKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleGlobalKeyDown));
  });

  return (
    <div ref={appContainerRef} class="app-root" classList={{ 'is-resizing': panels.isDragging() }}>
      {/* Drag overlay */}
      <Show when={dragDrop.isDragging()}>
        <div class="fixed inset-0 z-[1000] bg-zinc-900/90 flex items-center justify-center pointer-events-none">
          <div class={`p-12 rounded-2xl border-2 border-dashed transition-all ${dragDrop.isOver() ? "border-cyan-500 bg-cyan-500/20 scale-105" : "border-zinc-500 bg-zinc-800/50"}`}>
            <div class="text-6xl mb-4 text-center">📂</div>
            <div class="text-xl font-semibold text-zinc-200 text-center">
              {dragDrop.isOver() ? "Release to import" : "Drop evidence files here"}
            </div>
            <div class="text-sm text-zinc-400 text-center mt-2">
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
        onSearch={async (query: string, _filters: SearchFilter): Promise<SearchResult[]> => {
          // Search through discovered files
          const files = fileManager.discoveredFiles();
          const lowerQuery = query.toLowerCase();
          const results: SearchResult[] = [];
          
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
              });
            }
            
            // Limit results
            if (results.length >= 100) break;
          }
          
          // Sort by score (name matches first)
          return results.sort((a, b) => b.score - a.score);
        }}
        onSelectResult={(result: SearchResult) => {
          // Find and select the file
          const file = fileManager.discoveredFiles().find(f => f.path === result.path);
          if (file) {
            fileManager.setActiveFile(file);
            announce(`Selected ${result.name}`);
          }
        }}
        placeholder="Search files by name or path..."
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
        <div class="header-actions">
          <button 
            class="header-btn" 
            onClick={() => setShowSearchPanel(true)}
            title="Search (⌘F)"
          >
            <HiOutlineMagnifyingGlass class="w-4 h-4" />
          </button>
          <ThemeSwitcher compact />
          <button 
            class="header-btn" 
            onClick={() => setShowSettingsPanel(true)}
            title="Settings (⌘,)"
          >
            <HiOutlineCog6Tooth class="w-4 h-4" />
          </button>
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
        // Project management
        projectPath={projectManager.projectPath()}
        projectModified={projectManager.modified()}
        onSaveProject={async () => {
          const scanDir = fileManager.scanDir();
          if (scanDir) {
            const activeTabPath = fileManager.activeFile()?.path || null;
            try {
              await projectManager.saveProject({
                rootPath: scanDir,
                openTabs: openTabs(),
                activeTabPath,
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
              });
              toast.success("Project Saved", "Your project has been saved");
              announce("Project saved successfully");
            } catch (err) {
              logError(err instanceof Error ? err : new Error("Failed to save project"), {
                category: "filesystem",
                source: "App.onSaveProject",
              });
              toast.error("Save Failed", "Could not save the project");
            }
          }
        }}
        onLoadProject={async () => {
          try {
            const result = await projectManager.loadProject();
            if (result.project) {
              // Restore scan directory
              fileManager.setScanDir(result.project.root_path);
              
              // Restore UI state
              if (result.project.ui_state) {
                const ui = result.project.ui_state;
                if (ui.left_panel_width) setLeftWidth(ui.left_panel_width);
                if (ui.right_panel_width) setRightWidth(ui.right_panel_width);
                if (ui.left_panel_collapsed !== undefined) setLeftCollapsed(ui.left_panel_collapsed);
                if (ui.right_panel_collapsed !== undefined) setRightCollapsed(ui.right_panel_collapsed);
                if (ui.left_panel_tab) setLeftPanelTab(ui.left_panel_tab);
                if (ui.detail_view_mode) setCurrentViewMode(ui.detail_view_mode as TabViewMode);
              }
              
              // Log activity
              projectManager.logActivity('project', 'open', `Opened project: ${result.project.name}`);
              toast.success("Project Loaded", `Opened: ${result.project.name}`);
              announce(`Project ${result.project.name} loaded successfully`);
              
              console.log("Project loaded:", result.project.name);
            }
          } catch (err) {
            logError(err instanceof Error ? err : new Error("Failed to load project"), {
              category: "filesystem",
              source: "App.onLoadProject",
            });
            toast.error("Load Failed", "Could not load the project");
          }
        }}
        // Report generation
        onGenerateReport={() => setShowReportWizard(true)}
        // Responsive mode
        compact={isCompact()}
      />

      {/* Main Content Area */}
      <main class="app-main">
        {/* Left Panel */}
        <Show when={!leftCollapsed()}>
          <aside class="left-panel" style={{ width: `${leftWidth()}px` }}>
            {/* Panel Tab Switcher */}
            <div class={PANEL_TABS_CLASSES}>
              <button 
                class={`${PANEL_TAB_BASE} ${leftPanelTab() === "evidence" ? PANEL_TAB_ACTIVE : PANEL_TAB_INACTIVE}`}
                onClick={() => setLeftPanelTab("evidence")}
                title="Evidence Containers (E01, AD1, L01, etc.)"
              >
                <HiOutlineArchiveBox class={UI_ICON_COMPACT} /> Evidence
              </button>
              <button 
                class={`${PANEL_TAB_BASE} ${leftPanelTab() === "processed" ? PANEL_TAB_ACTIVE : PANEL_TAB_INACTIVE}`}
                onClick={() => setLeftPanelTab("processed")}
                title="Processed Databases (AXIOM, Cellebrite PA, etc.)"
              >
                <HiOutlineChartBar class={UI_ICON_COMPACT} /> Processed
              </button>
              <button 
                class={`${PANEL_TAB_BASE} ${leftPanelTab() === "casedocs" ? PANEL_TAB_ACTIVE : PANEL_TAB_INACTIVE}`}
                onClick={() => setLeftPanelTab("casedocs")}
                title="Case Documents (Chain of Custody, Intake Forms, etc.)"
              >
                <HiOutlineClipboardDocumentList class={UI_ICON_COMPACT} /> Case Docs
              </button>
            </div>
            
            {/* Tab Content */}
            <Show when={leftPanelTab() === "evidence"}>
              {/* Evidence View Mode Switcher */}
              <div class="flex items-center gap-0.5 mr-2 bg-zinc-800 rounded border border-zinc-700">
                <button 
                  class="flex items-center px-2 py-1 text-[10px] font-medium bg-transparent border-none text-zinc-400 cursor-pointer transition-all rounded-l"
                  classList={{ "bg-cyan-600 text-white": evidenceViewMode() === "tree" }}
                  onClick={() => setEvidenceViewMode("tree")}
                  title="Tree View - Show containers with internal file structure"
                >
                  <HiOutlineRectangleStack class="w-3.5 h-3.5" />
                </button>
                <button 
                  class="flex items-center px-2 py-1 text-[10px] font-medium bg-transparent border-none text-zinc-400 cursor-pointer transition-all rounded-r"
                  classList={{ "bg-cyan-600 text-white": evidenceViewMode() === "list" }}
                  onClick={() => setEvidenceViewMode("list")}
                  title="List View - Flat file list"
                >
                  <HiOutlineListBullet class="w-3.5 h-3.5" />
                </button>
              </div>
              
              {/* Show either EvidenceTree or FilePanel based on view mode */}
              <Show when={evidenceViewMode() === "tree"} fallback={
                <FilePanel
                  discoveredFiles={fileManager.discoveredFiles()}
                  filteredFiles={fileManager.filteredFiles()}
                  selectedFiles={fileManager.selectedFiles()}
                  activeFile={fileManager.activeFile()}
                  hoveredFile={fileManager.hoveredFile()}
                  focusedFileIndex={fileManager.focusedFileIndex()}
                  typeFilter={fileManager.typeFilter()}
                  containerStats={fileManager.containerStats()}
                  totalSize={fileManager.totalSize()}
                  fileInfoMap={fileManager.fileInfoMap()}
                  fileStatusMap={fileManager.fileStatusMap()}
                  fileHashMap={hashManager.fileHashMap()}
                  hashHistory={hashManager.hashHistory()}
                  busy={fileManager.busy()}
                  allFilesSelected={fileManager.allFilesSelected()}
                  onToggleTypeFilter={(type) => fileManager.toggleTypeFilter(type)}
                  onClearTypeFilter={() => fileManager.setTypeFilter(null)}
                  onToggleSelectAll={() => fileManager.toggleSelectAll()}
                  onSelectFile={(file) => fileManager.selectAndViewFile(file)}
                  onToggleFileSelection={(path) => fileManager.toggleFileSelection(path)}
                  onHashFile={(file) => hashManager.hashSingleFile(file)}
                  onHover={(path) => fileManager.setHoveredFile(path)}
                  onFocus={(index) => fileManager.setFocusedFileIndex(index)}
                  onKeyDown={(e) => fileManager.handleFileListKeyDown(
                    e,
                    (file) => fileManager.selectAndViewFile(file),
                    (path) => fileManager.toggleFileSelection(path)
                  )}
                  onContextMenu={(file, e) => {
                    fileManager.setActiveFile(file);
                    fileContextMenu.open(e, getFileContextMenuItems(fileManager.activeFile));
                  }}
                />
              }>
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
                />
              </Show>
            </Show>

            {/* Processed Databases Tab */}
            <Show when={leftPanelTab() === "processed"}>
              <ProcessedDatabasePanel 
                manager={processedDbManager}
                onSelectDatabase={(db) => {
                  processedDbManager.selectDatabase(db);
                  // Clear active forensic file when switching to processed view
                  fileManager.setActiveFile(null);
                }}
                onSelectArtifact={(db, artifact) => console.log('Selected artifact:', artifact.name, 'from', db.path)}
              />
            </Show>

            {/* Case Documents Tab */}
            <Show when={leftPanelTab() === "casedocs"}>
              <CaseDocumentsPanel 
                evidencePath={fileManager.activeFile()?.path || projectManager.currentProject()?.locations?.evidence_path}
                onDocumentSelect={(doc) => {
                  // If it's a PDF, open it in the PDF viewer
                  if (doc.path.toLowerCase().endsWith('.pdf')) {
                    // Create a DiscoveredFile-like object for the DetailPanel
                    const pdfFile = {
                      path: doc.path,
                      filename: doc.filename,
                      container_type: 'pdf',
                      size: doc.size,
                      segment_count: null,
                      segment_files: null,
                      segment_sizes: null,
                      total_segment_size: null,
                      created: doc.created_at,
                      modified: doc.modified_at,
                    };
                    fileManager.setActiveFile(pdfFile);
                    // Request PDF view mode
                    setRequestViewMode("pdf");
                  }
                }}
              />
            </Show>
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
            <Show when={currentViewMode() === "hex"} fallback={<TreePanel info={activeFileInfo()} />}>
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
          onGenerated={(path, format) => {
            console.log(`Report generated: ${path} (${format})`);
            fileManager.setOk(`Report saved to ${path}`);
          }}
        />
      </Show>
    </div>
  );
}

export default App;
