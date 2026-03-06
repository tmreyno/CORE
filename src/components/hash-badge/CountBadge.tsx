// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";

/** Small count badge overlay for hash totals */
export function CountBadge(props: { count: number; colorClass: string }) {
  return (
    <Show when={props.count > 0}>
      <span class={`count-badge ${props.colorClass}`}>{props.count}</span>
    </Show>
  );
}
