// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import { For } from "solid-js";
import { Kbd } from "../../ui/Kbd";

// =============================================================================
// ShortcutGroup sub-component
// =============================================================================

export const ShortcutGroup: Component<{
  title: string;
  shortcuts: { keys: string; desc: string }[];
}> = (props) => (
  <div>
    <h4 class="text-xs font-semibold text-txt-muted uppercase tracking-wider mb-2">
      {props.title}
    </h4>
    <div class="space-y-1">
      <For each={props.shortcuts}>
        {(s) => (
          <div class="flex items-center justify-between py-1.5 px-3 bg-bg-secondary rounded border border-border/30">
            <span class="text-txt text-sm">{s.desc}</span>
            <Kbd keys={s.keys} muted />
          </div>
        )}
      </For>
    </div>
  </div>
);

// =============================================================================
// KeyboardShortcutsContent
// =============================================================================

export const KeyboardShortcutsContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Press <Kbd keys="?" muted /> at any time to open the full keyboard shortcuts reference.
      Here are the most important shortcuts:
    </p>

    <div class="space-y-4">
      <ShortcutGroup
        title="General"
        shortcuts={[
          { keys: "Cmd+K", desc: "Open command palette" },
          { keys: "Cmd+,", desc: "Open settings" },
          { keys: "?", desc: "Show keyboard shortcuts" },
          { keys: "Esc", desc: "Close dialog / clear filter" },
        ]}
      />
      <ShortcutGroup
        title="Project"
        shortcuts={[
          { keys: "Cmd+Shift+N", desc: "New project" },
          { keys: "Cmd+O", desc: "Open project" },
          { keys: "Cmd+S", desc: "Save project" },
          { keys: "Cmd+Shift+S", desc: "Save project as…" },
        ]}
      />
      <ShortcutGroup
        title="View"
        shortcuts={[
          { keys: "Cmd+1", desc: "Info view" },
          { keys: "Cmd+2", desc: "Hex view" },
          { keys: "Cmd+3", desc: "Text view" },
          { keys: "Cmd+B", desc: "Toggle left panel" },
          { keys: "Cmd+Shift+B", desc: "Toggle right panel" },
        ]}
      />
      <ShortcutGroup
        title="Actions"
        shortcuts={[
          { keys: "Cmd+F", desc: "Search files" },
          { keys: "Cmd+P", desc: "Generate report" },
          { keys: "Cmd+H", desc: "Compute hash" },
          { keys: "Cmd+E", desc: "Export" },
        ]}
      />
    </div>
  </div>
);
