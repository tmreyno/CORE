/**
 * SecuritySettings Tab
 * Settings for clipboard security, audit logging, and hash verification
 */

import type { SettingsUpdateProps } from "../types";
import { SettingGroup } from "../SettingGroup";
import { SettingRow } from "../SettingRow";
import { SettingsSelect } from "../SettingsSelect";
import { Toggle } from "../../ui";
import type { HashVerificationMode } from "../../preferences";

export function SecuritySettings(props: SettingsUpdateProps) {
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
