// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, type Accessor } from "solid-js";
import type { SearchFilter } from "../SearchPanel";

interface SearchFiltersProps {
  filters: Accessor<SearchFilter>;
  updateFilter: <K extends keyof SearchFilter>(
    key: K,
    value: SearchFilter[K]
  ) => void;
  resetFilters: () => void;
}

export const SearchFilters: Component<SearchFiltersProps> = (props) => {
  return (
    <div class="px-5 py-3 border-b border-border/50 bg-bg-secondary/30">
      <div class="flex flex-wrap items-center gap-4 text-sm">
        {/* File Types */}
        <div class="flex items-center gap-2">
          <label class="text-txt-secondary">Type:</label>
          <select
            class="w-full px-2 py-1 text-xs bg-bg-tertiary border border-border rounded text-txt-primary placeholder:text-txt-muted focus:outline-none focus:ring-1 focus:ring-accent"
            onChange={(e) => {
              const val = e.currentTarget.value;
              props.updateFilter("fileTypes", val ? [val] : undefined);
            }}
          >
            <option value="">All</option>
            <option value=".E01">E01</option>
            <option value=".ad1">AD1</option>
            <option value=".zip">ZIP</option>
            <option value=".db">Database</option>
            <option value=".txt">Text</option>
          </select>
        </div>

        {/* Case Sensitive */}
        <label class="flex items-center gap-1.5 cursor-pointer">
          <input
            type="checkbox"
            class="accent-accent"
            checked={props.filters().caseSensitive ?? false}
            onChange={(e) =>
              props.updateFilter("caseSensitive", e.currentTarget.checked)
            }
          />
          <span class="text-txt-tertiary">Case sensitive</span>
        </label>

        {/* Regex */}
        <label class="flex items-center gap-1.5 cursor-pointer">
          <input
            type="checkbox"
            class="accent-accent"
            checked={props.filters().useRegex ?? false}
            onChange={(e) => props.updateFilter("useRegex", e.currentTarget.checked)}
          />
          <span class="text-txt-tertiary">Regex</span>
        </label>

        {/* Include Dirs */}
        <label class="flex items-center gap-1.5 cursor-pointer">
          <input
            type="checkbox"
            class="accent-accent"
            checked={props.filters().includeDirs ?? true}
            onChange={(e) =>
              props.updateFilter("includeDirs", e.currentTarget.checked)
            }
          />
          <span class="text-txt-tertiary">Include folders</span>
        </label>

        {/* Reset */}
        <button
          class="text-xs text-accent hover:text-accent-hover"
          onClick={props.resetFilters}
        >
          Reset filters
        </button>
      </div>
    </div>
  );
};
