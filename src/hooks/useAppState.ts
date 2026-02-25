// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAppState - Manages all UI state for the application using solid-js/store.
 * 
 * Uses createStore for efficient nested state updates and fine-grained reactivity.
 * 
 * Consolidates:
 * - Modal visibility states (command palette, settings, shortcuts, etc.)
 * - View modes (tab view, content view)
 * - Panel tabs
 * - Selected entries and tabs
 * - Project wizard state
 * - Case documents state
 */

import { createSignal, type Accessor, type Setter } from "solid-js";
import type { OpenTab, ParsedMetadata, SelectedEntry, TabViewMode, TreeExpansionState } from "../components";
import type { CaseDocument } from "../types";
import { getPreference } from "../components/preferences";

// Note: solid-js/store is available for complex nested state - see useAppStore.ts
// This module uses createSignal for compatibility with existing codebase

// =============================================================================
// Types
// =============================================================================

export interface ModalState {
  showCommandPalette: Accessor<boolean>;
  setShowCommandPalette: Setter<boolean>;
  showShortcutsModal: Accessor<boolean>;
  setShowShortcutsModal: Setter<boolean>;
  showPerformancePanel: Accessor<boolean>;
  setShowPerformancePanel: Setter<boolean>;
  showSettingsPanel: Accessor<boolean>;
  setShowSettingsPanel: Setter<boolean>;
  showSearchPanel: Accessor<boolean>;
  setShowSearchPanel: Setter<boolean>;
  showWelcomeModal: Accessor<boolean>;
  setShowWelcomeModal: Setter<boolean>;
  showReportWizard: Accessor<boolean>;
  setShowReportWizard: Setter<boolean>;
  showProjectWizard: Accessor<boolean>;
  setShowProjectWizard: Setter<boolean>;
  showEvidenceCollection: Accessor<boolean>;
  setShowEvidenceCollection: Setter<boolean>;
}

export interface ViewState {
  openTabs: Accessor<OpenTab[]>;
  setOpenTabs: Setter<OpenTab[]>;
  currentViewMode: Accessor<TabViewMode>;
  setCurrentViewMode: Setter<TabViewMode>;
  hexMetadata: Accessor<ParsedMetadata | null>;
  setHexMetadata: Setter<ParsedMetadata | null>;
  selectedContainerEntry: Accessor<SelectedEntry | null>;
  setSelectedContainerEntry: Setter<SelectedEntry | null>;
  entryContentViewMode: Accessor<"auto" | "hex" | "text" | "document">;
  setEntryContentViewMode: Setter<"auto" | "hex" | "text" | "document">;
  requestViewMode: Accessor<"info" | "hex" | "text" | "pdf" | "export" | null>;
  setRequestViewMode: Setter<"info" | "hex" | "text" | "pdf" | "export" | null>;
  hexNavigator: Accessor<((offset: number, size?: number) => void) | null>;
  setHexNavigator: Setter<((offset: number, size?: number) => void) | null>;
  treeExpansionState: Accessor<TreeExpansionState | null>;
  setTreeExpansionState: Setter<TreeExpansionState | null>;
}

export interface ProjectState {
  pendingProjectRoot: Accessor<string | null>;
  setPendingProjectRoot: Setter<string | null>;
  caseDocumentsPath: Accessor<string | null>;
  setCaseDocumentsPath: Setter<string | null>;
  caseDocuments: Accessor<CaseDocument[] | null>;
  setCaseDocuments: Setter<CaseDocument[] | null>;
}

/** View mode for left panel - tabs (separate) or unified (collapsible sections) */
export type LeftPanelMode = "tabs" | "unified";

export interface LeftPanelState {
  leftPanelTab: Accessor<"dashboard" | "evidence" | "processed" | "casedocs" | "activity" | "bookmarks">;
  setLeftPanelTab: Setter<"dashboard" | "evidence" | "processed" | "casedocs" | "activity" | "bookmarks">;
  leftPanelMode: Accessor<LeftPanelMode>;
  setLeftPanelMode: Setter<LeftPanelMode>;
}

export interface AppState {
  modals: ModalState;
  views: ViewState;
  project: ProjectState;
  leftPanel: LeftPanelState;
}

// =============================================================================
// Hook
// =============================================================================

export function useAppState(): AppState {
  // ---------------------------------------------------------------------------
  // Modal State
  // ---------------------------------------------------------------------------
  const [showCommandPalette, setShowCommandPalette] = createSignal(false);
  const [showShortcutsModal, setShowShortcutsModal] = createSignal(false);
  const [showPerformancePanel, setShowPerformancePanel] = createSignal(false);
  const [showSettingsPanel, setShowSettingsPanel] = createSignal(false);
  const [showSearchPanel, setShowSearchPanel] = createSignal(false);
  const [showWelcomeModal, setShowWelcomeModal] = createSignal(false);
  const [showReportWizard, setShowReportWizard] = createSignal(false);
  const [showProjectWizard, setShowProjectWizard] = createSignal(false);
  const [showEvidenceCollection, setShowEvidenceCollection] = createSignal(false);

  // ---------------------------------------------------------------------------
  // View State
  // ---------------------------------------------------------------------------
  const [openTabs, setOpenTabs] = createSignal<OpenTab[]>([]);
  const [currentViewMode, setCurrentViewMode] = createSignal<TabViewMode>("info");
  const [hexMetadata, setHexMetadata] = createSignal<ParsedMetadata | null>(null);
  const [selectedContainerEntry, setSelectedContainerEntry] = createSignal<SelectedEntry | null>(null);
  
  // Map preference ViewMode to entryContentViewMode: preview -> document
  const initialViewMode = (() => {
    const prefMode = getPreference("defaultViewMode");
    if (prefMode === "preview") return "document";
    if (prefMode === "auto" || prefMode === "hex" || prefMode === "text") return prefMode;
    return "hex";
  })();
  
  const [entryContentViewMode, setEntryContentViewMode] = createSignal<"auto" | "hex" | "text" | "document">(initialViewMode);
  const [requestViewMode, setRequestViewMode] = createSignal<"info" | "hex" | "text" | "pdf" | "export" | null>(null);
  const [hexNavigator, setHexNavigator] = createSignal<((offset: number, size?: number) => void) | null>(null);
  const [treeExpansionState, setTreeExpansionState] = createSignal<TreeExpansionState | null>(null);

  // ---------------------------------------------------------------------------
  // Project State
  // ---------------------------------------------------------------------------
  const [pendingProjectRoot, setPendingProjectRoot] = createSignal<string | null>(null);
  const [caseDocumentsPath, setCaseDocumentsPath] = createSignal<string | null>(null);
  const [caseDocuments, setCaseDocuments] = createSignal<CaseDocument[] | null>(null);

  // ---------------------------------------------------------------------------
  // Left Panel State
  // ---------------------------------------------------------------------------
  const [leftPanelTab, setLeftPanelTab] = createSignal<"dashboard" | "evidence" | "processed" | "casedocs" | "activity" | "bookmarks">("dashboard");
  const [leftPanelMode, setLeftPanelMode] = createSignal<LeftPanelMode>("tabs");

  return {
    modals: {
      showCommandPalette,
      setShowCommandPalette,
      showShortcutsModal,
      setShowShortcutsModal,
      showPerformancePanel,
      setShowPerformancePanel,
      showSettingsPanel,
      setShowSettingsPanel,
      showSearchPanel,
      setShowSearchPanel,
      showWelcomeModal,
      setShowWelcomeModal,
      showReportWizard,
      setShowReportWizard,
      showProjectWizard,
      setShowProjectWizard,
      showEvidenceCollection,
      setShowEvidenceCollection,
    },
    views: {
      openTabs,
      setOpenTabs,
      currentViewMode,
      setCurrentViewMode,
      hexMetadata,
      setHexMetadata,
      selectedContainerEntry,
      setSelectedContainerEntry,
      entryContentViewMode,
      setEntryContentViewMode,
      requestViewMode,
      setRequestViewMode,
      hexNavigator,
      setHexNavigator,
      treeExpansionState,
      setTreeExpansionState,
    },
    project: {
      pendingProjectRoot,
      setPendingProjectRoot,
      caseDocumentsPath,
      setCaseDocumentsPath,
      caseDocuments,
      setCaseDocuments,
    },
    leftPanel: {
      leftPanelTab,
      setLeftPanelTab,
      leftPanelMode,
      setLeftPanelMode,
    },
  };
}
