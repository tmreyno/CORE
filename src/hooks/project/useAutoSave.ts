// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Auto-save functionality for project
 */

import { AUTO_SAVE_INTERVAL_MS } from "../../types/project";
import type { ProjectStateSignals, ProjectStateSetters, AutoSaveManager } from "./types";

/**
 * Create auto-save management functions
 */
export function createAutoSaveManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters
): AutoSaveManager & { cleanup: () => void } {
  let autoSaveTimer: ReturnType<typeof setInterval> | null = null;
  let autoSaveCallback: (() => Promise<void>) | null = null;

  /** Start auto-save timer */
  const startAutoSave = () => {
    if (autoSaveTimer) {
      clearInterval(autoSaveTimer);
    }
    
    const settings = signals.project()?.settings;
    if (!settings?.auto_save || !signals.autoSaveEnabled()) return;
    
    const interval = settings.auto_save_interval || AUTO_SAVE_INTERVAL_MS;
    
    autoSaveTimer = setInterval(async () => {
      if (signals.modified() && signals.projectPath() && autoSaveCallback) {
        console.log("Auto-saving project...");
        try {
          await autoSaveCallback();
          console.log("Auto-save complete");
        } catch (e) {
          console.warn("Auto-save failed:", e);
        }
      }
    }, interval);
  };

  /** Stop auto-save timer */
  const stopAutoSave = () => {
    if (autoSaveTimer) {
      clearInterval(autoSaveTimer);
      autoSaveTimer = null;
    }
  };

  /** Set the auto-save callback (called from App.tsx with current state) */
  const setAutoSaveCallback = (callback: () => Promise<void>) => {
    autoSaveCallback = callback;
  };

  /** Clean up resources */
  const cleanup = () => {
    stopAutoSave();
    autoSaveCallback = null;
  };

  return {
    setAutoSaveEnabled: setters.setAutoSaveEnabled,
    setAutoSaveCallback,
    startAutoSave,
    stopAutoSave,
    cleanup,
  };
}
