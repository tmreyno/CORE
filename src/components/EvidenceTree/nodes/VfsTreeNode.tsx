// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * VfsTreeNode - Tree node components for VFS containers (E01, Raw, L01)
 * 
 * Renders VFS partition and filesystem entries with expand/collapse.
 * Supports inline expansion of nested containers (ZIP, AD1, etc. inside E01/Raw).
 */

import { For, Show, JSX, createMemo, createSignal } from "solid-js";
import { 
  HiOutlineCircleStack,
  HiOutlineServerStack,
  HiOutlineRectangleStack,
  HiOutlineCommandLine,
} from "../../icons";
import { 
  TreeRow,
  ExpandIcon,
  TreeEmptyState,
  TREE_ROW_BASE_CLASSES,
  TREE_ROW_NORMAL_CLASSES,
  getTreeIndent,
} from "../../tree";
import { formatBytes } from "../../../utils";
import type { VfsEntry, VfsPartitionInfo, NestedContainerEntry } from "../../../types";
import { isNestedContainerFile, getNestedContainerType } from "../containerDetection";

export interface VfsEntryRowProps {
  entry: VfsEntry;
  containerPath: string;
  depth: number;
  isExpanded: boolean;
  isLoading: boolean;
  isSelected: boolean;
  hasChildren: boolean;
  isNestedContainer?: boolean;
  partitionIndex: number;
  onToggle: () => void;
  onClick: () => void;
}

/**
 * VFS Entry Row - displays a single VFS file/folder entry
 */
export function VfsEntryRow(props: VfsEntryRowProps): JSX.Element {
  return (
    <TreeRow
      name={props.entry.name}
      path={props.entry.path}
      isDir={props.entry.isDir}
      size={props.entry.size}
      depth={props.depth}
      isSelected={props.isSelected}
      isExpanded={props.isExpanded}
      isLoading={props.isLoading}
      hasChildren={props.hasChildren}
      onClick={props.onClick}
      onToggle={props.onToggle}
      entryType={props.isNestedContainer ? "container" : undefined}
      data-entry-path={props.entry.path}
    />
  );
}

export interface VfsTreeNodeProps {
  entry: VfsEntry;
  containerPath: string;
  depth: number;
  partitionIndex: number;
  // State accessors
  isExpanded: (nodeKey: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (containerPath: string, vfsPath: string) => VfsEntry[];
  // Event handlers
  onToggle: (containerPath: string, vfsPath: string) => Promise<void>;
  onClick: (containerPath: string, entry: VfsEntry, partitionIndex: number) => void;
  // Nested container support (optional)
  isNestedExpanded?: (parentPath: string, nestedPath: string) => boolean;
  isNestedLoading?: (parentPath: string, nestedPath: string) => boolean;
  getNestedEntries?: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onToggleNested?: (parentPath: string, nestedPath: string) => Promise<void>;
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

/**
 * VFS Tree Node - recursive tree node for VFS filesystem entries
 * Supports inline expansion of nested containers (ZIP, AD1, etc.)
 */
export function VfsTreeNode(props: VfsTreeNodeProps): JSX.Element {
  const nodeKey = () => `${props.containerPath}::vfs::${props.entry.path}`;
  const isExpanded = () => props.isExpanded(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  const isSelected = () => props.isSelected(nodeKey());
  const children = () => props.getChildren(props.containerPath, props.entry.path);

  // Check if this entry is a nested container file
  const isNestedContainer = createMemo(() => {
    if (props.entry.isDir) return false;
    return isNestedContainerFile(props.entry.name);
  });
  
  const nestedContainerType = createMemo(() => {
    if (!isNestedContainer()) return null;
    return getNestedContainerType(props.entry.name);
  });
  
  // Check if nested container is expanded
  const isNestedExp = () => {
    if (!isNestedContainer() || !props.isNestedExpanded) return false;
    return props.isNestedExpanded(props.containerPath, props.entry.path);
  };
  
  const isNestedLoad = () => {
    if (!isNestedContainer() || !props.isNestedLoading) return false;
    return props.isNestedLoading(props.containerPath, props.entry.path);
  };
  
  // Get nested container entries
  const nestedEntries = createMemo(() => {
    if (!isNestedContainer() || !props.getNestedEntries) return [];
    return props.getNestedEntries(props.containerPath, props.entry.path);
  });
  
  // Root entries of the nested container
  const nestedRootEntries = createMemo(() => {
    const entries = nestedEntries();
    if (entries.length === 0) return [];
    return entries.filter(e => {
      const path = e.path.replace(/\/$/, '');
      const parts = path.split('/').filter(p => p);
      return parts.length === 1;
    }).sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  });

  // Determine if entry has children (directory or nested container)
  const hasChildren = () => {
    if (props.entry.isDir) return true;
    if (isNestedContainer()) return true;
    return false;
  };

  return (
    <>
      <VfsEntryRow
        entry={props.entry}
        containerPath={props.containerPath}
        depth={props.depth}
        isExpanded={isExpanded() || isNestedExp()}
        isLoading={isLoading() || isNestedLoad()}
        isSelected={isSelected()}
        hasChildren={hasChildren()}
        isNestedContainer={isNestedContainer()}
        partitionIndex={props.partitionIndex}
        onToggle={() => {
          if (isNestedContainer() && props.onToggleNested) {
            props.onToggleNested(props.containerPath, props.entry.path);
          } else {
            props.onToggle(props.containerPath, props.entry.path);
          }
        }}
        onClick={() => props.onClick(props.containerPath, props.entry, props.partitionIndex)}
      />
      
      {/* Regular directory children */}
      <Show when={isExpanded() && props.entry.isDir}>
        <For each={children()}>
          {(child) => (
            <VfsTreeNode
              entry={child}
              containerPath={props.containerPath}
              depth={props.depth + 1}
              partitionIndex={props.partitionIndex}
              isExpanded={props.isExpanded}
              isLoading={props.isLoading}
              isSelected={props.isSelected}
              getChildren={props.getChildren}
              onToggle={props.onToggle}
              onClick={props.onClick}
              isNestedExpanded={props.isNestedExpanded}
              isNestedLoading={props.isNestedLoading}
              getNestedEntries={props.getNestedEntries}
              getNestedChildren={props.getNestedChildren}
              onToggleNested={props.onToggleNested}
              onNestedClick={props.onNestedClick}
            />
          )}
        </For>
      </Show>
      
      {/* Nested container contents (when expanded) */}
      <Show when={isNestedExp() && isNestedContainer()}>
        <For each={nestedRootEntries()}>
          {(nestedEntry) => (
            <VfsNestedEntryNode
              entry={nestedEntry}
              parentContainerPath={props.containerPath}
              nestedContainerPath={props.entry.path}
              depth={props.depth + 1}
              isSelected={props.isSelected}
              isLoading={props.isLoading}
              getNestedChildren={props.getNestedChildren}
              onNestedClick={props.onNestedClick}
            />
          )}
        </For>
        <Show when={nestedRootEntries().length === 0 && !isNestedLoad()}>
          <TreeEmptyState 
            message={`Empty ${nestedContainerType() || 'container'}`} 
            depth={props.depth + 1} 
          />
        </Show>
      </Show>
    </>
  );
}

/**
 * Nested Container Entry Node for VFS trees
 * Renders entries from inside a nested container (e.g., files inside ZIP that's inside E01)
 */
interface VfsNestedEntryNodeProps {
  entry: NestedContainerEntry;
  parentContainerPath: string;
  nestedContainerPath: string;
  depth: number;
  isSelected: (key: string) => boolean;
  isLoading: (key: string) => boolean;
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

function VfsNestedEntryNode(props: VfsNestedEntryNodeProps): JSX.Element {
  const nodeKey = () => `${props.parentContainerPath}::nested::${props.nestedContainerPath}::${props.entry.path}`;
  const isSelected = () => props.isSelected(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  
  const [isExpanded, setIsExpanded] = createSignal(false);
  
  const children = createMemo(() => {
    if (!props.entry.isDir || !props.getNestedChildren) return [];
    return props.getNestedChildren(props.parentContainerPath, props.nestedContainerPath, props.entry.path);
  });
  
  const sortedChildren = createMemo(() => {
    return [...children()].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  });
  
  const hasChildren = () => props.entry.isDir || props.entry.isNestedContainer;
  
  const handleToggle = () => {
    if (props.entry.isDir) {
      setIsExpanded(!isExpanded());
    }
    props.onNestedClick?.(props.parentContainerPath, props.nestedContainerPath, props.entry);
  };
  
  return (
    <>
      <TreeRow
        name={props.entry.name}
        path={props.entry.path}
        isDir={props.entry.isDir}
        size={props.entry.size || 0}
        depth={props.depth}
        isSelected={isSelected()}
        isExpanded={isExpanded()}
        isLoading={isLoading()}
        hasChildren={hasChildren()}
        onClick={handleToggle}
        onToggle={handleToggle}
        entryType={props.entry.isNestedContainer ? "container" : undefined}
        data-entry-path={props.entry.path}
        data-source-type={props.entry.sourceType}
      />
      
      <Show when={isExpanded() && props.entry.isDir}>
        <For each={sortedChildren()}>
          {(child) => (
            <VfsNestedEntryNode
              entry={child}
              parentContainerPath={props.parentContainerPath}
              nestedContainerPath={props.nestedContainerPath}
              depth={props.depth + 1}
              isSelected={props.isSelected}
              isLoading={props.isLoading}
              getNestedChildren={props.getNestedChildren}
              onNestedClick={props.onNestedClick}
            />
          )}
        </For>
        <Show when={sortedChildren().length === 0 && !isLoading()}>
          <TreeEmptyState message="Empty folder" depth={props.depth + 1} />
        </Show>
      </Show>
    </>
  );
}

export interface PartitionNodeProps {
  partition: VfsPartitionInfo;
  containerPath: string;
  index: number;
  // State accessors
  isExpanded: (nodeKey: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (containerPath: string, vfsPath: string) => VfsEntry[];
  // Event handlers
  onToggle: (containerPath: string, vfsPath: string) => Promise<void>;
  onEntryClick: (containerPath: string, entry: VfsEntry, partitionIndex: number) => void;
  // Nested container support (optional)
  isNestedExpanded?: (parentPath: string, nestedPath: string) => boolean;
  isNestedLoading?: (parentPath: string, nestedPath: string) => boolean;
  getNestedEntries?: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onToggleNested?: (parentPath: string, nestedPath: string) => Promise<void>;
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

/**
 * Partition Node - displays a partition with its filesystem tree
 */
export function PartitionNode(props: PartitionNodeProps): JSX.Element {
  // Handle missing mountName with fallback
  const partitionRootPath = () => {
    const mountName = props.partition.mountName ?? `Partition${props.partition.number ?? props.index + 1}`;
    return `/${mountName}`;
  };
  const nodeKey = () => `${props.containerPath}::vfs::${partitionRootPath()}`;
  const isExpanded = () => props.isExpanded(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  const children = () => props.getChildren(props.containerPath, partitionRootPath());

  const togglePartition = () => props.onToggle(props.containerPath, partitionRootPath());

  // Get filesystem icon based on type
  const fsIcon = (): JSX.Element => {
    const fs = (props.partition.fsType ?? '').toLowerCase();
    const iconClass = "w-4 h-4";
    if (fs.includes("ntfs")) return <HiOutlineServerStack class={`${iconClass} text-blue-400`} />;
    if (fs.includes("fat")) return <HiOutlineRectangleStack class={`${iconClass} text-yellow-400`} />;
    if (fs.includes("ext")) return <HiOutlineCommandLine class={`${iconClass} text-orange-400`} />;
    if (fs.includes("hfs") || fs.includes("apfs")) return <HiOutlineCircleStack class={`${iconClass} text-txt-tertiary`} />;
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
        <span class="w-4 text-xs text-txt-muted flex items-center justify-center" aria-hidden="true">
          <ExpandIcon isLoading={isLoading()} isExpanded={isExpanded()} />
        </span>
        <span class="text-base" aria-hidden="true">{fsIcon()}</span>
        <span class="text-sm text-txt-tertiary">{props.partition.mountName ?? `Partition ${props.partition.number ?? props.index + 1}`}</span>
        <span class="text-xs text-txt-muted">
          {props.partition.fsType ?? 'Unknown'} • {formatBytes(props.partition.size ?? 0)}
        </span>
      </div>
      <Show when={isExpanded()}>
        <div>
          <For each={children()}>
            {(entry) => (
              <VfsTreeNode
                entry={entry}
                containerPath={props.containerPath}
                depth={1}
                partitionIndex={props.index}
                isExpanded={props.isExpanded}
                isLoading={props.isLoading}
                isSelected={props.isSelected}
                getChildren={props.getChildren}
                onToggle={props.onToggle}
                onClick={props.onEntryClick}
                isNestedExpanded={props.isNestedExpanded}
                isNestedLoading={props.isNestedLoading}
                getNestedEntries={props.getNestedEntries}
                getNestedChildren={props.getNestedChildren}
                onToggleNested={props.onToggleNested}
                onNestedClick={props.onNestedClick}
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
}
