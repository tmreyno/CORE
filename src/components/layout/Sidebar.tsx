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
import type { JSX } from "solid-js";
import { ThemeSwitcher } from "../";
import type { Theme, ResolvedTheme } from "../../hooks/useTheme";
import { ContextMenu, createContextMenu, type ContextMenuItem } from "../ContextMenu";
import { REPORT_TYPES } from "../report/constants";
import type { ReportType } from "../report/types";
import {
  HiOutlineArchiveBox,
  HiOutlineChartBar,
  HiOutlineClipboardDocumentList,
  HiOutlineClock,
  HiOutlineArrowUpTray,
  HiOutlineMagnifyingGlass,
  HiOutlineCog6Tooth,
  HiOutlineSquares2x2,
  HiOutlineQueueList,
  HiOutlineDocumentDuplicate,
  HiOutlineQuestionMarkCircle,
  HiOutlineCommandLine,
  HiOutlineBookmark,
  HiOutlineRectangleGroup,
} from "../icons";

// =============================================================================
// Types
// =============================================================================

export type LeftPanelTab = "dashboard" | "evidence" | "processed" | "casedocs" | "activity" | "bookmarks";
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
  
  // Project info
  projectName?: Accessor<string | null>;
  projectPath?: Accessor<string | null>;
  hasProject?: Accessor<boolean>;
  
  // Bookmark count for badge
  bookmarkCount?: Accessor<number>;
  
  // Actions
  onExport: () => void;
  onReport: () => void;
  onSearch: () => void;
  onSettings: () => void;
  
  // Context menu actions
  onReportType?: (type: ReportType) => void;
  onExportSelected?: () => void;
  onClearBookmarks?: () => void;
  onExportBookmarks?: () => void;
  
  // Optional new actions
  onDeduplication?: () => void;
  onCommandPalette?: () => void;
  onHelp?: () => void;
  onEvidenceCollection?: () => void;
  onEvidenceCollectionList?: () => void;
  
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
  children: JSX.Element;
}

const SidebarButton: Component<SidebarButtonProps> = (props) => {
  const baseClass = "flex items-center justify-center p-1 rounded-md transition-all duration-150 cursor-pointer relative focus:outline-none focus:ring-2 focus:ring-accent/50 group";
  
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
  <div class="w-full flex items-center gap-1 my-1.5">
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
  const toggleViewMode = () => {
    props.onViewModeChange(props.viewMode() === "tabs" ? "unified" : "tabs");
  };

  // Context menu state
  const contextMenu = createContextMenu();

  // ---- Report context menu items ----
  const reportMenuItems = (): ContextMenuItem[] => [
    ...REPORT_TYPES.map((rt) => ({
      id: `report-${rt.value}`,
      label: rt.label,
      icon: rt.icon,
      onSelect: () => {
        if (props.onReportType) {
          props.onReportType(rt.value);
        } else {
          props.onReport();
        }
      },
    })),
    { id: "report-sep", label: "", separator: true },
    {
      id: "evidence-collection",
      label: "New Evidence Collection…",
      icon: "📦",
      onSelect: () => props.onEvidenceCollection?.(),
    },
    {
      id: "evidence-collection-list",
      label: "Browse Collections…",
      icon: "📋",
      onSelect: () => props.onEvidenceCollectionList?.(),
    },
    {
      id: "report-wizard",
      label: "Open Report Wizard…",
      icon: "📝",
      shortcut: "⌘P",
      onSelect: () => props.onReport(),
    },
  ];

  // ---- Export context menu items ----
  const exportMenuItems = (): ContextMenuItem[] => [
    {
      id: "export-panel",
      label: "Open Export Panel",
      icon: "📤",
      onSelect: () => props.onExport(),
    },
    { id: "export-sep", label: "", separator: true },
    {
      id: "export-selected",
      label: "Export Selected Files",
      icon: "📁",
      disabled: !props.hasDiscoveredFiles(),
      onSelect: () => props.onExportSelected?.() ?? props.onExport(),
    },
  ];

  // ---- Bookmarks context menu items ----
  const bookmarkMenuItems = (): ContextMenuItem[] => [
    {
      id: "bookmarks-view",
      label: "View Bookmarks",
      icon: "📑",
      onSelect: () => props.onTabChange("bookmarks"),
    },
    { id: "bookmarks-sep", label: "", separator: true },
    {
      id: "bookmarks-export",
      label: "Export Bookmarks",
      icon: "💾",
      disabled: !props.bookmarkCount?.(),
      onSelect: () => props.onExportBookmarks?.(),
    },
    {
      id: "bookmarks-clear",
      label: "Clear All Bookmarks",
      icon: "🗑️",
      danger: true,
      disabled: !props.bookmarkCount?.(),
      onSelect: () => props.onClearBookmarks?.(),
    },
  ];

  // ---- Settings/Utility context menu items ----
  const settingsMenuItems = (): ContextMenuItem[] => [
    {
      id: "settings-open",
      label: "Open Settings",
      icon: "⚙️",
      shortcut: "⌘,",
      onSelect: () => props.onSettings(),
    },
    { id: "settings-sep", label: "", separator: true },
    {
      id: "settings-shortcuts",
      label: "Keyboard Shortcuts",
      icon: "⌨️",
      shortcut: "?",
      onSelect: () => props.onHelp?.(),
    },
    {
      id: "settings-command",
      label: "Command Palette",
      icon: "🔧",
      shortcut: "⌘K",
      onSelect: () => props.onCommandPalette?.(),
    },
  ];

  return (
    <div class="flex flex-col items-center gap-0.5 py-2 pl-2 pr-1 bg-bg-secondary border-r border-border h-full w-12 min-w-12">
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
          active={props.activeTab() === "dashboard"}
          onClick={() => props.onTabChange("dashboard")}
          title="Project Dashboard"
        >
          <HiOutlineRectangleGroup class="w-4 h-4" />
        </SidebarButton>
        
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
          onContextMenu={(e) => contextMenu.open(e, bookmarkMenuItems())}
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
      <SectionDivider />
      
      <SidebarButton
        onClick={props.onSearch}
        disabled={!props.hasProject?.()}
        title={props.hasProject?.() ? "Search" : "Search (open a project first)"}
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
      
      <Show when={props.onCommandPalette}>
        <SidebarButton
          onClick={props.onCommandPalette}
          title="Command Palette"
          shortcut="⌘K"
        >
          <HiOutlineCommandLine class="w-4 h-4" />
        </SidebarButton>
      </Show>
      
      <SectionDivider />
      
      {/* === Project Actions Section === */}
      <SidebarButton
        onClick={props.onExport}
        onContextMenu={(e) => { if (props.hasProject?.()) contextMenu.open(e, exportMenuItems()); }}
        disabled={props.busy() || !props.hasProject?.()}
        title={props.hasProject?.() ? "Export Files" : "Export Files (open a project first)"}
      >
        <HiOutlineArrowUpTray class="w-4 h-4" />
      </SidebarButton>
      
      <SidebarButton
        onClick={props.onReport}
        onContextMenu={(e) => { if (props.hasProject?.()) contextMenu.open(e, reportMenuItems()); }}
        disabled={props.busy() || !props.hasProject?.()}
        title={props.hasProject?.() ? "Generate Report" : "Generate Report (open a project first)"}
        shortcut="⌘P"
      >
        <HiOutlineClipboardDocumentList class="w-4 h-4" />
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
        onContextMenu={(e) => contextMenu.open(e, settingsMenuItems())}
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
      
      {/* Context Menu (portal-rendered) */}
      <ContextMenu
        items={contextMenu.items()}
        position={contextMenu.position()}
        onClose={contextMenu.close}
      />
    </div>
  );
};
