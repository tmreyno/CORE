// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For } from "solid-js";
import type { ParsedMetadata } from "../HexViewer";
import { formatOffsetLabel } from "../../utils";

interface HexRegionsSectionProps {
  metadata: ParsedMetadata | null;
  regionCount: number;
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

export const HexRegionsSection: Component<HexRegionsSectionProps> = (props) => {
  return (
    <Show when={props.regionCount > 0}>
      <div class="border-b border-border">
        <div
          class={props.categoryHeader}
          onClick={() => props.toggleCategory("_regions")}
        >
          <span class="text-2xs leading-tight text-txt-muted w-3">
            {props.isExpanded("_regions") ? "▾" : "▸"}
          </span>
          <span class="text-2xs leading-tight font-medium text-txt-tertiary flex-1">
            Hex Regions
          </span>
          <span class="text-2xs text-txt-muted">{props.regionCount}</span>
        </div>

        <Show when={props.isExpanded("_regions")}>
          <div class="py-0.5">
            <For each={props.metadata!.regions}>
              {(region) => (
                <div
                  class={`${props.rowBase} ${props.rowGrid} ${props.rowClickable}`}
                  onClick={(e) => {
                    e.stopPropagation();
                    props.handleRowClick(region.start);
                  }}
                  title={region.description}
                >
                  <span class={props.keyStyle}>{region.name}</span>
                  <span class={props.valueStyle}>{region.end - region.start} bytes</span>
                  <span class={`${props.offsetStyle} ${props.offsetClickable}`}>
                    {formatOffsetLabel(region.start)}
                  </span>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </Show>
  );
};
