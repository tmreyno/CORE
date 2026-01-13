// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, Show, For, onMount, onCleanup, JSX } from "solid-js";
import { useFocusTrap } from "../hooks/useFocusTrap";
import { Toggle, Slider } from "./ui";
import {
  HiOutlinePaintBrush,
  HiOutlineCog6Tooth,
  HiOutlineWrenchScrewdriver,
  HiOutlineCommandLine,
  HiOutlineXMark,
  HiOutlineArrowPath,
} from "./icons";

// ============================================================================
// Types
// ============================================================================

export type Theme = "dark" | "light" | "system";
export type TreeDensity = "compact" | "comfortable" | "spacious";
export type HashAlgorithm = "MD5" | "SHA1" | "SHA256" | "SHA512" | "Blake3" | "XXH3";

export interface AppPreferences {
  // Appearance
  theme: Theme;
  treeDensity: TreeDensity;
  showLineNumbers: boolean;
  fontSize: number; // 12-18
  
  // Defaults
  defaultHashAlgorithm: HashAlgorithm;
  autoExpandTree: boolean;
  showHiddenFiles: boolean;
  
  // Behavior
  confirmBeforeDelete: boolean;
  autoSaveProject: boolean;
  autoSaveIntervalMs: number;
  
  // Performance
  lazyLoadThreshold: number; // Number of items before lazy loading kicks in
  maxConcurrentOperations: number;
  
  // Keyboard Shortcuts (customizable)
  shortcuts: Record<string, string>;
}

const DEFAULT_PREFERENCES: AppPreferences = {
  theme: "dark",
  treeDensity: "comfortable",
  showLineNumbers: true,
  fontSize: 14,
  
  defaultHashAlgorithm: "SHA256",
  autoExpandTree: false,
  showHiddenFiles: false,
  
  confirmBeforeDelete: true,
  autoSaveProject: true,
  autoSaveIntervalMs: 60000,
  
  lazyLoadThreshold: 100,
  maxConcurrentOperations: 4,
  
  shortcuts: {
    "openCommandPalette": "Meta+k",
    "showShortcuts": "?",
    "openFile": "Meta+o",
    "closeModal": "Escape",
    "save": "Meta+s",
    "undo": "Meta+z",
    "redo": "Meta+Shift+z",
    "search": "Meta+f",
    "settings": "Meta+,",
  },
};

const STORAGE_KEY = "ffx-preferences";

// ============================================================================
// Preferences Hook
// ============================================================================

export function createPreferences() {
  const [preferences, setPreferences] = createSignal<AppPreferences>(DEFAULT_PREFERENCES);
  const [isDirty, setIsDirty] = createSignal(false);

  // Load from localStorage on mount
  onMount(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored);
        setPreferences({ ...DEFAULT_PREFERENCES, ...parsed });
      }
    } catch (e) {
      console.warn("Failed to load preferences:", e);
    }
  });

  // Save to localStorage when dirty
  createEffect(() => {
    if (isDirty()) {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences()));
        setIsDirty(false);
      } catch (e) {
        console.warn("Failed to save preferences:", e);
      }
    }
  });

  const updatePreference = <K extends keyof AppPreferences>(
    key: K,
    value: AppPreferences[K]
  ) => {
    setPreferences((prev) => ({ ...prev, [key]: value }));
    setIsDirty(true);
  };

  const updateShortcut = (action: string, shortcut: string) => {
    setPreferences((prev) => ({
      ...prev,
      shortcuts: { ...prev.shortcuts, [action]: shortcut },
    }));
    setIsDirty(true);
  };

  const resetToDefaults = () => {
    setPreferences(DEFAULT_PREFERENCES);
    setIsDirty(true);
  };

  return {
    preferences,
    updatePreference,
    updateShortcut,
    resetToDefaults,
    isDirty,
  };
}

// ============================================================================
// Settings Panel Component
// ============================================================================

type SettingsTab = "appearance" | "defaults" | "behavior" | "shortcuts";

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
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });

  // Tab icon component for consistent rendering
  const TabIcon = (props: { id: SettingsTab }) => {
    const iconClass = "w-4 h-4";
    switch (props.id) {
      case "appearance":
        return <HiOutlinePaintBrush class={iconClass} />;
      case "defaults":
        return <HiOutlineCog6Tooth class={iconClass} />;
      case "behavior":
        return <HiOutlineWrenchScrewdriver class={iconClass} />;
      case "shortcuts":
        return <HiOutlineCommandLine class={iconClass} />;
      default:
        return <HiOutlineCog6Tooth class={iconClass} />;
    }
  };

  const tabs: { id: SettingsTab; label: string }[] = [
    { id: "appearance", label: "Appearance" },
    { id: "defaults", label: "Defaults" },
    { id: "behavior", label: "Behavior" },
    { id: "shortcuts", label: "Shortcuts" },
  ];

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
        <div
          ref={modalRef}
          class="bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl w-[700px] max-h-[80vh] flex flex-col"
          role="dialog"
          aria-modal="true"
          aria-labelledby="settings-title"
        >
          {/* Header */}
          <div class="flex items-center justify-between px-5 py-4 border-b border-zinc-700">
            <h2 id="settings-title" class="text-lg font-semibold text-zinc-100 flex items-center gap-2">
              <HiOutlineCog6Tooth class="w-5 h-5" />
              <span>Settings</span>
            </h2>
            <button
              class="icon-btn-sm"
              onClick={props.onClose}
              aria-label="Close settings"
            >
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </div>

          {/* Tabs + Content */}
          <div class="flex flex-1 overflow-hidden">
            {/* Tab sidebar */}
            <div class="w-44 bg-zinc-800/50 border-r border-zinc-700 p-2 flex flex-col gap-1">
              <For each={tabs}>
                {(tab) => (
                  <button
                    class={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-left transition-colors ${
                      activeTab() === tab.id
                        ? "bg-cyan-600 text-white"
                        : "text-zinc-300 hover:bg-zinc-700"
                    }`}
                    onClick={() => setActiveTab(tab.id)}
                  >
                    <TabIcon id={tab.id} />
                    <span>{tab.label}</span>
                  </button>
                )}
              </For>

              {/* Reset button at bottom */}
              <div class="mt-auto pt-2 border-t border-zinc-700">
                <button
                  class="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-zinc-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
                  onClick={props.onResetToDefaults}
                >
                  <HiOutlineArrowPath class="w-4 h-4" />
                  <span>Reset to Defaults</span>
                </button>
              </div>
            </div>

            {/* Tab content */}
            <div class="flex-1 p-5 overflow-y-auto">
              <Show when={activeTab() === "appearance"}>
                <AppearanceSettings
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

function SettingGroup(props: { title: string; description?: string; children: JSX.Element }) {
  return (
    <div class="mb-6">
      <h3 class="text-sm font-semibold text-zinc-200 mb-1">{props.title}</h3>
      <Show when={props.description}>
        <p class="text-xs text-zinc-400 mb-3">{props.description}</p>
      </Show>
      <div class="space-y-3">{props.children}</div>
    </div>
  );
}

function SettingRow(props: { label: string; description?: string; children: JSX.Element }) {
  return (
    <div class="flex items-center justify-between gap-4 py-2">
      <div class="flex-1">
        <label class="text-sm text-zinc-200">{props.label}</label>
        <Show when={props.description}>
          <p class="text-xs text-zinc-500 mt-0.5">{props.description}</p>
        </Show>
      </div>
      <div class="shrink-0">{props.children}</div>
    </div>
  );
}

function SettingsSelect<T extends string>(props: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <select
      class="bg-zinc-800 border border-zinc-600 rounded px-3 py-1.5 text-sm text-zinc-200 focus:outline-none focus:border-cyan-500"
      value={props.value}
      onChange={(e) => props.onChange(e.currentTarget.value as T)}
    >
      <For each={props.options}>
        {(opt) => <option value={opt.value}>{opt.label}</option>}
      </For>
    </select>
  );
}

// Appearance Tab
function AppearanceSettings(props: {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}) {
  return (
    <>
      <SettingGroup title="Theme" description="Choose your preferred color scheme">
        <SettingRow label="Theme">
          <SettingsSelect
            value={props.preferences.theme}
            options={[
              { value: "dark", label: "Dark" },
              { value: "light", label: "Light" },
              { value: "system", label: "System" },
            ]}
            onChange={(v) => props.onUpdate("theme", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Tree View" description="Customize the evidence tree appearance">
        <SettingRow label="Density">
          <SettingsSelect
            value={props.preferences.treeDensity}
            options={[
              { value: "compact", label: "Compact" },
              { value: "comfortable", label: "Comfortable" },
              { value: "spacious", label: "Spacious" },
            ]}
            onChange={(v) => props.onUpdate("treeDensity", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Editor" description="Text and display settings">
        <SettingRow label="Font Size">
          <Slider
            value={props.preferences.fontSize}
            min={12}
            max={18}
            onChange={(v) => props.onUpdate("fontSize", v)}
          />
        </SettingRow>

        <SettingRow label="Show Line Numbers">
          <Toggle
            checked={props.preferences.showLineNumbers}
            onChange={(v) => props.onUpdate("showLineNumbers", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}

// Defaults Tab
function DefaultsSettings(props: {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}) {
  return (
    <>
      <SettingGroup title="Hash Algorithm" description="Default algorithm for integrity verification">
        <SettingRow label="Default Hash">
          <SettingsSelect
            value={props.preferences.defaultHashAlgorithm}
            options={[
              { value: "MD5", label: "MD5" },
              { value: "SHA1", label: "SHA-1" },
              { value: "SHA256", label: "SHA-256" },
              { value: "SHA512", label: "SHA-512" },
              { value: "Blake3", label: "BLAKE3" },
              { value: "XXH3", label: "XXH3" },
            ]}
            onChange={(v) => props.onUpdate("defaultHashAlgorithm", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="File Display" description="How files are shown in the tree">
        <SettingRow label="Auto-expand Tree" description="Automatically expand directories on load">
          <Toggle
            checked={props.preferences.autoExpandTree}
            onChange={(v) => props.onUpdate("autoExpandTree", v)}
          />
        </SettingRow>

        <SettingRow label="Show Hidden Files" description="Display files starting with a dot">
          <Toggle
            checked={props.preferences.showHiddenFiles}
            onChange={(v) => props.onUpdate("showHiddenFiles", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}

// Behavior Tab
function BehaviorSettings(props: {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}) {
  return (
    <>
      <SettingGroup title="Confirmations" description="Prompts before destructive actions">
        <SettingRow label="Confirm Before Delete">
          <Toggle
            checked={props.preferences.confirmBeforeDelete}
            onChange={(v) => props.onUpdate("confirmBeforeDelete", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Auto-save" description="Automatically save project changes">
        <SettingRow label="Enable Auto-save">
          <Toggle
            checked={props.preferences.autoSaveProject}
            onChange={(v) => props.onUpdate("autoSaveProject", v)}
          />
        </SettingRow>

        <Show when={props.preferences.autoSaveProject}>
          <SettingRow label="Save Interval" description="Time between auto-saves">
            <SettingsSelect
              value={String(props.preferences.autoSaveIntervalMs)}
              options={[
                { value: "30000", label: "30 seconds" },
                { value: "60000", label: "1 minute" },
                { value: "120000", label: "2 minutes" },
                { value: "300000", label: "5 minutes" },
              ]}
              onChange={(v) => props.onUpdate("autoSaveIntervalMs", Number(v))}
            />
          </SettingRow>
        </Show>
      </SettingGroup>

      <SettingGroup title="Performance" description="Adjust for your system">
        <SettingRow label="Lazy Load Threshold" description="Items before lazy loading activates">
          <Slider
            value={props.preferences.lazyLoadThreshold}
            min={50}
            max={500}
            step={50}
            onChange={(v) => props.onUpdate("lazyLoadThreshold", v)}
          />
        </SettingRow>

        <SettingRow label="Concurrent Operations" description="Maximum parallel operations">
          <Slider
            value={props.preferences.maxConcurrentOperations}
            min={1}
            max={8}
            onChange={(v) => props.onUpdate("maxConcurrentOperations", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}

// Shortcuts Tab
function ShortcutsSettings(props: {
  preferences: AppPreferences;
  onUpdateShortcut: (action: string, shortcut: string) => void;
  editingShortcut: () => string | null;
  setEditingShortcut: (action: string | null) => void;
}) {
  const shortcutLabels: Record<string, string> = {
    openCommandPalette: "Open Command Palette",
    showShortcuts: "Show Keyboard Shortcuts",
    openFile: "Open File",
    closeModal: "Close Modal/Dialog",
    save: "Save Project",
    undo: "Undo",
    redo: "Redo",
    search: "Search",
    settings: "Open Settings",
  };

  const formatShortcut = (shortcut: string) => {
    return shortcut
      .replace("Meta", "⌘")
      .replace("Shift", "⇧")
      .replace("Alt", "⌥")
      .replace("Control", "⌃")
      .replace(/\+/g, " ");
  };

  const handleKeyCapture = (action: string, e: KeyboardEvent) => {
    e.preventDefault();
    
    const parts: string[] = [];
    if (e.metaKey) parts.push("Meta");
    if (e.ctrlKey) parts.push("Control");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    
    // Add the actual key (if not a modifier)
    if (!["Meta", "Control", "Alt", "Shift"].includes(e.key)) {
      parts.push(e.key.length === 1 ? e.key.toLowerCase() : e.key);
    }
    
    if (parts.length > 0 && !["Meta", "Control", "Alt", "Shift"].includes(parts[parts.length - 1])) {
      props.onUpdateShortcut(action, parts.join("+"));
      props.setEditingShortcut(null);
    }
  };

  return (
    <>
      <SettingGroup title="Keyboard Shortcuts" description="Click on a shortcut to change it">
        <div class="space-y-2">
          <For each={Object.entries(props.preferences.shortcuts)}>
            {([action, shortcut]) => (
              <div class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-zinc-800 transition-colors">
                <span class="text-sm text-zinc-200">{shortcutLabels[action] ?? action}</span>
                <button
                  class={`px-3 py-1.5 rounded font-mono text-sm transition-colors ${
                    props.editingShortcut() === action
                      ? "bg-cyan-600 text-white animate-pulse"
                      : "bg-zinc-700 text-zinc-300 hover:bg-zinc-600"
                  }`}
                  onClick={() => props.setEditingShortcut(action)}
                  onKeyDown={(e) => {
                    if (props.editingShortcut() === action) {
                      handleKeyCapture(action, e);
                    }
                  }}
                >
                  {props.editingShortcut() === action ? "Press keys..." : formatShortcut(shortcut)}
                </button>
              </div>
            )}
          </For>
        </div>
      </SettingGroup>

      <p class="text-xs text-zinc-500 mt-4">
        Tip: Click a shortcut button then press your desired key combination.
        Press Escape to cancel.
      </p>
    </>
  );
}

export { DEFAULT_PREFERENCES };
export default SettingsPanel;
