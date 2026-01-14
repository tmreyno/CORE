// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LazyEntryRow - Renders a lazy-loaded tree entry using standardized TreeRow
 * 
 * Used for large containers (UFED, large archives) with pagination support.
 */

import { TreeRow } from "../../tree";
import type { LazyTreeEntry } from "../../../types/lazy-loading";
import type { SelectedEntry } from "../types";

export interface LazyEntryRowProps {
  entry: LazyTreeEntry;
  containerPath: string;
  depth: number;
  isExpanded: boolean;
  isLoading: boolean;
  isSelected: boolean;
  onEntryClick: (containerPath: string, entry: LazyTreeEntry) => void;
  onToggle: (containerPath: string, entryPath: string) => void;
}

/**
 * Lazy-loaded entry row component
 */
export function LazyEntryRow(props: LazyEntryRowProps) {
  const hasChildren = () => props.entry.is_dir && (props.entry.child_count ?? 0) > 0;
  
  const handleClick = () => {
    props.onEntryClick(props.containerPath, props.entry);
  };
  
  return (
    <TreeRow
      name={props.entry.name}
      path={props.entry.path}
      isDir={props.entry.is_dir}
      size={props.entry.size || 0}
      depth={props.depth}
      isSelected={props.isSelected}
      isExpanded={props.isExpanded}
      isLoading={props.isLoading}
      hasChildren={hasChildren()}
      onClick={handleClick}
      onToggle={() => props.onToggle(props.containerPath, props.entry.path)}
      entryType={props.entry.entry_type}
      hash={props.entry.hash}
      data-entry-path={props.entry.path}
    />
  );
}

/**
 * Creates a SelectedEntry from a lazy-loaded entry
 */
export function createLazySelectedEntry(containerPath: string, entry: LazyTreeEntry): SelectedEntry {
  return {
    containerPath,
    entryPath: entry.path,
    name: entry.name,
    size: entry.size || 0,
    isDir: entry.is_dir,
    isVfsEntry: false,
  };
}
