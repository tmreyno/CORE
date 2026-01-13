// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, Show, For, onMount, onCleanup, Accessor } from "solid-js";
import { formatBytes } from "../utils";
import {
  HiOutlineMagnifyingGlass,
  HiOutlineXMark,
  HiOutlineAdjustmentsHorizontal,
  HiOutlineClock,
  HiOutlineFolder,
  HiOutlineDocument,
} from "./icons";

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
  /** File path */
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
      console.warn("Failed to load search history:", e);
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
      console.warn("Failed to save search history:", e);
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
      console.error("Search failed:", e);
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
    document.addEventListener("mousedown", handleClickOutside);
    onCleanup(() => document.removeEventListener("mousedown", handleClickOutside));
  });

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-start justify-center pt-20">
        <div class="search-panel bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl w-[600px] max-h-[70vh] flex flex-col overflow-hidden">
          {/* Search Input */}
          <div class="flex items-center gap-3 px-4 py-3 border-b border-zinc-700">
            <HiOutlineMagnifyingGlass class="w-5 h-5 text-zinc-400" />
            <input
              ref={inputRef}
              type="text"
              class="flex-1 bg-transparent border-none outline-none text-zinc-100 placeholder-zinc-500"
              placeholder={props.placeholder ?? "Search files and content..."}
              value={search.query()}
              onInput={(e) => search.setQuery(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              onFocus={() => setShowHistory(true)}
              onBlur={() => setTimeout(() => setShowHistory(false), 200)}
            />
            <Show when={search.isSearching()}>
              <div class="w-4 h-4 border-2 border-cyan-500 border-t-transparent rounded-full animate-spin" />
            </Show>
            <button
              class="icon-btn-sm"
              onClick={() => setShowFilters(!showFilters())}
              title="Toggle filters"
            >
              <HiOutlineAdjustmentsHorizontal class="w-4 h-4" />
            </button>
            <button
              class="icon-btn-sm"
              onClick={props.onClose}
              title="Close"
            >
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </div>

          {/* Filters */}
          <Show when={showFilters()}>
            <div class="px-4 py-3 border-b border-zinc-700 bg-zinc-800/50">
              <div class="flex flex-wrap gap-3 text-sm">
                {/* File Types */}
                <div class="flex items-center gap-2">
                  <label class="text-zinc-400">Type:</label>
                  <select
                    class="bg-zinc-800 border border-zinc-600 rounded px-2 py-1 text-zinc-200 text-xs"
                    onChange={(e) => {
                      const val = e.currentTarget.value;
                      search.updateFilter("fileTypes", val ? [val] : undefined);
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
                    class="accent-cyan-500"
                    checked={search.filters().caseSensitive ?? false}
                    onChange={(e) => search.updateFilter("caseSensitive", e.currentTarget.checked)}
                  />
                  <span class="text-zinc-300">Case sensitive</span>
                </label>

                {/* Regex */}
                <label class="flex items-center gap-1.5 cursor-pointer">
                  <input
                    type="checkbox"
                    class="accent-cyan-500"
                    checked={search.filters().useRegex ?? false}
                    onChange={(e) => search.updateFilter("useRegex", e.currentTarget.checked)}
                  />
                  <span class="text-zinc-300">Regex</span>
                </label>

                {/* Include Dirs */}
                <label class="flex items-center gap-1.5 cursor-pointer">
                  <input
                    type="checkbox"
                    class="accent-cyan-500"
                    checked={search.filters().includeDirs ?? true}
                    onChange={(e) => search.updateFilter("includeDirs", e.currentTarget.checked)}
                  />
                  <span class="text-zinc-300">Include folders</span>
                </label>

                {/* Reset */}
                <button
                  class="text-xs text-cyan-400 hover:text-cyan-300"
                  onClick={search.resetFilters}
                >
                  Reset filters
                </button>
              </div>
            </div>
          </Show>

          {/* Search History (shown when input is focused and empty) */}
          <Show when={showHistory() && !search.query() && search.searchHistory().length > 0}>
            <div class="px-3 py-2 border-b border-zinc-700 bg-zinc-800/30">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-zinc-400">Recent searches</span>
                <button
                  class="text-xs text-zinc-500 hover:text-zinc-300"
                  onClick={search.clearHistory}
                >
                  Clear
                </button>
              </div>
              <div class="flex flex-wrap gap-2">
                <For each={search.searchHistory().slice(0, 8)}>
                  {(item) => (
                    <button
                      class="px-2 py-1 bg-zinc-700 hover:bg-zinc-600 rounded text-xs text-zinc-300 transition-colors"
                      onClick={() => search.setQuery(item)}
                    >
                      {item}
                    </button>
                  )}
                </For>
              </div>
            </div>
          </Show>

          {/* Results */}
          <div class="flex-1 overflow-y-auto">
            <Show when={search.results().length > 0}>
              <For each={search.results()}>
                {(result, index) => (
                  <button
                    class={`w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors ${
                      selectedIndex() === index()
                        ? "bg-cyan-600/20"
                        : "hover:bg-zinc-800"
                    }`}
                    onClick={() => {
                      props.onSelectResult(result);
                      props.onClose();
                    }}
                    onMouseEnter={() => setSelectedIndex(index())}
                  >
                    <span class="shrink-0">
                      {result.isDir 
                        ? <HiOutlineFolder class="w-5 h-5 text-amber-400" />
                        : <HiOutlineDocument class="w-5 h-5 text-zinc-400" />
                      }
                    </span>
                    <div class="flex-1 min-w-0">
                      <div class="text-sm text-zinc-100 truncate">{result.name}</div>
                      <div class="text-xs text-zinc-500 truncate">{result.path}</div>
                      <Show when={result.matchContext}>
                        <div class="text-xs text-zinc-400 mt-0.5 truncate">
                          ...{result.matchContext}...
                        </div>
                      </Show>
                    </div>
                    <span class="file-size">
                      {formatBytes(result.size)}
                    </span>
                  </button>
                )}
              </For>
            </Show>

            {/* No results */}
            <Show when={search.debouncedQuery() && !search.isSearching() && search.results().length === 0}>
              <div class="flex flex-col items-center justify-center py-12 text-zinc-500">
                <HiOutlineMagnifyingGlass class="w-10 h-10 mb-3 opacity-50" />
                <p class="text-sm">No results found for "{search.debouncedQuery()}"</p>
                <p class="text-xs mt-1">Try different keywords or adjust filters</p>
              </div>
            </Show>

            {/* Empty state */}
            <Show when={!search.query() && !showHistory()}>
              <div class="flex flex-col items-center justify-center py-12 text-zinc-500">
                <HiOutlineMagnifyingGlass class="w-10 h-10 mb-3 opacity-50" />
                <p class="text-sm">Start typing to search</p>
                <div class="flex gap-4 mt-4 text-xs">
                  <span>⌘F to open</span>
                  <span>↑↓ to navigate</span>
                  <span>Enter to select</span>
                </div>
              </div>
            </Show>
          </div>

          {/* Footer with result count */}
          <Show when={search.results().length > 0}>
            <div class="px-4 py-2 border-t border-zinc-700 bg-zinc-800/50 text-xs text-zinc-400">
              {search.results().length} result{search.results().length !== 1 ? "s" : ""}
              <span class="ml-4 inline-flex items-center gap-1">
                <HiOutlineClock class="w-3 h-3" /> Press ↑↓ to navigate, Enter to select
              </span>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
}

export default SearchPanel;
