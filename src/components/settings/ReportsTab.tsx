// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { Toggle } from "../ui";
import { SettingGroup, SettingRow, SettingsSelect } from "../settings";
import type { AppPreferences, ReportTemplate } from "../preferences";
import { logger } from "../../utils/logger";
const log = logger.scope("ReportsTab");

interface ReportsSettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const ReportsSettings: Component<ReportsSettingsProps> = (props) => {
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
      log.error("Failed to open file dialog:", err);
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
};
