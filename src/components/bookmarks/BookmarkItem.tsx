// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal } from "solid-js";
import { getBasename } from "../../utils/pathUtils";
import {
  HiOutlineBookmark,
  HiOutlineTrash,
  HiOutlinePencilSquare,
} from "../icons";
import type { BookmarkItemProps } from "./types";
import { getTargetTypeIcon, getBookmarkColorClass } from "./helpers";

export const BookmarkItem: Component<BookmarkItemProps> = (props) => {
  const [showActions, setShowActions] = createSignal(false);

  const Icon = getTargetTypeIcon(props.bookmark.target_type);

  const handleClick = () => {
    props.onNavigate?.(props.bookmark);
  };

  const handleRemove = (e: MouseEvent) => {
    e.stopPropagation();
    props.onRemove?.(props.bookmark.id);
  };

  const handleEdit = (e: MouseEvent) => {
    e.stopPropagation();
    props.onEdit?.(props.bookmark);
  };

  return (
    <div
      class="group flex items-start gap-2 px-2 py-2 hover:bg-bg-hover rounded-md cursor-pointer transition-colors relative"
      onClick={handleClick}
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
    >
      {/* Icon */}
      <div class={`flex-shrink-0 mt-0.5 ${getBookmarkColorClass(props.bookmark.color)}`}>
        <HiOutlineBookmark class="w-4 h-4" />
      </div>

      {/* Content */}
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="text-sm font-medium text-txt truncate">
            {props.bookmark.name}
          </span>
          <Icon class="w-3.5 h-3.5 text-txt-muted flex-shrink-0" />
        </div>

        <Show when={!props.compact}>
          <p class="text-xs text-txt-muted truncate mt-0.5" title={props.bookmark.target_path}>
            {getBasename(props.bookmark.target_path) || props.bookmark.target_path}
          </p>
        </Show>

        <Show when={props.bookmark.tags && props.bookmark.tags.length > 0 && !props.compact}>
          <div class="flex items-center gap-1 mt-1 flex-wrap">
            <For each={props.bookmark.tags!.slice(0, 3)}>
              {(tag) => (
                <span class="text-2xs px-1.5 py-0.5 bg-bg-secondary text-txt-muted rounded">
                  {tag}
                </span>
              )}
            </For>
            <Show when={props.bookmark.tags!.length > 3}>
              <span class="text-2xs text-txt-muted">
                +{props.bookmark.tags!.length - 3}
              </span>
            </Show>
          </div>
        </Show>
      </div>

      {/* Actions */}
      <Show when={showActions()}>
        <div class="flex items-center gap-1 absolute right-2 top-2 bg-bg-panel rounded shadow-sm">
          <button
            onClick={handleEdit}
            class="p-1 hover:bg-bg-hover rounded transition-colors"
            title="Edit bookmark"
          >
            <HiOutlinePencilSquare class="w-3.5 h-3.5 text-txt-muted hover:text-txt" />
          </button>
          <button
            onClick={handleRemove}
            class="p-1 hover:bg-error/10 rounded transition-colors"
            title="Remove bookmark"
          >
            <HiOutlineTrash class="w-3.5 h-3.5 text-txt-muted hover:text-error" />
          </button>
        </div>
      </Show>
    </div>
  );
};
