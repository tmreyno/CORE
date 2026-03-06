// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * FolderNode — recursive folder tree node for the PST viewer sidebar.
 */

import { createSignal, Show, For, type Component } from "solid-js";
import { FolderIcon, ChevronDownIcon, ChevronRightIcon } from "../icons";
import type { PstFolderInfo } from "../../types/pst";

export const FolderNode: Component<{
  folder: PstFolderInfo;
  selectedId: number | null;
  depth: number;
  onSelect: (folder: PstFolderInfo) => void;
}> = (props) => {
  const [expanded, setExpanded] = createSignal(props.depth < 2);
  const isSelected = () => props.selectedId === props.folder.nodeId;
  const hasChildren = () => props.folder.children.length > 0;

  return (
    <div>
      <button
        class="w-full flex items-center gap-1 px-2 py-1 text-sm rounded hover:bg-bg-hover transition-colors"
        classList={{
          "bg-bg-active text-accent": isSelected(),
          "text-txt": !isSelected(),
        }}
        style={{ "padding-left": `${props.depth * 16 + 8}px` }}
        onClick={() => {
          props.onSelect(props.folder);
          if (hasChildren()) setExpanded(!expanded());
        }}
      >
        <Show
          when={hasChildren()}
          fallback={<span class="w-4 inline-block" />}
        >
          <span class="w-4 h-4 flex items-center justify-center text-txt-muted">
            <Show when={expanded()} fallback={<ChevronRightIcon class="w-3 h-3" />}>
              <ChevronDownIcon class="w-3 h-3" />
            </Show>
          </span>
        </Show>
        <FolderIcon class="w-icon-sm h-icon-sm text-txt-muted flex-shrink-0" />
        <span class="truncate flex-1 text-left">{props.folder.name}</span>
        <Show when={props.folder.contentCount > 0}>
          <span class="text-xs text-txt-muted ml-1">
            {props.folder.contentCount}
          </span>
        </Show>
        <Show when={props.folder.unreadCount > 0}>
          <span class="text-xs text-accent font-semibold ml-1">
            ({props.folder.unreadCount})
          </span>
        </Show>
      </button>
      <Show when={expanded() && hasChildren()}>
        <For each={props.folder.children}>
          {(child) => (
            <FolderNode
              folder={child}
              selectedId={props.selectedId}
              depth={props.depth + 1}
              onSelect={props.onSelect}
            />
          )}
        </For>
      </Show>
    </div>
  );
};
