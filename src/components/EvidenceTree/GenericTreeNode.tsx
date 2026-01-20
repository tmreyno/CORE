// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * GenericTreeNode - Unified tree node component for all container types
 * 
 * This component provides a consistent rendering pattern for tree nodes
 * regardless of container type (AD1, E01, Archive, UFED, etc.)
 * 
 * Each container type provides an adapter that translates its data model
 * to the generic interface expected by this component.
 */

import { For, Show, createMemo, type JSX } from "solid-js";
import {
  TreeRow,
  TREE_ROW_BASE_CLASSES,
  TREE_ROW_NORMAL_CLASSES,
  getTreeIndent,
} from "../tree";
import {
  HiOutlineChevronDown,
  HiOutlineArrowPath,
} from "../icons";
import type { TreeNodeAdapter } from "./types";

export interface GenericTreeNodeProps<T> {
  entry: T;
  containerPath: string;
  depth: number;
  adapter: TreeNodeAdapter<T>;
  
  // State callbacks
  isExpanded: (key: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (entry: T) => T[];
  
  // Event handlers
  onSelect: (entry: T) => void;
  onToggle: (entry: T) => void;
  onDoubleClick?: (entry: T) => void;
  
  // Pagination (optional)
  hasMore?: (key: string) => boolean;
  totalCount?: (key: string) => number;
  isLoadingMore?: (key: string) => boolean;
  onLoadMore?: (entry: T) => void;
}

export function GenericTreeNode<T>(props: GenericTreeNodeProps<T>): JSX.Element {
  const adapter = props.adapter;
  
  // Compute key once
  const entryKey = () => adapter.getKey(props.entry, props.containerPath);
  
  // Reactive state checks
  const isExpanded = () => props.isExpanded(entryKey());
  const isLoading = () => props.isLoading(entryKey());
  const isSelected = () => props.isSelected(entryKey());
  
  // Memoized children (avoids re-sorting on every render)
  const children = createMemo(() => props.getChildren(props.entry));
  
  // Check if this is a directory with potential children
  const isDir = () => adapter.isDir(props.entry);
  const hasChildren = () => adapter.hasChildren(props.entry);
  
  // Handle click - select the entry
  const handleClick = () => {
    props.onSelect(props.entry);
  };
  
  // Handle toggle - expand/collapse for directories
  const handleToggle = () => {
    if (isDir()) {
      props.onToggle(props.entry);
    }
  };
  
  // Handle double click (e.g., for nested containers)
  const handleDoubleClick = () => {
    if (props.onDoubleClick) {
      props.onDoubleClick(props.entry);
    }
  };
  
  // Handle load more for pagination
  const handleLoadMore = (e: MouseEvent) => {
    e.stopPropagation();
    if (props.onLoadMore) {
      props.onLoadMore(props.entry);
    }
  };
  
  // Pagination state
  const hasMore = () => props.hasMore?.(entryKey()) ?? false;
  const totalCount = () => props.totalCount?.(entryKey()) ?? 0;
  const isLoadingMore = () => props.isLoadingMore?.(entryKey()) ?? false;
  
  return (
    <>
      <TreeRow
        name={adapter.getName(props.entry)}
        path={adapter.getPath(props.entry)}
        isDir={isDir()}
        size={adapter.getSize(props.entry) ?? 0}
        depth={props.depth}
        isSelected={isSelected()}
        isExpanded={isExpanded()}
        isLoading={isLoading()}
        hasChildren={hasChildren()}
        onClick={handleClick}
        onToggle={handleToggle}
        onDblClick={props.onDoubleClick ? handleDoubleClick : undefined}
        entryType={adapter.getEntryType?.(props.entry)}
        hash={adapter.getHash?.(props.entry)}
        badge={adapter.getBadge?.(props.entry, isSelected())}
        data-entry-path={adapter.getPath(props.entry)}
      />
      
      {/* Expanded children */}
      <Show when={isExpanded() && isDir()}>
        <For each={children()}>
          {(child) => (
            <GenericTreeNode
              entry={child}
              containerPath={props.containerPath}
              depth={props.depth + 1}
              adapter={adapter}
              isExpanded={props.isExpanded}
              isLoading={props.isLoading}
              isSelected={props.isSelected}
              getChildren={props.getChildren}
              onSelect={props.onSelect}
              onToggle={props.onToggle}
              onDoubleClick={props.onDoubleClick}
              hasMore={props.hasMore}
              totalCount={props.totalCount}
              isLoadingMore={props.isLoadingMore}
              onLoadMore={props.onLoadMore}
            />
          )}
        </For>
        
        {/* Load more button for pagination */}
        <Show when={hasMore()}>
          <div 
            class={`${TREE_ROW_BASE_CLASSES} ${TREE_ROW_NORMAL_CLASSES} cursor-pointer hover:bg-bg-hover/30`}
            style={{ "padding-left": getTreeIndent(props.depth + 1) }}
            onClick={handleLoadMore}
          >
            <Show 
              when={!isLoadingMore()} 
              fallback={
                <span class="flex items-center gap-2 text-xs text-txt-secondary">
                  <HiOutlineArrowPath class="w-3.5 h-3.5 animate-spin" />
                  Loading...
                </span>
              }
            >
              <span class="flex items-center gap-2 text-xs text-accent hover:text-accent-hover">
                <HiOutlineChevronDown class="w-3.5 h-3.5" />
                Load more ({children().length.toLocaleString()} of {totalCount().toLocaleString()})
              </span>
            </Show>
          </div>
        </Show>
      </Show>
    </>
  );
}

export default GenericTreeNode;
