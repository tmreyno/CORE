// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * BookmarksPanel — Displays and manages project bookmarks.
 *
 * Sub-components extracted to:
 *   - BookmarkItem.tsx (individual bookmark row)
 *   - BookmarkGroup.tsx (grouped bookmarks by type)
 *   - BookmarkEditDialog.tsx (edit modal)
 *   - helpers.ts (icon/label/color utilities)
 *   - types.ts (shared interfaces)
 */

import { Component, For, Show, createSignal, createMemo } from "solid-js";
import type { ProjectBookmark } from "../../types/project";
import {
  HiOutlineBookmark,
  HiOutlineMagnifyingGlass,
  HiOutlineXMark,
  HiOutlineFunnel,
} from "../icons";
import type { BookmarksPanelProps, BookmarkTargetType } from "./types";
import { getTargetTypeLabel } from "./helpers";
import { BookmarkGroup } from "./BookmarkGroup";
import { BookmarkEditDialog } from "./BookmarkEditDialog";

const TARGET_TYPES: BookmarkTargetType[] = ["file", "artifact", "search_result", "location"];

export const BookmarksPanel: Component<BookmarksPanelProps> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal("");
  const [filterType, setFilterType] = createSignal<BookmarkTargetType | "all">("all");
  const [showFilter, setShowFilter] = createSignal(false);
  const [editingBookmark, setEditingBookmark] = createSignal<ProjectBookmark | null>(null);

  const handleEditBookmark = (bookmark: ProjectBookmark) => {
    if (props.onUpdate) {
      setEditingBookmark(bookmark);
    } else {
      props.onEdit?.(bookmark);
    }
  };

  const handleSaveEdit = (
    bookmarkId: string,
    updates: Partial<Pick<ProjectBookmark, "name" | "color" | "tags" | "notes">>
  ) => {
    props.onUpdate?.(bookmarkId, updates);
    setEditingBookmark(null);
  };

  const filteredBookmarks = createMemo(() => {
    let result = props.bookmarks || [];

    const query = searchQuery().toLowerCase();
    if (query) {
      result = result.filter(
        (b) =>
          b.name.toLowerCase().includes(query) ||
          b.target_path.toLowerCase().includes(query) ||
          b.notes?.toLowerCase().includes(query) ||
          b.tags?.some((t) => t.toLowerCase().includes(query))
      );
    }

    if (filterType() !== "all") {
      result = result.filter((b) => b.target_type === filterType());
    }

    return result;
  });

  const groupedBookmarks = createMemo(() => {
    const groups: Record<BookmarkTargetType, ProjectBookmark[]> = {
      file: [],
      artifact: [],
      search_result: [],
      location: [],
    };
    filteredBookmarks().forEach((bookmark) => {
      if (groups[bookmark.target_type]) {
        groups[bookmark.target_type].push(bookmark);
      }
    });
    return groups;
  });

  const hasBookmarksInGroup = (type: BookmarkTargetType) =>
    groupedBookmarks()[type].length > 0;

  return (
    <div class="flex flex-col h-full bg-bg-panel">
      {/* Header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-border">
        <div class="flex items-center gap-2">
          <HiOutlineBookmark class="w-4 h-4 text-accent" />
          <span class="text-sm font-medium text-txt">Bookmarks</span>
          <Show when={props.bookmarks.length > 0}>
            <span class="text-xs text-txt-muted">({props.bookmarks.length})</span>
          </Show>
        </div>
        <div class="flex items-center gap-1">
          <button
            onClick={() => setShowFilter(!showFilter())}
            class={`p-1 rounded transition-colors ${
              showFilter() ? "bg-accent/20 text-accent" : "hover:bg-bg-hover text-txt-muted"
            }`}
            title="Filter bookmarks"
          >
            <HiOutlineFunnel class="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Search & Filter Bar */}
      <Show when={showFilter()}>
        <div class="px-3 py-2 border-b border-border/50 space-y-2">
          <div class="relative">
            <HiOutlineMagnifyingGlass class="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-txt-muted" />
            <input
              type="text"
              value={searchQuery()}
              onInput={(e) => setSearchQuery(e.currentTarget.value)}
              placeholder="Search bookmarks..."
              class="w-full pl-8 pr-8 py-1.5 text-sm bg-bg border border-border rounded-md focus:outline-none focus:ring-1 focus:ring-accent/50 text-txt placeholder:text-txt-muted"
            />
            <Show when={searchQuery()}>
              <button
                onClick={() => setSearchQuery("")}
                class="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 hover:bg-bg-hover rounded"
              >
                <HiOutlineXMark class="w-3.5 h-3.5 text-txt-muted" />
              </button>
            </Show>
          </div>

          <div class="flex items-center gap-1 flex-wrap">
            <button
              onClick={() => setFilterType("all")}
              class={`px-2 py-1 text-xs rounded transition-colors ${
                filterType() === "all"
                  ? "bg-accent text-white"
                  : "bg-bg-secondary text-txt-muted hover:text-txt"
              }`}
            >
              All
            </button>
            <For each={TARGET_TYPES}>
              {(type) => (
                <button
                  onClick={() => setFilterType(type)}
                  class={`px-2 py-1 text-xs rounded transition-colors ${
                    filterType() === type
                      ? "bg-accent text-white"
                      : "bg-bg-secondary text-txt-muted hover:text-txt"
                  }`}
                >
                  {getTargetTypeLabel(type)}
                </button>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Content */}
      <div class="flex-1 overflow-y-auto">
        <Show when={props.loading}>
          <div class="flex items-center justify-center py-8">
            <span class="text-sm text-txt-muted">Loading...</span>
          </div>
        </Show>

        <Show when={!props.loading && props.bookmarks.length === 0}>
          <div class="flex flex-col items-center justify-center py-12 px-4">
            <HiOutlineBookmark class="w-12 h-12 text-txt-muted/30 mb-3" />
            <p class="text-sm text-txt-muted text-center">No bookmarks yet</p>
            <p class="text-xs text-txt-muted/70 text-center mt-1">
              Right-click on files or artifacts to add bookmarks
            </p>
          </div>
        </Show>

        <Show when={!props.loading && props.bookmarks.length > 0 && filteredBookmarks().length === 0}>
          <div class="flex flex-col items-center justify-center py-12 px-4">
            <HiOutlineMagnifyingGlass class="w-10 h-10 text-txt-muted/30 mb-3" />
            <p class="text-sm text-txt-muted text-center">No matching bookmarks</p>
            <button
              onClick={() => {
                setSearchQuery("");
                setFilterType("all");
              }}
              class="text-xs text-accent hover:underline mt-2"
            >
              Clear filters
            </button>
          </div>
        </Show>

        <Show when={!props.loading && filteredBookmarks().length > 0}>
          <div class="py-1">
            <For each={TARGET_TYPES}>
              {(type) => (
                <Show when={hasBookmarksInGroup(type)}>
                  <BookmarkGroup
                    type={type}
                    bookmarks={groupedBookmarks()[type]}
                    onNavigate={props.onNavigate}
                    onRemove={props.onRemove}
                    onEdit={handleEditBookmark}
                    compact={props.compact}
                    defaultExpanded={true}
                  />
                </Show>
              )}
            </For>
          </div>
        </Show>
      </div>

      {/* Edit Dialog */}
      <Show when={editingBookmark()}>
        <BookmarkEditDialog
          bookmark={editingBookmark()!}
          onSave={handleSaveEdit}
          onCancel={() => setEditingBookmark(null)}
        />
      </Show>
    </div>
  );
};

export default BookmarksPanel;
