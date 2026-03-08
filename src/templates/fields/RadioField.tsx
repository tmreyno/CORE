// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, type Component } from "solid-js";
import type { FieldSchema, InlineOption } from "../types";

/** Radio button group field */
export const RadioField: Component<{
  field: FieldSchema;
  value: string;
  options: InlineOption[];
  onChange: (value: unknown) => void;
  readOnly: boolean;
}> = (props) => (
  <div class="flex flex-wrap gap-3">
    <For each={props.options}>
      {(opt) => (
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name={props.field.id}
            value={opt.value}
            checked={props.value === opt.value}
            onChange={() => props.onChange(opt.value)}
            disabled={props.readOnly}
            class="border-border"
          />
          <span class="text-sm">{opt.label}</span>
        </label>
      )}
    </For>
  </div>
);
