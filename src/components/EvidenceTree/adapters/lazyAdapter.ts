// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Lazy Tree Adapter
 * 
 * Adapter for lazy-loaded tree entries (unified format from backend).
 */

import type { LazyTreeEntry } from "../../../types/lazy-loading";
import type { TreeNodeAdapter } from "../types";

export const lazyAdapter: TreeNodeAdapter<LazyTreeEntry> = {
  getKey: (entry: LazyTreeEntry, containerPath: string) => {
    return `${containerPath}::lazy::${entry.path}`;
  },
  
  getName: (entry: LazyTreeEntry) => entry.name,
  
  getPath: (entry: LazyTreeEntry) => entry.path,
  
  isDir: (entry: LazyTreeEntry) => entry.is_dir,
  
  getSize: (entry: LazyTreeEntry) => entry.size ?? undefined,
  
  getHash: (entry: LazyTreeEntry) => entry.hash ?? undefined,
  
  getEntryType: (entry: LazyTreeEntry) => entry.entry_type ?? undefined,
  
  hasChildren: (entry: LazyTreeEntry) => {
    return entry.is_dir && (entry.child_count ?? 0) > 0;
  },
  
  isNestedContainer: () => false, // Lazy entries don't track nested containers
};

export default lazyAdapter;
