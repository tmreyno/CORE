// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Ad1EntryRow - Renders a single AD1 tree entry using standardized TreeRow
 */

import { Show } from "solid-js";
import { TreeRow } from "../../tree";
import { HiOutlineMapPin } from "../../icons";
import type { TreeEntry } from "../../../types";
import type { SelectedEntry } from "../types";

export interface Ad1EntryRowProps {
  entry: TreeEntry;
  containerPath: string;
  depth: number;
  isExpanded: boolean;
  isLoading: boolean;
  isSelected: boolean;
  onEntryClick: (containerPath: string, entry: TreeEntry) => void;
  onToggle: (containerPath: string, entry: TreeEntry) => void;
}

/**
 * AD1 entry row component
 */
export function Ad1EntryRow(props: Ad1EntryRowProps) {
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
      onClick={() => props.onEntryClick(props.containerPath, props.entry)}
      onToggle={() => props.onToggle(props.containerPath, props.entry)}
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

/**
 * Creates a SelectedEntry from an AD1 TreeEntry
 */
export function createAd1SelectedEntry(containerPath: string, entry: TreeEntry): SelectedEntry {
  return {
    containerPath,
    entryPath: entry.path,
    name: entry.name,
    size: entry.size,
    isDir: entry.is_dir,
    isVfsEntry: false,
    dataAddr: entry.data_addr,
    itemAddr: entry.item_addr,
    compressedSize: entry.compressed_size,
    dataEndAddr: entry.data_end_addr,
    metadataAddr: entry.metadata_addr,
    firstChildAddr: entry.first_child_addr,
  };
}
