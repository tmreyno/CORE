// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useMenuActions — Handles events from the native menu bar.
 *
 * Listens for "menu-action" events emitted by the Rust backend (menu.rs)
 * and dispatches them to the appropriate frontend handlers.
 */

import { onMount, onCleanup } from "solid-js";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { logger } from "../utils/logger";
import { isAcquireEdition } from "../utils/edition";

const log = logger.scope("MenuActions");

export interface UseMenuActionsDeps {
  /** Open a project file via dialog */
  onOpenProject: () => void;
  /** Open an evidence directory via dialog */
  onOpenDirectory: () => void;
  /** Save the current project */
  onSaveProject: () => void;
  /** Save the current project to a new location */
  onSaveProjectAs: () => void;
  /** Toggle left sidebar collapsed state */
  onToggleSidebar: () => void;
  /** Toggle right panel collapsed state */
  onToggleRightPanel: () => void;
  /** Show the keyboard shortcuts modal */
  onKeyboardShortcuts: () => void;
  /** Show the command palette */
  onCommandPalette: () => void;
  /** Open the New Project wizard */
  onNewProject: () => void;
  /** Open the Export panel */
  onExport: () => void;
  /** Open the Report Wizard */
  onGenerateReport: () => void;
  /** Trigger evidence scan */
  onScanEvidence: () => void;
  /** Toggle quick actions bar */
  onToggleQuickActions: () => void;
  /** Switch left panel to evidence tab */
  onShowEvidence: () => void;
  /** Switch left panel to case documents tab */
  onShowCaseDocs: () => void;
  /** Switch left panel to processed databases tab */
  onShowProcessed: () => void;
  /** Open evidence collection form */
  onEvidenceCollection: () => void;
  /** Open search panel */
  onSearchEvidence: () => void;
  /** Open settings panel */
  onSettings: () => void;
  /** Close all open tabs */
  onCloseAllTabs: () => void;
  /** Hash all discovered evidence files */
  onHashAll: () => void;
  /** Open evidence collection list */
  onEvidenceCollectionList: () => void;
  /** Open the user guide / help panel */
  onUserGuide: () => void;
  /** Show the welcome screen */
  onWelcomeScreen: () => void;
  /** Close the active tab */
  onCloseActiveTab: () => void;
  /** Toggle auto-save */
  onToggleAutoSave: () => void;
  /** Hash selected files only */
  onHashSelected: () => void;
  /** Hash the active file */
  onHashActive: () => void;
  /** Start guided tour */
  onStartTour: () => void;
  /** Switch left panel to dashboard */
  onShowDashboard: () => void;
  /** Switch left panel to activity timeline */
  onShowActivity: () => void;
  /** Switch left panel to bookmarks */
  onShowBookmarks: () => void;
  /** Switch to info view mode */
  onViewInfo: () => void;
  /** Switch to hex view mode */
  onViewHex: () => void;
  /** Switch to text view mode */
  onViewText: () => void;
  /** Cycle theme (light → dark → system) */
  onCycleTheme: () => void;
  /** Select all evidence files */
  onSelectAllEvidence: () => void;
  /** Open deduplication panel */
  onDeduplication: () => void;
  /** Load all file info */
  onLoadAllInfo: () => void;
  /** Clean the preview cache */
  onCleanCache: () => void;
  /** Check for application updates */
  onCheckForUpdates: () => void;
  /** Open the Merge Projects wizard */
  onMergeProjects: () => void;
  /** Open the Project Recovery modal */
  onProjectRecovery: () => void;
}

/**
 * Sets up a Tauri event listener for native menu actions.
 * Automatically cleans up when the component unmounts.
 */
export function useMenuActions(deps: UseMenuActionsDeps): void {
  let unlisten: UnlistenFn | undefined;

  onMount(async () => {
    unlisten = await listen<string>("menu-action", (event) => {
      const action = event.payload;
      log.debug(`Menu action received: ${action}`);

      // Safety net: block full-edition-only actions in acquire edition
      const FULL_ONLY_ACTIONS = new Set([
        "generate-report",
        "merge-projects",
        "deduplication",
        "show-dashboard",
        "show-processed",
        "show-casedocs",
        "show-activity",
      ]);
      if (isAcquireEdition() && FULL_ONLY_ACTIONS.has(action)) {
        log.warn(`Blocked full-edition action in acquire: ${action}`);
        return;
      }

      switch (action) {
        case "open-project":
          deps.onOpenProject();
          break;
        case "open-directory":
          deps.onOpenDirectory();
          break;
        case "save-project":
          deps.onSaveProject();
          break;
        case "save-project-as":
          deps.onSaveProjectAs();
          break;
        case "toggle-sidebar":
          deps.onToggleSidebar();
          break;
        case "toggle-right-panel":
          deps.onToggleRightPanel();
          break;
        case "keyboard-shortcuts":
          deps.onKeyboardShortcuts();
          break;
        case "command-palette":
          deps.onCommandPalette();
          break;
        case "new-project":
          deps.onNewProject();
          break;
        case "export":
          deps.onExport();
          break;
        case "generate-report":
          deps.onGenerateReport();
          break;
        case "scan-evidence":
          deps.onScanEvidence();
          break;
        case "toggle-quick-actions":
          deps.onToggleQuickActions();
          break;
        case "show-evidence":
          deps.onShowEvidence();
          break;
        case "show-casedocs":
          deps.onShowCaseDocs();
          break;
        case "show-processed":
          deps.onShowProcessed();
          break;
        case "evidence-collection":
          deps.onEvidenceCollection();
          break;
        case "search-evidence":
          deps.onSearchEvidence();
          break;
        case "settings":
          deps.onSettings();
          break;
        case "close-all-tabs":
          deps.onCloseAllTabs();
          break;
        case "hash-all":
          deps.onHashAll();
          break;
        case "evidence-collection-list":
          deps.onEvidenceCollectionList();
          break;
        case "user-guide":
          deps.onUserGuide();
          break;
        case "welcome-screen":
          deps.onWelcomeScreen();
          break;
        case "close-active-tab":
          deps.onCloseActiveTab();
          break;
        case "toggle-autosave":
          deps.onToggleAutoSave();
          break;
        case "hash-selected":
          deps.onHashSelected();
          break;
        case "hash-active":
          deps.onHashActive();
          break;
        case "start-tour":
          deps.onStartTour();
          break;
        case "show-dashboard":
          deps.onShowDashboard();
          break;
        case "show-activity":
          deps.onShowActivity();
          break;
        case "show-bookmarks":
          deps.onShowBookmarks();
          break;
        case "view-info":
          deps.onViewInfo();
          break;
        case "view-hex":
          deps.onViewHex();
          break;
        case "view-text":
          deps.onViewText();
          break;
        case "cycle-theme":
          deps.onCycleTheme();
          break;
        case "select-all-evidence":
          deps.onSelectAllEvidence();
          break;
        case "deduplication":
          deps.onDeduplication();
          break;
        case "load-all-info":
          deps.onLoadAllInfo();
          break;
        case "clean-cache":
          deps.onCleanCache();
          break;
        case "check-updates":
          deps.onCheckForUpdates();
          break;
        case "merge-projects":
          deps.onMergeProjects();
          break;
        case "project-recovery":
          deps.onProjectRecovery();
          break;
        default:
          log.warn(`Unknown menu action: ${action}`);
      }
    });

    log.info("Menu action listener registered");
  });

  onCleanup(() => {
    unlisten?.();
  });
}
