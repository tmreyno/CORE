// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, Show, For, onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import {
  HiOutlineMagnifyingGlass,
  HiOutlineXMark,
  HiOutlineAdjustmentsHorizontal,
  HiOutlineClock,
} from "./icons";
import { Kbd, ModifierKeys } from "./ui/Kbd";
import { SearchFilters } from "./search/SearchFilters";
import { SearchHistory } from "./search/SearchHistory";
import { SearchResultItem } from "./search/SearchResultItem";
import { SearchEmptyStates } from "./search/SearchEmptyStates";
import { logger } from "../utils/logger";
const log = logger.scope("SearchPanel");

// ============================================================================
// Types
// ============================================================================

export interface SearchFilter {
  /** Filter by file type extension */
  fileTypes?: string[];
  /** Filter by size range (in bytes) */
  sizeRange?: { min?: number; max?: number };
  /** Filter by date range */
  dateRange?: { start?: Date; end?: Date };
  /** Include directories */
  includeDirs?: boolean;
  /** Include hidden files */
  includeHidden?: boolean;
  /** Search in file content */
  searchContent?: boolean;
  /** Case sensitive search */
  caseSensitive?: boolean;
  /** Regex search */
  useRegex?: boolean;
}

export interface SearchResult {
  /** Unique ID */
  id: string;
  /** File path (or entry path within container) */
  path: string;
  /** File name */
  name: string;
  /** Matched text context */
  matchContext?: string;
  /** Line number if content search */
  lineNumber?: number;
  /** File size */
  size: number;
  /** Is directory */
  isDir: boolean;
  /** Match score for ranking */
  score: number;
  /** Container path if result is from within a container */
  containerPath?: string;
  /** Container type (ad1, zip, e01, etc.) */
  containerType?: string;
  /** Match type: "name", "path", or "both" */
  matchType?: string;
}

export interface SavedSearch {
  id: string;
  name: string;
  query: string;
  filters: SearchFilter;
  createdAt: Date;
}

// ============================================================================
// Search Hook
// ============================================================================

export interface UseSearchOptions {
  /** Debounce delay in ms */
  debounceMs?: number;
  /** Max search history items */
  maxHistory?: number;
  /** Storage key for history */
  storageKey?: string;
}

export function useSearch(options: UseSearchOptions = {}) {
  const debounceMs = options.debounceMs ?? 300;
  const maxHistory = options.maxHistory ?? 20;
  const storageKey = options.storageKey ?? "ffx-search-history";

  const [query, setQuery] = createSignal("");
  const [debouncedQuery, setDebouncedQuery] = createSignal("");
  const [filters, setFilters] = createSignal<SearchFilter>({});
  const [results, setResults] = createSignal<SearchResult[]>([]);
  const [isSearching, setIsSearching] = createSignal(false);
  const [searchHistory, setSearchHistory] = createSignal<string[]>([]);
  const [savedSearches, setSavedSearches] = createSignal<SavedSearch[]>([]);

  // Debounce query
  let debounceTimer: number | undefined;
  createEffect(() => {
    const q = query();
    clearTimeout(debounceTimer);
    debounceTimer = window.setTimeout(() => {
      setDebouncedQuery(q);
    }, debounceMs);
  });

  // Load history from localStorage
  onMount(() => {
    try {
      const stored = localStorage.getItem(storageKey);
      if (stored) {
        const data = JSON.parse(stored);
        setSearchHistory(data.history ?? []);
        setSavedSearches(data.saved ?? []);
      }
    } catch (e) {
      log.warn("Failed to load search history:", e);
    }
  });

  // Save history to localStorage
  const saveToStorage = () => {
    try {
      localStorage.setItem(storageKey, JSON.stringify({
        history: searchHistory(),
        saved: savedSearches(),
      }));
    } catch (e) {
      log.warn("Failed to save search history:", e);
    }
  };

  // Add to history
  const addToHistory = (q: string) => {
    if (!q.trim()) return;
    setSearchHistory((prev) => {
      const filtered = prev.filter((h) => h !== q);
      const updated = [q, ...filtered].slice(0, maxHistory);
      return updated;
    });
    saveToStorage();
  };

  // Clear history
  const clearHistory = () => {
    setSearchHistory([]);
    saveToStorage();
  };

  // Save current search
  const saveSearch = (name: string) => {
    const newSaved: SavedSearch = {
      id: `saved-${Date.now()}`,
      name,
      query: query(),
      filters: filters(),
      createdAt: new Date(),
    };
    setSavedSearches((prev) => [...prev, newSaved]);
    saveToStorage();
  };

  // Delete saved search
  const deleteSavedSearch = (id: string) => {
    setSavedSearches((prev) => prev.filter((s) => s.id !== id));
    saveToStorage();
  };

  // Load a saved search
  const loadSavedSearch = (saved: SavedSearch) => {
    setQuery(saved.query);
    setFilters(saved.filters);
  };

  // Update filter
  const updateFilter = <K extends keyof SearchFilter>(key: K, value: SearchFilter[K]) => {
    setFilters((prev) => ({ ...prev, [key]: value }));
  };

  // Reset filters
  const resetFilters = () => {
    setFilters({});
  };

  // Clear search
  const clearSearch = () => {
    setQuery("");
    setResults([]);
  };

  return {
    query,
    setQuery,
    debouncedQuery,
    filters,
    setFilters,
    updateFilter,
    resetFilters,
    results,
    setResults,
    isSearching,
    setIsSearching,
    searchHistory,
    addToHistory,
    clearHistory,
    savedSearches,
    saveSearch,
    deleteSavedSearch,
    loadSavedSearch,
    clearSearch,
  };
}

// ============================================================================
// Search Panel Component
// ============================================================================

export interface SearchPanelProps {
  /** Whether the panel is open */
  isOpen: boolean;
  /** Close handler */
  onClose: () => void;
  /** Search function - called with query and filters, should return results */
  onSearch: (query: string, filters: SearchFilter) => Promise<SearchResult[]>;
  /** Called when a result is selected */
  onSelectResult: (result: SearchResult) => void;
  /** Placeholder text */
  placeholder?: string;
}

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
          <div class="flex items-center gap-3 px-5 py-4 border-b border-border/50 bg-bg-secondary/30">
            <div class="p-2 rounded-lg bg-accent/10 text-accent">
              <HiOutlineMagnifyingGlass class="w-5 h-5" />
            </div>
            <input
              ref={inputRef}
              type="text"
              class="flex-1 bg-transparent border-none outline-none text-txt text-base placeholder-txt-muted"
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
            <div class="px-5 py-3 border-t border-border/50 bg-bg-secondary/30 flex items-center justify-between">
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
