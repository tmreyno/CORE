// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAppLifecycle — Manages the App component's mount / cleanup lifecycle.
 *
 * Extracted from App.tsx to keep the root component focused on composition.
 * Handles:
 *   - System stats listener setup
 *   - Window resize tracking
 *   - Workspace profile loading
 *   - Auto-save callback registration
 *   - Welcome modal first-run detection
 *   - Last-session restoration
 *   - Cleanup (preview cache, clipboard, auto-save)
 *   - Window title & close-confirmation wiring
 */

import { onMount, onCleanup, createSignal, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { makeEventListener } from "@solid-primitives/event-listener";
import { useWindowTitle, useCloseConfirmation } from "./index";
import { logger } from "../utils/logger";
import type { FileManager } from "./useFileManager";
import type { useProject } from "./project";
import type { buildSaveOptions } from "./project/projectHelpers";

const log = logger.scope("AppLifecycle");

// ─── Types ──────────────────────────────────────────────────────────────────

/** The subset of useProject() we need */
type ProjectManager = ReturnType<typeof useProject>;

/** Save-options blob produced by buildSaveOptions */
type SaveOptions = ReturnType<typeof buildSaveOptions>;

export interface UseAppLifecycleDeps {
  fileManager: FileManager;
  projectManager: ProjectManager;
  workspaceProfiles: {
    listProfiles: () => Promise<unknown>;
    getActiveProfile: () => Promise<unknown>;
  };
  db: {
    restoreLastSession: () => Promise<{ name: string; root_path: string } | null>;
  };
  tour: {
    hasCompleted: () => boolean;
  };
  preferences: {
    preferences: () => { clearClipboardOnClose?: boolean };
  };
  /** Builds save-options from current state. Returns null if nothing to save. */
  getSaveOptions: () => SaveOptions;
  /** Setter for the welcome modal signal */
  setShowWelcomeModal: (show: boolean) => void;
}

// ─── Hook ───────────────────────────────────────────────────────────────────

export function useAppLifecycle(deps: UseAppLifecycleDeps) {
  const {
    fileManager,
    projectManager,
    workspaceProfiles,
    db,
    tour,
    preferences,
    getSaveOptions,
    setShowWelcomeModal,
  } = deps;

  // Window width signal for compact-mode detection
  const [windowWidth, setWindowWidth] = createSignal(window.innerWidth);
  const isCompact = () => windowWidth() < 900;

  // ── Debug Effect ────────────────────────────────────────────────────────

  createEffect(() => {
    const isModified = projectManager.modified();
    const projectName = projectManager.projectName();
    log.debug(`Modified state changed: ${isModified}, Project: ${projectName || "none"}`);
  });

  // ── Window Title ────────────────────────────────────────────────────────

  useWindowTitle({
    projectName: projectManager.projectName,
    modified: projectManager.modified,
    projectPath: projectManager.projectPath,
  });

  // ── Close Confirmation ──────────────────────────────────────────────────

  useCloseConfirmation({
    hasUnsavedChanges: projectManager.modified,
    onSave: async () => {
      log.debug("Close confirmation: onSave triggered");
      const options = getSaveOptions();
      if (options) {
        const result = await projectManager.saveProject(options);
        return result.success;
      }
      return false;
    },
    dialogTitle: "Save Project?",
    dialogMessage: "You have unsaved changes. Would you like to save before closing?",
  });

  // ── Mount ───────────────────────────────────────────────────────────────

  let cleanupSystemStats: (() => void) | undefined;
  const handleResize = () => setWindowWidth(window.innerWidth);

  onMount(async () => {
    const startupStart = performance.now();
    log.info("App onMount triggered");

    // System stats listener
    const t1 = performance.now();
    const unlisten = await fileManager.setupSystemStatsListener();
    log.debug(`setupSystemStatsListener: ${(performance.now() - t1).toFixed(0)}ms`);
    cleanupSystemStats = unlisten;

    // Window resize handling - makeEventListener auto-cleans up
    makeEventListener(window, "resize", handleResize);

    // Load workspace profiles (run in parallel)
    const t2 = performance.now();
    await Promise.all([workspaceProfiles.listProfiles(), workspaceProfiles.getActiveProfile()]);
    log.debug(`workspaceProfiles: ${(performance.now() - t2).toFixed(0)}ms`);

    // Auto-save callback
    projectManager.setAutoSaveCallback(async () => {
      log.debug("AutoSave callback triggered");
      const options = getSaveOptions();
      if (options) await projectManager.saveProject(options);
    });

    // Welcome modal for first-time users
    const hasSeenWelcome = localStorage.getItem("ffx-welcome-seen");
    const tourCompleted = tour.hasCompleted();
    log.debug(`Welcome check: hasSeenWelcome=${hasSeenWelcome}, tourCompleted=${tourCompleted}`);
    if (!hasSeenWelcome && !tourCompleted) {
      log.debug("Showing welcome modal in 500ms...");
      setTimeout(() => setShowWelcomeModal(true), 500);
    }

    // Restore last session (non-blocking)
    db.restoreLastSession()
      .then((lastSession) => {
        if (lastSession && !projectManager.hasProject()) {
          fileManager.setScanDir(lastSession.root_path);
          log.info(`Restored session: ${lastSession.name} (${lastSession.root_path})`);
        }
      })
      .catch((e) => log.warn("Failed to restore last session:", e));

    log.info(`Total onMount: ${(performance.now() - startupStart).toFixed(0)}ms`);
  });

  // ── Cleanup ─────────────────────────────────────────────────────────────

  onCleanup(() => {
    cleanupSystemStats?.();
    projectManager.stopAutoSave();

    // Clean up temporary preview/thumbnail files
    invoke("cleanup_preview_cache").catch((e: unknown) => log.warn("Failed to cleanup preview cache:", e));

    // Clear clipboard on close if preference is set (security feature)
    if (preferences.preferences().clearClipboardOnClose) {
      navigator.clipboard.writeText("").catch(() => {});
    }
  });

  return { windowWidth, isCompact };
}
