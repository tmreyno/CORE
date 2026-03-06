// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { HiOutlineArchiveBox } from "../../icons";
import { formatOffsetLabel } from "../../../utils";
import type { RowStyles } from "./types";

interface DetailRowProps extends RowStyles {
  label: string;
  value: string | number | null | undefined;
  offset?: number | null;
  offsetSize?: number;
  clickable?: boolean;
  onRowClick?: (offset: number, size?: number) => void;
}

/** A single key-value-offset row in the container detail grid */
export function DetailRow(props: DetailRowProps) {
  const isClickable = () => props.clickable && props.offset != null;

  return (
    <div
      class={`${props.rowBase} ${props.rowGrid} ${isClickable() ? props.rowClickable : ""}`}
      onClick={() => {
        if (isClickable() && props.onRowClick) {
          props.onRowClick(props.offset!, props.offsetSize);
        }
      }}
      title={isClickable() ? `Click to view at ${formatOffsetLabel(props.offset!)}` : undefined}
    >
      <span class={props.keyStyle}>{props.label}</span>
      <span class={props.valueStyle}>{props.value ?? ""}</span>
      <span class={`${props.offsetStyle} ${isClickable() ? props.offsetClickable : ""}`}>
        {props.offset != null ? formatOffsetLabel(props.offset) : ""}
      </span>
    </div>
  );
}

interface HeaderFieldRowProps extends RowStyles {
  label: string;
  value: string | null | undefined;
  headerDataStart: number;
  onRowClick: (offset: number) => void;
}

/** A case-info row linking to compressed header data (with archive icon) */
export function HeaderFieldRow(props: HeaderFieldRowProps) {
  return (
    <Show when={props.value}>
      <div
        class={`${props.rowBase} ${props.rowGrid} ${props.rowClickable}`}
        onClick={() => props.onRowClick(props.headerDataStart)}
        title="Click to view compressed header data (zlib stream)"
      >
        <span class={props.keyStyle}>{props.label}</span>
        <span class={props.valueStyle}>{props.value}</span>
        <span class={`${props.offsetStyle} ${props.offsetClickable}`}>
          {formatOffsetLabel(props.headerDataStart)}{" "}
          <HiOutlineArchiveBox class="w-2 h-2 inline" />
        </span>
      </div>
    </Show>
  );
}
