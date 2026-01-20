// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ArchiveTreeNode - Tree node components for archive containers (ZIP, 7Z, etc.)
 * 
 * Renders archive file entries with expand/collapse for nested content.
 * Supports inline expansion of nested containers (AD1, ZIP, etc. inside archives).
 */

import { For, Show, JSX, createMemo, createSignal } from "solid-js";
import { TreeRow, TreeEmptyState } from "../../tree";
import type { ArchiveTreeEntry, NestedContainerEntry } from "../../../types";
import { isNestedContainerFile, getNestedContainerType } from "../containerDetection";

export interface ArchiveEntryRowProps {
  entry: ArchiveTreeEntry;
  containerPath: string;
  depth: number;
  isSelected: boolean;
  isExpanded: boolean;
  isLoading: boolean;
  hasChildren: boolean;
  isNestedContainer?: boolean;
  onClick: () => void;
  onToggle: () => void;
  onDblClick?: () => void;
}

/**
 * Archive Entry Row - displays a single archive file/folder entry
 * 
 * Extracts filename from entry.name or falls back to path extraction
 */
export function ArchiveEntryRow(props: ArchiveEntryRowProps): JSX.Element {
  // Extract just the filename from the path
  const fileName = () => {
    if (props.entry.name) return props.entry.name;
    const parts = props.entry.path.split('/').filter(p => p);
    return parts[parts.length - 1] || props.entry.path;
  };

  return (
    <TreeRow
      name={fileName()}
      path={props.entry.path}
      isDir={props.entry.isDir}
      size={props.entry.size || 0}
      depth={props.depth}
      isSelected={props.isSelected}
      isExpanded={props.isExpanded}
      isLoading={props.isLoading}
      hasChildren={props.hasChildren}
      onClick={props.onClick}
      onToggle={props.onToggle}
      onDblClick={props.onDblClick}
      entryType={props.isNestedContainer ? "container" : undefined}
      data-entry-path={props.entry.path}
    />
  );
}

export interface ArchiveTreeNodeProps {
  entry: ArchiveTreeEntry;
  containerPath: string;
  depth: number;
  // State accessors
  isExpanded: (nodeKey: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (containerPath: string, archivePath: string) => ArchiveTreeEntry[];
  // Event handlers
  onToggle: (containerPath: string, archivePath: string) => Promise<void>;
  onClick: (containerPath: string, entry: ArchiveTreeEntry) => void;
  // Nested container support (optional)
  isNestedExpanded?: (parentPath: string, nestedPath: string) => boolean;
  isNestedLoading?: (parentPath: string, nestedPath: string) => boolean;
  getNestedEntries?: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onToggleNested?: (parentPath: string, nestedPath: string) => Promise<void>;
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

/**
 * Archive Tree Node - recursive tree node for archive entries
 * Supports inline expansion of nested containers (AD1, ZIP, etc.)
 */
export function ArchiveTreeNode(props: ArchiveTreeNodeProps): JSX.Element {
  const nodeKey = () => `${props.containerPath}::archive::${props.entry.path}`;
  const isExpanded = () => props.isExpanded(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  const isSelected = () => props.isSelected(nodeKey());
  const children = () => props.getChildren(props.containerPath, props.entry.path);

  // Check if this entry is a nested container file
  const isNestedContainer = createMemo(() => {
    if (props.entry.isDir) return false;
    const name = props.entry.name || props.entry.path.split('/').pop() || '';
    return isNestedContainerFile(name);
  });
  
  const nestedContainerType = createMemo(() => {
    if (!isNestedContainer()) return null;
    const name = props.entry.name || props.entry.path.split('/').pop() || '';
    return getNestedContainerType(name);
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
    // Filter to only root-level entries
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

  // Determine if entry has children (directory, nested archive, or nested container with data)
  const hasChildren = () => {
    if (props.entry.isDir) return children().length > 0;
    if (isNestedContainer()) return true; // Nested containers are always expandable
    return false;
  };
  
  // Handle double-click to toggle nested container
  const handleDoubleClick = () => {
    if (isNestedContainer() && props.onToggleNested) {
      props.onToggleNested(props.containerPath, props.entry.path);
    }
  };

  return (
    <>
      <ArchiveEntryRow
        entry={props.entry}
        containerPath={props.containerPath}
        depth={props.depth}
        isSelected={isSelected()}
        isExpanded={isExpanded() || isNestedExp()}
        isLoading={isLoading() || isNestedLoad()}
        hasChildren={hasChildren()}
        isNestedContainer={isNestedContainer()}
        onClick={() => props.onClick(props.containerPath, props.entry)}
        onToggle={() => {
          if (isNestedContainer() && props.onToggleNested) {
            props.onToggleNested(props.containerPath, props.entry.path);
          } else {
            props.onToggle(props.containerPath, props.entry.path);
          }
        }}
        onDblClick={isNestedContainer() ? handleDoubleClick : undefined}
      />
      
      {/* Regular directory children */}
      <Show when={isExpanded() && props.entry.isDir}>
        <For each={children()}>
          {(child) => (
            <ArchiveTreeNode
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
        <Show when={children().length === 0 && !isLoading()}>
          <TreeEmptyState message="Empty folder" depth={props.depth + 1} />
        </Show>
      </Show>
      
      {/* Nested container contents (when expanded) */}
      <Show when={isNestedExp() && isNestedContainer()}>
        <For each={nestedRootEntries()}>
          {(nestedEntry) => (
            <NestedContainerEntryNode
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
 * Nested Container Entry Node - renders entries from inside a nested container
 */
interface NestedContainerEntryNodeProps {
  entry: NestedContainerEntry;
  parentContainerPath: string;
  nestedContainerPath: string;
  depth: number;
  isSelected: (key: string) => boolean;
  isLoading: (key: string) => boolean;
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

function NestedContainerEntryNode(props: NestedContainerEntryNodeProps): JSX.Element {
  const nodeKey = () => `${props.parentContainerPath}::nested::${props.nestedContainerPath}::${props.entry.path}`;
  const isSelected = () => props.isSelected(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  
  // Track local expansion for nested entry directories
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
      
      {/* Nested directory children */}
      <Show when={isExpanded() && props.entry.isDir}>
        <For each={sortedChildren()}>
          {(child) => (
            <NestedContainerEntryNode
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

export interface ArchiveRootListProps {
  entries: ArchiveTreeEntry[];
  containerPath: string;
  // State accessors
  isExpanded: (nodeKey: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (containerPath: string, archivePath: string) => ArchiveTreeEntry[];
  // Event handlers
  onToggle: (containerPath: string, archivePath: string) => Promise<void>;
  onClick: (containerPath: string, entry: ArchiveTreeEntry) => void;
  // Nested container support (optional)
  isNestedExpanded?: (parentPath: string, nestedPath: string) => boolean;
  isNestedLoading?: (parentPath: string, nestedPath: string) => boolean;
  getNestedEntries?: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  getNestedChildren?: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  onToggleNested?: (parentPath: string, nestedPath: string) => Promise<void>;
  onNestedClick?: (parentPath: string, nestedPath: string, entry: NestedContainerEntry) => void;
}

/**
 * Archive Root List - renders the top-level entries of an archive
 */
export function ArchiveRootList(props: ArchiveRootListProps): JSX.Element {
  return (
    <For each={props.entries} fallback={<TreeEmptyState message="Empty archive" depth={1} />}>
      {(entry) => (
        <ArchiveTreeNode
          entry={entry}
          containerPath={props.containerPath}
          depth={1}
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
  );
}
