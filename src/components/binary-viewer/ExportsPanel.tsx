// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, type Accessor } from "solid-js";
import type { ExportInfo } from "./types";
import { formatHex } from "./helpers";
import { SectionHeader } from "./SectionHeader";

interface ExportsPanelProps {
  exports: ExportInfo[];
  open: Accessor<boolean>;
  onToggle: () => void;
}

export function ExportsPanel(props: ExportsPanelProps) {
  return (
    <Show when={props.exports.length > 0}>
      <div>
        <SectionHeader
          title="Exports"
          count={props.exports.length}
          open={props.open()}
          onToggle={props.onToggle}
        />
        <Show when={props.open()}>
          <table class="w-full text-xs mt-1">
            <thead class="bg-bg-secondary">
              <tr>
                <th class="text-left p-1.5 text-txt-muted font-medium">Name</th>
                <th class="text-left p-1.5 text-txt-muted font-medium">Address</th>
              </tr>
            </thead>
            <tbody>
              <For each={props.exports.slice(0, 100)}>
                {(exp) => (
                  <tr class="border-b border-border/30 hover:bg-bg-hover">
                    <td class="p-1.5 font-mono text-txt">{exp.name}</td>
                    <td class="p-1.5 font-mono text-txt-secondary">{formatHex(exp.address)}</td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
          <Show when={props.exports.length > 100}>
            <div class="text-xs text-txt-muted p-2 text-center">
              Showing 100 of {props.exports.length} exports
            </div>
          </Show>
        </Show>
      </div>
    </Show>
  );
}
