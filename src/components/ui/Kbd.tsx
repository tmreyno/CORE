// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
// Kbd.tsx - Centralized Keyboard Shortcut Display Components
// =============================================================================
// Single source of truth for all keyboard shortcut characters.
// Import these components throughout the app for consistent styling.
// =============================================================================

import { Component, JSX, For, Show } from "solid-js";
import { isMac } from "../../utils/platform";

// =============================================================================
// Modifier Key Symbols - Platform-aware
// =============================================================================

export const ModifierKeys = {
  // Primary modifiers - platform-aware
  command: isMac ? "⌘" : "Ctrl",
  cmd: isMac ? "⌘" : "Ctrl",
  shift: isMac ? "⇧" : "Shift",
  option: isMac ? "⌥" : "Alt",
  alt: isMac ? "⌥" : "Alt",
  control: isMac ? "⌃" : "Ctrl",
  ctrl: isMac ? "⌃" : "Ctrl",
  
  // Cross-platform primary modifier (Cmd on Mac, Ctrl on Windows)
  mod: isMac ? "⌘" : "Ctrl",
  
  // Arrow keys - universal
  up: "↑",
  down: "↓",
  left: "←",
  right: "→",
  
  // Special keys - platform-aware
  enter: isMac ? "↵" : "Enter",
  return: isMac ? "↵" : "Enter",
  tab: isMac ? "⇥" : "Tab",
  escape: "Esc",
  esc: "Esc",
  space: isMac ? "␣" : "Space",
  backspace: isMac ? "⌫" : "Backspace",
  delete: isMac ? "⌦" : "Del",
  
  // Function keys (universal)
  f1: "F1",
  f2: "F2",
  f3: "F3",
  f4: "F4",
  f5: "F5",
  f6: "F6",
  f7: "F7",
  f8: "F8",
  f9: "F9",
  f10: "F10",
  f11: "F11",
  f12: "F12",
} as const;

// =============================================================================
// Types
// =============================================================================

export type ModifierKeyName = keyof typeof ModifierKeys;

export interface KbdProps {
  /** The key or keys to display. Can be a string like "⌘K" or array ["⌘", "K"] */
  keys: string | string[];
  /** Size variant */
  size?: "sm" | "md" | "lg";
  /** Additional CSS classes */
  class?: string;
  /** Whether to show a muted/secondary color */
  muted?: boolean;
}

export interface ShortcutProps {
  /** Modifier keys like "cmd", "shift", "alt" */
  modifiers?: ModifierKeyName[];
  /** The main key (letter, number, or special key name) */
  keyName: string;
  /** Size variant */
  size?: "sm" | "md" | "lg";
  /** Additional CSS classes */
  class?: string;
  /** Whether to show a muted/secondary color */
  muted?: boolean;
  /** Label to show after the shortcut */
  label?: string;
}

// =============================================================================
// Size Configuration
// =============================================================================

const sizeClasses = {
  sm: "text-xs",
  md: "text-sm", 
  lg: "text-base",
};

const gapClasses = {
  sm: "gap-1",
  md: "gap-1",
  lg: "gap-1.5",
};

// =============================================================================
// Kbd Component - Single key display
// =============================================================================

/**
 * Displays a single keyboard key or sequence of keys
 * Uses system-ui font which renders ⌘⇧⌥⌃ at same size as letters on macOS
 * 
 * @example
 * <Kbd keys="⌘" />
 * <Kbd keys="⌘K" />
 * <Kbd keys={["⌘", "Shift", "N"]} />
 */
export const Kbd: Component<KbdProps> = (props) => {
  const size = () => props.size || "md";
  const keys = () => typeof props.keys === "string" ? [props.keys] : props.keys;
  
  return (
    <span 
      class={`inline-flex items-center font-medium ${gapClasses[size()]} ${sizeClasses[size()]} ${props.muted ? "text-txt-muted" : "text-txt-secondary"} ${props.class || ""}`}
      style={{ "font-family": "system-ui, -apple-system, BlinkMacSystemFont, sans-serif" }}
    >
      <For each={keys()}>
        {(key) => <span>{key}</span>}
      </For>
    </span>
  );
};

// =============================================================================
// Shortcut Component - Structured shortcut display
// =============================================================================

/**
 * Displays a keyboard shortcut with modifiers and a key
 * Uses system-ui font for consistent symbol sizing
 * 
 * @example
 * <Shortcut modifiers={["cmd"]} keyName="K" />
 * <Shortcut modifiers={["cmd", "shift"]} keyName="N" label="New Project" />
 */
export const Shortcut: Component<ShortcutProps> = (props) => {
  const size = () => props.size || "md";
  
  const resolveKey = (key: string): string => {
    const lower = key.toLowerCase();
    if (lower in ModifierKeys) {
      return ModifierKeys[lower as ModifierKeyName];
    }
    return key.toUpperCase();
  };
  
  const modifierSymbols = () => (props.modifiers || []).map(m => ModifierKeys[m]);
  const mainKey = () => resolveKey(props.keyName);
  
  return (
    <span class={`inline-flex items-center ${gapClasses[size()]} ${props.class || ""}`}>
      <span 
        class={`inline-flex items-center font-medium ${gapClasses[size()]} ${sizeClasses[size()]} ${props.muted ? "text-txt-muted" : "text-txt-secondary"}`}
        style={{ "font-family": "system-ui, -apple-system, BlinkMacSystemFont, sans-serif" }}
      >
        <For each={modifierSymbols()}>
          {(symbol) => <span>{symbol}</span>}
        </For>
        <span>{mainKey()}</span>
      </span>
      <Show when={props.label}>
        <span class={`${sizeClasses[size()]} text-txt-muted ml-1`}>{props.label}</span>
      </Show>
    </span>
  );
};

// =============================================================================
// ShortcutHint Component - For inline hints like "Press ⌘K anywhere"
// =============================================================================

export interface ShortcutHintProps {
  /** Text before the shortcut */
  prefix?: string;
  /** The shortcut keys to display */
  keys: string;
  /** Text after the shortcut */
  suffix?: string;
  /** Size variant */
  size?: "sm" | "md" | "lg";
  /** Additional CSS classes */
  class?: string;
}

/**
 * Displays an inline hint with a keyboard shortcut
 * 
 * @example
 * <ShortcutHint prefix="Press" keys="⌘K" suffix="anywhere" />
 * <ShortcutHint prefix="Press" keys="?" suffix="for help" />
 */
export const ShortcutHint: Component<ShortcutHintProps> = (props) => {
  const size = () => props.size || "md";
  
  return (
    <span class={`inline-flex items-center gap-1.5 ${sizeClasses[size()]} text-txt-muted ${props.class || ""}`}>
      <Show when={props.prefix}>
        <span>{props.prefix}</span>
      </Show>
      <Kbd keys={props.keys} size={size()} />
      <Show when={props.suffix}>
        <span>{props.suffix}</span>
      </Show>
    </span>
  );
};

// =============================================================================
// Common Shortcuts - Pre-built shortcuts for consistency
// =============================================================================

export const CommonShortcuts = {
  // Navigation
  newProject: { modifiers: ["cmd", "shift"] as ModifierKeyName[], keyName: "N" },
  open: { modifiers: ["cmd"] as ModifierKeyName[], keyName: "O" },
  commandPalette: { modifiers: ["cmd"] as ModifierKeyName[], keyName: "K" },
  search: { modifiers: ["cmd"] as ModifierKeyName[], keyName: "F" },
  save: { modifiers: ["cmd"] as ModifierKeyName[], keyName: "S" },
  close: { modifiers: ["cmd"] as ModifierKeyName[], keyName: "W" },
  
  // UI
  help: { modifiers: [] as ModifierKeyName[], keyName: "?" },
  escape: { modifiers: [] as ModifierKeyName[], keyName: "esc" },
  enter: { modifiers: [] as ModifierKeyName[], keyName: "enter" },
  
  // Navigation keys
  up: { modifiers: [] as ModifierKeyName[], keyName: "up" },
  down: { modifiers: [] as ModifierKeyName[], keyName: "down" },
  left: { modifiers: [] as ModifierKeyName[], keyName: "left" },
  right: { modifiers: [] as ModifierKeyName[], keyName: "right" },
} as const;

// =============================================================================
// Helper to render common shortcuts
// =============================================================================

export const renderShortcut = (
  shortcut: { modifiers: readonly ModifierKeyName[]; keyName: string },
  options?: { size?: "sm" | "md" | "lg"; muted?: boolean; label?: string }
): JSX.Element => {
  return (
    <Shortcut 
      modifiers={[...shortcut.modifiers]} 
      keyName={shortcut.keyName}
      size={options?.size}
      muted={options?.muted}
      label={options?.label}
    />
  );
};

// =============================================================================
// Exports
// =============================================================================

export default Kbd;
