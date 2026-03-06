// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * InfoRow / InfoRows — reusable detail row components for rendering
 * key-value metadata fields with consistent styling.
 */

import { Show, For } from "solid-js";
import type { InfoField } from "./types";
import { formatBytes } from "../../utils";

/**
 * Renders a single info row with consistent styling.
 * Skips rendering if no value (unless `condition` explicitly set).
 */
export function InfoRow(props: InfoField) {
  const shouldShow = () => {
    if (props.condition !== undefined) return props.condition;
    return props.value !== undefined && props.value !== null && props.value !== "";
  };

  const formatValue = () => {
    const val = props.value;
    if (val === undefined || val === null) return "";
    if (props.format === "bytes" && typeof val === "number") return formatBytes(val);
    return String(val);
  };

  const rowClass = () => {
    const base = "flex items-start gap-2 py-1 px-1.5 text-xs rounded";
    if (props.type === "highlight") return `${base} bg-accent/20`;
    if (props.type === "device") return `${base} bg-amber-900/10`;
    if (props.type === "full-width") return `${base} col-span-2`;
    if (props.type === "hash") return `${base} bg-bg-panel/50 col-span-2`;
    if (props.type === "warning" || props.format === "warning")
      return `${base} bg-red-900/20 col-span-2`;
    return base;
  };

  const valueClass = () => {
    const base = "text-txt flex-1";
    if (props.format === "mono") return `${base} font-mono text-[10px] leading-tight`;
    if (props.format === "notes") return `${base} text-txt-secondary italic`;
    if (props.format === "list") return `${base} text-txt-secondary`;
    if (props.type === "hash")
      return `${base} font-mono text-[10px] leading-tight break-all`;
    if (props.format === "warning") return `${base} text-red-400`;
    return base;
  };

  return (
    <Show when={shouldShow()}>
      <div class={rowClass()}>
        <span class="text-txt-muted shrink-0 w-24">{props.label}</span>
        <span class={valueClass()}>{formatValue()}</span>
      </div>
    </Show>
  );
}

/**
 * Renders multiple InfoRow components from an array of field definitions.
 */
export function InfoRows(props: { fields: InfoField[] }) {
  return (
    <For each={props.fields}>{(field) => <InfoRow {...field} />}</For>
  );
}
