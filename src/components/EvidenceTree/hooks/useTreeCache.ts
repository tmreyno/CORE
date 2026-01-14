// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useTreeCache - Manages cached children for tree nodes
 * 
 * Provides caching for loaded children to avoid re-fetching
 * when nodes are collapsed and re-expanded.
 */

import { createSignal } from "solid-js";

export interface UseTreeCacheReturn<T> {
  /** Cache of children by key */
  cache: () => Map<string, T[]>;
  
  /** Get cached children for a key */
  get: (key: string) => T[] | undefined;
  
  /** Set children for a key */
  set: (key: string, children: T[]) => void;
  
  /** Append children for a key (for pagination) */
  append: (key: string, children: T[]) => void;
  
  /** Check if key has cached children */
  has: (key: string) => boolean;
  
  /** Clear cache for a key */
  clear: (key: string) => void;
  
  /** Clear all cache */
  clearAll: () => void;
}

export function useTreeCache<T>(): UseTreeCacheReturn<T> {
  const [cache, setCache] = createSignal<Map<string, T[]>>(new Map());

  const get = (key: string): T[] | undefined => {
    return cache().get(key);
  };

  const set = (key: string, children: T[]) => {
    setCache(prev => {
      const next = new Map(prev);
      next.set(key, children);
      return next;
    });
  };

  const append = (key: string, children: T[]) => {
    setCache(prev => {
      const next = new Map(prev);
      const existing = next.get(key) || [];
      next.set(key, [...existing, ...children]);
      return next;
    });
  };

  const has = (key: string): boolean => {
    return cache().has(key);
  };

  const clear = (key: string) => {
    setCache(prev => {
      const next = new Map(prev);
      next.delete(key);
      return next;
    });
  };

  const clearAll = () => {
    setCache(new Map<string, T[]>());
  };

  return {
    cache,
    get,
    set,
    append,
    has,
    clear,
    clearAll,
  };
}
