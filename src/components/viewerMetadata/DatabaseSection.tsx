// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, createMemo } from "solid-js";
import { formatBytes } from "../../utils";
import type { DatabaseMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function DatabaseSection(props: { data: DatabaseMetadataSection }) {
  const userTables = createMemo(() =>
    props.data.tables.filter((t) => !t.isSystem),
  );
  const systemTables = createMemo(() =>
    props.data.tables.filter((t) => t.isSystem),
  );

  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Database Info" defaultOpen>
        <MetadataRow
          label="Page Size"
          value={formatBytes(props.data.pageSize)}
        />
        <MetadataRow
          label="Pages"
          value={String(props.data.pageCount)}
        />
        <MetadataRow
          label="Size"
          value={formatBytes(props.data.sizeBytes)}
        />
        <MetadataRow
          label="Tables"
          value={String(props.data.tableCount)}
        />
      </CollapsibleGroup>

      <Show when={props.data.selectedTable}>
        <CollapsibleGroup title="Selected Table" defaultOpen>
          <MetadataRow
            label="Name"
            value={props.data.selectedTable!}
            mono
          />
          {(() => {
            const table = props.data.tables.find(
              (t) => t.name === props.data.selectedTable,
            );
            if (!table) return null;
            return (
              <>
                <MetadataRow
                  label="Rows"
                  value={String(table.rowCount)}
                />
                <MetadataRow
                  label="Columns"
                  value={String(table.columnCount)}
                />
              </>
            );
          })()}
        </CollapsibleGroup>
      </Show>

      <Show when={userTables().length > 0}>
        <CollapsibleGroup
          title={`Tables (${userTables().length})`}
          defaultOpen={false}
        >
          <div class="space-y-0.5">
            <For each={userTables()}>
              {(table) => (
                <div class="flex items-center justify-between text-xs py-0.5">
                  <span
                    class="text-txt truncate font-mono"
                    title={table.name}
                  >
                    {table.name}
                  </span>
                  <span class="text-txt-muted shrink-0 ml-2">
                    {table.rowCount} rows
                  </span>
                </div>
              )}
            </For>
          </div>
        </CollapsibleGroup>
      </Show>

      <Show when={systemTables().length > 0}>
        <CollapsibleGroup
          title={`System Tables (${systemTables().length})`}
          defaultOpen={false}
        >
          <div class="space-y-0.5">
            <For each={systemTables()}>
              {(table) => (
                <div class="flex items-center justify-between text-xs py-0.5">
                  <span
                    class="text-txt-muted truncate font-mono"
                    title={table.name}
                  >
                    {table.name}
                  </span>
                  <span class="text-txt-muted shrink-0 ml-2">
                    {table.rowCount}
                  </span>
                </div>
              )}
            </For>
          </div>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}
