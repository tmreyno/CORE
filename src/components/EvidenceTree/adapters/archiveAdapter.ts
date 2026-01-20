// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Archive Tree Adapter
 * 
 * Adapter for archive entries (ZIP, 7z, RAR, TAR).
 */

import type { ArchiveTreeEntry } from "../../../types";
import type { TreeNodeAdapter } from "../types";
import { isContainerFile } from "../containerDetection";

export const archiveAdapter: TreeNodeAdapter<ArchiveTreeEntry> = {
  getKey: (entry: ArchiveTreeEntry, containerPath: string) => {
    return `${containerPath}::archive::${entry.path}`;
  },
  
  getName: (entry: ArchiveTreeEntry) => {
    if (entry.name) return entry.name;
    const parts = entry.path.split('/').filter(p => p);
    return parts[parts.length - 1] || entry.path;
  },
  
  getPath: (entry: ArchiveTreeEntry) => entry.path,
  
  isDir: (entry: ArchiveTreeEntry) => entry.isDir,
  
  getSize: (entry: ArchiveTreeEntry) => entry.size,
  
  getHash: (entry: ArchiveTreeEntry) => entry.crc32 ? entry.crc32.toString(16).toUpperCase() : undefined,
  
  hasChildren: (entry: ArchiveTreeEntry) => entry.isDir,
  
  getEntryType: (entry: ArchiveTreeEntry) => {
    // Mark nested containers with special type
    if (!entry.isDir && isContainerFile(entry.name || entry.path)) {
      return "container";
    }
    return undefined;
  },
  
  isNestedContainer: (entry: ArchiveTreeEntry) => {
    return !entry.isDir && isContainerFile(entry.name || entry.path);
  },
};

export default archiveAdapter;
