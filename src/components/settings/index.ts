/**
 * Settings Module
 * Barrel export for all settings components, types, and utilities
 */

// Types
export type {
  SettingsTab,
  TabConfig,
  SettingsUpdateProps,
  SettingsPanelProps,
  ShortcutsSettingsProps,
  SettingGroupProps,
  SettingRowProps,
  SettingsSelectProps,
} from "./types";

// Constants
export { SETTINGS_TABS, SHORTCUT_LABELS, formatShortcut } from "./constants";

// Helper Components
export { SettingGroup } from "./SettingGroup";
export { SettingRow } from "./SettingRow";
export { SettingsSelect } from "./SettingsSelect";

// Tab Components
export { ActivityDisplaySettings } from "./ActivityDisplaySettings";
export { AppearanceSettings } from "./AppearanceTab";
export { DefaultsSettings } from "./DefaultsTab";
export { BehaviorSettings } from "./BehaviorTab";
export { PerformanceSettings } from "./PerformanceTab";
export { SecuritySettings } from "./SecurityTab";
export { PathsSettings } from "./PathsTab";
export { ReportsSettings } from "./ReportsTab";
export { ShortcutsSettings } from "./ShortcutsTab";
