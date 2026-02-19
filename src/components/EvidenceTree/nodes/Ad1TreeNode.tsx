// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Ad1TreeNode - Tree node components for AD1 containers
 * 
 * Renders AD1 container entries with expand/collapse, selection, and metadata display.
 * Supports inline expansion of nested containers (ZIP, E01, etc. inside AD1).
 */

import { For, Show, JSX, createMemo, createSignal } from "solid-js";
import { HiOutlineMapPin } from "../../icons";
import { TreeRow, TreeEmptyState } from "../../tree";
import type { TreeEntry, NestedContainerEntry } from "../../../types";
import { isNestedContainerFile, getNestedContainerType } from "../containerDetection";

export interface Ad1EntryRowProps {
  entry: TreeEntry;
  containerPath: string;
  depth: number;
  isExpanded: boolean;
  isLoading: boolean;
  isSelected: boolean;
  hasChildren: boolean;
  isNestedContainer?: boolean;
  onToggle: () => void;
  onClick: () => void;
}

/**
 * AD1 Entry Row - displays a single AD1 file/folder entry
 */
export function Ad1EntryRow(props: Ad1EntryRowProps): JSX.Element {
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
      hasChildren={props.hasChildren}
      onClick={props.onClick}
      onToggle={props.onToggle}
      entryType={props.isNestedContainer ? "container" : undefined}
      hash={props.entry.md5_hash || props.entry.sha1_hash}
      data-entry-path={props.entry.path}
      data-entry-addr={props.entry.item_addr || undefined}
      badge={
        <Show when={props.isSelected && props.entry.item_addr != null}>
          <span class="text-xs" title={`Item at 0x${props.entry.item_addr!.toString(16).toUpperCase()}`}>
            <HiOutlineMapPin class="w-3 h-3 text-accent" />
          </span>
        </Show>
      }
    />
  );
}

export interface Ad1TreeNodeProps {
  entry: TreeEntry;
  containerPath: string;
  depth: number;
  // State accessors
  isExpanded: (containerPath: string, entry: TreeEntry) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (containerPath: string, entry: TreeEntry) => TreeEntry[];
  // Event handlers
  onToggle: (containerPath: string, entry: TreeEntry) => void;
  onClick: (containerPath: string, entry: TreeEntry) => void;
  // Nested container support (optional)
  isNestedExpanded?: (parentPath: string, nestedPath: string) => boolean;
  isNestedLoading?: (parentPath: string, nestedPath: string) => boolean;
  getNestedEntries?: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onToggleNested?: (parentPath: string, nestedPath: string) => Promise<void>;
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

/**
 * AD1 Tree Node - recursive tree node for AD1 containers
 * Supports inline expansion of nested containers (ZIP, E01, etc.)
 */
export function Ad1TreeNode(props: Ad1TreeNodeProps): JSX.Element {
  const nodeKey = () => {
    const addr = props.entry.item_addr;
    return addr 
      ? `${props.containerPath}::addr:${addr}` 
      : `${props.containerPath}::path:${props.entry.path}`;
  };
  
  const entryKey = () => `${props.containerPath}::${props.entry.item_addr ?? props.entry.path}`;
  const isExpanded = () => props.isExpanded(props.containerPath, props.entry);
  const isLoading = () => props.isLoading(nodeKey());
  const isSelected = () => props.isSelected(entryKey());
  const children = () => props.getChildren(props.containerPath, props.entry);

  // Check if this entry is a nested container file
  const isNestedContainer = createMemo(() => {
    if (props.entry.is_dir) return false;
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

  // Determine if entry has children
  const hasChildren = () => {
    if (props.entry.is_dir) return (props.entry.first_child_addr ?? 0) > 0;
    if (isNestedContainer()) return true;
    return false;
  };

  return (
    <>
      <Ad1EntryRow
        entry={props.entry}
        containerPath={props.containerPath}
        depth={props.depth}
        isExpanded={isExpanded() || isNestedExp()}
        isLoading={isLoading() || isNestedLoad()}
        isSelected={isSelected()}
        hasChildren={hasChildren()}
        isNestedContainer={isNestedContainer()}
        onToggle={() => {
          if (isNestedContainer() && props.onToggleNested) {
            props.onToggleNested(props.containerPath, props.entry.path);
          } else {
            props.onToggle(props.containerPath, props.entry);
          }
        }}
        onClick={() => props.onClick(props.containerPath, props.entry)}
      />
      
      {/* Regular directory children */}
      <Show when={isExpanded() && props.entry.is_dir}>
        <For each={children()}>
          {(child) => (
            <Ad1TreeNode
              entry={child}
              containerPath={props.containerPath}
              depth={props.depth + 1}
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
            <Ad1NestedEntryNode
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
 * Nested Container Entry Node for AD1 trees
 * Renders entries from inside a nested container (e.g., files inside ZIP that's inside AD1)
 */
interface Ad1NestedEntryNodeProps {
  entry: NestedContainerEntry;
  parentContainerPath: string;
  nestedContainerPath: string;
  depth: number;
  isSelected: (key: string) => boolean;
  isLoading: (key: string) => boolean;
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

function Ad1NestedEntryNode(props: Ad1NestedEntryNodeProps): JSX.Element {
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
            <Ad1NestedEntryNode
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
