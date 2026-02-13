// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useEntryNavigation — Manages entry selection and navigation handlers
 * 
 * Extracts handler functions from App.tsx that deal with selecting and opening
 * entries in the center pane: evidence files, container entries, case documents,
 * processed databases, and nested containers.
 */

import type { Setter } from "solid-js";
import { logger } from "../utils/logger";
import type { SelectedEntry } from "../components/EvidenceTree";
import type { DiscoveredFile, CaseDocument } from "../types";
import type { CenterPaneTabsState } from "./useCenterPaneTabs";
import type { FileManager } from "./useFileManager";
import type { ProcessedDatabasesManager } from "./useProcessedDatabases";

const log = logger.scope("EntryNavigation");

export interface UseEntryNavigationDeps {
  /** File manager for selecting/activating files */
  fileManager: Pick<FileManager, "selectAndViewFile" | "addDiscoveredFile" | "setActiveFile">;
  /** Center pane tabs for opening content in tabs */
  centerPaneTabs: Pick<CenterPaneTabsState, "openContainerEntry" | "openEvidenceFile" | "openCaseDocument" | "openProcessedDatabase">;
  /** Processed database manager */
  processedDbManager: Pick<ProcessedDatabasesManager, "selectDatabase">;
  /** Setter for selected container entry (legacy state) */
  setSelectedContainerEntry: Setter<SelectedEntry | null>;
  /** Setter for entry content view mode */
  setEntryContentViewMode: Setter<"auto" | "hex" | "text" | "document">;
  /** Toast notifications */
  toast: { success: (title: string, message?: string) => void };
}

export interface EntryNavigation {
  /** Handle selecting a container entry — opens in center pane tab */
  handleSelectEntry: (entry: SelectedEntry) => void;
  /** Handle selecting an evidence file — opens in center pane tab */
  handleSelectEvidenceFile: (file: DiscoveredFile) => void;
  /** Handle opening a nested container extracted from a parent */
  handleOpenNestedContainer: (tempPath: string, originalName: string, containerType: string, parentPath: string) => void;
  /** Handle selecting a processed database */
  handleSelectProcessedDb: (db: Parameters<ProcessedDatabasesManager["selectDatabase"]>[0]) => void;
  /** Handle selecting a case document — opens in a document tab */
  handleCaseDocumentSelect: (doc: CaseDocument) => void;
  /** Handle viewing a case document in hex mode */
  handleCaseDocViewHex: (doc: CaseDocument) => void;
  /** Handle viewing a case document in text mode */
  handleCaseDocViewText: (doc: CaseDocument) => void;
}

export function useEntryNavigation(deps: UseEntryNavigationDeps): EntryNavigation {
  const {
    fileManager,
    centerPaneTabs,
    processedDbManager,
    setSelectedContainerEntry,
    setEntryContentViewMode,
    toast,
  } = deps;

  /** Handle selecting a container entry - opens in center pane tab */
  const handleSelectEntry = (entry: SelectedEntry) => {
    // Set the entry for legacy views
    setSelectedContainerEntry(entry);

    // Open in unified center pane tab
    centerPaneTabs.openContainerEntry(entry);

    // Use "auto" mode to let ContainerEntryViewer pick the best viewer
    setEntryContentViewMode("auto");

    log.debug(`Selected entry: ${entry.entryPath} from ${entry.containerPath}`);
  };

  /** Handle selecting an evidence file - opens in center pane tab */
  const handleSelectEvidenceFile = (file: DiscoveredFile) => {
    fileManager.selectAndViewFile(file);
    centerPaneTabs.openEvidenceFile(file);
  };

  /** Handle opening a nested container */
  const handleOpenNestedContainer = (
    tempPath: string,
    originalName: string,
    containerType: string,
    parentPath: string,
  ) => {
    const nestedFile: DiscoveredFile = {
      path: tempPath,
      filename: `📦 ${originalName} (from ${parentPath.split("/").pop() || parentPath})`,
      container_type: containerType.toUpperCase(),
      size: 0,
      segment_count: 1,
    };
    fileManager.addDiscoveredFile(nestedFile);
    handleSelectEvidenceFile(nestedFile);
    toast.success("Nested Container", `Opened ${originalName}`);
  };

  /** Handle selecting a processed database */
  const handleSelectProcessedDb = (
    db: Parameters<ProcessedDatabasesManager["selectDatabase"]>[0],
  ) => {
    processedDbManager.selectDatabase(db);
    fileManager.setActiveFile(null);
    // Open in unified center pane
    if (db) {
      centerPaneTabs.openProcessedDatabase(db);
    }
  };

  /** Handle selecting a case document - opens in a document tab */
  const handleCaseDocumentSelect = (doc: CaseDocument) => {
    centerPaneTabs.openCaseDocument(doc);
    setEntryContentViewMode("auto");
  };

  /** Handle viewing case document as hex */
  const handleCaseDocViewHex = (doc: CaseDocument) => {
    centerPaneTabs.openCaseDocument(doc);
    setEntryContentViewMode("hex");
  };

  /** Handle viewing case document as text */
  const handleCaseDocViewText = (doc: CaseDocument) => {
    centerPaneTabs.openCaseDocument(doc);
    setEntryContentViewMode("text");
  };

  return {
    handleSelectEntry,
    handleSelectEvidenceFile,
    handleOpenNestedContainer,
    handleSelectProcessedDb,
    handleCaseDocumentSelect,
    handleCaseDocViewHex,
    handleCaseDocViewText,
  };
}
