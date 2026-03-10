// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { Toggle, Slider } from "../ui";
import { SettingGroup, SettingRow, SettingsSelect } from "../settings";
import type { AppPreferences, LogLevel } from "../preferences";

interface BehaviorSettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const BehaviorSettings: Component<BehaviorSettingsProps> = (props) => {
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
        <SettingRow label="Auto-hash on Selection" description="Automatically hash files when selected. Warning: this slows down evidence viewing and is not recommended for large containers.">
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
};
