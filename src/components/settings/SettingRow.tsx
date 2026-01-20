/**
 * SettingRow Component
 * A single row in the settings panel with label, description, and control
 */

import type { SettingRowProps } from "./types";

export function SettingRow(props: SettingRowProps) {
  return (
    <div class="flex items-center justify-between py-2">
      <div class="flex flex-col">
        <span class="text-sm text-txt">{props.label}</span>
        {props.description && (
          <span class="text-xs text-txt-muted mt-0.5">{props.description}</span>
        )}
      </div>
      <div class="flex items-center gap-2">
        {props.children}
      </div>
    </div>
  );
}
