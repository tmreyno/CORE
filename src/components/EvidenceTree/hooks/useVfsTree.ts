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
import { logger } from "../../../utils/logger";

const log = logger.scope("VfsTree");

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
  
  // State persistence
  restoreExpandedPaths: (paths: string[]) => void;
}

/**
 * Hook for managing VFS container tree state and operations
 */
export function useVfsTree(): UseVfsTreeReturn {
  log.debug("Hook initialized");
  
  // Cache mount info by container path
  const [vfsMountCache, setVfsMountCache] = createSignal<Map<string, VfsMountInfo>>(new Map());
  // Cache directory listings
  const [vfsChildrenCache, setVfsChildrenCache] = createSignal<Map<string, VfsEntry[]>>(new Map());
  // Track expanded paths
  const [expandedVfsPaths, setExpandedVfsPaths] = createSignal<Set<string>>(new Set());

  // Mount a disk image container and get partition info
  const mountVfsContainer = async (containerPath: string): Promise<VfsMountInfo | null> => {
    const startTime = performance.now();
    log.debug(`mountVfsContainer called for ${containerPath}`);
    const cached = vfsMountCache().get(containerPath);
    if (cached) {
      log.debug(`mountVfsContainer - returning cached mount with ${cached.partitions.length} partitions (${(performance.now() - startTime).toFixed(1)}ms)`);
      return cached;
    }

    try {
      log.debug("mountVfsContainer - invoking vfs_mount_image...");
      const invokeStart = performance.now();
      const mountInfo = await invoke<VfsMountInfo>("vfs_mount_image", {
        containerPath,
      });
      
      log.debug(`mountVfsContainer - backend returned in ${(performance.now() - invokeStart).toFixed(1)}ms, ${mountInfo.partitions.length} partitions, diskSize=${mountInfo.diskSize}`);
      
      setVfsMountCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, mountInfo);
        return next;
      });
      
      log.debug(`mountVfsContainer - total time: ${(performance.now() - startTime).toFixed(1)}ms`);
      return mountInfo;
    } catch (err) {
      log.error("mountVfsContainer FAILED:", err);
      return null;
    }
  };

  // Load directory contents
  const loadVfsChildren = async (containerPath: string, vfsPath: string): Promise<VfsEntry[]> => {
    const cacheKey = `${containerPath}::vfs::${vfsPath}`;
    
    log.debug(`loadVfsChildren called: containerPath=${containerPath}, vfsPath=${vfsPath}`);
    
    const cached = vfsChildrenCache().get(cacheKey);
    if (cached) {
      log.debug(`loadVfsChildren - returning ${cached.length} cached entries`);
      return cached;
    }

    try {
      log.debug("loadVfsChildren - invoking vfs_list_dir...");
      const children = await invoke<VfsEntry[]>("vfs_list_dir", {
        containerPath,
        dirPath: vfsPath,
      });
      
      log.debug(`loadVfsChildren - backend returned ${children.length} entries for ${vfsPath}`);
      
      setVfsChildrenCache(prev => {
        const next = new Map(prev);
        next.set(cacheKey, children);
        return next;
      });
      
      return children;
    } catch (err) {
      log.error(`loadVfsChildren failed for ${vfsPath}:`, err);
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
    
    log.debug(`toggleVfsDir called: path=${vfsPath}, currently expanded=${expanded.has(nodeKey)}`);
    
    if (expanded.has(nodeKey)) {
      expanded.delete(nodeKey);
      setExpandedVfsPaths(new Set(expanded));
    } else {
      const cacheKey = nodeKey;
      const needsLoad = !vfsChildrenCache().has(cacheKey);
      
      if (needsLoad) {
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
    log.debug(`expandAllVfsDirs called for ${containerPath}`);
    const mountInfo = vfsMountCache().get(containerPath);
    if (!mountInfo) {
      log.debug(`expandAllVfsDirs - no mount info found for ${containerPath}`);
      return;
    }
    
    // For VFS containers, expand the root children of each partition
    const partitionsToLoad: { mountName: string; rootPath: string }[] = [];
    for (let i = 0; i < (mountInfo.partitions || []).length; i++) {
      const partition = mountInfo.partitions[i];
      // Use mountName as the root path for the partition, with fallback
      const mountName = partition.mountName ?? `Partition${partition.number ?? i + 1}`;
      const rootPath = `/${mountName}`;
      partitionsToLoad.push({ mountName, rootPath });
    }
    
    log.debug(`expandAllVfsDirs - found ${partitionsToLoad.length} partitions to load`);
    
    // Load children for each partition in parallel
    if (partitionsToLoad.length > 0) {
      await Promise.all(
        partitionsToLoad.map(async ({ rootPath }) => {
          try {
            await loadVfsChildren(containerPath, rootPath);
          } catch (error) {
            log.error(`expandAllVfsDirs - failed to load ${rootPath}:`, error);
          }
        })
      );
      
      // After loading, mark as expanded
      setExpandedVfsPaths(prev => {
        const next = new Set(prev);
        partitionsToLoad.forEach(({ rootPath }) => {
          const key = `${containerPath}::vfs::${rootPath}`;
          next.add(key);
        });
        return next;
      });
      
      log.debug(`expandAllVfsDirs - completed for ${containerPath}`);
    }
  };

  // Collapse all VFS directories
  const collapseAllVfsDirs = (): void => {
    setExpandedVfsPaths(new Set<string>());
  };

  // Restore expanded paths from serialized state
  const restoreExpandedPaths = (paths: string[]): void => {
    setExpandedVfsPaths(new Set(paths));
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
    restoreExpandedPaths,
  };
}
