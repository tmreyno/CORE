// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useProjectActions — Project save / load / directory handling
 *
 * Extracts the following action handlers from App.tsx into a reusable hook:
 *  - getSaveOptions()        — build the serialisable options blob
 *  - handleSaveProject()     — save to current path (or show dialog)
 *  - handleSaveProjectAs()   — always show save-as dialog
 *  - handleLoadProject()     — load project from picker or specific path
 *  - handleOpenDirectory()   — open evidence directory picker
 *  - handleProjectSetupComplete() — handle project wizard completion
 *
 * Consumers get a single `projectActions` object they can thread into
 * child hooks and template callbacks.
 */

import type { Accessor, Setter } from "solid-js";
import { logger } from "../utils/logger";
import {
  buildSaveOptions,
  handleLoadProject as loadProjectHandler,
  handleOpenDirectory as openDirectoryHandler,
  handleProjectSetupComplete as projectSetupHandler,
} from "./project/projectHelpers";
import type { useFileManager } from "./useFileManager";
import type { useHashManager } from "./useHashManager";
import type { useProject } from "./useProject";
import type { useProcessedDatabases } from "./useProcessedDatabases";
import type { CenterPaneTabsState } from "./useCenterPaneTabs";
import type { ProjectLocations } from "../components";
import type { SelectedEntry } from "../components/EvidenceTree";
import type { TreeExpansionState } from "../components/EvidenceTree/types";
import type { OpenTab, TabViewMode, LeftPanelTab } from "../components";
import type { CaseDocument } from "../types";

const log = logger.scope("ProjectActions");

// ---------------------------------------------------------------------------
// Dependency interface
// ---------------------------------------------------------------------------

export interface UseProjectActionsDeps {
  /** Core managers */
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  projectManager: ReturnType<typeof useProject>;
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  centerPaneTabs: CenterPaneTabsState;
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    warning: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };

  /** UI state — accessors needed by buildSaveOptions */
  openTabs: Accessor<OpenTab[]>;
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

  /** UI state — setters needed by load / setup handlers */
  setLeftWidth: (width: number) => void;
  setRightWidth: (width: number) => void;
  setLeftCollapsed: (collapsed: boolean | ((prev: boolean) => boolean)) => void;
  setRightCollapsed: (collapsed: boolean | ((prev: boolean) => boolean)) => void;
  setLeftPanelTab: Setter<LeftPanelTab>;
  setCurrentViewMode: Setter<TabViewMode>;
  setEntryContentViewMode: Setter<"auto" | "hex" | "text" | "document">;
  setCaseDocumentsPath: Setter<string | null>;
  setTreeExpansionState: Setter<TreeExpansionState | null>;
  setSelectedContainerEntry: Setter<SelectedEntry | null>;
  setOpenTabs: Setter<OpenTab[]>;
  setCaseDocuments: Setter<CaseDocument[] | null>;

  /** Project wizard state */
  setPendingProjectRoot: Setter<string | null>;
  setShowProjectWizard: Setter<boolean>;
}

// ---------------------------------------------------------------------------
// Return type
// ---------------------------------------------------------------------------

export interface ProjectActions {
  /** Build the save-options blob (returns null when no evidence dir open). */
  getSaveOptions: () => ReturnType<typeof buildSaveOptions>;
  /** Save to current project path, or show dialog if none. */
  handleSaveProject: () => Promise<void>;
  /** Always show the save-as dialog. */
  handleSaveProjectAs: () => Promise<void>;
  /** Load a project — from picker if no path, or from the given path. */
  handleLoadProject: (projectPath?: string) => Promise<void>;
  /** Open an evidence directory (shows native picker). */
  handleOpenDirectory: () => void;
  /** Handle project wizard completion. */
  handleProjectSetupComplete: (locations: ProjectLocations) => Promise<void>;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useProjectActions(deps: UseProjectActionsDeps): ProjectActions {
  const {
    fileManager,
    hashManager,
    projectManager,
    processedDbManager,
    centerPaneTabs,
    toast,
  } = deps;

  // ---- getSaveOptions ----
  const getSaveOptions = () =>
    buildSaveOptions({
      fileManager,
      hashManager,
      processedDbManager,
      centerTabs: centerPaneTabs.tabs,
      activeTabId: centerPaneTabs.activeTabId,
      viewMode: centerPaneTabs.viewMode,
      openTabs: deps.openTabs,
      selectedContainerEntry: deps.selectedContainerEntry,
      leftWidth: deps.leftWidth,
      rightWidth: deps.rightWidth,
      leftCollapsed: deps.leftCollapsed,
      rightCollapsed: deps.rightCollapsed,
      leftPanelTab: deps.leftPanelTab,
      currentViewMode: deps.currentViewMode,
      entryContentViewMode: deps.entryContentViewMode,
      caseDocumentsPath: deps.caseDocumentsPath,
      treeExpansionState: deps.treeExpansionState,
      caseDocuments: deps.caseDocuments,
    });

  // ---- handleSaveProject ----
  const handleSaveProject = async () => {
    const options = getSaveOptions();
    if (options) {
      try {
        const existingPath = projectManager.projectPath();
        log.debug(`Save: existingPath=${existingPath}`);
        const result = existingPath
          ? await projectManager.saveProject(options, existingPath)
          : await projectManager.saveProject(options);
        log.debug("Save: result=", result);
        if (result.success) {
          toast.success("Project Saved", "Your project has been saved");
        } else if (result.error && result.error !== "Save cancelled") {
          log.error(`Save failed: ${result.error}`);
          toast.error("Save Failed", result.error);
        }
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        log.error(`Save exception: ${errorMsg}`, err);
        toast.error("Save Failed", errorMsg || "Could not save the project");
      }
    } else {
      log.warn("Save: No evidence directory open");
      toast.error("No Evidence", "Open an evidence directory first");
    }
  };

  // ---- handleSaveProjectAs ----
  const handleSaveProjectAs = async () => {
    const options = getSaveOptions();
    if (options) {
      try {
        const result = await projectManager.saveProject(options);
        if (result.success) {
          toast.success("Project Saved", "Your project has been saved to a new location");
        } else if (result.error && result.error !== "Save cancelled") {
          toast.error("Save Failed", result.error);
        }
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        toast.error("Save Failed", errorMsg || "Could not save the project");
      }
    } else {
      toast.error("No Evidence", "Open an evidence directory first");
    }
  };

  // ---- handleLoadProject ----
  const handleLoadProject = async (projectPath?: string) => {
    await loadProjectHandler({
      fileManager,
      hashManager,
      projectManager,
      processedDbManager,
      setLeftWidth: deps.setLeftWidth,
      setRightWidth: deps.setRightWidth,
      setLeftCollapsed: deps.setLeftCollapsed,
      setRightCollapsed: deps.setRightCollapsed,
      setLeftPanelTab: deps.setLeftPanelTab,
      setCurrentViewMode: deps.setCurrentViewMode,
      setEntryContentViewMode: deps.setEntryContentViewMode,
      setCaseDocumentsPath: deps.setCaseDocumentsPath,
      setTreeExpansionState: deps.setTreeExpansionState,
      setSelectedContainerEntry: deps.setSelectedContainerEntry,
      setOpenTabs: deps.setOpenTabs,
      setCaseDocuments: deps.setCaseDocuments,
      setCenterTabs: centerPaneTabs.setTabs,
      setActiveTabId: centerPaneTabs.setActiveTabId,
      setCenterViewMode: centerPaneTabs.setViewMode,
      toast,
      projectPath,
    });
  };

  // ---- handleOpenDirectory ----
  const handleOpenDirectory = () =>
    openDirectoryHandler({
      setPendingProjectRoot: deps.setPendingProjectRoot,
      setShowProjectWizard: deps.setShowProjectWizard,
      toast,
    });

  // ---- handleProjectSetupComplete ----
  const handleProjectSetupComplete = async (locations: ProjectLocations) =>
    projectSetupHandler(
      {
        fileManager,
        hashManager,
        processedDbManager,
        projectManager,
        setShowProjectWizard: deps.setShowProjectWizard,
        setCaseDocumentsPath: deps.setCaseDocumentsPath,
        setLeftPanelTab: deps.setLeftPanelTab,
        setPendingProjectRoot: deps.setPendingProjectRoot,
        toast,
        getSaveOptions: () => getSaveOptions(),
      },
      locations,
    );

  return {
    getSaveOptions,
    handleSaveProject,
    handleSaveProjectAs,
    handleLoadProject,
    handleOpenDirectory,
    handleProjectSetupComplete,
  };
}
