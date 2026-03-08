// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, For, Show, type Component } from "solid-js";
import { HiOutlineXMark } from "../../components/icons";
import type { FieldSchema } from "../types";

/** Comma-separated list rendered as tags with an input */
export const CommaListField: Component<{
  field: FieldSchema;
  value: string[];
  onChange: (value: unknown) => void;
  readOnly: boolean;
}> = (props) => {
  const [input, setInput] = createSignal("");

  const addValue = () => {
    const trimmed = input().trim();
    if (trimmed && !props.value.includes(trimmed)) {
      props.onChange([...props.value, trimmed]);
      setInput("");
    }
  };

  const removeValue = (val: string) => {
    props.onChange(props.value.filter((v) => v !== val));
  };

  return (
    <div class="space-y-2">
      <Show when={!props.readOnly}>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-sm flex-1"
            value={input()}
            onInput={(e) => setInput(e.currentTarget.value)}
            onKeyDown={(e) => e.key === "Enter" && (e.preventDefault(), addValue())}
            placeholder={props.field.placeholder}
          />
          <button
            class="px-2.5 py-1.5 rounded-md text-xs font-medium bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
            onClick={addValue}
          >
            + Add
          </button>
        </div>
      </Show>
      <div class="flex flex-wrap gap-2">
        <For each={props.value}>
          {(val) => (
            <span class="inline-flex items-center gap-1 px-2 py-0.5 bg-accent/10 text-accent rounded-full text-xs font-medium">
              {val}
              <Show when={!props.readOnly}>
                <button
                  class="w-3.5 h-3.5 rounded-full bg-accent/20 hover:bg-accent/30 flex items-center justify-center"
                  onClick={() => removeValue(val)}
                >
                  <HiOutlineXMark class="w-2.5 h-2.5" />
                </button>
              </Show>
            </span>
          )}
        </For>
        <Show when={props.value.length === 0}>
          <span class="text-sm text-txt/40 italic">None added</span>
        </Show>
      </div>
    </div>
  );
};
