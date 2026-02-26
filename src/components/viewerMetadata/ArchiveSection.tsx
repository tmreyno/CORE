// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { formatBytes } from "../../utils";
import type { ArchiveMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function ArchiveSection(props: { data: ArchiveMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Archive Info" defaultOpen>
        <MetadataRow
          label="Format"
          value={props.data.archiveFormat}
        />
        <MetadataRow
          label="Total Files"
          value={props.data.totalFiles.toLocaleString()}
        />
        <MetadataRow
          label="Total Folders"
          value={props.data.totalFolders.toLocaleString()}
        />
        <MetadataRow
          label="Total Entries"
          value={props.data.totalEntries.toLocaleString()}
        />
        <Show when={props.data.archiveSize > 0}>
          <MetadataRow
            label="Archive Size"
            value={formatBytes(props.data.archiveSize)}
          />
        </Show>
        <MetadataRow
          label="Encrypted"
          value={props.data.encrypted ? "Yes" : "No"}
          highlight={props.data.encrypted}
        />
      </CollapsibleGroup>

      <Show when={props.data.entryPath}>
        <CollapsibleGroup title="Entry Details" defaultOpen>
          <MetadataRow
            label="Path"
            value={props.data.entryPath!}
            mono
            truncate
          />
          <Show when={props.data.entryCompressedSize != null}>
            <MetadataRow
              label="Compressed"
              value={formatBytes(props.data.entryCompressedSize!)}
            />
          </Show>
          <Show
            when={
              props.data.entryCrc32 != null &&
              props.data.entryCrc32 !== 0
            }
          >
            <MetadataRow
              label="CRC-32"
              value={`0x${props.data.entryCrc32!.toString(16).toUpperCase().padStart(8, "0")}`}
              mono
            />
          </Show>
          <Show when={props.data.entryModified}>
            <MetadataRow
              label="Modified"
              value={props.data.entryModified!}
            />
          </Show>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}
