// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * VirtualList - High-performance virtualized list component
 * 
 * Renders only visible items plus overscan buffer for smooth scrolling.
 * Uses windowing technique to handle lists of any size efficiently.
 * 
 * Features:
 * - Fixed or variable item heights
 * - Overscan for smooth scrolling
 * - Keyboard navigation support
 * - Scroll restoration
 * - Dynamic content measurement
 * 
 * For tree views, use with TREE_ROW_HEIGHT from tree/constants.ts
 */

import { createSignal, createEffect, For, JSX, Accessor, createMemo } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { TREE_ROW_HEIGHT, VIRTUAL_LIST_OVERSCAN } from "./tree/constants";

// ============================================================================
// Types
// ============================================================================

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

// ============================================================================
// Main VirtualList Component
// ============================================================================

export function VirtualList<T>(props: VirtualListProps<T>) {
  const [scrollTop, setScrollTop] = createSignal(props.initialScrollTop || 0);
  let containerRef: HTMLDivElement | undefined;
  let scrollRef: HTMLDivElement | undefined;

  // Use shared constant for default overscan (optimized for tree views)
  const overscan = () => props.overscan ?? VIRTUAL_LIST_OVERSCAN;
  
  // Calculate container height in pixels
  const containerHeight = createMemo(() => {
    if (typeof props.height === 'number') return props.height;
    return 400; // Default fallback
  });

  // Calculate visible range
  const visibleRange = createMemo(() => {
    const h = containerHeight();
    const itemH = props.itemHeight;
    const top = scrollTop();
    
    const startIndex = Math.max(0, Math.floor(top / itemH) - overscan());
    const endIndex = Math.min(
      props.items.length,
      Math.ceil((top + h) / itemH) + overscan()
    );
    
    return { startIndex, endIndex };
  });

  // Calculate total height
  const totalHeight = createMemo(() => props.items.length * props.itemHeight);

  // Get visible items with their indices
  const visibleItems = createMemo(() => {
    const { startIndex, endIndex } = visibleRange();
    const result: Array<{ item: T; index: number; offset: number }> = [];
    
    for (let i = startIndex; i < endIndex; i++) {
      if (i < props.items.length) {
        result.push({
          item: props.items[i],
          index: i,
          offset: i * props.itemHeight,
        });
      }
    }
    
    return result;
  });

  // Handle scroll
  const handleScroll = (e: Event) => {
    const target = e.target as HTMLDivElement;
    const newScrollTop = target.scrollTop;
    setScrollTop(newScrollTop);
    props.onScroll?.(newScrollTop);
  };

  // Keyboard navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!props.keyboardNavigation || props.focusedIndex === undefined) return;

    const currentIndex = props.focusedIndex;
    let newIndex = currentIndex;

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        newIndex = Math.min(currentIndex + 1, props.items.length - 1);
        break;
      case "ArrowUp":
        e.preventDefault();
        newIndex = Math.max(currentIndex - 1, 0);
        break;
      case "PageDown":
        e.preventDefault();
        const pageSize = Math.floor(containerHeight() / props.itemHeight);
        newIndex = Math.min(currentIndex + pageSize, props.items.length - 1);
        break;
      case "PageUp":
        e.preventDefault();
        const pageSizeUp = Math.floor(containerHeight() / props.itemHeight);
        newIndex = Math.max(currentIndex - pageSizeUp, 0);
        break;
      case "Home":
        e.preventDefault();
        newIndex = 0;
        break;
      case "End":
        e.preventDefault();
        newIndex = props.items.length - 1;
        break;
      case "Enter":
        e.preventDefault();
        if (currentIndex >= 0 && currentIndex < props.items.length) {
          props.onItemActivate?.(props.items[currentIndex], currentIndex);
        }
        return;
    }

    if (newIndex !== currentIndex) {
      props.onFocusChange?.(newIndex);
      scrollToIndex(newIndex);
    }
  };

  // Scroll to specific index
  const scrollToIndex = (index: number) => {
    if (!scrollRef) return;
    
    const itemTop = index * props.itemHeight;
    const itemBottom = itemTop + props.itemHeight;
    const viewTop = scrollTop();
    const viewBottom = viewTop + containerHeight();

    if (itemTop < viewTop) {
      scrollRef.scrollTop = itemTop;
    } else if (itemBottom > viewBottom) {
      scrollRef.scrollTop = itemBottom - containerHeight();
    }
  };

  // Expose scroll methods
  createEffect(() => {
    if (props.initialScrollTop && scrollRef) {
      scrollRef.scrollTop = props.initialScrollTop;
    }
  });

  // Clean up scroll listener
  createEffect(() => {
    if (scrollRef) {
      // makeEventListener auto-cleans up when effect re-runs or component unmounts
      makeEventListener(scrollRef, "scroll", handleScroll, { passive: true });
    }
  });

  return (
    <div
      ref={containerRef}
      class={`virtual-list-wrapper ${props.class || ""}`}
      style={{
        position: "relative",
        ...(props.style || {}),
      }}
      onKeyDown={handleKeyDown}
      tabIndex={props.keyboardNavigation ? 0 : undefined}
    >
      <div
        ref={scrollRef}
        class="virtual-list-scroller overflow-y-auto"
        style={{
          height: typeof props.height === 'number' ? `${props.height}px` : props.height,
          width: "100%",
        }}
      >
        {/* Spacer for total content height */}
        <div
          style={{
            height: `${totalHeight()}px`,
            position: "relative",
          }}
        >
          {/* Visible items positioned absolutely */}
          <For each={visibleItems()}>
            {(vItem) => {
              const index = () => vItem.index;
              const key = props.getKey 
                ? props.getKey(vItem.item, vItem.index)
                : vItem.index;
                
              return (
                <div
                  data-index={vItem.index}
                  data-key={key}
                  style={{
                    position: "absolute",
                    top: `${vItem.offset}px`,
                    left: 0,
                    right: 0,
                    height: `${props.itemHeight}px`,
                  }}
                >
                  {props.children(vItem.item, index)}
                </div>
              );
            }}
          </For>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Flattened Tree for Virtual List
// ============================================================================

interface FlattenedNode<T> {
  item: T;
  depth: number;
  isExpanded: boolean;
  hasChildren: boolean;
}

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

// ============================================================================
// VirtualTree Component (wraps VirtualList for tree structures)
// ============================================================================

export function VirtualTree<T>(props: VirtualTreeProps<T>) {
  // Flatten tree structure for virtual rendering
  const flattenedItems = createMemo(() => 
    flattenTree(
      props.items,
      props.getChildren,
      props.hasChildren,
      props.isExpanded
    )
  );

  return (
    <VirtualList
      items={flattenedItems()}
      itemHeight={props.itemHeight}
      height={props.height}
      overscan={props.overscan}
      class={props.class}
      style={props.style}
      onScroll={props.onScroll}
      initialScrollTop={props.initialScrollTop}
      keyboardNavigation={props.keyboardNavigation}
      focusedIndex={props.focusedIndex}
      onFocusChange={props.onFocusChange}
      getKey={(node, i) => `${i}-${node.depth}`}
    >
      {(node, index) => props.children(node.item, index, node.depth)}
    </VirtualList>
  );
}

// ============================================================================
// Utility Hook for Virtual List State
// ============================================================================

export interface UseVirtualListOptions<T> {
  items: T[];
  itemHeight: number;
  containerHeight: number;
}

export function useVirtualList<T>(options: UseVirtualListOptions<T>) {
  const [scrollTop, setScrollTop] = createSignal(0);
  const [focusedIndex, setFocusedIndex] = createSignal(-1);

  const scrollToIndex = (index: number) => {
    const itemTop = index * options.itemHeight;
    const itemBottom = itemTop + options.itemHeight;
    const viewTop = scrollTop();
    const viewBottom = viewTop + options.containerHeight;

    if (itemTop < viewTop) {
      setScrollTop(itemTop);
    } else if (itemBottom > viewBottom) {
      setScrollTop(itemBottom - options.containerHeight);
    }
  };

  const scrollToItem = (item: T) => {
    const index = options.items.indexOf(item);
    if (index >= 0) {
      scrollToIndex(index);
    }
  };

  return {
    scrollTop,
    setScrollTop,
    focusedIndex,
    setFocusedIndex,
    scrollToIndex,
    scrollToItem,
    totalHeight: () => options.items.length * options.itemHeight,
  };
}

// Re-export tree constants for convenience when using VirtualList with trees
export { TREE_ROW_HEIGHT, VIRTUAL_LIST_OVERSCAN };

export default VirtualList;
