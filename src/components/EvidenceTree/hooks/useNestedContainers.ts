// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useNestedContainers - Hook for managing nested container expansion in evidence tree
 * 
 * Handles inline expansion of containers inside other containers (e.g., AD1 inside ZIP).
 * Uses caching to avoid re-extraction of nested containers.
 */

import { createSignal, Accessor } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { NestedContainerEntry, NestedContainerInfo } from "../../../types";

/** Cache key format: parentPath::nestedPath */
type NestedCacheKey = string;

export interface UseNestedContainersReturn {
  // State accessors
  nestedEntriesCache: Accessor<Map<NestedCacheKey, NestedContainerEntry[]>>;
  nestedInfoCache: Accessor<Map<NestedCacheKey, NestedContainerInfo>>;
  expandedNestedPaths: Accessor<Set<string>>;
  loadingNested: Accessor<Set<string>>;
  
  // Operations
  loadNestedContainerTree: (parentPath: string, nestedPath: string) => Promise<NestedContainerEntry[]>;
  loadNestedContainerInfo: (parentPath: string, nestedPath: string) => Promise<NestedContainerInfo | null>;
  toggleNestedContainer: (parentPath: string, nestedPath: string) => Promise<void>;
  clearNestedCache: () => Promise<void>;
  
  // Getters
  getNestedEntries: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  getNestedRootEntries: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  getNestedChildren: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  isNestedExpanded: (parentPath: string, nestedPath: string) => boolean;
  isNestedLoading: (parentPath: string, nestedPath: string) => boolean;
  hasNestedData: (parentPath: string, nestedPath: string) => boolean;
  
  // Utilities
  buildNestedKey: (parentPath: string, nestedPath: string) => string;
  sortNestedEntries: (entries: NestedContainerEntry[]) => NestedContainerEntry[];
}

/**
 * Hook for managing nested container tree state and operations
 */
export function useNestedContainers(): UseNestedContainersReturn {
  // Cache for nested container entries
  const [nestedEntriesCache, setNestedEntriesCache] = createSignal<Map<NestedCacheKey, NestedContainerEntry[]>>(new Map());
  
  // Cache for nested container info (quick metadata)
  const [nestedInfoCache, setNestedInfoCache] = createSignal<Map<NestedCacheKey, NestedContainerInfo>>(new Map());
  
  // Set of expanded nested container paths
  const [expandedNestedPaths, setExpandedNestedPaths] = createSignal<Set<string>>(new Set());
  
  // Loading states
  const [loadingNested, setLoadingNested] = createSignal<Set<string>>(new Set());
  
  // Build cache key from parent and nested paths
  const buildNestedKey = (parentPath: string, nestedPath: string): string => {
    return `${parentPath}::nested::${nestedPath}`;
  };
  
  // Sort entries: directories first, then by name
  const sortNestedEntries = (entries: NestedContainerEntry[]): NestedContainerEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  };

  // Synthesize virtual directory entries from file paths
  // This handles archives that don't include explicit directory entries (like Google Takeout)
  const synthesizeDirectories = (entries: NestedContainerEntry[]): NestedContainerEntry[] => {
    const existingPaths = new Set(entries.map(e => e.path.replace(/\/$/, '')));
    const syntheticDirs = new Map<string, NestedContainerEntry>();
    const dirSizes = new Map<string, number>();
    
    // First pass: build all synthetic directories and accumulate sizes
    for (const entry of entries) {
      const parts = entry.path.replace(/\/$/, '').split('/');
      // Build all ancestor directories
      for (let i = 1; i < parts.length; i++) {
        const dirPath = parts.slice(0, i).join('/');
        
        // Accumulate file sizes for this directory
        if (!entry.isDir) {
          dirSizes.set(dirPath, (dirSizes.get(dirPath) || 0) + entry.size);
        }
        
        if (!existingPaths.has(dirPath) && !syntheticDirs.has(dirPath)) {
          syntheticDirs.set(dirPath, {
            path: dirPath + '/',
            name: parts[i - 1],
            isDir: true,
            size: 0, // Will be set below
            hash: undefined,
            modified: undefined,
            sourceType: entry.sourceType,
            isNestedContainer: false,
            nestedType: null,
          });
        }
      }
    }
    
    // Second pass: set directory sizes
    for (const [dirPath, dir] of syntheticDirs) {
      dir.size = dirSizes.get(dirPath) || 0;
    }
    
    // Also update sizes for existing directories
    const result = entries.map(e => {
      if (e.isDir) {
        const dirPath = e.path.replace(/\/$/, '');
        const calcSize = dirSizes.get(dirPath);
        if (calcSize && calcSize > e.size) {
          return { ...e, size: calcSize };
        }
      }
      return e;
    });
    
    return [...result, ...syntheticDirs.values()];
  };
  
  // Load nested container tree
  const loadNestedContainerTree = async (parentPath: string, nestedPath: string): Promise<NestedContainerEntry[]> => {
    const cacheKey = buildNestedKey(parentPath, nestedPath);
    
    // Check cache first
    const cached = nestedEntriesCache().get(cacheKey);
    if (cached) return cached;
    
    // Set loading state
    setLoadingNested(prev => new Set([...prev, cacheKey]));
    
    try {
      const entries = await invoke<NestedContainerEntry[]>("nested_container_get_tree", {
        parentContainerPath: parentPath,
        nestedEntryPath: nestedPath,
      });
      
      // Cache the result
      setNestedEntriesCache(prev => {
        const next = new Map(prev);
        next.set(cacheKey, entries);
        return next;
      });
      
      return entries;
    } catch (err) {
      console.error("[useNestedContainers] Failed to load nested tree:", err);
      return [];
    } finally {
      setLoadingNested(prev => {
        const next = new Set(prev);
        next.delete(cacheKey);
        return next;
      });
    }
  };
  
  // Load nested container info (quick metadata)
  const loadNestedContainerInfo = async (parentPath: string, nestedPath: string): Promise<NestedContainerInfo | null> => {
    const cacheKey = buildNestedKey(parentPath, nestedPath);
    
    // Check cache first
    const cached = nestedInfoCache().get(cacheKey);
    if (cached) return cached;
    
    try {
      const info = await invoke<NestedContainerInfo>("nested_container_get_info", {
        parentContainerPath: parentPath,
        nestedEntryPath: nestedPath,
      });
      
      // Cache the result
      setNestedInfoCache(prev => {
        const next = new Map(prev);
        next.set(cacheKey, info);
        return next;
      });
      
      return info;
    } catch (err) {
      console.error("[useNestedContainers] Failed to load nested info:", err);
      return null;
    }
  };
  
  // Toggle nested container expansion
  const toggleNestedContainer = async (parentPath: string, nestedPath: string): Promise<void> => {
    const cacheKey = buildNestedKey(parentPath, nestedPath);
    
    const expanded = expandedNestedPaths();
    if (expanded.has(cacheKey)) {
      // Collapse
      setExpandedNestedPaths(prev => {
        const next = new Set(prev);
        next.delete(cacheKey);
        return next;
      });
    } else {
      // Expand - load data if not cached
      if (!nestedEntriesCache().has(cacheKey)) {
        await loadNestedContainerTree(parentPath, nestedPath);
      }
      setExpandedNestedPaths(prev => new Set([...prev, cacheKey]));
    }
  };
  
  // Clear all nested container cache (called on app cleanup)
  const clearNestedCache = async (): Promise<void> => {
    try {
      await invoke("nested_container_clear_cache");
      setNestedEntriesCache(new Map());
      setNestedInfoCache(new Map());
      setExpandedNestedPaths(new Set<string>());
    } catch (err) {
      console.error("[useNestedContainers] Failed to clear cache:", err);
    }
  };
  
  // Get cached entries (raw, without synthesis)
  const getNestedEntries = (parentPath: string, nestedPath: string): NestedContainerEntry[] => {
    const cacheKey = buildNestedKey(parentPath, nestedPath);
    return nestedEntriesCache().get(cacheKey) || [];
  };

  // Get all entries with synthesized directories
  const getNestedEntriesWithDirs = (parentPath: string, nestedPath: string): NestedContainerEntry[] => {
    const entries = getNestedEntries(parentPath, nestedPath);
    return synthesizeDirectories(entries);
  };
  
  // Get root-level entries (with synthesized directories)
  const getNestedRootEntries = (parentPath: string, nestedPath: string): NestedContainerEntry[] => {
    const entries = getNestedEntriesWithDirs(parentPath, nestedPath);
    return entries.filter(entry => {
      const pathParts = entry.path.replace(/\/$/, '').split('/').filter(p => p);
      return pathParts.length === 1;
    });
  };
  
  // Get children of a specific path within the nested container (with synthesized directories)
  const getNestedChildren = (parentPath: string, nestedPath: string, entryPath: string): NestedContainerEntry[] => {
    const entries = getNestedEntriesWithDirs(parentPath, nestedPath);
    const normalizedParent = entryPath.replace(/\/$/, '');
    
    return entries.filter(entry => {
      const entryPathNorm = entry.path.replace(/\/$/, '');
      if (!entryPathNorm.startsWith(normalizedParent + '/')) return false;
      const remaining = entryPathNorm.substring(normalizedParent.length + 1);
      return !remaining.includes('/');
    });
  };
  
  // Check if nested container is expanded
  const isNestedExpanded = (parentPath: string, nestedPath: string): boolean => {
    const cacheKey = buildNestedKey(parentPath, nestedPath);
    return expandedNestedPaths().has(cacheKey);
  };
  
  // Check if nested container is loading
  const isNestedLoading = (parentPath: string, nestedPath: string): boolean => {
    const cacheKey = buildNestedKey(parentPath, nestedPath);
    return loadingNested().has(cacheKey);
  };
  
  // Check if we have data for a nested container
  const hasNestedData = (parentPath: string, nestedPath: string): boolean => {
    const cacheKey = buildNestedKey(parentPath, nestedPath);
    return nestedEntriesCache().has(cacheKey);
  };
  
  return {
    nestedEntriesCache,
    nestedInfoCache,
    expandedNestedPaths,
    loadingNested,
    loadNestedContainerTree,
    loadNestedContainerInfo,
    toggleNestedContainer,
    clearNestedCache,
    getNestedEntries,
    getNestedRootEntries,
    getNestedChildren,
    isNestedExpanded,
    isNestedLoading,
    hasNestedData,
    buildNestedKey,
    sortNestedEntries,
  };
}
