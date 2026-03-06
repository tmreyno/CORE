// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * GlobalExaminersPanel — Collapsible panel listing all unique examiners
 * across every project in the merge.
 */

import { Component, For, Show } from "solid-js";
import {
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineUserGroup,
} from "../icons";
import type { GlobalExaminer } from "./types";

export interface GlobalExaminersPanelProps {
  examiners: GlobalExaminer[];
  isExpanded: boolean;
  onToggle: () => void;
}

export const GlobalExaminersPanel: Component<GlobalExaminersPanelProps> = (props) => {
  return (
    <div class="p-3 rounded-lg bg-bg-panel border border-border">
      <button
        class="flex items-center gap-2 text-sm font-semibold text-txt w-full text-left"
        onClick={props.onToggle}
      >
        <Show when={props.isExpanded} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5" />}>
          <HiOutlineChevronDown class="w-3.5 h-3.5" />
        </Show>
        <HiOutlineUserGroup class="w-4 h-4 text-accent" />
        All DB Users &amp; Examiners ({props.examiners.length})
      </button>
      <Show when={props.isExpanded}>
        <div class="mt-2 col gap-1.5">
          <For each={props.examiners}>
            {(ex) => {
              const label = ex.displayName || ex.name;
              const roleClass =
                ex.role === "project owner"
                  ? "badge badge-success"
                  : ex.role.includes("COC")
                    ? "badge badge-warning"
                    : "badge";
              return (
                <div class="flex items-center gap-2 text-xs">
                  <span class="text-txt font-medium">{label}</span>
                  <span class={roleClass} style="font-size: 10px; padding: 1px 5px;">
                    {ex.role}
                  </span>
                  <span class="text-txt-muted" style="font-size: 10px;">
                    ({ex.source})
                  </span>
                  <span
                    class="text-txt-muted ml-auto"
                    style="font-size: 10px;"
                    title={ex.projects.join(", ")}
                  >
                    {ex.projects.length > 1 ? `${ex.projects.length} projects` : ex.projects[0]}
                  </span>
                </div>
              );
            }}
          </For>
        </div>
      </Show>
    </div>
  );
};
