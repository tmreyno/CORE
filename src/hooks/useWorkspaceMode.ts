// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useWorkspaceMode — Reactive hook for workspace mode management.
 *
 * Provides:
 *  - activeMode()        — current WorkspaceModePreset
 *  - enabledModules()    — resolved set of enabled FeatureModule IDs
 *  - isModuleEnabled(m)  — boolean check for a specific module
 *  - setMode(id)         — switch to a preset mode
 *  - toggleModule(m)     — toggle a single module (auto-switches to "custom")
 *  - setCustomModules(ms)— bulk-set custom modules
 *
 * All state is persisted via AppPreferences (localStorage).
 */

import { createMemo } from "solid-js";
import {
  usePreferences,
  getWorkspacePreset,
  WORKSPACE_PRESETS,
  type FeatureModule,
  type WorkspaceModePreset,
} from "../components/preferences";
import type { LeftPanelTab } from "../components/layout/sidebar/types";

/**
 * Maps each sidebar navigation tab to the module that gates it.
 * Used by the auto-tab-switch effect and the sidebar gating logic.
 */
export const TAB_MODULE_MAP: Partial<Record<LeftPanelTab, FeatureModule>> = {
  dashboard: "caseManagement",
  evidence: "forensicExplorer",
  processed: "searchAnalysis",
  casedocs: "documentReview",
  activity: "caseManagement",
  drives: "reportExport",
  // bookmarks — always available (universal annotation tool)
};

/**
 * Maps quick action IDs to the feature module that controls them.
 * Actions not listed here are always visible (e.g., settings, command palette).
 */
export const ACTION_MODULE_MAP: Record<string, FeatureModule> = {
  hash: "forensicExplorer",
  verify: "forensicExplorer",
  // search — always available (universal tool for all workflows)
  dedup: "searchAnalysis",
  export: "reportExport",
  report: "reportExport",
  evidence: "evidenceCollection",
  // bookmarks — always available (universal annotation tool)
};

export function useWorkspaceMode() {
  const { preferences, updatePreference } = usePreferences();

  // ---------------------------------------------------------------------------
  // Derived state
  // ---------------------------------------------------------------------------

  /** The currently active preset (resolved from preferences) */
  const activeMode = createMemo<WorkspaceModePreset>(() => {
    const modeId = preferences().workspaceMode || "full";
    return getWorkspacePreset(modeId);
  });

  /** Resolved set of enabled modules for the current mode */
  const enabledModules = createMemo<FeatureModule[]>(() => {
    const mode = activeMode();
    if (mode.isCustom) {
      return preferences().customEnabledModules ?? [];
    }
    return mode.modules;
  });

  /** Fast check: is a specific module enabled? */
  const isModuleEnabled = (module: FeatureModule): boolean => {
    return enabledModules().includes(module);
  };

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  /** Switch to a named workspace mode preset */
  const setMode = (modeId: string) => {
    // Validate the mode ID exists
    const preset = WORKSPACE_PRESETS.find(p => p.id === modeId);
    if (!preset) return;
    updatePreference("workspaceMode", modeId);
  };

  /** Toggle a single module on/off. Auto-switches to "custom" mode. */
  const toggleModule = (module: FeatureModule) => {
    const current = [...(preferences().customEnabledModules ?? [])];
    const idx = current.indexOf(module);
    if (idx >= 0) {
      current.splice(idx, 1);
    } else {
      current.push(module);
    }
    updatePreference("customEnabledModules", current);
    // Auto-switch to custom mode if not already
    if (preferences().workspaceMode !== "custom") {
      updatePreference("workspaceMode", "custom");
    }
  };

  /** Bulk-set the custom modules list */
  const setCustomModules = (modules: FeatureModule[]) => {
    updatePreference("customEnabledModules", modules);
    if (preferences().workspaceMode !== "custom") {
      updatePreference("workspaceMode", "custom");
    }
  };

  /** Return the first sidebar tab whose module is enabled, or "evidence" as fallback. */
  const getFirstEnabledTab = (): LeftPanelTab => {
    const ORDERED_TABS: LeftPanelTab[] = ["dashboard", "evidence", "processed", "casedocs", "activity", "drives", "bookmarks"];
    const mods = enabledModules();
    for (const tab of ORDERED_TABS) {
      const requiredMod = TAB_MODULE_MAP[tab];
      if (!requiredMod || mods.includes(requiredMod)) return tab;
    }
    return "bookmarks"; // bookmarks is always available as ultimate fallback
  };

  return {
    activeMode,
    enabledModules,
    isModuleEnabled,
    setMode,
    toggleModule,
    setCustomModules,
    getFirstEnabledTab,
  };
}
