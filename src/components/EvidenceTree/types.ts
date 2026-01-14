// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceTree Types
 * 
 * Unified types for all tree node components across container types.
 */

import type { JSX } from "solid-js";

/** Container category types */
export type ContainerCategory = "ad1" | "vfs" | "archive" | "ufed" | "lazy" | "unknown";

/** Props for selecting an entry to view - includes hex location info */
export interface SelectedEntry {
  containerPath: string;
  entryPath: string;
  name: string;
  size: number;
  isDir: boolean;
  /** Whether this entry is from a VFS container (E01/Raw) */
  isVfsEntry?: boolean;
  /** Direct address for reading file data */
  dataAddr?: number | null;
  // === HEX LOCATION FIELDS ===
  /** Address of the item header in the container */
  itemAddr?: number | null;
  /** Size of compressed data in bytes */
  compressedSize?: number | null;
  /** Address where compressed data ends */
  dataEndAddr?: number | null;
  /** Address of first metadata entry for this item */
  metadataAddr?: number | null;
  /** Address of first child (for folders) */
  firstChildAddr?: number | null;
}

// NOTE: ArchiveQuickMetadata is defined in ./hooks/useArchiveTree.ts to avoid duplication

/**
 * Generic adapter interface for tree nodes
 * 
 * Each container type implements this adapter to provide consistent
 * behavior across AD1, VFS, Archive, UFED, etc.
 */
export interface TreeNodeAdapter<T> {
  /** Get unique key for the entry (used for selection, expansion tracking) */
  getKey: (entry: T, containerPath: string) => string;
  
  /** Get display name */
  getName: (entry: T) => string;
  
  /** Get full path within container */
  getPath: (entry: T) => string;
  
  /** Check if entry is a directory */
  isDir: (entry: T) => boolean;
  
  /** Get file size (undefined for dirs without stored size) */
  getSize: (entry: T) => number | undefined;
  
  /** Get hash if available */
  getHash?: (entry: T) => string | undefined;
  
  /** Get entry type for special icons (e.g., "container" for nested containers) */
  getEntryType?: (entry: T) => string | undefined;
  
  /** Check if this entry can contain children */
  hasChildren: (entry: T) => boolean;
  
  /** Get additional badge JSX to display */
  getBadge?: (entry: T, isSelected: boolean) => JSX.Element | undefined;
  
  /** Check if this entry is a nested container that can be opened */
  isNestedContainer?: (entry: T) => boolean;
}

/**
 * Props for GenericTreeNode component
 */
export interface GenericTreeNodeProps<T> {
  entry: T;
  containerPath: string;
  depth: number;
  adapter: TreeNodeAdapter<T>;
  
  // State callbacks
  isExpanded: (key: string) => boolean;
  isLoading: (key: string) => boolean;
  isSelected: (key: string) => boolean;
  getChildren: (entry: T) => T[];
  
  // Event handlers
  onSelect: (entry: T) => void;
  onToggle: (entry: T) => void;
  onDoubleClick?: (entry: T) => void;
  
  // Pagination (optional)
  hasMore?: (key: string) => boolean;
  totalCount?: (key: string) => number;
  isLoadingMore?: (key: string) => boolean;
  onLoadMore?: (entry: T) => void;
}

/**
 * Container header display info
 */
export interface ContainerDisplayInfo {
  icon: JSX.Element;
  label: string;
  sublabel?: string;
  stats?: {
    items?: number;
    size?: number;
    files?: number;
    dirs?: number;
  };
}
