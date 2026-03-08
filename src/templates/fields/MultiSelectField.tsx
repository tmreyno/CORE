// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, type Component } from "solid-js";
import { HiOutlineXMark } from "../../components/icons";
import type { FieldSchema, InlineOption } from "../types";

/** Multi-select rendered as chips/tags */
export const MultiSelectField: Component<{
  field: FieldSchema;
  value: string[];
  options: InlineOption[];
  onChange: (value: unknown) => void;
  readOnly: boolean;
}> = (props) => {
  const available = () => props.options.filter((o) => !props.value.includes(o.value));

  const add = (val: string) => {
    if (val && !props.value.includes(val)) {
      props.onChange([...props.value, val]);
    }
  };

  const remove = (val: string) => {
    props.onChange(props.value.filter((v) => v !== val));
  };

  return (
    <div class="space-y-2">
      <Show when={!props.readOnly && available().length > 0}>
        <select
          class="input-sm"
          onChange={(e) => {
            add(e.currentTarget.value);
            e.currentTarget.value = "";
          }}
        >
          <option value="">Add {props.field.label}...</option>
          <For each={available()}>
            {(opt) => <option value={opt.value}>{opt.label}</option>}
          </For>
        </select>
      </Show>
      <div class="flex flex-wrap gap-2">
        <For each={props.value}>
          {(val) => {
            const opt = props.options.find((o) => o.value === val);
            return (
              <span class="inline-flex items-center gap-1 px-2 py-0.5 bg-accent/10 text-accent rounded-full text-xs font-medium">
                {opt?.label ?? val}
                <Show when={!props.readOnly}>
                  <button
                    class="w-3.5 h-3.5 rounded-full bg-accent/20 hover:bg-accent/30 flex items-center justify-center"
                    onClick={() => remove(val)}
                  >
                    <HiOutlineXMark class="w-2.5 h-2.5" />
                  </button>
                </Show>
              </span>
            );
          }}
        </For>
      </div>
    </div>
  );
};
