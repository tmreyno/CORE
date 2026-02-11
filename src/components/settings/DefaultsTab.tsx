// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component } from "solid-js";
import { Toggle } from "../ui";
import { SettingGroup, SettingRow, SettingsSelect } from "../settings";
import type { 
  AppPreferences, 
  HashAlgorithm, 
  ExportFormat, 
  ViewMode, 
  SortOrder, 
  DateFormat 
} from "../preferences";

interface DefaultsSettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const DefaultsSettings: Component<DefaultsSettingsProps> = (props) => {
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
};
