// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Sidebar - Vertical icon bar for the left panel.
 * 
 * Contains:
 * - View mode toggle (tabs vs unified)
 * - Panel tab navigation (evidence, processed, case docs, activity)
 * - Project actions (save, load, export, report)
 * - Tools section (deduplication, performance, search)
 * - Utility buttons (theme, settings, help)
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
  HiOutlineDocumentDuplicate,
  HiOutlineBolt,
  HiOutlineQuestionMarkCircle,
  HiOutlineCommandLine,
  HiOutlinePlusCircle,
  HiOutlineBookmark,
} from "../icons";

// =============================================================================
// Types
// =============================================================================

export type LeftPanelTab = "evidence" | "processed" | "casedocs" | "activity" | "bookmarks";
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
  
  // Project info
  projectName?: Accessor<string | null>;
  projectPath?: Accessor<string | null>;
  
  // Bookmark count for badge
  bookmarkCount?: Accessor<number>;
  
  // Actions
  onSave: () => void;
  onSaveContextMenu: (e: MouseEvent) => void;
  onLoad: () => void;
  onNew?: () => void;  // New project action
  onExport: () => void;
  onReport: () => void;
  onSearch: () => void;
  onSettings: () => void;
  
  // Optional new actions
  onDeduplication?: () => void;
  onPerformance?: () => void;
  onCommandPalette?: () => void;
  onHelp?: () => void;
  
  // Theme
  theme: Accessor<Theme>;
  resolvedTheme: Accessor<ResolvedTheme>;
  cycleTheme: () => void;
}

// =============================================================================
// Sidebar Button Component
// =============================================================================

interface SidebarButtonProps {
  active?: boolean;
  disabled?: boolean;
  warning?: boolean;
  badge?: number | string;
  badgeColor?: "accent" | "warning" | "success";
  title: string;
  shortcut?: string;
  onClick?: () => void;
  onContextMenu?: (e: MouseEvent) => void;
  children: any;
}

const SidebarButton: Component<SidebarButtonProps> = (props) => {
  const baseClass = "flex items-center justify-center p-1.5 rounded-md transition-all duration-150 cursor-pointer relative focus:outline-none focus:ring-2 focus:ring-accent/50 group";
  
  const stateClass = () => {
    if (props.disabled) return "opacity-40 cursor-not-allowed";
    if (props.active) return "bg-accent text-white shadow-sm";
    if (props.warning) return "text-warning hover:text-warning hover:bg-warning/10";
    return "text-txt-secondary hover:text-txt hover:bg-bg-hover";
  };
  
  const badgeColorClass = () => {
    switch (props.badgeColor) {
      case "warning": return "bg-warning text-bg";
      case "success": return "bg-success text-white";
      default: return "bg-accent text-white";
    }
  };
  
  const fullTitle = () => props.shortcut ? `${props.title} (${props.shortcut})` : props.title;
  
  return (
    <button
      class={`${baseClass} ${stateClass()}`}
      onClick={props.disabled ? undefined : props.onClick}
      onContextMenu={props.onContextMenu}
      disabled={props.disabled}
      title={fullTitle()}
      aria-label={props.title}
    >
      {props.children}
      <Show when={props.badge !== undefined && props.badge !== 0}>
        <span class={`absolute -top-1 -right-1 flex items-center justify-center min-w-[14px] h-3.5 px-0.5 text-[9px] leading-tight font-bold rounded-full animate-pulse ${badgeColorClass()}`}>
          {props.badge}
        </span>
      </Show>
    </button>
  );
};

// =============================================================================
// Section Divider Component
// =============================================================================

const SectionDivider: Component<{ label?: string }> = (props) => (
  <div class="w-full flex items-center gap-1 px-1 my-1.5">
    <div class="flex-1 border-t border-border/40" />
    <Show when={props.label}>
      <span class="text-[9px] font-medium text-txt-muted/60 uppercase tracking-wider">{props.label}</span>
      <div class="flex-1 border-t border-border/40" />
    </Show>
  </div>
);

// =============================================================================
// Main Sidebar Component
// =============================================================================

export const Sidebar: Component<SidebarProps> = (props) => {
  const activeTransferCount = () => 
    props.transferJobs().filter(j => j.status === "running" || j.status === "pending").length;

  const toggleViewMode = () => {
    props.onViewModeChange(props.viewMode() === "tabs" ? "unified" : "tabs");
  };

  return (
    <div class="flex flex-col items-center gap-0.5 py-2 px-1.5 bg-bg-secondary border-r border-border h-full w-10 min-w-10">
      {/* === View Mode Section === */}
      <SidebarButton
        onClick={toggleViewMode}
        title={props.viewMode() === "tabs" ? "Switch to Unified View" : "Switch to Tab View"}
      >
        <Show when={props.viewMode() === "tabs"} fallback={<HiOutlineSquares2x2 class="w-4 h-4" />}>
          <HiOutlineQueueList class="w-4 h-4" />
        </Show>
      </SidebarButton>
      
      <SectionDivider />
      
      {/* === Navigation Section === */}
      <Show when={props.viewMode() === "tabs"}>
        <SidebarButton
          active={props.activeTab() === "evidence"}
          onClick={() => props.onTabChange("evidence")}
          title="Evidence Containers"
        >
          <HiOutlineArchiveBox class="w-4 h-4" />
        </SidebarButton>
        
        <SidebarButton
          active={props.activeTab() === "processed"}
          onClick={() => props.onTabChange("processed")}
          title="Processed Databases"
        >
          <HiOutlineChartBar class="w-4 h-4" />
        </SidebarButton>
        
        <SidebarButton
          active={props.activeTab() === "casedocs"}
          onClick={() => props.onTabChange("casedocs")}
          title="Case Documents"
        >
          <HiOutlineClipboardDocumentList class="w-4 h-4" />
        </SidebarButton>
        
        <SidebarButton
          active={props.activeTab() === "activity"}
          onClick={() => props.onTabChange("activity")}
          title="Activity Timeline"
        >
          <HiOutlineClock class="w-4 h-4" />
        </SidebarButton>
        
        <SidebarButton
          active={props.activeTab() === "bookmarks"}
          onClick={() => props.onTabChange("bookmarks")}
          title="Bookmarks"
          badge={props.bookmarkCount?.() || undefined}
          badgeColor="accent"
        >
          <HiOutlineBookmark class="w-4 h-4" />
        </SidebarButton>
        
        <SectionDivider />
      </Show>
      
      {/* === Spacer === */}
      <div class="flex-1" />
      
      {/* === Tools Section === */}
      <SectionDivider label="Tools" />
      
      <SidebarButton
        onClick={props.onSearch}
        title="Search"
        shortcut="⌘F"
      >
        <HiOutlineMagnifyingGlass class="w-4 h-4" />
      </SidebarButton>
      
      <Show when={props.onDeduplication}>
        <SidebarButton
          onClick={props.onDeduplication}
          title="File Deduplication"
          disabled={!props.hasDiscoveredFiles()}
        >
          <HiOutlineDocumentDuplicate class="w-4 h-4" />
        </SidebarButton>
      </Show>
      
      <Show when={props.onPerformance}>
        <SidebarButton
          onClick={props.onPerformance}
          title="Performance Monitor"
        >
          <HiOutlineBolt class="w-4 h-4" />
        </SidebarButton>
      </Show>
      
      <Show when={props.onCommandPalette}>
        <SidebarButton
          onClick={props.onCommandPalette}
          title="Command Palette"
          shortcut="⌘K"
        >
          <HiOutlineCommandLine class="w-4 h-4" />
        </SidebarButton>
      </Show>
      
      <SectionDivider label="Project" />
      
      {/* === Project Actions Section === */}
      <Show when={props.onNew}>
        <SidebarButton
          onClick={props.onNew!}
          disabled={props.busy()}
          title="New Project"
          shortcut="⌘⇧N"
        >
          <HiOutlinePlusCircle class="w-4 h-4" />
        </SidebarButton>
      </Show>
      
      <SidebarButton
        warning={props.projectModified()}
        onClick={props.onSave}
        onContextMenu={props.onSaveContextMenu}
        disabled={props.busy() || !props.hasEvidence()}
        title={props.projectModified() ? "Save Project (unsaved changes)" : "Save Project"}
        shortcut="⌘S"
      >
        <HiOutlineDocumentArrowDown class="w-4 h-4" />
      </SidebarButton>
      
      <SidebarButton
        onClick={props.onLoad}
        disabled={props.busy()}
        title="Open Project"
        shortcut="⌘O"
      >
        <HiOutlineFolderArrowDown class="w-4 h-4" />
      </SidebarButton>
      
      <SidebarButton
        onClick={props.onExport}
        disabled={props.busy()}
        badge={activeTransferCount() > 0 ? activeTransferCount() : undefined}
        badgeColor="accent"
        title={activeTransferCount() > 0 ? `Export (${activeTransferCount()} active)` : "Export Files"}
      >
        <HiOutlineArrowUpTray class="w-4 h-4" />
      </SidebarButton>
      
      <SidebarButton
        onClick={props.onReport}
        disabled={props.busy() || !props.hasDiscoveredFiles()}
        title="Generate Report"
        shortcut="⌘P"
      >
        <HiOutlineDocumentText class="w-4 h-4" />
      </SidebarButton>
      
      <SectionDivider />
      
      {/* === Utility Section === */}
      <ThemeSwitcher 
        compact 
        theme={props.theme}
        resolvedTheme={props.resolvedTheme}
        cycleTheme={props.cycleTheme}
      />
      
      <SidebarButton
        onClick={props.onSettings}
        title="Settings"
        shortcut="⌘,"
      >
        <HiOutlineCog6Tooth class="w-4 h-4" />
      </SidebarButton>
      
      <Show when={props.onHelp}>
        <SidebarButton
          onClick={props.onHelp}
          title="Help & Shortcuts"
          shortcut="?"
        >
          <HiOutlineQuestionMarkCircle class="w-4 h-4" />
        </SidebarButton>
      </Show>
    </div>
  );
};
