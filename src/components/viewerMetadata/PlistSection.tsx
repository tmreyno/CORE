// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import type { PlistMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function PlistSection(props: { data: PlistMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Property List" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <MetadataRow label="Root Type" value={props.data.rootType} />
        <MetadataRow
          label="Entries"
          value={String(props.data.entryCount)}
        />
      </CollapsibleGroup>

      <Show
        when={
          props.data.notableKeys && props.data.notableKeys.length > 0
        }
      >
        <CollapsibleGroup title="Notable Keys" defaultOpen>
          <For each={props.data.notableKeys!}>
            {(entry) => (
              <MetadataRow label={entry.key} value={entry.value} mono />
            )}
          </For>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}
