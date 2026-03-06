// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ToolsTabId } from "./types";
import type { Accessor } from "solid-js";

const TABS: { id: ToolsTabId; label: string }[] = [
  { id: "test", label: "Test" },
  { id: "repair", label: "Repair" },
  { id: "validate", label: "Validate" },
  { id: "extract", label: "Extract Split" },
  { id: "compress", label: "Compress" },
  { id: "decompress", label: "Decompress" },
];

interface ToolsTabSelectorProps {
  active: Accessor<ToolsTabId>;
  onSelect: (tab: ToolsTabId) => void;
}

export function ToolsTabSelector(props: ToolsTabSelectorProps) {
  return (
    <div class="flex gap-1 border-b border-border">
      {TABS.map((tab) => (
        <button
          class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
            props.active() === tab.id
              ? "border-accent text-accent"
              : "border-transparent text-txt-secondary hover:text-txt"
          }`}
          onClick={() => props.onSelect(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
