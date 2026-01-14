// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Key Generation Utilities for EvidenceTree
 * 
 * Provides consistent key generation for tree node selection and expansion tracking.
 */

import type { TreeEntry, VfsEntry, ArchiveTreeEntry, UfedTreeEntry } from "../../../types";
import type { LazyTreeEntry } from "../../../types/lazy-loading";

/**
 * Generate a unique key for an AD1 entry
 * Uses item_addr for address-based navigation, falls back to path
 */
export function getAd1EntryKey(containerPath: string, entry: TreeEntry): string {
  return `${containerPath}::${entry.item_addr ?? entry.path}`;
}

/**
 * Generate a node key for AD1 tree expansion tracking
 * Uses first_child_addr for address-based loading
 */
export function getAd1NodeKey(containerPath: string, entry: TreeEntry): string {
  const addr = entry.first_child_addr;
  return addr 
    ? `${containerPath}::addr:${addr}` 
    : `${containerPath}::path:${entry.path}`;
}

/**
 * Generate a unique key for a VFS entry
 */
export function getVfsEntryKey(containerPath: string, entry: VfsEntry): string {
  return `${containerPath}::vfs::${entry.path}`;
}

/**
 * Generate a unique key for an archive entry
 */
export function getArchiveEntryKey(containerPath: string, entry: ArchiveTreeEntry): string {
  return `${containerPath}::archive::${entry.path}`;
}

/**
 * Generate a unique key for a lazy-loaded entry
 */
export function getLazyEntryKey(containerPath: string, entry: LazyTreeEntry): string {
  return `${containerPath}::lazy::${entry.path}`;
}

/**
 * Generate a unique key for a UFED entry
 */
export function getUfedEntryKey(containerPath: string, entry: UfedTreeEntry): string {
  return `${containerPath}::ufed::${entry.path}`;
}

/**
 * Type-safe entry key type union
 */
export type EntryKeyType = "ad1" | "vfs" | "archive" | "lazy" | "ufed";

/**
 * Generate entry key based on container type
 */
export function getEntryKey(
  containerPath: string,
  entryPath: string,
  type: EntryKeyType
): string {
  switch (type) {
    case "ad1":
      return `${containerPath}::${entryPath}`;
    case "vfs":
      return `${containerPath}::vfs::${entryPath}`;
    case "archive":
      return `${containerPath}::archive::${entryPath}`;
    case "lazy":
      return `${containerPath}::lazy::${entryPath}`;
    case "ufed":
      return `${containerPath}::ufed::${entryPath}`;
  }
}
