/**
 * PathsSettings Tab
 * Settings for default file paths and recent files
 */

import { open } from "@tauri-apps/plugin-dialog";
import type { SettingsUpdateProps } from "../types";
import { SettingGroup } from "../SettingGroup";
import { SettingRow } from "../SettingRow";
import { Slider } from "../../ui";
import { logger } from "../../../utils/logger";
const log = logger.scope("PathsSettings");

export function PathsSettings(props: SettingsUpdateProps) {
  const handleBrowse = async (key: "defaultEvidencePath" | "defaultExportPath" | "tempFolderPath") => {
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
        <SettingRow label="Default Evidence Path" description="Where to look for evidence files">
          <div class="flex items-center gap-2">
            <input
              type="text"
              class="input-inline"
              value={props.preferences.defaultEvidencePath}
              onInput={(e) => props.onUpdate("defaultEvidencePath", e.currentTarget.value)}
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
              onInput={(e) => props.onUpdate("defaultExportPath", e.currentTarget.value)}
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
              onInput={(e) => props.onUpdate("tempFolderPath", e.currentTarget.value)}
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
}
