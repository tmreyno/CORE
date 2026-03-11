// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { HiOutlineClipboardDocument } from "../../icons";
import { FormatInfoRows } from "./FormatInfoRows";
import { CaseInfoRows } from "./CaseInfoRows";
import { StoredHashRows } from "./StoredHashRows";
import { EnhancedEwfPanel } from "./EnhancedEwfPanel";
import type { ContainerDetailsSectionProps } from "./types";

export const ContainerDetailsSectionComponent: Component<ContainerDetailsSectionProps> = (props) => {
  return (
    <div class="border-b border-border">
      <div
        class={props.categoryHeader}
        onClick={() => props.toggleCategory("_container")}
      >
        <span class="text-2xs leading-tight text-txt-muted w-3">
          {props.isExpanded("_container") ? "▾" : "▸"}
        </span>
        <span class="flex items-center gap-1 text-2xs leading-tight font-medium text-txt-tertiary flex-1">
          <HiOutlineClipboardDocument class="w-3 h-3" /> CONTAINER DETAILS
        </span>
      </div>

      <Show when={props.isExpanded("_container")}>
        <div class="py-0.5">
          <FormatInfoRows {...props} />
          <CaseInfoRows {...props} />
          <StoredHashRows {...props} />
          <EnhancedEwfPanel {...props} />
        </div>
      </Show>
    </div>
  );
};
