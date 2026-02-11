// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useUnifiedContainer - React/Solid hook for unified container access
 * 
 * This hook provides a single, unified API for accessing data from ANY
 * container type (AD1, E01, UFED, ZIP, 7z, TAR, RAR, RAW).
 * 
 * Key Features:
 * - Automatic container type detection
 * - Type-safe dispatch to appropriate handler
 * - Lazy loading with pagination support
 * - Caching of loaded entries
 * - Loading and error state management
 * 
 * This replaces the fragmented approach of having separate hooks/commands
 * for each container type.
 * 
 * @example
 * ```tsx
 * const {
 *   summary,
 *   rootChildren,
 *   loadRootChildren,
 *   loadChildren,
 *   isLoading,
 * } = useUnifiedContainer(() => containerPath);
 * 
 * // Load summary first to check container info
 * await loadSummary();
 * 
 * // Load root-level entries
 * await loadRootChildren();
 * 
 * // Load children when folder is expanded
 * await loadChildren('/path/to/folder');
 * ```
 */

import { createSignal, createEffect, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  ContainerSummary,
  LazyLoadResult,
  LazyLoadConfig,
  FileEntry,
  ContainerType,
} from "../types/lazy-loading";
import { DEFAULT_LAZY_LOAD_CONFIG } from "../types/lazy-loading";
import { logger } from "../utils/logger";
const log = logger.scope("UnifiedContainer");

// =============================================================================
// TYPES
// =============================================================================

export interface UseUnifiedContainerOptions {
  /** Initial batch size override */
  batchSize?: number;
  /** Whether to auto-load summary on mount */
  autoLoadSummary?: boolean;
  /** Whether to auto-load root children after summary */
  autoLoadRoot?: boolean;
  /** Callback when loading starts */
  onLoadStart?: () => void;
  /** Callback when loading completes */
  onLoadComplete?: (result: LazyLoadResult) => void;
  /** Callback on error */
  onError?: (error: Error) => void;
}

export interface UseUnifiedContainerReturn {
  // === State ===
  /** Container summary (type, entry count, size, etc.) */
  summary: () => ContainerSummary | null;
  /** Detected container type */
  containerType: () => ContainerType | null;
  /** Root-level entries */
  rootChildren: () => FileEntry[];
  /** Total count of root entries */
  rootTotalCount: () => number;
  /** Whether there are more root entries to load */
  hasMoreRoot: () => boolean;
  /** Whether root entries are loading */
  isLoadingRoot: () => boolean;
  /** Whether any operation is in progress */
  isLoading: () => boolean;
  /** Current error if any */
  error: () => Error | null;
  /** Current config settings */
  config: () => LazyLoadConfig;
  /** Cache of children by parent path */
  childrenCache: () => Map<string, FileEntry[]>;
  /** Set of paths currently loading */
  loadingPaths: () => Set<string>;

  // === Actions ===
  /** Load container summary */
  loadSummary: () => Promise<ContainerSummary | null>;
  /** Load root-level children */
  loadRootChildren: (offset?: number, limit?: number) => Promise<LazyLoadResult | null>;
  /** Load more root children (pagination) */
  loadMoreRoot: () => Promise<LazyLoadResult | null>;
  /** Load children at a specific path */
  loadChildren: (
    parentPath: string,
    offset?: number,
    limit?: number
  ) => Promise<LazyLoadResult | null>;
  /** Get cached children for a path */
  getCachedChildren: (parentPath: string) => FileEntry[] | undefined;
  /** Check if children are cached for a path */
  hasChildren: (parentPath: string) => boolean;
  /** Check if a path is currently loading */
  isPathLoading: (parentPath: string) => boolean;
  /** Clear all caches */
  clearCache: () => void;
  /** Reset all state */
  reset: () => void;
}

// =============================================================================
// HOOK IMPLEMENTATION
// =============================================================================

export function useUnifiedContainer(
  containerPath: () => string | null,
  options: UseUnifiedContainerOptions = {}
): UseUnifiedContainerReturn {
  // === Configuration ===
  const batchSize = options.batchSize ?? 100;

  // === State ===
  const [summary, setSummary] = createSignal<ContainerSummary | null>(null);
  const [containerType, setContainerType] = createSignal<ContainerType | null>(null);
  const [rootChildren, setRootChildren] = createSignal<FileEntry[]>([]);
  const [rootTotalCount, setRootTotalCount] = createSignal(0);
  const [hasMoreRoot, setHasMoreRoot] = createSignal(false);
  const [rootOffset, setRootOffset] = createSignal(0);
  const [isLoadingRoot, setIsLoadingRoot] = createSignal(false);
  const [loadingPaths, setLoadingPaths] = createSignal<Set<string>>(new Set());
  const [error, setError] = createSignal<Error | null>(null);
  const [config, setConfig] = createSignal<LazyLoadConfig>(DEFAULT_LAZY_LOAD_CONFIG);
  const [childrenCache, setChildrenCache] = createSignal<Map<string, FileEntry[]>>(
    new Map()
  );

  // Pagination state per path
  const paginationState = new Map<
    string,
    { offset: number; total: number; hasMore: boolean }
  >();

  // === Computed ===
  const isLoading = () => isLoadingRoot() || loadingPaths().size > 0;

  // === Helper Functions ===

  const addLoadingPath = (path: string) => {
    setLoadingPaths((prev) => {
      const next = new Set(prev);
      next.add(path);
      return next;
    });
  };

  const removeLoadingPath = (path: string) => {
    setLoadingPaths((prev) => {
      const next = new Set(prev);
      next.delete(path);
      return next;
    });
  };

  // === Actions ===

  /**
   * Load container summary to get type, entry count, and lazy loading recommendation
   */
  const loadSummary = async (): Promise<ContainerSummary | null> => {
    const path = containerPath();
    if (!path) return null;

    try {
      setError(null);
      options.onLoadStart?.();

      const result = await invoke<ContainerSummary>("unified_get_summary", {
        containerPath: path,
      });

      setSummary(result);
      // Parse container type from string
      setContainerType(result.container_type as ContainerType);

      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      log.error("[useUnifiedContainer] loadSummary failed:", err);
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

    const actualOffset = offset ?? 0;
    const actualLimit = limit ?? batchSize;

    try {
      setError(null);
      setIsLoadingRoot(true);
      options.onLoadStart?.();

      const result = await invoke<LazyLoadResult>("unified_get_root_children", {
        containerPath: path,
        offset: actualOffset,
        limit: actualLimit,
      });

      // Convert LazyTreeEntry to FileEntry if needed
      const entries = result.entries.map((e) => ({
        id: e.id,
        name: e.name,
        path: e.path,
        is_directory: e.is_dir,
        size: e.size,
        file_type: e.entry_type,
        child_count: e.child_count,
      }));

      if (actualOffset === 0) {
        // Fresh load - replace entries
        setRootChildren(entries);
      } else {
        // Pagination - append entries
        setRootChildren((prev) => [...prev, ...entries]);
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
      log.error("[useUnifiedContainer] loadRootChildren failed:", err);
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
    return loadRootChildren(rootOffset(), batchSize);
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

    const actualOffset = offset ?? 0;
    const actualLimit = limit ?? batchSize;

    try {
      setError(null);
      addLoadingPath(parentPath);
      options.onLoadStart?.();

      const result = await invoke<LazyLoadResult>("unified_get_children", {
        containerPath: path,
        parentPath,
        offset: actualOffset,
        limit: actualLimit,
      });

      // Convert LazyTreeEntry to FileEntry if needed
      const entries = result.entries.map((e) => ({
        id: e.id,
        name: e.name,
        path: e.path,
        is_directory: e.is_dir,
        size: e.size,
        file_type: e.entry_type,
        child_count: e.child_count,
      }));

      // Update cache
      setChildrenCache((prev) => {
        const next = new Map(prev);
        if (actualOffset === 0) {
          // Fresh load - replace entries
          next.set(parentPath, entries);
        } else {
          // Pagination - append entries
          const existing = next.get(parentPath) ?? [];
          next.set(parentPath, [...existing, ...entries]);
        }
        return next;
      });

      // Update pagination state
      paginationState.set(parentPath, {
        offset: result.next_offset,
        total: result.total_count,
        hasMore: result.has_more,
      });

      options.onLoadComplete?.(result);
      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      options.onError?.(error);
      log.error("[useUnifiedContainer] loadChildren failed:", err);
      return null;
    } finally {
      removeLoadingPath(parentPath);
    }
  };

  /**
   * Get cached children for a path
   */
  const getCachedChildren = (parentPath: string): FileEntry[] | undefined => {
    return childrenCache().get(parentPath);
  };

  /**
   * Check if children are cached for a path
   */
  const hasChildren = (parentPath: string): boolean => {
    return childrenCache().has(parentPath);
  };

  /**
   * Check if a path is currently loading
   */
  const isPathLoading = (parentPath: string): boolean => {
    return loadingPaths().has(parentPath);
  };

  /**
   * Clear all caches
   */
  const clearCache = () => {
    setChildrenCache(new Map());
    paginationState.clear();
  };

  /**
   * Reset all state
   */
  const reset = () => {
    setSummary(null);
    setContainerType(null);
    setRootChildren([]);
    setRootTotalCount(0);
    setHasMoreRoot(false);
    setRootOffset(0);
    setError(null);
    clearCache();
  };

  // === Effects ===

  // Auto-load summary when path changes
  createEffect(() => {
    const path = containerPath();
    if (path && options.autoLoadSummary !== false) {
      loadSummary().then((s) => {
        if (s && options.autoLoadRoot) {
          loadRootChildren();
        }
      });
    } else if (!path) {
      reset();
    }
  });

  // Cleanup on unmount
  onCleanup(() => {
    clearCache();
  });

  // === Return ===
  return {
    // State
    summary,
    containerType,
    rootChildren,
    rootTotalCount,
    hasMoreRoot,
    isLoadingRoot,
    isLoading,
    error,
    config,
    childrenCache,
    loadingPaths,

    // Actions
    loadSummary,
    loadRootChildren,
    loadMoreRoot,
    loadChildren,
    getCachedChildren,
    hasChildren,
    isPathLoading,
    clearCache,
    reset,
  };
}

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

/**
 * Direct function to get container summary without hook
 */
export async function getContainerSummary(
  containerPath: string
): Promise<ContainerSummary> {
  return invoke<ContainerSummary>("unified_get_summary", { containerPath });
}

/**
 * Direct function to get root children without hook
 */
export async function getRootChildren(
  containerPath: string,
  offset = 0,
  limit = 100
): Promise<LazyLoadResult> {
  return invoke<LazyLoadResult>("unified_get_root_children", {
    containerPath,
    offset,
    limit,
  });
}

/**
 * Direct function to get children without hook
 */
export async function getChildren(
  containerPath: string,
  parentPath: string,
  offset = 0,
  limit = 100
): Promise<LazyLoadResult> {
  return invoke<LazyLoadResult>("unified_get_children", {
    containerPath,
    parentPath,
    offset,
    limit,
  });
}
