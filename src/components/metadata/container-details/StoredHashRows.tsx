// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import { formatOffsetLabel } from "../../../utils";
import type { ContainerDetailsSectionProps } from "./types";

type StoredHashRowsProps = Pick<
  ContainerDetailsSectionProps,
  | "ewf"
  | "handleRowClick"
  | "rowBase"
  | "rowGrid"
  | "rowClickable"
  | "keyStyle"
  | "valueStyle"
  | "offsetStyle"
  | "offsetClickable"
>;

export function StoredHashRows(props: StoredHashRowsProps) {
  return (
    <Show when={props.ewf().stored_hashes && props.ewf().stored_hashes!.length > 0}>
      <For each={props.ewf().stored_hashes}>
        {(hash) => {
          const hasOffset = hash.offset != null && hash.size != null;
          return (
            <div
              class={`${props.rowBase} ${props.rowGrid} ${hasOffset ? props.rowClickable : ""}`}
              onClick={() => {
                if (hasOffset) {
                  props.handleRowClick(hash.offset!, hash.size!);
                }
              }}
            >
              <span class={props.keyStyle}>🔒 {hash.algorithm.toUpperCase()}</span>
              <span
                class={`${props.valueStyle} text-2xs leading-tight text-txt-muted select-all`}
              >
                {hash.hash}
              </span>
              <span class={`${props.offsetStyle} ${hasOffset ? props.offsetClickable : ""}`}>
                {hasOffset ? formatOffsetLabel(hash.offset!) : ""}
              </span>
            </div>
          );
        }}
      </For>
    </Show>
  );
}
