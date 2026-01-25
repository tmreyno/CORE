/**
 * Settings Panel Types
 * Type definitions for the modular settings panel components
 */

import type { JSX } from "solid-js";
import type { AppPreferences } from "../preferences";

/**
 * Settings tab identifiers
 */
export type SettingsTab = 
  | "appearance"
  | "defaults"
  | "behavior"
  | "performance"
  | "security"
  | "paths"
  | "reports"
  | "shortcuts";

/**
 * Tab configuration for rendering
 */
export interface TabConfig {
  id: SettingsTab;
  label: string;
  icon: string;
}

/**
 * Base props for settings components that update preferences
 */
export interface SettingsUpdateProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

/**
 * Props for the main SettingsPanel component
 */
export interface SettingsPanelProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
  onUpdateShortcut: (action: string, shortcut: string) => void;
  onResetAll: () => void;
  onClose: () => void;
}

/**
 * Props for shortcuts settings tab
 */
export interface ShortcutsSettingsProps extends SettingsUpdateProps {
  onUpdateShortcut: (action: string, shortcut: string) => void;
  editingShortcut: () => string | null;
  setEditingShortcut: (action: string | null) => void;
}

/**
 * Props for SettingGroup component
 */
export interface SettingGroupProps {
  title: string;
  description?: string;
  children: JSX.Element;
}

/**
 * Props for SettingRow component
 */
export interface SettingRowProps {
  label: string;
  description?: string;
  children: JSX.Element;
}

/**
 * Props for SettingsSelect component
 */
export interface SettingsSelectProps {
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}
