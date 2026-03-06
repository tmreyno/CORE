// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

export { VirtualListComponent as VirtualList } from "./VirtualListComponent";
export { VirtualTreeComponent as VirtualTree } from "./VirtualTreeComponent";
export { flattenTree } from "./treeUtils";
export { useVirtualList } from "./useVirtualListState";
export type {
  VirtualListProps,
  VirtualTreeProps,
  FlattenedNode,
  UseVirtualListOptions,
} from "./types";

// Re-export tree constants for convenience
export { TREE_ROW_HEIGHT, VIRTUAL_LIST_OVERSCAN } from "../tree/constants";
