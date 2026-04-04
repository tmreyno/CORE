// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LazyTreeNode - Tree node components for lazy-loaded containers
 *
 * Renders entries with on-demand loading for large evidence sets.
 */

import { For, Show, JSX, createMemo, createSignal } from "solid-js";
import { TreeRow, TreeEmptyState } from "../../tree";
import type { LazyTreeEntry } from "../../../types/lazy-loading";
import type { NestedContainerEntry, UfedTreeEntry } from "../../../types";
import { getNestedContainerType, isNestedContainerFile } from "../containerDetection";

export interface UfedEntryRowProps {
  entry: UfedTreeEntry;
  containerPath: string;
  depth: number;
  isSelected: boolean;
  onClick: () => void;
}

/**
 * UFED Entry Row - displays a single UFED mobile extraction entry
 */
export function UfedEntryRow(props: UfedEntryRowProps): JSX.Element {
  return (
    <TreeRow
      name={props.entry.name}
      path={props.entry.path}
      isDir={props.entry.isDir}
      size={props.entry.size || 0}
      depth={props.depth}
      isSelected={props.isSelected}
      isExpanded={false}
      isLoading={false}
      hasChildren={false}
      onClick={props.onClick}
      entryType={props.entry.entryType}
      hash={props.entry.hash}
      data-entry-path={props.entry.path}
    />
  );
}

export interface LazyEntryRowProps {
  entry: LazyTreeEntry;
  containerPath: string;
  depth: number;
  isSelected: boolean;
  isExpanded: boolean;
  isLoading: boolean;
  hasChildren: boolean;
  isNestedContainer?: boolean;
  hasMoreChildren?: boolean;
  loadedChildren?: number;
  totalChildren?: number;
  onClick: () => void;
  onToggle: () => void;
  onLoadMore?: () => void;
}

/**
 * Lazy Entry Row - displays a single lazy-loaded file/folder entry
 *
 * Shows entry with expand/collapse, loading indicators
 */
export function LazyEntryRow(props: LazyEntryRowProps): JSX.Element {
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
      data-entry-path={props.entry.path}
    />
  );
}

export interface LazyTreeNodeProps {
  entry: LazyTreeEntry;
  containerPath: string;
  depth: number;
  isExpanded: (containerPath: string, entryPath: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (containerPath: string, parentPath: string) => LazyTreeEntry[];
  hasMoreChildren: (containerPath: string, parentPath: string) => boolean;
  getLoadedCount: (containerPath: string, parentPath: string) => number;
  getTotalCount: (containerPath: string, parentPath: string) => number;
  onToggle: (containerPath: string, entryPath: string) => void;
  onClick: (containerPath: string, entry: LazyTreeEntry) => void;
  onLoadMore?: (containerPath: string, parentPath: string) => void;
  isNestedExpanded?: (parentPath: string, nestedPath: string) => boolean;
  isNestedLoading?: (parentPath: string, nestedPath: string) => boolean;
  getNestedEntries?: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onToggleNested?: (parentPath: string, nestedPath: string) => Promise<void>;
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

/**
 * Lazy Tree Node - recursive tree node with lazy loading support
 */
export function LazyTreeNode(props: LazyTreeNodeProps): JSX.Element {
  const nodeKey = () => `${props.containerPath}::lazy::${props.entry.path}`;
  const isExpanded = () => props.isExpanded(props.containerPath, props.entry.path);
  const isLoading = () => props.isLoading(nodeKey());
  const isSelected = () => props.isSelected(nodeKey());
  const children = () => props.getChildren(props.containerPath, props.entry.path);
  const hasMore = () => props.hasMoreChildren(props.containerPath, props.entry.path);
  const loadedCount = () => props.getLoadedCount(props.containerPath, props.entry.path);
  const totalCount = () => props.getTotalCount(props.containerPath, props.entry.path);
  const isNestedContainer = createMemo(() => {
    if (props.entry.is_dir) return false;
    return isNestedContainerFile(props.entry.name);
  });
  const nestedContainerType = createMemo(() => {
    if (!isNestedContainer()) return null;
    return getNestedContainerType(props.entry.name);
  });
  const isNestedExp = () => {
    if (!isNestedContainer() || !props.isNestedExpanded) return false;
    return props.isNestedExpanded(props.containerPath, props.entry.path);
  };
  const isNestedLoad = () => {
    if (!isNestedContainer() || !props.isNestedLoading) return false;
    return props.isNestedLoading(props.containerPath, props.entry.path);
  };
  const nestedRootEntries = createMemo(() => {
    if (!isNestedContainer() || !props.getNestedEntries) return [];
    return [...props.getNestedEntries(props.containerPath, props.entry.path)].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  });
  // child_count: -1 = unknown (assume has children), 0 = empty, >0 = known count
  const hasChildren = () => {
    if (props.entry.is_dir) {
      return (props.entry.child_count ?? -1) !== 0;
    }
    return isNestedContainer();
  };

  return (
    <>
      <LazyEntryRow
        entry={props.entry}
        containerPath={props.containerPath}
        depth={props.depth}
        isSelected={isSelected()}
        isExpanded={isExpanded() || isNestedExp()}
        isLoading={isLoading() || isNestedLoad()}
        hasChildren={hasChildren()}
        isNestedContainer={isNestedContainer()}
        hasMoreChildren={props.entry.is_dir ? hasMore() : undefined}
        loadedChildren={props.entry.is_dir ? loadedCount() : undefined}
        totalChildren={props.entry.is_dir ? totalCount() : undefined}
        onClick={() => props.onClick(props.containerPath, props.entry)}
        onToggle={() => {
          if (isNestedContainer() && props.onToggleNested) {
            props.onToggleNested(props.containerPath, props.entry.path);
          } else {
            props.onToggle(props.containerPath, props.entry.path);
          }
        }}
        onLoadMore={props.onLoadMore ? () => props.onLoadMore!(props.containerPath, props.entry.path) : undefined}
      />
      <Show when={isExpanded() && props.entry.is_dir}>
        <For each={children()}>
          {(child) => (
            <LazyTreeNode
              entry={child}
              containerPath={props.containerPath}
              depth={props.depth + 1}
              isExpanded={props.isExpanded}
              isLoading={props.isLoading}
              isSelected={props.isSelected}
              getChildren={props.getChildren}
              hasMoreChildren={props.hasMoreChildren}
              getLoadedCount={props.getLoadedCount}
              getTotalCount={props.getTotalCount}
              onToggle={props.onToggle}
              onClick={props.onClick}
              onLoadMore={props.onLoadMore}
              isNestedExpanded={props.isNestedExpanded}
              isNestedLoading={props.isNestedLoading}
              getNestedEntries={props.getNestedEntries}
              getNestedChildren={props.getNestedChildren}
              onToggleNested={props.onToggleNested}
              onNestedClick={props.onNestedClick}
            />
          )}
        </For>
        <Show when={hasMore() && !isLoading()}>
          <LoadMoreButton
            depth={props.depth + 1}
            loadedCount={loadedCount()}
            totalCount={totalCount()}
            onClick={() => props.onLoadMore?.(props.containerPath, props.entry.path)}
          />
        </Show>
        <Show when={children().length === 0 && !isLoading()}>
          <TreeEmptyState message="Empty folder" depth={props.depth + 1} />
        </Show>
      </Show>
      <Show when={isNestedExp() && isNestedContainer()}>
        <For each={nestedRootEntries()}>
          {(nestedEntry) => (
            <LazyNestedEntryNode
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
          <TreeEmptyState message={`Empty ${nestedContainerType() || "container"}`} depth={props.depth + 1} />
        </Show>
      </Show>
    </>
  );
}

interface LazyNestedEntryNodeProps {
  entry: NestedContainerEntry;
  parentContainerPath: string;
  nestedContainerPath: string;
  depth: number;
  isSelected: (key: string) => boolean;
  isLoading: (key: string) => boolean;
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

function LazyNestedEntryNode(props: LazyNestedEntryNodeProps): JSX.Element {
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
            <LazyNestedEntryNode
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

export interface LoadMoreButtonProps {
  depth: number;
  loadedCount: number;
  totalCount: number;
  onClick: () => void;
}

/**
 * Load More Button - pagination button for lazy-loaded directories
 */
export function LoadMoreButton(props: LoadMoreButtonProps): JSX.Element {
  const remaining = () => props.totalCount - props.loadedCount;
  const paddingLeft = () => ((props.depth + 1) * 16 + 8) + "px";

  return (
    <button
      class="flex items-center gap-2 w-full py-1 text-xs text-blue-400 hover:text-blue-300 hover:bg-blue-500/10 transition-colors"
      style={{ "padding-left": paddingLeft() }}
      onClick={(e) => {
        e.stopPropagation();
        props.onClick();
      }}
    >
      <span>Load more ({remaining()} remaining)</span>
    </button>
  );
}

export interface LazyRootListProps {
  entries: LazyTreeEntry[];
  containerPath: string;
  isExpanded: (containerPath: string, entryPath: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (containerPath: string, parentPath: string) => LazyTreeEntry[];
  hasMoreChildren: (containerPath: string, parentPath: string) => boolean;
  getLoadedCount: (containerPath: string, parentPath: string) => number;
  getTotalCount: (containerPath: string, parentPath: string) => number;
  onToggle: (containerPath: string, entryPath: string) => void;
  onClick: (containerPath: string, entry: LazyTreeEntry) => void;
  onLoadMore?: (containerPath: string, parentPath: string) => void;
}

/**
 * Lazy Root List - renders the top-level entries of a lazy-loaded container
 */
export function LazyRootList(props: LazyRootListProps): JSX.Element {
  return (
    <For each={props.entries} fallback={<TreeEmptyState message="Empty container" depth={1} />}>
      {(entry) => (
        <LazyTreeNode
          entry={entry}
          containerPath={props.containerPath}
          depth={1}
          isExpanded={props.isExpanded}
          isLoading={props.isLoading}
          isSelected={props.isSelected}
          getChildren={props.getChildren}
          hasMoreChildren={props.hasMoreChildren}
          getLoadedCount={props.getLoadedCount}
          getTotalCount={props.getTotalCount}
          onToggle={props.onToggle}
          onClick={props.onClick}
          onLoadMore={props.onLoadMore}
        />
      )}
    </For>
  );
}
