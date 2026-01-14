// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * VfsTreeNode - Tree node components for VFS containers (E01, Raw, L01)
 * 
 * Renders VFS partition and filesystem entries with expand/collapse.
 */

import { For, Show, JSX } from "solid-js";
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
import type { VfsEntry, VfsPartitionInfo } from "../../../types";

export interface VfsEntryRowProps {
  entry: VfsEntry;
  containerPath: string;
  depth: number;
  isExpanded: boolean;
  isLoading: boolean;
  isSelected: boolean;
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
      isDir={props.entry.is_dir}
      size={props.entry.size}
      depth={props.depth}
      isSelected={props.isSelected}
      isExpanded={props.isExpanded}
      isLoading={props.isLoading}
      hasChildren={props.entry.is_dir}
      onClick={props.onClick}
      onToggle={props.onToggle}
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
}

/**
 * VFS Tree Node - recursive tree node for VFS filesystem entries
 */
export function VfsTreeNode(props: VfsTreeNodeProps): JSX.Element {
  const nodeKey = () => `${props.containerPath}::vfs::${props.entry.path}`;
  const isExpanded = () => props.isExpanded(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  const isSelected = () => props.isSelected(nodeKey());
  const children = () => props.getChildren(props.containerPath, props.entry.path);

  return (
    <>
      <VfsEntryRow
        entry={props.entry}
        containerPath={props.containerPath}
        depth={props.depth}
        isExpanded={isExpanded()}
        isLoading={isLoading()}
        isSelected={isSelected()}
        partitionIndex={props.partitionIndex}
        onToggle={() => props.onToggle(props.containerPath, props.entry.path)}
        onClick={() => props.onClick(props.containerPath, props.entry, props.partitionIndex)}
      />
      <Show when={isExpanded() && props.entry.is_dir}>
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
            />
          )}
        </For>
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
}

/**
 * Partition Node - displays a partition with its filesystem tree
 */
export function PartitionNode(props: PartitionNodeProps): JSX.Element {
  const partitionRootPath = () => `/${props.partition.mount_name}`;
  const nodeKey = () => `${props.containerPath}::vfs::${partitionRootPath()}`;
  const isExpanded = () => props.isExpanded(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  const children = () => props.getChildren(props.containerPath, partitionRootPath());

  const togglePartition = () => props.onToggle(props.containerPath, partitionRootPath());

  // Get filesystem icon based on type
  const fsIcon = (): JSX.Element => {
    const fs = props.partition.fs_type.toLowerCase();
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
        <span class="text-sm text-zinc-300">{props.partition.mount_name}</span>
        <span class="text-xs text-zinc-500">
          {props.partition.fs_type} • {formatBytes(props.partition.size)}
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
