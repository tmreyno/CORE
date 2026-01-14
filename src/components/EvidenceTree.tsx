// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceTree - Unified lazy-loading tree for forensic containers
 * Supports: AD1, E01/Raw (VFS), Archives (ZIP/7z/RAR/TAR), UFED
 */

import { For, Show, createSignal, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineCircleStack,
  HiOutlineServerStack,
  HiOutlineRectangleStack,
  HiOutlineCommandLine,
  HiOutlineFolder,
  HiOutlineDocument,
} from "./icons";
import { 
  TreeEmptyState, 
  TreeErrorState, 
  ExpandIcon, 
  ContainerHeader,
  LoadMoreButton,
  TREE_ROW_BASE_CLASSES,
  TREE_ROW_NORMAL_CLASSES,
  TREE_INFO_BAR_CLASSES,
  TREE_INFO_BAR_PADDING,
  getTreeIndent,
} from "./tree";
import type { DiscoveredFile, TreeEntry, VfsEntry, VfsPartitionInfo, ArchiveTreeEntry, UfedTreeEntry } from "../types";
import type { LazyTreeEntry } from "../types/lazy-loading";
import { formatBytes } from "../utils";
import {
  isVfsContainer,
  isL01Container,
  isAd1Container,
  isArchiveContainer,
  isUfedContainer,
  isContainerFile,
  getContainerType,
} from "./EvidenceTree/containerDetection";
import { TypeFilterBar } from "./TypeFilterBar";
import { 
  Ad1EntryRow,
  VfsEntryRow as VfsEntryRowComponent,
  ArchiveEntryRow,
  LazyEntryRow,
  UfedEntryRow,
} from "./EvidenceTree/nodes";
import {
  sortTreeEntries,
  sortArchiveEntries,
  sortLazyEntries,
  sortUfedEntries,
} from "./EvidenceTree/utils";
import { useTreeLoading, useTreeSelection, useTreeExpansion, useTreeCache, useVfsTree, useArchiveTree, useLazyTree, useAd1Tree } from "./EvidenceTree/hooks";

/** Props for selecting an entry to view - includes hex location info */
export interface SelectedEntry {
  containerPath: string;
  entryPath: string;
  name: string;
  size: number;
  isDir: boolean;
  /** Whether this entry is from a VFS container (E01/Raw) */
  isVfsEntry?: boolean;
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

interface EvidenceTreeProps {
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

export function EvidenceTree(props: EvidenceTreeProps) {
  // === Core State (using extracted hooks) ===
  const treeLoading = useTreeLoading();
  const treeSelection = useTreeSelection();
  const containerExpansion = useTreeExpansion();  // For container-level expansion
  const dirExpansion = useTreeExpansion();        // For directory-level expansion
  const childrenCacheHook = useTreeCache<TreeEntry>();
  
  // Expose for backward compatibility with existing code
  const loading = treeLoading.loadingKeys;
  const startLoading = treeLoading.startLoading;
  const stopLoading = treeLoading.stopLoading;
  const selectedEntryKey = treeSelection.selectedKey;
  const setSelectedEntryKey = treeSelection.select;
  const expandedContainers = containerExpansion.expandedKeys;
  const expandedDirs = dirExpansion.expandedKeys;
  const childrenCache = childrenCacheHook.cache;

  // Legacy setters for complex operations (will phase out)
  const setExpandedContainers = (value: Set<string> | ((prev: Set<string>) => Set<string>)) => {
    if (typeof value === 'function') {
      const newSet = value(expandedContainers());
      // Sync with hook
      containerExpansion.collapseAll();
      [...newSet].forEach(k => containerExpansion.expand(k));
    } else {
      containerExpansion.collapseAll();
      [...value].forEach(k => containerExpansion.expand(k));
    }
  };
  const setExpandedDirs = (value: Set<string> | ((prev: Set<string>) => Set<string>)) => {
    if (typeof value === 'function') {
      const newSet = value(expandedDirs());
      dirExpansion.collapseAll();
      [...newSet].forEach(k => dirExpansion.expand(k));
    } else {
      dirExpansion.collapseAll();
      [...value].forEach(k => dirExpansion.expand(k));
    }
  };
  const setChildrenCache = (fn: (prev: Map<string, TreeEntry[]>) => Map<string, TreeEntry[]>) => {
    const newMap = fn(childrenCache());
    // Sync with hook - for new entries
    newMap.forEach((entries, key) => {
      if (!childrenCacheHook.has(key)) {
        childrenCacheHook.set(key, entries);
      }
    });
  };

  // === Container-specific hooks ===
  const vfsTree = useVfsTree();
  const archiveTree = useArchiveTree();
  const lazyTree = useLazyTree();
  const ad1Tree = useAd1Tree();
  
  // Expose state from hooks for component use
  const vfsMountCache = vfsTree.vfsMountCache;
  const expandedVfsPaths = vfsTree.expandedVfsPaths;
  const archiveTreeCache = archiveTree.archiveTreeCache;
  const lazySummaryCache = lazyTree.lazySummaryCache;
  const lazyChildrenCache = lazyTree.lazyChildrenCache;
  const lazyTotalCounts = lazyTree.lazyTotalCounts;
  const lazyHasMore = lazyTree.lazyHasMore;
  const expandedLazyPaths = lazyTree.expandedLazyPaths;
  const ad1InfoCache = ad1Tree.ad1InfoCache;
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const ufedTreeCache = () => new Map<string, UfedTreeEntry[]>(); // Legacy - not used

  /** Keyboard navigation handler for tree */
  const handleTreeKeyDown = (e: KeyboardEvent) => {
    const treeItems = (e.currentTarget as HTMLElement).querySelectorAll<HTMLElement>('[role="treeitem"], [data-tree-item]');
    const items = Array.from(treeItems);
    if (items.length === 0) return;
    
    const focusedItem = (e.currentTarget as HTMLElement).querySelector<HTMLElement>(':focus, [data-focused="true"]');
    let currentIndex = focusedItem ? items.indexOf(focusedItem) : -1;
    
    const navigate = (index: number) => { items[index]?.focus(); items[index]?.click(); };
    
    switch (e.key) {
      case "ArrowDown": e.preventDefault(); navigate(Math.min(currentIndex + 1, items.length - 1)); break;
      case "ArrowUp": e.preventDefault(); navigate(Math.max(currentIndex - 1, 0)); break;
      case "ArrowRight": e.preventDefault(); if (focusedItem?.getAttribute("aria-expanded") === "false") focusedItem.click(); break;
      case "ArrowLeft": e.preventDefault(); if (focusedItem?.getAttribute("aria-expanded") === "true") focusedItem.click(); break;
      case "Enter": case " ": e.preventDefault(); focusedItem?.click(); break;
      case "Home": e.preventDefault(); navigate(0); break;
      case "End": e.preventDefault(); navigate(items.length - 1); break;
    }
  };

  // === Cache Key Helper ===
  const lazyKey = (containerPath: string, path: string = "root") => `${containerPath}::lazy::${path}`;

  // Filtered container files
  const filteredFiles = createMemo(() => {
    const filter = props.typeFilter;
    return filter 
      ? props.discoveredFiles.filter(f => f.container_type === filter)
      : props.discoveredFiles;
  });

  // === AD1 Operations (delegated to hook) ===
  const loadAd1Info = ad1Tree.loadAd1Info;

  // === VFS Operations (delegated to hook) ===
  const mountVfsContainer = vfsTree.mountVfsContainer;

  // Toggle VFS directory expansion (delegated to hook)
  const toggleVfsDir = async (containerPath: string, vfsPath: string) => {
    await vfsTree.toggleVfsDir(containerPath, vfsPath, loading(), () => {
      // Loading state is managed by treeLoading hook
    });
  };

  const getVfsChildren = vfsTree.getVfsChildren;

  // === Archive Operations (delegated to hook) ===
  const loadArchiveMetadata = archiveTree.loadArchiveMetadata;
  const loadArchiveTree = archiveTree.loadArchiveTree;

  const getArchiveRootEntries = (entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] => 
    entries.filter(entry => !entry.path.replace(/\/$/, '').includes('/'));

  const getArchiveChildren = (entries: ArchiveTreeEntry[], parentPath: string): ArchiveTreeEntry[] => {
    const normalizedParent = parentPath.replace(/\/$/, '');
    return entries.filter(entry => {
      const entryPath = entry.path.replace(/\/$/, '');
      if (!entryPath.startsWith(normalizedParent + '/')) return false;
      const remaining = entryPath.substring(normalizedParent.length + 1);
      return !remaining.includes('/');
    });
  };

  const isArchiveDirExpanded = (containerPath: string, archivePath: string): boolean => 
    archiveTree.expandedArchivePaths().has(`${containerPath}::archive::${archivePath}`);

  const toggleArchiveDir = (containerPath: string, archivePath: string) => {
    archiveTree.toggleArchiveDir(containerPath, archivePath);
  };

  const openNestedContainer = async (containerPath: string, entryPath: string, entryName: string) => {
    if (!props.onOpenNestedContainer) return;
    
    const nodeKey = `${containerPath}::nested::${entryPath}`;
    startLoading(nodeKey);
    
    try {
      const tempPath = await invoke<string>("archive_extract_entry", {
        containerPath,
        entryPath,
      });
      const containerType = getContainerType(entryName);
      props.onOpenNestedContainer(tempPath, entryName, containerType, containerPath);
    } catch (err) {
      console.error("[openNestedContainer] Failed to extract:", err);
    } finally {
      stopLoading(nodeKey);
    }
  };

  // === Lazy Loading Operations (delegated to hook) ===
  const loadLazySummary = lazyTree.loadLazySummary;
  const loadLazyRootChildren = lazyTree.loadLazyRootChildren;
  
  /** Toggle lazy directory expansion (using hook with loading state) */
  const toggleLazyDir = async (containerPath: string, entryPath: string) => {
    await lazyTree.toggleLazyDir(containerPath, entryPath, loading(), () => {
      // Update loading state - hooks manage expansion internally
    });
  };
  
  /** Load more entries for a lazy path (pagination) */
  const loadMoreLazyEntries = async (containerPath: string, parentPath: string = "root") => {
    await lazyTree.loadMoreLazyEntries(containerPath, parentPath, loading(), () => {
      // Loading state managed by hook
    });
  };
  
  /** Check if currently loading more for a lazy path */
  const isLoadingMoreLazy = (containerPath: string, parentPath: string = "root"): boolean => 
    lazyTree.isLoadingMoreLazy(containerPath, parentPath, loading());
  
  /** Check if there are more entries to load */
  const hasMoreLazyEntries = (containerPath: string, parentPath: string = "root"): boolean => 
    lazyHasMore().get(lazyKey(containerPath, parentPath)) || false;
  
  const getLazyTotalCount = (containerPath: string, parentPath: string = "root"): number => 
    lazyTotalCounts().get(lazyKey(containerPath, parentPath)) || 0;

  const [containerErrors, setContainerErrors] = createSignal<Map<string, string>>(new Map());

  // === AD1 Tree Operations ===
  const loadRootChildren = async (containerPath: string, containerType: string): Promise<TreeEntry[]> => {
    const cacheKey = `${containerPath}::root`;
    const cached = childrenCache().get(cacheKey);
    if (cached) return cached;
    if (!isAd1Container(containerType)) return [];
    
    try {
      const children = await invoke<TreeEntry[]>("container_get_root_children_v2", { containerPath });
      setContainerErrors(prev => {
        const next = new Map(prev);
        next.delete(containerPath);
        return next;
      });
      setChildrenCache(prev => new Map(prev).set(cacheKey, children));
      return children;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error("Failed to load root children:", errorMsg);
      setContainerErrors(prev => new Map(prev).set(containerPath, errorMsg));
      return [];
    }
  };

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
      setChildrenCache(prev => new Map(prev).set(cacheKey, children));
      return children;
    } catch (err) {
      console.error("Failed to load children at addr:", err);
      return [];
    }
  };

  const loadChildrenByPath = async (containerPath: string, entryPath: string): Promise<TreeEntry[]> => {
    const cacheKey = `${containerPath}::path:${entryPath}`;
    const cached = childrenCache().get(cacheKey);
    if (cached) return cached;
    
    try {
      const children = await invoke<TreeEntry[]>("container_get_children", {
        containerPath,
        parentPath: entryPath,
      });
      
      setChildrenCache(prev => new Map(prev).set(cacheKey, children));
      
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
        if (!vfsMountCache().has(path)) {
          startLoading(path);
          await mountVfsContainer(path);
          stopLoading(path);
        }
      } else if (isArchiveContainer(file.container_type)) {
        startLoading(path);
        await loadArchiveMetadata(path);
        expanded.add(path);
        setExpandedContainers(new Set(expanded));
        if (!archiveTreeCache().has(path)) {
          await loadArchiveTree(path);
        }
        stopLoading(path);
        return;
      } else if (isUfedContainer(file.container_type)) {
        startLoading(path);
        await loadLazySummary(path);
        expanded.add(path);
        setExpandedContainers(new Set(expanded));
        if (!lazyChildrenCache().has(lazyKey(path))) {
          await loadLazyRootChildren(path, 0, 100);
        }
        stopLoading(path);
        return;
      } else if (isAd1Container(file.container_type)) {
        const cacheKey = `${path}::root`;
        expanded.add(path);
        setExpandedContainers(expanded);
        if (!childrenCache().has(cacheKey)) {
          startLoading(path);
          await Promise.all([
            loadRootChildren(path, file.container_type),
            loadAd1Info(path),
          ]);
          stopLoading(path);
        }
        return;
      }
      expanded.add(path);
      setExpandedContainers(expanded);
    }
  };

  const getEntryNodeKey = (containerPath: string, entry: TreeEntry): string => {
    const addr = entry.item_addr;
    return addr ? `${containerPath}::addr:${addr}` : `${containerPath}::path:${entry.path}`;
  };

  const toggleDirByAddr = async (containerPath: string, entry: TreeEntry) => {
    const nodeKey = getEntryNodeKey(containerPath, entry);
    const expanded = new Set(expandedDirs());
    
    if (expanded.has(nodeKey)) {
      expanded.delete(nodeKey);
      setExpandedDirs(expanded);
    } else {
      expanded.add(nodeKey);
      setExpandedDirs(expanded);
      
      if (!childrenCache().has(nodeKey)) {
        startLoading(nodeKey);
        if (entry.item_addr) {
          await loadChildrenByAddr(containerPath, entry.item_addr, entry.path);
        } else {
          await loadChildrenByPath(containerPath, entry.path);
        }
        stopLoading(nodeKey);
      }
    }
  };

  const getChildrenForEntry = (containerPath: string, entry: TreeEntry): TreeEntry[] => 
    sortTreeEntries(childrenCache().get(getEntryNodeKey(containerPath, entry)) || []);

  const isDirExpanded = (containerPath: string, entry: TreeEntry): boolean => 
    expandedDirs().has(getEntryNodeKey(containerPath, entry));

  const handleEntryClick = (containerPath: string, entry: TreeEntry) => {
    const entryKey = `${containerPath}::${entry.item_addr ?? entry.path}`;
    setSelectedEntryKey(entryKey);
    if (entry.is_dir) toggleDirByAddr(containerPath, entry);
    
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
  };

  // === Node Components ===
  const VfsTreeNode = (nodeProps: {
    entry: VfsEntry;
    containerPath: string;
    depth: number;
    partitionIndex: number;
  }) => {
    const entryKey = `${nodeProps.containerPath}::vfs::${nodeProps.entry.path}`;
    const isExpanded = () => expandedVfsPaths().has(entryKey);
    const isLoading = () => loading().has(entryKey);
    const isSelected = () => selectedEntryKey() === entryKey;
    const children = () => getVfsChildren(nodeProps.containerPath, nodeProps.entry.path);
    
    const handleClick = () => {
      setSelectedEntryKey(entryKey);
      if (nodeProps.entry.is_dir) toggleVfsDir(nodeProps.containerPath, nodeProps.entry.path);
      props.onSelectEntry({
        containerPath: nodeProps.containerPath,
        entryPath: nodeProps.entry.path,
        name: nodeProps.entry.name,
        size: nodeProps.entry.size,
        isDir: nodeProps.entry.is_dir,
        isVfsEntry: true,
      });
    };

    return (
      <>
        <VfsEntryRowComponent
          entry={nodeProps.entry}
          containerPath={nodeProps.containerPath}
          depth={nodeProps.depth}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          isSelected={isSelected()}
          partitionIndex={nodeProps.partitionIndex}
          onClick={handleClick}
          onToggle={() => toggleVfsDir(nodeProps.containerPath, nodeProps.entry.path)}
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
    // Use the actual mount_name from the partition (e.g., "Partition1_NTFS")
    const partitionRootPath = `/${partition.mount_name}`;
    const nodeKey = `${containerPath}::vfs::${partitionRootPath}`;
    const isExpanded = () => expandedVfsPaths().has(nodeKey);
    const isLoading = () => loading().has(nodeKey);
    const children = () => getVfsChildren(containerPath, partitionRootPath);

    const togglePartition = async () => {
      await toggleVfsDir(containerPath, partitionRootPath);
    };

    // Get filesystem icon
    const fsIcon = () => {
      const fs = partition.fs_type.toLowerCase();
      const iconClass = "w-4 h-4";
      if (fs.includes("ntfs")) return <HiOutlineServerStack class={`${iconClass} text-blue-400`} />;
      if (fs.includes("fat")) return <HiOutlineRectangleStack class={`${iconClass} text-yellow-400`} />;
      if (fs.includes("ext")) return <HiOutlineCommandLine class={`${iconClass} text-orange-400`} />;
      if (fs.includes("hfs") || fs.includes("apfs")) return <HiOutlineCircleStack class={`${iconClass} text-zinc-300`} />;
      return <HiOutlineCircleStack class={iconClass} />;
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
          <span class="w-4 text-xs text-zinc-500 flex items-center justify-center" aria-hidden="true">
            <ExpandIcon isLoading={isLoading()} isExpanded={isExpanded()} />
          </span>
          <span class="text-base" aria-hidden="true">{fsIcon()}</span>
          <span class="text-sm text-zinc-300">{partition.mount_name}</span>
          <span class="text-xs text-zinc-500">
            {partition.fs_type} • {formatBytes(partition.size)}
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

  // Archive entry node - wraps extracted ArchiveEntryRow with local state
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
    
    return (
      <>
        <ArchiveEntryRow
          entry={nodeProps.entry}
          containerPath={nodeProps.containerPath}
          depth={nodeProps.depth}
          isSelected={isSelected()}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          hasChildren={nodeProps.entry.is_dir && children().length > 0}
          isNestedContainer={isNestedContainer()}
          onClick={handleClick}
          onToggle={handleClick}
          onDblClick={isNestedContainer() ? handleDoubleClick : undefined}
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
    const entryKey = lazyKey(nodeProps.containerPath, nodeProps.entry.path);
    const isSelected = () => selectedEntryKey() === entryKey;
    const isExpanded = () => expandedLazyPaths().has(entryKey);
    const isLoading = () => loading().has(entryKey);
    
    // Get children from cache with memo
    const children = createMemo(() => {
      const cached = lazyChildrenCache().get(entryKey) || [];
      if (cached.length <= 1) return cached;
      return [...cached].sort((a, b) => {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
    });
    
    // Pagination state
    const hasMore = () => lazyHasMore().get(entryKey) || false;
    const totalCount = () => lazyTotalCounts().get(entryKey) || 0;
    const isLoadingMore = () => loading().has(`${entryKey}::more`);
    
    const handleClick = () => {
      setSelectedEntryKey(entryKey);
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
    
    return (
      <>
        <LazyEntryRow
          entry={nodeProps.entry}
          containerPath={nodeProps.containerPath}
          depth={nodeProps.depth}
          isSelected={isSelected()}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          onClick={handleClick}
          onToggle={handleToggle}
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
            <LoadMoreButton
              loadedCount={children().length}
              totalCount={totalCount()}
              isLoading={isLoadingMore()}
              depth={nodeProps.depth + 1}
              onClick={handleLoadMore}
            />
          </Show>
        </Show>
      </>
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
    
    // Selection state (inlined from former EntryRow wrapper)
    const entryKey = () => `${nodeProps.containerPath}::${nodeProps.entry.item_addr ?? nodeProps.entry.path}`;
    const isSelected = () => selectedEntryKey() === entryKey();

    return (
      <>
        <Ad1EntryRow
          entry={nodeProps.entry}
          containerPath={nodeProps.containerPath}
          depth={nodeProps.depth}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          isSelected={isSelected()}
          onToggle={() => toggleDirByAddr(nodeProps.containerPath, nodeProps.entry)}
          onClick={() => handleEntryClick(nodeProps.containerPath, nodeProps.entry)}
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
    const mountInfo = () => vfsMountCache().get(file.path);
    
    // AD1 root children
    const rootChildren = createMemo(() => {
      const entries = childrenCache().get(`${file.path}::root`) || [];
      return sortTreeEntries(entries);
    });
    
    // Archive entries
    const allArchiveEntries = createMemo(() => archiveTreeCache().get(file.path) || []);
    const archiveRootEntries = createMemo(() => sortArchiveEntries(getArchiveRootEntries(allArchiveEntries())));
    const archiveEntries = allArchiveEntries;
    
    // UFED entries (legacy fallback)
    const ufedEntries = createMemo(() => sortUfedEntries(ufedTreeCache().get(file.path) || []));
    
    // Lazy-loaded entries (for large containers)
    const lazyRootEntries = createMemo(() => sortLazyEntries(lazyChildrenCache().get(lazyKey(file.path)) || []));
    const lazySummary = () => lazySummaryCache().get(file.path) || null;
    const hasLazyData = () => lazyRootEntries().length > 0 || lazySummary() !== null;
    
    // Root-level pagination for UFED
    const hasMoreRootEntries = () => hasMoreLazyEntries(file.path);
    const rootTotalCount = () => getLazyTotalCount(file.path);
    const isLoadingMoreRoot = () => isLoadingMoreLazy(file.path, "root");
    const handleLoadMoreRoot = (e: MouseEvent) => {
      e.stopPropagation();
      loadMoreLazyEntries(file.path, "root");
    };
    
    // AD1 info
    const ad1Info = () => ad1InfoCache().get(file.path) || null;
    
    return (
      <div class="border-b border-zinc-800/50">
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
              <HiOutlineCircleStack class="w-4 h-4 text-cyan-400" />
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
                <HiOutlineCircleStack class="w-3.5 h-3.5 text-zinc-400" />
                <span class="text-zinc-400">{formatBytes(mountInfo()!.disk_size)}</span>
                <span>•</span>
                <span class="text-zinc-400">{mountInfo()!.partitions.length} partition(s)</span>
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
                <div class="py-2 text-xs text-zinc-500 italic" style={{ "padding-left": "32px" }}>
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
                <HiOutlineDocument class="w-3.5 h-3.5 text-zinc-400" />
                <span class="text-zinc-400">
                  {archiveEntries().filter(e => !e.is_dir).length.toLocaleString()} files
                </span>
                <span>•</span>
                <HiOutlineFolder class="w-3.5 h-3.5 text-zinc-400" />
                <span class="text-zinc-400">
                  {archiveEntries().filter(e => e.is_dir).length.toLocaleString()} folders
                </span>
                <span>•</span>
                <span class="text-zinc-400">
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
                      <HiOutlineDocument class="w-3.5 h-3.5 text-zinc-400" />
                      <span class="text-zinc-400">
                        {ufedEntries().filter(e => !e.is_dir).length.toLocaleString()} files
                      </span>
                      <span>•</span>
                      <HiOutlineFolder class="w-3.5 h-3.5 text-zinc-400" />
                      <span class="text-zinc-400">
                        {ufedEntries().filter(e => e.is_dir).length.toLocaleString()} folders
                      </span>
                    </>
                  }
                >
                  {/* Lazy loading summary */}
                  <HiOutlineServerStack class="w-3.5 h-3.5 text-cyan-400" />
                  <span class="text-cyan-400">
                    {lazySummary()!.entry_count.toLocaleString()} total entries
                  </span>
                  <Show when={lazySummary()!.lazy_loading_recommended}>
                    <span>•</span>
                    <span class="text-amber-400 text-xs">lazy loading</span>
                  </Show>
                </Show>
                <span>•</span>
                <span class="text-zinc-400">
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
                  // Fallback to old full-tree loading - inline UFED entry with selection
                  <For each={ufedEntries()}>
                    {(entry) => {
                      const entryKey = `${file.path}::ufed::${entry.path}`;
                      const isSelected = () => selectedEntryKey() === entryKey;
                      const handleClick = () => {
                        setSelectedEntryKey(entryKey);
                        props.onSelectEntry({
                          containerPath: file.path,
                          entryPath: entry.path,
                          name: entry.name,
                          size: entry.size || 0,
                          isDir: entry.is_dir,
                          isVfsEntry: false,
                        });
                      };
                      return (
                        <UfedEntryRow
                          entry={entry}
                          containerPath={file.path}
                          depth={0}
                          isSelected={isSelected()}
                          onClick={handleClick}
                        />
                      );
                    }}
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
                  <LoadMoreButton
                    loadedCount={lazyRootEntries().length}
                    totalCount={rootTotalCount()}
                    isLoading={isLoadingMoreRoot()}
                    depth={0}
                    onClick={handleLoadMoreRoot}
                  />
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
                  <HiOutlineDocument class="w-3.5 h-3.5 text-zinc-400" />
                  <span class="text-zinc-400">{ad1Info()!.file_count.toLocaleString()} files</span>
                  <span>•</span>
                  <HiOutlineFolder class="w-3.5 h-3.5 text-zinc-400" />
                  <span class="text-zinc-400">{ad1Info()!.dir_count.toLocaleString()} folders</span>
                  <span>•</span>
                  <span class="text-zinc-400">{formatBytes(ad1Info()!.total_size)}</span>
                  <Show when={ad1Info()!.source_name}>
                    <span>•</span>
                    <span class="text-cyan-400/80 truncate max-w-[200px]" title={ad1Info()!.source_name!}>
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
            
            {/* Unknown container type - display message */}
            <Show when={!isVfs && !isArchive && !isUfed && !isAd1}>
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
      class="flex flex-col h-full bg-zinc-900 text-sm"
      tabIndex={0}
      role="tree"
      aria-label="Evidence file tree"
      onKeyDown={handleTreeKeyDown}
    >
      {/* Type filter bar - shared component */}
      <TypeFilterBar
        containerStats={props.containerStats}
        totalCount={props.discoveredFiles.length}
        typeFilter={props.typeFilter}
        onToggleTypeFilter={props.onToggleTypeFilter}
        onClearTypeFilter={props.onClearTypeFilter}
        compact={true}
      />
      
      {/* Tree content */}
      <div class="flex-1 overflow-auto">
        <Show when={props.busy}>
          <div class="flex items-center justify-center py-8 text-zinc-500">Loading containers...</div>
        </Show>
        
        <Show when={!props.busy && filteredFiles().length === 0}>
          <div class="flex items-center justify-center py-8 text-zinc-500 text-center">
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
