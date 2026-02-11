// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useLazyTree - Lazy loading tree operations hook
 * 
 * Manages state and operations for lazy-loading large containers (UFED, large archives).
 * Supports pagination and progressive loading for optimal performance with 10k+ entries.
 */

import { createSignal, Accessor } from "solid-js";
import type { UfedTreeEntry } from "../../../types";
import type { LazyTreeEntry, ContainerSummary } from "../../../types/lazy-loading";
import { getContainerSummary, getRootChildren, getChildren } from "../../../hooks/useLazyLoading";
import { logger } from "../../../utils/logger";

const log = logger.scope("LazyTree");

export interface UseLazyTreeReturn {
  // State accessors
  lazySummaryCache: Accessor<Map<string, ContainerSummary>>;
  lazyChildrenCache: Accessor<Map<string, LazyTreeEntry[]>>;
  lazyTotalCounts: Accessor<Map<string, number>>;
  lazyHasMore: Accessor<Map<string, boolean>>;
  expandedLazyPaths: Accessor<Set<string>>;
  ufedTreeCache: Accessor<Map<string, UfedTreeEntry[]>>;
  
  // Operations
  loadLazySummary: (containerPath: string) => Promise<ContainerSummary | null>;
  loadLazyRootChildren: (containerPath: string, offset?: number, limit?: number) => Promise<LazyTreeEntry[]>;
  loadLazyChildren: (containerPath: string, parentPath: string, offset?: number, limit?: number) => Promise<LazyTreeEntry[]>;
  toggleLazyDir: (containerPath: string, entryPath: string, loading: Set<string>, setLoading: (fn: (prev: Set<string>) => Set<string>) => void) => Promise<void>;
  loadMoreLazyEntries: (containerPath: string, parentPath: string, loading: Set<string>, setLoading: (fn: (prev: Set<string>) => Set<string>) => void) => Promise<void>;
  
  // Getters
  hasMoreLazyEntries: (containerPath: string, parentPath?: string) => boolean;
  getLazyTotalCount: (containerPath: string, parentPath?: string) => number;
  isLoadingMoreLazy: (containerPath: string, parentPath: string, loading: Set<string>) => boolean;
  isLazyDirExpanded: (containerPath: string, entryPath: string) => boolean;
  
  // Utilities
  sortLazyEntries: (entries: LazyTreeEntry[]) => LazyTreeEntry[];
  sortUfedEntries: (entries: UfedTreeEntry[]) => UfedTreeEntry[];
  
  // Expand/Collapse all
  expandAllLazyDirs: (containerPath: string) => Promise<void>;
  collapseAllLazyDirs: () => void;
  
  // State persistence
  restoreExpandedPaths: (paths: string[]) => void;
}

/**
 * Hook for managing lazy-loaded container tree state and operations
 */
export function useLazyTree(): UseLazyTreeReturn {
  log.debug("Hook initialized");
  
  // Cache container summaries
  const [lazySummaryCache, setLazySummaryCache] = createSignal<Map<string, ContainerSummary>>(new Map());
  // Cache lazy-loaded children
  const [lazyChildrenCache, setLazyChildrenCache] = createSignal<Map<string, LazyTreeEntry[]>>(new Map());
  // Track total counts for pagination
  const [lazyTotalCounts, setLazyTotalCounts] = createSignal<Map<string, number>>(new Map());
  // Track if there are more entries to load
  const [lazyHasMore, setLazyHasMore] = createSignal<Map<string, boolean>>(new Map());
  // Track expanded lazy paths
  const [expandedLazyPaths, setExpandedLazyPaths] = createSignal<Set<string>>(new Set());
  // Legacy UFED cache (fallback)
  const [ufedTreeCache] = createSignal<Map<string, UfedTreeEntry[]>>(new Map());

  // Get container summary with lazy loading recommendation
  const loadLazySummary = async (containerPath: string): Promise<ContainerSummary | null> => {
    log.debug(`loadLazySummary called for ${containerPath}`);
    const cached = lazySummaryCache().get(containerPath);
    if (cached) {
      log.debug(`loadLazySummary - returning cached summary, entryCount=${cached.entry_count}`);
      return cached;
    }
    
    try {
      log.debug("loadLazySummary - fetching container summary");
      const summary = await getContainerSummary(containerPath);
      log.debug(`loadLazySummary - got summary: entryCount=${summary.entry_count}, totalSize=${summary.total_size}`);
      
      setLazySummaryCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, summary);
        return next;
      });
      return summary;
    } catch (err) {
      log.error("loadLazySummary FAILED:", err);
      return null;
    }
  };
  
  // Load lazy root children with pagination
  const loadLazyRootChildren = async (
    containerPath: string, 
    offset: number = 0, 
    limit: number = 100
  ): Promise<LazyTreeEntry[]> => {
    log.debug(`loadLazyRootChildren called, path=${containerPath}, offset=${offset}, limit=${limit}`);
    const cacheKey = `${containerPath}::lazy::root`;
    
    // If offset is 0, check cache first
    if (offset === 0) {
      const cached = lazyChildrenCache().get(cacheKey);
      if (cached && cached.length > 0) {
        log.debug(`loadLazyRootChildren - returning ${cached.length} cached entries`);
        return cached;
      }
    }
    
    try {
      const result = await getRootChildren(containerPath, offset, limit);
      
      // Update total count
      setLazyTotalCounts(prev => {
        const next = new Map(prev);
        next.set(cacheKey, result.total_count);
        return next;
      });
      
      // Update has_more
      setLazyHasMore(prev => {
        const next = new Map(prev);
        next.set(cacheKey, result.has_more);
        return next;
      });
      
      // Append to cache (for pagination) or set (for initial load)
      setLazyChildrenCache(prev => {
        const next = new Map(prev);
        const existing = offset > 0 ? (next.get(cacheKey) || []) : [];
        next.set(cacheKey, [...existing, ...result.entries]);
        return next;
      });
      
      return result.entries;
    } catch (err) {
      console.error("[loadLazyRootChildren] Failed:", err);
      return [];
    }
  };
  
  // Load lazy children at a specific path with pagination
  const loadLazyChildren = async (
    containerPath: string,
    parentPath: string,
    offset: number = 0,
    limit: number = 100
  ): Promise<LazyTreeEntry[]> => {
    const cacheKey = `${containerPath}::lazy::${parentPath}`;
    
    // If offset is 0, check cache first
    if (offset === 0) {
      const cached = lazyChildrenCache().get(cacheKey);
      if (cached && cached.length > 0) return cached;
    }
    
    try {
      const result = await getChildren(containerPath, parentPath, offset, limit);
      
      // Update total count
      setLazyTotalCounts(prev => {
        const next = new Map(prev);
        next.set(cacheKey, result.total_count);
        return next;
      });
      
      // Update has_more
      setLazyHasMore(prev => {
        const next = new Map(prev);
        next.set(cacheKey, result.has_more);
        return next;
      });
      
      // Append to cache
      setLazyChildrenCache(prev => {
        const next = new Map(prev);
        const existing = offset > 0 ? (next.get(cacheKey) || []) : [];
        next.set(cacheKey, [...existing, ...result.entries]);
        return next;
      });
      
      return result.entries;
    } catch (err) {
      console.error("[loadLazyChildren] Failed:", err);
      return [];
    }
  };
  
  // Sort lazy entries: directories first, then alphabetically
  const sortLazyEntries = (entries: LazyTreeEntry[]): LazyTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };

  // Sort UFED entries: directories first, then alphabetically
  const sortUfedEntries = (entries: UfedTreeEntry[]): UfedTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  // Toggle lazy directory expansion
  const toggleLazyDir = async (
    containerPath: string, 
    entryPath: string,
    _loading: Set<string>,
    setLoading: (fn: (prev: Set<string>) => Set<string>) => void
  ): Promise<void> => {
    const nodeKey = `${containerPath}::lazy::${entryPath}`;
    const expanded = new Set(expandedLazyPaths());
    
    if (expanded.has(nodeKey)) {
      expanded.delete(nodeKey);
      setExpandedLazyPaths(new Set(expanded));
    } else {
      // Load children if not cached
      if (!lazyChildrenCache().has(nodeKey)) {
        setLoading(prev => new Set([...prev, nodeKey]));
        await loadLazyChildren(containerPath, entryPath);
        setLoading(prev => {
          const next = new Set(prev);
          next.delete(nodeKey);
          return next;
        });
      }
      expanded.add(nodeKey);
      setExpandedLazyPaths(new Set(expanded));
    }
  };
  
  // Load more entries for a lazy path (pagination)
  const loadMoreLazyEntries = async (
    containerPath: string, 
    parentPath: string = "root",
    _loading: Set<string>,
    setLoading: (fn: (prev: Set<string>) => Set<string>) => void
  ): Promise<void> => {
    const cacheKey = parentPath === "root"
      ? `${containerPath}::lazy::root`
      : `${containerPath}::lazy::${parentPath}`;
    
    const currentEntries = lazyChildrenCache().get(cacheKey) || [];
    const offset = currentEntries.length;
    
    setLoading(prev => new Set([...prev, `${cacheKey}::more`]));
    
    if (parentPath === "root") {
      await loadLazyRootChildren(containerPath, offset, 100);
    } else {
      await loadLazyChildren(containerPath, parentPath, offset, 100);
    }
    
    setLoading(prev => {
      const next = new Set(prev);
      next.delete(`${cacheKey}::more`);
      return next;
    });
  };
  
  // Check if currently loading more for a lazy path
  const isLoadingMoreLazy = (containerPath: string, parentPath: string = "root", loading: Set<string>): boolean => {
    const cacheKey = parentPath === "root"
      ? `${containerPath}::lazy::root`
      : `${containerPath}::lazy::${parentPath}`;
    return loading.has(`${cacheKey}::more`);
  };
  
  // Check if there are more entries to load
  const hasMoreLazyEntries = (containerPath: string, parentPath: string = "root"): boolean => {
    const cacheKey = parentPath === "root"
      ? `${containerPath}::lazy::root`
      : `${containerPath}::lazy::${parentPath}`;
    return lazyHasMore().get(cacheKey) || false;
  };
  
  // Get total count for a lazy path
  const getLazyTotalCount = (containerPath: string, parentPath: string = "root"): number => {
    const cacheKey = parentPath === "root"
      ? `${containerPath}::lazy::root`
      : `${containerPath}::lazy::${parentPath}`;
    return lazyTotalCounts().get(cacheKey) || 0;
  };

  // Check if lazy directory is expanded
  const isLazyDirExpanded = (containerPath: string, entryPath: string): boolean => {
    const nodeKey = `${containerPath}::lazy::${entryPath}`;
    return expandedLazyPaths().has(nodeKey);
  };

  // Expand all lazy directories (root level only)
  const expandAllLazyDirs = async (containerPath: string): Promise<void> => {
    const rootKey = `${containerPath}::lazy::root`;
    const rootChildren = lazyChildrenCache().get(rootKey) || [];
    
    // Expand root-level directories
    const keysToExpand: string[] = [];
    for (const entry of rootChildren) {
      if (entry.is_dir) {
        const nodeKey = `${containerPath}::lazy::${entry.path}`;
        keysToExpand.push(nodeKey);
      }
    }
    
    if (keysToExpand.length > 0) {
      setExpandedLazyPaths(prev => {
        const next = new Set(prev);
        keysToExpand.forEach(key => next.add(key));
        return next;
      });
    }
  };

  // Collapse all lazy directories
  const collapseAllLazyDirs = (): void => {
    setExpandedLazyPaths(new Set<string>());
  };

  // Restore expanded paths from serialized state
  const restoreExpandedPaths = (paths: string[]): void => {
    setExpandedLazyPaths(new Set(paths));
  };

  return {
    lazySummaryCache,
    lazyChildrenCache,
    lazyTotalCounts,
    lazyHasMore,
    expandedLazyPaths,
    ufedTreeCache,
    loadLazySummary,
    loadLazyRootChildren,
    loadLazyChildren,
    toggleLazyDir,
    loadMoreLazyEntries,
    hasMoreLazyEntries,
    getLazyTotalCount,
    isLoadingMoreLazy,
    isLazyDirExpanded,
    sortLazyEntries,
    sortUfedEntries,
    expandAllLazyDirs,
    collapseAllLazyDirs,
    restoreExpandedPaths,
  };
}
