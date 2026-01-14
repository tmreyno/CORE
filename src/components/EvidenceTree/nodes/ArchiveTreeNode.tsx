// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ArchiveTreeNode - Tree node components for archive containers (ZIP, 7Z, etc.)
 * 
 * Renders archive file entries with expand/collapse for nested content.
 */

import { For, Show, JSX } from "solid-js";
import { TreeRow, TreeEmptyState } from "../../tree";
import type { ArchiveTreeEntry } from "../../../types";

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
      isDir={props.entry.is_dir}
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
}

/**
 * Archive Tree Node - recursive tree node for archive entries
 */
export function ArchiveTreeNode(props: ArchiveTreeNodeProps): JSX.Element {
  const nodeKey = () => `${props.containerPath}::archive::${props.entry.path}`;
  const isExpanded = () => props.isExpanded(nodeKey());
  const isLoading = () => props.isLoading(nodeKey());
  const isSelected = () => props.isSelected(nodeKey());
  const children = () => props.getChildren(props.containerPath, props.entry.path);

  // Determine if entry has children (directory or nested archive)
  const hasChildren = () => props.entry.is_dir || children().length > 0;

  return (
    <>
      <ArchiveEntryRow
        entry={props.entry}
        containerPath={props.containerPath}
        depth={props.depth}
        isSelected={isSelected()}
        isExpanded={isExpanded()}
        isLoading={isLoading()}
        hasChildren={hasChildren()}
        onClick={() => props.onClick(props.containerPath, props.entry)}
        onToggle={() => props.onToggle(props.containerPath, props.entry.path)}
      />
      <Show when={isExpanded() && (props.entry.is_dir || hasChildren())}>
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
            />
          )}
        </For>
        <Show when={children().length === 0 && !isLoading()}>
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
        />
      )}
    </For>
  );
}
