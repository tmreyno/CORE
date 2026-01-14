// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * VFS Tree Adapter
 * 
 * Adapter for VFS (E01, Raw, L01) filesystem entries.
 */

import type { VfsEntry } from "../../../types";
import type { TreeNodeAdapter } from "../types";

export const vfsAdapter: TreeNodeAdapter<VfsEntry> = {
  getKey: (entry: VfsEntry, containerPath: string) => {
    return `${containerPath}::vfs::${entry.path}`;
  },
  
  getName: (entry: VfsEntry) => entry.name,
  
  getPath: (entry: VfsEntry) => entry.path,
  
  isDir: (entry: VfsEntry) => entry.is_dir,
  
  getSize: (entry: VfsEntry) => entry.size,
  
  getHash: () => undefined, // VFS entries don't store hashes in the tree
  
  hasChildren: (entry: VfsEntry) => entry.is_dir,
  
  isNestedContainer: () => false, // Could be extended to detect nested containers
};

export default vfsAdapter;
