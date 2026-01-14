// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Sorting Utilities for Tree Entries
 * 
 * Provides consistent sorting functions for all tree entry types.
 * Standard sort order: directories first, then alphabetically by name.
 */

import type { TreeEntry, VfsEntry, ArchiveTreeEntry, UfedTreeEntry } from "../../../types";
import type { LazyTreeEntry } from "../../../types/lazy-loading";

/** Generic sortable entry interface */
interface SortableEntry {
  is_dir: boolean;
  name?: string;
  path?: string;
}

/**
 * Generic sort function for any entry type
 * Sorts directories first, then alphabetically by name
 */
export function sortByDirFirst<T extends SortableEntry>(entries: T[]): T[] {
  return [...entries].sort((a, b) => {
    if (a.is_dir && !b.is_dir) return -1;
    if (!a.is_dir && b.is_dir) return 1;
    const aName = a.name || a.path || "";
    const bName = b.name || b.path || "";
    return aName.localeCompare(bName);
  });
}

/**
 * Sort AD1 tree entries
 */
export function sortTreeEntries(entries: TreeEntry[]): TreeEntry[] {
  return sortByDirFirst(entries);
}

/**
 * Sort VFS entries
 */
export function sortVfsEntries(entries: VfsEntry[]): VfsEntry[] {
  return sortByDirFirst(entries);
}

/**
 * Sort archive entries by path
 */
export function sortArchiveEntries(entries: ArchiveTreeEntry[]): ArchiveTreeEntry[] {
  return [...entries].sort((a, b) => {
    if (a.is_dir && !b.is_dir) return -1;
    if (!a.is_dir && b.is_dir) return 1;
    return a.path.localeCompare(b.path);
  });
}

/**
 * Sort UFED entries
 */
export function sortUfedEntries(entries: UfedTreeEntry[]): UfedTreeEntry[] {
  return sortByDirFirst(entries);
}

/**
 * Sort lazy-loaded entries
 */
export function sortLazyEntries(entries: LazyTreeEntry[]): LazyTreeEntry[] {
  return sortByDirFirst(entries);
}
