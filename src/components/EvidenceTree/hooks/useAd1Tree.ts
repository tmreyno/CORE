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

export interface UseAd1TreeReturn {
  // State accessors
  childrenCache: Accessor<Map<string, TreeEntry[]>>;
  expandedDirs: Accessor<Set<string>>;
  ad1InfoCache: Accessor<Map<string, Ad1ContainerSummary>>;
  containerErrors: Accessor<Map<string, string>>;
  
  // Operations
  loadAd1Info: (containerPath: string) => Promise<Ad1ContainerSummary | null>;
  loadRootChildren: (containerPath: string) => Promise<TreeEntry[]>;
  loadChildrenByAddr: (containerPath: string, addr: number, parentPath?: string) => Promise<TreeEntry[]>;
  loadChildrenByPath: (containerPath: string, entryPath: string) => Promise<TreeEntry[]>;
  toggleDirByAddr: (containerPath: string, entry: TreeEntry, loading: Set<string>, setLoading: (fn: (prev: Set<string>) => Set<string>) => void) => Promise<void>;
  
  // Getters
  getChildrenForEntry: (containerPath: string, entry: TreeEntry) => TreeEntry[];
  isDirExpanded: (containerPath: string, entry: TreeEntry) => boolean;
  
  // Utilities
  sortEntries: (entries: TreeEntry[]) => TreeEntry[];
}

/**
 * Hook for managing AD1 container tree state and operations
 */
export function useAd1Tree(): UseAd1TreeReturn {
  // Cache children by key (containerPath::addr or containerPath::path)
  const [childrenCache, setChildrenCache] = createSignal<Map<string, TreeEntry[]>>(new Map());
  // Track expanded directories
  const [expandedDirs, setExpandedDirs] = createSignal<Set<string>>(new Set());
  // Cache AD1 container info
  const [ad1InfoCache, setAd1InfoCache] = createSignal<Map<string, Ad1ContainerSummary>>(new Map());
  // Track loading errors
  const [containerErrors, setContainerErrors] = createSignal<Map<string, string>>(new Map());
  
  // Load AD1 container summary info
  const loadAd1Info = async (containerPath: string): Promise<Ad1ContainerSummary | null> => {
    const cached = ad1InfoCache().get(containerPath);
    if (cached) return cached;

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
      console.error("[loadAd1Info] Failed:", err);
      return null;
    }
  };

  // Load root children - uses V2 API
  const loadRootChildren = async (containerPath: string): Promise<TreeEntry[]> => {
    const cacheKey = `${containerPath}::root`;
    
    const cached = childrenCache().get(cacheKey);
    if (cached) return cached;
    
    try {
      const children = await invoke<TreeEntry[]>("container_get_root_children_v2", {
        containerPath,
      });
      
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
      
      return children;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error("Failed to load root children:", errorMsg);
      
      setContainerErrors(prev => {
        const next = new Map(prev);
        next.set(containerPath, errorMsg);
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
      console.error("Failed to load children at addr:", err);
      return [];
    }
  };

  // Load children by path (fallback when addr not available)
  const loadChildrenByPath = async (containerPath: string, entryPath: string): Promise<TreeEntry[]> => {
    const cacheKey = `${containerPath}::path:${entryPath}`;
    
    const cached = childrenCache().get(cacheKey);
    if (cached) return cached;
    
    try {
      const children = await invoke<TreeEntry[]>("container_get_children", {
        containerPath,
        parentPath: entryPath,
      });
      
      setChildrenCache(prev => {
        const next = new Map(prev);
        next.set(cacheKey, children);
        return next;
      });
      
      return children;
    } catch (err) {
      console.error("Failed to load children at path:", err);
      return [];
    }
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
      setExpandedDirs(expanded);
    } else {
      // Expand immediately
      expanded.add(nodeKey);
      setExpandedDirs(expanded);
      
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

  return {
    childrenCache,
    expandedDirs,
    ad1InfoCache,
    containerErrors,
    loadAd1Info,
    loadRootChildren,
    loadChildrenByAddr,
    loadChildrenByPath,
    toggleDirByAddr,
    getChildrenForEntry,
    isDirExpanded,
    sortEntries,
  };
}
