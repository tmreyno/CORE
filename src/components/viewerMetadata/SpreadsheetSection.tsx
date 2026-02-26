// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import type { SpreadsheetMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function SpreadsheetSection(props: {
  data: SpreadsheetMetadataSection;
}) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Spreadsheet Info" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <MetadataRow
          label="Sheets"
          value={String(props.data.sheetCount)}
        />
        <Show when={props.data.selectedSheet}>
          <MetadataRow
            label="Active Sheet"
            value={props.data.selectedSheet!}
          />
        </Show>
      </CollapsibleGroup>

      <CollapsibleGroup
        title={`Sheets (${props.data.sheets.length})`}
        defaultOpen
      >
        <For each={props.data.sheets}>
          {(sheet) => (
            <div class="flex items-center justify-between text-xs py-0.5">
              <span class="text-txt truncate" title={sheet.name}>
                {sheet.name}
              </span>
              <span class="text-txt-muted shrink-0 ml-2">
                {sheet.rowCount}×{sheet.columnCount}
              </span>
            </div>
          )}
        </For>
      </CollapsibleGroup>
    </div>
  );
}
