// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { DiscoveredFile } from "../../types";

export type TabViewMode = "info" | "hex" | "text" | "pdf" | "export";

export interface OpenTab {
  file: DiscoveredFile;
  id: string;
  viewMode?: TabViewMode;
}

export interface ContextMenuState {
  x: number;
  y: number;
  tabId: string;
}

export interface TabBarProps {
  tabs: OpenTab[];
  activeTabId: string | null;
  viewMode: TabViewMode;
  onTabSelect: (tab: OpenTab) => void;
  onTabClose: (tabId: string, e?: MouseEvent) => void;
  onCloseOthers: (tabId: string) => void;
  onCloseAll: () => void;
  onCloseToRight?: (tabId: string) => void;
  onTabMove: (fromIndex: number, toIndex: number) => void;
  onViewModeChange: (mode: TabViewMode) => void;
  onCopyPath?: (path: string) => void;
  onRevealInTree?: (tabId: string) => void;
}
