// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAd1Tree - AD1 container tree operations hook
 * 
 * Manages state and operations for navigating AD1 (AccessData Logical) containers.
 * Uses V2 backend APIs for optimal performance with address-based navigation.
 */

import { createSignal, Accessor } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { TreeEntry, Ad1ContainerSummary } from "../../../types";
import { logger } from "../../../utils/logger";

const log = logger.scope("Ad1Tree");

/**
 * On-demand metadata for an item (loaded lazily)
 */
export interface ItemMetadata {
  itemAddr: number;
  md5Hash?: string | null;
  sha1Hash?: string | null;
  created?: string | null;
  accessed?: string | null;
  modified?: string | null;
  attributes?: string[] | null;
}

/**
 * Container status (segment availability for partial AD1 support)
 */
export interface ContainerStatus {
  path: string;
  expectedSegments: number;
  availableSegments: number;
  missingSegments: number[];
  isComplete: boolean;
  statusMessage: string;
}

export interface UseAd1TreeReturn {
  // State accessors
  childrenCache: Accessor<Map<string, TreeEntry[]>>;
  expandedDirs: Accessor<Set<string>>;
  ad1InfoCache: Accessor<Map<string, Ad1ContainerSummary>>;
  containerErrors: Accessor<Map<string, string>>;
  metadataCache: Accessor<Map<string, ItemMetadata>>;
  containerStatusCache: Accessor<Map<string, ContainerStatus>>;
  
  // Operations
  loadAd1Info: (containerPath: string) => Promise<Ad1ContainerSummary | null>;
  loadContainerStatus: (containerPath: string) => Promise<ContainerStatus | null>;
  loadRootChildren: (containerPath: string) => Promise<TreeEntry[]>;
  loadChildrenByAddr: (containerPath: string, addr: number, parentPath?: string) => Promise<TreeEntry[]>;
  loadChildrenByPath: (containerPath: string, entryPath: string) => Promise<TreeEntry[]>;
  toggleDirByAddr: (containerPath: string, entry: TreeEntry, loading: Set<string>, setLoading: (fn: (prev: Set<string>) => Set<string>) => void) => Promise<void>;
  
  // On-demand metadata loading
  loadItemMetadata: (containerPath: string, itemAddr: number) => Promise<ItemMetadata | null>;
  loadItemsMetadata: (containerPath: string, itemAddrs: number[]) => Promise<ItemMetadata[]>;
  getItemMetadata: (containerPath: string, itemAddr: number) => ItemMetadata | undefined;
  
  // Getters
  getChildrenForEntry: (containerPath: string, entry: TreeEntry) => TreeEntry[];
  isDirExpanded: (containerPath: string, entry: TreeEntry) => boolean;
  getContainerStatus: (containerPath: string) => ContainerStatus | undefined;
  
  // Utilities
  sortEntries: (entries: TreeEntry[]) => TreeEntry[];
  
  // Expand/Collapse all
  expandAllAd1Dirs: (containerPath: string, loading: Set<string>, setLoading: (fn: (prev: Set<string>) => Set<string>) => void) => Promise<void>;
  collapseAllDirs: () => void;
  
  // State persistence
  restoreExpandedDirs: (keys: string[]) => void;
}

/**
 * Hook for managing AD1 container tree state and operations
 */
export function useAd1Tree(): UseAd1TreeReturn {
  log.debug("Hook initialized");
  
  // Cache children by key (containerPath::addr or containerPath::path)
  const [childrenCache, setChildrenCache] = createSignal<Map<string, TreeEntry[]>>(new Map());
  // Track expanded directories
  const [expandedDirs, setExpandedDirs] = createSignal<Set<string>>(new Set());
  // Cache AD1 container info
  const [ad1InfoCache, setAd1InfoCache] = createSignal<Map<string, Ad1ContainerSummary>>(new Map());
  // Track loading errors
  const [containerErrors, setContainerErrors] = createSignal<Map<string, string>>(new Map());
  // Cache for on-demand loaded metadata (key: containerPath::itemAddr)
  const [metadataCache, setMetadataCache] = createSignal<Map<string, ItemMetadata>>(new Map());
  // Cache for container status (segment availability)
  const [containerStatusCache, setContainerStatusCache] = createSignal<Map<string, ContainerStatus>>(new Map());

  // Load container status (segment availability)
  const loadContainerStatus = async (containerPath: string): Promise<ContainerStatus | null> => {
    log.debug(`loadContainerStatus called for ${containerPath}`);
    const cached = containerStatusCache().get(containerPath);
    if (cached) {
      return cached;
    }

    try {
      const status = await invoke<ContainerStatus>("container_get_status_v2", {
        containerPath,
      });
      log.debug(`Container status loaded - complete: ${status.isComplete}, segments: ${status.availableSegments}/${status.expectedSegments}`);
      
      // Cache the status
      setContainerStatusCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, status);
        return next;
      });
      
      return status;
    } catch (e) {
      log.warn(`Failed to load container status: ${e}`);
      return null;
    }
  };

  // Get cached container status
  const getContainerStatus = (containerPath: string): ContainerStatus | undefined => {
    return containerStatusCache().get(containerPath);
  };
  
  // Load AD1 container summary info
  const loadAd1Info = async (containerPath: string): Promise<Ad1ContainerSummary | null> => {
    log.debug(`loadAd1Info called for ${containerPath}`);
    const cached = ad1InfoCache().get(containerPath);
    if (cached) {
      log.debug("loadAd1Info - returning cached info");
      return cached;
    }

    try {
      const info = await invoke<{
        total_items: number;
        total_size: number;
        file_count: number;
        dir_count: number;
        logical_header: { data_source_name: string };
      }>("container_get_info_v2", {
        containerPath,
        includeTree: false,
      });
      
      const summary: Ad1ContainerSummary = {
        total_items: Number(info.total_items),
        total_size: Number(info.total_size),
        file_count: Number(info.file_count),
        dir_count: Number(info.dir_count),
        source_name: info.logical_header?.data_source_name || null,
      };
      
      setAd1InfoCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, summary);
        return next;
      });
      
      return summary;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      log.error("Failed:", errorMsg);
      
      // Check for missing segment error and store user-friendly message
      if (errorMsg.includes("Failed to open segment") && errorMsg.includes("No such file or directory")) {
        const segmentMatch = errorMsg.match(/segment ([^:]+\.ad\d+)/i);
        const segmentName = segmentMatch ? segmentMatch[1] : "segment file";
        const userMessage = `⚠️ Incomplete AD1: Missing ${segmentName}. This container has missing segment files.`;
        setContainerErrors(prev => {
          const next = new Map(prev);
          next.set(containerPath, userMessage);
          return next;
        });
      }
      
      return null;
    }
  };

  // Load root children - uses V2 API
  const loadRootChildren = async (containerPath: string): Promise<TreeEntry[]> => {
    const cacheKey = `${containerPath}::root`;
    const startTime = performance.now();
    log.debug(`loadRootChildren called for ${containerPath}`);
    
    const cached = childrenCache().get(cacheKey);
    if (cached) {
      log.debug(`loadRootChildren - returning ${cached.length} cached children (${(performance.now() - startTime).toFixed(1)}ms)`);
      return cached;
    }
    
    try {
      log.debug(`loadRootChildren - invoking container_get_root_children_v2...`);
      const invokeStart = performance.now();
      const children = await invoke<TreeEntry[]>("container_get_root_children_v2", {
        containerPath,
      });
      
      log.debug(`loadRootChildren - backend returned ${children.length} children in ${(performance.now() - invokeStart).toFixed(1)}ms`);
      
      // Clear any previous error
      setContainerErrors(prev => {
        const next = new Map(prev);
        next.delete(containerPath);
        return next;
      });
      
      setChildrenCache(prev => {
        const next = new Map(prev);
        next.set(cacheKey, children);
        return next;
      });
      
      log.debug(`loadRootChildren - total time: ${(performance.now() - startTime).toFixed(1)}ms`);
      return children;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      log.error("loadRootChildren FAILED:", errorMsg);
      
      // Check for missing segment error and provide user-friendly message
      let userMessage = errorMsg;
      if (errorMsg.includes("Failed to open segment") && errorMsg.includes("No such file or directory")) {
        // Extract the missing segment name from the error
        const segmentMatch = errorMsg.match(/segment ([^:]+\.ad\d+)/i);
        const segmentName = segmentMatch ? segmentMatch[1] : "segment file";
        userMessage = `⚠️ Incomplete AD1: Missing ${segmentName}. This container has missing segment files and cannot be fully loaded. Please ensure all AD1 segment files (.ad1, .ad2, .ad3, etc.) are present in the same directory.`;
        log.warn(`AD1 Warning: Container ${containerPath} is incomplete:`, errorMsg);
      }
      
      setContainerErrors(prev => {
        const next = new Map(prev);
        next.set(containerPath, userMessage);
        return next;
      });
      
      return [];
    }
  };

  // Load children by address - uses V2 API (fastest method)
  const loadChildrenByAddr = async (containerPath: string, addr: number, parentPath: string = ""): Promise<TreeEntry[]> => {
    const cacheKey = `${containerPath}::addr:${addr}`;
    
    const cached = childrenCache().get(cacheKey);
    if (cached) return cached;
    
    try {
      const children = await invoke<TreeEntry[]>("container_get_children_at_addr_v2", {
        containerPath,
        addr,
        parentPath,
      });
      
      setChildrenCache(prev => {
        const next = new Map(prev);
        next.set(cacheKey, children);
        return next;
      });
      
      return children;
    } catch (err) {
      log.error("Failed to load children at addr:", err);
      return [];
    }
  };

  // Load children by path - DEPRECATED: V1 API removed for performance
  // This is a fallback that should rarely be called - AD1 entries always have addresses
  const loadChildrenByPath = async (_containerPath: string, entryPath: string): Promise<TreeEntry[]> => {
    log.warn(
      `[DEPRECATED] loadChildrenByPath called for "${entryPath}". ` +
      `AD1 entries should always have item_addr. Using V2 API is ~8000x faster.`
    );
    
    // Return empty - caller should use loadChildrenByAddr with entry.first_child_addr
    // The V1 container_get_children API has been removed for performance reasons
    return [];
  };

  // Toggle directory expansion
  const toggleDirByAddr = async (
    containerPath: string, 
    entry: TreeEntry, 
    _loading: Set<string>,
    setLoading: (fn: (prev: Set<string>) => Set<string>) => void
  ): Promise<void> => {
    const addr = entry.item_addr;
    const entryPath = entry.path;
    
    const nodeKey = addr 
      ? `${containerPath}::addr:${addr}` 
      : `${containerPath}::path:${entryPath}`;
    
    const expanded = new Set(expandedDirs());
    
    if (expanded.has(nodeKey)) {
      expanded.delete(nodeKey);
      setExpandedDirs(new Set(expanded));
    } else {
      // Expand immediately
      expanded.add(nodeKey);
      setExpandedDirs(new Set(expanded));
      
      if (!childrenCache().has(nodeKey)) {
        setLoading(prev => new Set([...prev, nodeKey]));
        
        if (addr) {
          await loadChildrenByAddr(containerPath, addr, entryPath);
        } else {
          await loadChildrenByPath(containerPath, entryPath);
        }
        
        setLoading(prev => {
          const next = new Set(prev);
          next.delete(nodeKey);
          return next;
        });
      }
    }
  };

  // Get cached children for an entry
  const getChildrenForEntry = (containerPath: string, entry: TreeEntry): TreeEntry[] => {
    const addr = entry.item_addr;
    const cacheKey = addr 
      ? `${containerPath}::addr:${addr}` 
      : `${containerPath}::path:${entry.path}`;
    const children = childrenCache().get(cacheKey) || [];
    return sortEntries(children);
  };

  // Check if directory is expanded
  const isDirExpanded = (containerPath: string, entry: TreeEntry): boolean => {
    const addr = entry.item_addr;
    const nodeKey = addr 
      ? `${containerPath}::addr:${addr}` 
      : `${containerPath}::path:${entry.path}`;
    return expandedDirs().has(nodeKey);
  };

  // Sort entries: folders first, then alphabetically
  const sortEntries = (entries: TreeEntry[]): TreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };

  // Expand all AD1 directories (root level only to avoid deep recursion)
  const expandAllAd1Dirs = async (
    containerPath: string, 
    _loading: Set<string>, 
    _setLoading: (fn: (prev: Set<string>) => Set<string>) => void
  ): Promise<void> => {
    const rootKey = `${containerPath}::root`;
    const rootChildren = childrenCache().get(rootKey) || [];
    
    // Expand root-level directories
    const keysToExpand: string[] = [];
    for (const entry of rootChildren) {
      if (entry.is_dir) {
        const addr = entry.item_addr;
        const nodeKey = addr !== undefined && addr !== null
          ? `${containerPath}::addr:${addr}` 
          : `${containerPath}::path:${entry.path}`;
        keysToExpand.push(nodeKey);
        
        // Load children if not cached
        if (addr !== undefined && addr !== null) {
          const cacheKey = `${containerPath}::${addr}`;
          if (!childrenCache().has(cacheKey)) {
            await loadChildrenByAddr(containerPath, addr, entry.path);
          }
        }
      }
    }
    
    if (keysToExpand.length > 0) {
      setExpandedDirs(prev => {
        const next = new Set(prev);
        keysToExpand.forEach(key => next.add(key));
        return next;
      });
    }
  };

  // Collapse all AD1 directories
  const collapseAllDirs = (): void => {
    setExpandedDirs(new Set<string>());
  };

  // Restore expanded directories from serialized state
  const restoreExpandedDirs = (keys: string[]): void => {
    setExpandedDirs(new Set(keys));
  };

  // ==========================================================================
  // On-Demand Metadata Loading
  // ==========================================================================

  /**
   * Load metadata for a single item (hashes, timestamps, attributes)
   * Results are cached for subsequent requests.
   */
  const loadItemMetadata = async (containerPath: string, itemAddr: number): Promise<ItemMetadata | null> => {
    const cacheKey = `${containerPath}::${itemAddr}`;
    
    // Return cached if available
    const cached = metadataCache().get(cacheKey);
    if (cached) {
      return cached;
    }

    try {
      const metadata = await invoke<ItemMetadata>("container_get_item_metadata_v2", {
        containerPath,
        itemAddr,
      });
      
      // Cache the result
      setMetadataCache(prev => {
        const next = new Map(prev);
        next.set(cacheKey, metadata);
        return next;
      });
      
      return metadata;
    } catch (err) {
      log.error(`loadItemMetadata Failed for addr=${itemAddr}:`, err);
      return null;
    }
  };

  /**
   * Load metadata for multiple items in batch (more efficient than individual calls)
   * Results are cached for subsequent requests.
   */
  const loadItemsMetadata = async (containerPath: string, itemAddrs: number[]): Promise<ItemMetadata[]> => {
    // Filter out already cached items
    const uncachedAddrs = itemAddrs.filter(addr => {
      const cacheKey = `${containerPath}::${addr}`;
      return !metadataCache().has(cacheKey);
    });

    if (uncachedAddrs.length === 0) {
      // All items are cached, return from cache
      return itemAddrs
        .map(addr => metadataCache().get(`${containerPath}::${addr}`))
        .filter((m): m is ItemMetadata => m !== undefined);
    }

    try {
      const results = await invoke<ItemMetadata[]>("container_get_items_metadata_v2", {
        containerPath,
        itemAddrs: uncachedAddrs,
      });
      
      // Cache all results
      setMetadataCache(prev => {
        const next = new Map(prev);
        for (const metadata of results) {
          const cacheKey = `${containerPath}::${metadata.itemAddr}`;
          next.set(cacheKey, metadata);
        }
        return next;
      });
      
      // Return all requested items (from cache + new results)
      return itemAddrs
        .map(addr => metadataCache().get(`${containerPath}::${addr}`))
        .filter((m): m is ItemMetadata => m !== undefined);
    } catch (err) {
      log.error(`loadItemsMetadata Failed for ${uncachedAddrs.length} items:`, err);
      return [];
    }
  };

  /**
   * Get cached metadata for an item (synchronous, returns undefined if not loaded)
   */
  const getItemMetadata = (containerPath: string, itemAddr: number): ItemMetadata | undefined => {
    const cacheKey = `${containerPath}::${itemAddr}`;
    return metadataCache().get(cacheKey);
  };

  return {
    childrenCache,
    expandedDirs,
    ad1InfoCache,
    containerErrors,
    metadataCache,
    containerStatusCache,
    loadAd1Info,
    loadContainerStatus,
    loadRootChildren,
    loadChildrenByAddr,
    loadChildrenByPath,
    toggleDirByAddr,
    loadItemMetadata,
    loadItemsMetadata,
    getItemMetadata,
    getChildrenForEntry,
    isDirExpanded,
    getContainerStatus,
    sortEntries,
    expandAllAd1Dirs,
    collapseAllDirs,
    restoreExpandedDirs,
  };
}
