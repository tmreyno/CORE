/**
 * DefaultsSettings Tab
 * Settings for default views, hash algorithms, and filters
 */

import type { SettingsUpdateProps } from "../types";
import { SettingGroup } from "../SettingGroup";
import { SettingRow } from "../SettingRow";
import { SettingsSelect } from "../SettingsSelect";
import { Toggle } from "../../ui";
import type { HashAlgorithm, ExportFormat, ViewMode, SortOrder, DateFormat } from "../../preferences";

export function DefaultsSettings(props: SettingsUpdateProps) {
  return (
    <>
      <SettingGroup title="View Defaults" description="Default display settings">
        <SettingRow label="Default View Mode">
          <SettingsSelect
            value={props.preferences.defaultViewMode}
            options={[
              { value: "hex", label: "Hex View" },
              { value: "text", label: "Text View" },
              { value: "preview", label: "Preview" },
              { value: "auto", label: "Auto Detect" },
            ]}
            onChange={(v) => props.onUpdate("defaultViewMode", v as ViewMode)}
          />
        </SettingRow>

        <SettingRow label="Default Sort Order">
          <SettingsSelect
            value={props.preferences.defaultSortOrder}
            options={[
              { value: "name", label: "Name" },
              { value: "size", label: "Size" },
              { value: "date", label: "Date" },
              { value: "type", label: "Type" },
            ]}
            onChange={(v) => props.onUpdate("defaultSortOrder", v as SortOrder)}
          />
        </SettingRow>

        <SettingRow label="Date Format">
          <SettingsSelect
            value={props.preferences.dateFormat}
            options={[
              { value: "iso", label: "ISO (YYYY-MM-DD)" },
              { value: "us", label: "US (MM/DD/YYYY)" },
              { value: "eu", label: "EU (DD/MM/YYYY)" },
              { value: "relative", label: "Relative" },
            ]}
            onChange={(v) => props.onUpdate("dateFormat", v as DateFormat)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Hash Settings" description="Default hash algorithm settings">
        <SettingRow label="Default Hash Algorithm">
          <SettingsSelect
            value={props.preferences.defaultHashAlgorithm}
            options={[
              { value: "SHA256", label: "SHA-256" },
              { value: "SHA1", label: "SHA-1" },
              { value: "MD5", label: "MD5" },
              { value: "SHA512", label: "SHA-512" },
              { value: "Blake3", label: "BLAKE3" },
              { value: "XXH3", label: "XXH3" },
            ]}
            onChange={(v) => props.onUpdate("defaultHashAlgorithm", v as HashAlgorithm)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Export Defaults" description="Default export settings">
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
      </SettingGroup>

      <SettingGroup title="File Display" description="File display options">
        <SettingRow label="Show Hidden Files">
          <Toggle
            checked={props.preferences.showHiddenFiles}
            onChange={(v) => props.onUpdate("showHiddenFiles", v)}
          />
        </SettingRow>

        <SettingRow label="Show File Sizes">
          <Toggle
            checked={props.preferences.showFileSizes}
            onChange={(v) => props.onUpdate("showFileSizes", v)}
          />
        </SettingRow>

        <SettingRow label="Show File Extensions">
          <Toggle
            checked={props.preferences.showFileExtensions}
            onChange={(v) => props.onUpdate("showFileExtensions", v)}
          />
        </SettingRow>

        <SettingRow label="Auto-expand Tree">
          <Toggle
            checked={props.preferences.autoExpandTree}
            onChange={(v) => props.onUpdate("autoExpandTree", v)}
          />
        </SettingRow>

        <SettingRow label="Remember Last Path">
          <Toggle
            checked={props.preferences.rememberLastPath}
            onChange={(v) => props.onUpdate("rememberLastPath", v)}
          />
        </SettingRow>

        <SettingRow label="Case-sensitive Search">
          <Toggle
            checked={props.preferences.caseSensitiveSearch}
            onChange={(v) => props.onUpdate("caseSensitiveSearch", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}
