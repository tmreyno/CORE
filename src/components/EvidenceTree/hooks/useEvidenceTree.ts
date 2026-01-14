// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useEvidenceTree - Master composing hook for EvidenceTree component
 * 
 * Combines all container-specific hooks (AD1, VFS, Archive, Lazy/UFED) into
 * a single unified API for the EvidenceTree component.
 */

import { createSignal, createMemo, Accessor } from "solid-js";
import { useAd1Tree } from "./useAd1Tree";
import { useVfsTree } from "./useVfsTree";
import { useArchiveTree } from "./useArchiveTree";
import { useLazyTree } from "./useLazyTree";
import type { DiscoveredFile, TreeEntry, VfsMountInfo, VfsEntry, ArchiveTreeEntry, UfedTreeEntry, Ad1ContainerSummary } from "../../../types";
import type { LazyTreeEntry, ContainerSummary } from "../../../types/lazy-loading";
import { 
  isVfsContainer, 
  isAd1Container, 
  isArchiveContainer, 
  isUfedContainer 
} from "../containerDetection";

/** Selected entry passed to parent for viewing */
export interface SelectedEntry {
  containerPath: string;
  entryPath: string;
  name: string;
  size: number;
  isDir: boolean;
  isVfsEntry?: boolean;
  dataAddr?: number | null;
  itemAddr?: number | null;
  compressedSize?: number | null;
  dataEndAddr?: number | null;
  metadataAddr?: number | null;
  firstChildAddr?: number | null;
}

export interface UseEvidenceTreeProps {
  discoveredFiles: Accessor<DiscoveredFile[]>;
  typeFilter: Accessor<string | null>;
  onSelectEntry: (entry: SelectedEntry) => void;
  onOpenNestedContainer?: (tempPath: string, originalName: string, containerType: string, parentPath: string) => void;
}

export interface UseEvidenceTreeReturn {
  // Filtered files
  filteredFiles: Accessor<DiscoveredFile[]>;
  
  // Container expansion
  expandedContainers: Accessor<Set<string>>;
  isContainerExpanded: (path: string) => boolean;
  toggleContainer: (file: DiscoveredFile) => Promise<void>;
  
  // Loading state
  loading: Accessor<Set<string>>;
  isLoading: (key: string) => boolean;
  
  // Selection
  selectedEntryKey: Accessor<string | null>;
  setSelectedEntryKey: (key: string | null) => void;
  isSelected: (key: string) => boolean;
  
  // AD1 specific
  ad1: ReturnType<typeof useAd1Tree>;
  getAd1Info: (containerPath: string) => Ad1ContainerSummary | null;
  getAd1RootChildren: (containerPath: string) => TreeEntry[];
  
  // VFS specific (E01, Raw, L01)
  vfs: ReturnType<typeof useVfsTree>;
  getVfsMountInfo: (containerPath: string) => VfsMountInfo | null;
  
  // Archive specific (ZIP, 7z, TAR)
  archive: ReturnType<typeof useArchiveTree>;
  getArchiveEntries: (containerPath: string) => ArchiveTreeEntry[];
  getArchiveRootEntries: (containerPath: string) => ArchiveTreeEntry[];
  
  // Lazy loading (UFED, large containers)
  lazy: ReturnType<typeof useLazyTree>;
  getLazySummary: (containerPath: string) => ContainerSummary | null;
  getLazyRootEntries: (containerPath: string) => LazyTreeEntry[];
  hasLazyData: (containerPath: string) => boolean;
  
  // Entry click handler
  handleEntryClick: (containerPath: string, entry: TreeEntry) => void;
  
  // Sorting utilities
  sortEntries: (entries: TreeEntry[]) => TreeEntry[];
  sortVfsEntries: (entries: VfsEntry[]) => VfsEntry[];
  sortArchiveEntries: (entries: ArchiveTreeEntry[]) => ArchiveTreeEntry[];
  sortLazyEntries: (entries: LazyTreeEntry[]) => LazyTreeEntry[];
  sortUfedEntries: (entries: UfedTreeEntry[]) => UfedTreeEntry[];
}

/**
 * Master hook that composes all tree management functionality
 */
export function useEvidenceTree(props: UseEvidenceTreeProps): UseEvidenceTreeReturn {
  // Container expansion state
  const [expandedContainers, setExpandedContainers] = createSignal<Set<string>>(new Set());
  const [loading, setLoading] = createSignal<Set<string>>(new Set());
  const [selectedEntryKey, setSelectedEntryKey] = createSignal<string | null>(null);
  
  // Initialize container-specific hooks
  const ad1 = useAd1Tree();
  const vfs = useVfsTree();
  const archive = useArchiveTree();
  const lazy = useLazyTree();
  
  // Filtered files based on type filter
  const filteredFiles = createMemo(() => {
    const filter = props.typeFilter();
    const files = props.discoveredFiles();
    return filter ? files.filter(f => f.container_type === filter) : files;
  });
  
  // Container expansion helpers
  const isContainerExpanded = (path: string): boolean => expandedContainers().has(path);
  
  const isLoading = (key: string): boolean => loading().has(key);
  
  const isSelected = (key: string): boolean => selectedEntryKey() === key;
  
  // Set loading state helper
  const setLoadingState = (key: string, isLoadingNow: boolean) => {
    setLoading(prev => {
      const next = new Set(prev);
      if (isLoadingNow) next.add(key);
      else next.delete(key);
      return next;
    });
  };
  
  // Toggle container expansion - dispatches to appropriate handler
  const toggleContainer = async (file: DiscoveredFile): Promise<void> => {
    const path = file.path;
    const expanded = new Set(expandedContainers());
    
    if (expanded.has(path)) {
      expanded.delete(path);
      setExpandedContainers(expanded);
      return;
    }
    
    const containerType = file.container_type.toLowerCase();
    
    if (isVfsContainer(containerType)) {
      // Mount VFS container (E01, Raw, L01)
      if (!vfs.vfsMountCache().has(path)) {
        setLoadingState(path, true);
        await vfs.mountVfsContainer(path);
        setLoadingState(path, false);
      }
    } else if (isArchiveContainer(containerType)) {
      // Load archive metadata first (fast), then tree
      setLoadingState(path, true);
      await archive.loadArchiveMetadata(path);
      expanded.add(path);
      setExpandedContainers(new Set(expanded));
      
      if (!archive.archiveTreeCache().has(path)) {
        await archive.loadArchiveTree(path);
      }
      setLoadingState(path, false);
      return;
    } else if (isUfedContainer(containerType)) {
      // UFED uses lazy loading
      setLoadingState(path, true);
      await lazy.loadLazySummary(path);
      expanded.add(path);
      setExpandedContainers(new Set(expanded));
      
      const rootKey = `${path}::lazy::root`;
      if (!lazy.lazyChildrenCache().has(rootKey)) {
        await lazy.loadLazyRootChildren(path, 0, 100);
      }
      setLoadingState(path, false);
      return;
    } else if (isAd1Container(containerType)) {
      // AD1 container - load tree and info
      const cacheKey = `${path}::root`;
      expanded.add(path);
      setExpandedContainers(expanded);
      
      if (!ad1.childrenCache().has(cacheKey)) {
        setLoadingState(path, true);
        await Promise.all([
          ad1.loadRootChildren(path),
          ad1.loadAd1Info(path),
        ]);
        setLoadingState(path, false);
      }
      return;
    }
    
    expanded.add(path);
    setExpandedContainers(expanded);
  };
  
  // AD1 getters
  const getAd1Info = (containerPath: string): Ad1ContainerSummary | null => {
    return ad1.ad1InfoCache().get(containerPath) || null;
  };
  
  const getAd1RootChildren = (containerPath: string): TreeEntry[] => {
    const cacheKey = `${containerPath}::root`;
    const entries = ad1.childrenCache().get(cacheKey) || [];
    return sortEntries(entries);
  };
  
  // VFS getters
  const getVfsMountInfo = (containerPath: string): VfsMountInfo | null => {
    return vfs.vfsMountCache().get(containerPath) || null;
  };
  
  // Archive getters
  const getArchiveEntries = (containerPath: string): ArchiveTreeEntry[] => {
    return archive.archiveTreeCache().get(containerPath) || [];
  };
  
  const getArchiveRootEntries = (containerPath: string): ArchiveTreeEntry[] => {
    const entries = getArchiveEntries(containerPath);
    // Find entries at root level
    return entries.filter(entry => {
      const path = entry.path.replace(/\/$/, '');
      return !path.includes('/');
    });
  };
  
  // Lazy/UFED getters
  const getLazySummary = (containerPath: string): ContainerSummary | null => {
    return lazy.lazySummaryCache().get(containerPath) || null;
  };
  
  const getLazyRootEntries = (containerPath: string): LazyTreeEntry[] => {
    const entries = lazy.lazyChildrenCache().get(`${containerPath}::lazy::root`) || [];
    return sortLazyEntries(entries);
  };
  
  const hasLazyData = (containerPath: string): boolean => {
    return getLazyRootEntries(containerPath).length > 0 || getLazySummary(containerPath) !== null;
  };
  
  // Entry click handler for AD1 entries
  const handleEntryClick = (containerPath: string, entry: TreeEntry): void => {
    const entryKey = `${containerPath}::${entry.item_addr ?? entry.path}`;
    setSelectedEntryKey(entryKey);
    
    props.onSelectEntry({
      containerPath,
      entryPath: entry.path,
      name: entry.name,
      size: entry.size,
      isDir: entry.is_dir,
      isVfsEntry: false,
      dataAddr: entry.data_addr,
      itemAddr: entry.item_addr,
      compressedSize: entry.compressed_size,
      dataEndAddr: entry.data_end_addr,
      metadataAddr: entry.metadata_addr,
      firstChildAddr: entry.first_child_addr,
    });
    
    if (entry.is_dir) {
      ad1.toggleDirByAddr(containerPath, entry, loading(), setLoading);
    }
  };
  
  // Sorting utilities
  const sortEntries = (entries: TreeEntry[]): TreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  const sortVfsEntries = (entries: VfsEntry[]): VfsEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  const sortArchiveEntries = (entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.path.localeCompare(b.path);
    });
  };
  
  const sortLazyEntries = (entries: LazyTreeEntry[]): LazyTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  const sortUfedEntries = (entries: UfedTreeEntry[]): UfedTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  return {
    filteredFiles,
    expandedContainers,
    isContainerExpanded,
    toggleContainer,
    loading,
    isLoading,
    selectedEntryKey,
    setSelectedEntryKey,
    isSelected,
    ad1,
    getAd1Info,
    getAd1RootChildren,
    vfs,
    getVfsMountInfo,
    archive,
    getArchiveEntries,
    getArchiveRootEntries,
    lazy,
    getLazySummary,
    getLazyRootEntries,
    hasLazyData,
    handleEntryClick,
    sortEntries,
    sortVfsEntries,
    sortArchiveEntries,
    sortLazyEntries,
    sortUfedEntries,
  };
}
