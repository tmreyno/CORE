// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For, type Accessor, type Setter } from "solid-js";

interface SearchHistoryProps {
  history: Accessor<string[]>;
  setQuery: Setter<string>;
  clearHistory: () => void;
  maxItems?: number;
}

export const SearchHistory: Component<SearchHistoryProps> = (props) => {
  const maxItems = props.maxItems ?? 8;

  return (
    <div class="px-3 py-2 border-b border-border bg-bg-panel/30">
      <div class="flex items-center justify-between mb-2">
        <span class="text-xs text-txt-secondary">Recent searches</span>
        <button
          class="text-xs text-txt-muted hover:text-txt-tertiary"
          onClick={props.clearHistory}
        >
          Clear
        </button>
      </div>
      <div class="flex flex-wrap gap-2">
        <For each={props.history().slice(0, maxItems)}>
          {(item) => (
            <button
              class="px-2 py-1 bg-bg-hover hover:bg-bg-active rounded text-xs text-txt-tertiary transition-colors"
              onClick={() => props.setQuery(item)}
            >
              {item}
            </button>
          )}
        </For>
      </div>
    </div>
  );
};
