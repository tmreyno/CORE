// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show, For, onMount } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { makeEventListener } from "@solid-primitives/event-listener";
import { useFocusTrap } from "../hooks/useFocusTrap";
import { Toggle, Slider } from "./ui";
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
} from "./icons";

// Import settings components (use these instead of inline definitions)
import { 
  SettingGroup, 
  SettingRow, 
  SettingsSelect,
  ActivityDisplaySettings,
} from "./settings";

// Import types and hook from extracted preferences module
import type { 
  AppPreferences,
  Theme,
  AccentColor,
  SidebarPosition,
  TreeDensity,
  IconSet,
  HashAlgorithm,
  ExportFormat,
  ViewMode,
  SortOrder,
  DateFormat,
  LogLevel,
  HashVerificationMode,
  ReportTemplate,
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
  ReportTemplate,
} from "./preferences";

// ============================================================================
// Settings Panel Component
// ============================================================================

type SettingsTab = 
  | "appearance" 
  | "activity"
  | "defaults" 
  | "behavior" 
  | "performance" 
  | "security" 
  | "paths" 
  | "reports" 
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
      case "shortcuts":
        return <HiOutlineCommandLine class="w-3.5 h-3.5" />;
      default:
        return <HiOutlineCog6Tooth class="w-3.5 h-3.5" />;
    }
  };

  const tabs: { id: SettingsTab; label: string }[] = [
    { id: "appearance", label: "Appearance" },
    { id: "activity", label: "Activity Display" },
    { id: "defaults", label: "Defaults" },
    { id: "behavior", label: "Behavior" },
    { id: "performance", label: "Performance" },
    { id: "security", label: "Security" },
    { id: "paths", label: "Paths" },
    { id: "reports", label: "Reports" },
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
            <h2 id="settings-title" class="text-lg font-semibold text-txt flex items-center gap-2">
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
            <div class="w-48 shrink-0 border-r border-border bg-bg-panel p-2 flex flex-col gap-1">
              <For each={tabs}>
                {(tab) => (
                  <button
                    class={`flex items-center gap-2 px-3 py-2 rounded text-sm transition-colors cursor-pointer ${
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
            <div class="flex-1 overflow-y-auto p-4">
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
// Settings Sections (using imported components from ./settings module)
// ============================================================================

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
              { value: "light", label: "Light (Auto)" },
              { value: "light-macos", label: "Light (macOS)" },
              { value: "light-windows", label: "Light (Windows)" },
              { value: "midnight", label: "Midnight" },
              { value: "system", label: "System" },
            ]}
            onChange={(v) => props.onUpdate("theme", v as Theme)}
          />
        </SettingRow>

        <SettingRow label="Accent Color" description="Primary accent color for UI elements">
          <SettingsSelect
            value={props.preferences.accentColor}
            options={[
              { value: "cyan", label: "Cyan" },
              { value: "blue", label: "Blue" },
              { value: "green", label: "Green" },
              { value: "purple", label: "Purple" },
              { value: "orange", label: "Orange" },
              { value: "red", label: "Red" },
            ]}
            onChange={(v) => props.onUpdate("accentColor", v as AccentColor)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Layout" description="Customize the interface layout">
        <SettingRow label="Sidebar Position">
          <SettingsSelect
            value={props.preferences.sidebarPosition}
            options={[
              { value: "left", label: "Left" },
              { value: "right", label: "Right" },
            ]}
            onChange={(v) => props.onUpdate("sidebarPosition", v as SidebarPosition)}
          />
        </SettingRow>

        <SettingRow label="Show Status Bar" description="Display the status bar at the bottom">
          <Toggle
            checked={props.preferences.showStatusBar}
            onChange={(v) => props.onUpdate("showStatusBar", v)}
          />
        </SettingRow>

        <SettingRow label="Enable Animations" description="Enable smooth transitions and animations">
          <Toggle
            checked={props.preferences.animationsEnabled}
            onChange={(v) => props.onUpdate("animationsEnabled", v)}
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
            onChange={(v) => props.onUpdate("treeDensity", v as TreeDensity)}
          />
        </SettingRow>

        <SettingRow label="Icon Style">
          <SettingsSelect
            value={props.preferences.iconSet}
            options={[
              { value: "outlined", label: "Outlined" },
              { value: "solid", label: "Solid" },
              { value: "mini", label: "Mini" },
            ]}
            onChange={(v) => props.onUpdate("iconSet", v as IconSet)}
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
            onChange={(v) => props.onUpdate("defaultHashAlgorithm", v as HashAlgorithm)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Export Options" description="Default export settings">
        <SettingRow label="Default Export Format">
          <SettingsSelect
            value={props.preferences.defaultExportFormat}
            options={[
              { value: "csv", label: "CSV" },
              { value: "json", label: "JSON" },
              { value: "pdf", label: "PDF" },
              { value: "html", label: "HTML" },
              { value: "xml", label: "XML" },
            ]}
            onChange={(v) => props.onUpdate("defaultExportFormat", v as ExportFormat)}
          />
        </SettingRow>

        <SettingRow label="Default View Mode">
          <SettingsSelect
            value={props.preferences.defaultViewMode}
            options={[
              { value: "auto", label: "Auto" },
              { value: "hex", label: "Hex" },
              { value: "text", label: "Text" },
              { value: "preview", label: "Preview" },
            ]}
            onChange={(v) => props.onUpdate("defaultViewMode", v as ViewMode)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Display Options" description="How content is displayed">
        <SettingRow label="Default Sort Order">
          <SettingsSelect
            value={props.preferences.defaultSortOrder}
            options={[
              { value: "name", label: "Name" },
              { value: "date", label: "Date" },
              { value: "size", label: "Size" },
              { value: "type", label: "Type" },
            ]}
            onChange={(v) => props.onUpdate("defaultSortOrder", v as SortOrder)}
          />
        </SettingRow>

        <SettingRow label="Date Format">
          <SettingsSelect
            value={props.preferences.dateFormat}
            options={[
              { value: "iso", label: "ISO (2024-01-15)" },
              { value: "us", label: "US (01/15/2024)" },
              { value: "eu", label: "EU (15/01/2024)" },
              { value: "relative", label: "Relative" },
            ]}
            onChange={(v) => props.onUpdate("dateFormat", v as DateFormat)}
          />
        </SettingRow>

        <SettingRow label="Case-Sensitive Search" description="Make search case-sensitive by default">
          <Toggle
            checked={props.preferences.caseSensitiveSearch}
            onChange={(v) => props.onUpdate("caseSensitiveSearch", v)}
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

        <SettingRow label="Show File Sizes" description="Display file sizes in the tree">
          <Toggle
            checked={props.preferences.showFileSizes}
            onChange={(v) => props.onUpdate("showFileSizes", v)}
          />
        </SettingRow>

        <SettingRow label="Show File Extensions" description="Display file extensions in the tree">
          <Toggle
            checked={props.preferences.showFileExtensions}
            onChange={(v) => props.onUpdate("showFileExtensions", v)}
          />
        </SettingRow>

        <SettingRow label="Remember Last Path" description="Open to last used location">
          <Toggle
            checked={props.preferences.rememberLastPath}
            onChange={(v) => props.onUpdate("rememberLastPath", v)}
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
      <SettingGroup title="Confirmations" description="Prompts before actions">
        <SettingRow label="Confirm Before Delete">
          <Toggle
            checked={props.preferences.confirmBeforeDelete}
            onChange={(v) => props.onUpdate("confirmBeforeDelete", v)}
          />
        </SettingRow>

        <SettingRow label="Confirm Before Export" description="Prompt before exporting files">
          <Toggle
            checked={props.preferences.confirmBeforeExport}
            onChange={(v) => props.onUpdate("confirmBeforeExport", v)}
          />
        </SettingRow>

        <SettingRow label="Confirm Before Hash" description="Prompt before hash operations">
          <Toggle
            checked={props.preferences.confirmBeforeHash}
            onChange={(v) => props.onUpdate("confirmBeforeHash", v)}
          />
        </SettingRow>

        <SettingRow label="Warn on Large Containers" description="Alert when opening large evidence files">
          <Toggle
            checked={props.preferences.warnOnLargeContainers}
            onChange={(v) => props.onUpdate("warnOnLargeContainers", v)}
          />
        </SettingRow>

        <Show when={props.preferences.warnOnLargeContainers}>
          <SettingRow label="Large Container Threshold (GB)" description="Size threshold for warnings">
            <Slider
              value={props.preferences.largeContainerThresholdGb}
              min={10}
              max={500}
              step={10}
              onChange={(v) => props.onUpdate("largeContainerThresholdGb", v)}
            />
          </SettingRow>
        </Show>
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

      <SettingGroup title="Hash Operations" description="Hash behavior settings">
        <SettingRow label="Auto-verify Hashes" description="Automatically verify file hashes on load">
          <Toggle
            checked={props.preferences.autoVerifyHashes}
            onChange={(v) => props.onUpdate("autoVerifyHashes", v)}
          />
        </SettingRow>

        <SettingRow label="Copy Hash to Clipboard" description="Auto-copy computed hashes">
          <Toggle
            checked={props.preferences.copyHashToClipboard}
            onChange={(v) => props.onUpdate("copyHashToClipboard", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Notifications" description="Alerts and sounds">
        <SettingRow label="Enable Sounds" description="Play sounds for events">
          <Toggle
            checked={props.preferences.enableSounds}
            onChange={(v) => props.onUpdate("enableSounds", v)}
          />
        </SettingRow>

        <SettingRow label="Enable Notifications" description="Show system notifications">
          <Toggle
            checked={props.preferences.enableNotifications}
            onChange={(v) => props.onUpdate("enableNotifications", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Logging" description="Application logging settings">
        <SettingRow label="Log Level" description="Detail level for application logs">
          <SettingsSelect
            value={props.preferences.logLevel}
            options={[
              { value: "error", label: "Error" },
              { value: "warn", label: "Warning" },
              { value: "info", label: "Info" },
              { value: "debug", label: "Debug" },
            ]}
            onChange={(v) => props.onUpdate("logLevel", v as LogLevel)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}

// Performance Tab
function PerformanceSettings(props: {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}) {
  return (
    <>
      <SettingGroup title="Loading" description="Control how data is loaded">
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
            max={16}
            onChange={(v) => props.onUpdate("maxConcurrentOperations", v)}
          />
        </SettingRow>

        <SettingRow label="Worker Threads" description="Background worker threads">
          <Slider
            value={props.preferences.workerThreads}
            min={1}
            max={16}
            onChange={(v) => props.onUpdate("workerThreads", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Memory" description="Memory and caching settings">
        <SettingRow label="Cache Size (MB)" description="Memory allocated for caching">
          <Slider
            value={props.preferences.cacheSizeMb}
            min={64}
            max={1024}
            step={64}
            onChange={(v) => props.onUpdate("cacheSizeMb", v)}
          />
        </SettingRow>

        <SettingRow label="Max Preview Size (MB)" description="Maximum file size for previews">
          <Slider
            value={props.preferences.maxPreviewSizeMb}
            min={1}
            max={100}
            onChange={(v) => props.onUpdate("maxPreviewSizeMb", v)}
          />
        </SettingRow>

        <SettingRow label="Chunk Size (KB)" description="Data chunk size for processing">
          <SettingsSelect
            value={String(props.preferences.chunkSizeKb)}
            options={[
              { value: "256", label: "256 KB" },
              { value: "512", label: "512 KB" },
              { value: "1024", label: "1 MB" },
              { value: "2048", label: "2 MB" },
              { value: "4096", label: "4 MB" },
            ]}
            onChange={(v) => props.onUpdate("chunkSizeKb", Number(v))}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Advanced" description="Advanced performance options">
        <SettingRow label="Hardware Acceleration" description="Use GPU for rendering">
          <Toggle
            checked={props.preferences.useHardwareAcceleration}
            onChange={(v) => props.onUpdate("useHardwareAcceleration", v)}
          />
        </SettingRow>

        <SettingRow label="Enable Memory Mapping" description="Use mmap for large files">
          <Toggle
            checked={props.preferences.enableMmap}
            onChange={(v) => props.onUpdate("enableMmap", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}

// Security Tab
function SecuritySettings(props: {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}) {
  return (
    <>
      <SettingGroup title="Clipboard" description="Clipboard security settings">
        <SettingRow label="Clear Clipboard on Close" description="Remove copied data when app closes">
          <Toggle
            checked={props.preferences.clearClipboardOnClose}
            onChange={(v) => props.onUpdate("clearClipboardOnClose", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Audit" description="Audit and logging settings">
        <SettingRow label="Enable Audit Logging" description="Log all forensic operations">
          <Toggle
            checked={props.preferences.auditLogging}
            onChange={(v) => props.onUpdate("auditLogging", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Hash Verification" description="Hash verification settings">
        <SettingRow label="Verification Mode" description="How hashes are verified">
          <SettingsSelect
            value={props.preferences.hashVerificationMode}
            options={[
              { value: "any", label: "Any Algorithm" },
              { value: "same-algo", label: "Same Algorithm Only" },
              { value: "multiple", label: "Multiple Algorithms" },
            ]}
            onChange={(v) => props.onUpdate("hashVerificationMode", v as HashVerificationMode)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}

// Paths Tab
function PathsSettings(props: {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}) {
  const handleBrowse = async (key: "defaultEvidencePath" | "defaultExportPath" | "tempFolderPath") => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Folder",
      });
      if (selected && typeof selected === "string") {
        props.onUpdate(key, selected);
      }
    } catch (err) {
      console.error("Failed to open folder dialog:", err);
    }
  };

  return (
    <>
      <SettingGroup title="Default Paths" description="Default folder locations">
        <SettingRow label="Default Evidence Path" description="Where to look for evidence files">
          <div class="flex items-center gap-2">
            <input
              type="text"
              class="input-inline"
              value={props.preferences.defaultEvidencePath}
              onChange={(e) => props.onUpdate("defaultEvidencePath", e.currentTarget.value)}
              placeholder="Not set"
            />
            <button
              class="btn-sm"
              onClick={() => handleBrowse("defaultEvidencePath")}
            >
              Browse
            </button>
          </div>
        </SettingRow>

        <SettingRow label="Default Export Path" description="Where to save exported files">
          <div class="flex items-center gap-2">
            <input
              type="text"
              class="input-inline"
              value={props.preferences.defaultExportPath}
              onChange={(e) => props.onUpdate("defaultExportPath", e.currentTarget.value)}
              placeholder="Not set"
            />
            <button
              class="btn-sm"
              onClick={() => handleBrowse("defaultExportPath")}
            >
              Browse
            </button>
          </div>
        </SettingRow>

        <SettingRow label="Temp Folder Path" description="Location for temporary files">
          <div class="flex items-center gap-2">
            <input
              type="text"
              class="input-inline"
              value={props.preferences.tempFolderPath}
              onChange={(e) => props.onUpdate("tempFolderPath", e.currentTarget.value)}
              placeholder="System default"
            />
            <button
              class="btn-sm"
              onClick={() => handleBrowse("tempFolderPath")}
            >
              Browse
            </button>
          </div>
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Recent Files" description="Recent files settings">
        <SettingRow label="Recent Files Count" description="Number of recent files to remember">
          <Slider
            value={props.preferences.recentFilesCount}
            min={5}
            max={50}
            step={5}
            onChange={(v) => props.onUpdate("recentFilesCount", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}

// Reports Tab
function ReportsSettings(props: {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}) {
  const handleBrowseLogo = async () => {
    try {
      const selected = await open({
        multiple: false,
        title: "Select Logo Image",
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "svg"] }],
      });
      if (selected && typeof selected === "string") {
        props.onUpdate("reportLogoPath", selected);
      }
    } catch (err) {
      console.error("Failed to open file dialog:", err);
    }
  };

  return (
    <>
      <SettingGroup title="Report Template" description="Default report settings">
        <SettingRow label="Default Template">
          <SettingsSelect
            value={props.preferences.defaultReportTemplate}
            options={[
              { value: "standard", label: "Standard" },
              { value: "detailed", label: "Detailed" },
              { value: "summary", label: "Summary" },
              { value: "custom", label: "Custom" },
            ]}
            onChange={(v) => props.onUpdate("defaultReportTemplate", v as ReportTemplate)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Report Content" description="What to include in reports">
        <SettingRow label="Include Hashes" description="Add file hashes to reports">
          <Toggle
            checked={props.preferences.includeHashesInReports}
            onChange={(v) => props.onUpdate("includeHashesInReports", v)}
          />
        </SettingRow>

        <SettingRow label="Include Timestamps" description="Add timestamps to reports">
          <Toggle
            checked={props.preferences.includeTimestampsInReports}
            onChange={(v) => props.onUpdate("includeTimestampsInReports", v)}
          />
        </SettingRow>

        <SettingRow label="Include Metadata" description="Add file metadata to reports">
          <Toggle
            checked={props.preferences.includeMetadataInReports}
            onChange={(v) => props.onUpdate("includeMetadataInReports", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Branding" description="Organization branding for reports">
        <SettingRow label="Report Logo" description="Logo image for reports">
          <div class="flex items-center gap-2">
            <input
              type="text"
              class="input-inline"
              value={props.preferences.reportLogoPath}
              onChange={(e) => props.onUpdate("reportLogoPath", e.currentTarget.value)}
              placeholder="No logo set"
            />
            <button
              class="btn-sm"
              onClick={handleBrowseLogo}
            >
              Browse
            </button>
          </div>
        </SettingRow>

        <SettingRow label="Examiner Name" description="Name shown on reports">
          <input
            type="text"
            class="input-inline"
            value={props.preferences.examinerName}
            onChange={(e) => props.onUpdate("examinerName", e.currentTarget.value)}
            placeholder="Enter name"
          />
        </SettingRow>

        <SettingRow label="Organization Name" description="Organization shown on reports">
          <input
            type="text"
            class="input-inline"
            value={props.preferences.organizationName}
            onChange={(e) => props.onUpdate("organizationName", e.currentTarget.value)}
            placeholder="Enter organization"
          />
        </SettingRow>

        <SettingRow label="Case Number Prefix" description="Prefix for case numbers">
          <input
            type="text"
            class="input-inline"
            value={props.preferences.caseNumberPrefix}
            onChange={(e) => props.onUpdate("caseNumberPrefix", e.currentTarget.value)}
            placeholder="e.g., CASE-"
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
              <div class="flex items-center justify-between py-1.5">
                <span class="text-sm text-txt">{shortcutLabels[action] ?? action}</span>
                <button
                  class={`px-2 py-1 text-xs rounded border transition-colors ${
                    props.editingShortcut() === action
                      ? "border-accent bg-accent/10 text-accent"
                      : "border-border hover:border-accent text-txt-secondary"
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

      <p class="text-xs text-txt-muted mt-1">
        Tip: Click a shortcut button then press your desired key combination.
        Press Escape to cancel.
      </p>
    </>
  );
}
