// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal } from "solid-js";
import type { UseVirtualListOptions } from "./types";

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
