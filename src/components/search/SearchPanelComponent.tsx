// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SearchPanel — modal search overlay with debounced input,
 * keyboard navigation, filters, history, and result list.
 */

import { createSignal, createEffect, Show, For, onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import {
  HiOutlineMagnifyingGlass,
  HiOutlineXMark,
  HiOutlineAdjustmentsHorizontal,
  HiOutlineClock,
} from "../icons";
import { Kbd, ModifierKeys } from "../ui/Kbd";
import { SearchFilters } from "./SearchFilters";
import { SearchHistory } from "./SearchHistory";
import { SearchResultItem } from "./SearchResultItem";
import { SearchEmptyStates } from "./SearchEmptyStates";
import { useSearch } from "./useSearch";
import type { SearchPanelProps } from "./types";
import { logger } from "../../utils/logger";

const log = logger.scope("SearchPanel");

export function SearchPanel(props: SearchPanelProps) {
  let inputRef: HTMLInputElement | undefined;
  const search = useSearch();
  const [showFilters, setShowFilters] = createSignal(false);
  const [showHistory, setShowHistory] = createSignal(false);
  const [selectedIndex, setSelectedIndex] = createSignal(0);

  // Focus input when opened
  createEffect(() => {
    if (props.isOpen) {
      setTimeout(() => inputRef?.focus(), 50);
    }
  });

  // Apply initialQuery when panel opens with one
  createEffect(() => {
    if (props.isOpen && props.initialQuery) {
      search.setQuery(props.initialQuery);
      props.onInitialQueryConsumed?.();
    }
  });

  // Perform search when debounced query changes
  createEffect(async () => {
    const q = search.debouncedQuery();
    if (!q.trim()) {
      search.setResults([]);
      return;
    }

    search.setIsSearching(true);
    try {
      const results = await props.onSearch(q, search.filters());
      search.setResults(results);
      search.addToHistory(q);
      setSelectedIndex(0);
    } catch (e) {
      log.error("Search failed:", e);
      search.setResults([]);
    } finally {
      search.setIsSearching(false);
    }
  });

  // Keyboard navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    const results = search.results();
    
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex((i) => Math.min(i + 1, results.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex((i) => Math.max(i - 1, 0));
        break;
      case "Enter":
        e.preventDefault();
        if (results[selectedIndex()]) {
          props.onSelectResult(results[selectedIndex()]);
          props.onClose();
        }
        break;
      case "Escape":
        e.preventDefault();
        props.onClose();
        break;
    }
  };

  // Close on click outside
  onMount(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (props.isOpen && !target.closest(".search-panel")) {
        props.onClose();
      }
    };
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(document, "mousedown", handleClickOutside);
  });

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-start justify-center pt-16 animate-fade-in">
        <div class="search-panel w-[640px] max-h-[75vh] flex flex-col bg-bg-panel border border-border rounded-2xl shadow-2xl overflow-hidden animate-slide-up"
          style={{ "box-shadow": "0 25px 50px -12px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(var(--color-accent-rgb), 0.1)" }}
        >
          {/* Search Input */}
          <div class="flex items-center gap-2.5 px-4 py-3 border-b border-border/50 bg-bg-secondary/30">
            <div class="p-1.5 rounded-md bg-accent/10 text-accent">
              <HiOutlineMagnifyingGlass class="w-4 h-4" />
            </div>
            <input
              ref={inputRef}
              type="text"
              class="flex-1 bg-transparent border-none outline-none text-txt text-sm placeholder-txt-muted"
              placeholder={props.placeholder ?? "Search files and content..."}
              value={search.query()}
              onInput={(e) => search.setQuery(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              onFocus={() => setShowHistory(true)}
              onBlur={() => setTimeout(() => setShowHistory(false), 200)}
            />
            <Show when={search.isSearching()}>
              <div class="w-5 h-5 border-2 border-accent border-t-transparent rounded-full animate-spin" />
            </Show>
            <Show when={search.query() && !search.isSearching()}>
              <span class="text-xs text-txt-muted px-2 py-1 bg-bg-secondary rounded-md">
                {search.results().length} found
              </span>
            </Show>
            <button
              class={`p-2 rounded-lg transition-colors ${showFilters() ? "bg-accent/15 text-accent" : "hover:bg-bg-hover text-txt-secondary"}`}
              onClick={() => setShowFilters(!showFilters())}
              title="Toggle filters"
            >
              <HiOutlineAdjustmentsHorizontal class="w-4 h-4" />
            </button>
            <button
              class="p-2 rounded-lg hover:bg-bg-hover text-txt-secondary transition-colors"
              onClick={props.onClose}
              title="Close (Esc)"
            >
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </div>

          {/* Filters */}
          <Show when={showFilters()}>
            <SearchFilters
              filters={search.filters}
              updateFilter={search.updateFilter}
              resetFilters={search.resetFilters}
            />
          </Show>

          {/* Search History (shown when input is focused and empty) */}
          <Show when={showHistory() && !search.query() && search.searchHistory().length > 0}>
            <SearchHistory
              history={search.searchHistory}
              setQuery={search.setQuery}
              clearHistory={search.clearHistory}
            />
          </Show>

          {/* Results */}
          <div class="flex-1 overflow-y-auto">
            <Show when={search.results().length > 0}>
              <For each={search.results()}>
                {(result, index) => (
                  <SearchResultItem
                    result={result}
                    isSelected={selectedIndex() === index()}
                    onSelect={() => {
                      props.onSelectResult(result);
                      props.onClose();
                    }}
                    onMouseEnter={() => setSelectedIndex(index())}
                  />
                )}
              </For>
            </Show>

            {/* No results or empty states */}
            <Show when={search.debouncedQuery() && !search.isSearching() && search.results().length === 0}>
              <SearchEmptyStates
                query={search.debouncedQuery()}
                showFilters={showFilters()}
                resetFilters={search.resetFilters}
              />
            </Show>

            <Show when={!search.query() && !showHistory()}>
              <SearchEmptyStates
                query=""
                showFilters={showFilters()}
                resetFilters={search.resetFilters}
              />
            </Show>
          </div>

          {/* Footer with result count */}
          <Show when={search.results().length > 0}>
            <div class="px-4 py-2.5 border-t border-border/50 bg-bg-secondary/30 flex items-center justify-between">
              <span class="text-xs text-txt-secondary">
                {search.results().length} result{search.results().length !== 1 ? "s" : ""} found
              </span>
              <span class="text-xs text-txt-muted flex items-center gap-1.5">
                <HiOutlineClock class="w-3.5 h-3.5" />
                <Kbd keys={ModifierKeys.enter} muted /> to open
              </span>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
}
