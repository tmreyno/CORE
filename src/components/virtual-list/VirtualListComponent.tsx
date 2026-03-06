// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, For, createMemo } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { VIRTUAL_LIST_OVERSCAN } from "../tree/constants";
import type { VirtualListProps } from "./types";

export function VirtualListComponent<T>(props: VirtualListProps<T>) {
  const [scrollTop, setScrollTop] = createSignal(props.initialScrollTop || 0);
  let containerRef: HTMLDivElement | undefined;
  let scrollRef: HTMLDivElement | undefined;

  const overscan = () => props.overscan ?? VIRTUAL_LIST_OVERSCAN;
  
  const containerHeight = createMemo(() => {
    if (typeof props.height === 'number') return props.height;
    return 400;
  });

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

  const totalHeight = createMemo(() => props.items.length * props.itemHeight);

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

  const handleScroll = (e: Event) => {
    const target = e.target as HTMLDivElement;
    const newScrollTop = target.scrollTop;
    setScrollTop(newScrollTop);
    props.onScroll?.(newScrollTop);
  };

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
        newIndex = Math.min(currentIndex + Math.floor(containerHeight() / props.itemHeight), props.items.length - 1);
        break;
      case "PageUp":
        e.preventDefault();
        newIndex = Math.max(currentIndex - Math.floor(containerHeight() / props.itemHeight), 0);
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

  createEffect(() => {
    if (props.initialScrollTop && scrollRef) {
      scrollRef.scrollTop = props.initialScrollTop;
    }
  });

  createEffect(() => {
    if (scrollRef) {
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
        <div
          style={{
            height: `${totalHeight()}px`,
            position: "relative",
          }}
        >
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
