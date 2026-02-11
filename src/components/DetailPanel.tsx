// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, createSignal, createEffect } from "solid-js";
import type { DiscoveredFile, ContainerInfo, TreeEntry, HashHistoryEntry, StoredHash } from "../types";
import type { HashAlgorithmName } from "../types/hash";
import type { FileStatus, FileHashInfo } from "../hooks";
import { DetailPanelContent } from "./DetailPanelContent";
import { HexViewer } from "./HexViewer";
import type { ParsedMetadata } from "./HexViewer";
import { TextViewer } from "./TextViewer";
import { PdfViewer } from "./PdfViewer";
import { logger } from "../utils/logger";
import { ExportPanel } from "./ExportPanel";
import { TabBar } from "./TabBar";
import type { TabViewMode, OpenTab } from "./TabBar";
import { Breadcrumb, type BreadcrumbItem } from "./Breadcrumb";

// Re-export TabViewMode and OpenTab for backward compatibility
export type { TabViewMode, OpenTab };

interface DetailPanelProps {
  // Current active file from file manager
  activeFile: DiscoveredFile | null;
  // Accessors to get info maps (must be functions for reactivity)
  fileInfoMap: () => Map<string, ContainerInfo>;
  fileStatusMap: () => Map<string, FileStatus>;
  fileHashMap: () => Map<string, FileHashInfo>;
  hashHistory: () => Map<string, HashHistoryEntry[]>;
  // Tree data for active tab
  tree: TreeEntry[];
  filteredTree: TreeEntry[];
  treeFilter: string;
  onTreeFilterChange: (filter: string) => void;
  // Other props
  selectedHashAlgorithm: HashAlgorithmName;
  storedHashesGetter: (info: ContainerInfo | undefined) => StoredHash[];
  busy: boolean;
  onLoadInfo: (file: DiscoveredFile) => void;
  formatHashDate: (timestamp: string) => string;
  // Tab switching callback (to update file manager's active file, null = all tabs closed)
  onTabSelect: (file: DiscoveredFile | null) => void;
  // Callback to notify parent of tab changes (for project save)
  onTabsChange?: (tabs: OpenTab[]) => void;
  // Callback when metadata is loaded from hex viewer (for right panel)
  onMetadataLoaded?: (metadata: ParsedMetadata | null) => void;
  // Callback when view mode changes (for right panel switching)
  onViewModeChange?: (mode: TabViewMode) => void;
  // Callback when hex viewer navigation is ready (for metadata panel to navigate)
  onHexNavigatorReady?: (navigateTo: (offset: number, size?: number) => void) => void;
  // Request to change view mode from parent (for metadata panel navigation)
  requestViewMode?: TabViewMode | null;
  // Callback to clear the view mode request after processing
  onViewModeRequestHandled?: () => void;
  // Breadcrumb navigation
  breadcrumbItems?: BreadcrumbItem[];
  onBreadcrumbNavigate?: (path: string) => void;
  // Export panel props
  scanDir?: string;
  selectedFiles?: DiscoveredFile[];
  // Callback when transfer starts (for opening right panel)
  onTransferStart?: () => void;
  // Callback when hashes are computed during transfer
  onHashComputed?: (entries: HashHistoryEntry[]) => void;
}

export function DetailPanel(props: DetailPanelProps) {
  // Tab state
  const [openTabs, setOpenTabs] = createSignal<OpenTab[]>([]);
  const [activeTabId, setActiveTabId] = createSignal<string | null>(null);
  // Track recently closed tabs to prevent immediate re-opening
  const [recentlyClosed, setRecentlyClosed] = createSignal<Set<string>>(new Set());
  // Track view mode per tab (default to "info")
  const [tabViewModes, setTabViewModes] = createSignal<Map<string, TabViewMode>>(new Map());
  // Global view mode for tab-independent views like Export
  const [globalViewMode, setGlobalViewMode] = createSignal<TabViewMode | null>(null);
  
  // Notify parent of tab changes (for project save)
  createEffect(() => {
    const tabs = openTabs();
    if (props.onTabsChange) {
      props.onTabsChange(tabs);
    }
  });
  
  // Special tab ID for the Export panel
  const EXPORT_TAB_ID = "__export__";
  
  // Handle view mode request from parent (e.g., from Toolbar or MetadataPanel click)
  createEffect(() => {
    const requestedMode = props.requestViewMode;
    if (requestedMode) {
      // Export mode creates a special Export tab
      if (requestedMode === "export") {
        // Check if Export tab already exists
        const tabs = openTabs();
        const existingExportTab = tabs.find(t => t.id === EXPORT_TAB_ID);
        
        if (!existingExportTab) {
          // Create a special Export tab (file is null/placeholder)
          const exportTab: OpenTab = {
            file: {
              path: EXPORT_TAB_ID,
              filename: "Export",
              container_type: "export",
              size: 0,
            },
            id: EXPORT_TAB_ID,
          };
          setOpenTabs([...tabs, exportTab]);
        }
        // Switch to Export tab
        setActiveTabId(EXPORT_TAB_ID);
        // Set view mode for this tab
        setTabViewModes(prev => {
          const next = new Map(prev);
          next.set(EXPORT_TAB_ID, "export");
          return next;
        });
        // Notify parent of view mode change
        if (props.onViewModeChange) {
          props.onViewModeChange(requestedMode);
        }
      } else {
        // Other modes require an active tab
        const id = activeTabId();
        if (id) {
          // Clear global mode when switching to tab-specific modes
          setGlobalViewMode(null);
          setTabViewModes(prev => {
            const next = new Map(prev);
            next.set(id, requestedMode);
            return next;
          });
          // Notify parent of view mode change
          if (props.onViewModeChange) {
            props.onViewModeChange(requestedMode);
          }
        }
      }
      // Clear the request
      if (props.onViewModeRequestHandled) {
        props.onViewModeRequestHandled();
      }
    }
  });
  
  // When activeFile changes from file manager, open it as a tab
  createEffect(() => {
    const file = props.activeFile;
    if (!file) return;
    
    // Don't re-open recently closed tabs (prevents immediate re-opening)
    if (recentlyClosed().has(file.path)) {
      return;
    }
    
    // Clear recently closed when opening a new/different file
    // This allows previously closed files to be re-opened on explicit click
    if (recentlyClosed().size > 0) {
      setRecentlyClosed(new Set<string>());
    }
    
    // Clear global view mode when selecting a file (exit Export view)
    if (globalViewMode()) {
      setGlobalViewMode(null);
    }
    
    const tabs = openTabs();
    const currentActiveId = activeTabId();
    const existingTabIndex = tabs.findIndex(t => t.id === file.path);
    
    if (existingTabIndex >= 0) {
      // Tab already exists - only update if not already active (prevents infinite loops)
      if (currentActiveId !== file.path) {
        setActiveTabId(file.path);
      }
      // Don't update tab data here - it causes unnecessary re-renders
    } else {
      // Add new tab
      const newTab: OpenTab = { file, id: file.path };
      setOpenTabs([...tabs, newTab]);
      setActiveTabId(file.path);
    }
  });
  
  // Get the currently active tab's file
  const activeTab = () => {
    const id = activeTabId();
    if (!id) return null;
    return openTabs().find(t => t.id === id) ?? null;
  };
  
  // Close a tab
  const closeTab = (tabId: string, e?: MouseEvent) => {
    e?.stopPropagation();
    e?.preventDefault();
    
    const tabs = openTabs();
    const tabIndex = tabs.findIndex(t => t.id === tabId);
    if (tabIndex === -1) return;
    
    // Mark as recently closed to prevent re-opening from activeFile prop
    setRecentlyClosed(prev => new Set([...prev, tabId]));
    
    const newTabs = tabs.filter(t => t.id !== tabId);
    setOpenTabs(newTabs);
    
    // If we closed the active tab, activate another
    if (activeTabId() === tabId) {
      if (newTabs.length > 0) {
        // Activate the tab to the left, or the first tab
        const newActiveIndex = Math.min(tabIndex, newTabs.length - 1);
        const newActiveTab = newTabs[newActiveIndex];
        setActiveTabId(newActiveTab.id);
        props.onTabSelect(newActiveTab.file);
      } else {
        setActiveTabId(null);
        // Notify parent that no tabs are open
        props.onTabSelect(null);
      }
    }
  };
  
  // Close all tabs except the given one
  const closeOtherTabs = (keepTabId: string) => {
    const tabs = openTabs();
    const keepTab = tabs.find(t => t.id === keepTabId);
    if (keepTab) {
      // Mark closed tabs as recently closed
      const closedIds = tabs.filter(t => t.id !== keepTabId).map(t => t.id);
      setRecentlyClosed(prev => new Set([...prev, ...closedIds]));
      
      setOpenTabs([keepTab]);
      setActiveTabId(keepTabId);
      props.onTabSelect(keepTab.file);
    }
  };
  
  // Close all tabs
  const closeAllTabs = () => {
    // Mark all tabs as recently closed
    const closedIds = openTabs().map(t => t.id);
    setRecentlyClosed(prev => new Set([...prev, ...closedIds]));
    
    setOpenTabs([]);
    setActiveTabId(null);
    // Notify parent that no tabs are open
    props.onTabSelect(null);
  };
  
  // Select a tab
  const selectTab = (tab: OpenTab) => {
    // Clear global view mode when selecting a tab (exit Export view)
    if (globalViewMode()) {
      setGlobalViewMode(null);
    }
    setActiveTabId(tab.id);
    props.onTabSelect(tab.file);
  };
  
  // Move tab to a new position
  const moveTab = (fromIndex: number, toIndex: number) => {
    if (fromIndex === toIndex) return;
    const tabs = [...openTabs()];
    const [movedTab] = tabs.splice(fromIndex, 1);
    tabs.splice(toIndex, 0, movedTab);
    setOpenTabs(tabs);
  };
  
  // Get data for active tab
  const activeTabFile = () => activeTab()?.file ?? null;
  const activeFileInfo = () => {
    const file = activeTabFile();
    return file ? props.fileInfoMap().get(file.path) : undefined;
  };
  const activeFileHash = () => {
    const file = activeTabFile();
    return file ? props.fileHashMap().get(file.path) : undefined;
  };
  const activeFileStatus = () => {
    const file = activeTabFile();
    return file ? props.fileStatusMap().get(file.path) : undefined;
  };
  const activeFileHashHistory = () => {
    const file = activeTabFile();
    return file ? props.hashHistory().get(file.path) ?? [] : [];
  };
  
  // Get current view mode for active tab
  const getActiveViewMode = (): TabViewMode => {
    // Global view mode takes precedence (for tab-independent views like Export)
    const global = globalViewMode();
    if (global) return global;
    
    const id = activeTabId();
    if (!id) return "info";
    return tabViewModes().get(id) ?? "info";
  };
  
  // Set view mode for active tab
  const setActiveViewMode = (mode: TabViewMode) => {
    // Export mode creates a special Export tab
    if (mode === "export") {
      // Check if Export tab already exists
      const tabs = openTabs();
      const existingExportTab = tabs.find(t => t.id === EXPORT_TAB_ID);
      
      if (!existingExportTab) {
        // Create a special Export tab
        const exportTab: OpenTab = {
          file: {
            path: EXPORT_TAB_ID,
            filename: "Export",
            container_type: "export",
            size: 0,
          },
          id: EXPORT_TAB_ID,
        };
        setOpenTabs([...tabs, exportTab]);
      }
      // Switch to Export tab
      setActiveTabId(EXPORT_TAB_ID);
      // Set view mode for this tab
      setTabViewModes(prev => {
        const next = new Map(prev);
        next.set(EXPORT_TAB_ID, "export");
        return next;
      });
      // Notify parent of view mode change
      if (props.onViewModeChange) {
        props.onViewModeChange(mode);
      }
      return;
    }
    
    // Clear global mode when switching to tab-specific modes
    setGlobalViewMode(null);
    
    const id = activeTabId();
    if (!id) return;
    setTabViewModes(prev => {
      const next = new Map(prev);
      next.set(id, mode);
      return next;
    });
    // Notify parent of view mode change
    if (props.onViewModeChange) {
      props.onViewModeChange(mode);
    }
  };
  
  return (
    <main class="flex flex-col h-full overflow-hidden" role="main" aria-label="File detail view">
      {/* Tab bar component */}
      <TabBar
        tabs={openTabs()}
        activeTabId={activeTabId()}
        viewMode={getActiveViewMode()}
        onTabSelect={selectTab}
        onTabClose={closeTab}
        onCloseOthers={closeOtherTabs}
        onCloseAll={closeAllTabs}
        onTabMove={moveTab}
        onViewModeChange={setActiveViewMode}
      />
      
      {/* Breadcrumb navigation - under TabBar */}
      <Show when={props.breadcrumbItems && props.breadcrumbItems.length > 0}>
        <div class="px-2 py-0.5 border-b border-border/50 bg-bg/50">
          <Breadcrumb 
            items={props.breadcrumbItems!} 
            onNavigate={(path) => props.onBreadcrumbNavigate?.(path)}
          />
        </div>
      </Show>
      
      {/* Content area - switches based on view mode */}
      <div class="flex-1 overflow-y-auto min-h-0" role="tabpanel" aria-label={`${getActiveViewMode()} view for ${activeTabFile()?.filename || 'no file selected'}`}>
        {/* Info view (default) */}
        <Show when={getActiveViewMode() === "info"}>
          <DetailPanelContent
            activeFile={activeTabFile()}
            fileInfo={activeFileInfo()}
            fileHash={activeFileHash()}
            fileStatus={activeFileStatus()}
            tree={props.tree}
            filteredTree={props.filteredTree}
            treeFilter={props.treeFilter}
            onTreeFilterChange={props.onTreeFilterChange}
            selectedHashAlgorithm={props.selectedHashAlgorithm}
            hashHistory={activeFileHashHistory()}
            storedHashes={props.storedHashesGetter(activeFileInfo())}
            busy={props.busy}
            onLoadInfo={() => activeTabFile() && props.onLoadInfo(activeTabFile()!)}
            formatHashDate={props.formatHashDate}
          />
        </Show>
        
        {/* Hex view */}
        <Show when={getActiveViewMode() === "hex" && activeTabFile()}>
          <HexViewer
            file={activeTabFile()!}
            onMetadataLoaded={props.onMetadataLoaded}
            onNavigatorReady={props.onHexNavigatorReady}
          />
        </Show>
        
        {/* Text view */}
        <Show when={getActiveViewMode() === "text" && activeTabFile()}>
          <TextViewer
            file={activeTabFile()!}
          />
        </Show>
        
        {/* PDF view */}
        <Show when={getActiveViewMode() === "pdf" && activeTabFile()}>
          <PdfViewer
            path={activeTabFile()!.path}
          />
        </Show>
        
        {/* Export view */}
        <Show when={getActiveViewMode() === "export"}>
          <ExportPanel
            initialSources={props.selectedFiles?.map(f => f.path) || []}
            onComplete={(destination) => {
              logger.debug("Export completed:", destination);
            }}
          />
        </Show>
      </div>
    </main>
  );
}
