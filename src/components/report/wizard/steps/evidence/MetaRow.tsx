// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * MetaRow — a labeled metadata row for container detail panels.
 */

import { Show } from "solid-js";

export function MetaRow(props: { label: string; value: string | number | undefined | null }) {
  return (
    <Show when={props.value != null && props.value !== ""}>
      <div class="flex items-baseline gap-2 text-xs">
        <span class="text-txt/40 min-w-[100px] shrink-0">{props.label}</span>
        <span class="text-txt/80 font-mono break-all">{String(props.value)}</span>
      </div>
    </Show>
  );
}
