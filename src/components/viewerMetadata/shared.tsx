// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared UI primitives for viewer metadata sections.
 */

import { Show, createSignal, type JSX } from "solid-js";
import { ChevronDownIcon, ChevronRightIcon } from "../icons";

// ─── CollapsibleGroup ────────────────────────────────────────────────────────

/** Collapsible group with title */
export function CollapsibleGroup(props: {
  title: string;
  defaultOpen?: boolean;
  children: JSX.Element;
}) {
  const [open, setOpen] = createSignal(props.defaultOpen !== false);

  return (
    <div class="border-b border-border/30 pb-2">
      <button
        class="flex items-center gap-1 w-full text-left py-1 group"
        onClick={() => setOpen(!open())}
      >
        <Show
          when={open()}
          fallback={<ChevronRightIcon class="w-3 h-3 text-txt-muted" />}
        >
          <ChevronDownIcon class="w-3 h-3 text-txt-muted" />
        </Show>
        <span class="text-[10px] uppercase tracking-wider text-txt-muted font-medium group-hover:text-txt-secondary">
          {props.title}
        </span>
      </button>
      <Show when={open()}>
        <div class="pl-4 space-y-1 mt-1">{props.children}</div>
      </Show>
    </div>
  );
}

// ─── MetadataRow ─────────────────────────────────────────────────────────────

/** Single metadata key-value row */
export function MetadataRow(props: {
  label: string;
  value: string;
  highlight?: boolean;
  mono?: boolean;
  truncate?: boolean;
}) {
  return (
    <div class="flex items-baseline gap-2 text-xs py-0.5">
      <span class="text-txt-muted shrink-0 w-20">{props.label}</span>
      <span
        class={`text-txt min-w-0 ${props.highlight ? "text-accent" : ""} ${props.mono ? "font-mono text-[11px]" : ""} ${props.truncate ? "truncate" : "break-all"}`}
        title={props.value}
      >
        {props.value}
      </span>
    </div>
  );
}
