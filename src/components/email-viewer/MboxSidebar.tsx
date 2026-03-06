// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For } from "solid-js";
import type { EmailInfo } from "./types";
import { formatEmailDate } from "./helpers";
import type { Accessor } from "solid-js";

interface MboxSidebarProps {
  emails: EmailInfo[];
  selectedIndex: Accessor<number>;
  onSelect: (index: number) => void;
}

export function MboxSidebar(props: MboxSidebarProps) {
  return (
    <div class="w-64 shrink-0 border-r border-border overflow-y-auto bg-bg-secondary">
      <div class="p-2 text-xs text-txt-muted font-medium border-b border-border">
        Messages ({props.emails.length})
      </div>
      <For each={props.emails}>
        {(email, i) => (
          <button
            class="w-full text-left p-2 border-b border-border/50 hover:bg-bg-hover transition-colors"
            classList={{ "bg-bg-active": props.selectedIndex() === i() }}
            onClick={() => props.onSelect(i())}
          >
            <div class="text-xs font-medium text-txt truncate">{email.subject || "(No Subject)"}</div>
            <div class="text-[10px] text-txt-muted truncate">
              {email.from.length > 0 ? (email.from[0].name || email.from[0].address) : "Unknown"}
            </div>
            <div class="text-[10px] text-txt-muted">{formatEmailDate(email.date)}</div>
          </button>
        )}
      </For>
    </div>
  );
}
