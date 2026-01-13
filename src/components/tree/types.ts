// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Tree Types - Shared type definitions for tree components
 */

import type { TreeEntry, ArchiveTreeEntry, UfedTreeEntry, VfsEntry } from '../../types';

/** State of a tree node (loading, expanded, selected) */
export interface TreeNodeState {
  /** Whether the node is currently loading children */
  isLoading: boolean;
  /** Whether the node is expanded */
  isExpanded: boolean;
  /** Whether the node is selected */
  isSelected: boolean;
  /** Error message if loading failed */
  error?: string | null;
}

/** Generic tree item data that can represent any container type */
export interface TreeItemData {
  /** Unique identifier/path for this item */
  id: string;
  /** Display name */
  name: string;
  /** Full path within container */
  path: string;
  /** Whether this is a directory/folder */
  isDir: boolean;
  /** Size in bytes (for files) */
  size: number;
  /** Container type this entry belongs to */
  containerType: 'ad1' | 'e01' | 'vfs' | 'archive' | 'ufed' | 'raw' | 'l01';
  /** File extension (derived from name) */
  extension?: string;
  /** Item type code (AD1-specific) */
  itemType?: number;
  /** Address for lazy loading children */
  childAddr?: number | null;
  /** Address for reading file data */
  dataAddr?: number | null;
  /** Item address for hex navigation */
  itemAddr?: number | null;
  /** Hash value if available */
  hash?: string | null;
  /** Entry type label (UFED-specific) */
  entryType?: string;
  /** Original entry data for format-specific operations */
  originalEntry?: TreeEntry | ArchiveTreeEntry | UfedTreeEntry | VfsEntry;
}

/** Depth level for indentation (0 = root) */
export type TreeDepth = number;

/** Container type classification for styling */
export type ContainerTypeClass = 
  | 'ad1' 
  | 'e01' 
  | 'l01' 
  | 'raw' 
  | 'vfs' 
  | 'archive' 
  | 'ufed' 
  | 'zip' 
  | '7z' 
  | 'tar';
