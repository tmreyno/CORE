// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component } from "solid-js";
import { Toggle } from "../ui";
import { SettingGroup, SettingRow, SettingsSelect } from "../settings";
import type { AppPreferences, HashVerificationMode } from "../preferences";

interface SecuritySettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const SecuritySettings: Component<SecuritySettingsProps> = (props) => {
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
};
