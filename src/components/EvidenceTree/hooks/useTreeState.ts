// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useTreeState - Core tree state management hook
 * 
 * Provides shared state primitives for tree expansion, loading, and caching.
 * Used by all container-specific tree hooks (AD1, VFS, Archive, UFED).
 */

import { createSignal, Accessor } from "solid-js";

/** Core state for tree expansion and loading */
export interface TreeStateReturn {
  // Expansion state
  expandedPaths: Accessor<Set<string>>;
  isExpanded: (key: string) => boolean;
  setExpanded: (key: string, expanded: boolean) => void;
  toggleExpanded: (key: string) => void;
  
  // Loading state
  loading: Accessor<Set<string>>;
  isLoading: (key: string) => boolean;
  setLoading: (key: string, isLoading: boolean) => void;
  
  // Selection state
  selectedKey: Accessor<string | null>;
  setSelectedKey: (key: string | null) => void;
  isSelected: (key: string) => boolean;
}

/**
 * Core tree state hook - provides expansion, loading, and selection primitives
 */
export function useTreeState(): TreeStateReturn {
  const [expandedPaths, setExpandedPaths] = createSignal<Set<string>>(new Set());
  const [loading, setLoadingState] = createSignal<Set<string>>(new Set());
  const [selectedKey, setSelectedKey] = createSignal<string | null>(null);
  
  const isExpanded = (key: string): boolean => {
    return expandedPaths().has(key);
  };
  
  const setExpanded = (key: string, expanded: boolean): void => {
    setExpandedPaths(prev => {
      const next = new Set(prev);
      if (expanded) {
        next.add(key);
      } else {
        next.delete(key);
      }
      return next;
    });
  };
  
  const toggleExpanded = (key: string): void => {
    setExpanded(key, !isExpanded(key));
  };
  
  const isLoading = (key: string): boolean => {
    return loading().has(key);
  };
  
  const setLoading = (key: string, isLoadingNow: boolean): void => {
    setLoadingState(prev => {
      const next = new Set(prev);
      if (isLoadingNow) {
        next.add(key);
      } else {
        next.delete(key);
      }
      return next;
    });
  };
  
  const isSelected = (key: string): boolean => {
    return selectedKey() === key;
  };
  
  return {
    expandedPaths,
    isExpanded,
    setExpanded,
    toggleExpanded,
    loading,
    isLoading,
    setLoading,
    selectedKey,
    setSelectedKey,
    isSelected,
  };
}

/** Cache state for storing tree entries */
export interface CacheStateReturn<T> {
  cache: Accessor<Map<string, T>>;
  get: (key: string) => T | undefined;
  set: (key: string, value: T) => void;
  has: (key: string) => boolean;
  append: (key: string, items: T extends Array<infer U> ? U[] : never) => void;
  clear: (key?: string) => void;
}

/**
 * Cache state hook - provides type-safe caching for tree entries
 */
export function useCacheState<T>(): CacheStateReturn<T> {
  const [cache, setCache] = createSignal<Map<string, T>>(new Map());
  
  const get = (key: string): T | undefined => {
    return cache().get(key);
  };
  
  const set = (key: string, value: T): void => {
    setCache(prev => {
      const next = new Map(prev);
      next.set(key, value);
      return next;
    });
  };
  
  const has = (key: string): boolean => {
    return cache().has(key);
  };
  
  // Type-safe append for array caches
  const append = (key: string, items: T extends Array<infer U> ? U[] : never): void => {
    setCache(prev => {
      const next = new Map(prev);
      const existing = (next.get(key) as unknown[] | undefined) || [];
      next.set(key, [...existing, ...items] as T);
      return next;
    });
  };
  
  const clear = (key?: string): void => {
    if (key) {
      setCache(prev => {
        const next = new Map(prev);
        next.delete(key);
        return next;
      });
    } else {
      setCache(new Map());
    }
  };
  
  return {
    cache,
    get,
    set,
    has,
    append,
    clear,
  };
}
