// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { FlattenedNode } from "./types";

/**
 * Flatten a tree structure for virtual list rendering
 */
export function flattenTree<T>(
  items: T[],
  getChildren: (item: T) => T[],
  hasChildren: (item: T) => boolean,
  isExpanded: (item: T) => boolean,
  depth = 0
): FlattenedNode<T>[] {
  const result: FlattenedNode<T>[] = [];
  
  for (const item of items) {
    const hasKids = hasChildren(item);
    const expanded = isExpanded(item);
    
    result.push({
      item,
      depth,
      isExpanded: expanded,
      hasChildren: hasKids,
    });
    
    if (hasKids && expanded) {
      const children = getChildren(item);
      result.push(...flattenTree(children, getChildren, hasChildren, isExpanded, depth + 1));
    }
  }
  
  return result;
}
