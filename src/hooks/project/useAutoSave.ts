// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Auto-save functionality for project
 * Uses solid-primitives/scheduled for debounced saves
 */

import { debounce } from "@solid-primitives/scheduled";
import { AUTO_SAVE_INTERVAL_MS } from "../../types/project";
import { getPreference } from "../../components/preferences";
import { logger } from "../../utils/logger";
import type { ProjectStateSignals, ProjectStateSetters, AutoSaveManager } from "./types";

const log = logger.scope("AutoSave");

/**
 * Create auto-save management functions
 * Uses debounced saves to prevent excessive disk writes
 */
export function createAutoSaveManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters
): AutoSaveManager & { cleanup: () => void } {
  let autoSaveTimer: ReturnType<typeof setInterval> | null = null;
  let autoSaveCallback: (() => Promise<void>) | null = null;
  let isSaving = false;

  // Debounced save function to prevent rapid consecutive saves
  const debouncedSave = debounce(async () => {
    log.debug(`debouncedSave triggered, isSaving=${isSaving}, hasCallback=${!!autoSaveCallback}`);
    if (isSaving || !autoSaveCallback) return;
    
    log.debug(`modified=${signals.modified()}, projectPath=${signals.projectPath()}`);
    if (!signals.modified() || !signals.projectPath()) return;
    
    isSaving = true;
    log.info("Saving project...");
    try {
      await autoSaveCallback();
      log.info("Save complete");
    } catch (e) {
      log.warn("Save failed:", e);
    } finally {
      isSaving = false;
    }
  }, 2000); // Debounce by 2 seconds to batch rapid changes

  /** Start auto-save timer */
  const startAutoSave = () => {
    log.debug("startAutoSave called");
    if (autoSaveTimer) {
      log.debug("Clearing existing timer");
      clearInterval(autoSaveTimer);
    }
    
    // Check preference first, then project settings
    const prefEnabled = getPreference("autoSaveProject");
    const prefInterval = getPreference("autoSaveIntervalMs");
    
    const settings = signals.project()?.settings;
    const autoSaveEnabled = settings?.auto_save ?? prefEnabled;
    
    log.debug(`prefEnabled=${prefEnabled}, settings.auto_save=${settings?.auto_save}, autoSaveEnabled=${signals.autoSaveEnabled()}`);
    
    if (!autoSaveEnabled || !signals.autoSaveEnabled()) {
      log.debug("Auto-save disabled, not starting timer");
      return;
    }
    
    const interval = settings?.auto_save_interval || prefInterval || AUTO_SAVE_INTERVAL_MS;
    log.debug(`Starting timer with interval ${interval}ms`);
    
    autoSaveTimer = setInterval(() => {
      // Use debounced save to prevent overlapping saves
      debouncedSave();
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
    debouncedSave.clear(); // Clear any pending debounced saves
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
