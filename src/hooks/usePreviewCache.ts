// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Preview Cache Hook
 * 
 * Tracks extracted preview files to avoid re-extraction when switching between
 * files in the evidence tree. The cache persists in project save data.
 */

import { createSignal } from "solid-js";
import type { PreviewCache, PreviewCacheEntry } from "../types/project";
import { nowISO } from "../types/project";

/** Create a cache key from container and entry paths */
export function createCacheKey(containerPath: string, entryPath: string): string {
  return `${containerPath}::${entryPath}`;
}

/**
 * Hook for managing preview file cache
 */
export function usePreviewCache() {
  const [cache, setCache] = createSignal<Map<string, PreviewCacheEntry>>(new Map());
  const [cacheDir, setCacheDir] = createSignal<string | null>(null);

  /**
   * Get a cached preview path if it exists and is valid
   */
  const getCachedPath = (containerPath: string, entryPath: string): string | null => {
    const key = createCacheKey(containerPath, entryPath);
    const entry = cache().get(key);
    
    if (entry && entry.valid !== false) {
      return entry.temp_path;
    }
    return null;
  };

  /**
   * Add an extracted file to the cache
   */
  const addToCache = (
    containerPath: string,
    entryPath: string,
    tempPath: string,
    entrySize: number
  ) => {
    const key = createCacheKey(containerPath, entryPath);
    const entry: PreviewCacheEntry = {
      key,
      container_path: containerPath,
      entry_path: entryPath,
      temp_path: tempPath,
      entry_size: entrySize,
      extracted_at: nowISO(),
      valid: true,
    };

    setCache(prev => {
      const next = new Map(prev);
      next.set(key, entry);
      return next;
    });
  };

  /**
   * Remove an entry from the cache
   */
  const removeFromCache = (containerPath: string, entryPath: string) => {
    const key = createCacheKey(containerPath, entryPath);
    setCache(prev => {
      const next = new Map(prev);
      next.delete(key);
      return next;
    });
  };

  /**
   * Clear all cache entries
   */
  const clearCache = () => {
    setCache(new Map());
  };

  /**
   * Export cache state for project save
   */
  const exportCache = (): PreviewCache => {
    const entries = Array.from(cache().values());
    return {
      entries,
      cached_at: nowISO(),
      cache_dir: cacheDir() || undefined,
    };
  };

  /**
   * Import cache state from loaded project
   */
  const importCache = (projectCache: PreviewCache | undefined) => {
    if (!projectCache || !projectCache.entries) {
      clearCache();
      return;
    }

    const newCache = new Map<string, PreviewCacheEntry>();
    for (const entry of projectCache.entries) {
      // Mark as potentially invalid - will be validated on use
      newCache.set(entry.key, { ...entry, valid: undefined });
    }
    setCache(newCache);
    setCacheDir(projectCache.cache_dir || null);
  };

  /**
   * Get cache statistics
   */
  const getStats = () => {
    const entries = Array.from(cache().values());
    return {
      count: entries.length,
      totalSize: entries.reduce((sum, e) => sum + e.entry_size, 0),
    };
  };

  return {
    // Getters
    getCachedPath,
    getStats,
    
    // Mutations
    addToCache,
    removeFromCache,
    clearCache,
    
    // Project save/load
    exportCache,
    importCache,
    
    // Cache directory
    cacheDir,
    setCacheDir,
  };
}

export type PreviewCacheManager = ReturnType<typeof usePreviewCache>;
