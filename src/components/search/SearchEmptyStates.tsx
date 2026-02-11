// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { HiOutlineMagnifyingGlass } from "../icons";
import { Kbd, ModifierKeys } from "../ui/Kbd";

interface SearchEmptyStatesProps {
  /** Current search query */
  query: string;
  /** Whether filters are shown */
  showFilters: boolean;
  /** Reset filters callback */
  resetFilters: () => void;
}

export const SearchEmptyStates: Component<SearchEmptyStatesProps> = (props) => {
  return (
    <>
      {/* No results */}
      <Show when={props.query}>
        <div class="flex flex-col items-center justify-center py-16 text-center">
          <div class="w-16 h-16 mb-4 rounded-2xl bg-bg-secondary flex items-center justify-center">
            <HiOutlineMagnifyingGlass class="w-8 h-8 text-txt-muted" />
          </div>
          <p class="text-txt font-medium mb-1">No results found</p>
          <p class="text-sm text-txt-muted max-w-xs">
            No files match "{props.query}". Try different keywords or adjust
            filters.
          </p>
          <Show when={props.showFilters}>
            <button
              class="mt-4 text-sm text-accent hover:text-accent-hover"
              onClick={props.resetFilters}
            >
              Clear all filters
            </button>
          </Show>
        </div>
      </Show>

      {/* Empty state - no query */}
      <Show when={!props.query}>
        <div class="flex flex-col items-center justify-center py-16 text-center">
          <div class="w-16 h-16 mb-4 rounded-2xl bg-accent/10 flex items-center justify-center">
            <HiOutlineMagnifyingGlass class="w-8 h-8 text-accent" />
          </div>
          <p class="text-txt font-medium mb-1">Search evidence files</p>
          <p class="text-sm text-txt-muted max-w-xs mb-6">
            Search across all loaded evidence containers, processed databases,
            and case documents.
          </p>
          <div class="flex items-center gap-6 text-xs text-txt-muted">
            <span class="flex items-center gap-1.5">
              <Kbd keys={[ModifierKeys.up, ModifierKeys.down]} muted />
              Navigate
            </span>
            <span class="flex items-center gap-1.5">
              <Kbd keys={ModifierKeys.enter} muted />
              Select
            </span>
            <span class="flex items-center gap-1.5">
              <Kbd keys={ModifierKeys.esc} muted />
              Close
            </span>
          </div>
        </div>
      </Show>
    </>
  );
};
