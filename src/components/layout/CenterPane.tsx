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

import { Component, Show, For, createMemo, type Accessor } from "solid-js";
import {
  HiOutlineDocumentText,
  HiOutlineClipboardDocumentList,
  HiOutlineXMark,
  HiOutlineInformationCircle,
  HiOutlineCodeBracket,
  HiOutlineDocument,
  HiOutlineArrowUpTray,
  HiOutlineTableCells,
} from "../icons";
import type { DiscoveredFile, ProcessedDatabase } from "../../types";
import type { SelectedEntry } from "../EvidenceTree";
// =============================================================================
// Types
// =============================================================================

/** Types of tabs that can be opened */
export type CenterTabType = "evidence" | "document" | "entry" | "export" | "processed";

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
  
  // Children - the actual content rendered based on active tab
  children: any;
}

// =============================================================================
// Tab Bar Sub-component
// =============================================================================

interface TabItemProps {
  tab: CenterTab;
  isActive: boolean;
  onSelect: () => void;
  onClose: () => void;
}

const TabItem: Component<TabItemProps> = (props) => {
  const Icon = () => {
    if (props.tab.icon) {
      const IconComp = props.tab.icon;
      return <IconComp class="w-3.5 h-3.5 shrink-0" />;
    }
    // Default icons based on type
    switch (props.tab.type) {
      case "evidence":
        return <HiOutlineDocumentText class="w-3.5 h-3.5 shrink-0" />;
      case "document":
        return <HiOutlineClipboardDocumentList class="w-3.5 h-3.5 shrink-0" />;
      case "entry":
        return <HiOutlineDocument class="w-3.5 h-3.5 shrink-0" />;
      case "export":
        return <HiOutlineArrowUpTray class="w-3.5 h-3.5 shrink-0" />;
      case "processed":
        return <HiOutlineTableCells class="w-3.5 h-3.5 shrink-0" />;
      default:
        return <HiOutlineDocument class="w-3.5 h-3.5 shrink-0" />;
    }
  };

  return (
    <div
      class="flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-t transition-colors group cursor-pointer select-none"
      classList={{
        "bg-bg text-txt border-t border-l border-r border-border -mb-px": props.isActive,
        "text-txt-muted hover:text-txt hover:bg-bg-hover": !props.isActive,
      }}
      onClick={props.onSelect}
      onMouseDown={(e) => {
        // Middle click to close
        if (e.button === 1 && props.tab.closable !== false) {
          e.preventDefault();
          props.onClose();
        }
      }}
      title={props.tab.subtitle || props.tab.title}
    >
      <Icon />
      <span class="truncate max-w-[120px]">{props.tab.title}</span>
      <Show when={props.tab.subtitle}>
        <span class="text-txt-muted truncate max-w-[80px]">
          — {props.tab.subtitle}
        </span>
      </Show>
      <Show when={props.tab.closable !== false}>
        <button
          class="ml-0.5 p-0.5 rounded hover:bg-bg-hover opacity-0 group-hover:opacity-100 transition-opacity"
          onClick={(e) => {
            e.stopPropagation();
            props.onClose();
          }}
          title="Close tab"
        >
          <HiOutlineXMark class="w-3 h-3" />
        </button>
      </Show>
    </div>
  );
};

// =============================================================================
// View Mode Selector Sub-component
// =============================================================================

interface ViewModeSelectorProps {
  currentMode: CenterPaneViewMode;
  availableModes: CenterPaneViewMode[];
  onModeChange: (mode: CenterPaneViewMode) => void;
}

const ViewModeSelector: Component<ViewModeSelectorProps> = (props) => {
  const modeConfig: Record<CenterPaneViewMode, { icon: Component<{ class?: string }>; label: string }> = {
    info: { icon: HiOutlineInformationCircle, label: "Info" },
    hex: { icon: HiOutlineCodeBracket, label: "Hex" },
    text: { icon: HiOutlineDocumentText, label: "Text" },
    pdf: { icon: HiOutlineDocument, label: "PDF" },
    document: { icon: HiOutlineClipboardDocumentList, label: "Preview" },
    export: { icon: HiOutlineArrowUpTray, label: "Export" },
  };

  return (
    <div class="flex items-center gap-0.5 px-1 border-l border-border ml-auto">
      <For each={props.availableModes}>
        {(mode) => {
          const config = modeConfig[mode];
          const Icon = config.icon;
          return (
            <button
              class="flex items-center gap-1 px-2 py-0.5 text-xs rounded transition-colors"
              classList={{
                "bg-accent/20 text-accent": props.currentMode === mode,
                "text-txt-muted hover:text-txt hover:bg-bg-hover": props.currentMode !== mode,
              }}
              onClick={() => props.onModeChange(mode)}
              title={config.label}
            >
              <Icon class="w-3.5 h-3.5" />
              <span class="hidden sm:inline">{config.label}</span>
            </button>
          );
        }}
      </For>
    </div>
  );
};

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
      default:
        return [];
    }
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

  return (
    <div class="flex flex-col h-full overflow-hidden">
      {/* Tab bar with view mode selector */}
      <div class="flex items-center bg-bg-secondary border-b border-border px-1 gap-0.5 shrink-0 h-8 min-h-[32px]">
        {/* Tabs */}
        <div class="flex items-center gap-0.5 overflow-x-auto scrollbar-thin">
          <For each={props.tabs()}>
            {(tab) => (
              <TabItem
                tab={tab}
                isActive={props.activeTabId() === tab.id}
                onSelect={() => handleTabSelect(tab.id)}
                onClose={() => handleTabClose(tab.id)}
              />
            )}
          </For>
        </div>
        
        {/* View mode selector - only show when there are available modes */}
        <Show when={availableViewModes().length > 1}>
          <ViewModeSelector
            currentMode={props.viewMode()}
            availableModes={availableViewModes()}
            onModeChange={props.onViewModeChange}
          />
        </Show>
      </div>

      {/* Content area */}
      <div class="flex-1 overflow-hidden">
        <Show when={props.tabs().length > 0} fallback={
          <div class="flex items-center justify-center h-full text-txt-muted text-sm">
            <div class="text-center">
              <HiOutlineDocumentText class="w-12 h-12 mx-auto mb-2 opacity-30" />
              <p>Select a file or document to view</p>
            </div>
          </div>
        }>
          {props.children}
        </Show>
      </div>
    </div>
  );
};

export default CenterPane;
