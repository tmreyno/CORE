// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component, Accessor, JSX } from "solid-js";
import type { DiscoveredFile, ProcessedDatabase } from "../../../types";
import type { SelectedEntry } from "../../EvidenceTree";

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
