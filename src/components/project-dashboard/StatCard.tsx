// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import type { StatCardProps } from "./types";

/** Compact stat card for dashboard grid */
export const StatCard: Component<StatCardProps> = (props) => (
  <button
    class={`stat-box cursor-pointer hover:bg-bg-hover transition-colors ${props.onClick ? "" : "cursor-default"}`}
    onClick={props.onClick}
    disabled={!props.onClick}
    title={props.onClick ? `View ${props.label}` : undefined}
  >
    <div class="flex items-center gap-1.5 mb-1">
      <props.icon class={`w-3.5 h-3.5 ${props.accent ? "text-accent" : "text-txt-muted"}`} />
      <span class="text-[10px] font-medium text-txt-muted uppercase tracking-wide truncate">
        {props.label}
      </span>
    </div>
    <div class={`text-lg font-semibold ${props.accent ? "text-accent" : "text-txt"}`}>
      {props.value}
    </div>
  </button>
);
