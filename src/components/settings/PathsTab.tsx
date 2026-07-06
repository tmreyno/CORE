// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, createSignal } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { Slider } from "../ui";
import { SettingGroup, SettingRow } from "../settings";
import type { AppPreferences } from "../preferences";
import { isTauri } from "../../utils/platform";
import { logger } from "../../utils/logger";
const log = logger.scope("PathsTab");

const BROWSER_FOLDER_MESSAGE =
  "Folder browsing is available in the desktop app. In browser preview, enter the path manually.";

interface PathsSettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const PathsSettings: Component<PathsSettingsProps> = (props) => {
  const [browseMessage, setBrowseMessage] = createSignal<string | null>(null);

  const handleBrowse = async (key: "defaultEvidencePath" | "defaultExportPath" | "tempFolderPath") => {
    if (!isTauri) {
      setBrowseMessage(BROWSER_FOLDER_MESSAGE);
      return;
    }

    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Folder",
      });
      if (selected && typeof selected === "string") {
        props.onUpdate(key, selected);
      }
    } catch (err) {
      log.error("Failed to open folder dialog:", err);
    }
  };

  return (
    <>
      <SettingGroup title="Default Paths" description="Default folder locations">
        <Show when={browseMessage()}>
          {(message) => (
            <div class="text-xs text-warning bg-warning/10 border border-warning/20 rounded px-2 py-1.5 mb-2">
              {message()}
            </div>
          )}
        </Show>

        <SettingRow label="Default Evidence Path" description="Where to look for evidence files">
          <div class="flex items-center gap-2">
            <input
              type="text"
              class="input-inline"
              value={props.preferences.defaultEvidencePath}
              onChange={(e) => props.onUpdate("defaultEvidencePath", e.currentTarget.value)}
              placeholder="Not set"
            />
            <button
              class="btn-sm"
              onClick={() => handleBrowse("defaultEvidencePath")}
            >
              Browse
            </button>
          </div>
        </SettingRow>

        <SettingRow label="Default Export Path" description="Where to save exported files">
          <div class="flex items-center gap-2">
            <input
              type="text"
              class="input-inline"
              value={props.preferences.defaultExportPath}
              onChange={(e) => props.onUpdate("defaultExportPath", e.currentTarget.value)}
              placeholder="Not set"
            />
            <button
              class="btn-sm"
              onClick={() => handleBrowse("defaultExportPath")}
            >
              Browse
            </button>
          </div>
        </SettingRow>

        <SettingRow label="Temp Folder Path" description="Location for temporary files">
          <div class="flex items-center gap-2">
            <input
              type="text"
              class="input-inline"
              value={props.preferences.tempFolderPath}
              onChange={(e) => props.onUpdate("tempFolderPath", e.currentTarget.value)}
              placeholder="System default"
            />
            <button
              class="btn-sm"
              onClick={() => handleBrowse("tempFolderPath")}
            >
              Browse
            </button>
          </div>
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Recent Files" description="Recent files settings">
        <SettingRow label="Recent Files Count" description="Number of recent files to remember">
          <Slider
            value={props.preferences.recentFilesCount}
            min={5}
            max={50}
            step={5}
            onChange={(v) => props.onUpdate("recentFilesCount", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
};
