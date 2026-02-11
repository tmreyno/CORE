// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For } from "solid-js";
import { SettingGroup } from "../settings";
import type { AppPreferences } from "../preferences";

interface ShortcutsSettingsProps {
  preferences: AppPreferences;
  onUpdateShortcut: (action: string, shortcut: string) => void;
  editingShortcut: () => string | null;
  setEditingShortcut: (action: string | null) => void;
}

export const ShortcutsSettings: Component<ShortcutsSettingsProps> = (props) => {
  const shortcutLabels: Record<string, string> = {
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

  const formatShortcut = (shortcut: string) => {
    return shortcut
      .replace("Meta", "⌘")
      .replace("Shift", "⇧")
      .replace("Alt", "⌥")
      .replace("Control", "⌃")
      .replace(/\+/g, " ");
  };

  const handleKeyCapture = (action: string, e: KeyboardEvent) => {
    e.preventDefault();
    
    const parts: string[] = [];
    if (e.metaKey) parts.push("Meta");
    if (e.ctrlKey) parts.push("Control");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    
    // Add the actual key (if not a modifier)
    if (!["Meta", "Control", "Alt", "Shift"].includes(e.key)) {
      parts.push(e.key.length === 1 ? e.key.toLowerCase() : e.key);
    }
    
    if (parts.length > 0 && !["Meta", "Control", "Alt", "Shift"].includes(parts[parts.length - 1])) {
      props.onUpdateShortcut(action, parts.join("+"));
      props.setEditingShortcut(null);
    }
  };

  return (
    <>
      <SettingGroup title="Keyboard Shortcuts" description="Click on a shortcut to change it">
        <div class="space-y-2">
          <For each={Object.entries(props.preferences.shortcuts)}>
            {([action, shortcut]) => (
              <div class="flex items-center justify-between py-1.5">
                <span class="text-sm text-txt">{shortcutLabels[action] ?? action}</span>
                <button
                  class={`px-2 py-1 text-xs rounded border transition-colors ${
                    props.editingShortcut() === action
                      ? "border-accent bg-accent/10 text-accent"
                      : "border-border hover:border-accent text-txt-secondary"
                  }`}
                  onClick={() => props.setEditingShortcut(action)}
                  onKeyDown={(e) => {
                    if (props.editingShortcut() === action) {
                      handleKeyCapture(action, e);
                    }
                  }}
                >
                  {props.editingShortcut() === action ? "Press keys..." : formatShortcut(shortcut)}
                </button>
              </div>
            )}
          </For>
        </div>
      </SettingGroup>

      <p class="text-xs text-txt-muted mt-1">
        Tip: Click a shortcut button then press your desired key combination.
        Press Escape to cancel.
      </p>
    </>
  );
};
