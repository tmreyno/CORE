// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Ad1TreeNode - Tree node components for AD1 containers
 * 
 * Renders AD1 container entries with expand/collapse, selection, and metadata display.
 */

import { For, Show, JSX } from "solid-js";
import { HiOutlineMapPin } from "../../icons";
import { TreeRow } from "../../tree";
import type { TreeEntry } from "../../../types";

export interface Ad1EntryRowProps {
  entry: TreeEntry;
  containerPath: string;
  depth: number;
  isExpanded: boolean;
  isLoading: boolean;
  isSelected: boolean;
  onToggle: () => void;
  onClick: () => void;
}

/**
 * AD1 Entry Row - displays a single AD1 file/folder entry
 */
export function Ad1EntryRow(props: Ad1EntryRowProps): JSX.Element {
  const hasChildren = () => props.entry.is_dir && (props.entry.first_child_addr ?? 0) > 0;
  
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
      hasChildren={hasChildren()}
      onClick={props.onClick}
      onToggle={props.onToggle}
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
}

/**
 * AD1 Tree Node - recursive tree node for AD1 containers
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

  return (
    <>
      <Ad1EntryRow
        entry={props.entry}
        containerPath={props.containerPath}
        depth={props.depth}
        isExpanded={isExpanded()}
        isLoading={isLoading()}
        isSelected={isSelected()}
        onToggle={() => props.onToggle(props.containerPath, props.entry)}
        onClick={() => props.onClick(props.containerPath, props.entry)}
      />
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
            />
          )}
        </For>
      </Show>
    </>
  );
}
