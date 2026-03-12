// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show, For, onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { useFocusTrap } from "../hooks/useFocusTrap";
import {
  HiOutlinePaintBrush,
  HiOutlineCog6Tooth,
  HiOutlineWrenchScrewdriver,
  HiOutlineCommandLine,
  HiOutlineXMark,
  HiOutlineArrowPath,
  HiOutlineBolt,
  HiOutlineShieldCheck,
  HiOutlineFolderOpen,
  HiOutlineDocumentText,
  HiOutlineArrowUpTray,
  HiOutlineUsers,
  HiOutlineSquares2x2,
} from "./icons";

// Import settings components (use these instead of inline definitions)
import { 
  ActivityDisplaySettings,
} from "./settings";

// Import extracted tab components
import { AppearanceSettings } from "./settings/AppearanceTab";
import { DefaultsSettings } from "./settings/DefaultsTab";
import { BehaviorSettings } from "./settings/BehaviorTab";
import { PerformanceSettings } from "./settings/PerformanceTab";
import { SecuritySettings } from "./settings/SecurityTab";
import { PathsSettings } from "./settings/PathsTab";
import { ReportsSettings } from "./settings/ReportsTab";
import { ShortcutsSettings } from "./settings/ShortcutsTab";
import { UserProfilesSettings } from "./settings/UserProfilesTab";
import { WorkspaceModeSettings } from "./settings/WorkspaceModeTab";

// Import types and hook from extracted preferences module
import type { 
  AppPreferences,
} from "./preferences";
export { createPreferences, DEFAULT_PREFERENCES } from "./preferences";
export type { 
  AppPreferences, 
  Theme, 
  TreeDensity, 
  HashAlgorithm,
  AccentColor,
  IconSet,
  SidebarPosition,
  ExportFormat,
  ViewMode,
  SortOrder,
  DateFormat,
  LogLevel,
  HashVerificationMode,
  ReportPreset,
  UserProfile,
  FeatureModule,
  FeatureModuleInfo,
  WorkspaceModePreset,
} from "./preferences";
export { FEATURE_MODULES, WORKSPACE_PRESETS, getWorkspacePreset } from "./preferences";

// ============================================================================
// Settings Panel Component
// ============================================================================

type SettingsTab = 
  | "workspace"
  | "appearance" 
  | "activity"
  | "defaults" 
  | "behavior" 
  | "performance" 
  | "security" 
  | "paths" 
  | "reports" 
  | "users"
  | "shortcuts";

export interface SettingsPanelProps {
  isOpen: boolean;
  onClose: () => void;
  preferences: AppPreferences;
  onUpdatePreference: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
  onUpdateShortcut: (action: string, shortcut: string) => void;
  onResetToDefaults: () => void;
}

export function SettingsPanel(props: SettingsPanelProps) {
  let modalRef: HTMLDivElement | undefined;
  useFocusTrap(() => modalRef, () => props.isOpen);

  const [activeTab, setActiveTab] = createSignal<SettingsTab>("appearance");
  const [editingShortcut, setEditingShortcut] = createSignal<string | null>(null);

  // Close on Escape
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && props.isOpen) {
        if (editingShortcut()) {
          setEditingShortcut(null);
        } else {
          props.onClose();
        }
      }
    };
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(document, "keydown", handleKeyDown);
  });

  // Tab icon component for consistent rendering
  const TabIcon = (props: { id: SettingsTab }) => {
    switch (props.id) {
      case "workspace":
        return <HiOutlineSquares2x2 class="w-3.5 h-3.5" />;
      case "appearance":
        return <HiOutlinePaintBrush class="w-3.5 h-3.5" />;
      case "activity":
        return <HiOutlineArrowUpTray class="w-3.5 h-3.5" />;
      case "defaults":
        return <HiOutlineCog6Tooth class="w-3.5 h-3.5" />;
      case "behavior":
        return <HiOutlineWrenchScrewdriver class="w-3.5 h-3.5" />;
      case "performance":
        return <HiOutlineBolt class="w-3.5 h-3.5" />;
      case "security":
        return <HiOutlineShieldCheck class="w-3.5 h-3.5" />;
      case "paths":
        return <HiOutlineFolderOpen class="w-3.5 h-3.5" />;
      case "reports":
        return <HiOutlineDocumentText class="w-3.5 h-3.5" />;
      case "users":
        return <HiOutlineUsers class="w-3.5 h-3.5" />;
      case "shortcuts":
        return <HiOutlineCommandLine class="w-3.5 h-3.5" />;
      default:
        return <HiOutlineCog6Tooth class="w-3.5 h-3.5" />;
    }
  };

  const tabs: { id: SettingsTab; label: string }[] = [
    { id: "workspace", label: "Workspace Mode" },
    { id: "appearance", label: "Appearance" },
    { id: "activity", label: "Activity Display" },
    { id: "defaults", label: "Defaults" },
    { id: "behavior", label: "Behavior" },
    { id: "performance", label: "Performance" },
    { id: "security", label: "Security" },
    { id: "paths", label: "Paths" },
    { id: "reports", label: "Reports" },
    { id: "users", label: "Users & Profiles" },
    { id: "shortcuts", label: "Shortcuts" },
  ];

  return (
    <Show when={props.isOpen}>
      <div class="modal-overlay">
        <div
          ref={modalRef}
          class="modal-content w-[800px] max-w-[90vw] h-[600px] max-h-[80vh] flex flex-col"
          role="dialog"
          aria-modal="true"
          aria-labelledby="settings-title"
        >
          {/* Header */}
          <div class="modal-header shrink-0">
            <h2 id="settings-title" class="text-sm font-semibold text-txt flex items-center gap-2">
              <HiOutlineCog6Tooth class="w-3.5 h-3.5" />
              <span>Settings</span>
            </h2>
            <button
              class="icon-btn-sm"
              onClick={props.onClose}
              aria-label="Close settings"
            >
              <HiOutlineXMark class="w-3.5 h-3.5" />
            </button>
          </div>

          {/* Tabs + Content */}
          <div class="flex flex-1 overflow-hidden">
            {/* Tab sidebar */}
            <div class="w-44 shrink-0 border-r border-border bg-bg-panel p-2 flex flex-col gap-0.5">
              <For each={tabs}>
                {(tab) => (
                  <button
                    class={`flex items-center gap-2 px-2.5 py-1.5 rounded text-xs transition-colors cursor-pointer ${
                      activeTab() === tab.id
                        ? "bg-accent text-white"
                        : "hover:bg-bg-hover text-txt-secondary"
                    }`}
                    onClick={() => setActiveTab(tab.id)}
                  >
                    <TabIcon id={tab.id} />
                    <span>{tab.label}</span>
                  </button>
                )}
              </For>

              {/* Reset button at bottom */}
              <div class={`mt-auto pt-1.5 border-t border-border/50`}>
                <button
                  class="px-2 py-1 text-xs rounded text-txt-muted hover:text-txt hover:bg-bg-hover transition-colors"
                  onClick={props.onResetToDefaults}
                >
                  <HiOutlineArrowPath class="w-3 h-3" />
                  <span>Reset</span>
                </button>
              </div>
            </div>

            {/* Tab content */}
            <div class="flex-1 overflow-y-auto p-3">
              <Show when={activeTab() === "workspace"}>
                <WorkspaceModeSettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "appearance"}>
                <AppearanceSettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "activity"}>
                <ActivityDisplaySettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "defaults"}>
                <DefaultsSettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "behavior"}>
                <BehaviorSettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "performance"}>
                <PerformanceSettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "security"}>
                <SecuritySettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "paths"}>
                <PathsSettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "reports"}>
                <ReportsSettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "users"}>
                <UserProfilesSettings
                  preferences={props.preferences}
                  onUpdate={props.onUpdatePreference}
                />
              </Show>

              <Show when={activeTab() === "shortcuts"}>
                <ShortcutsSettings
                  preferences={props.preferences}
                  onUpdateShortcut={props.onUpdateShortcut}
                  editingShortcut={editingShortcut}
                  setEditingShortcut={setEditingShortcut}
                />
              </Show>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
}

// ============================================================================
// Settings Sections
// ============================================================================
// All settings tabs have been extracted to separate files in ./settings/
// - AppearanceTab.tsx
// - DefaultsTab.tsx
// - BehaviorTab.tsx
// - PerformanceTab.tsx
// - SecurityTab.tsx
// - PathsTab.tsx
