// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useLazyLoading - React/Solid hook for unified lazy loading across all containers
 * 
 * This hook provides a unified API for lazy loading evidence tree data from
 * any supported container type (AD1, E01, UFED, ZIP, etc.).
 * 
 * Features:
 * - Automatic container type detection
 * - Configurable batch sizes and thresholds
 * - Pagination support for large directories
 * - Caching of loaded entries
 * - Loading state management
 * 
 * @example
 * ```tsx
 * const { 
 *   summary, 
 *   rootChildren, 
 *   loadChildren, 
 *   isLoading 
 * } = useLazyLoading(containerPath);
 * 
 * // Check if lazy loading is recommended
 * if (summary()?.lazy_loading_recommended) {
 *   // Load root entries
 *   await loadRootChildren();
 * }
 * 
 * // Load children when folder is expanded
 * await loadChildren(folderPath);
 * ```
 */

import { createSignal, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";
import type { 
  LazyLoadConfig, 
  LazyTreeEntry, 
  LazyLoadResult, 
  ContainerSummary
} from "../types/lazy-loading";

const log = logger.scope("LazyLoading");

// =============================================================================
// TYPES
// =============================================================================

export interface UseLazyLoadingOptions {
  /** Initial batch size override */
  batchSize?: number;
  /** Whether to auto-load root children on mount */
  autoLoad?: boolean;
  /** Callback when loading starts */
  onLoadStart?: () => void;
  /** Callback when loading completes */
  onLoadComplete?: (result: LazyLoadResult) => void;
  /** Callback on error */
  onError?: (error: Error) => void;
}

export interface UseLazyLoadingReturn {
  // State
  /** Container summary (entry count, lazy loading recommendation) */
  summary: () => ContainerSummary | null;
  /** Root-level children */
  rootChildren: () => LazyTreeEntry[];
  /** Total count of root children */
  rootTotalCount: () => number;
  /** Whether root children are loading */
  isLoadingRoot: () => boolean;
  /** Whether any operation is in progress */
  isLoading: () => boolean;
  /** Current error if any */
  error: () => Error | null;
  /** Current config */
  config: () => LazyLoadConfig;
  /** Cache of children by parent path */
  childrenCache: () => Map<string, LazyTreeEntry[]>;
  /** Whether there are more root entries to load */
  hasMoreRoot: () => boolean;
  
  // Actions
  /** Load container summary */
  loadSummary: () => Promise<ContainerSummary | null>;
  /** Load root-level children */
  loadRootChildren: (offset?: number, limit?: number) => Promise<LazyLoadResult | null>;
  /** Load more root children (pagination) */
  loadMoreRoot: () => Promise<LazyLoadResult | null>;
  /** Load children at a specific path */
  loadChildren: (parentPath: string, offset?: number, limit?: number) => Promise<LazyLoadResult | null>;
  /** Get cached children for a path */
  getCachedChildren: (parentPath: string) => LazyTreeEntry[] | undefined;
  /** Check if children are cached for a path */
  hasChildren: (parentPath: string) => boolean;
  /** Clear all caches */
  clearCache: () => void;
  /** Update lazy loading settings */
  updateSettings: (settings: Partial<LazyLoadConfig>) => Promise<LazyLoadConfig>;
  /** Refresh settings from backend */
  refreshSettings: () => Promise<LazyLoadConfig>;
}

// =============================================================================
// HOOK IMPLEMENTATION
// =============================================================================

export function useLazyLoading(
  containerPath: () => string | null,
  options: UseLazyLoadingOptions = {}
): UseLazyLoadingReturn {
  // === State ===
  const [summary, setSummary] = createSignal<ContainerSummary | null>(null);
  const [rootChildren, setRootChildren] = createSignal<LazyTreeEntry[]>([]);
  const [childrenCache, setChildrenCache] = createSignal<Map<string, LazyTreeEntry[]>>(new Map());
  const [isLoadingRoot, setIsLoadingRoot] = createSignal(false);
  const [isLoadingChildren, setIsLoadingChildren] = createSignal<Set<string>>(new Set());
  const [error, setError] = createSignal<Error | null>(null);
  const [config, setConfig] = createSignal<LazyLoadConfig>({
    enabled: true,
    batch_size: options.batchSize ?? 100,
    auto_expand_threshold: 50,
    large_container_threshold: 10_000,
    pagination_threshold: 500,
    show_entry_count: true,
    count_timeout_ms: 5_000,
    load_timeout_ms: 30_000,
  });
  const [rootOffset, setRootOffset] = createSignal(0);
  const [rootTotalCount, setRootTotalCount] = createSignal(0);
  const [hasMoreRoot, setHasMoreRoot] = createSignal(false);

  // === Computed ===
  const isLoading = () => isLoadingRoot() || isLoadingChildren().size > 0;

  // === Actions ===

  /**
   * Load container summary to determine if lazy loading should be used
   */
  const loadSummary = async (): Promise<ContainerSummary | null> => {
    const path = containerPath();
    if (!path) return null;

    try {
      setError(null);
      const result = await invoke<ContainerSummary>("lazy_get_container_summary", {
        containerPath: path,
      });
      setSummary(result);
      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      log.error("loadSummary failed:", err);
      return null;
    }
  };

  /**
   * Load root-level children with optional pagination
   */
  const loadRootChildren = async (
    offset?: number, 
    limit?: number
  ): Promise<LazyLoadResult | null> => {
    const path = containerPath();
    if (!path) return null;

    try {
      setError(null);
      setIsLoadingRoot(true);
      options.onLoadStart?.();

      const result = await invoke<LazyLoadResult>("lazy_get_root_children", {
        containerPath: path,
        offset: offset ?? 0,
        limit: limit ?? config().batch_size,
      });

      // Update state based on offset
      if (offset === 0 || offset === undefined) {
        // Fresh load - replace entries
        setRootChildren(result.entries);
      } else {
        // Pagination - append entries
        setRootChildren(prev => [...prev, ...result.entries]);
      }

      setRootOffset(result.next_offset);
      setRootTotalCount(result.total_count);
      setHasMoreRoot(result.has_more);
      setConfig(result.config);

      options.onLoadComplete?.(result);
      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      log.error("loadRootChildren failed:", err);
      return null;
    } finally {
      setIsLoadingRoot(false);
    }
  };

  /**
   * Load more root children (pagination)
   */
  const loadMoreRoot = async (): Promise<LazyLoadResult | null> => {
    if (!hasMoreRoot()) return null;
    return loadRootChildren(rootOffset(), config().batch_size);
  };

  /**
   * Load children at a specific path
   */
  const loadChildren = async (
    parentPath: string,
    offset?: number,
    limit?: number
  ): Promise<LazyLoadResult | null> => {
    const path = containerPath();
    if (!path) return null;

    try {
      setError(null);
      
      // Track loading state
      setIsLoadingChildren(prev => {
        const next = new Set(prev);
        next.add(parentPath);
        return next;
      });

      const result = await invoke<LazyLoadResult>("lazy_get_children", {
        containerPath: path,
        parentPath,
        offset: offset ?? 0,
        limit: limit ?? config().batch_size,
      });

      // Update cache
      setChildrenCache(prev => {
        const next = new Map(prev);
        if (offset === 0 || offset === undefined) {
          // Fresh load
          next.set(parentPath, result.entries);
        } else {
          // Pagination - append
          const existing = prev.get(parentPath) || [];
          next.set(parentPath, [...existing, ...result.entries]);
        }
        return next;
      });

      setConfig(result.config);
      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      log.error("loadChildren failed:", err);
      return null;
    } finally {
      setIsLoadingChildren(prev => {
        const next = new Set(prev);
        next.delete(parentPath);
        return next;
      });
    }
  };

  /**
   * Get cached children for a path
   */
  const getCachedChildren = (parentPath: string): LazyTreeEntry[] | undefined => {
    return childrenCache().get(parentPath);
  };

  /**
   * Check if children are cached for a path
   */
  const hasChildren = (parentPath: string): boolean => {
    return childrenCache().has(parentPath);
  };

  /**
   * Clear all caches
   */
  const clearCache = (): void => {
    setRootChildren([]);
    setChildrenCache(new Map());
    setSummary(null);
    setRootOffset(0);
    setRootTotalCount(0);
    setHasMoreRoot(false);
  };

  /**
   * Update lazy loading settings
   */
  const updateSettings = async (
    settings: Partial<LazyLoadConfig>
  ): Promise<LazyLoadConfig> => {
    try {
      const result = await invoke<LazyLoadConfig>("lazy_update_settings", {
        batchSize: settings.batch_size,
        largeContainerThreshold: settings.large_container_threshold,
        paginationThreshold: settings.pagination_threshold,
      });
      setConfig(result);
      return result;
    } catch (err) {
      log.error("updateSettings failed:", err);
      throw err;
    }
  };

  /**
   * Refresh settings from backend
   */
  const refreshSettings = async (): Promise<LazyLoadConfig> => {
    try {
      const result = await invoke<LazyLoadConfig>("lazy_get_settings");
      setConfig(result);
      return result;
    } catch (err) {
      log.error("refreshSettings failed:", err);
      throw err;
    }
  };

  // === Effects ===

  // Clear cache when container path changes
  createEffect(() => {
    const path = containerPath();
    if (path) {
      clearCache();
      // Auto-load if enabled
      if (options.autoLoad) {
        loadSummary().then(summary => {
          if (summary) {
            loadRootChildren();
          }
        });
      }
    }
  });

  // Load settings on mount
  createEffect(() => {
    refreshSettings().catch(() => {
      // Ignore - use defaults
    });
  });

  return {
    // State
    summary,
    rootChildren,
    rootTotalCount,
    isLoadingRoot,
    isLoading,
    error,
    config,
    childrenCache,
    hasMoreRoot,
    
    // Actions
    loadSummary,
    loadRootChildren,
    loadMoreRoot,
    loadChildren,
    getCachedChildren,
    hasChildren,
    clearCache,
    updateSettings,
    refreshSettings,
  };
}

// =============================================================================
// STANDALONE FUNCTIONS
// =============================================================================

/**
 * Get container summary (standalone function for one-off calls)
 */
export async function getContainerSummary(
  containerPath: string
): Promise<ContainerSummary> {
  return invoke<ContainerSummary>("lazy_get_container_summary", {
    containerPath,
  });
}

/**
 * Get root children (standalone function for one-off calls)
 */
export async function getRootChildren(
  containerPath: string,
  offset?: number,
  limit?: number
): Promise<LazyLoadResult> {
  return invoke<LazyLoadResult>("lazy_get_root_children", {
    containerPath,
    offset,
    limit,
  });
}

/**
 * Get children at path (standalone function for one-off calls)
 */
export async function getChildren(
  containerPath: string,
  parentPath: string,
  offset?: number,
  limit?: number
): Promise<LazyLoadResult> {
  return invoke<LazyLoadResult>("lazy_get_children", {
    containerPath,
    parentPath,
    offset,
    limit,
  });
}

/**
 * Get current lazy loading settings
 */
export async function getLazyLoadSettings(): Promise<LazyLoadConfig> {
  return invoke<LazyLoadConfig>("lazy_get_settings");
}

/**
 * Update lazy loading settings
 */
export async function updateLazyLoadSettings(
  settings: Partial<{
    batchSize: number;
    largeContainerThreshold: number;
    paginationThreshold: number;
  }>
): Promise<LazyLoadConfig> {
  return invoke<LazyLoadConfig>("lazy_update_settings", settings);
}

export default useLazyLoading;
