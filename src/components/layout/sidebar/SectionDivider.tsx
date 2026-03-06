// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SectionDivider — Thin horizontal rule with optional label.
 */

import { type Component, Show } from "solid-js";

export const SectionDivider: Component<{ label?: string }> = (props) => (
  <div class="w-full flex items-center gap-1 my-1.5">
    <div class="flex-1 border-t border-border/40" />
    <Show when={props.label}>
      <span class="text-[9px] font-medium text-txt-muted/60 uppercase tracking-wider">
        {props.label}
      </span>
      <div class="flex-1 border-t border-border/40" />
    </Show>
  </div>
);
