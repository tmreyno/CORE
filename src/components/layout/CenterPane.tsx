// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CenterPane - Unified tabbed interface for the center panel
 * 
 * Consolidates all content types into a single tabbed interface:
 * - Evidence files (with info/hex/text/pdf view modes)
 * - Case documents (with document preview)
 * - Container entries (files inside containers)
 * - Export panel
 * - Processed databases
 */

import { Component, Show, For, createMemo, type Accessor, type JSX } from "solid-js";
import {
  HiOutlineDocumentText,
  HiOutlineInformationCircle,
  HiOutlineDocument,
  HiOutlinePlusCircle,
  HiOutlineFolder,
  HiOutlineArchiveBox,
  HiOutlineFolderOpen,
  HiOutlineXMark,
  HiOutlineCheckCircle,
} from "../icons";
import type { DiscoveredFile, ProcessedDatabase } from "../../types";
import type { SelectedEntry } from "../EvidenceTree";
import { TabItem } from "./TabItem";
import { Shortcut, CommonShortcuts } from "../ui/Kbd";
import { RecentProjectsList } from "../RecentProjectsList";

// =============================================================================
// Types
// =============================================================================

/** Types of tabs that can be opened */
export type CenterTabType = "evidence" | "document" | "entry" | "export" | "processed" | "collection" | "help";

/** View modes for content display - uses CenterPane prefix to avoid conflict with existing TabViewMode */
export type CenterPaneViewMode = "info" | "hex" | "text" | "pdf" | "document" | "export";

/** A single tab in the center pane */
export interface CenterTab {
  id: string;
  type: CenterTabType;
  title: string;
  subtitle?: string;
  icon?: Component<{ class?: string }>;
  /** For evidence tabs */
  file?: DiscoveredFile;
  /** For case document tabs - the path */
  documentPath?: string;
  /** For case document tabs - the entry for viewing */
  documentEntry?: SelectedEntry;
  /** For entry tabs (files inside containers) */
  entry?: SelectedEntry;
  /** For processed database tabs */
  processedDb?: ProcessedDatabase;
  /** For collection tabs — the DB collection ID (undefined = new / list view) */
  collectionId?: string;
  /** For collection tabs — open in read-only/review mode */
  collectionReadOnly?: boolean;
  /** For collection tabs — show list view instead of single form */
  collectionListView?: boolean;
  /** Can this tab be closed? */
  closable?: boolean;
}

export interface CenterPaneProps {
  // Tab management
  tabs: Accessor<CenterTab[]>;
  activeTabId: Accessor<string | null>;
  onTabSelect: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
  onTabsChange: (tabs: CenterTab[]) => void;
  
  // View mode
  viewMode: Accessor<CenterPaneViewMode>;
  onViewModeChange: (mode: CenterPaneViewMode) => void;
  
  // Optional: Project action handlers for empty state
  onOpenProject?: (path?: string) => void;
  onNewProject?: () => void;
  
  // Optional: Project info for context-aware empty state
  projectName?: Accessor<string | null>;
  projectRoot?: Accessor<string | null>;
  evidenceCount?: Accessor<number>;
  
  // Children - the actual content rendered based on active tab
  children: JSX.Element;
}

// =============================================================================
// Main CenterPane Component
// =============================================================================

export const CenterPane: Component<CenterPaneProps> = (props) => {
  // Get available view modes based on active tab type
  const availableViewModes = createMemo((): CenterPaneViewMode[] => {
    const activeId = props.activeTabId();
    if (!activeId) return [];
    
    const activeTab = props.tabs().find(t => t.id === activeId);
    if (!activeTab) return [];
    
    switch (activeTab.type) {
      case "evidence":
        return ["info", "hex", "text", "pdf"];
      case "document":
        return ["document", "hex", "text"];
      case "entry":
        return ["document", "hex", "text"];
      case "processed":
        return []; // Processed DBs have their own internal view
      case "export":
        return ["export"];
      case "collection":
        return []; // Collection forms have their own internal view
      case "help":
        return []; // Help panel has its own internal navigation
      default:
        return [];
    }
  });
  
  // Separate tabs into container-level and entry-level
  const containerTabs = createMemo(() => 
    props.tabs().filter(t => t.type === "evidence" || t.type === "processed" || t.type === "export" || t.type === "document" || t.type === "collection")
  );
  
  const entryTabs = createMemo(() => 
    props.tabs().filter(t => t.type === "entry")
  );
  
  // Group entry tabs by their parent container
  const activeContainerTab = createMemo(() => {
    const activeId = props.activeTabId();
    if (!activeId) return null;
    
    // If active tab is a container, return it
    const containerTab = containerTabs().find(t => t.id === activeId);
    if (containerTab) return containerTab;
    
    // If active tab is an entry, find its parent container
    const entryTab = entryTabs().find(t => t.id === activeId);
    if (entryTab?.entry) {
      // Find the parent container tab based on containerPath
      return containerTabs().find(t => 
        t.type === "evidence" && t.file?.path === entryTab.entry?.containerPath
      ) || null;
    }
    
    return null;
  });
  
  // Get entry tabs for the currently active container
  const entriesForActiveContainer = createMemo(() => {
    const container = activeContainerTab();
    if (!container || container.type !== "evidence") return [];
    
    return entryTabs().filter(t => 
      t.entry?.containerPath === container.file?.path
    );
  });
  
  // Check if we're viewing an entry (not the container itself)
  const isViewingEntry = createMemo(() => {
    const activeId = props.activeTabId();
    return entryTabs().some(t => t.id === activeId);
  });

  const handleTabSelect = (tabId: string) => {
    props.onTabSelect(tabId);
    
    // Auto-select appropriate view mode for tab type
    const tab = props.tabs().find(t => t.id === tabId);
    if (tab) {
      const modes = availableViewModes();
      if (modes.length > 0 && !modes.includes(props.viewMode())) {
        props.onViewModeChange(modes[0]);
      }
    }
  };

  const handleTabClose = (tabId: string) => {
    // Delegate to the hook's closeTab which handles recently-closed tracking
    props.onTabClose(tabId);
  };
  
  // Tab count for header badge
  const tabCount = createMemo(() => props.tabs().length);
  const hasMultipleTabs = createMemo(() => tabCount() > 1);

  return (
    <div class="flex flex-col h-full overflow-hidden bg-bg">
      {/* Primary Tab Bar - Container-level tabs */}
      <div class="flex items-center bg-bg-secondary border-b border-border px-2 gap-1 shrink-0 h-9 min-h-[36px]">
        {/* Tab count indicator */}
        <Show when={hasMultipleTabs()}>
          <span class="flex items-center justify-center min-w-[18px] h-4 px-1 text-[10px] font-medium text-txt-muted bg-bg-hover rounded mr-1" title={`${tabCount()} open tabs`}>
            {tabCount()}
          </span>
        </Show>
        
        {/* Container Tabs - scrollable container */}
        <div class="flex items-center gap-0.5 overflow-x-auto scrollbar-thin flex-1 py-0.5">
          <For each={containerTabs()}>
            {(tab) => (
              <TabItem
                tab={tab}
                isActive={
                  props.activeTabId() === tab.id || 
                  (tab.type === "evidence" && activeContainerTab()?.id === tab.id)
                }
                onSelect={() => handleTabSelect(tab.id)}
                onClose={() => handleTabClose(tab.id)}
              />
            )}
          </For>
        </div>
      </div>
      
      {/* Secondary Tab Bar - Entry tabs from current container */}
      <Show when={entriesForActiveContainer().length > 0}>
        <div class="flex items-center bg-bg-panel/50 border-b border-border/50 px-2 gap-1 shrink-0 h-7 min-h-[28px]">
          {/* Breadcrumb indicator showing we're inside a container */}
          <div class="flex items-center gap-1 text-[10px] text-txt-muted mr-2 shrink-0">
            <HiOutlineFolderOpen class="w-3 h-3" />
            <span class="truncate max-w-[100px]">{activeContainerTab()?.title}</span>
            <span class="text-txt-muted/50">/</span>
          </div>
          
          {/* Entry tabs */}
          <div class="flex items-center gap-0.5 overflow-x-auto scrollbar-thin flex-1">
            <For each={entriesForActiveContainer()}>
              {(tab) => (
                <div
                  class={`flex items-center gap-1 px-2 py-1 text-[11px] rounded transition-all duration-150 group cursor-pointer select-none ${
                    props.activeTabId() === tab.id
                      ? "bg-bg text-txt border border-border/50 shadow-sm"
                      : "text-txt-muted hover:text-txt hover:bg-bg-hover/70"
                  }`}
                  onClick={() => handleTabSelect(tab.id)}
                  title={tab.entry?.entryPath}
                >
                  <HiOutlineDocument class="w-3 h-3 shrink-0" />
                  <span class="truncate max-w-[120px]">{tab.title}</span>
                  <button
                    class={`ml-0.5 p-0.5 rounded transition-all ${
                      props.activeTabId() === tab.id
                        ? "hover:bg-bg-hover opacity-60 hover:opacity-100"
                        : "hover:bg-bg-active opacity-0 group-hover:opacity-60 hover:!opacity-100"
                    }`}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleTabClose(tab.id);
                    }}
                    title="Close"
                  >
                    <HiOutlineXMark class="w-2.5 h-2.5" />
                  </button>
                </div>
              )}
            </For>
          </div>
          
          {/* Back to container button when viewing an entry */}
          <Show when={isViewingEntry() && activeContainerTab()}>
            <button
              class="flex items-center gap-1 px-2 py-1 text-[10px] text-accent hover:text-accent-hover hover:bg-accent/10 rounded transition-colors ml-1"
              onClick={() => handleTabSelect(activeContainerTab()!.id)}
              title="Back to container info"
            >
              <HiOutlineInformationCircle class="w-3 h-3" />
              <span>Info</span>
            </button>
          </Show>
        </div>
      </Show>

      {/* Content area */}
      <div class="flex-1 overflow-hidden">
        <Show when={props.tabs().length > 0} fallback={
          <div class="flex items-center justify-center h-full text-txt-muted text-sm">
            {/* Show project-active state when a project is open */}
            <Show when={props.projectName?.() || props.projectRoot?.()} fallback={
              /* No project - show welcome/new project screen */
              <div class="text-center p-8 max-w-lg">
                <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-bg-secondary flex items-center justify-center">
                  <HiOutlineDocumentText class="w-8 h-8 opacity-40" />
                </div>
                <h3 class="text-txt font-medium mb-2">No file selected</h3>
                <p class="text-txt-muted text-sm mb-6">
                  Select an evidence container, case document, or processed database from the sidebar to view its contents here.
                </p>
                
                {/* Quick Actions - New and Open buttons */}
                <Show when={props.onNewProject || props.onOpenProject}>
                  <div class="flex items-center justify-center gap-3 mb-6">
                    <Show when={props.onNewProject}>
                      <button
                        onClick={props.onNewProject}
                        class="btn btn-primary"
                      >
                        <HiOutlinePlusCircle class="w-4 h-4" />
                        New Project
                      </button>
                    </Show>
                    <Show when={props.onOpenProject}>
                      <button
                        onClick={() => props.onOpenProject?.()}
                        class="btn btn-secondary"
                      >
                        <HiOutlineFolderOpen class="w-4 h-4" />
                        Open Project
                      </button>
                    </Show>
                  </div>
                </Show>
                
                {/* Recent Projects - only show if handler provided */}
                <Show when={props.onOpenProject}>
                  <div class="mt-2 mb-6 text-left">
                    <RecentProjectsList 
                      onOpenProject={props.onOpenProject!} 
                      maxItems={4}
                    />
                  </div>
                </Show>
                
                <div class="flex items-center justify-center gap-6 text-xs text-txt-muted">
                  <span class="flex items-center gap-1.5">
                    <Shortcut {...CommonShortcuts.newProject} />
                    <span>New</span>
                  </span>
                  <span class="flex items-center gap-1.5">
                    <Shortcut {...CommonShortcuts.open} />
                    <span>Open</span>
                  </span>
                  <span class="flex items-center gap-1.5">
                    <Shortcut {...CommonShortcuts.commandPalette} />
                    <span>Commands</span>
                  </span>
                </div>
              </div>
            }>
              {/* Project is active - show project info and prompt to select file */}
              <div class="text-center p-8 max-w-lg">
                <div class="w-20 h-20 mx-auto mb-6 rounded-full bg-accent/10 flex items-center justify-center">
                  <HiOutlineCheckCircle class="w-10 h-10 text-accent" />
                </div>
                <h3 class="text-txt font-semibold text-lg mb-1">
                  {props.projectName?.() || "Project Ready"}
                </h3>
                <p class="text-txt-muted text-sm mb-6">
                  <Show when={props.projectRoot?.()} fallback="Project loaded successfully">
                    <span class="font-mono text-xs bg-bg-secondary px-2 py-1 rounded">
                      {props.projectRoot!()}
                    </span>
                  </Show>
                </p>
                
                {/* Evidence count if available */}
                <Show when={props.evidenceCount && props.evidenceCount() > 0}>
                  <div class="flex items-center justify-center gap-2 mb-6 text-sm">
                    <HiOutlineArchiveBox class="w-5 h-5 text-accent" />
                    <span class="text-txt">
                      <span class="font-semibold">{props.evidenceCount!()}</span> evidence file{props.evidenceCount!() !== 1 ? 's' : ''} discovered
                    </span>
                  </div>
                </Show>
                
                <div class="p-4 bg-bg-secondary rounded-lg mb-6">
                  <div class="flex items-center gap-3 text-left">
                    <HiOutlineFolder class="w-6 h-6 text-accent shrink-0" />
                    <div>
                      <p class="text-txt text-sm font-medium">Select a file to begin</p>
                      <p class="text-txt-muted text-xs">
                        Choose an evidence container, case document, or processed database from the sidebar
                      </p>
                    </div>
                  </div>
                </div>
                
                <div class="flex items-center justify-center gap-6 text-xs text-txt-muted">
                  <span class="flex items-center gap-1.5">
                    <Shortcut {...CommonShortcuts.open} />
                    <span>Open File</span>
                  </span>
                  <span class="flex items-center gap-1.5">
                    <Shortcut {...CommonShortcuts.commandPalette} />
                    <span>Commands</span>
                  </span>
                </div>
              </div>
            </Show>
          </div>
        }>
          {props.children}
        </Show>
      </div>
    </div>
  );
};
