// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared UI primitives for right-panel metadata display.
 *
 * ─── RIGHT PANEL UI STANDARD ────────────────────────────────────────────────
 *
 * ALL right-panel components should use these shared primitives for visual
 * consistency. The standard defines:
 *
 * Layout:
 *   • Root container:   `flex flex-col h-full bg-bg`
 *   • Scrollable body:  `flex-1 overflow-y-auto`
 *   • Content padding:  `p-3 space-y-3` (sections) or `p-2 space-y-2` (lists)
 *
 * Tabs (when present):
 *   • Header:  `flex items-center border-b border-border bg-bg-secondary`
 *   • Active:  `text-accent border-b-2 border-accent`
 *   • Tab btn: `px-3 py-2 text-xs font-medium transition-colors`
 *
 * Sections (collapsible groups):
 *   • Use `CollapsibleGroup` — `text-2xs uppercase tracking-wider
 *     text-txt-muted font-medium`, ChevronDown/Right w-3 h-3
 *   • Border: `border-b border-border/30 pb-2`
 *   • Content indent: `pl-4 space-y-1 mt-1`
 *
 * Key-value rows:
 *   • Use `MetadataRow` — `flex items-baseline gap-2 text-xs py-0.5`
 *   • Label: `w-20 text-txt-muted shrink-0` (LEFT-aligned, NOT right)
 *   • Value: `text-txt break-all` (or `truncate` for paths)
 *
 * Section headers (non-collapsible):
 *   • Use `SectionHeader` — `text-2xs uppercase tracking-wider
 *     text-txt-muted font-medium`
 *
 * Summary rows (stat-like icon + label + value):
 *   • Use `SummaryRow` — `text-xs` (NOT text-sm), `bg-bg-secondary rounded`
 *
 * Status badges:
 *   • Use `StatusBadge` — `text-2xs font-medium px-1.5 py-0.5 rounded`
 *
 * Panel headers (non-tabbed):
 *   • `flex items-center justify-between px-3 py-2 border-b border-border`
 *   • Title: `text-xs font-medium text-txt` (NOT text-sm)
 *
 * Empty states:
 *   • `flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2`
 *   • Icon `w-8 h-8 opacity-30` + descriptive text
 * ─────────────────────────────────────────────────────────────────────────────
 */

import { Show, createSignal, type JSX } from "solid-js";
import { ChevronDownIcon, ChevronRightIcon } from "../icons";

// ─── CollapsibleGroup ────────────────────────────────────────────────────────

/** Collapsible group with title — standard section wrapper for right panel */
export function CollapsibleGroup(props: {
  title: string;
  defaultOpen?: boolean;
  /** Optional trailing element (count badge, action button) */
  trailing?: JSX.Element;
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
        <span class="text-2xs uppercase tracking-wider text-txt-muted font-medium group-hover:text-txt-secondary flex-1">
          {props.title}
        </span>
        <Show when={props.trailing}>{props.trailing}</Show>
      </button>
      <Show when={open()}>
        <div class="pl-4 space-y-1 mt-1">{props.children}</div>
      </Show>
    </div>
  );
}

// ─── MetadataRow ─────────────────────────────────────────────────────────────

/** Single metadata key-value row — standard for all right-panel data display */
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
        class={`text-txt min-w-0 ${props.highlight ? "text-accent" : ""} ${props.mono ? "font-mono text-compact" : ""} ${props.truncate ? "truncate" : "break-all"}`}
        title={props.value}
      >
        {props.value}
      </span>
    </div>
  );
}

// ─── OptionalMetadataRow ─────────────────────────────────────────────────────

/**
 * Key-value row that auto-hides when value is undefined/empty.
 * Use this instead of wrapping MetadataRow in `<Show when={...}>`.
 */
export function OptionalMetadataRow(props: {
  label: string;
  value?: string;
  highlight?: boolean;
  mono?: boolean;
  truncate?: boolean;
}) {
  return (
    <Show when={props.value}>
      <div class="flex items-baseline gap-2 text-xs py-0.5">
        <span class="text-txt-muted shrink-0 w-20">{props.label}</span>
        <span
          class={`text-txt min-w-0 ${props.highlight ? "text-accent" : ""} ${props.mono ? "font-mono text-compact" : ""} ${props.truncate ? "truncate" : "break-all"}`}
          title={props.value}
        >
          {props.value}
        </span>
      </div>
    </Show>
  );
}

// ─── SectionHeader ───────────────────────────────────────────────────────────

/** Non-collapsible section heading — matches CollapsibleGroup title style */
export function SectionHeader(props: { label: string }) {
  return (
    <div class="text-2xs font-medium text-txt-muted uppercase tracking-wider">
      {props.label}
    </div>
  );
}

// ─── SummaryRow ──────────────────────────────────────────────────────────────

/** Icon + label + value row for summary statistics */
export function SummaryRow(props: {
  icon: JSX.Element;
  label: string;
  value: number | string;
}) {
  return (
    <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-bg-secondary">
      {props.icon}
      <span class="flex-1 text-xs text-txt">{props.label}</span>
      <span class="text-xs font-medium text-txt">{props.value}</span>
    </div>
  );
}

// ─── StatusBadge ─────────────────────────────────────────────────────────────

/** Status indicator badge for draft/locked/voided/complete states */
export function StatusBadge(props: { status: string }) {
  return (
    <span
      class="text-2xs font-medium px-1.5 py-0.5 rounded"
      classList={{
        "text-success bg-success/10": props.status === "complete" || props.status === "locked",
        "text-warning bg-warning/10": props.status === "draft",
        "text-error bg-error/10": props.status === "voided",
        "text-txt-muted bg-bg-hover": !["complete", "locked", "draft", "voided"].includes(props.status),
      }}
    >
      {props.status || "draft"}
    </span>
  );
}
