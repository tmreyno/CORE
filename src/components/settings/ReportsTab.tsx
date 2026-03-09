// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { Toggle } from "../ui";
import { SettingGroup, SettingRow, SettingsSelect } from "../settings";
import type { AppPreferences, ReportPreset } from "../preferences";
import { getActiveUserProfile } from "../preferences";
import { REPORT_PRESETS } from "../report/constants";
import { HiOutlineUserCircle } from "../icons";
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
      <SettingGroup title="Report Preset" description="Default report settings">
        <SettingRow label="Default Preset">
          <SettingsSelect
            value={props.preferences.defaultReportPreset}
            options={REPORT_PRESETS.map(p => ({ value: p.id, label: p.name }))}
            onChange={(v) => props.onUpdate("defaultReportPreset", v as ReportPreset)}
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
        <Show when={getActiveUserProfile()}>
          {(profile) => (
            <div class="flex items-center gap-2 mb-2 px-2 py-1.5 rounded-lg bg-accent/5 border border-accent/20 text-xs">
              <HiOutlineUserCircle class="w-4 h-4 text-accent" />
              <span class="text-txt-muted">Linked to profile:</span>
              <span class="font-medium text-txt">{profile().name}</span>
              <span class="text-txt-muted ml-auto">Edit in Users & Profiles tab</span>
            </div>
          )}
        </Show>
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

        <SettingRow label="Title / Role" description="Job title or role">
          <input
            type="text"
            class="input-inline"
            value={props.preferences.examinerTitle}
            onChange={(e) => props.onUpdate("examinerTitle", e.currentTarget.value)}
            placeholder="e.g., Senior Digital Forensic Examiner"
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

        <SettingRow label="Default Agency" description="Requesting agency for reports">
          <input
            type="text"
            class="input-inline"
            value={props.preferences.defaultAgency}
            onChange={(e) => props.onUpdate("defaultAgency", e.currentTarget.value)}
            placeholder="e.g., Metro Police Department"
          />
        </SettingRow>

        <SettingRow label="Badge Number" description="Badge or ID number">
          <input
            type="text"
            class="input-inline"
            value={props.preferences.examinerBadge}
            onChange={(e) => props.onUpdate("examinerBadge", e.currentTarget.value)}
            placeholder="Enter badge number"
          />
        </SettingRow>

        <SettingRow label="Email" description="Contact email for reports">
          <input
            type="text"
            class="input-inline"
            value={props.preferences.examinerEmail}
            onChange={(e) => props.onUpdate("examinerEmail", e.currentTarget.value)}
            placeholder="examiner@example.com"
          />
        </SettingRow>

        <SettingRow label="Phone" description="Contact phone for reports">
          <input
            type="text"
            class="input-inline"
            value={props.preferences.examinerPhone}
            onChange={(e) => props.onUpdate("examinerPhone", e.currentTarget.value)}
            placeholder="(555) 123-4567"
          />
        </SettingRow>

        <SettingRow label="Certifications" description="Comma-separated list of certifications">
          <input
            type="text"
            class="input-inline"
            value={(props.preferences.examinerCertifications || []).join(", ")}
            onChange={(e) => props.onUpdate("examinerCertifications", 
              e.currentTarget.value.split(",").map(s => s.trim()).filter(Boolean)
            )}
            placeholder="e.g., EnCE, GCFE, CFCE"
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
