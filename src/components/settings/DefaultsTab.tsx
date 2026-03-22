// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
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
import { isAcquireEdition, isFullEdition } from "../../utils/edition";

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

      <Show when={isAcquireEdition()}>
        <SettingGroup title="Acquisition Defaults" description="Default settings for forensic image creation">
          <SettingRow label="Default Format" description="Forensic image container format">
            <SettingsSelect
              value={props.preferences.defaultAcquisitionFormat}
              options={[
                { value: "e01", label: "E01 (EnCase)" },
                { value: "raw", label: "Raw (.dd)" },
                { value: "l01", label: "L01 (Logical)" },
                { value: "aff4", label: "AFF4" },
                { value: "7z", label: "7z Archive" },
              ]}
              onChange={(v) => props.onUpdate("defaultAcquisitionFormat", v)}
            />
          </SettingRow>

          <SettingRow label="Compression" description="Default compression level">
            <SettingsSelect
              value={props.preferences.defaultAcquisitionCompression}
              options={[
                { value: "none", label: "None (fastest)" },
                { value: "fast", label: "Fast" },
                { value: "best", label: "Best (smallest)" },
              ]}
              onChange={(v) => props.onUpdate("defaultAcquisitionCompression", v)}
            />
          </SettingRow>

          <SettingRow label="Segment Size" description="Split output into segments">
            <SettingsSelect
              value={String(props.preferences.defaultAcquisitionSegmentMb)}
              options={[
                { value: "0", label: "No splitting" },
                { value: "650", label: "650 MB (CD)" },
                { value: "2048", label: "2 GB (FAT32/FTK)" },
                { value: "4096", label: "4 GB (DVD)" },
                { value: "4700", label: "4.7 GB (DVD-SL)" },
              ]}
              onChange={(v) => props.onUpdate("defaultAcquisitionSegmentMb", Number(v))}
            />
          </SettingRow>

          <SettingRow label="Hash: MD5" description="Compute MD5 hash during acquisition">
            <Toggle
              checked={props.preferences.defaultAcquisitionHashMd5}
              onChange={(v) => props.onUpdate("defaultAcquisitionHashMd5", v)}
            />
          </SettingRow>

          <SettingRow label="Hash: SHA-1" description="Compute SHA-1 hash during acquisition">
            <Toggle
              checked={props.preferences.defaultAcquisitionHashSha1}
              onChange={(v) => props.onUpdate("defaultAcquisitionHashSha1", v)}
            />
          </SettingRow>

          <SettingRow label="Hash: SHA-256" description="Compute SHA-256 hash during acquisition">
            <Toggle
              checked={props.preferences.defaultAcquisitionHashSha256}
              onChange={(v) => props.onUpdate("defaultAcquisitionHashSha256", v)}
            />
          </SettingRow>

          <SettingRow label="Verify After Acquisition" description="Automatically verify the image after creation">
            <Toggle
              checked={props.preferences.autoVerifyAfterAcquisition}
              onChange={(v) => props.onUpdate("autoVerifyAfterAcquisition", v)}
            />
          </SettingRow>

          <SettingRow label="Write Companion File" description="Create .ffx-companion.json sidecar after acquisition">
            <Toggle
              checked={props.preferences.writeCompanionFile}
              onChange={(v) => props.onUpdate("writeCompanionFile", v)}
            />
          </SettingRow>
        </SettingGroup>
      </Show>

      <Show when={isFullEdition()}>
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
      </Show>

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

      <Show when={isFullEdition()}>
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
      </Show>
    </>
  );
};
