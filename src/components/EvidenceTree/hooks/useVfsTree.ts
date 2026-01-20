// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useVfsTree - VFS (Virtual File System) tree operations hook
 * 
 * Manages state and operations for navigating disk image containers (E01, Raw, L01).
 * Handles partition mounting and filesystem browsing.
 */

import { createSignal, Accessor } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { VfsMountInfo, VfsEntry } from "../../../types";

export interface UseVfsTreeReturn {
  // State accessors
  vfsMountCache: Accessor<Map<string, VfsMountInfo>>;
  vfsChildrenCache: Accessor<Map<string, VfsEntry[]>>;
  expandedVfsPaths: Accessor<Set<string>>;
  
  // Operations
  mountVfsContainer: (containerPath: string) => Promise<VfsMountInfo | null>;
  loadVfsChildren: (containerPath: string, vfsPath: string) => Promise<VfsEntry[]>;
  toggleVfsDir: (containerPath: string, vfsPath: string, loading: Set<string>, setLoading: (fn: (prev: Set<string>) => Set<string>) => void) => Promise<void>;
  
  // Getters
  getVfsChildren: (containerPath: string, vfsPath: string) => VfsEntry[];
  isVfsDirExpanded: (containerPath: string, vfsPath: string) => boolean;
  
  // Utilities
  sortVfsEntries: (entries: VfsEntry[]) => VfsEntry[];
  
  // Expand/Collapse all
  expandAllVfsDirs: (containerPath: string) => Promise<void>;
  collapseAllVfsDirs: () => void;
}

/**
 * Hook for managing VFS container tree state and operations
 */
export function useVfsTree(): UseVfsTreeReturn {
  // Cache mount info by container path
  const [vfsMountCache, setVfsMountCache] = createSignal<Map<string, VfsMountInfo>>(new Map());
  // Cache directory listings
  const [vfsChildrenCache, setVfsChildrenCache] = createSignal<Map<string, VfsEntry[]>>(new Map());
  // Track expanded paths
  const [expandedVfsPaths, setExpandedVfsPaths] = createSignal<Set<string>>(new Set());

  // Mount a disk image container and get partition info
  const mountVfsContainer = async (containerPath: string): Promise<VfsMountInfo | null> => {
    const cached = vfsMountCache().get(containerPath);
    if (cached) return cached;

    try {
      const mountInfo = await invoke<VfsMountInfo>("vfs_mount_image", {
        containerPath,
      });
      
      setVfsMountCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, mountInfo);
        return next;
      });
      
      return mountInfo;
    } catch (err) {
      console.error("Failed to mount VFS container:", err);
      return null;
    }
  };

  // Load directory contents
  const loadVfsChildren = async (containerPath: string, vfsPath: string): Promise<VfsEntry[]> => {
    const cacheKey = `${containerPath}::vfs::${vfsPath}`;
    
    const cached = vfsChildrenCache().get(cacheKey);
    if (cached) return cached;

    try {
      const children = await invoke<VfsEntry[]>("vfs_list_dir", {
        containerPath,
        dirPath: vfsPath,
      });
      
      setVfsChildrenCache(prev => {
        const next = new Map(prev);
        next.set(cacheKey, children);
        return next;
      });
      
      return children;
    } catch (err) {
      console.error("Failed to load VFS directory:", err);
      return [];
    }
  };

  // Toggle directory expansion
  const toggleVfsDir = async (
    containerPath: string, 
    vfsPath: string,
    _loading: Set<string>,
    setLoading: (fn: (prev: Set<string>) => Set<string>) => void
  ): Promise<void> => {
    const nodeKey = `${containerPath}::vfs::${vfsPath}`;
    const expanded = new Set(expandedVfsPaths());
    
    if (expanded.has(nodeKey)) {
      expanded.delete(nodeKey);
      setExpandedVfsPaths(new Set(expanded));
    } else {
      const cacheKey = nodeKey;
      if (!vfsChildrenCache().has(cacheKey)) {
        setLoading(prev => new Set([...prev, nodeKey]));
        await loadVfsChildren(containerPath, vfsPath);
        setLoading(prev => {
          const next = new Set(prev);
          next.delete(nodeKey);
          return next;
        });
      }
      expanded.add(nodeKey);
      setExpandedVfsPaths(new Set(expanded));
    }
  };

  // Get cached children
  const getVfsChildren = (containerPath: string, vfsPath: string): VfsEntry[] => {
    const cacheKey = `${containerPath}::vfs::${vfsPath}`;
    const children = vfsChildrenCache().get(cacheKey) || [];
    return sortVfsEntries(children);
  };

  // Check if directory is expanded
  const isVfsDirExpanded = (containerPath: string, vfsPath: string): boolean => {
    const nodeKey = `${containerPath}::vfs::${vfsPath}`;
    return expandedVfsPaths().has(nodeKey);
  };

  // Sort entries: directories first, then alphabetically
  const sortVfsEntries = (entries: VfsEntry[]): VfsEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  };

  // Expand all VFS directories for a container (loads root level directories)
  const expandAllVfsDirs = async (containerPath: string): Promise<void> => {
    const mountInfo = vfsMountCache().get(containerPath);
    if (!mountInfo) return;
    
    // For VFS containers, expand the root children of each partition
    const keysToExpand: string[] = [];
    for (let i = 0; i < (mountInfo.partitions || []).length; i++) {
      const partition = mountInfo.partitions[i];
      // Use mountName as the root path for the partition, with fallback
      const mountName = partition.mountName ?? `Partition${partition.number ?? i + 1}`;
      const rootKey = `${containerPath}::vfs::/${mountName}`;
      keysToExpand.push(rootKey);
    }
    
    if (keysToExpand.length > 0) {
      setExpandedVfsPaths(prev => {
        const next = new Set(prev);
        keysToExpand.forEach(key => next.add(key));
        return next;
      });
    }
  };

  // Collapse all VFS directories
  const collapseAllVfsDirs = (): void => {
    setExpandedVfsPaths(new Set<string>());
  };

  return {
    vfsMountCache,
    vfsChildrenCache,
    expandedVfsPaths,
    mountVfsContainer,
    loadVfsChildren,
    toggleVfsDir,
    getVfsChildren,
    isVfsDirExpanded,
    sortVfsEntries,
    expandAllVfsDirs,
    collapseAllVfsDirs,
  };
}
