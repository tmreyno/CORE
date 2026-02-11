// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceTreeLazy - Unified lazy-loading hierarchical view for ALL forensic evidence containers
 * 
 * UNIFIED TREE COMPONENT - Supports all container types with consistent UI:
 * 
 * AD1 Containers (AccessData Logical Image):
 * - Uses V2 backend APIs for 50x faster startup
 * - ADDRESS-BASED navigation for instant performance
 * - Lazy-loads directory contents on demand
 * 
 * VFS Containers (E01, Raw, L01):
 * - Mounts partitions with detected filesystems
 * - Navigates FAT/NTFS filesystems directly
 * - Lazy-loads directory contents on demand
 * 
 * Archive Containers (ZIP, 7z, RAR, TAR):
 * - Loads archive tree on expansion
 * - Shows compressed/uncompressed sizes
 * 
 * UFED Containers (Mobile extractions):
 * - Displays mobile extraction tree
 * - Shows entry types and hashes
 * 
 * All containers use standardized tree components with Tailwind CSS.
 */

import { For, Show, createSignal, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineCircleStack,
  HiOutlineMapPin,
  HiOutlineXMark,
  HiOutlineServerStack,
  HiOutlineRectangleStack,
  HiOutlineCommandLine,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineArrowPath,
  HiOutlineChevronDown,
  HiOutlineCpuChip,
} from "./icons";
import { 
  TreeRow, 
  TreeEmptyState, 
  TreeErrorState, 
  ExpandIcon, 
  ContainerHeader,
  TREE_ROW_BASE_CLASSES,
  TREE_ROW_NORMAL_CLASSES,
  TREE_INFO_BAR_CLASSES,
  TREE_INFO_BAR_PADDING,
  getTreeIndent,
  getContainerTypeIcon,
} from "./tree";
import type { DiscoveredFile, TreeEntry, VfsMountInfo, VfsEntry, VfsPartitionInfo, ArchiveTreeEntry, UfedTreeEntry, Ad1ContainerSummary } from "../types";
import type { LazyTreeEntry, ContainerSummary } from "../types/lazy-loading";
import { formatBytes } from "../utils";
import { getContainerSummary, getRootChildren, getChildren } from "../hooks/useLazyLoading";
import {
  isVfsContainer,
  isL01Container,
  isAd1Container,
  isArchiveContainer,
  isUfedContainer,
  isMemoryDump,
  isMemoryDumpFile,
  isContainerFile,
  getContainerType,
} from "./EvidenceTree/containerDetection";

/** Props for selecting an entry to view - includes hex location info */
export interface SelectedEntry {
  containerPath: string;
  entryPath: string;
  name: string;
  size: number;
  isDir: boolean;
  /** Whether this entry is from a VFS container (E01/Raw) */
  isVfsEntry?: boolean;
  /** Whether this is a disk file (not inside a container) */
  isDiskFile?: boolean;
  /** Container type for special handling */
  containerType?: string;
  /** Additional metadata */
  metadata?: Record<string, unknown>;
  /** Direct address for reading file data */
  dataAddr?: number | null;
  // === HEX LOCATION FIELDS ===
  /** Address of the item header in the container */
  itemAddr?: number | null;
  /** Size of compressed data in bytes */
  compressedSize?: number | null;
  /** Address where compressed data ends */
  dataEndAddr?: number | null;
  /** Address of first metadata entry for this item */
  metadataAddr?: number | null;
  /** Address of first child (for folders) */
  firstChildAddr?: number | null;
}

/** Quick metadata for archives (fast header-only read) */
interface ArchiveQuickMetadata {
  entry_count: number;
  archive_size: number;
  format: string;
  encrypted: boolean;
}

interface EvidenceTreeLazyProps {
  discoveredFiles: DiscoveredFile[];
  activeFile: DiscoveredFile | null;
  busy: boolean;
  onSelectContainer: (file: DiscoveredFile) => void;
  onSelectEntry: (entry: SelectedEntry) => void;
  typeFilter: string | null;
  onToggleTypeFilter: (type: string) => void;
  onClearTypeFilter: () => void;
  containerStats: Record<string, number>;
  /** Callback to open a nested container (container inside an archive) */
  onOpenNestedContainer?: (tempPath: string, originalName: string, containerType: string, parentPath: string) => void;
}

export function EvidenceTreeLazy(props: EvidenceTreeLazyProps) {
  // Track expanded containers
  const [expandedContainers, setExpandedContainers] = createSignal<Set<string>>(new Set());
  // Track expanded directories by unique key (containerPath::addr)
  const [expandedDirs, setExpandedDirs] = createSignal<Set<string>>(new Set());
  // Track loading states
  const [loading, setLoading] = createSignal<Set<string>>(new Set());
  // Cache children by key (containerPath::addr)
  const [childrenCache, setChildrenCache] = createSignal<Map<string, TreeEntry[]>>(new Map());
  // Track selected entry for highlighting
  const [selectedEntryKey, setSelectedEntryKey] = createSignal<string | null>(null);

  // === VFS State for disk images ===
  // Cache mount info by container path
  const [vfsMountCache, setVfsMountCache] = createSignal<Map<string, VfsMountInfo>>(new Map());
  // Cache VFS directory listings by key (containerPath::partition::path)
  const [vfsChildrenCache, setVfsChildrenCache] = createSignal<Map<string, VfsEntry[]>>(new Map());
  // Track expanded VFS paths
  const [expandedVfsPaths, setExpandedVfsPaths] = createSignal<Set<string>>(new Set());

  // === Archive State ===
  // Cache archive tree entries by container path
  const [archiveTreeCache, setArchiveTreeCache] = createSignal<Map<string, ArchiveTreeEntry[]>>(new Map());
  // Cache quick metadata for archives (nearly instant header-only reads)
  const [archiveMetaCache, setArchiveMetaCache] = createSignal<Map<string, ArchiveQuickMetadata>>(new Map());
  // Track expanded archive directories (containerPath::archivePath)
  const [expandedArchivePaths, setExpandedArchivePaths] = createSignal<Set<string>>(new Set());
  
  // === UFED State ===
  // Cache UFED tree entries by container path (legacy - used as fallback)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [ufedTreeCache, _setUfedTreeCache] = createSignal<Map<string, UfedTreeEntry[]>>(new Map());

  // === Lazy Loading State (unified for large containers) ===
  // Cache container summaries (entry counts, lazy loading recommendations)
  const [lazySummaryCache, setLazySummaryCache] = createSignal<Map<string, ContainerSummary>>(new Map());
  // Cache lazy-loaded children by key (containerPath::parentPath or containerPath::root)
  const [lazyChildrenCache, setLazyChildrenCache] = createSignal<Map<string, LazyTreeEntry[]>>(new Map());
  // Track total counts for each parent (for "showing X of Y" display)
  const [lazyTotalCounts, setLazyTotalCounts] = createSignal<Map<string, number>>(new Map());
  // Track if there are more entries to load for each parent
  const [lazyHasMore, setLazyHasMore] = createSignal<Map<string, boolean>>(new Map());
  // Track expanded lazy paths
  const [expandedLazyPaths, setExpandedLazyPaths] = createSignal<Set<string>>(new Set());

  // === AD1 Container Info ===
  // Cache AD1 container summary info (item counts, sizes)
  const [ad1InfoCache, setAd1InfoCache] = createSignal<Map<string, Ad1ContainerSummary>>(new Map());

  // Filtered container files
  const filteredFiles = createMemo(() => {
    const filter = props.typeFilter;
    return filter 
      ? props.discoveredFiles.filter(f => f.container_type === filter)
      : props.discoveredFiles;
  });

  // === AD1 Info Operations ===
  
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

  // === VFS Operations ===

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

  // Load VFS directory contents
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

  // Toggle VFS directory expansion
  const toggleVfsDir = async (containerPath: string, vfsPath: string) => {
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

  // Get cached VFS children
  const getVfsChildren = (containerPath: string, vfsPath: string): VfsEntry[] => {
    const cacheKey = `${containerPath}::vfs::${vfsPath}`;
    const children = vfsChildrenCache().get(cacheKey) || [];
    return sortVfsEntries(children);
  };

  // Sort VFS entries: directories first, then alphabetically
  const sortVfsEntries = (entries: VfsEntry[]): VfsEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };

  // === Archive Operations ===
  
  /** Fetch quick metadata (nearly instant - only reads headers) */
  const loadArchiveMetadata = async (containerPath: string): Promise<ArchiveQuickMetadata | null> => {
    const cached = archiveMetaCache().get(containerPath);
    if (cached) return cached;
    
    try {
      const meta = await invoke<ArchiveQuickMetadata>('archive_get_metadata', {
        containerPath,
      });
      
      console.log(`[loadArchiveMetadata] Got ${meta.format} with ${meta.entry_count} entries`);
      
      setArchiveMetaCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, meta);
        return next;
      });
      
      return meta;
    } catch (err) {
      console.error('[loadArchiveMetadata] Failed:', err);
      return null;
    }
  };
  
  // Load archive tree entries
  const loadArchiveTree = async (containerPath: string): Promise<ArchiveTreeEntry[]> => {
    const cached = archiveTreeCache().get(containerPath);
    if (cached) return cached;

    try {
      const entries = await invoke<ArchiveTreeEntry[]>("archive_get_tree", {
        containerPath,
      });
      
      setArchiveTreeCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, entries);
        return next;
      });
      
      return entries;
    } catch (err) {
      console.error("[loadArchiveTree] Failed:", err);
      return [];
    }
  };

  // Sort archive entries: directories first, then alphabetically
  const sortArchiveEntries = (entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.path.localeCompare(b.path);
    });
  };

  // Get root-level archive entries (entries with no parent or at root level)
  const getArchiveRootEntries = (entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] => {
    // Find entries that are at the root level (no '/' in path, or only at the end for dirs)
    return entries.filter(entry => {
      const path = entry.path.replace(/\/$/, ''); // Remove trailing slash
      return !path.includes('/');
    });
  };

  // Get children of a specific archive directory path
  const getArchiveChildren = (entries: ArchiveTreeEntry[], parentPath: string): ArchiveTreeEntry[] => {
    const normalizedParent = parentPath.replace(/\/$/, ''); // Remove trailing slash
    return entries.filter(entry => {
      const entryPath = entry.path.replace(/\/$/, '');
      // Check if this entry is a direct child of parentPath
      if (!entryPath.startsWith(normalizedParent + '/')) return false;
      // Get the remaining path after the parent
      const remaining = entryPath.substring(normalizedParent.length + 1);
      // Should not have any more slashes (direct child only)
      return !remaining.includes('/');
    });
  };

  // Check if an archive directory is expanded
  const isArchiveDirExpanded = (containerPath: string, archivePath: string): boolean => {
    const key = `${containerPath}::archive::${archivePath}`;
    return expandedArchivePaths().has(key);
  };

  // Toggle archive directory expansion
  const toggleArchiveDir = (containerPath: string, archivePath: string) => {
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

  // Open a nested container (extract from archive and add as new container)
  const openNestedContainer = async (containerPath: string, entryPath: string, entryName: string) => {
    if (!props.onOpenNestedContainer) {
      console.warn("[openNestedContainer] No callback provided for nested containers");
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
      props.onOpenNestedContainer(tempPath, entryName, containerType, containerPath);
    } catch (err) {
      console.error("[openNestedContainer] Failed to extract:", err);
      // Could show a toast notification here
    } finally {
      setLoading(prev => {
        const next = new Set(prev);
        next.delete(nodeKey);
        return next;
      });
    }
  };

  // === UFED Operations ===
  
  // NOTE: loadUfedTree is replaced by unified lazy loading API for large UFED containers
  // The old function loaded ALL entries at once, which was slow for 18GB+ containers
  // Now we use loadLazyRootChildren() and loadLazyChildren() for pagination
  
  // Sort UFED entries: directories first, then alphabetically
  const sortUfedEntries = (entries: UfedTreeEntry[]): UfedTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };

  // === Unified Lazy Loading Operations ===
  // Used for large containers (UFED ZIP, large archives, etc.)
  
  /** Get container summary with lazy loading recommendation */
  const loadLazySummary = async (containerPath: string): Promise<ContainerSummary | null> => {
    const cached = lazySummaryCache().get(containerPath);
    if (cached) return cached;
    
    try {
      const summary = await getContainerSummary(containerPath);
      setLazySummaryCache(prev => {
        const next = new Map(prev);
        next.set(containerPath, summary);
        return next;
      });
      return summary;
    } catch (err) {
      console.error("[loadLazySummary] Failed:", err);
      return null;
    }
  };
  
  /** Load lazy root children with pagination */
  const loadLazyRootChildren = async (
    containerPath: string, 
    offset: number = 0, 
    limit: number = 100
  ): Promise<LazyTreeEntry[]> => {
    const cacheKey = `${containerPath}::lazy::root`;
    
    // If offset is 0, check cache first
    if (offset === 0) {
      const cached = lazyChildrenCache().get(cacheKey);
      if (cached && cached.length > 0) return cached;
    }
    
    try {
      const result = await getRootChildren(containerPath, offset, limit);
      
      // Update total count
      setLazyTotalCounts(prev => {
        const next = new Map(prev);
        next.set(cacheKey, result.total_count);
        return next;
      });
      
      // Update has_more
      setLazyHasMore(prev => {
        const next = new Map(prev);
        next.set(cacheKey, result.has_more);
        return next;
      });
      
      // Append to cache (for pagination) or set (for initial load)
      setLazyChildrenCache(prev => {
        const next = new Map(prev);
        const existing = offset > 0 ? (next.get(cacheKey) || []) : [];
        next.set(cacheKey, [...existing, ...result.entries]);
        return next;
      });
      
      return result.entries;
    } catch (err) {
      console.error("[loadLazyRootChildren] Failed:", err);
      return [];
    }
  };
  
  /** Load lazy children at a specific path with pagination */
  const loadLazyChildren = async (
    containerPath: string,
    parentPath: string,
    offset: number = 0,
    limit: number = 100
  ): Promise<LazyTreeEntry[]> => {
    const cacheKey = `${containerPath}::lazy::${parentPath}`;
    
    // If offset is 0, check cache first
    if (offset === 0) {
      const cached = lazyChildrenCache().get(cacheKey);
      if (cached && cached.length > 0) return cached;
    }
    
    try {
      const result = await getChildren(containerPath, parentPath, offset, limit);
      
      // Update total count
      setLazyTotalCounts(prev => {
        const next = new Map(prev);
        next.set(cacheKey, result.total_count);
        return next;
      });
      
      // Update has_more
      setLazyHasMore(prev => {
        const next = new Map(prev);
        next.set(cacheKey, result.has_more);
        return next;
      });
      
      // Append to cache (for pagination) or set (for initial load)
      setLazyChildrenCache(prev => {
        const next = new Map(prev);
        const existing = offset > 0 ? (next.get(cacheKey) || []) : [];
        next.set(cacheKey, [...existing, ...result.entries]);
        return next;
      });
      
      return result.entries;
    } catch (err) {
      console.error("[loadLazyChildren] Failed:", err);
      return [];
    }
  };
  
  /** Sort lazy entries: directories first, then alphabetically */
  const sortLazyEntries = (entries: LazyTreeEntry[]): LazyTreeEntry[] => {
    return [...entries].sort((a, b) => {
      if (a.is_dir && !b.is_dir) return -1;
      if (!a.is_dir && b.is_dir) return 1;
      return a.name.localeCompare(b.name);
    });
  };
  
  /** Toggle lazy directory expansion */
  const toggleLazyDir = async (containerPath: string, entryPath: string) => {
    const nodeKey = `${containerPath}::lazy::${entryPath}`;
    const expanded = new Set(expandedLazyPaths());
    
    if (expanded.has(nodeKey)) {
      expanded.delete(nodeKey);
      setExpandedLazyPaths(expanded);
    } else {
      // Load children if not cached
      if (!lazyChildrenCache().has(nodeKey)) {
        setLoading(prev => new Set([...prev, nodeKey]));
        await loadLazyChildren(containerPath, entryPath);
        setLoading(prev => {
          const next = new Set(prev);
          next.delete(nodeKey);
          return next;
        });
      }
      expanded.add(nodeKey);
      setExpandedLazyPaths(expanded);
    }
  };
  
  /** Load more entries for a lazy path (pagination) */
  const loadMoreLazyEntries = async (containerPath: string, parentPath: string = "root") => {
    const cacheKey = parentPath === "root"
      ? `${containerPath}::lazy::root`
      : `${containerPath}::lazy::${parentPath}`;
    
    const currentEntries = lazyChildrenCache().get(cacheKey) || [];
    const offset = currentEntries.length;
    
    setLoading(prev => new Set([...prev, `${cacheKey}::more`]));
    
    if (parentPath === "root") {
      await loadLazyRootChildren(containerPath, offset, 100);
    } else {
      await loadLazyChildren(containerPath, parentPath, offset, 100);
    }
    
    setLoading(prev => {
      const next = new Set(prev);
      next.delete(`${cacheKey}::more`);
      return next;
    });
  };
  
  /** Check if currently loading more for a lazy path */
  const isLoadingMoreLazy = (containerPath: string, parentPath: string = "root"): boolean => {
    const cacheKey = parentPath === "root"
      ? `${containerPath}::lazy::root`
      : `${containerPath}::lazy::${parentPath}`;
    return loading().has(`${cacheKey}::more`);
  };
  
  /** Check if there are more entries to load */
  const hasMoreLazyEntries = (containerPath: string, parentPath: string = "root"): boolean => {
    const cacheKey = parentPath === "root"
      ? `${containerPath}::lazy::root`
      : `${containerPath}::lazy::${parentPath}`;
    return lazyHasMore().get(cacheKey) || false;
  };
  
  /** Get total count for a lazy path */
  const getLazyTotalCount = (containerPath: string, parentPath: string = "root"): number => {
    const cacheKey = parentPath === "root"
      ? `${containerPath}::lazy::root`
      : `${containerPath}::lazy::${parentPath}`;
    return lazyTotalCounts().get(cacheKey) || 0;
  };

  // Track loading errors per container
  const [containerErrors, setContainerErrors] = createSignal<Map<string, string>>(new Map());

  // Load root children - uses V2 API for better performance
  const loadRootChildren = async (containerPath: string, containerType: string): Promise<TreeEntry[]> => {
    const cacheKey = `${containerPath}::root`;
    
    const cached = childrenCache().get(cacheKey);
    if (cached) return cached;
    
    // Only AD1 containers support tree navigation
    if (!isAd1Container(containerType)) {
      return [];
    }
    
    try {
      // Use V2 API for AD1 - better performance with lazy loading
      const children = await invoke<TreeEntry[]>("container_get_root_children_v2", {
        containerPath,
      });
      
      // Clear any previous error for this container
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
      
      // Store the error for display
      setContainerErrors(prev => {
        const next = new Map(prev);
        next.set(containerPath, errorMsg);
        return next;
      });
      
      return [];
    }
  };

  // Load children by address - uses V2 API for best performance
  const loadChildrenByAddr = async (containerPath: string, addr: number, parentPath: string = ""): Promise<TreeEntry[]> => {
    const cacheKey = `${containerPath}::addr:${addr}`;
    
    const cached = childrenCache().get(cacheKey);
    if (cached) return cached;
    
    try {
      // Use V2 API for address-based loading - fastest method
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

  // Load children by path (fallback when addr is not available)
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

  // Toggle container expansion
  const toggleContainer = async (file: DiscoveredFile) => {
    const path = file.path;
    const expanded = new Set(expandedContainers());
    
    if (expanded.has(path)) {
      expanded.delete(path);
      setExpandedContainers(expanded);
    } else {
      // Check container type and load appropriate content
      if (isVfsContainer(file.container_type)) {
        // Mount the VFS container if not already mounted (E01, Raw, L01)
        if (!vfsMountCache().has(path)) {
          setLoading(prev => new Set([...prev, path]));
          await mountVfsContainer(path);
          setLoading(prev => {
            const next = new Set(prev);
            next.delete(path);
            return next;
          });
        }
      } else if (isArchiveContainer(file.container_type)) {
        // PERFORMANCE: For archives, load fast metadata first, expand immediately, 
        // then load full tree in background
        
        // Step 1: Load quick metadata (nearly instant - only reads ZIP EOCD/7z headers)
        setLoading(prev => new Set([...prev, path]));
        await loadArchiveMetadata(path);
        
        // Step 2: Expand immediately to show loading state with metadata preview
        expanded.add(path);
        setExpandedContainers(new Set(expanded));
        
        // Step 3: Load full tree in background (can be slow for large archives)
        if (!archiveTreeCache().has(path)) {
          await loadArchiveTree(path);
        }
        
        setLoading(prev => {
          const next = new Set(prev);
          next.delete(path);
          return next;
        });
        return; // Already expanded above
      } else if (isUfedContainer(file.container_type)) {
        // UFED containers use LAZY LOADING for large ZIP files (e.g., 18GB extractions)
        // Step 1: Get container summary first to determine size
        setLoading(prev => new Set([...prev, path]));
        
        const summary = await loadLazySummary(path);
        console.log(`[toggleContainer] UFED summary for ${file.filename}:`, summary);
        
        // Step 2: Expand immediately to show loading state
        expanded.add(path);
        setExpandedContainers(new Set(expanded));
        
        // Step 3: Load root children with lazy loading (paginated)
        const rootKey = `${path}::lazy::root`;
        if (!lazyChildrenCache().has(rootKey)) {
          await loadLazyRootChildren(path, 0, 100);
        }
        
        setLoading(prev => {
          const next = new Set(prev);
          next.delete(path);
          return next;
        });
        return; // Already expanded above
      } else if (isAd1Container(file.container_type)) {
        // Regular AD1 container - load tree and info
        const cacheKey = `${path}::root`;
        // Expand immediately to show loading state
        expanded.add(path);
        setExpandedContainers(expanded);
        
        if (!childrenCache().has(cacheKey)) {
          setLoading(prev => new Set([...prev, path]));
          // Load both tree children and container info in parallel
          await Promise.all([
            loadRootChildren(path, file.container_type),
            loadAd1Info(path),
          ]);
          setLoading(prev => {
            const next = new Set(prev);
            next.delete(path);
            return next;
          });
        }
        return; // Already expanded above
      }
      expanded.add(path);
      setExpandedContainers(expanded);
    }
  };

  // Toggle directory expansion - supports both address-based and path-based loading
  const toggleDirByAddr = async (containerPath: string, entry: TreeEntry) => {
    // Use item_addr (the parent's address) for V2 API - it will read first_child_addr internally
    const addr = entry.item_addr;
    const entryPath = entry.path;
    
    // Use address if available, otherwise fall back to path
    const nodeKey = addr 
      ? `${containerPath}::addr:${addr}` 
      : `${containerPath}::path:${entryPath}`;
    
    const expanded = new Set(expandedDirs());
    
    if (expanded.has(nodeKey)) {
      expanded.delete(nodeKey);
      setExpandedDirs(expanded);
    } else {
      // Expand immediately to show loading state (better UX)
      expanded.add(nodeKey);
      setExpandedDirs(expanded);
      
      if (!childrenCache().has(nodeKey)) {
        setLoading(prev => new Set([...prev, nodeKey]));
        
        // Load by address if available, otherwise by path
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

  // Get cached children - supports both address and path keys
  const getChildrenForEntry = (containerPath: string, entry: TreeEntry): TreeEntry[] => {
    // Use item_addr for cache key (matches V2 API which takes parent address)
    const addr = entry.item_addr;
    const cacheKey = addr 
      ? `${containerPath}::addr:${addr}` 
      : `${containerPath}::path:${entry.path}`;
    const children = childrenCache().get(cacheKey) || [];
    return sortEntries(children);
  };

  // Check if directory is expanded
  const isDirExpanded = (containerPath: string, entry: TreeEntry): boolean => {
    // Use item_addr for key (matches V2 API which takes parent address)
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

  // Handle entry click
  const handleEntryClick = (containerPath: string, entry: TreeEntry) => {
    // Set the selected entry key for highlighting
    const entryKey = `${containerPath}::${entry.item_addr ?? entry.path}`;
    setSelectedEntryKey(entryKey);
    
    if (entry.is_dir) {
      toggleDirByAddr(containerPath, entry);
      // Also notify parent about folder selection for metadata display
      props.onSelectEntry({
        containerPath,
        entryPath: entry.path,
        name: entry.name,
        size: entry.size,
        isDir: true,
        isVfsEntry: false, // AD1 container entry
        dataAddr: entry.data_addr,
        // Hex location fields
        itemAddr: entry.item_addr,
        compressedSize: entry.compressed_size,
        dataEndAddr: entry.data_end_addr,
        metadataAddr: entry.metadata_addr,
        firstChildAddr: entry.first_child_addr,
      });
    } else {
      props.onSelectEntry({
        containerPath,
        entryPath: entry.path,
        name: entry.name,
        size: entry.size,
        isDir: false,
        isVfsEntry: false, // AD1 container entry
        dataAddr: entry.data_addr,
        // Hex location fields
        itemAddr: entry.item_addr,
        compressedSize: entry.compressed_size,
        dataEndAddr: entry.data_end_addr,
        metadataAddr: entry.metadata_addr,
        firstChildAddr: entry.first_child_addr,
      });
    }
  };

  // Render a single AD1 entry row using standardized TreeRow component
  const EntryRow = (rowProps: {
    entry: TreeEntry;
    containerPath: string;
    depth: number;
    isExpanded: boolean;
    isLoading: boolean;
  }) => {
    const hasChildren = () => rowProps.entry.is_dir && (rowProps.entry.first_child_addr ?? 0) > 0;
    const entryKey = () => `${rowProps.containerPath}::${rowProps.entry.item_addr ?? rowProps.entry.path}`;
    const isSelected = () => selectedEntryKey() === entryKey();
    
    return (
      <TreeRow
        name={rowProps.entry.name}
        path={rowProps.entry.path}
        isDir={rowProps.entry.is_dir}
        size={rowProps.entry.size}
        depth={rowProps.depth}
        isSelected={isSelected()}
        isExpanded={rowProps.isExpanded}
        isLoading={rowProps.isLoading}
        hasChildren={hasChildren()}
        onClick={() => handleEntryClick(rowProps.containerPath, rowProps.entry)}
        onToggle={() => toggleDirByAddr(rowProps.containerPath, rowProps.entry)}
        hash={rowProps.entry.md5_hash || rowProps.entry.sha1_hash}
        data-entry-path={rowProps.entry.path}
        data-entry-addr={rowProps.entry.item_addr || undefined}
        badge={
          <Show when={isSelected() && rowProps.entry.item_addr != null}>
            <span class="text-xs" title={`Item at 0x${rowProps.entry.item_addr!.toString(16).toUpperCase()}`}>
              <HiOutlineMapPin class={`w-2 h-2 text-accent`} />
            </span>
          </Show>
        }
      />
    );
  };

  // VFS Entry Row - displays a VFS filesystem entry using standardized TreeRow
  const VfsEntryRow = (rowProps: {
    entry: VfsEntry;
    containerPath: string;
    depth: number;
    isExpanded: boolean;
    isLoading: boolean;
    partitionIndex: number;
  }) => {
    const entryKey = () => `${rowProps.containerPath}::vfs::${rowProps.entry.path}`;
    const isSelected = () => selectedEntryKey() === entryKey();
    
    const handleClick = () => {
      setSelectedEntryKey(entryKey());
      
      if (rowProps.entry.is_dir) {
        toggleVfsDir(rowProps.containerPath, rowProps.entry.path);
      }
      
      // Notify parent of selection
      props.onSelectEntry({
        containerPath: rowProps.containerPath,
        entryPath: rowProps.entry.path,
        name: rowProps.entry.name,
        size: rowProps.entry.size,
        isDir: rowProps.entry.is_dir,
        isVfsEntry: true, // VFS container entry (E01, Raw, etc.)
        // VFS entries don't have AD1-style addresses
        dataAddr: null,
        itemAddr: null,
        compressedSize: null,
        dataEndAddr: null,
        metadataAddr: null,
        firstChildAddr: null,
      });
    };
    
    return (
      <TreeRow
        name={rowProps.entry.name}
        path={rowProps.entry.path}
        isDir={rowProps.entry.is_dir}
        size={rowProps.entry.size}
        depth={rowProps.depth}
        isSelected={isSelected()}
        isExpanded={rowProps.isExpanded}
        isLoading={rowProps.isLoading}
        hasChildren={rowProps.entry.is_dir}
        onClick={handleClick}
        onToggle={() => toggleVfsDir(rowProps.containerPath, rowProps.entry.path)}
        data-entry-path={rowProps.entry.path}
      />
    );
  };

  // Recursive VFS tree node component
  const VfsTreeNode = (nodeProps: {
    entry: VfsEntry;
    containerPath: string;
    depth: number;
    partitionIndex: number;
  }) => {
    const nodeKey = `${nodeProps.containerPath}::vfs::${nodeProps.entry.path}`;
    const isExpanded = () => expandedVfsPaths().has(nodeKey);
    const isLoading = () => loading().has(nodeKey);
    const children = () => getVfsChildren(nodeProps.containerPath, nodeProps.entry.path);

    return (
      <>
        <VfsEntryRow
          entry={nodeProps.entry}
          containerPath={nodeProps.containerPath}
          depth={nodeProps.depth}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          partitionIndex={nodeProps.partitionIndex}
        />
        <Show when={isExpanded() && nodeProps.entry.is_dir}>
          <For each={children()}>
            {(child) => (
              <VfsTreeNode
                entry={child}
                containerPath={nodeProps.containerPath}
                depth={nodeProps.depth + 1}
                partitionIndex={nodeProps.partitionIndex}
              />
            )}
          </For>
        </Show>
      </>
    );
  };

  // Partition node - shows a partition with its filesystem tree
  const PartitionNode = (partitionProps: {
    partition: VfsPartitionInfo;
    containerPath: string;
    index: number;
  }) => {
    const { partition, containerPath, index } = partitionProps;
    // Use the actual mountName from the partition (e.g., "Partition1_NTFS") with fallback
    const mountName = partition.mountName ?? `Partition${partition.number ?? index + 1}`;
    const partitionRootPath = `/${mountName}`;
    const nodeKey = `${containerPath}::vfs::${partitionRootPath}`;
    const isExpanded = () => expandedVfsPaths().has(nodeKey);
    const isLoading = () => loading().has(nodeKey);
    const children = () => getVfsChildren(containerPath, partitionRootPath);

    const togglePartition = async () => {
      await toggleVfsDir(containerPath, partitionRootPath);
    };

    // Get filesystem icon
    const fsIcon = () => {
      const fs = (partition.fsType ?? '').toLowerCase();
      if (fs.includes("ntfs")) return <HiOutlineServerStack class={`w-3.5 h-3.5 text-blue-400`} />;
      if (fs.includes("fat")) return <HiOutlineRectangleStack class={`w-3.5 h-3.5 text-yellow-400`} />;
      if (fs.includes("ext")) return <HiOutlineCommandLine class={`w-3.5 h-3.5 text-orange-400`} />;
      if (fs.includes("hfs") || fs.includes("apfs")) return <HiOutlineCircleStack class={`w-3.5 h-3.5 text-txt-tertiary`} />;
      return <HiOutlineCircleStack class="w-3.5 h-3.5" />;
    };

    return (
      <div class="mb-1">
        <div 
          class={`${TREE_ROW_BASE_CLASSES} ${TREE_ROW_NORMAL_CLASSES}`}
          onClick={togglePartition}
          style={{ "padding-left": getTreeIndent(0) }}
          role="treeitem"
          aria-expanded={isExpanded()}
          tabIndex={0}
          data-tree-item
        >
          <span class="w-4 text-xs text-txt-muted flex items-center justify-center" aria-hidden="true">
            <ExpandIcon isLoading={isLoading()} isExpanded={isExpanded()} />
          </span>
          <span class="text-base" aria-hidden="true">{fsIcon()}</span>
          <span class="text-sm text-txt-tertiary">{mountName}</span>
          <span class="text-xs text-txt-muted">
            {partition.fsType ?? 'Unknown'} • {formatBytes(partition.size ?? 0)}
          </span>
        </div>
        <Show when={isExpanded()}>
          <div>
            <For each={children()}>
              {(entry) => (
                <VfsTreeNode
                  entry={entry}
                  containerPath={containerPath}
                  depth={1}
                  partitionIndex={index}
                />
              )}
            </For>
            <Show when={children().length === 0 && !isLoading()}>
              <TreeEmptyState message="Empty filesystem" depth={1} />
            </Show>
          </div>
        </Show>
      </div>
    );
  };

  // Archive entry node component using standardized TreeRow
  const ArchiveEntryNode = (nodeProps: {
    entry: ArchiveTreeEntry;
    containerPath: string;
    depth: number;
    allEntries: ArchiveTreeEntry[];
  }) => {
    const entryKey = `${nodeProps.containerPath}::archive::${nodeProps.entry.path}`;
    const isSelected = () => selectedEntryKey() === entryKey;
    const isExpanded = () => isArchiveDirExpanded(nodeProps.containerPath, nodeProps.entry.path);
    const isLoading = () => loading().has(`${nodeProps.containerPath}::nested::${nodeProps.entry.path}`);
    
    // Check if this entry is itself a container that can be opened
    const isNestedContainer = () => !nodeProps.entry.is_dir && isContainerFile(nodeProps.entry.name || nodeProps.entry.path);
    
    // Get children for this directory
    const children = createMemo(() => {
      if (!nodeProps.entry.is_dir) return [];
      return sortArchiveEntries(getArchiveChildren(nodeProps.allEntries, nodeProps.entry.path));
    });
    
    const handleClick = () => {
      if (nodeProps.entry.is_dir) {
        toggleArchiveDir(nodeProps.containerPath, nodeProps.entry.path);
      }
      setSelectedEntryKey(entryKey);
      // Notify parent about selection
      props.onSelectEntry({
        containerPath: nodeProps.containerPath,
        entryPath: nodeProps.entry.path,
        name: nodeProps.entry.name || nodeProps.entry.path.split('/').pop() || nodeProps.entry.path,
        size: nodeProps.entry.size,
        isDir: nodeProps.entry.is_dir,
        isVfsEntry: false, // Archive entry
      });
    };

    // Handle double-click to open nested containers
    const handleDoubleClick = () => {
      if (isNestedContainer() && props.onOpenNestedContainer) {
        const entryName = nodeProps.entry.name || nodeProps.entry.path.split('/').pop() || 'nested';
        openNestedContainer(nodeProps.containerPath, nodeProps.entry.path, entryName);
      }
    };

    // Extract just the filename from the path
    const fileName = () => {
      if (nodeProps.entry.name) return nodeProps.entry.name;
      const parts = nodeProps.entry.path.split('/').filter(p => p);
      return parts[parts.length - 1] || nodeProps.entry.path;
    };
    
    return (
      <>
        <TreeRow
          name={fileName()}
          path={nodeProps.entry.path}
          isDir={nodeProps.entry.is_dir}
          size={nodeProps.entry.size}
          depth={nodeProps.depth}
          isSelected={isSelected()}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          hasChildren={nodeProps.entry.is_dir && children().length > 0}
          onClick={handleClick}
          onDblClick={isNestedContainer() ? handleDoubleClick : undefined}
          entryType={isNestedContainer() ? "container" : undefined}
          data-entry-path={nodeProps.entry.path}
        />
        <Show when={isExpanded() && nodeProps.entry.is_dir}>
          <For each={children()}>
            {(child) => (
              <ArchiveEntryNode
                entry={child}
                containerPath={nodeProps.containerPath}
                depth={nodeProps.depth + 1}
                allEntries={nodeProps.allEntries}
              />
            )}
          </For>
        </Show>
      </>
    );
  };

  // Lazy-loaded entry node component (used for UFED, large archives, etc.)
  // Simplified to match AD1/E01 TreeNode pattern for consistent, fast UI
  const LazyEntryNode = (nodeProps: {
    entry: LazyTreeEntry;
    containerPath: string;
    depth: number;
  }) => {
    // Pre-compute entry key once (not reactive)
    const entryKey = `${nodeProps.containerPath}::lazy::${nodeProps.entry.path}`;
    const cacheKey = `${nodeProps.containerPath}::lazy::${nodeProps.entry.path}`;
    
    // Use memos for efficient reactive updates (like TreeNode does)
    const isSelected = () => selectedEntryKey() === entryKey;
    const isExpanded = () => expandedLazyPaths().has(cacheKey);
    const isLoading = () => loading().has(cacheKey);
    
    // Get children from cache with memo (avoid sorting on every render)
    const children = createMemo(() => {
      const cached = lazyChildrenCache().get(cacheKey) || [];
      // Only sort if needed (most backends return sorted)
      if (cached.length <= 1) return cached;
      return [...cached].sort((a, b) => {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
    });
    
    // Pagination state (direct lookups)
    const hasMore = () => lazyHasMore().get(cacheKey) || false;
    const totalCount = () => lazyTotalCounts().get(cacheKey) || 0;
    const isLoadingMore = () => loading().has(`${cacheKey}::more`);
    
    // Click handler - select and toggle in one action
    const handleClick = () => {
      setSelectedEntryKey(entryKey);
      
      // Notify parent about selection immediately
      props.onSelectEntry({
        containerPath: nodeProps.containerPath,
        entryPath: nodeProps.entry.path,
        name: nodeProps.entry.name,
        size: nodeProps.entry.size || 0,
        isDir: nodeProps.entry.is_dir,
        isVfsEntry: false,
      });
    };
    
    // Toggle expansion separately from click (like AD1 tree)
    const handleToggle = () => {
      if (nodeProps.entry.is_dir) {
        toggleLazyDir(nodeProps.containerPath, nodeProps.entry.path);
      }
    };
    
    const handleLoadMore = (e: MouseEvent) => {
      e.stopPropagation();
      loadMoreLazyEntries(nodeProps.containerPath, nodeProps.entry.path);
    };
    
    const hasChildren = () => nodeProps.entry.is_dir && (nodeProps.entry.child_count ?? 0) > 0;
    
    return (
      <>
        <TreeRow
          name={nodeProps.entry.name}
          path={nodeProps.entry.path}
          isDir={nodeProps.entry.is_dir}
          size={nodeProps.entry.size || 0}
          depth={nodeProps.depth}
          isSelected={isSelected()}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          hasChildren={hasChildren()}
          onClick={handleClick}
          onToggle={handleToggle}
          entryType={nodeProps.entry.entry_type}
          hash={nodeProps.entry.hash}
          data-entry-path={nodeProps.entry.path}
        />
        {/* Expanded children - same pattern as TreeNode */}
        <Show when={isExpanded() && nodeProps.entry.is_dir}>
          <For each={children()}>
            {(child) => (
              <LazyEntryNode
                entry={child}
                containerPath={nodeProps.containerPath}
                depth={nodeProps.depth + 1}
              />
            )}
          </For>
          {/* Load more button - only show if pagination needed */}
          <Show when={hasMore()}>
            <div 
              class={`${TREE_ROW_BASE_CLASSES} ${TREE_ROW_NORMAL_CLASSES} cursor-pointer hover:bg-bg-hover/30`}
              style={{ "padding-left": getTreeIndent(nodeProps.depth + 1) }}
              onClick={handleLoadMore}
            >
              <Show 
                when={!isLoadingMore()} 
                fallback={
                  <span class="flex items-center gap-2 text-xs text-txt-secondary">
                    <HiOutlineArrowPath class={`w-3 h-3 animate-spin`} />
                    Loading...
                  </span>
                }
              >
                <span class="flex items-center gap-2 text-xs text-accent hover:text-accent-hover">
                  <HiOutlineChevronDown class="w-3 h-3" />
                  Load more ({children().length.toLocaleString()} of {totalCount().toLocaleString()})
                </span>
              </Show>
            </div>
          </Show>
        </Show>
      </>
    );
  };

  // UFED entry node component using standardized TreeRow
  const UfedEntryNode = (nodeProps: {
    entry: UfedTreeEntry;
    containerPath: string;
    depth: number;
  }) => {
    const entryKey = `${nodeProps.containerPath}::ufed::${nodeProps.entry.path}`;
    const isSelected = () => selectedEntryKey() === entryKey;
    
    const handleClick = () => {
      setSelectedEntryKey(entryKey);
      // Notify parent about selection
      props.onSelectEntry({
        containerPath: nodeProps.containerPath,
        entryPath: nodeProps.entry.path,
        name: nodeProps.entry.name,
        size: nodeProps.entry.size || 0,
        isDir: nodeProps.entry.is_dir,
        isVfsEntry: false, // UFED entry
      });
    };
    
    return (
      <TreeRow
        name={nodeProps.entry.name}
        path={nodeProps.entry.path}
        isDir={nodeProps.entry.is_dir}
        size={nodeProps.entry.size || 0}
        depth={nodeProps.depth}
        isSelected={isSelected()}
        isExpanded={false}
        isLoading={false}
        hasChildren={false}
        onClick={handleClick}
        entryType={nodeProps.entry.entry_type}
        hash={nodeProps.entry.hash}
        data-entry-path={nodeProps.entry.path}
      />
    );
  };

  // Recursive tree node component
  const TreeNode = (nodeProps: {
    entry: TreeEntry;
    containerPath: string;
    depth: number;
  }) => {
    // Use the helper functions that support both addr and path-based loading
    const isExpanded = () => isDirExpanded(nodeProps.containerPath, nodeProps.entry);
    const nodeKey = () => {
      const addr = nodeProps.entry.first_child_addr;
      return addr 
        ? `${nodeProps.containerPath}::addr:${addr}` 
        : `${nodeProps.containerPath}::path:${nodeProps.entry.path}`;
    };
    const isLoading = () => loading().has(nodeKey());
    const children = () => getChildrenForEntry(nodeProps.containerPath, nodeProps.entry);

    return (
      <>
        <EntryRow
          entry={nodeProps.entry}
          containerPath={nodeProps.containerPath}
          depth={nodeProps.depth}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
        />
        <Show when={isExpanded() && nodeProps.entry.is_dir}>
          <For each={children()}>
            {(child) => (
              <TreeNode
                entry={child}
                containerPath={nodeProps.containerPath}
                depth={nodeProps.depth + 1}
              />
            )}
          </For>
        </Show>
      </>
    );
  };

  // Render container with its tree
  const ContainerNode = (containerProps: { file: DiscoveredFile }) => {
    const file = containerProps.file;
    const isExpanded = () => expandedContainers().has(file.path);
    const isLoading = () => loading().has(file.path);
    const containerType = file.container_type.toLowerCase();
    const isVfs = isVfsContainer(containerType);
    const isL01 = isL01Container(containerType);
    const isArchive = isArchiveContainer(containerType);
    const isUfed = isUfedContainer(containerType);
    const isAd1 = isAd1Container(containerType);
    const isMemory = isMemoryDump(containerType) || isMemoryDumpFile(file.path);
    const mountInfo = () => vfsMountCache().get(file.path);
    
    // Explicitly reactive AD1 root children - read cache in a memo
    const rootChildren = createMemo(() => {
      const cache = childrenCache();
      const cacheKey = `${file.path}::root`;
      const entries = cache.get(cacheKey) || [];
      return sortEntries(entries);
    });
    
    // Explicitly reactive archive entries - read cache in a memo
    const allArchiveEntries = createMemo(() => {
      const cache = archiveTreeCache();
      return cache.get(file.path) || [];
    });
    
    // Root-level archive entries (sorted) - only entries without parent
    const archiveRootEntries = createMemo(() => {
      return sortArchiveEntries(getArchiveRootEntries(allArchiveEntries()));
    });
    
    // For info bar - count from all entries
    const archiveEntries = allArchiveEntries;
    
    // Explicitly reactive UFED entries (OLD - used as fallback)
    const ufedEntries = createMemo(() => {
      const cache = ufedTreeCache();
      const entries = cache.get(file.path) || [];
      return sortUfedEntries(entries);
    });
    
    // Lazy-loaded UFED root entries (NEW - used for large containers)
    const lazyRootEntries = createMemo(() => {
      const cache = lazyChildrenCache();
      const entries = cache.get(`${file.path}::lazy::root`) || [];
      return sortLazyEntries(entries);
    });
    
    // Lazy loading summary
    const lazySummary = createMemo(() => {
      return lazySummaryCache().get(file.path) || null;
    });
    
    // Check if we have lazy-loaded data for this UFED container
    const hasLazyData = createMemo(() => {
      return lazyRootEntries().length > 0 || lazySummary() !== null;
    });
    
    // Root-level "Load more" button state for UFED
    const hasMoreRootEntries = () => hasMoreLazyEntries(file.path, "root");
    const rootTotalCount = () => getLazyTotalCount(file.path, "root");
    const isLoadingMoreRoot = () => isLoadingMoreLazy(file.path, "root");
    
    const handleLoadMoreRoot = (e: MouseEvent) => {
      e.stopPropagation();
      loadMoreLazyEntries(file.path, "root");
    };
    
    // Explicitly reactive AD1 info
    const ad1Info = createMemo(() => {
      const cache = ad1InfoCache();
      return cache.get(file.path) || null;
    });
    
    return (
      <div class="border-b border-border/50">
        {/* Container header - standardized component */}
        <ContainerHeader
          name={file.filename || file.path.split('/').pop() || file.path}
          path={file.path}
          containerType={file.container_type}
          size={file.size}
          isActive={props.activeFile?.path === file.path}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          segmentCount={file.segment_count}
          onClick={() => {
            props.onSelectContainer(file);
            toggleContainer(file);
          }}
          statusIcon={isVfs && mountInfo() ? (
            <span title="Mounted disk image">
              <HiOutlineCircleStack class={`w-3.5 h-3.5 text-accent`} />
            </span>
          ) : undefined}
        />
        
        {/* Container contents */}
        <Show when={isExpanded()}>
          <div class="pb-1">
            {/* VFS container - show partitions (E01, Raw) */}
            <Show when={isVfs && mountInfo()}>
              {/* Info bar - consistent with other containers */}
              <div class={TREE_INFO_BAR_CLASSES} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                <HiOutlineCircleStack class={`w-3 h-3 text-txt-secondary`} />
                <span class="text-txt-secondary">{formatBytes(mountInfo()!.diskSize)}</span>
                <span>•</span>
                <span class="text-txt-secondary">{mountInfo()!.partitions.length} partition(s)</span>
              </div>
              <For each={mountInfo()!.partitions}>
                {(partition, index) => (
                  <PartitionNode
                    partition={partition}
                    containerPath={file.path}
                    index={index()}
                  />
                )}
              </For>
              <Show when={mountInfo()!.partitions.length === 0}>
                <div class="py-2 text-xs text-txt-muted italic" style={{ "padding-left": "32px" }}>
                  (no recognized partitions - may be raw filesystem or unpartitioned)
                </div>
              </Show>
            </Show>
            
            {/* VFS container that failed to mount (DMG, ISO, L01, etc.) */}
            <Show when={isVfs && !mountInfo() && !isLoading()}>
              <Show when={isL01}>
                <TreeEmptyState 
                  message="L01 logical evidence" 
                  hint="File tree browsing not yet implemented"
                />
              </Show>
              <Show when={!isL01}>
                <TreeEmptyState 
                  message={`Format "${file.container_type}" not supported`}
                  hint="VFS mounting failed or not supported"
                />
              </Show>
            </Show>
            
            {/* Archive container - show archive entries (ZIP, 7z, RAR, TAR) */}
            <Show when={isArchive}>
              {/* Info bar */}
              <div class={TREE_INFO_BAR_CLASSES} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                <HiOutlineDocument class={`w-3 h-3 text-txt-secondary`} />
                <span class="text-txt-secondary">
                  {archiveEntries().filter(e => !e.is_dir).length.toLocaleString()} files
                </span>
                <span>•</span>
                <HiOutlineFolder class={`w-3 h-3 text-txt-secondary`} />
                <span class="text-txt-secondary">
                  {archiveEntries().filter(e => e.is_dir).length.toLocaleString()} folders
                </span>
                <span>•</span>
                <span class="text-txt-secondary">
                  {formatBytes(archiveEntries().reduce((sum, e) => sum + e.size, 0))}
                </span>
              </div>
              <For each={archiveRootEntries()}>
                {(entry) => (
                  <ArchiveEntryNode
                    entry={entry}
                    containerPath={file.path}
                    depth={0}
                    allEntries={allArchiveEntries()}
                  />
                )}
              </For>
              <Show when={archiveRootEntries().length === 0 && !isLoading()}>
                <TreeEmptyState message="Empty archive" />
              </Show>
            </Show>

            {/* UFED container - show UFED entries with LAZY LOADING */}
            <Show when={isUfed}>
              {/* Info bar - shows lazy summary if available */}
              <div class={TREE_INFO_BAR_CLASSES} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                <Show 
                  when={lazySummary()}
                  fallback={
                    <>
                      <HiOutlineDocument class={`w-3 h-3 text-txt-secondary`} />
                      <span class="text-txt-secondary">
                        {ufedEntries().filter(e => !e.is_dir).length.toLocaleString()} files
                      </span>
                      <span>•</span>
                      <HiOutlineFolder class={`w-3 h-3 text-txt-secondary`} />
                      <span class="text-txt-secondary">
                        {ufedEntries().filter(e => e.is_dir).length.toLocaleString()} folders
                      </span>
                    </>
                  }
                >
                  {/* Lazy loading summary */}
                  <HiOutlineServerStack class={`w-3 h-3 text-accent`} />
                  <span class="text-accent">
                    {lazySummary()!.entry_count.toLocaleString()} total entries
                  </span>
                  <Show when={lazySummary()!.lazy_loading_recommended}>
                    <span>•</span>
                    <span class="text-amber-400 text-xs">lazy loading</span>
                  </Show>
                </Show>
                <span>•</span>
                <span class="text-txt-secondary">
                  {hasLazyData() 
                    ? formatBytes(lazyRootEntries().reduce((sum, e) => sum + (e.size || 0), 0))
                    : formatBytes(ufedEntries().reduce((sum, e) => sum + e.size, 0))
                  }
                </span>
              </div>
              
              {/* Use lazy-loaded entries when available */}
              <Show 
                when={hasLazyData()}
                fallback={
                  // Fallback to old full-tree loading
                  <For each={ufedEntries()}>
                    {(entry) => (
                      <UfedEntryNode
                        entry={entry}
                        containerPath={file.path}
                        depth={0}
                      />
                    )}
                  </For>
                }
              >
                {/* Lazy-loaded entries with pagination */}
                <For each={lazyRootEntries()}>
                  {(entry) => (
                    <LazyEntryNode
                      entry={entry}
                      containerPath={file.path}
                      depth={0}
                    />
                  )}
                </For>
                
                {/* Load more button for root level */}
                <Show when={hasMoreRootEntries()}>
                  <div 
                    class={`${TREE_ROW_BASE_CLASSES} ${TREE_ROW_NORMAL_CLASSES} cursor-pointer hover:bg-bg-hover/30`}
                    style={{ "padding-left": getTreeIndent(0) }}
                    onClick={handleLoadMoreRoot}
                  >
                    <Show 
                      when={!isLoadingMoreRoot()} 
                      fallback={
                        <span class="flex items-center gap-2 text-xs text-txt-secondary">
                          <HiOutlineArrowPath class={`w-3 h-3 animate-spin`} />
                          Loading more entries...
                        </span>
                      }
                    >
                      <span class="flex items-center gap-2 text-xs text-accent hover:text-accent-hover">
                        <HiOutlineChevronDown class="w-3 h-3" />
                        Load more ({lazyRootEntries().length.toLocaleString()} of {rootTotalCount().toLocaleString()})
                      </span>
                    </Show>
                  </div>
                </Show>
              </Show>
              
              <Show when={lazyRootEntries().length === 0 && ufedEntries().length === 0 && !isLoading()}>
                <TreeEmptyState message="Empty UFED extraction" />
              </Show>
            </Show>
            
            {/* AD1 container - show file tree with info bar like EWF */}
            <Show when={isAd1}>
              {/* Info bar - similar to EWF disk info */}
              <Show when={ad1Info()}>
                <div class={TREE_INFO_BAR_CLASSES} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                  <HiOutlineDocument class={`w-3 h-3 text-txt-secondary`} />
                  <span class="text-txt-secondary">{ad1Info()!.file_count.toLocaleString()} files</span>
                  <span>•</span>
                  <HiOutlineFolder class={`w-3 h-3 text-txt-secondary`} />
                  <span class="text-txt-secondary">{ad1Info()!.dir_count.toLocaleString()} folders</span>
                  <span>•</span>
                  <span class="text-txt-secondary">{formatBytes(ad1Info()!.total_size)}</span>
                  <Show when={ad1Info()!.source_name}>
                    <span>•</span>
                    <span class="text-accent/80 truncate max-w-[200px]" title={ad1Info()!.source_name!}>
                      {ad1Info()!.source_name}
                    </span>
                  </Show>
                </div>
              </Show>
              
              {/* Tree entries */}
              <For each={rootChildren()}>
                {(entry) => (
                  <TreeNode
                    entry={entry}
                    containerPath={file.path}
                    depth={0}
                  />
                )}
              </For>
              <Show when={rootChildren().length === 0 && !isLoading()}>
                <Show when={containerErrors().has(file.path)}>
                  <TreeErrorState 
                    message={containerErrors().get(file.path)!}
                    onRetry={() => toggleContainer(file)}
                  />
                </Show>
                <Show when={!containerErrors().has(file.path)}>
                  <TreeEmptyState message="Empty container" />
                </Show>
              </Show>
            </Show>
            
            {/* Memory dump container - show as single raw file */}
            <Show when={isMemory}>
              {/* Info bar */}
              <div class={TREE_INFO_BAR_CLASSES} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                <HiOutlineCpuChip class={`w-3 h-3 text-purple-400`} />
                <span class="text-purple-400">Memory Dump</span>
                <span>•</span>
                <span class="text-txt-secondary">{formatBytes(file.size)}</span>
                <Show when={(file.segment_count || 1) > 1}>
                  <span>•</span>
                  <span class="text-txt-secondary">{file.segment_count} segments</span>
                </Show>
              </div>
              {/* Single raw file entry */}
              <div 
                class={`${TREE_ROW_BASE_CLASSES} ${TREE_ROW_NORMAL_CLASSES} cursor-pointer hover:bg-bg-hover/30`}
                style={{ "padding-left": getTreeIndent(0) }}
                onClick={() => {
                  const filename = file.filename || file.path.split('/').pop() || 'memory.raw';
                  props.onSelectEntry?.({
                    containerPath: file.path,
                    entryPath: file.path,
                    name: filename,
                    size: file.size,
                    isDir: false,
                    containerType: "memory",
                    metadata: {
                      type: "memory_dump",
                      segments: file.segment_count || 1,
                    },
                  });
                }}
              >
                <HiOutlineRectangleStack class={`w-3.5 h-3.5 text-purple-400 mr-2 flex-shrink-0`} />
                <span class="truncate">{file.filename || file.path.split('/').pop() || 'memory.raw'}</span>
                <span class="ml-auto text-xs text-txt-muted">{formatBytes(file.size)}</span>
              </div>
              <div class="py-1.5 text-xs text-txt-muted italic" style={{ "padding-left": getTreeIndent(0) }}>
                Raw memory capture - use Hex Viewer to analyze
              </div>
            </Show>
            
            {/* Unknown container type - display message */}
            <Show when={!isVfs && !isArchive && !isUfed && !isAd1 && !isMemory}>
              <TreeEmptyState 
                message={`Format "${file.container_type}" not supported`}
                hint="Tree browsing unavailable"
              />
            </Show>
          </div>
        </Show>
      </div>
    );
  };

  return (
    <div 
      class="flex flex-col h-full bg-bg text-sm"
      tabIndex={0}
      role="tree"
      aria-label="Evidence file tree"
      onKeyDown={(e) => {
        // Get all visible tree items
        const treeItems = e.currentTarget.querySelectorAll<HTMLElement>('[role="treeitem"], [data-tree-item]');
        const items = Array.from(treeItems);
        if (items.length === 0) return;
        
        // Find currently focused item
        const focusedItem = e.currentTarget.querySelector<HTMLElement>(':focus, [data-focused="true"]');
        let currentIndex = focusedItem ? items.indexOf(focusedItem) : -1;
        
        switch (e.key) {
          case "ArrowDown":
            e.preventDefault();
            currentIndex = Math.min(currentIndex + 1, items.length - 1);
            items[currentIndex]?.focus();
            items[currentIndex]?.click();
            break;
          case "ArrowUp":
            e.preventDefault();
            currentIndex = Math.max(currentIndex - 1, 0);
            items[currentIndex]?.focus();
            items[currentIndex]?.click();
            break;
          case "ArrowRight":
            e.preventDefault();
            // Expand current node
            if (focusedItem?.getAttribute("aria-expanded") === "false") {
              focusedItem.click();
            }
            break;
          case "ArrowLeft":
            e.preventDefault();
            // Collapse current node
            if (focusedItem?.getAttribute("aria-expanded") === "true") {
              focusedItem.click();
            }
            break;
          case "Enter":
          case " ":
            e.preventDefault();
            focusedItem?.click();
            break;
          case "Home":
            e.preventDefault();
            items[0]?.focus();
            items[0]?.click();
            break;
          case "End":
            e.preventDefault();
            items[items.length - 1]?.focus();
            items[items.length - 1]?.click();
            break;
        }
      }}
    >
      {/* Type filter bar */}
      <Show when={Object.keys(props.containerStats).length > 0}>
        <div class="flex flex-wrap items-center gap-0.5 px-1.5 py-0.5 border-b border-border bg-bg-panel/50">
          {/* All button */}
          <button
            class={`flex items-center gap-0.5 px-1 py-px text-[10px] leading-tight rounded transition-colors ${!props.typeFilter ? 'bg-accent text-white' : 'text-txt-secondary hover:bg-bg-hover hover:text-txt'}`}
            onClick={props.onClearTypeFilter}
            title="Show all containers"
          >
            <span>All:</span>
            <span>{props.discoveredFiles.length}</span>
          </button>
          {/* Type filters */}
          <For each={Object.entries(props.containerStats)}>
            {([type, count]) => {
              const IconComponent = getContainerTypeIcon(type);
              const isActive = props.typeFilter === type;
              return (
                <button
                  class={`flex items-center gap-0.5 px-1 py-0.5 text-[10px] leading-tight rounded transition-colors ${isActive ? 'bg-accent text-white' : 'text-txt-secondary hover:bg-bg-hover hover:text-txt'}`}
                  onClick={() => props.onToggleTypeFilter(type)}
                  title={`Filter by ${type} (${count} files)`}
                >
                  <IconComponent class="w-[10px] h-[10px]" />
                  <span>{type}:</span>
                  <span>{count}</span>
                  <Show when={isActive}>
                    <HiOutlineXMark class="w-[10px] h-[10px] ml-0.5 opacity-70 hover:opacity-100" />
                  </Show>
                </button>
              );
            }}
          </For>
        </div>
      </Show>
      
      {/* Tree content */}
      <div class="flex-1 overflow-auto">
        <Show when={props.busy}>
          <div class="flex items-center justify-center py-8 text-txt-muted">Loading containers...</div>
        </Show>
        
        <Show when={!props.busy && filteredFiles().length === 0}>
          <div class="flex items-center justify-center py-8 text-txt-muted text-center">
            No forensic containers found. Add evidence files to begin.
          </div>
        </Show>
        
        <For each={filteredFiles()}>
          {(file) => <ContainerNode file={file} />}
        </For>
      </div>
    </div>
  );
}
