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
export {
  AppearanceSettings,
  ActivityDisplaySettings,
  DefaultsSettings,
  BehaviorSettings,
  PerformanceSettings,
  SecuritySettings,
  PathsSettings,
  ReportsSettings,
  ShortcutsSettings,
} from "./tabs";
