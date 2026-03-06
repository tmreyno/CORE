// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createMemo } from "solid-js";
import { VirtualListComponent } from "./VirtualListComponent";
import { flattenTree } from "./treeUtils";
import type { VirtualTreeProps } from "./types";

export function VirtualTreeComponent<T>(props: VirtualTreeProps<T>) {
  const flattenedItems = createMemo(() => 
    flattenTree(
      props.items,
      props.getChildren,
      props.hasChildren,
      props.isExpanded
    )
  );

  return (
    <VirtualListComponent
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
    </VirtualListComponent>
  );
}
