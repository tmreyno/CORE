/**
 * Settings Panel Constants
 * Tab configurations and shared constants for settings components
 */

import type { TabConfig } from "./types";

/**
 * Settings tab configuration with icons and labels
 */
export const SETTINGS_TABS: TabConfig[] = [
  { id: "appearance", label: "Appearance", icon: "🎨" },
  { id: "defaults", label: "Defaults", icon: "📋" },
  { id: "behavior", label: "Behavior", icon: "⚙️" },
  { id: "performance", label: "Performance", icon: "🚀" },
  { id: "security", label: "Security", icon: "🔒" },
  { id: "paths", label: "Paths", icon: "📁" },
  { id: "reports", label: "Reports", icon: "📊" },
  { id: "shortcuts", label: "Shortcuts", icon: "⌨️" },
];

/**
 * Shortcut action display labels
 */
export const SHORTCUT_LABELS: Record<string, string> = {
  openCommandPalette: "Open Command Palette",
  showShortcuts: "Show Keyboard Shortcuts",
  openFile: "Open File",
  closeModal: "Close Modal/Dialog",
  save: "Save Project",
  undo: "Undo",
  redo: "Redo",
  search: "Search",
  settings: "Open Settings",
};

/**
 * Format a keyboard shortcut for display
 */
export function formatShortcut(shortcut: string): string {
  return shortcut
    .replace("Meta", "⌘")
    .replace("Shift", "⇧")
    .replace("Alt", "⌥")
    .replace("Control", "⌃")
    .replace(/\+/g, " ");
}
