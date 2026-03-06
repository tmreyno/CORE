// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Reactive search hook — debounced query, search history,
 * saved searches, filters with localStorage persistence.
 */

import { createSignal, createEffect, onMount } from "solid-js";
import type { SearchFilter, SearchResult, SavedSearch, UseSearchOptions } from "./types";
import { logger } from "../../utils/logger";

const log = logger.scope("useSearch");

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
