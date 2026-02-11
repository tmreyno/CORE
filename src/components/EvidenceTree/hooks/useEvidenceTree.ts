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

import { createSignal, createMemo, Accessor, onMount, createEffect, on } from "solid-js";
import { useAd1Tree } from "./useAd1Tree";
import { useVfsTree } from "./useVfsTree";
import { useArchiveTree } from "./useArchiveTree";
import { useLazyTree } from "./useLazyTree";
import { useNestedContainers } from "./useNestedContainers";
import { getPreference } from "../../../components/preferences";
import { logger } from "../../../utils/logger";
import type { DiscoveredFile, TreeEntry, VfsMountInfo, VfsEntry, ArchiveTreeEntry, UfedTreeEntry, Ad1ContainerSummary } from "../../../types";

const log = logger.scope('EvidenceTree');
import type { LazyTreeEntry, ContainerSummary } from "../../../types/lazy-loading";
import type { SelectedEntry, TreeExpansionState } from "../types";
import { 
  isVfsContainer, 
  isAd1Container, 
  isArchiveContainer, 
  isUfedContainer,
  isUnsupportedVfsContainer,
} from "../containerDetection";

export interface UseEvidenceTreeProps {
  discoveredFiles: Accessor<DiscoveredFile[]>;
  typeFilter: Accessor<string | null>;
  onSelectEntry: (entry: SelectedEntry) => void;
  onOpenNestedContainer?: (tempPath: string, originalName: string, containerType: string, parentPath: string) => void;
  // Tree expansion state persistence
  initialExpansionState?: TreeExpansionState;
  onExpansionStateChange?: (state: TreeExpansionState) => void;
}

export interface UseEvidenceTreeReturn {
  // Filtered files
  filteredFiles: Accessor<DiscoveredFile[]>;
  
  // Container expansion
  expandedContainers: Accessor<Set<string>>;
  isContainerExpanded: (path: string) => boolean;
  toggleContainer: (file: DiscoveredFile) => Promise<void>;
  expandAllContainers: () => Promise<void>;
  collapseAllContainers: () => void;
  
  // Loading state
  loading: Accessor<Set<string>>;
  isLoading: (key: string) => boolean;
  setLoadingState: (key: string, isLoadingNow: boolean) => void;
  
  // Selection
  selectedEntryKey: Accessor<string | null>;
  setSelectedEntryKey: (key: string | null) => void;
  isSelected: (key: string) => boolean;
  
  // AD1 specific
  ad1: ReturnType<typeof useAd1Tree>;
  getAd1Info: (containerPath: string) => Ad1ContainerSummary | null;
  getAd1RootChildren: (containerPath: string) => TreeEntry[];
  getAd1ContainerStatus: (containerPath: string) => { isComplete: boolean; statusMessage: string; expectedSegments: number; availableSegments: number; missingSegments: number[] } | undefined;
  
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
  
  // Nested containers (containers inside other containers)
  nested: ReturnType<typeof useNestedContainers>;
  
  // Entry click handler
  handleEntryClick: (containerPath: string, entry: TreeEntry) => void;
  
  // Sorting utilities
  sortEntries: (entries: TreeEntry[]) => TreeEntry[];
  sortVfsEntries: (entries: VfsEntry[]) => VfsEntry[];
  sortArchiveEntries: (entries: ArchiveTreeEntry[]) => ArchiveTreeEntry[];
  sortLazyEntries: (entries: LazyTreeEntry[]) => LazyTreeEntry[];
  sortUfedEntries: (entries: UfedTreeEntry[]) => UfedTreeEntry[];
  
  // State persistence for project save/restore
  getExpansionState: () => TreeExpansionState;
  restoreExpansionState: (state: TreeExpansionState) => void;
}

/**
 * Master hook that composes all tree management functionality
 */
export function useEvidenceTree(props: UseEvidenceTreeProps): UseEvidenceTreeReturn {
  log.debug(' Hook called/re-created');
  
  // Container expansion state
  const [expandedContainers, setExpandedContainers] = createSignal<Set<string>>(new Set());
  const [loading, setLoading] = createSignal<Set<string>>(new Set());
  const [selectedEntryKey, setSelectedEntryKey] = createSignal<string | null>(null);
  
  // Guard against concurrent toggle operations on the same container
  const pendingToggles = new Set<string>();
  
  // Initialize container-specific hooks
  const ad1 = useAd1Tree();
  const vfs = useVfsTree();
  const archive = useArchiveTree();
  const lazy = useLazyTree();
  const nested = useNestedContainers();
  
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
    
    // Guard against concurrent toggles on the same container
    if (pendingToggles.has(path)) {
      log.debug(' SKIPPED - already toggling:', path);
      return;
    }
    pendingToggles.add(path);
    
    try {
      const expanded = new Set(expandedContainers());
      
      log.debug(' path:', path, 'currently expanded:', expanded.has(path), 'set size:', expanded.size);
      console.trace('[toggleContainer] call stack');
      
      if (expanded.has(path)) {
        expanded.delete(path);
        setExpandedContainers(new Set(expanded));
        log.debug(' COLLAPSED - new size:', expandedContainers().size);
        return;
      }
    
    const containerType = file.container_type.toLowerCase();
    
    // Check for unsupported VFS containers (currently none - DMG now uses archive interface)
    if (isUnsupportedVfsContainer(containerType)) {
      console.warn(`[toggleContainer] Container type '${containerType}' is not yet supported for browsing:`, path);
      // Just expand to show a "not supported" message in the UI
      expanded.add(path);
      setExpandedContainers(new Set(expanded));
      return;
    }
    
    if (isVfsContainer(containerType)) {
      // Mount VFS container (E01, Raw, L01)
      const needsMount = !vfs.vfsMountCache().has(path);
      
      // Set loading BEFORE expanding to prevent "Empty container" flash
      if (needsMount) {
        setLoadingState(path, true);
      }
      
      expanded.add(path);
      setExpandedContainers(new Set(expanded));
      
      if (needsMount) {
        await vfs.mountVfsContainer(path);
        setLoadingState(path, false);
      }
      return;
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
      // AD1 container - load tree, info, and status
      const cacheKey = `${path}::root`;
      const needsLoad = !ad1.childrenCache().has(cacheKey);
      
      // Set loading BEFORE expanding to prevent "Empty container" flash
      if (needsLoad) {
        setLoadingState(path, true);
      }
      
      expanded.add(path);
      setExpandedContainers(new Set(expanded));
      log.debug(' AD1 EXPANDED - set contains path:', expandedContainers().has(path), 'size:', expandedContainers().size);
      
      if (needsLoad) {
        await Promise.all([
          ad1.loadRootChildren(path),
          ad1.loadAd1Info(path),
          ad1.loadContainerStatus(path),  // Load segment status for incomplete container detection
        ]);
        setLoadingState(path, false);
        log.debug(' AD1 loading complete - still expanded:', expandedContainers().has(path));
      }
      return;
    }
    
    expanded.add(path);
    setExpandedContainers(new Set(expanded));
    } finally {
      pendingToggles.delete(path);
    }
  };
  
  // Expand all containers and their internal directories (parallel loading)
  const expandAllContainers = async (): Promise<void> => {
    const files = filteredFiles();
    
    // First, expand all top-level containers IN PARALLEL
    // This significantly speeds up loading when multiple containers are present
    const unexpandedFiles = files.filter(file => !isContainerExpanded(file.path));
    await Promise.all(unexpandedFiles.map(file => toggleContainer(file)));
    
    // Then expand all internal directories for each container type (also parallel)
    // Group files by container type for parallel processing
    const archiveFiles: typeof files = [];
    const vfsFiles: typeof files = [];
    const ad1Files: typeof files = [];
    const ufedFiles: typeof files = [];
    
    for (const file of files) {
      const containerType = file.container_type.toLowerCase();
      if (isArchiveContainer(containerType)) archiveFiles.push(file);
      else if (isVfsContainer(containerType)) vfsFiles.push(file);
      else if (isAd1Container(containerType)) ad1Files.push(file);
      else if (isUfedContainer(containerType)) ufedFiles.push(file);
    }
    
    // Archive containers - expand all directories (synchronous, just sets state)
    for (const file of archiveFiles) {
      const entries = archive.archiveTreeCache().get(file.path) || [];
      const dirPaths = entries
        .filter(e => e.isDir)
        .map(e => `${file.path}::${e.path}`);
      if (dirPaths.length > 0) {
        archive.expandAllArchiveDirs(file.path, dirPaths);
      }
    }
    
    // Expand VFS, AD1, and UFED directories in parallel
    await Promise.all([
      // VFS containers - expand all directories
      ...vfsFiles.map(file => vfs.expandAllVfsDirs(file.path)),
      // AD1 containers - expand root directories
      ...ad1Files.map(file => ad1.expandAllAd1Dirs(file.path, loading(), setLoading)),
      // Lazy/UFED containers - expand root level
      ...ufedFiles.map(file => lazy.expandAllLazyDirs(file.path)),
    ]);
  };
  
  // Collapse all containers (clears all expansion states)
  const collapseAllContainers = (): void => {
    setExpandedContainers(new Set<string>());
    // Also collapse internal directories
    ad1.collapseAllDirs();
    vfs.collapseAllVfsDirs();
    archive.collapseAllArchiveDirs();
    lazy.collapseAllLazyDirs();
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
  
  const getAd1ContainerStatus = (containerPath: string) => {
    return ad1.getContainerStatus(containerPath);
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
  
  // Sorting utilities with hidden file filtering
  const filterHidden = <T extends { name: string }>(entries: T[]): T[] => {
    if (getPreference("showHiddenFiles")) {
      return entries;
    }
    return entries.filter(e => !e.name.startsWith("."));
  };
  
  const filterHiddenByPath = <T extends { path: string }>(entries: T[]): T[] => {
    if (getPreference("showHiddenFiles")) {
      return entries;
    }
    return entries.filter(e => {
      const name = e.path.split("/").pop() || "";
      return !name.startsWith(".");
    });
  };
  
  const sortEntries = (entries: TreeEntry[]): TreeEntry[] => {
    return filterHidden([...entries]).sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  const sortVfsEntries = (entries: VfsEntry[]): VfsEntry[] => {
    return filterHidden([...entries]).sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  const sortArchiveEntries = (entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] => {
    return filterHiddenByPath([...entries]).sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.path.localeCompare(b.path);
    });
  };
  
  const sortLazyEntries = (entries: LazyTreeEntry[]): LazyTreeEntry[] => {
    return filterHidden([...entries]).sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  const sortUfedEntries = (entries: UfedTreeEntry[]): UfedTreeEntry[] => {
    return filterHidden([...entries]).sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  /**
   * Get all expansion state for project persistence.
   * Converts Sets to arrays for JSON serialization.
   */
  const getExpansionState = (): TreeExpansionState => {
    return {
      containers: Array.from(expandedContainers()),
      vfs: Array.from(vfs.expandedVfsPaths()),
      archive: Array.from(archive.expandedArchivePaths()),
      lazy: Array.from(lazy.expandedLazyPaths()),
      ad1: Array.from(ad1.expandedDirs()),
      selectedKey: selectedEntryKey(),
    };
  };
  
  /**
   * Restore expansion state from project load.
   * Note: This only restores UI state - cached data must be reloaded separately.
   */
  const restoreExpansionState = (state: TreeExpansionState): void => {
    log.debug(' Restoring tree expansion state:', {
      containers: state.containers.length,
      vfs: state.vfs.length,
      archive: state.archive.length,
      lazy: state.lazy.length,
      ad1: state.ad1.length,
      selectedKey: state.selectedKey,
    });
    
    // Restore container-level expansion
    setExpandedContainers(new Set(state.containers));
    
    // Restore sub-tree expansions
    vfs.restoreExpandedPaths(state.vfs);
    archive.restoreExpandedPaths(state.archive);
    lazy.restoreExpandedPaths(state.lazy);
    ad1.restoreExpandedDirs(state.ad1);
    
    // Restore selection
    if (state.selectedKey) {
      setSelectedEntryKey(state.selectedKey);
    }
  };
  
  // Initialize from props.initialExpansionState if provided
  onMount(() => {
    if (props.initialExpansionState) {
      log.debug(' Restoring initial expansion state');
      restoreExpansionState(props.initialExpansionState);
    }
  });
  
  // Auto-expand containers when new files are discovered (if preference enabled)
  // Track which containers we've auto-expanded to avoid repeated expansion
  const autoExpandedContainers = new Set<string>();
  
  createEffect(on(
    () => filteredFiles(),
    async (files) => {
      if (!getPreference("autoExpandTree")) return;
      
      // Find newly discovered containers that haven't been auto-expanded yet
      const newContainers = files.filter(f => !autoExpandedContainers.has(f.path) && !isContainerExpanded(f.path));
      
      if (newContainers.length === 0) return;
      
      log.debug(' Auto-expanding new containers:', newContainers.map(f => f.path));
      
      // Mark as auto-expanded and expand
      for (const file of newContainers) {
        autoExpandedContainers.add(file.path);
        await toggleContainer(file);
      }
    },
    { defer: true }
  ));
  
  // Track state changes and notify parent via callback
  // Using a combined effect that watches all relevant expansion states
  createEffect(on(
    // Watch all expansion-related signals
    () => ({
      containers: expandedContainers(),
      vfs: vfs.expandedVfsPaths(),
      archive: archive.expandedArchivePaths(),
      lazy: lazy.expandedLazyPaths(),
      ad1: ad1.expandedDirs(),
      selectedKey: selectedEntryKey(),
    }),
    (_current, prev) => {
      // Skip the first run (initialization) and if no callback provided
      if (!props.onExpansionStateChange || prev === undefined) return;
      
      // Emit the new state
      const state = getExpansionState();
      props.onExpansionStateChange(state);
    },
    { defer: true } // Don't run on first render
  ));
  
  return {
    filteredFiles,
    expandedContainers,
    isContainerExpanded,
    toggleContainer,
    expandAllContainers,
    collapseAllContainers,
    loading,
    isLoading,
    setLoadingState,
    selectedEntryKey,
    setSelectedEntryKey,
    isSelected,
    ad1,
    getAd1Info,
    getAd1RootChildren,
    getAd1ContainerStatus,
    vfs,
    getVfsMountInfo,
    archive,
    getArchiveEntries,
    getArchiveRootEntries,
    lazy,
    getLazySummary,
    getLazyRootEntries,
    hasLazyData,
    nested,
    handleEntryClick,
    sortEntries,
    sortVfsEntries,
    sortArchiveEntries,
    sortLazyEntries,
    sortUfedEntries,
    getExpansionState,
    restoreExpansionState,
  };
}
