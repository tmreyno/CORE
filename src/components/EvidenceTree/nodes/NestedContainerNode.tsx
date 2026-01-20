// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * NestedContainerNode - Tree node component for nested containers
 * 
 * Renders entries inside nested containers (e.g., files inside AD1 that's inside ZIP).
 * Supports recursive nesting - a nested container can contain more nested containers.
 */

import { For, Show, JSX, createMemo } from "solid-js";
import { TreeRow, TreeEmptyState } from "../../tree";
import type { NestedContainerEntry } from "../../../types";

export interface NestedContainerEntryRowProps {
  entry: NestedContainerEntry;
  /** Full path chain: parentContainerPath::nestedContainerPath */
  containerChain: string;
  depth: number;
  isSelected: boolean;
  isExpanded: boolean;
  isLoading: boolean;
  hasChildren: boolean;
  onClick: () => void;
  onToggle: () => void;
  onDblClick?: () => void;
}

/**
 * Single row for a nested container entry
 */
export function NestedContainerEntryRow(props: NestedContainerEntryRowProps): JSX.Element {
  // Get entry type indicator
  const entryType = () => {
    if (props.entry.isNestedContainer) return "container";
    return undefined;
  };

  return (
    <TreeRow
      name={props.entry.name}
      path={props.entry.path}
      isDir={props.entry.isDir}
      size={props.entry.size || 0}
      depth={props.depth}
      isSelected={props.isSelected}
      isExpanded={props.isExpanded}
      isLoading={props.isLoading}
      hasChildren={props.hasChildren || props.entry.isNestedContainer}
      onClick={props.onClick}
      onToggle={props.onToggle}
      onDblClick={props.onDblClick}
      entryType={entryType()}
      hash={props.entry.hash || undefined}
      data-entry-path={props.entry.path}
      data-source-type={props.entry.sourceType}
    />
  );
}

export interface NestedContainerNodeProps {
  entry: NestedContainerEntry;
  /** Parent container path (outer container like ZIP) */
  parentContainerPath: string;
  /** Path to the nested container inside the parent */
  nestedContainerPath: string;
  depth: number;
  // State accessors
  isExpanded: (key: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  // For nested containers inside nested containers
  isNestedContainerExpanded: (parentPath: string, nestedPath: string) => boolean;
  isNestedContainerLoading: (parentPath: string, nestedPath: string) => boolean;
  getNestedContainerEntries: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  // Event handlers
  onToggle: (entryPath: string) => void;
  onClick: (entry: NestedContainerEntry) => void;
  onToggleNestedContainer: (nestedPath: string) => Promise<void>;
}

/**
 * Recursive tree node for nested container entries
 */
export function NestedContainerNode(props: NestedContainerNodeProps): JSX.Element {
  // Build unique key for this entry
  const nodeKey = () => `${props.parentContainerPath}::nested::${props.nestedContainerPath}::${props.entry.path}`;
  const isExpanded = () => props.isExpanded(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  const isSelected = () => props.isSelected(nodeKey());
  
  // Get children for directories
  const children = createMemo(() => {
    if (!props.entry.isDir) return [];
    return props.getChildren(props.parentContainerPath, props.nestedContainerPath, props.entry.path);
  });
  
  const hasChildren = () => {
    if (props.entry.isDir) return children().length > 0;
    if (props.entry.isNestedContainer) return true; // Nested containers always expandable
    return false;
  };
  
  // Handle click on entry
  const handleClick = () => {
    if (props.entry.isDir) {
      props.onToggle(props.entry.path);
    }
    props.onClick(props.entry);
  };
  
  // Handle double-click on nested containers
  const handleDoubleClick = () => {
    if (props.entry.isNestedContainer) {
      // This entry is itself a container - toggle its expansion
      const nestedPath = `${props.nestedContainerPath}/${props.entry.path}`.replace(/\/+/g, '/');
      props.onToggleNestedContainer(nestedPath);
    }
  };
  
  // Check if this entry has nested container data loaded
  const nestedContainerPath = () => `${props.nestedContainerPath}/${props.entry.path}`.replace(/\/+/g, '/');
  const isNestedExpanded = () => props.entry.isNestedContainer && props.isNestedContainerExpanded(props.parentContainerPath, nestedContainerPath());
  const isNestedLoading = () => props.entry.isNestedContainer && props.isNestedContainerLoading(props.parentContainerPath, nestedContainerPath());
  const nestedEntries = createMemo(() => {
    if (!props.entry.isNestedContainer) return [];
    return props.getNestedContainerEntries(props.parentContainerPath, nestedContainerPath());
  });
  
  // Sort entries
  const sortedChildren = createMemo(() => {
    return [...children()].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  });
  
  const sortedNestedEntries = createMemo(() => {
    const entries = nestedEntries();
    // Get root entries only
    const rootEntries = entries.filter(e => {
      const parts = e.path.replace(/\/$/, '').split('/').filter(p => p);
      return parts.length === 1;
    });
    return [...rootEntries].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  });

  return (
    <>
      <NestedContainerEntryRow
        entry={props.entry}
        containerChain={`${props.parentContainerPath}::${props.nestedContainerPath}`}
        depth={props.depth}
        isSelected={isSelected()}
        isExpanded={isExpanded() || isNestedExpanded()}
        isLoading={isLoading() || isNestedLoading()}
        hasChildren={hasChildren()}
        onClick={handleClick}
        onToggle={handleClick}
        onDblClick={props.entry.isNestedContainer ? handleDoubleClick : undefined}
      />
      
      {/* Regular directory children */}
      <Show when={isExpanded() && props.entry.isDir}>
        <For each={sortedChildren()}>
          {(child) => (
            <NestedContainerNode
              entry={child}
              parentContainerPath={props.parentContainerPath}
              nestedContainerPath={props.nestedContainerPath}
              depth={props.depth + 1}
              isExpanded={props.isExpanded}
              isLoading={props.isLoading}
              isSelected={props.isSelected}
              getChildren={props.getChildren}
              isNestedContainerExpanded={props.isNestedContainerExpanded}
              isNestedContainerLoading={props.isNestedContainerLoading}
              getNestedContainerEntries={props.getNestedContainerEntries}
              onToggle={props.onToggle}
              onClick={props.onClick}
              onToggleNestedContainer={props.onToggleNestedContainer}
            />
          )}
        </For>
        <Show when={sortedChildren().length === 0 && !isLoading()}>
          <TreeEmptyState message="Empty folder" depth={props.depth + 1} />
        </Show>
      </Show>
      
      {/* Nested container children (when expanded) */}
      <Show when={isNestedExpanded() && props.entry.isNestedContainer}>
        <For each={sortedNestedEntries()}>
          {(child) => (
            <NestedContainerNode
              entry={child}
              parentContainerPath={props.parentContainerPath}
              nestedContainerPath={nestedContainerPath()}
              depth={props.depth + 1}
              isExpanded={props.isExpanded}
              isLoading={props.isLoading}
              isSelected={props.isSelected}
              getChildren={props.getChildren}
              isNestedContainerExpanded={props.isNestedContainerExpanded}
              isNestedContainerLoading={props.isNestedContainerLoading}
              getNestedContainerEntries={props.getNestedContainerEntries}
              onToggle={props.onToggle}
              onClick={props.onClick}
              onToggleNestedContainer={props.onToggleNestedContainer}
            />
          )}
        </For>
        <Show when={sortedNestedEntries().length === 0 && !isNestedLoading()}>
          <TreeEmptyState message={`Empty ${props.entry.nestedType || 'container'}`} depth={props.depth + 1} />
        </Show>
      </Show>
    </>
  );
}

export interface NestedContainerRootListProps {
  entries: NestedContainerEntry[];
  parentContainerPath: string;
  nestedContainerPath: string;
  baseDepth: number;
  // State accessors
  isExpanded: (key: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (parentPath: string, nestedPath: string, entryPath: string) => NestedContainerEntry[];
  isNestedContainerExpanded: (parentPath: string, nestedPath: string) => boolean;
  isNestedContainerLoading: (parentPath: string, nestedPath: string) => boolean;
  getNestedContainerEntries: (parentPath: string, nestedPath: string) => NestedContainerEntry[];
  // Event handlers
  onToggle: (entryPath: string) => void;
  onClick: (entry: NestedContainerEntry) => void;
  onToggleNestedContainer: (nestedPath: string) => Promise<void>;
}

/**
 * Root list of nested container entries
 */
export function NestedContainerRootList(props: NestedContainerRootListProps): JSX.Element {
  // Sort entries: directories first, then by name
  const sortedEntries = createMemo(() => {
    return [...props.entries].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
  });

  return (
    <For each={sortedEntries()}>
      {(entry) => (
        <NestedContainerNode
          entry={entry}
          parentContainerPath={props.parentContainerPath}
          nestedContainerPath={props.nestedContainerPath}
          depth={props.baseDepth}
          isExpanded={props.isExpanded}
          isLoading={props.isLoading}
          isSelected={props.isSelected}
          getChildren={props.getChildren}
          isNestedContainerExpanded={props.isNestedContainerExpanded}
          isNestedContainerLoading={props.isNestedContainerLoading}
          getNestedContainerEntries={props.getNestedContainerEntries}
          onToggle={props.onToggle}
          onClick={props.onClick}
          onToggleNestedContainer={props.onToggleNestedContainer}
        />
      )}
    </For>
  );
}
