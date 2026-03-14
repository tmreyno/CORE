// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Sidebar — Vertical icon bar for the left panel.
 *
 * Sub-components:
 *   - SidebarButton (icon button with optional badge)
 *   - SectionDivider (thin hr with optional label)
 *   - contextMenuItems (12 menu builders)
 */

import { type Component, Show } from "solid-js";
import { ThemeSwitcher } from "../../";
import { ContextMenu, createContextMenu } from "../../ContextMenu";
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
  HiOutlineCircleStack,
} from "../../icons";
import { isFullEdition } from "../../../utils/edition";
import type { SidebarProps } from "./types";
import { SidebarButton } from "./SidebarButton";
import { SectionDivider } from "./SectionDivider";
import {
  viewModeMenuItems,
  dashboardMenuItems,
  evidenceMenuItems,
  processedMenuItems,
  caseDocsMenuItems,
  activityMenuItems,
  bookmarkMenuItems,
  reportMenuItems,
  exportMenuItems,
  searchMenuItems,
  settingsMenuItems,
  helpMenuItems,
} from "./contextMenuItems";

export const Sidebar: Component<SidebarProps> = (props) => {
  const toggleViewMode = () => {
    props.onViewModeChange(props.viewMode() === "tabs" ? "unified" : "tabs");
  };

  /** Returns true when the module is enabled (or when no mode filter is active). */
  const mod = (m: import("../../preferences").FeatureModule) =>
    !props.isModuleEnabled || props.isModuleEnabled(m);

  const contextMenu = createContextMenu();

  return (
    <div class="flex flex-col items-center gap-0.5 py-2 pl-2 pr-1 bg-bg-secondary border-r border-border h-full w-12 min-w-12">
      {/* View Mode Toggle */}
      <SidebarButton
        onClick={toggleViewMode}
        onContextMenu={(e) => contextMenu.open(e, viewModeMenuItems(props))}
        title={props.viewMode() === "tabs" ? "Switch to Unified View" : "Switch to Tab View"}
      >
        <Show when={props.viewMode() === "tabs"} fallback={<HiOutlineSquares2x2 class="w-4 h-4" />}>
          <HiOutlineQueueList class="w-4 h-4" />
        </Show>
      </SidebarButton>

      <SectionDivider />

      {/* Navigation (in tab mode) */}
      <Show when={props.viewMode() === "tabs"}>
        <Show when={isFullEdition() && mod("caseManagement")}>
          <SidebarButton
            active={props.activeTab() === "dashboard"}
            onClick={() => props.onTabChange("dashboard")}
            onContextMenu={(e) => contextMenu.open(e, dashboardMenuItems(props))}
            title="Project Dashboard"
          >
            <HiOutlineRectangleGroup class="w-4 h-4" />
          </SidebarButton>
        </Show>

        <Show when={mod("forensicExplorer")}>
          <SidebarButton
            active={props.activeTab() === "evidence"}
            onClick={() => props.onTabChange("evidence")}
            onContextMenu={(e) => contextMenu.open(e, evidenceMenuItems(props))}
            title="Evidence Containers"
          >
            <HiOutlineArchiveBox class="w-4 h-4" />
          </SidebarButton>
        </Show>

        <Show when={isFullEdition() && mod("searchAnalysis")}>
          <SidebarButton
            active={props.activeTab() === "processed"}
            onClick={() => props.onTabChange("processed")}
            onContextMenu={(e) => contextMenu.open(e, processedMenuItems(props))}
            title="Processed Databases"
          >
            <HiOutlineChartBar class="w-4 h-4" />
          </SidebarButton>
        </Show>

        <Show when={isFullEdition() && mod("documentReview")}>
          <SidebarButton
            active={props.activeTab() === "casedocs"}
            onClick={() => props.onTabChange("casedocs")}
            onContextMenu={(e) => contextMenu.open(e, caseDocsMenuItems(props))}
            title="Case Documents"
          >
            <HiOutlineClipboardDocumentList class="w-4 h-4" />
          </SidebarButton>
        </Show>

        <Show when={isFullEdition() && mod("caseManagement")}>
          <SidebarButton
            active={props.activeTab() === "activity"}
            onClick={() => props.onTabChange("activity")}
            onContextMenu={(e) => contextMenu.open(e, activityMenuItems(props))}
            title="Activity Timeline"
          >
            <HiOutlineClock class="w-4 h-4" />
          </SidebarButton>
        </Show>

        <SidebarButton
          active={props.activeTab() === "bookmarks"}
          onClick={() => props.onTabChange("bookmarks")}
          onContextMenu={(e) => contextMenu.open(e, bookmarkMenuItems(props))}
          title="Bookmarks & Notes"
          badge={(props.bookmarkCount?.() || 0) + (props.noteCount?.() || 0) || undefined}
          badgeColor="accent"
        >
          <HiOutlineBookmark class="w-4 h-4" />
        </SidebarButton>

        <Show when={mod("reportExport")}>
          <SidebarButton
            active={props.activeTab() === "drives"}
            onClick={() => props.onTabChange("drives")}
            title="Drives & Volumes"
          >
            <HiOutlineCircleStack class="w-4 h-4" />
          </SidebarButton>
        </Show>

        <SectionDivider />
      </Show>

      {/* Spacer */}
      <div class="flex-1" />

      {/* Tools */}
      <SectionDivider />

      <SidebarButton
        onClick={props.onSearch}
        onContextMenu={(e) => { if (props.hasProject?.()) contextMenu.open(e, searchMenuItems(props)); }}
        disabled={!props.hasProject?.()}
        title={props.hasProject?.() ? "Search" : "Search (open a project first)"}
        shortcut="⌘F"
      >
        <HiOutlineMagnifyingGlass class="w-4 h-4" />
      </SidebarButton>

      <Show when={isFullEdition() && props.onDeduplication && mod("searchAnalysis")}>
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

      {/* Project Actions */}
      <Show when={mod("reportExport")}>
        <SidebarButton
          onClick={props.onExport}
          onContextMenu={(e) => { if (props.hasProject?.()) contextMenu.open(e, exportMenuItems(props)); }}
          disabled={!props.hasProject?.()}
          title={props.hasProject?.() ? "Export Files" : "Export Files (open a project first)"}
        >
          <HiOutlineArrowUpTray class="w-4 h-4" />
        </SidebarButton>

        <Show when={isFullEdition()}>
          <SidebarButton
            onClick={props.onReport}
            onContextMenu={(e) => { if (props.hasProject?.()) contextMenu.open(e, reportMenuItems(props)); }}
            disabled={!props.hasProject?.()}
            title={props.hasProject?.() ? "Generate Report" : "Generate Report (open a project first)"}
            shortcut="⌘P"
          >
            <HiOutlineClipboardDocumentList class="w-4 h-4" />
          </SidebarButton>
        </Show>
      </Show>

      <SectionDivider />

      {/* Utility */}
      <ThemeSwitcher
        compact
        theme={props.theme}
        resolvedTheme={props.resolvedTheme}
        cycleTheme={props.cycleTheme}
      />

      <SidebarButton
        onClick={props.onSettings}
        onContextMenu={(e) => contextMenu.open(e, settingsMenuItems(props))}
        title="Settings"
        shortcut="⌘,"
      >
        <HiOutlineCog6Tooth class="w-4 h-4" />
      </SidebarButton>

      <Show when={props.onHelp}>
        <SidebarButton
          onClick={props.onHelp}
          onContextMenu={(e) => contextMenu.open(e, helpMenuItems(props))}
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
