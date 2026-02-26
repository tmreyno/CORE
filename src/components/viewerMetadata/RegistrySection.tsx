// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import type { RegistryMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function RegistrySection(props: { data: RegistryMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Hive Info" defaultOpen>
        <MetadataRow label="Hive Name" value={props.data.hiveName} />
        <MetadataRow label="Type" value={props.data.hiveType} />
        <MetadataRow
          label="Root Key"
          value={props.data.rootKeyName}
          mono
        />
        <MetadataRow
          label="Total Keys"
          value={String(props.data.totalKeys)}
        />
        <MetadataRow
          label="Total Values"
          value={String(props.data.totalValues)}
        />
        <Show when={props.data.lastModified}>
          <MetadataRow
            label="Modified"
            value={props.data.lastModified!}
          />
        </Show>
      </CollapsibleGroup>

      <Show when={props.data.selectedKeyPath}>
        <CollapsibleGroup title="Selected Key" defaultOpen>
          <MetadataRow
            label="Path"
            value={props.data.selectedKeyPath!}
            mono
            truncate
          />
          <Show when={props.data.selectedKeyInfo}>
            <MetadataRow
              label="Subkeys"
              value={String(props.data.selectedKeyInfo!.subkeyCount)}
            />
            <MetadataRow
              label="Values"
              value={String(props.data.selectedKeyInfo!.valueCount)}
            />
            <Show when={props.data.selectedKeyInfo!.lastModified}>
              <MetadataRow
                label="Modified"
                value={props.data.selectedKeyInfo!.lastModified!}
              />
            </Show>
            <Show when={props.data.selectedKeyInfo!.className}>
              <MetadataRow
                label="Class"
                value={props.data.selectedKeyInfo!.className!}
              />
            </Show>
          </Show>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}
