// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Context menu item builders for each Sidebar button.
 * Each function returns a ContextMenuItem[] array using callbacks from SidebarProps.
 */

import type { ContextMenuItem } from "../../ContextMenu";
import { REPORT_TYPES } from "../../report/constants";
import type { SidebarProps } from "./types";

// ---------- View Mode ----------

export const viewModeMenuItems = (props: SidebarProps): ContextMenuItem[] => [
  {
    id: "viewmode-tabs",
    label: "Tab View",
    icon: "📑",
    checked: props.viewMode() === "tabs",
    onSelect: () => props.onViewModeChange("tabs"),
  },
  {
    id: "viewmode-unified",
    label: "Unified View",
    icon: "📋",
    checked: props.viewMode() === "unified",
    onSelect: () => props.onViewModeChange("unified"),
  },
];

// ---------- Dashboard ----------

export const dashboardMenuItems = (props: SidebarProps): ContextMenuItem[] => [
  {
    id: "dash-view",
    label: "Show Dashboard",
    icon: "📊",
    onSelect: () => props.onTabChange("dashboard"),
  },
  { id: "dash-sep", label: "", separator: true },
  {
    id: "dash-toggle-sidebar",
    label: "Toggle Sidebar",
    icon: "◀️",
    shortcut: "⌘B",
    onSelect: () => props.onToggleSidebar?.(),
  },
  {
    id: "dash-toggle-right",
    label: "Toggle Right Panel",
    icon: "▶️",
    shortcut: "⇧⌘B",
    onSelect: () => props.onToggleRightPanel?.(),
  },
  {
    id: "dash-toggle-quick",
    label: "Toggle Quick Actions",
    icon: "⚡",
    onSelect: () => props.onToggleQuickActions?.(),
  },
];

// ---------- Evidence ----------

export const evidenceMenuItems = (props: SidebarProps): ContextMenuItem[] => [
  {
    id: "ev-view",
    label: "Show Evidence",
    icon: "📦",
    onSelect: () => props.onTabChange("evidence"),
  },
  { id: "ev-sep1", label: "", separator: true },
  {
    id: "ev-scan",
    label: "Scan for Files",
    icon: "🔍",
    shortcut: "⌘R",
    onSelect: () => props.onScanEvidence?.(),
  },
  {
    id: "ev-load-all",
    label: "Load All Info",
    icon: "📥",
    disabled: !props.hasDiscoveredFiles(),
    onSelect: () => props.onLoadAllInfo?.(),
  },
  {
    id: "ev-select-all",
    label: "Select All Evidence",
    icon: "☑️",
    disabled: !props.hasDiscoveredFiles(),
    onSelect: () => props.onSelectAllEvidence?.(),
  },
];

// ---------- Processed Databases ----------

export const processedMenuItems = (props: SidebarProps): ContextMenuItem[] => [
  {
    id: "proc-view",
    label: "Show Processed Databases",
    icon: "📈",
    onSelect: () => props.onTabChange("processed"),
  },
  { id: "proc-sep", label: "", separator: true },
  {
    id: "proc-refresh",
    label: "Refresh Databases",
    icon: "🔄",
    onSelect: () => props.onRefreshProcessed?.(),
  },
];

// ---------- Case Documents ----------

export const caseDocsMenuItems = (props: SidebarProps): ContextMenuItem[] => [
  {
    id: "docs-view",
    label: "Show Case Documents",
    icon: "📋",
    onSelect: () => props.onTabChange("casedocs"),
  },
  { id: "docs-sep", label: "", separator: true },
  {
    id: "docs-refresh",
    label: "Refresh Documents",
    icon: "🔄",
    onSelect: () => props.onRefreshCaseDocs?.(),
  },
];

// ---------- Activity ----------

export const activityMenuItems = (props: SidebarProps): ContextMenuItem[] => [
  {
    id: "activity-view",
    label: "Show Activity Timeline",
    icon: "🕐",
    onSelect: () => props.onTabChange("activity"),
  },
];

// ---------- Bookmarks ----------

export const bookmarkMenuItems = (props: SidebarProps): ContextMenuItem[] => [
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

// ---------- Report ----------

export const reportMenuItems = (props: SidebarProps): ContextMenuItem[] => [
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

// ---------- Export ----------

export const exportMenuItems = (props: SidebarProps): ContextMenuItem[] => [
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

// ---------- Search ----------

export const searchMenuItems = (props: SidebarProps): ContextMenuItem[] => [
  {
    id: "search-open",
    label: "Open Search Panel",
    icon: "🔍",
    shortcut: "⌘F",
    onSelect: () => props.onSearch(),
  },
  { id: "search-sep", label: "", separator: true },
  {
    id: "search-dedup",
    label: "File Deduplication",
    icon: "📄",
    disabled: !props.hasDiscoveredFiles(),
    onSelect: () => props.onDeduplication?.(),
  },
];

// ---------- Settings ----------

export const settingsMenuItems = (props: SidebarProps): ContextMenuItem[] => [
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

// ---------- Help ----------

export const helpMenuItems = (props: SidebarProps): ContextMenuItem[] => [
  {
    id: "help-shortcuts",
    label: "Keyboard Shortcuts",
    icon: "⌨️",
    shortcut: "?",
    onSelect: () => props.onHelp?.(),
  },
  {
    id: "help-guide",
    label: "User Guide",
    icon: "📖",
    onSelect: () => props.onOpenHelpTab?.(),
  },
  { id: "help-sep", label: "", separator: true },
  {
    id: "help-command",
    label: "Command Palette",
    icon: "🔧",
    shortcut: "⌘K",
    onSelect: () => props.onCommandPalette?.(),
  },
];
