// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { JSX, Accessor } from "solid-js";

export interface VirtualListProps<T> {
  /** Array of items to render */
  items: T[];
  /** Fixed height for each item (required for performance) */
  itemHeight: number;
  /** Height of the container */
  height: number | string;
  /** Number of items to render above/below visible area */
  overscan?: number;
  /** CSS class for the container */
  class?: string;
  /** Render function for each item */
  children: (item: T, index: Accessor<number>) => JSX.Element;
  /** Callback when scroll position changes */
  onScroll?: (scrollTop: number) => void;
  /** Initial scroll position */
  initialScrollTop?: number;
  /** Get unique key for item */
  getKey?: (item: T, index: number) => string | number;
  /** Custom container style */
  style?: JSX.CSSProperties;
  /** Enable keyboard navigation */
  keyboardNavigation?: boolean;
  /** Currently focused/selected index */
  focusedIndex?: number;
  /** Callback when focused index changes via keyboard */
  onFocusChange?: (index: number) => void;
  /** Callback when item is activated (Enter/click) */
  onItemActivate?: (item: T, index: number) => void;
}

export interface VirtualTreeProps<T> extends Omit<VirtualListProps<T>, 'items' | 'children'> {
  /** Tree data with nested structure */
  items: T[];
  /** Get children of a node */
  getChildren: (item: T) => T[];
  /** Check if node has children */
  hasChildren: (item: T) => boolean;
  /** Check if node is expanded */
  isExpanded: (item: T) => boolean;
  /** Toggle expansion state */
  onToggle: (item: T) => void;
  /** Render function for each item */
  children: (item: T, index: Accessor<number>, depth: number) => JSX.Element;
  /** Get item depth for indentation */
  getDepth?: (item: T) => number;
}

export interface FlattenedNode<T> {
  item: T;
  depth: number;
  isExpanded: boolean;
  hasChildren: boolean;
}

export interface UseVirtualListOptions<T> {
  items: T[];
  itemHeight: number;
  containerHeight: number;
}
