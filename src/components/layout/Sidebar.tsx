// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Sidebar - Vertical icon bar for the left panel.
 * 
 * Contains:
 * - Panel tab navigation (evidence, processed, case docs, activity)
 * - Project actions (save, load, export, report)
 * - Utility buttons (search, theme, settings)
 */

import { type Component, Show, type Accessor } from "solid-js";
import { ThemeSwitcher, type TransferJob } from "../";
import type { Theme, ResolvedTheme } from "../../hooks/useTheme";
import {
  HiOutlineArchiveBox,
  HiOutlineChartBar,
  HiOutlineClipboardDocumentList,
  HiOutlineClock,
  HiOutlineDocumentArrowDown,
  HiOutlineFolderArrowDown,
  HiOutlineArrowUpTray,
  HiOutlineDocumentText,
  HiOutlineMagnifyingGlass,
  HiOutlineCog6Tooth,
  HiOutlineSquares2x2,
  HiOutlineQueueList,
} from "../icons";

// =============================================================================
// Types
// =============================================================================

export type LeftPanelTab = "evidence" | "processed" | "casedocs" | "activity";
export type LeftPanelMode = "tabs" | "unified";

export interface SidebarProps {
  // Current tab
  activeTab: Accessor<LeftPanelTab>;
  onTabChange: (tab: LeftPanelTab) => void;
  
  // View mode (tabs vs unified collapsible)
  viewMode: Accessor<LeftPanelMode>;
  onViewModeChange: (mode: LeftPanelMode) => void;
  
  // State
  busy: Accessor<boolean>;
  hasEvidence: Accessor<boolean>;
  hasDiscoveredFiles: Accessor<boolean>;
  projectModified: Accessor<boolean>;
  transferJobs: Accessor<TransferJob[]>;
  
  // Actions
  onSave: () => void;
  onSaveContextMenu: (e: MouseEvent) => void;
  onLoad: () => void;
  onExport: () => void;
  onReport: () => void;
  onSearch: () => void;
  onSettings: () => void;
  
  // Theme
  theme: Accessor<Theme>;
  resolvedTheme: Accessor<ResolvedTheme>;
  cycleTheme: () => void;
}

// =============================================================================
// Component
// =============================================================================

export const Sidebar: Component<SidebarProps> = (props) => {
  const activeTransferCount = () => 
    props.transferJobs().filter(j => j.status === "running" || j.status === "pending").length;

  const toggleViewMode = () => {
    props.onViewModeChange(props.viewMode() === "tabs" ? "unified" : "tabs");
  };

  return (
    <div class="flex flex-col items-center gap-1.5 py-2 px-2.5 bg-bg-secondary border-r border-border h-full">
      {/* View Mode Toggle */}
      <button
        class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover mb-1"
        onClick={toggleViewMode}
        title={props.viewMode() === "tabs" ? "Switch to Unified View" : "Switch to Tab View"}
      >
        <Show when={props.viewMode() === "tabs"} fallback={<HiOutlineSquares2x2 class="w-4 h-4" />}>
          <HiOutlineQueueList class="w-4 h-4" />
        </Show>
      </button>
      
      {/* Divider */}
      <div class="w-4 border-t border-border/50 mb-1" />
      
      {/* Panel Tab Navigation - Only show in tabs mode */}
      <Show when={props.viewMode() === "tabs"}>
        <button 
          class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${props.activeTab() === "evidence" ? "bg-accent text-white" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
          onClick={() => props.onTabChange("evidence")}
          title="Evidence Containers (E01, AD1, L01, etc.)"
        >
          <HiOutlineArchiveBox class="w-4 h-4" />
        </button>
        <button 
          class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${props.activeTab() === "processed" ? "bg-accent text-white" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
          onClick={() => props.onTabChange("processed")}
          title="Processed Databases (AXIOM, Cellebrite PA, etc.)"
        >
          <HiOutlineChartBar class="w-4 h-4" />
        </button>
        <button 
          class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${props.activeTab() === "casedocs" ? "bg-accent text-white" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
          onClick={() => props.onTabChange("casedocs")}
          title="Case Documents (Chain of Custody, Intake Forms, etc.)"
        >
          <HiOutlineClipboardDocumentList class="w-4 h-4" />
        </button>
        <button 
          class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${props.activeTab() === "activity" ? "bg-accent text-white" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
          onClick={() => props.onTabChange("activity")}
          title="Activity Log & Sessions"
        >
          <HiOutlineClock class="w-4 h-4" />
        </button>
      </Show>
      
      {/* Spacer to push action and utility icons to bottom */}
      <div class="flex-1" />
      
      {/* Project Actions - Above Utilities */}
      <div class="flex flex-col items-center gap-1.5 pb-2 border-b border-border/50">
        <button 
          class={`flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer ${props.projectModified() ? "text-warning" : "text-txt-secondary hover:text-txt hover:bg-bg-hover"}`}
          onClick={props.onSave}
          onContextMenu={props.onSaveContextMenu}
          disabled={props.busy() || !props.hasEvidence()}
          title={`${props.projectModified() ? "Save Project (unsaved changes)" : "Save Project"} - Right-click for options`}
        >
          <HiOutlineDocumentArrowDown class="w-4 h-4" />
        </button>
        <button 
          class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover"
          onClick={props.onLoad}
          disabled={props.busy()}
          title="Load Project (⌘O)"
        >
          <HiOutlineFolderArrowDown class="w-4 h-4" />
        </button>
        <button 
          class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover relative"
          onClick={props.onExport}
          disabled={props.busy()}
          title="Export/Transfer Files"
        >
          <HiOutlineArrowUpTray class="w-4 h-4" />
          <Show when={activeTransferCount() > 0}>
            <span class="absolute -top-1 -right-1 flex items-center justify-center min-w-[14px] h-3.5 px-0.5 text-[9px] leading-tight font-bold text-white bg-accent rounded-full animate-pulse">
              {activeTransferCount()}
            </span>
          </Show>
        </button>
        <button 
          class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover"
          onClick={props.onReport}
          disabled={props.busy() || !props.hasDiscoveredFiles()}
          title="Generate Report (⌘P)"
        >
          <HiOutlineDocumentText class="w-4 h-4" />
        </button>
      </div>
      
      {/* Utility Icons - Bottom Section */}
      <div class="flex flex-col items-center gap-1.5 pt-2">
        <button 
          class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover"
          onClick={props.onSearch}
          title="Search (⌘F)"
        >
          <HiOutlineMagnifyingGlass class="w-4 h-4" />
        </button>
        <ThemeSwitcher 
          compact 
          theme={props.theme}
          resolvedTheme={props.resolvedTheme}
          cycleTheme={props.cycleTheme}
        />
        <button 
          class="flex items-center justify-center p-1.5 rounded transition-colors cursor-pointer text-txt-secondary hover:text-txt hover:bg-bg-hover"
          onClick={props.onSettings}
          title="Settings (⌘,)"
        >
          <HiOutlineCog6Tooth class="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};
