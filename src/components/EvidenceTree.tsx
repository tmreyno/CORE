// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceTree - Unified lazy-loading tree for forensic containers
 * Supports: AD1, E01/Raw (VFS), Archives (ZIP/7z/RAR/TAR), UFED
 * 
 * FULLY MODULARIZED: Uses useEvidenceTree hook and extracted node components
 */

import { For, Show, createMemo, createSignal } from "solid-js";
import {
  HiOutlineCircleStack,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineExclamationTriangle,
} from "./icons";
import { getBasename } from "../utils";
import { 
  TreeEmptyState, 
  TreeErrorState, 
  ContainerHeader,
  LoadMoreButton,
  TREE_INFO_BAR_CLASSES,
  TREE_INFO_BAR_PADDING,
} from "./tree";
import type { DiscoveredFile, ArchiveTreeEntry } from "../types";
import {
  isVfsContainer,
  isL01Container,
  isAd1Container,
  isArchiveContainer,
  isUfedContainer,
} from "./EvidenceTree/containerDetection";
import { TypeFilterBar } from "./TypeFilterBar";

// Import SelectedEntry from canonical location
import type { SelectedEntry, TreeExpansionState } from "./EvidenceTree/types";
// Re-export for backward compatibility
export type { SelectedEntry };

// Extracted node components
import { 
  Ad1TreeNode,
  PartitionNode,
  ArchiveTreeNode,
  LazyTreeNode,
} from "./EvidenceTree/nodes";

// Master hook for all tree state
import { useEvidenceTree } from "./EvidenceTree/hooks";

import type { FileStatus, FileHashInfo } from "../hooks";
import type { HashHistoryEntry, ContainerInfo } from "../types";

// ---------------------------------------------------------------------------
// Render-cap: avoid rendering thousands of DOM nodes in flat directories.
// When a directory has more entries than this, a "Show more" toggle appears.
// ---------------------------------------------------------------------------
const RENDER_CAP = 200;
const RENDER_CAP_STEP = 200;

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
  onOpenNestedContainer?: (tempPath: string, originalName: string, containerType: string, parentPath: string) => void;
  
  // Tree expansion state persistence
  initialExpansionState?: TreeExpansionState;
  onExpansionStateChange?: (state: TreeExpansionState) => void;
  
  // Selection & Hashing Props
  selectedFiles?: Set<string>;
  fileHashMap?: Map<string, FileHashInfo>;
  hashHistory?: Map<string, HashHistoryEntry[]>;
  fileStatusMap?: Map<string, FileStatus>;
  fileInfoMap?: Map<string, ContainerInfo>;
  onToggleFileSelection?: (path: string) => void;
  onHashFile?: (file: DiscoveredFile) => void;
  onContextMenu?: (file: DiscoveredFile, e: MouseEvent) => void;
  allFilesSelected?: boolean;
  onToggleSelectAll?: () => void;
}

export function EvidenceTree(props: EvidenceTreeProps) {
  // Use the master hook for all tree state management
  const tree = useEvidenceTree({
    discoveredFiles: () => props.discoveredFiles,
    typeFilter: () => props.typeFilter,
    onSelectEntry: props.onSelectEntry,
    onOpenNestedContainer: props.onOpenNestedContainer,
    initialExpansionState: props.initialExpansionState,
    onExpansionStateChange: props.onExpansionStateChange,
  });

  // Keyboard navigation
  const handleTreeKeyDown = (e: KeyboardEvent) => {
    const items = Array.from((e.currentTarget as HTMLElement).querySelectorAll<HTMLElement>('[role="treeitem"], [data-tree-item]'));
    if (items.length === 0) return;
    
    const focusedItem = (e.currentTarget as HTMLElement).querySelector<HTMLElement>(':focus, [data-focused="true"]');
    const currentIndex = focusedItem ? items.indexOf(focusedItem) : -1;
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

  // Archive children helper - use the hook's synthesized version
  const getArchiveChildren = (allEntries: ArchiveTreeEntry[], parentPath: string): ArchiveTreeEntry[] => {
    return tree.archive.getArchiveChildren(allEntries, parentPath);
  };

  // Container Node component
  const ContainerNode = (containerProps: { file: DiscoveredFile }) => {
    const file = containerProps.file;
    const isExpanded = () => tree.isContainerExpanded(file.path);
    const isLoading = () => tree.isLoading(file.path);
    const containerType = file.container_type.toLowerCase();
    const isVfs = isVfsContainer(containerType);
    const isL01 = isL01Container(containerType);
    const isArchive = isArchiveContainer(containerType);
    const isUfed = isUfedContainer(containerType);
    const isAd1 = isAd1Container(containerType);
    
    const mountInfo = () => tree.getVfsMountInfo(file.path);
    const rootChildren = createMemo(() => tree.getAd1RootChildren(file.path));
    const archiveRootEntries = createMemo(() => tree.sortArchiveEntries(tree.getArchiveRootEntries(file.path)));
    const allArchiveEntries = createMemo(() => tree.getArchiveEntries(file.path));
    const allSynthesizedEntries = createMemo(() => tree.archive.getAllWithSyntheticDirs(allArchiveEntries()));
    const archiveFileCount = createMemo(() => allSynthesizedEntries().filter(e => !e.isDir).length);
    const archiveFolderCount = createMemo(() => allSynthesizedEntries().filter(e => e.isDir).length);
    const lazyRootEntries = createMemo(() => tree.getLazyRootEntries(file.path));
    const hasLazyData = () => tree.hasLazyData(file.path);
    const ad1Info = () => tree.getAd1Info(file.path);
    
    // AD1 container status (segment availability)
    const ad1ContainerStatus = createMemo(() => isAd1 ? tree.getAd1ContainerStatus(file.path) : undefined);
    const isIncompleteAd1 = () => ad1ContainerStatus() && !ad1ContainerStatus()!.isComplete;
    const incompleteMessage = () => ad1ContainerStatus()?.statusMessage || "Container has missing segments";

    // Lazy loading helpers
    const lazyKey = (path: string = "root") => `${file.path}::lazy::${path}`;

    // --- Render caps for flat directories ---------------------------------
    // Track per-container render caps (archive root, AD1 root)
    const [archiveRenderLimit, setArchiveRenderLimit] = createSignal(RENDER_CAP);
    const [ad1RenderLimit, setAd1RenderLimit] = createSignal(RENDER_CAP);

    // Reset caps when container is collapsed/re-expanded
    const onContainerClick = () => {
      props.onSelectContainer(file);
      tree.toggleContainer(file);
      setArchiveRenderLimit(RENDER_CAP);
      setAd1RenderLimit(RENDER_CAP);
    };

    return (
      <div>
        <ContainerHeader
          name={file.filename || getBasename(file.path) || file.path}
          path={file.path}
          containerType={file.container_type}
          isActive={props.activeFile?.path === file.path}
          isExpanded={isExpanded()}
          isLoading={isLoading()}
          segmentCount={file.segment_count}
          isIncomplete={isIncompleteAd1()}
          incompleteMessage={incompleteMessage()}
          onClick={onContainerClick}
          statusIcon={isVfs && mountInfo() ? <span title="Mounted disk image"><HiOutlineCircleStack class={`w-3.5 h-3.5 text-accent`} /></span> : undefined}
          isChecked={props.selectedFiles?.has(file.path)}
          onToggleSelection={props.onToggleFileSelection ? (e) => { e.stopPropagation(); props.onToggleFileSelection!(file.path); } : undefined}
          onHash={props.onHashFile ? (e) => { e.stopPropagation(); props.onHashFile!(file); } : undefined}
          fileStatus={props.fileStatusMap?.get(file.path)}
          fileHash={props.fileHashMap?.get(file.path)}
          hashHistory={props.hashHistory?.get(file.path)}
          fileInfo={props.fileInfoMap?.get(file.path)}
          busy={props.busy}
          onContextMenu={props.onContextMenu ? (e) => { e.preventDefault(); props.onContextMenu!(file, e); } : undefined}
        />
        
        <Show when={isExpanded()}>
          <div class="pb-1">
            {/* VFS Container (E01, Raw) */}
            <Show when={isVfs && mountInfo()}>
              <div class={TREE_INFO_BAR_CLASSES} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                <HiOutlineCircleStack class={`w-3 h-3 text-txt-secondary`} />
                <span class="text-txt-secondary">{mountInfo()!.partitions.length} partition(s)</span>
              </div>
              <For each={mountInfo()!.partitions}>
                {(partition, index) => (
                  <PartitionNode
                    partition={partition}
                    containerPath={file.path}
                    index={index()}
                    isExpanded={(key) => tree.vfs.expandedVfsPaths().has(key)}
                    isLoading={(key) => tree.isLoading(key)}
                    isSelected={(key) => tree.isSelected(key)}
                    getChildren={(cp, vp) => tree.sortVfsEntries(tree.vfs.getVfsChildren(cp, vp))}
                    onToggle={async (cp, vp) => {
                      // Use proper loading state management via exposed setLoadingState
                      const nodeKey = `${cp}::vfs::${vp}`;
                      const setLoadingForNode = (fn: (prev: Set<string>) => Set<string>) => {
                        const newLoading = fn(tree.loading());
                        if (newLoading.has(nodeKey)) {
                          tree.setLoadingState(nodeKey, true);
                        } else {
                          tree.setLoadingState(nodeKey, false);
                        }
                      };
                      await tree.vfs.toggleVfsDir(cp, vp, tree.loading(), setLoadingForNode);
                    }}
                    onEntryClick={(cp, entry, _pi) => {
                      tree.setSelectedEntryKey(`${cp}::vfs::${entry.path}`);
                      props.onSelectEntry({ containerPath: cp, entryPath: entry.path, name: entry.name, size: entry.size, isDir: entry.isDir, isVfsEntry: true });
                    }}
                    // Nested container support
                    isNestedExpanded={(parentPath, nestedPath) => tree.nested.isNestedExpanded(parentPath, nestedPath)}
                    isNestedLoading={(parentPath, nestedPath) => tree.nested.isNestedLoading(parentPath, nestedPath)}
                    getNestedEntries={(parentPath, nestedPath) => tree.nested.getNestedRootEntries(parentPath, nestedPath)}
                    getNestedChildren={(parentPath, nestedPath, entryPath) => tree.nested.getNestedChildren(parentPath, nestedPath, entryPath)}
                    onToggleNested={async (parentPath, nestedPath) => tree.nested.toggleNestedContainer(parentPath, nestedPath)}
                    onNestedClick={(parentPath, nestedPath, entry) => {
                      tree.setSelectedEntryKey(`${parentPath}::nested::${nestedPath}::${entry.path}`);
                      props.onSelectEntry({ 
                        containerPath: parentPath, 
                        entryPath: `${nestedPath}::${entry.path}`, 
                        name: entry.name, 
                        size: entry.size || 0, 
                        isDir: entry.isDir, 
                        isVfsEntry: false,
                        isArchiveEntry: true,
                        containerType: entry.nestedType ?? undefined,
                        metadata: {
                          archiveFormat: entry.sourceType ?? "Unknown",
                          totalEntries: 0,
                          archiveSize: 0,
                          encrypted: false,
                          totalFiles: 0,
                          totalFolders: 0,
                          entryModified: entry.modified || undefined,
                        },
                      });
                    }}
                  />
                )}
              </For>
            </Show>
            
            <Show when={isVfs && !mountInfo() && !isLoading()}>
              <Show when={isL01}><TreeEmptyState message="L01 logical evidence container" hint="Logical evidence files (L01) store individual items without a filesystem structure. Use the Info panel to view acquisition details and stored hashes." /></Show>
              <Show when={!isL01}><TreeEmptyState message={`Format "${file.container_type}" not supported`} hint="VFS mounting failed" /></Show>
            </Show>
            
            {/* Archive Container */}
            <Show when={isArchive}>
              <div class={TREE_INFO_BAR_CLASSES} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                <HiOutlineDocument class={`w-3 h-3 text-txt-secondary`} />
                <span class="text-txt-secondary">{archiveFileCount().toLocaleString()} files</span>
                <span>•</span>
                <HiOutlineFolder class={`w-3 h-3 text-txt-secondary`} />
                <span class="text-txt-secondary">{archiveFolderCount().toLocaleString()} folders</span>
              </div>
              <For each={archiveRootEntries().slice(0, archiveRenderLimit())}>
                {(entry) => (
                  <ArchiveTreeNode
                    entry={entry}
                    containerPath={file.path}
                    depth={0}
                    isExpanded={(key) => tree.archive.expandedArchivePaths().has(key)}
                    isLoading={(key) => tree.isLoading(key)}
                    isSelected={(key) => tree.isSelected(key)}
                    getChildren={(_cp, ap) => tree.sortArchiveEntries(getArchiveChildren(allArchiveEntries(), ap))}
                    onToggle={async (cp, ap) => tree.archive.toggleArchiveDir(cp, ap)}
                    onClick={(cp, entry) => {
                      tree.setSelectedEntryKey(`${cp}::archive::${entry.path}`);
                      if (entry.isDir) tree.archive.toggleArchiveDir(cp, entry.path);
                      const archiveMeta = tree.archive.archiveMetaCache().get(cp);
                      props.onSelectEntry({ 
                        containerPath: cp, 
                        entryPath: entry.path, 
                        name: entry.name || getBasename(entry.path) || entry.path, 
                        size: entry.size, 
                        isDir: entry.isDir, 
                        isVfsEntry: false,
                        isArchiveEntry: true,
                        metadata: {
                          archiveFormat: archiveMeta?.format ?? "Unknown",
                          totalEntries: archiveMeta?.entry_count ?? 0,
                          archiveSize: archiveMeta?.archive_size ?? 0,
                          encrypted: archiveMeta?.encrypted ?? false,
                          totalFiles: archiveFileCount(),
                          totalFolders: archiveFolderCount(),
                          entryCompressedSize: entry.compressedSize || undefined,
                          entryCrc32: entry.crc32 || undefined,
                          entryModified: entry.modified || undefined,
                        },
                      });
                    }}
                    // Nested container support
                    isNestedExpanded={(parentPath, nestedPath) => tree.nested.isNestedExpanded(parentPath, nestedPath)}
                    isNestedLoading={(parentPath, nestedPath) => tree.nested.isNestedLoading(parentPath, nestedPath)}
                    getNestedEntries={(parentPath, nestedPath) => tree.nested.getNestedRootEntries(parentPath, nestedPath)}
                    getNestedChildren={(parentPath, nestedPath, entryPath) => tree.nested.getNestedChildren(parentPath, nestedPath, entryPath)}
                    onToggleNested={async (parentPath, nestedPath) => tree.nested.toggleNestedContainer(parentPath, nestedPath)}
                    onNestedClick={(parentPath, nestedPath, entry) => {
                      tree.setSelectedEntryKey(`${parentPath}::nested::${nestedPath}::${entry.path}`);
                      const archiveMeta = tree.archive.archiveMetaCache().get(parentPath);
                      props.onSelectEntry({ 
                        containerPath: parentPath, 
                        entryPath: `${nestedPath}::${entry.path}`, 
                        name: entry.name, 
                        size: entry.size || 0, 
                        isDir: entry.isDir, 
                        isVfsEntry: false,
                        isArchiveEntry: true,
                        containerType: entry.nestedType ?? undefined,
                        metadata: {
                          archiveFormat: archiveMeta?.format ?? "Unknown",
                          totalEntries: archiveMeta?.entry_count ?? 0,
                          archiveSize: archiveMeta?.archive_size ?? 0,
                          encrypted: archiveMeta?.encrypted ?? false,
                          totalFiles: archiveFileCount(),
                          totalFolders: archiveFolderCount(),
                          entryModified: entry.modified || undefined,
                        },
                      });
                    }}
                  />
                )}
              </For>
              <Show when={archiveRootEntries().length > archiveRenderLimit()}>
                <LoadMoreButton
                  loadedCount={archiveRenderLimit()}
                  totalCount={archiveRootEntries().length}
                  isLoading={false}
                  depth={0}
                  onClick={() => setArchiveRenderLimit(prev => prev + RENDER_CAP_STEP)}
                />
              </Show>
              <Show when={archiveRootEntries().length === 0 && !isLoading()}><TreeEmptyState message="Empty archive" /></Show>
            </Show>

            {/* UFED Container */}
            <Show when={isUfed && hasLazyData()}>
              <For each={lazyRootEntries()}>
                {(entry) => (
                  <LazyTreeNode
                    entry={entry}
                    containerPath={file.path}
                    depth={0}
                    isExpanded={(_cp, ep) => tree.lazy.expandedLazyPaths().has(lazyKey(ep))}
                    isLoading={(key) => tree.isLoading(key)}
                    isSelected={(key) => tree.isSelected(key)}
                    getChildren={(_cp, pp) => tree.sortLazyEntries(tree.lazy.lazyChildrenCache().get(lazyKey(pp)) || [])}
                    hasMoreChildren={(_cp, pp) => tree.lazy.lazyHasMore().get(lazyKey(pp)) || false}
                    getLoadedCount={(_cp, pp) => (tree.lazy.lazyChildrenCache().get(lazyKey(pp)) || []).length}
                    getTotalCount={(_cp, pp) => tree.lazy.lazyTotalCounts().get(lazyKey(pp)) || 0}
                    onToggle={(cp, ep) => tree.lazy.toggleLazyDir(cp, ep, tree.loading(), () => {})}
                    onClick={(cp, entry) => {
                      tree.setSelectedEntryKey(lazyKey(entry.path));
                      props.onSelectEntry({ 
                        containerPath: cp, 
                        entryPath: entry.path, 
                        name: entry.name, 
                        size: entry.size || 0, 
                        isDir: entry.is_dir, 
                        isVfsEntry: false,
                        isArchiveEntry: true,
                        containerType: "ufed",
                      });
                    }}
                    onLoadMore={(cp, pp) => tree.lazy.loadMoreLazyEntries(cp, pp, tree.loading(), () => {})}
                  />
                )}
              </For>
              <Show when={tree.lazy.lazyHasMore().get(lazyKey())}>
                <LoadMoreButton
                  loadedCount={lazyRootEntries().length}
                  totalCount={tree.lazy.lazyTotalCounts().get(lazyKey()) || 0}
                  isLoading={tree.isLoading(`${lazyKey()}::more`)}
                  depth={0}
                  onClick={(e) => { e.stopPropagation(); tree.lazy.loadMoreLazyEntries(file.path, "root", tree.loading(), () => {}); }}
                />
              </Show>
            </Show>
            <Show when={isUfed && !hasLazyData() && !isLoading()}><TreeEmptyState message="Empty UFED extraction" /></Show>
            
            {/* AD1 Container */}
            <Show when={isAd1}>
              {/* Segment status warning for incomplete containers */}
              <Show when={isIncompleteAd1()}>
                <div class={`${TREE_INFO_BAR_CLASSES} bg-warning/10 border-l-2 border-warning`} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                  <HiOutlineExclamationTriangle class={`w-3 h-3 text-warning`} />
                  <span class="text-warning text-xs">
                    {ad1ContainerStatus()?.availableSegments} of {ad1ContainerStatus()?.expectedSegments} segments available
                  </span>
                  <span class="text-txt-muted text-xs">
                    (missing: {ad1ContainerStatus()?.missingSegments.map(i => `.ad${i}`).join(', ')})
                  </span>
                </div>
              </Show>
              <Show when={ad1Info()}>
                <div class={TREE_INFO_BAR_CLASSES} style={{ "padding-left": TREE_INFO_BAR_PADDING }}>
                  <HiOutlineDocument class={`w-3 h-3 text-txt-secondary`} />
                  <span class="text-txt-secondary">{ad1Info()!.file_count.toLocaleString()} files</span>
                  <span>•</span>
                  <HiOutlineFolder class={`w-3 h-3 text-txt-secondary`} />
                  <span class="text-txt-secondary">{ad1Info()!.dir_count.toLocaleString()} folders</span>
                </div>
              </Show>
              <For each={rootChildren().slice(0, ad1RenderLimit())}>
                {(entry) => (
                  <Ad1TreeNode
                    entry={entry}
                    containerPath={file.path}
                    depth={0}
                    isExpanded={(cp, e) => tree.ad1.isDirExpanded(cp, e)}
                    isLoading={(key) => tree.isLoading(key)}
                    isSelected={(key) => tree.isSelected(key)}
                    getChildren={(cp, e) => tree.ad1.getChildrenForEntry(cp, e)}
                    onToggle={(cp, e) => tree.ad1.toggleDirByAddr(cp, e, tree.loading(), () => {})}
                    onClick={tree.handleEntryClick}
                    // Nested container support
                    isNestedExpanded={(parentPath, nestedPath) => tree.nested.isNestedExpanded(parentPath, nestedPath)}
                    isNestedLoading={(parentPath, nestedPath) => tree.nested.isNestedLoading(parentPath, nestedPath)}
                    getNestedEntries={(parentPath, nestedPath) => tree.nested.getNestedRootEntries(parentPath, nestedPath)}
                    getNestedChildren={(parentPath, nestedPath, entryPath) => tree.nested.getNestedChildren(parentPath, nestedPath, entryPath)}
                    onToggleNested={async (parentPath, nestedPath) => tree.nested.toggleNestedContainer(parentPath, nestedPath)}
                    onNestedClick={(parentPath, nestedPath, entry) => {
                      tree.setSelectedEntryKey(`${parentPath}::nested::${nestedPath}::${entry.path}`);
                      props.onSelectEntry({ 
                        containerPath: parentPath, 
                        entryPath: `${nestedPath}::${entry.path}`, 
                        name: entry.name, 
                        size: entry.size || 0, 
                        isDir: entry.isDir, 
                        isVfsEntry: false,
                        isArchiveEntry: true,
                        containerType: entry.nestedType ?? undefined,
                        metadata: {
                          archiveFormat: entry.sourceType ?? "Unknown",
                          totalEntries: 0,
                          archiveSize: 0,
                          encrypted: false,
                          totalFiles: 0,
                          totalFolders: 0,
                          entryModified: entry.modified || undefined,
                        },
                      });
                    }}
                  />
                )}
              </For>
              <Show when={rootChildren().length > ad1RenderLimit()}>
                <LoadMoreButton
                  loadedCount={ad1RenderLimit()}
                  totalCount={rootChildren().length}
                  isLoading={false}
                  depth={0}
                  onClick={() => setAd1RenderLimit(prev => prev + RENDER_CAP_STEP)}
                />
              </Show>
              <Show when={rootChildren().length === 0 && !isLoading()}>
                <Show when={tree.ad1.containerErrors().has(file.path)}>
                  <TreeErrorState message={tree.ad1.containerErrors().get(file.path)!} onRetry={() => tree.toggleContainer(file)} />
                </Show>
                <Show when={!tree.ad1.containerErrors().has(file.path)}><TreeEmptyState message="Empty container" /></Show>
              </Show>
            </Show>

            <Show when={!isVfs && !isArchive && !isUfed && !isAd1}>
              <TreeEmptyState message={`Format "${file.container_type}" not supported`} hint="Tree browsing unavailable" />
            </Show>
          </div>
        </Show>
      </div>
    );
  };

  return (
    <div class="flex flex-col h-full bg-bg text-sm" tabIndex={0} role="tree" aria-label="Evidence file tree" onKeyDown={handleTreeKeyDown}>
      <TypeFilterBar containerStats={props.containerStats} totalCount={props.discoveredFiles.length} typeFilter={props.typeFilter} onToggleTypeFilter={props.onToggleTypeFilter} onClearTypeFilter={props.onClearTypeFilter} compact={true} />
      
      <Show when={tree.filteredFiles().length > 0}>
        <div class="flex items-center gap-1.5 px-2 py-1 bg-bg-panel/20">
          {/* Expand/Collapse All Buttons */}
          <div class="flex items-center gap-0.5">
            <button
              class="w-5 h-5 flex items-center justify-center text-txt-secondary hover:text-txt hover:bg-bg-hover/50 rounded transition-colors"
              title="Expand all containers"
              onClick={() => tree.expandAllContainers()}
            >
              <span class="text-sm font-medium">+</span>
            </button>
            <button
              class="w-5 h-5 flex items-center justify-center text-txt-secondary hover:text-txt hover:bg-bg-hover/50 rounded transition-colors"
              title="Collapse all containers"
              onClick={() => tree.collapseAllContainers()}
            >
              <span class="text-sm font-medium">−</span>
            </button>
          </div>
          
          <Show when={props.onToggleSelectAll}>
            <span class="text-txt-muted">|</span>
            <label class={`flex items-center gap-1.5 text-2xs leading-tight text-txt-secondary cursor-pointer hover:text-txt-tertiary transition-colors`}>
              <input type="checkbox" class="w-2.5 h-2.5 accent-accent" checked={props.allFilesSelected || false} onChange={() => props.onToggleSelectAll?.()} />
              <span>{props.allFilesSelected ? "Deselect All" : "Select All"}{props.typeFilter ? ` (${tree.filteredFiles().length} shown)` : ""}</span>
            </label>
          </Show>
        </div>
      </Show>
      
      <div class="flex-1 overflow-auto">
        <Show when={props.busy}><div class="flex items-center justify-center py-8 text-txt-muted">Loading containers...</div></Show>
        <Show when={!props.busy && tree.filteredFiles().length === 0}><div class="flex items-center justify-center py-8 text-txt-muted text-center">No forensic containers found. Add evidence files to begin.</div></Show>
        <For each={tree.filteredFiles()}>{(file) => <ContainerNode file={file} />}</For>
      </div>
    </div>
  );
}
