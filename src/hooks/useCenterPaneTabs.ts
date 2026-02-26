// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useCenterPaneTabs - Manages unified center pane tab state
 * 
 * Consolidates tab management for all content types:
 * - Evidence files
 * - Case documents
 * - Container entries
 * - Export panel
 * - Processed databases
 */

import { createSignal, createMemo, type Accessor, type Setter } from "solid-js";
import type { CenterTab, CenterTabType, CenterPaneViewMode } from "../components/layout/CenterPane";
import type { DiscoveredFile, CaseDocument, ProcessedDatabase } from "../types";
import type { SelectedEntry } from "../components/EvidenceTree";
import { createDocumentEntry } from "./project/projectHelpers";

// =============================================================================
// Types
// =============================================================================

export interface CenterPaneTabsState {
  // Tab state
  tabs: Accessor<CenterTab[]>;
  setTabs: Setter<CenterTab[]>;
  activeTabId: Accessor<string | null>;
  setActiveTabId: Setter<string | null>;
  
  // View mode
  viewMode: Accessor<CenterPaneViewMode>;
  setViewMode: Setter<CenterPaneViewMode>;
  
  // Active tab helpers
  activeTab: Accessor<CenterTab | null>;
  activeTabType: Accessor<CenterTabType | null>;
  
  // Tab operations
  openEvidenceFile: (file: DiscoveredFile) => void;
  openCaseDocument: (doc: CaseDocument) => void;
  openContainerEntry: (entry: SelectedEntry) => void;
  openProcessedDatabase: (db: ProcessedDatabase) => void;
  openExportTab: () => void;
  openEvidenceCollection: (collectionId?: string, readOnly?: boolean) => void;
  openEvidenceCollectionList: () => void;
  closeTab: (tabId: string) => void;
  closeAllTabs: () => void;
  
  // Recently closed tracking (prevents immediate re-open)
  recentlyClosed: Accessor<Set<string>>;
  clearRecentlyClosed: () => void;
}

// =============================================================================
// Hook
// =============================================================================

export function useCenterPaneTabs(): CenterPaneTabsState {
  // Core state
  const [tabs, setTabs] = createSignal<CenterTab[]>([]);
  const [activeTabId, setActiveTabId] = createSignal<string | null>(null);
  const [viewMode, setViewMode] = createSignal<CenterPaneViewMode>("info");
  const [recentlyClosed, setRecentlyClosed] = createSignal<Set<string>>(new Set());

  // Derived state
  const activeTab = createMemo(() => {
    const id = activeTabId();
    if (!id) return null;
    return tabs().find(t => t.id === id) ?? null;
  });

  const activeTabType = createMemo(() => activeTab()?.type ?? null);

  // Helper to generate tab ID
  const generateTabId = (type: CenterTabType, path: string) => `${type}:${path}`;

  // Open an evidence file as a tab
  const openEvidenceFile = (file: DiscoveredFile) => {
    const tabId = generateTabId("evidence", file.path);
    
    // Clear recently closed when opening any item
    if (recentlyClosed().size > 0) {
      setRecentlyClosed(new Set<string>());
    }
    
    // Check if tab already exists
    const existing = tabs().find(t => t.id === tabId);
    if (existing) {
      setActiveTabId(tabId);
      setViewMode("info"); // Default view for evidence files
      return;
    }
    
    // Create new tab
    const newTab: CenterTab = {
      id: tabId,
      type: "evidence",
      title: file.filename,
      subtitle: file.container_type,
      file,
      closable: true,
    };
    
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(tabId);
    setViewMode("info");
  };

  // Open a case document as a tab
  const openCaseDocument = (doc: CaseDocument) => {
    const tabId = generateTabId("document", doc.path);
    
    // Clear recently closed when opening any item
    if (recentlyClosed().size > 0) {
      setRecentlyClosed(new Set<string>());
    }
    
    const existing = tabs().find(t => t.id === tabId);
    if (existing) {
      setActiveTabId(tabId);
      setViewMode("document");
      return;
    }
    
    // Create entry for the viewer
    const documentEntry = createDocumentEntry(doc);
    
    const newTab: CenterTab = {
      id: tabId,
      type: "document",
      title: doc.filename,
      documentPath: doc.path,
      documentEntry,
      closable: true,
    };
    
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(tabId);
    setViewMode("document");
  };

  // Open a container entry (file inside a container) as a tab
  const openContainerEntry = (entry: SelectedEntry) => {
    if (entry.isDir) return; // Don't open directories as tabs
    
    const tabId = generateTabId("entry", entry.entryPath);
    
    // Clear recently closed when opening any item
    if (recentlyClosed().size > 0) {
      setRecentlyClosed(new Set<string>());
    }
    
    const existing = tabs().find(t => t.id === tabId);
    if (existing) {
      setActiveTabId(tabId);
      setViewMode("document");
      return;
    }
    
    const newTab: CenterTab = {
      id: tabId,
      type: "entry",
      title: entry.name,
      subtitle: entry.containerPath.split('/').pop(),
      entry,
      closable: true,
    };
    
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(tabId);
    setViewMode("document");
  };

  // Open a processed database as a tab
  const openProcessedDatabase = (db: ProcessedDatabase) => {
    const tabId = generateTabId("processed", db.path);
    
    // Clear recently closed when opening any item
    if (recentlyClosed().size > 0) {
      setRecentlyClosed(new Set<string>());
    }
    
    const existing = tabs().find(t => t.id === tabId);
    if (existing) {
      setActiveTabId(tabId);
      return;
    }
    
    const newTab: CenterTab = {
      id: tabId,
      type: "processed",
      title: db.name || db.case_name || "Database",
      subtitle: db.db_type,
      processedDb: db,
      closable: true,
    };
    
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(tabId);
  };

  // Open the export tab
  const openExportTab = () => {
    const tabId = "__export__";
    
    const existing = tabs().find(t => t.id === tabId);
    if (existing) {
      setActiveTabId(tabId);
      setViewMode("export");
      return;
    }
    
    const newTab: CenterTab = {
      id: tabId,
      type: "export",
      title: "Export",
      closable: true,
    };
    
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(tabId);
    setViewMode("export");
  };

  // Open an evidence collection form as a tab
  const openEvidenceCollection = (collectionId?: string, readOnly?: boolean) => {
    const tabId = collectionId ? `collection:${collectionId}` : "__collection_new__";
    
    // Clear recently closed when opening any item
    if (recentlyClosed().size > 0) {
      setRecentlyClosed(new Set<string>());
    }
    
    const existing = tabs().find(t => t.id === tabId);
    if (existing) {
      setActiveTabId(tabId);
      return;
    }
    
    const newTab: CenterTab = {
      id: tabId,
      type: "collection",
      title: collectionId ? "Evidence Collection" : "New Collection",
      subtitle: readOnly ? "Review" : undefined,
      collectionId,
      collectionReadOnly: readOnly ?? false,
      collectionListView: false,
      closable: true,
    };
    
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(tabId);
  };

  // Open the evidence collection list/browse view as a tab
  const openEvidenceCollectionList = () => {
    const tabId = "__collection_list__";
    
    // Clear recently closed when opening any item
    if (recentlyClosed().size > 0) {
      setRecentlyClosed(new Set<string>());
    }
    
    const existing = tabs().find(t => t.id === tabId);
    if (existing) {
      setActiveTabId(tabId);
      return;
    }
    
    const newTab: CenterTab = {
      id: tabId,
      type: "collection",
      title: "Evidence Collections",
      collectionListView: true,
      closable: true,
    };
    
    setTabs(prev => [...prev, newTab]);
    setActiveTabId(tabId);
  };

  // Close a tab
  const closeTab = (tabId: string) => {
    const currentTabs = tabs();
    const tabIndex = currentTabs.findIndex(t => t.id === tabId);
    if (tabIndex === -1) return;
    
    // Mark as recently closed
    setRecentlyClosed(prev => new Set([...prev, tabId]));
    
    // Remove the tab
    const newTabs = currentTabs.filter(t => t.id !== tabId);
    setTabs(newTabs);
    
    // If closing active tab, select another
    if (activeTabId() === tabId) {
      if (newTabs.length > 0) {
        const newIndex = Math.min(tabIndex, newTabs.length - 1);
        setActiveTabId(newTabs[newIndex].id);
      } else {
        setActiveTabId(null);
      }
    }
  };

  // Close all tabs
  const closeAllTabs = () => {
    const closedIds = tabs().map(t => t.id);
    setRecentlyClosed(prev => new Set([...prev, ...closedIds]));
    setTabs([]);
    setActiveTabId(null);
  };

  // Clear recently closed tracking
  const clearRecentlyClosed = () => {
    setRecentlyClosed(new Set<string>());
  };

  return {
    tabs,
    setTabs,
    activeTabId,
    setActiveTabId,
    viewMode,
    setViewMode,
    activeTab,
    activeTabType,
    openEvidenceFile,
    openCaseDocument,
    openContainerEntry,
    openProcessedDatabase,
    openExportTab,
    openEvidenceCollection,
    openEvidenceCollectionList,
    closeTab,
    closeAllTabs,
    recentlyClosed,
    clearRecentlyClosed,
  };
}

export type { CenterTab, CenterTabType, CenterPaneViewMode };
