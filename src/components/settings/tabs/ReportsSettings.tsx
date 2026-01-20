/**
 * ReportsSettings Tab
 * Settings for report templates, content, and branding
 */

import { open } from "@tauri-apps/plugin-dialog";
import type { SettingsUpdateProps } from "../types";
import { SettingGroup } from "../SettingGroup";
import { SettingRow } from "../SettingRow";
import { SettingsSelect } from "../SettingsSelect";
import { Toggle } from "../../ui";
import type { ReportTemplate } from "../../preferences";

export function ReportsSettings(props: SettingsUpdateProps) {
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
              class="flex-1 px-2 py-1 text-xs bg-bg-panel border border-border rounded text-txt"
              value={props.preferences.reportLogoPath}
              onInput={(e) => props.onUpdate("reportLogoPath", e.currentTarget.value)}
              placeholder="No logo set"
            />
            <button
              class="px-2 py-1 text-xs rounded border border-border bg-bg-panel text-txt hover:bg-bg-hover transition-colors"
              onClick={handleBrowseLogo}
            >
              Browse
            </button>
          </div>
        </SettingRow>

        <SettingRow label="Examiner Name" description="Name shown on reports">
          <input
            type="text"
            class="flex-1 px-2 py-1.5 text-sm bg-bg-panel border border-border rounded text-txt focus:outline-none focus:ring-1 focus:ring-accent"
            value={props.preferences.examinerName}
            onInput={(e) => props.onUpdate("examinerName", e.currentTarget.value)}
            placeholder="Enter name"
          />
        </SettingRow>

        <SettingRow label="Organization Name" description="Organization shown on reports">
          <input
            type="text"
            class="flex-1 px-2 py-1.5 text-sm bg-bg-panel border border-border rounded text-txt focus:outline-none focus:ring-1 focus:ring-accent"
            value={props.preferences.organizationName}
            onInput={(e) => props.onUpdate("organizationName", e.currentTarget.value)}
            placeholder="Enter organization"
          />
        </SettingRow>

        <SettingRow label="Case Number Prefix" description="Prefix for case numbers">
          <input
            type="text"
            class="flex-1 px-2 py-1 text-xs bg-bg-panel border border-border rounded text-txt focus:outline-none focus:ring-1 focus:ring-accent"
            value={props.preferences.caseNumberPrefix}
            onInput={(e) => props.onUpdate("caseNumberPrefix", e.currentTarget.value)}
            placeholder="e.g., CASE-"
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}
