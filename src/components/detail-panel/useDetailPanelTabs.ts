// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useDetailPanelTabs — manages tab state, open/close/move/select,
 * view mode per tab, export tab, and parent notifications.
 */

import { createSignal, createEffect } from "solid-js";
import type { DiscoveredFile } from "../../types";
import type { TabViewMode, OpenTab } from "../TabBar";

/** Special tab ID for the Export panel. */
export const EXPORT_TAB_ID = "__export__";

interface UseDetailPanelTabsOptions {
  /** Reactive getter for activeFile from the file manager. */
  activeFile: () => DiscoveredFile | null;
  /** Called when a tab is selected (or null when all tabs closed). */
  onTabSelect: (file: DiscoveredFile | null) => void;
  /** Notified whenever the tab list changes. */
  onTabsChange?: (tabs: OpenTab[]) => void;
  /** Notified whenever the active view mode changes. */
  onViewModeChange?: (mode: TabViewMode) => void;
  /** An optional view mode request from the parent (e.g. toolbar click). */
  requestViewMode?: () => TabViewMode | null | undefined;
  /** Called after the view mode request is processed. */
  onViewModeRequestHandled?: () => void;
}

export function useDetailPanelTabs(opts: UseDetailPanelTabsOptions) {
  const [openTabs, setOpenTabs] = createSignal<OpenTab[]>([]);
  const [activeTabId, setActiveTabId] = createSignal<string | null>(null);
  const [recentlyClosed, setRecentlyClosed] = createSignal<Set<string>>(new Set());
  const [tabViewModes, setTabViewModes] = createSignal<Map<string, TabViewMode>>(new Map());
  const [globalViewMode, setGlobalViewMode] = createSignal<TabViewMode | null>(null);

  // Notify parent of tab changes
  createEffect(() => {
    const tabs = openTabs();
    opts.onTabsChange?.(tabs);
  });

  // Handle view mode request from parent
  createEffect(() => {
    const requestedMode = opts.requestViewMode?.();
    if (!requestedMode) return;

    if (requestedMode === "export") {
      openExportTab();
    } else {
      const id = activeTabId();
      if (id) {
        setGlobalViewMode(null);
        setTabViewModes((prev) => {
          const next = new Map(prev);
          next.set(id, requestedMode);
          return next;
        });
        opts.onViewModeChange?.(requestedMode);
      }
    }
    opts.onViewModeRequestHandled?.();
  });

  // When activeFile changes, open it as a tab
  createEffect(() => {
    const file = opts.activeFile();
    if (!file) return;

    if (recentlyClosed().has(file.path)) return;

    if (recentlyClosed().size > 0) {
      setRecentlyClosed(new Set<string>());
    }
    if (globalViewMode()) {
      setGlobalViewMode(null);
    }

    const tabs = openTabs();
    const currentActiveId = activeTabId();
    const existingTabIndex = tabs.findIndex((t) => t.id === file.path);

    if (existingTabIndex >= 0) {
      if (currentActiveId !== file.path) {
        setActiveTabId(file.path);
      }
    } else {
      const newTab: OpenTab = { file, id: file.path };
      setOpenTabs([...tabs, newTab]);
      setActiveTabId(file.path);
    }
  });

  // ---- Helpers ----

  const activeTab = () => {
    const id = activeTabId();
    if (!id) return null;
    return openTabs().find((t) => t.id === id) ?? null;
  };

  const activeTabFile = () => activeTab()?.file ?? null;

  const closeTab = (tabId: string, e?: MouseEvent) => {
    e?.stopPropagation();
    e?.preventDefault();

    const tabs = openTabs();
    const tabIndex = tabs.findIndex((t) => t.id === tabId);
    if (tabIndex === -1) return;

    setRecentlyClosed((prev) => new Set([...prev, tabId]));

    const newTabs = tabs.filter((t) => t.id !== tabId);
    setOpenTabs(newTabs);

    if (activeTabId() === tabId) {
      if (newTabs.length > 0) {
        const newActiveIndex = Math.min(tabIndex, newTabs.length - 1);
        const newActiveTab = newTabs[newActiveIndex];
        setActiveTabId(newActiveTab.id);
        opts.onTabSelect(newActiveTab.file);
      } else {
        setActiveTabId(null);
        opts.onTabSelect(null);
      }
    }
  };

  const closeOtherTabs = (keepTabId: string) => {
    const tabs = openTabs();
    const keepTab = tabs.find((t) => t.id === keepTabId);
    if (keepTab) {
      const closedIds = tabs.filter((t) => t.id !== keepTabId).map((t) => t.id);
      setRecentlyClosed((prev) => new Set([...prev, ...closedIds]));
      setOpenTabs([keepTab]);
      setActiveTabId(keepTabId);
      opts.onTabSelect(keepTab.file);
    }
  };

  const closeAllTabs = () => {
    const closedIds = openTabs().map((t) => t.id);
    setRecentlyClosed((prev) => new Set([...prev, ...closedIds]));
    setOpenTabs([]);
    setActiveTabId(null);
    opts.onTabSelect(null);
  };

  const closeTabsToRight = (tabId: string) => {
    const tabs = openTabs();
    const idx = tabs.findIndex((t) => t.id === tabId);
    if (idx === -1) return;
    const closedIds = tabs.slice(idx + 1).map((t) => t.id);
    setRecentlyClosed((prev) => new Set([...prev, ...closedIds]));
    setOpenTabs(tabs.slice(0, idx + 1));
    const activeId = activeTabId();
    if (activeId && closedIds.includes(activeId)) {
      setActiveTabId(tabId);
      const keepTab = tabs[idx];
      if (keepTab) opts.onTabSelect(keepTab.file);
    }
  };

  const copyPath = (path: string) => {
    navigator.clipboard.writeText(path).catch(() => {});
  };

  const selectTab = (tab: OpenTab) => {
    if (globalViewMode()) setGlobalViewMode(null);
    setActiveTabId(tab.id);
    opts.onTabSelect(tab.file);
  };

  const moveTab = (fromIndex: number, toIndex: number) => {
    if (fromIndex === toIndex) return;
    const tabs = [...openTabs()];
    const [movedTab] = tabs.splice(fromIndex, 1);
    tabs.splice(toIndex, 0, movedTab);
    setOpenTabs(tabs);
  };

  // ---- View mode ----

  const getActiveViewMode = (): TabViewMode => {
    const global = globalViewMode();
    if (global) return global;
    const id = activeTabId();
    if (!id) return "info";
    return tabViewModes().get(id) ?? "info";
  };

  const openExportTab = () => {
    const tabs = openTabs();
    if (!tabs.find((t) => t.id === EXPORT_TAB_ID)) {
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
    setActiveTabId(EXPORT_TAB_ID);
    setTabViewModes((prev) => {
      const next = new Map(prev);
      next.set(EXPORT_TAB_ID, "export");
      return next;
    });
    opts.onViewModeChange?.("export");
  };

  const setActiveViewMode = (mode: TabViewMode) => {
    if (mode === "export") {
      openExportTab();
      return;
    }
    setGlobalViewMode(null);
    const id = activeTabId();
    if (!id) return;
    setTabViewModes((prev) => {
      const next = new Map(prev);
      next.set(id, mode);
      return next;
    });
    opts.onViewModeChange?.(mode);
  };

  return {
    openTabs,
    activeTabId,
    activeTab,
    activeTabFile,
    getActiveViewMode,
    setActiveViewMode,
    selectTab,
    closeTab,
    closeOtherTabs,
    closeAllTabs,
    closeTabsToRight,
    moveTab,
    copyPath,
  };
}
