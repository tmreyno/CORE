// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Build save-options payload for project persistence (.cffx).
 *
 * Used by auto-save, Cmd+S, and the manual save button.
 */

import type { Accessor } from "solid-js";
import type { OpenTab, TabViewMode, SelectedEntry, TreeExpansionState } from "../../components";
import type { CenterTab, CenterPaneViewMode } from "../../components/layout/CenterPane";
import type { CaseDocument } from "../../types";
import type { ProjectTabType } from "../../types/project";
import type { useFileManager, useHashManager, useProcessedDatabases } from "../../hooks";
import type { LeftPanelTab } from "../../components";
import type { CenterTabForSave } from "./types";

// ─── Params Interface ────────────────────────────────────────────────────────

export interface BuildSaveOptionsParams {
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  /** @deprecated Use centerTabs instead */
  openTabs?: Accessor<OpenTab[]>;
  /** New: Center pane tabs (unified system) */
  centerTabs?: Accessor<CenterTab[]>;
  /** New: Active tab ID */
  activeTabId?: Accessor<string | null>;
  /** New: View mode */
  viewMode?: Accessor<CenterPaneViewMode>;
  selectedContainerEntry: Accessor<SelectedEntry | null>;
  leftWidth: Accessor<number>;
  rightWidth: Accessor<number>;
  leftCollapsed: Accessor<boolean>;
  rightCollapsed: Accessor<boolean>;
  leftPanelTab: Accessor<LeftPanelTab>;
  currentViewMode: Accessor<TabViewMode>;
  entryContentViewMode: Accessor<"auto" | "hex" | "text" | "document">;
  caseDocumentsPath: Accessor<string | null>;
  treeExpansionState: Accessor<TreeExpansionState | null>;
  caseDocuments: Accessor<CaseDocument[] | null>;
}

// ─── Build Save Options ──────────────────────────────────────────────────────

/**
 * Build the save options object for project save operations.
 * Used by auto-save, Cmd+S, and manual save button.
 *
 * Saves the following state:
 * - Root path (scan directory)
 * - Open tabs (evidence, documents, entries, processed databases)
 * - Active tab ID and view mode
 * - Hash history for all files
 * - Processed databases state (including detail view type)
 * - UI state:
 *   - Panel dimensions and collapse states
 *   - Active left panel tab
 *   - Detail view mode
 *   - Selected container entry (for resuming work)
 *   - Entry content view mode (auto/hex/text/document)
 *   - Case documents path
 *   - Tree expansion state (which containers/folders are expanded)
 * - Filter state (type filter for evidence tree)
 * - Evidence cache (discovered files, container info, computed hashes)
 * - Processed database cache (full database objects, AXIOM case info, artifacts)
 * - Case documents cache (discovered documents and search path)
 */
export function buildSaveOptions(params: BuildSaveOptionsParams) {
  const scanDir = params.fileManager.scanDir();
  if (!scanDir) return null;

  // Capture selected entry for restoration
  const entry = params.selectedContainerEntry();
  const selectedEntryData = entry
    ? {
        containerPath: entry.containerPath,
        entryPath: entry.entryPath,
        name: entry.name,
      }
    : null;

  // Convert CenterTabs to serializable format
  const centerTabs: CenterTabForSave[] = params.centerTabs?.()
    ? params.centerTabs().map((tab) => ({
        id: tab.id,
        type: tab.type as ProjectTabType,
        title: tab.title,
        subtitle: tab.subtitle,
        file: tab.file,
        documentPath: tab.documentPath,
        entry: tab.entry,
        processedDb: tab.processedDb,
      }))
    : [];

  // Also include legacy openTabs for backwards compatibility if no centerTabs
  const legacyOpenTabs = params.openTabs?.() || [];

  return {
    rootPath: scanDir,
    // New center tabs system
    centerTabs: centerTabs.length > 0 ? centerTabs : undefined,
    activeTabId: params.activeTabId?.() || null,
    viewMode: params.viewMode?.() || "info",
    // Legacy tabs for backwards compatibility
    openTabs: centerTabs.length === 0 ? legacyOpenTabs : [],
    activeTabPath: params.fileManager.activeFile()?.path || null,
    hashHistory: params.hashManager.hashHistory(),
    processedDatabases: params.processedDbManager.databases(),
    selectedProcessedDb: params.processedDbManager.selectedDatabase(),
    uiState: {
      left_panel_width: params.leftWidth(),
      right_panel_width: params.rightWidth(),
      left_panel_collapsed: params.leftCollapsed(),
      right_panel_collapsed: params.rightCollapsed(),
      left_panel_tab: params.leftPanelTab(),
      detail_view_mode: params.currentViewMode(),
      // New fields for improved restoration
      selected_entry: selectedEntryData,
      entry_content_view_mode: params.entryContentViewMode(),
      case_documents_path: params.caseDocumentsPath() || undefined,
      // Tree expansion state for restoring which containers/folders are expanded
      tree_expansion_state: params.treeExpansionState() || undefined,
    },
    // Save filter state for evidence tree
    filterState: {
      type_filter: params.fileManager.typeFilter(),
      status_filter: null,
      search_query: null,
      sort_by: "name",
      sort_direction: "asc" as const,
    },
    evidenceCache: {
      discoveredFiles: params.fileManager.discoveredFiles(),
      fileInfoMap: params.fileManager.fileInfoMap(),
      fileHashMap: params.hashManager.fileHashMap(),
    },
    processedDbCache: {
      databases: params.processedDbManager.databases(),
      axiomCaseInfo: params.processedDbManager.axiomCaseInfo(),
      artifactCategories: params.processedDbManager.artifactCategories(),
      detailViewType: params.processedDbManager.detailView()?.type || null,
    },
    caseDocumentsCache: params.caseDocuments()
      ? {
          documents: params.caseDocuments()!,
          searchPath: params.caseDocumentsPath() || scanDir,
        }
      : undefined,
  };
}
