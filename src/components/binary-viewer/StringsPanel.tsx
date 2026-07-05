// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, type Accessor } from "solid-js";
import { SearchIcon } from "../icons";
import { SectionHeader } from "./SectionHeader";

interface StringsPanelProps {
  strings: string[];
  open: Accessor<boolean>;
  onToggle: () => void;
  filteredStrings: Accessor<string[]>;
  stringFilter: Accessor<string>;
  setStringFilter: (v: string) => void;
}

export function StringsPanel(props: StringsPanelProps) {
  return (
    <Show when={props.strings.length > 0}>
      <div>
        <SectionHeader
          title="Strings"
          count={props.strings.length}
          open={props.open()}
          onToggle={props.onToggle}
        />
        <Show when={props.open()}>
          <div class="relative mt-1 mb-2">
            <SearchIcon class="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-txt-muted" />
            <input
              type="text"
              class="input-xs pl-7 w-full"
              placeholder="Filter strings..."
              value={props.stringFilter()}
              onInput={(e) => props.setStringFilter(e.currentTarget.value)}
            />
          </div>
          <div class="space-y-1 max-h-80 overflow-y-auto">
            <For each={props.filteredStrings().slice(0, 300)}>
              {(value) => (
                <div class="text-xs font-mono p-1.5 rounded bg-bg-secondary text-txt break-all">
                  {value}
                </div>
              )}
            </For>
          </div>
          <Show when={props.filteredStrings().length > 300}>
            <div class="text-xs text-txt-muted p-2 text-center">
              Showing 300 of {props.filteredStrings().length} matching strings
            </div>
          </Show>
        </Show>
      </div>
    </Show>
  );
}
