/**
 * BehaviorSettings Tab
 * Settings for application behavior and confirmations
 */

import type { SettingsUpdateProps } from "../types";
import { SettingGroup } from "../SettingGroup";
import { SettingRow } from "../SettingRow";
import { Toggle, Slider } from "../../ui";
import { SettingsSelect } from "../SettingsSelect";
import type { LogLevel } from "../../preferences";

export function BehaviorSettings(props: SettingsUpdateProps) {
  return (
    <>
      <SettingGroup title="Confirmations" description="When to show confirmation dialogs">
        <SettingRow label="Confirm Before Delete" description="Ask before deleting files">
          <Toggle
            checked={props.preferences.confirmBeforeDelete}
            onChange={(v) => props.onUpdate("confirmBeforeDelete", v)}
          />
        </SettingRow>

        <SettingRow label="Confirm Before Export" description="Ask before exporting files">
          <Toggle
            checked={props.preferences.confirmBeforeExport}
            onChange={(v) => props.onUpdate("confirmBeforeExport", v)}
          />
        </SettingRow>

        <SettingRow label="Confirm Before Hash" description="Ask before computing hashes">
          <Toggle
            checked={props.preferences.confirmBeforeHash}
            onChange={(v) => props.onUpdate("confirmBeforeHash", v)}
          />
        </SettingRow>

        <SettingRow label="Warn on Large Containers" description="Warn when opening large containers">
          <Toggle
            checked={props.preferences.warnOnLargeContainers}
            onChange={(v) => props.onUpdate("warnOnLargeContainers", v)}
          />
        </SettingRow>

        <SettingRow label="Large Container Threshold (GB)" description="Size threshold for warnings">
          <Slider
            value={props.preferences.largeContainerThresholdGb}
            min={10}
            max={200}
            step={10}
            onChange={(v) => props.onUpdate("largeContainerThresholdGb", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Auto-save" description="Automatic project saving">
        <SettingRow label="Auto-save Project" description="Automatically save project changes">
          <Toggle
            checked={props.preferences.autoSaveProject}
            onChange={(v) => props.onUpdate("autoSaveProject", v)}
          />
        </SettingRow>

        <SettingRow label="Auto-save Interval (sec)" description="Time between auto-saves">
          <Slider
            value={props.preferences.autoSaveIntervalMs / 1000}
            min={30}
            max={300}
            step={30}
            onChange={(v) => props.onUpdate("autoSaveIntervalMs", v * 1000)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Hash Behavior" description="Hash computation behavior">
        <SettingRow label="Auto-verify Hashes" description="Automatically verify hashes on load">
          <Toggle
            checked={props.preferences.autoVerifyHashes}
            onChange={(v) => props.onUpdate("autoVerifyHashes", v)}
          />
        </SettingRow>

        <SettingRow label="Copy Hash to Clipboard" description="Copy computed hashes to clipboard">
          <Toggle
            checked={props.preferences.copyHashToClipboard}
            onChange={(v) => props.onUpdate("copyHashToClipboard", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Notifications" description="Notification settings">
        <SettingRow label="Enable Sounds" description="Play sounds for notifications">
          <Toggle
            checked={props.preferences.enableSounds}
            onChange={(v) => props.onUpdate("enableSounds", v)}
          />
        </SettingRow>

        <SettingRow label="Enable Notifications" description="Show desktop notifications">
          <Toggle
            checked={props.preferences.enableNotifications}
            onChange={(v) => props.onUpdate("enableNotifications", v)}
          />
        </SettingRow>

        <SettingRow label="Log Level" description="Logging verbosity">
          <SettingsSelect
            value={props.preferences.logLevel}
            options={[
              { value: "error", label: "Error Only" },
              { value: "warn", label: "Warnings" },
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
