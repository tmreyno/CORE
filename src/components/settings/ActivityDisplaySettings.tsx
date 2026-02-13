// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ActivityDisplaySettings Tab
 * Settings for activity tracking display options
 */

import type { SettingsUpdateProps } from "./types";
import { SettingGroup } from "./SettingGroup";
import { SettingRow } from "./SettingRow";
import { SettingsSelect } from "./SettingsSelect";
import { Toggle } from "../ui";
import type { ActivityGrouping, ActivitySortOrder } from "../preferences";

export function ActivityDisplaySettings(props: SettingsUpdateProps) {
  return (
    <>
      <SettingGroup title="Display Options" description="Control what information is shown for each activity">
        <SettingRow label="Show Transfer Speed" description="Display current transfer speed (MB/s) for active operations">
          <Toggle
            checked={props.preferences.activityShowSpeed}
            onChange={(checked) => props.onUpdate("activityShowSpeed", checked)}
          />
        </SettingRow>
        
        <SettingRow label="Show Estimated Time" description="Display estimated time remaining (ETA) for active operations">
          <Toggle
            checked={props.preferences.activityShowETA}
            onChange={(checked) => props.onUpdate("activityShowETA", checked)}
          />
        </SettingRow>
        
        <SettingRow label="Show Current File" description="Display the name of the file currently being processed">
          <Toggle
            checked={props.preferences.activityShowCurrentFile}
            onChange={(checked) => props.onUpdate("activityShowCurrentFile", checked)}
          />
        </SettingRow>
        
        <SettingRow label="Show File Count" description="Display progress as file count (e.g., 23/150 files)">
          <Toggle
            checked={props.preferences.activityShowFileCount}
            onChange={(checked) => props.onUpdate("activityShowFileCount", checked)}
          />
        </SettingRow>
        
        <SettingRow label="Show Compression Ratio" description="Display compression percentage for archive operations">
          <Toggle
            checked={props.preferences.activityShowCompressionRatio}
            onChange={(checked) => props.onUpdate("activityShowCompressionRatio", checked)}
          />
        </SettingRow>
        
        <SettingRow label="Show Thread Count" description="Display number of threads used for the operation">
          <Toggle
            checked={props.preferences.activityShowThreadCount}
            onChange={(checked) => props.onUpdate("activityShowThreadCount", checked)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Visual Effects" description="Customize visual feedback and animations">
        <SettingRow label="Color-Coded Speed" description="Use colors to indicate transfer speed (green: fast, blue: medium, yellow: slow)">
          <Toggle
            checked={props.preferences.activityColorCodedSpeed}
            onChange={(checked) => props.onUpdate("activityColorCodedSpeed", checked)}
          />
        </SettingRow>
        
        <SettingRow label="Pulse Animation" description="Show animated pulse effect for active operations">
          <Toggle
            checked={props.preferences.activityPulseAnimation}
            onChange={(checked) => props.onUpdate("activityPulseAnimation", checked)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Organization" description="Configure how activities are grouped and sorted">
        <SettingRow label="Group By" description="Control how activities are organized in the panel">
          <SettingsSelect
            value={props.preferences.activityGrouping}
            options={[
              { value: "none", label: "No Grouping" },
              { value: "status", label: "Status (Active / Finished)" },
              { value: "type", label: "Type (Archives / Exports / Copies)" },
            ]}
            onChange={(v) => props.onUpdate("activityGrouping", v as ActivityGrouping)}
          />
        </SettingRow>

        <SettingRow label="Sort By" description="Control the order of activities within each group">
          <SettingsSelect
            value={props.preferences.activitySortOrder}
            options={[
              { value: "newest", label: "Newest First" },
              { value: "oldest", label: "Oldest First" },
              { value: "name", label: "Name (A-Z)" },
              { value: "progress", label: "Progress (%)" },
            ]}
            onChange={(v) => props.onUpdate("activitySortOrder", v as ActivitySortOrder)}
          />
        </SettingRow>

        <SettingRow label="Maximum Visible" description="Limit how many activities are shown (1-100)">
          <input
            type="number"
            class="input-xs w-20"
            min={1}
            max={100}
            value={props.preferences.activityMaxVisible}
            onChange={(e) => props.onUpdate("activityMaxVisible", parseInt(e.currentTarget.value, 10))}
          />
        </SettingRow>

        <SettingRow label="Auto-Collapse Finished" description="Automatically collapse completed or failed activities">
          <Toggle
            checked={props.preferences.activityAutoCollapse}
            onChange={(checked) => props.onUpdate("activityAutoCollapse", checked)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}
