// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useArchiveTree - Archive container tree operations hook
 * 
 * Manages state and operations for navigating archive containers (ZIP, 7z, RAR, TAR).
 * Supports nested container detection and extraction.
 */

import { createSignal, Accessor } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { ArchiveTreeEntry } from "../../../types";
import { getContainerType, isContainerFile } from "../containerDetection";
import { logger } from "../../../utils/logger";

const log = logger.scope("ArchiveTree");

/** Quick metadata from archive headers (fast - doesn't scan all entries) */
export interface ArchiveQuickMetadata {
  entry_count: number;
  archive_size: number;
  format: string;
  encrypted: boolean;
}

export interface UseArchiveTreeReturn {
  // State accessors
  archiveTreeCache: Accessor<Map<string, ArchiveTreeEntry[]>>;
  archiveMetaCache: Accessor<Map<string, ArchiveQuickMetadata>>;
  expandedArchivePaths: Accessor<Set<string>>;
  
  // Operations
  loadArchiveMetadata: (containerPath: string) => Promise<ArchiveQuickMetadata | null>;
  loadArchiveTree: (containerPath: string) => Promise<ArchiveTreeEntry[]>;
  toggleArchiveDir: (containerPath: string, archivePath: string) => void;
  openNestedContainer: (
    containerPath: string, 
    entryPath: string, 
    entryName: string,
    onOpenNestedContainer: ((tempPath: string, originalName: string, containerType: string, parentPath: string) => void) | undefined,
    loading: Set<string>,
    setLoading: (fn: (prev: Set<string>) => Set<string>) => void
  ) => Promise<void>;
  
  // Getters
  getArchiveRootEntries: (entries: ArchiveTreeEntry[]) => ArchiveTreeEntry[];
  getArchiveChildren: (entries: ArchiveTreeEntry[], parentPath: string) => ArchiveTreeEntry[];
  isArchiveDirExpanded: (containerPath: string, archivePath: string) => boolean;
  
  // Utilities
  sortArchiveEntries: (entries: ArchiveTreeEntry[]) => ArchiveTreeEntry[];
  isNestedContainer: (entry: ArchiveTreeEntry) => boolean;
  
  // Expand/Collapse all
  expandAllArchiveDirs: (containerPath: string, dirKeys: string[]) => void;
  collapseAllArchiveDirs: () => void;
  
  // State persistence
  restoreExpandedPaths: (paths: string[]) => void;
}

/**
 * Hook for managing archive container tree state and operations
 */
export function useArchiveTree(): UseArchiveTreeReturn {
  log.debug("Hook initialized");
  
  // Cache archive tree entries by container path
  const [archiveTreeCache, setArchiveTreeCache] = createSignal<Map<string, ArchiveTreeEntry[]>>(new Map());
  // Cache quick metadata (nearly instant header-only reads)
  const [archiveMetaCache, setArchiveMetaCache] = createSignal<Map<string, ArchiveQuickMetadata>>(new Map());
  // Track expanded archive directories
  const [expandedArchivePaths, setExpandedArchivePaths] = createSignal<Set<string>>(new Set());

  // Fetch quick metadata (nearly instant - only reads headers)
  const loadArchiveMetadata = async (containerPath: string): Promise<ArchiveQuickMetadata | null> => {
    log.debug(`loadArchiveMetadata called for ${containerPath}`);
    const cached = archiveMetaCache().get(containerPath);
    if (cached) {
      log.debug(`loadArchiveMetadata - returning cached meta, entryCount=${cached.entry_count}`);
      return cached;
    }
    
    try {
      log.debug("loadArchiveMetadata - invoking archive_get_metadata");
      const meta = await invoke<ArchiveQuickMetadata>('archive_get_metadata', {
        containerPath,
      });
      
      log.debug(`loadArchiveMetadata - got metadata: ${meta.entry_count} entries, format=${meta.format}, encrypted=${meta.encrypted}`);
      
      setArchiveMetaCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, meta);
        return next;
      });
      
      return meta;
    } catch (err) {
      log.error("loadArchiveMetadata FAILED:", err);
      return null;
    }
  };
  
  // Load archive tree entries
  const loadArchiveTree = async (containerPath: string): Promise<ArchiveTreeEntry[]> => {
    const startTime = performance.now();
    log.debug(`loadArchiveTree called for ${containerPath}`);
    const cached = archiveTreeCache().get(containerPath);
    if (cached) {
      log.debug(`loadArchiveTree - returning ${cached.length} cached entries (${(performance.now() - startTime).toFixed(1)}ms)`);
      return cached;
    }

    try {
      log.debug("loadArchiveTree - invoking archive_get_tree...");
      const invokeStart = performance.now();
      const entries = await invoke<ArchiveTreeEntry[]>("archive_get_tree", {
        containerPath,
      });
      log.debug(`loadArchiveTree - backend returned ${entries.length} entries in ${(performance.now() - invokeStart).toFixed(1)}ms`);
      
      setArchiveTreeCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, entries);
        return next;
      });
      
      log.debug(`loadArchiveTree - total time: ${(performance.now() - startTime).toFixed(1)}ms`);
      return entries;
    } catch (err) {
      log.error("loadArchiveTree FAILED:", err);
      return [];
    }
  };

  // Sort entries: directories first, then alphabetically
  const sortArchiveEntries = (entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.path.localeCompare(b.path);
    });
  };

  // Synthesize virtual directory entries from file paths with calculated sizes
  // This handles archives that don't include explicit directory entries
  const synthesizeDirectories = (entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] => {
    const existingPaths = new Set(entries.map(e => e.path.replace(/\/$/, '')));
    const syntheticDirs = new Map<string, ArchiveTreeEntry>();
    const dirSizes = new Map<string, number>();
    
    // First pass: build all synthetic directories
    for (const entry of entries) {
      const parts = entry.path.replace(/\/$/, '').split('/');
      // Build all ancestor directories and accumulate sizes
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
            compressedSize: 0,
            crc32: 0,
            modified: '',
          });
        }
      }
    }
    
    // Second pass: set directory sizes
    for (const [dirPath, dir] of syntheticDirs) {
      dir.size = dirSizes.get(dirPath) || 0;
    }
    
    // Also update sizes for existing directories that have child file sizes
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

  // Get root-level archive entries (with synthetic directories)
  const getArchiveRootEntries = (entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] => {
    // First, synthesize any missing directory entries
    const allEntries = synthesizeDirectories(entries);
    
    return allEntries.filter(entry => {
      const path = entry.path.replace(/\/$/, '');
      return !path.includes('/');
    });
  };

  // Get children of a specific archive directory path (with synthetic directories)
  const getArchiveChildren = (entries: ArchiveTreeEntry[], parentPath: string): ArchiveTreeEntry[] => {
    // First, synthesize any missing directory entries
    const allEntries = synthesizeDirectories(entries);
    
    const normalizedParent = parentPath.replace(/\/$/, '');
    return allEntries.filter(entry => {
      const entryPath = entry.path.replace(/\/$/, '');
      if (!entryPath.startsWith(normalizedParent + '/')) return false;
      const remaining = entryPath.substring(normalizedParent.length + 1);
      return !remaining.includes('/');
    });
  };

  // Check if an archive directory is expanded
  const isArchiveDirExpanded = (containerPath: string, archivePath: string): boolean => {
    const key = `${containerPath}::archive::${archivePath}`;
    return expandedArchivePaths().has(key);
  };

  // Toggle archive directory expansion
  const toggleArchiveDir = (containerPath: string, archivePath: string): void => {
    const key = `${containerPath}::archive::${archivePath}`;
    setExpandedArchivePaths(prev => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  };

  // Check if entry is a nested container
  const isNestedContainer = (entry: ArchiveTreeEntry): boolean => {
    return !entry.isDir && isContainerFile(entry.name || entry.path);
  };

  // Open a nested container (extract from archive and add as new container)
  const openNestedContainer = async (
    containerPath: string, 
    entryPath: string, 
    entryName: string,
    onOpenNestedContainer: ((tempPath: string, originalName: string, containerType: string, parentPath: string) => void) | undefined,
    _loading: Set<string>,
    setLoading: (fn: (prev: Set<string>) => Set<string>) => void
  ): Promise<void> => {
    if (!onOpenNestedContainer) {
      log.warn("No callback provided for nested containers");
      return;
    }
    
    const nodeKey = `${containerPath}::nested::${entryPath}`;
    setLoading(prev => new Set([...prev, nodeKey]));
    
    try {
      // Extract the nested container to a temp file
      const tempPath = await invoke<string>("archive_extract_entry", {
        containerPath,
        entryPath,
      });
      
      // Determine container type from filename
      const containerType = getContainerType(entryName);
      
      // Call the callback to add this as a new discovered file
      onOpenNestedContainer(tempPath, entryName, containerType, containerPath);
    } catch (err) {
      log.error("Failed to extract:", err);
    } finally {
      setLoading(prev => {
        const next = new Set(prev);
        next.delete(nodeKey);
        return next;
      });
    }
  };

  // Expand all archive directories for a container
  const expandAllArchiveDirs = (_containerPath: string, dirKeys: string[]): void => {
    setExpandedArchivePaths(prev => {
      const next = new Set(prev);
      dirKeys.forEach(key => next.add(key));
      return next;
    });
  };

  // Collapse all archive directories
  const collapseAllArchiveDirs = (): void => {
    setExpandedArchivePaths(new Set<string>());
  };

  // Restore expanded paths from serialized state
  const restoreExpandedPaths = (paths: string[]): void => {
    setExpandedArchivePaths(new Set(paths));
  };

  return {
    archiveTreeCache,
    archiveMetaCache,
    expandedArchivePaths,
    loadArchiveMetadata,
    loadArchiveTree,
    toggleArchiveDir,
    openNestedContainer,
    getArchiveRootEntries,
    getArchiveChildren,
    isArchiveDirExpanded,
    sortArchiveEntries,
    isNestedContainer,
    expandAllArchiveDirs,
    collapseAllArchiveDirs,
    restoreExpandedPaths,
  };
}
