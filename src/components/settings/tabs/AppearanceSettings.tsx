/**
 * AppearanceSettings Tab
 * Settings for theme, fonts, and visual appearance
 */

import type { SettingsUpdateProps } from "../types";
import { SettingGroup } from "../SettingGroup";
import { SettingRow } from "../SettingRow";
import { SettingsSelect } from "../SettingsSelect";
import { Toggle, Slider } from "../../ui";
import type { AccentColor, TreeDensity, IconSet, SidebarPosition } from "../../preferences";
import type { Theme } from "../../../hooks/useTheme";

export function AppearanceSettings(props: SettingsUpdateProps) {
  return (
    <>
      <SettingGroup title="Theme" description="Customize the application's visual appearance">
        <SettingRow label="Theme Mode">
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

        <SettingRow label="Accent Color">
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

        <SettingRow label="Enable Animations" description="Show UI animations and transitions">
          <Toggle
            checked={props.preferences.animationsEnabled}
            onChange={(v) => props.onUpdate("animationsEnabled", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Layout" description="Interface layout options">
        <SettingRow label="Tree Density">
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

        <SettingRow label="UI Font Size" description="Base font size for interface">
          <Slider
            value={props.preferences.fontSize}
            min={12}
            max={18}
            onChange={(v) => props.onUpdate("fontSize", v)}
          />
        </SettingRow>

        <SettingRow label="Icon Set">
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

        <SettingRow label="Show Line Numbers" description="Display line numbers in viewers">
          <Toggle
            checked={props.preferences.showLineNumbers}
            onChange={(v) => props.onUpdate("showLineNumbers", v)}
          />
        </SettingRow>

        <SettingRow label="Show Status Bar" description="Display status bar at bottom">
          <Toggle
            checked={props.preferences.showStatusBar}
            onChange={(v) => props.onUpdate("showStatusBar", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
}
