// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAppHandlers — App-level action handlers extracted from App.tsx
 *
 * Contains:
 *  - handleLocationSelect()  — routes toolbar location selection to the
 *    appropriate scan (evidence vs processed databases)
 *  - handleQuickAction()     — dispatches QuickAction.command strings to
 *    the correct handler calls
 */

import type { Setter } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { ProcessedDatabase } from "../types/processed";
import type { useFileManager } from "./useFileManager";
import type { useHashManager } from "./useHashManager";
import type { useProject } from "./useProject";
import type { useProcessedDatabases } from "./useProcessedDatabases";
import type { CenterPaneTabsState } from "./useCenterPaneTabs";
import type { LeftPanelTab } from "../components";
import type { QuickAction } from "./useWorkspaceProfiles";
import { logger } from "../utils/logger";

const log = logger.scope("AppHandlers");

// ---------------------------------------------------------------------------
// Dependency interface
// ---------------------------------------------------------------------------

export interface UseAppHandlersDeps {
  /** Core managers */
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  projectManager: ReturnType<typeof useProject>;
  centerPaneTabs: CenterPaneTabsState;
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };

  /** UI state setters */
  setLeftPanelTab: Setter<LeftPanelTab>;
  setLeftCollapsed: (value: boolean | ((prev: boolean) => boolean)) => void;
  handleScanEvidence: () => void;
  setShowSearchPanel: Setter<boolean>;
  setShowReportWizard: Setter<boolean>;
  setShowSettingsPanel: Setter<boolean>;
  setShowCommandPalette: Setter<boolean>;
}

// ---------------------------------------------------------------------------
// Return type
// ---------------------------------------------------------------------------

export interface AppHandlers {
  /** Route toolbar location dropdown selection to the correct scan. */
  handleLocationSelect: (path: string, locationId: string) => Promise<void>;
  /** Dispatch a QuickAction command to the matching handler. */
  handleQuickAction: (action: QuickAction) => void;
}

// ---------------------------------------------------------------------------
// Hook implementation
// ---------------------------------------------------------------------------

export function useAppHandlers(deps: UseAppHandlersDeps): AppHandlers {
  const {
    processedDbManager,
    fileManager,
    hashManager,
    projectManager,
    centerPaneTabs,
    toast,
    setLeftPanelTab,
    setLeftCollapsed,
    handleScanEvidence,
    setShowSearchPanel,
    setShowReportWizard,
    setShowSettingsPanel,
    setShowCommandPalette,
  } = deps;

  /**
   * Handle location selection from the toolbar dropdown.
   * Routes to the appropriate scan based on location type:
   * - "evidence" → scan for evidence files (default behavior)
   * - "processed" → scan for processed databases and sync to ffxdb
   * - "documents" → just set the scan directory
   */
  const handleLocationSelect = async (path: string, locationId: string) => {
    if (locationId === "processed") {
      // Scan for processed databases and sync to ffxdb
      log.info(`Scanning for processed databases at: ${path}`);
      try {
        const results = await invoke<ProcessedDatabase[]>(
          "scan_processed_databases",
          { path, recursive: true }
        );

        if (results.length > 0) {
          // Add to in-memory manager
          processedDbManager.addDatabases(results);

          // Sync each to ffxdb
          const { dbSync } = await import("./project/useProjectDbSync");
          for (const db of results) {
            dbSync.upsertProcessedDatabase(db);
          }

          // Select the first one if none selected
          if (!processedDbManager.selectedDatabase()) {
            await processedDbManager.selectDatabase(results[0]);
          }

          // Switch to processed tab in left panel
          setLeftPanelTab("processed");

          toast.success(
            "Databases Found",
            `Discovered ${results.length} processed database${results.length !== 1 ? "s" : ""}`
          );
          log.info(`Synced ${results.length} processed databases to ffxdb`);
        } else {
          toast.info("No Databases", "No processed databases found in this directory");
        }
      } catch (err) {
        log.error("Failed to scan for processed databases:", err);
        toast.error("Scan Failed", err instanceof Error ? err.message : String(err));
      }
    } else {
      // Default: set scan dir and scan for evidence files
      fileManager.setScanDir(path);
      handleScanEvidence();
    }
  };

  /** Quick action dispatch — maps QuickAction.command to handler calls */
  const handleQuickAction = (action: QuickAction) => {
    switch (action.command) {
      case "hash_selected": hashManager.hashSelectedFiles(); break;
      case "hash_all": hashManager.hashAllFiles(); break;
      case "open_search": setShowSearchPanel(true); break;
      case "export_selected": centerPaneTabs.openExportTab(); break;
      case "verify_hashes": hashManager.hashAllFiles(); break;
      case "generate_report":
        if (projectManager.hasProject()) setShowReportWizard(true);
        break;
      case "evidence_collection":
        if (projectManager.hasProject()) centerPaneTabs.openEvidenceCollection();
        break;
      case "deduplication": toast.info("Deduplication", "Feature coming soon"); break;
      case "show_bookmarks": setLeftCollapsed(false); setLeftPanelTab("bookmarks"); break;
      case "open_settings": setShowSettingsPanel(true); break;
      case "command_palette": setShowCommandPalette(true); break;
      default: toast.info("Action", action.name);
    }
  };

  return { handleLocationSelect, handleQuickAction };
}
