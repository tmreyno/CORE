// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component } from "solid-js";
import { Toggle, Slider } from "../ui";
import { SettingGroup, SettingRow, SettingsSelect } from "../settings";
import type { 
  AppPreferences, 
  Theme, 
  AccentColor, 
  SidebarPosition, 
  TreeDensity, 
  IconSet 
} from "../preferences";

interface AppearanceSettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const AppearanceSettings: Component<AppearanceSettingsProps> = (props) => {
  return (
    <>
      <SettingGroup title="Theme" description="Choose your preferred color scheme">
        <SettingRow label="Theme">
          <SettingsSelect
            value={props.preferences.theme}
            options={[
              { value: "dark", label: "Dark" },
              { value: "light", label: "Light (Auto)" },
              { value: "light-macos", label: "Light (macOS)" },
              { value: "light-windows", label: "Light (Windows)" },
              { value: "midnight", label: "Midnight" },
              { value: "system", label: "System" },
            ]}
            onChange={(v) => props.onUpdate("theme", v as Theme)}
          />
        </SettingRow>

        <SettingRow label="Accent Color" description="Primary accent color for UI elements">
          <SettingsSelect
            value={props.preferences.accentColor}
            options={[
              { value: "cyan", label: "Cyan" },
              { value: "blue", label: "Blue" },
              { value: "green", label: "Green" },
              { value: "purple", label: "Purple" },
              { value: "orange", label: "Orange" },
              { value: "red", label: "Red" },
            ]}
            onChange={(v) => props.onUpdate("accentColor", v as AccentColor)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Layout" description="Customize the interface layout">
        <SettingRow label="Sidebar Position">
          <SettingsSelect
            value={props.preferences.sidebarPosition}
            options={[
              { value: "left", label: "Left" },
              { value: "right", label: "Right" },
            ]}
            onChange={(v) => props.onUpdate("sidebarPosition", v as SidebarPosition)}
          />
        </SettingRow>

        <SettingRow label="Show Status Bar" description="Display the status bar at the bottom">
          <Toggle
            checked={props.preferences.showStatusBar}
            onChange={(v) => props.onUpdate("showStatusBar", v)}
          />
        </SettingRow>

        <SettingRow label="Enable Animations" description="Enable smooth transitions and animations">
          <Toggle
            checked={props.preferences.animationsEnabled}
            onChange={(v) => props.onUpdate("animationsEnabled", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Tree View" description="Customize the evidence tree appearance">
        <SettingRow label="Density">
          <SettingsSelect
            value={props.preferences.treeDensity}
            options={[
              { value: "compact", label: "Compact" },
              { value: "comfortable", label: "Comfortable" },
              { value: "spacious", label: "Spacious" },
            ]}
            onChange={(v) => props.onUpdate("treeDensity", v as TreeDensity)}
          />
        </SettingRow>

        <SettingRow label="Icon Style">
          <SettingsSelect
            value={props.preferences.iconSet}
            options={[
              { value: "outlined", label: "Outlined" },
              { value: "solid", label: "Solid" },
              { value: "mini", label: "Mini" },
            ]}
            onChange={(v) => props.onUpdate("iconSet", v as IconSet)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Editor" description="Text and display settings">
        <SettingRow label="Font Size" description="Base font size for all UI text (12–18px)">
          <Slider
            value={props.preferences.fontSize}
            min={12}
            max={18}
            suffix="px"
            onChange={(v) => props.onUpdate("fontSize", v)}
          />
        </SettingRow>

        <SettingRow label="Show Line Numbers">
          <Toggle
            checked={props.preferences.showLineNumbers}
            onChange={(v) => props.onUpdate("showLineNumbers", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
};
