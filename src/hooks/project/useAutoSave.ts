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
import type { ProjectStateSignals, ProjectStateSetters, AutoSaveManager } from "./types";

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
    console.log("[DEBUG] AutoSave: debouncedSave triggered, isSaving=", isSaving, "hasCallback=", !!autoSaveCallback);
    if (isSaving || !autoSaveCallback) return;
    
    console.log("[DEBUG] AutoSave: modified=", signals.modified(), "projectPath=", signals.projectPath());
    if (!signals.modified() || !signals.projectPath()) return;
    
    isSaving = true;
    console.log("[AutoSave] Saving project...");
    try {
      await autoSaveCallback();
      console.log("[AutoSave] Save complete");
    } catch (e) {
      console.warn("[AutoSave] Save failed:", e);
    } finally {
      isSaving = false;
    }
  }, 2000); // Debounce by 2 seconds to batch rapid changes

  /** Start auto-save timer */
  const startAutoSave = () => {
    console.log("[DEBUG] AutoSave: startAutoSave called");
    if (autoSaveTimer) {
      console.log("[DEBUG] AutoSave: Clearing existing timer");
      clearInterval(autoSaveTimer);
    }
    
    // Check preference first, then project settings
    const prefEnabled = getPreference("autoSaveProject");
    const prefInterval = getPreference("autoSaveIntervalMs");
    
    const settings = signals.project()?.settings;
    const autoSaveEnabled = settings?.auto_save ?? prefEnabled;
    
    console.log("[DEBUG] AutoSave: prefEnabled=", prefEnabled, "settings.auto_save=", settings?.auto_save, "signals.autoSaveEnabled()=", signals.autoSaveEnabled());
    
    if (!autoSaveEnabled || !signals.autoSaveEnabled()) {
      console.log("[DEBUG] AutoSave: Auto-save disabled, not starting timer");
      return;
    }
    
    const interval = settings?.auto_save_interval || prefInterval || AUTO_SAVE_INTERVAL_MS;
    console.log("[DEBUG] AutoSave: Starting timer with interval", interval, "ms");
    
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
