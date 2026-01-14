// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LoadMoreButton - Pagination button for lazy-loaded tree entries
 */

import { Show } from "solid-js";
import { HiOutlineArrowPath, HiOutlineChevronDown } from "../icons";
import { TREE_ROW_BASE_CLASSES, TREE_ROW_NORMAL_CLASSES, getTreeIndent } from "./constants";

export interface LoadMoreButtonProps {
  /** Current number of loaded entries */
  loadedCount: number;
  /** Total count of entries */
  totalCount: number;
  /** Whether more entries are currently loading */
  isLoading: boolean;
  /** Depth level for indentation */
  depth: number;
  /** Click handler */
  onClick: (e: MouseEvent) => void;
}

export function LoadMoreButton(props: LoadMoreButtonProps) {
  return (
    <div 
      class={`${TREE_ROW_BASE_CLASSES} ${TREE_ROW_NORMAL_CLASSES} cursor-pointer hover:bg-zinc-700/30`}
      style={{ "padding-left": getTreeIndent(props.depth) }}
      onClick={props.onClick}
    >
      <Show 
        when={!props.isLoading} 
        fallback={
          <span class="flex items-center gap-2 text-xs text-zinc-400">
            <HiOutlineArrowPath class="w-3.5 h-3.5 animate-spin" />
            Loading...
          </span>
        }
      >
        <span class="flex items-center gap-2 text-xs text-cyan-400 hover:text-cyan-300">
          <HiOutlineChevronDown class="w-3.5 h-3.5" />
          Load more ({props.loadedCount.toLocaleString()} of {props.totalCount.toLocaleString()})
        </span>
      </Show>
    </div>
  );
}
