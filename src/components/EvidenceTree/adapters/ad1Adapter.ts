// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AD1 Tree Adapter
 * 
 * Adapter for AD1 container tree entries (TreeEntry type).
 */

import type { TreeEntry } from "../../../types";
import type { TreeNodeAdapter } from "../types";

export const ad1Adapter: TreeNodeAdapter<TreeEntry> = {
  getKey: (entry: TreeEntry, containerPath: string) => {
    // Use item_addr if available for unique identification
    return entry.item_addr != null
      ? `${containerPath}::addr:${entry.item_addr}`
      : `${containerPath}::path:${entry.path}`;
  },
  
  getName: (entry: TreeEntry) => entry.name,
  
  getPath: (entry: TreeEntry) => entry.path,
  
  isDir: (entry: TreeEntry) => entry.is_dir,
  
  getSize: (entry: TreeEntry) => entry.size,
  
  getHash: (entry: TreeEntry) => entry.md5_hash || entry.sha1_hash || undefined,
  
  hasChildren: (entry: TreeEntry) => {
    return entry.is_dir && (entry.first_child_addr ?? 0) > 0;
  },
  
  isNestedContainer: () => false, // AD1 doesn't have nested containers
};

export default ad1Adapter;
