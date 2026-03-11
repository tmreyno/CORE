// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import type { BinaryMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function BinarySection(props: { data: BinaryMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Binary Info" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <Show when={props.data.architecture}>
          <MetadataRow
            label="Architecture"
            value={props.data.architecture!}
          />
        </Show>
        <Show when={props.data.entryPoint}>
          <MetadataRow
            label="Entry Point"
            value={props.data.entryPoint!}
            mono
          />
        </Show>
        <Show when={props.data.subsystem}>
          <MetadataRow
            label="Subsystem"
            value={props.data.subsystem!}
          />
        </Show>
        <Show when={props.data.compiler}>
          <MetadataRow
            label="Compiler"
            value={props.data.compiler!}
          />
        </Show>
        <Show when={props.data.compiledDate}>
          <MetadataRow
            label="Compiled"
            value={props.data.compiledDate!}
            highlight
          />
        </Show>
      </CollapsibleGroup>

      <CollapsibleGroup title="Structure" defaultOpen>
        <Show when={props.data.sectionCount != null}>
          <MetadataRow
            label="Sections"
            value={String(props.data.sectionCount)}
          />
        </Show>
        <Show when={props.data.importCount != null}>
          <MetadataRow
            label="Imports"
            value={String(props.data.importCount)}
          />
        </Show>
        <Show when={props.data.exportCount != null}>
          <MetadataRow
            label="Exports"
            value={String(props.data.exportCount)}
          />
        </Show>
        <MetadataRow
          label="Stripped"
          value={props.data.isStripped ? "Yes" : "No"}
        />
        <MetadataRow
          label="Dynamic"
          value={props.data.isDynamic ? "Yes" : "No"}
        />
      </CollapsibleGroup>

      <Show
        when={
          props.data.characteristics &&
          props.data.characteristics.length > 0
        }
      >
        <CollapsibleGroup title="Characteristics" defaultOpen={false}>
          <div class="flex flex-wrap gap-1">
            <For each={props.data.characteristics!}>
              {(char) => (
                <span class="px-1.5 py-0.5 text-2xs bg-bg-hover text-txt-secondary rounded">
                  {char}
                </span>
              )}
            </For>
          </div>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}
