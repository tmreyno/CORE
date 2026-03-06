// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal } from "solid-js";
import {
  HiOutlineChevronDown,
  HiOutlineChevronRight,
} from "../icons";
import type { BookmarkGroupProps } from "./types";
import { getTargetTypeIcon, getTargetTypeLabel } from "./helpers";
import { BookmarkItem } from "./BookmarkItem";

export const BookmarkGroup: Component<BookmarkGroupProps> = (props) => {
  const [expanded, setExpanded] = createSignal(props.defaultExpanded ?? true);

  const Icon = getTargetTypeIcon(props.type);

  return (
    <div class="border-b border-border/30 last:border-b-0">
      {/* Group Header */}
      <button
        class="flex items-center gap-2 w-full px-2 py-2 hover:bg-bg-hover/50 transition-colors"
        onClick={() => setExpanded(!expanded())}
      >
        <Show when={expanded()} fallback={<HiOutlineChevronRight class="w-4 h-4 text-txt-muted" />}>
          <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
        </Show>
        <Icon class="w-4 h-4 text-txt-secondary" />
        <span class="text-sm font-medium text-txt-secondary">
          {getTargetTypeLabel(props.type)}
        </span>
        <span class="text-xs text-txt-muted ml-auto">
          {props.bookmarks.length}
        </span>
      </button>

      {/* Group Items */}
      <Show when={expanded()}>
        <div class="pl-2">
          <For each={props.bookmarks}>
            {(bookmark) => (
              <BookmarkItem
                bookmark={bookmark}
                onNavigate={props.onNavigate}
                onRemove={props.onRemove}
                onEdit={props.onEdit}
                compact={props.compact}
              />
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};
