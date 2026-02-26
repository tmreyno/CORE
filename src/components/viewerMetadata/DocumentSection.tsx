// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import type { DocumentMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function DocumentSection(props: { data: DocumentMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Document Info" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <Show when={props.data.title}>
          <MetadataRow label="Title" value={props.data.title!} />
        </Show>
        <Show when={props.data.author}>
          <MetadataRow label="Author" value={props.data.author!} />
        </Show>
        <Show when={props.data.creator}>
          <MetadataRow label="Creator" value={props.data.creator!} />
        </Show>
        <Show when={props.data.producer}>
          <MetadataRow label="Producer" value={props.data.producer!} />
        </Show>
        <Show when={props.data.pageCount != null}>
          <MetadataRow
            label="Pages"
            value={String(props.data.pageCount)}
          />
        </Show>
        <Show when={props.data.wordCount != null}>
          <MetadataRow
            label="Words"
            value={String(props.data.wordCount)}
          />
        </Show>
        <Show when={props.data.encrypted != null}>
          <MetadataRow
            label="Encrypted"
            value={props.data.encrypted ? "Yes" : "No"}
          />
        </Show>
      </CollapsibleGroup>

      <Show
        when={props.data.creationDate || props.data.modificationDate}
      >
        <CollapsibleGroup title="Dates" defaultOpen>
          <Show when={props.data.creationDate}>
            <MetadataRow
              label="Created"
              value={props.data.creationDate!}
              highlight
            />
          </Show>
          <Show when={props.data.modificationDate}>
            <MetadataRow
              label="Modified"
              value={props.data.modificationDate!}
              highlight
            />
          </Show>
        </CollapsibleGroup>
      </Show>

      <Show
        when={props.data.keywords && props.data.keywords.length > 0}
      >
        <CollapsibleGroup title="Keywords" defaultOpen={false}>
          <div class="flex flex-wrap gap-1">
            <For each={props.data.keywords!}>
              {(kw) => (
                <span class="px-1.5 py-0.5 text-[10px] bg-bg-hover text-txt-secondary rounded">
                  {kw}
                </span>
              )}
            </For>
          </div>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}
