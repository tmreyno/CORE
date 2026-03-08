// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, type Component } from "solid-js";
import type { FieldSchema, InlineOption } from "../types";

/** Select dropdown field */
export const SelectField: Component<{
  field: FieldSchema;
  value: string;
  options: InlineOption[];
  onChange: (value: unknown) => void;
  readOnly: boolean;
}> = (props) => (
  <select
    class={`input-sm ${props.field.input_class ?? ""}`}
    value={props.value}
    onChange={(e) => props.onChange(e.currentTarget.value || undefined)}
    disabled={props.readOnly}
  >
    <Show when={props.field.placeholder}>
      <option value="">{props.field.placeholder}</option>
    </Show>
    <For each={props.options}>
      {(opt) => (
        <option
          value={opt.disabled ? "" : opt.value}
          disabled={opt.disabled}
          class={opt.disabled ? "text-txt-muted font-semibold bg-bg-secondary" : ""}
        >
          {opt.label}
        </option>
      )}
    </For>
  </select>
);
