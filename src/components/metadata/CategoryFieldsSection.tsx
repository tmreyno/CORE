// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For } from "solid-js";
import type { MetadataField } from "../HexViewer";
import { formatOffsetLabel } from "../../utils";

interface CategoryFieldsSectionProps {
  categoryEntries: [string, MetadataField[]][];
  isExpanded: (category: string) => boolean;
  toggleCategory: (category: string) => void;
  handleRowClick: (offset: number | undefined | null, size?: number) => void;
  categoryHeader: string;
  rowBase: string;
  rowGrid: string;
  rowClickable: string;
  keyStyle: string;
  valueStyle: string;
  offsetStyle: string;
  offsetClickable: string;
}

export const CategoryFieldsSection: Component<CategoryFieldsSectionProps> = (props) => {
  return (
    <div class="flex-1">
      <For each={props.categoryEntries}>
        {([category, fields]) => (
          <div class="border-b border-border last:border-b-0">
            <div
              class={props.categoryHeader}
              onClick={() => props.toggleCategory(category)}
            >
              <span class="text-[10px] leading-tight text-txt-muted w-3">
                {props.isExpanded(category) ? "▾" : "▸"}
              </span>
              <span class="text-[10px] leading-tight font-medium text-txt-tertiary flex-1">
                {category}
              </span>
              <span class="text-[9px] text-txt-muted">{fields.length}</span>
            </div>

            <Show when={props.isExpanded(category)}>
              <div class="py-0.5">
                <For each={fields}>
                  {(field) => {
                    const hasOffset =
                      field.source_offset !== undefined && field.source_offset !== null;
                    return (
                      <div
                        class={`${props.rowBase} ${props.rowGrid} ${hasOffset ? props.rowClickable : ""}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          props.handleRowClick(field.source_offset);
                        }}
                        title={
                          hasOffset
                            ? `Click to view hex at ${formatOffsetLabel(field.source_offset)}`
                            : field.value
                        }
                      >
                        <span class={props.keyStyle}>{field.key}</span>
                        <span class={props.valueStyle}>{field.value}</span>
                        <span
                          class={`${props.offsetStyle} ${hasOffset ? props.offsetClickable : ""}`}
                        >
                          {formatOffsetLabel(field.source_offset)}
                        </span>
                      </div>
                    );
                  }}
                </For>
              </div>
            </Show>
          </div>
        )}
      </For>
    </div>
  );
};
