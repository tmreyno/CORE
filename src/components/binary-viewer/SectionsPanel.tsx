// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, type Accessor } from "solid-js";
import { formatBytes } from "../../utils";
import type { SectionInfo } from "./types";
import { formatHex } from "./helpers";
import { SectionHeader } from "./SectionHeader";

interface SectionsPanelProps {
  sections: SectionInfo[];
  open: Accessor<boolean>;
  onToggle: () => void;
}

export function SectionsPanel(props: SectionsPanelProps) {
  return (
    <Show when={props.sections.length > 0}>
      <div>
        <SectionHeader
          title="Sections"
          count={props.sections.length}
          open={props.open()}
          onToggle={props.onToggle}
        />
        <Show when={props.open()}>
          <table class="w-full text-xs mt-1">
            <thead class="bg-bg-secondary">
              <tr>
                <th class="text-left p-1.5 text-txt-muted font-medium">Name</th>
                <th class="text-left p-1.5 text-txt-muted font-medium">Virtual Addr</th>
                <th class="text-left p-1.5 text-txt-muted font-medium">Virtual Size</th>
                <th class="text-left p-1.5 text-txt-muted font-medium">Raw Size</th>
                <th class="text-left p-1.5 text-txt-muted font-medium">Flags</th>
              </tr>
            </thead>
            <tbody>
              <For each={props.sections}>
                {(sec) => (
                  <tr class="border-b border-border/30 hover:bg-bg-hover">
                    <td class="p-1.5 font-mono text-txt">{sec.name}</td>
                    <td class="p-1.5 font-mono text-txt-secondary">{formatHex(sec.virtual_address)}</td>
                    <td class="p-1.5 text-txt-secondary">{formatBytes(sec.virtual_size)}</td>
                    <td class="p-1.5 text-txt-secondary">{formatBytes(sec.raw_size)}</td>
                    <td class="p-1.5 font-mono text-txt-muted">{sec.characteristics}</td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </Show>
      </div>
    </Show>
  );
}
