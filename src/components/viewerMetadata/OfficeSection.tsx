// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import type { OfficeMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function OfficeSection(props: { data: OfficeMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Document Info" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <Show when={props.data.title}>
          <MetadataRow label="Title" value={props.data.title!} />
        </Show>
        <Show when={props.data.creator}>
          <MetadataRow label="Author" value={props.data.creator!} />
        </Show>
        <Show when={props.data.subject}>
          <MetadataRow label="Subject" value={props.data.subject!} />
        </Show>
        <Show when={props.data.application}>
          <MetadataRow
            label="Application"
            value={props.data.application!}
          />
        </Show>
        <Show when={props.data.created}>
          <MetadataRow label="Created" value={props.data.created!} />
        </Show>
        <Show when={props.data.modified}>
          <MetadataRow label="Modified" value={props.data.modified!} />
        </Show>
      </CollapsibleGroup>

      <CollapsibleGroup title="Statistics" defaultOpen>
        <Show when={props.data.pageCount != null}>
          <MetadataRow
            label="Pages"
            value={String(props.data.pageCount)}
          />
        </Show>
        <Show when={props.data.wordCount != null}>
          <MetadataRow
            label="Words (meta)"
            value={props.data.wordCount!.toLocaleString()}
          />
        </Show>
        <Show when={props.data.charCount != null}>
          <MetadataRow
            label="Characters"
            value={props.data.charCount!.toLocaleString()}
          />
        </Show>
        <MetadataRow
          label="Sections"
          value={String(props.data.sectionCount)}
        />
        <MetadataRow
          label="Words (extracted)"
          value={props.data.totalWords.toLocaleString()}
        />
        <MetadataRow
          label="Chars (extracted)"
          value={props.data.totalChars.toLocaleString()}
        />
        <MetadataRow
          label="Complete"
          value={props.data.extractionComplete ? "Yes" : "Partial"}
        />
      </CollapsibleGroup>

      <Show when={props.data.warnings.length > 0}>
        <CollapsibleGroup
          title={`Warnings (${props.data.warnings.length})`}
          defaultOpen={false}
        >
          <For each={props.data.warnings}>
            {(warning) => (
              <div class="text-xs text-warning py-0.5">{warning}</div>
            )}
          </For>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}
