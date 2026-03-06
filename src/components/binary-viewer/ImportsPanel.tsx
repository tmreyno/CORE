// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, type Accessor } from "solid-js";
import { SearchIcon } from "../icons";
import type { ImportInfo } from "./types";
import { SectionHeader } from "./SectionHeader";

interface ImportsPanelProps {
  imports: ImportInfo[];
  totalFunctions: Accessor<number>;
  open: Accessor<boolean>;
  onToggle: () => void;
  filteredImports: Accessor<ImportInfo[]>;
  importFilter: Accessor<string>;
  setImportFilter: (v: string) => void;
}

export function ImportsPanel(props: ImportsPanelProps) {
  return (
    <Show when={props.imports.length > 0}>
      <div>
        <SectionHeader
          title="Imports"
          count={props.totalFunctions()}
          open={props.open()}
          onToggle={props.onToggle}
        />
        <Show when={props.open()}>
          {/* Import filter */}
          <div class="relative mt-1 mb-2">
            <SearchIcon class="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-txt-muted" />
            <input
              type="text"
              class="input-xs pl-7 w-full"
              placeholder="Filter imports..."
              value={props.importFilter()}
              onInput={(e) => props.setImportFilter(e.currentTarget.value)}
            />
          </div>
          <div class="space-y-1 max-h-64 overflow-y-auto">
            <For each={props.filteredImports()}>
              {(imp) => (
                <div class="text-xs p-1.5 rounded bg-bg-secondary">
                  <div class="font-medium text-accent">{imp.library}</div>
                  <Show when={imp.functions.length > 0}>
                    <div class="mt-0.5 pl-3 text-txt-muted space-y-0.5">
                      <For each={imp.functions.slice(0, 20)}>
                        {(fn) => <div class="font-mono">{fn}</div>}
                      </For>
                      <Show when={imp.functions.length > 20}>
                        <div class="text-txt-muted italic">... and {imp.functions.length - 20} more</div>
                      </Show>
                    </div>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </Show>
  );
}
