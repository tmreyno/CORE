// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createMemo } from "solid-js";
import type { CenterPaneProps, CenterPaneViewMode, CenterTab } from "./types";

export function useCenterPaneLogic(props: CenterPaneProps) {
  /** Available view modes for the current active tab type */
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
        return [];
      case "export":
        return ["export"];
      case "collection":
        return [];
      case "help":
        return [];
      default:
        return [];
    }
  });
  
  /** Container-level tabs (evidence, processed, export, document, collection, help) */
  const containerTabs = createMemo(() => 
    props.tabs().filter(t => t.type === "evidence" || t.type === "processed" || t.type === "export" || t.type === "document" || t.type === "collection" || t.type === "help")
  );
  
  /** Entry-level tabs (files inside containers) */
  const entryTabs = createMemo(() => 
    props.tabs().filter(t => t.type === "entry")
  );
  
  /** Currently active container tab (direct or parent of active entry) */
  const activeContainerTab = createMemo((): CenterTab | null => {
    const activeId = props.activeTabId();
    if (!activeId) return null;
    
    const containerTab = containerTabs().find(t => t.id === activeId);
    if (containerTab) return containerTab;
    
    const entryTab = entryTabs().find(t => t.id === activeId);
    if (entryTab?.entry) {
      return containerTabs().find(t => 
        t.type === "evidence" && t.file?.path === entryTab.entry?.containerPath
      ) || null;
    }
    
    return null;
  });
  
  /** Entry tabs belonging to the currently active container */
  const entriesForActiveContainer = createMemo(() => {
    const container = activeContainerTab();
    if (!container || container.type !== "evidence") return [];
    
    return entryTabs().filter(t => 
      t.entry?.containerPath === container.file?.path
    );
  });
  
  /** Whether we're currently viewing an entry (not the container itself) */
  const isViewingEntry = createMemo(() => {
    const activeId = props.activeTabId();
    return entryTabs().some(t => t.id === activeId);
  });

  const handleTabSelect = (tabId: string) => {
    props.onTabSelect(tabId);
    
    const tab = props.tabs().find(t => t.id === tabId);
    if (tab) {
      const modes = availableViewModes();
      if (modes.length > 0 && !modes.includes(props.viewMode())) {
        props.onViewModeChange(modes[0]);
      }
    }
  };

  const handleTabClose = (tabId: string) => {
    props.onTabClose(tabId);
  };
  
  const tabCount = createMemo(() => props.tabs().length);
  const hasMultipleTabs = createMemo(() => tabCount() > 1);

  return {
    availableViewModes,
    containerTabs,
    entryTabs,
    activeContainerTab,
    entriesForActiveContainer,
    isViewingEntry,
    handleTabSelect,
    handleTabClose,
    tabCount,
    hasMultipleTabs,
  };
}
