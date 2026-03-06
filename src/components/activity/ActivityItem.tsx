// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import type { ActivityLogEntry } from "../../types/project";
import { getCategoryIcon, formatTimestamp } from "./helpers";

interface ActivityItemProps {
  entry: ActivityLogEntry;
}

export const ActivityItem: Component<ActivityItemProps> = (props) => {
  return (
    <div class="px-3 py-2 border-b border-border/30 hover:bg-bg-hover group">
      <div class="flex items-start gap-2">
        <div class="mt-0.5 text-txt-muted">
          {getCategoryIcon(props.entry.category)}
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-sm text-txt truncate">{props.entry.description}</div>
          <div class="flex items-center gap-2 mt-0.5 text-[10px] text-txt-muted">
            <span>{formatTimestamp(props.entry.timestamp)}</span>
            <span class="opacity-50">•</span>
            <span class="capitalize">{props.entry.action}</span>
            <Show when={props.entry.user}>
              <span class="opacity-50">•</span>
              <span>{props.entry.user}</span>
            </Show>
          </div>
        </div>
      </div>
    </div>
  );
};
