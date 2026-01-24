// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ArchiveEntryRow - Renders an archive tree entry using standardized TreeRow
 */

import { TreeRow } from "../../tree";
import { getBasename } from "../../../utils";
import type { ArchiveTreeEntry } from "../../../types";
import type { SelectedEntry } from "../types";
import { isContainerFile } from "../containerDetection";

export interface ArchiveEntryRowProps {
  entry: ArchiveTreeEntry;
  containerPath: string;
  depth: number;
  isExpanded: boolean;
  isLoading: boolean;
  isSelected: boolean;
  hasChildren: boolean;
  onEntryClick: (containerPath: string, entry: ArchiveTreeEntry) => void;
  onToggle: (containerPath: string, archivePath: string) => void;
  onDoubleClick?: (containerPath: string, entry: ArchiveTreeEntry) => void;
}

/**
 * Archive entry row component
 */
export function ArchiveEntryRow(props: ArchiveEntryRowProps) {
  // Check if this entry is a nested container
  const isNestedContainer = () => !props.entry.isDir && isContainerFile(props.entry.name || props.entry.path);
  
  // Extract filename from path
  const fileName = () => {
    if (props.entry.name) return props.entry.name;
    const parts = props.entry.path.split('/').filter(p => p);
    return parts[parts.length - 1] || props.entry.path;
  };
  
  const handleClick = () => {
    props.onEntryClick(props.containerPath, props.entry);
  };
  
  const handleDoubleClick = () => {
    if (isNestedContainer() && props.onDoubleClick) {
      props.onDoubleClick(props.containerPath, props.entry);
    }
  };
  
  return (
    <TreeRow
      name={fileName()}
      path={props.entry.path}
      isDir={props.entry.isDir}
      size={props.entry.size}
      depth={props.depth}
      isSelected={props.isSelected}
      isExpanded={props.isExpanded}
      isLoading={props.isLoading}
      hasChildren={props.hasChildren}
      onClick={handleClick}
      onDblClick={isNestedContainer() ? handleDoubleClick : undefined}
      onToggle={() => props.onToggle(props.containerPath, props.entry.path)}
      entryType={isNestedContainer() ? "container" : undefined}
      data-entry-path={props.entry.path}
    />
  );
}

/**
 * Creates a SelectedEntry from an archive entry
 */
export function createArchiveSelectedEntry(containerPath: string, entry: ArchiveTreeEntry): SelectedEntry {
  return {
    containerPath,
    entryPath: entry.path,
    name: entry.name || getBasename(entry.path) || entry.path,
    size: entry.size,
    isDir: entry.isDir,
    isVfsEntry: false,
  };
}
