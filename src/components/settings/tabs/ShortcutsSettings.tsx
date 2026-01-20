/**
 * ShortcutsSettings Tab
 * Settings for keyboard shortcut customization
 */

import { For } from "solid-js";
import type { ShortcutsSettingsProps } from "../types";
import { SettingGroup } from "../SettingGroup";
import { SHORTCUT_LABELS, formatShortcut } from "../constants";

export function ShortcutsSettings(props: ShortcutsSettingsProps) {
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
                <span class="text-sm text-txt">{SHORTCUT_LABELS[action] ?? action}</span>
                <button
                  class={`px-2 py-1 text-xs rounded border transition-colors ${
                    props.editingShortcut() === action
                      ? "bg-accent border-accent text-white"
                      : "border-border bg-bg-panel text-txt-secondary hover:bg-bg-hover"
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
}
