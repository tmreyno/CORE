// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * WorkspaceModeTab — Settings tab for selecting workspace modes
 * and toggling individual feature modules.
 *
 * Presets: Full Suite, Forensic Explorer, Evidence Collection & COC,
 *          Document Review, Search & Analysis, Report & Export, Custom
 *
 * Custom mode exposes per-module checkboxes.
 */

import { Component, For, Show, createMemo } from "solid-js";
import { SettingGroup } from "../settings";
import type { AppPreferences, FeatureModule } from "../preferences";
import {
  WORKSPACE_PRESETS,
  FEATURE_MODULES,
  getWorkspacePreset,
} from "../preferences";
import {
  HiOutlineArchiveBox,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineDocumentText,
  HiOutlineMagnifyingGlass,
  HiOutlineArrowUpTray,
  HiOutlineChartBar,
  HiOutlineCheckCircle,
} from "../icons";

// Map module IDs to icon components for the settings UI
const MODULE_ICONS: Record<string, Component<{ class?: string }>> = {
  forensicExplorer: HiOutlineArchiveBox,
  evidenceCollection: HiOutlineArchiveBoxArrowDown,
  documentReview: HiOutlineDocumentText,
  searchAnalysis: HiOutlineMagnifyingGlass,
  reportExport: HiOutlineArrowUpTray,
  caseManagement: HiOutlineChartBar,
};

interface WorkspaceModeSettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const WorkspaceModeSettings: Component<WorkspaceModeSettingsProps> = (props) => {
  const currentModeId = () => props.preferences.workspaceMode || "full";
  const currentPreset = createMemo(() => getWorkspacePreset(currentModeId()));
  const isCustom = () => currentModeId() === "custom";

  /** Resolved list of enabled modules for the current mode */
  const enabledModules = createMemo<FeatureModule[]>(() => {
    if (isCustom()) {
      return props.preferences.customEnabledModules ?? [];
    }
    return currentPreset().modules;
  });

  const isModuleEnabled = (m: FeatureModule) => enabledModules().includes(m);

  const handleModeSelect = (modeId: string) => {
    props.onUpdate("workspaceMode", modeId);
    // When switching to a named preset from custom, seed customEnabledModules
    // so switching back to custom preserves the last known state
    const preset = getWorkspacePreset(modeId);
    if (!preset.isCustom) {
      props.onUpdate("customEnabledModules", [...preset.modules]);
    }
  };

  const handleModuleToggle = (module: FeatureModule) => {
    const current = [...(props.preferences.customEnabledModules ?? [])];
    const idx = current.indexOf(module);
    if (idx >= 0) {
      current.splice(idx, 1);
    } else {
      current.push(module);
    }
    props.onUpdate("customEnabledModules", current);
    // Auto-switch to custom mode
    if (!isCustom()) {
      props.onUpdate("workspaceMode", "custom");
    }
  };

  return (
    <>
      {/* Preset Selector */}
      <SettingGroup
        title="Workspace Mode"
        description="Choose a preset that shows only the features you need, or build a custom configuration."
      >
        <div class="grid grid-cols-2 gap-2">
          <For each={WORKSPACE_PRESETS}>
            {(preset) => {
              const isActive = () => currentModeId() === preset.id;
              return (
                <button
                  class={`relative flex flex-col items-start gap-1 p-3 rounded-lg border transition-all text-left cursor-pointer ${
                    isActive()
                      ? "border-accent bg-accent/10 ring-1 ring-accent/30"
                      : "border-border bg-bg-secondary hover:border-border-strong hover:bg-bg-hover"
                  }`}
                  onClick={() => handleModeSelect(preset.id)}
                >
                  <div class="flex items-center gap-2 w-full">
                    <span class={`text-xs font-medium ${isActive() ? "text-accent" : "text-txt"}`}>
                      {preset.name}
                    </span>
                    <Show when={isActive()}>
                      <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent ml-auto shrink-0" />
                    </Show>
                  </div>
                  <span class="text-2xs text-txt-muted leading-tight">{preset.description}</span>
                  {/* Module count badge */}
                  <Show when={!preset.isCustom}>
                    <span class="text-2xs text-txt-muted mt-0.5">
                      {preset.modules.length} of {FEATURE_MODULES.length} modules
                    </span>
                  </Show>
                  <Show when={preset.isCustom}>
                    <span class="text-2xs text-txt-muted mt-0.5">
                      {props.preferences.customEnabledModules?.length ?? 0} of {FEATURE_MODULES.length} modules
                    </span>
                  </Show>
                </button>
              );
            }}
          </For>
        </div>
      </SettingGroup>

      {/* Module Toggles (always visible, auto-switch to custom on toggle) */}
      <SettingGroup
        title="Feature Modules"
        description={
          isCustom()
            ? "Toggle individual modules to build your custom workspace."
            : `Showing modules for "${currentPreset().name}". Toggling any module switches to Custom mode.`
        }
      >
        <div class="space-y-1">
          <For each={FEATURE_MODULES}>
            {(mod) => {
              const enabled = () => isModuleEnabled(mod.id);
              const IconComponent = MODULE_ICONS[mod.id];
              return (
                <button
                  class={`flex items-center gap-3 w-full p-2.5 rounded-md border transition-all text-left cursor-pointer ${
                    enabled()
                      ? "border-accent/30 bg-accent/5"
                      : "border-border/50 bg-bg-secondary opacity-60"
                  }`}
                  onClick={() => handleModuleToggle(mod.id)}
                >
                  {/* Checkbox */}
                  <div
                    class={`w-4 h-4 rounded border flex items-center justify-center shrink-0 transition-colors ${
                      enabled()
                        ? "bg-accent border-accent"
                        : "border-border bg-bg"
                    }`}
                  >
                    <Show when={enabled()}>
                      <svg class="w-2.5 h-2.5 text-white" viewBox="0 0 12 12" fill="none">
                        <path d="M2 6l3 3 5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                      </svg>
                    </Show>
                  </div>

                  {/* Icon */}
                  <Show when={IconComponent}>
                    {(_) => {
                      const Ic = IconComponent!;
                      return <Ic class={`w-4 h-4 shrink-0 ${enabled() ? "text-accent" : "text-txt-muted"}`} />;
                    }}
                  </Show>

                  {/* Label + description */}
                  <div class="flex flex-col min-w-0">
                    <span class={`text-xs font-medium ${enabled() ? "text-txt" : "text-txt-muted"}`}>
                      {mod.name}
                    </span>
                    <span class="text-2xs text-txt-muted truncate">{mod.description}</span>
                  </div>
                </button>
              );
            }}
          </For>
        </div>
      </SettingGroup>

      {/* Active mode summary */}
      <div class="mt-2 px-3 py-2 rounded-md bg-bg-secondary border border-border/50">
        <div class="flex items-center gap-2">
          <span class="text-2xs uppercase tracking-wider font-medium text-txt-muted">Active Mode:</span>
          <span class="text-xs font-medium text-accent">{currentPreset().name}</span>
        </div>
        <div class="text-2xs text-txt-muted mt-1">
          {enabledModules().length} of {FEATURE_MODULES.length} feature modules enabled.
          {" "}
          <Show when={enabledModules().length < FEATURE_MODULES.length}>
            Hidden features are accessible via the Settings panel or by changing your workspace mode.
          </Show>
        </div>
      </div>
    </>
  );
};
