// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * File Info tab — always-visible metadata about the selected file.
 */

import { Show } from "solid-js";
import { formatBytes, formatDateByPreference } from "../../utils";
import type { ViewerMetadata } from "../../types/viewerMetadata";
import { MetadataRow } from "./shared";

export function FileInfoTab(props: { metadata: ViewerMetadata }) {
  const info = () => props.metadata.fileInfo;
  const hasCaseDocInfo = () =>
    !!(
      info().caseNumber ||
      info().evidenceId ||
      info().documentType ||
      info().modified
    );

  return (
    <div class="p-3 space-y-3">
      {/* File name */}
      <div class="space-y-1">
        <div class="text-[10px] uppercase tracking-wider text-txt-muted font-medium">
          File
        </div>
        <div
          class="text-sm text-txt font-medium truncate"
          title={info().name}
        >
          {info().name}
        </div>
      </div>

      {/* Path */}
      <MetadataRow label="Path" value={info().path} truncate />

      {/* Size */}
      <MetadataRow label="Size" value={formatBytes(info().size)} />

      {/* Format */}
      <Show when={info().format}>
        <MetadataRow label="Format" value={info().format!} />
      </Show>

      {/* Extension */}
      <Show when={info().extension}>
        <MetadataRow label="Extension" value={info().extension!} />
      </Show>

      {/* Modified date */}
      <Show when={info().modified}>
        <MetadataRow
          label="Modified"
          value={formatDateByPreference(info().modified, false)}
        />
      </Show>

      {/* Case Document attributes */}
      <Show when={hasCaseDocInfo()}>
        <div class="pt-2 border-t border-border/50">
          <div class="text-[10px] uppercase tracking-wider text-txt-muted font-medium mb-2">
            Case Info
          </div>
          <Show when={info().caseNumber}>
            <MetadataRow
              label="Case #"
              value={info().caseNumber!}
              highlight
            />
          </Show>
          <Show when={info().evidenceId}>
            <MetadataRow
              label="Evidence ID"
              value={info().evidenceId!}
              highlight
            />
          </Show>
          <Show when={info().documentType}>
            <MetadataRow label="Doc Type" value={info().documentType!} />
          </Show>
        </div>
      </Show>

      {/* Container info */}
      <Show when={info().containerPath}>
        <div class="pt-2 border-t border-border/50">
          <div class="text-[10px] uppercase tracking-wider text-txt-muted font-medium mb-2">
            Container
          </div>
          <MetadataRow
            label="Path"
            value={info().containerPath!}
            truncate
          />
          <Show when={info().containerType}>
            <MetadataRow label="Type" value={info().containerType!} />
          </Show>
        </div>
      </Show>

      {/* Source type badges */}
      <div class="flex flex-wrap gap-1 pt-1">
        <Show when={info().isDiskFile}>
          <span class="badge badge-neutral">Disk File</span>
        </Show>
        <Show when={info().isVfsEntry}>
          <span class="badge badge-info">VFS Entry</span>
        </Show>
        <Show when={info().isArchiveEntry}>
          <span class="badge badge-warning">Archive Entry</span>
        </Show>
      </div>

      {/* Viewer type */}
      <div class="pt-2 border-t border-border/50">
        <MetadataRow label="Viewer" value={props.metadata.viewerType} />
      </div>
    </div>
  );
}
