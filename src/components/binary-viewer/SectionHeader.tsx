// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { ChevronDownIcon, ChevronRightIcon } from "../icons";

export interface SectionHeaderProps {
  title: string;
  count?: number;
  open: boolean;
  onToggle: () => void;
}

export function SectionHeader(p: SectionHeaderProps) {
  return (
    <button
      class="flex items-center gap-2 w-full text-left py-1.5 px-2 rounded hover:bg-bg-hover"
      onClick={p.onToggle}
    >
      <Show when={p.open} fallback={<ChevronRightIcon class="w-4 h-4 text-txt-muted" />}>
        <ChevronDownIcon class="w-4 h-4 text-txt-muted" />
      </Show>
      <span class="text-sm font-medium text-txt">{p.title}</span>
      <Show when={p.count !== undefined}>
        <span class="text-xs text-txt-muted">({p.count})</span>
      </Show>
    </button>
  );
}
