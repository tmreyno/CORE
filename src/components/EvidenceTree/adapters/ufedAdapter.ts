// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * UFED Tree Adapter
 * 
 * Adapter for UFED mobile extraction tree entries.
 */

import type { UfedTreeEntry } from "../../../types";
import type { TreeNodeAdapter } from "../types";

export const ufedAdapter: TreeNodeAdapter<UfedTreeEntry> = {
  getKey: (entry: UfedTreeEntry, containerPath: string) => {
    return `${containerPath}::ufed::${entry.path}`;
  },
  
  getName: (entry: UfedTreeEntry) => entry.name,
  
  getPath: (entry: UfedTreeEntry) => entry.path,
  
  isDir: (entry: UfedTreeEntry) => entry.isDir,
  
  getSize: (entry: UfedTreeEntry) => entry.size,
  
  getHash: (entry: UfedTreeEntry) => entry.hash ?? undefined,
  
  getEntryType: (entry: UfedTreeEntry) => entry.entryType ?? undefined,
  
  hasChildren: (entry: UfedTreeEntry) => entry.isDir,
  
  isNestedContainer: () => false,
};

export default ufedAdapter;
