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
      setExpandedVfsPaths(expanded);
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
      setExpandedVfsPaths(expanded);
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
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
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
  };
}
