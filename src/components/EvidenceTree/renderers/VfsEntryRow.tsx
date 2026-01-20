// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * VfsEntryRow - Renders a VFS (disk image) tree entry using standardized TreeRow
 */

import { TreeRow } from "../../tree";
import type { VfsEntry } from "../../../types";
import type { SelectedEntry } from "../types";

export interface VfsEntryRowProps {
  entry: VfsEntry;
  containerPath: string;
  depth: number;
  isExpanded: boolean;
  isLoading: boolean;
  isSelected: boolean;
  onEntryClick: (containerPath: string, entry: VfsEntry) => void;
  onToggle: (containerPath: string, vfsPath: string) => void;
}

/**
 * VFS entry row component
 */
export function VfsEntryRow(props: VfsEntryRowProps) {
  const handleClick = () => {
    props.onEntryClick(props.containerPath, props.entry);
  };
  
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
      hasChildren={props.entry.isDir}
      onClick={handleClick}
      onToggle={() => props.onToggle(props.containerPath, props.entry.path)}
      data-entry-path={props.entry.path}
    />
  );
}

/**
 * Creates a SelectedEntry from a VFS entry
 */
export function createVfsSelectedEntry(containerPath: string, entry: VfsEntry): SelectedEntry {
  return {
    containerPath,
    entryPath: entry.path,
    name: entry.name,
    size: entry.size,
    isDir: entry.isDir,
    isVfsEntry: true,
    // VFS entries don't have AD1-style addresses
    dataAddr: null,
    itemAddr: null,
    compressedSize: null,
    dataEndAddr: null,
    metadataAddr: null,
    firstChildAddr: null,
  };
}
