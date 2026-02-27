// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * BookmarksPanel - Displays and manages project bookmarks
 *
 * Features:
 * - Lists all bookmarks from the current project
 * - Groups bookmarks by target_type (files, artifacts, searches, locations)
 * - Click to navigate to bookmarked item
 * - Context menu to edit/delete bookmarks
 * - Color-coded bookmark categories
 * - Search/filter bookmarks
 */

import { Component, For, Show, createSignal, createMemo } from "solid-js";
import { getBasename } from "../utils/pathUtils";
import type { ProjectBookmark } from "../types/project";
import {
  HiOutlineBookmark,
  HiOutlineDocument,
  HiOutlineMagnifyingGlass,
  HiOutlineTrash,
  HiOutlineTag,
  HiOutlinePencilSquare,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineMapPin,
  HiOutlineXMark,
  HiOutlineFunnel,
} from "./icons";

// =============================================================================
// Types
// =============================================================================

export interface BookmarksPanelProps {
  /** Bookmarks from the project */
  bookmarks: ProjectBookmark[];
  /** Handler to navigate to a bookmarked item */
  onNavigate?: (bookmark: ProjectBookmark) => void;
  /** Handler to remove a bookmark */
  onRemove?: (bookmarkId: string) => void;
  /** Handler to edit a bookmark (legacy - opens inline editor) */
  onEdit?: (bookmark: ProjectBookmark) => void;
  /** Handler to update a bookmark's properties */
  onUpdate?: (bookmarkId: string, updates: Partial<Pick<ProjectBookmark, 'name' | 'color' | 'tags' | 'notes'>>) => void;
  /** Loading state */
  loading?: boolean;
  /** Whether the panel is compact (sidebar mode) */
  compact?: boolean;
}

type BookmarkTargetType = ProjectBookmark["target_type"];

// =============================================================================
// Helper Functions
// =============================================================================

const getTargetTypeIcon = (type: BookmarkTargetType) => {
  switch (type) {
    case "file":
      return HiOutlineDocument;
    case "artifact":
      return HiOutlineTag;
    case "search_result":
      return HiOutlineDocumentMagnifyingGlass;
    case "location":
      return HiOutlineMapPin;
    default:
      return HiOutlineBookmark;
  }
};

const getTargetTypeLabel = (type: BookmarkTargetType): string => {
  switch (type) {
    case "file":
      return "Files";
    case "artifact":
      return "Artifacts";
    case "search_result":
      return "Search Results";
    case "location":
      return "Locations";
    default:
      return "Other";
  }
};

const getBookmarkColorClass = (color?: string): string => {
  if (!color) return "text-accent";
  const colorMap: Record<string, string> = {
    red: "text-error",
    yellow: "text-warning",
    green: "text-success",
    blue: "text-info",
    purple: "text-accent",
    orange: "text-type-ad1",
  };
  return colorMap[color.toLowerCase()] || "text-accent";
};

// =============================================================================
// Bookmark Item Component
// =============================================================================

interface BookmarkItemProps {
  bookmark: ProjectBookmark;
  onNavigate?: (bookmark: ProjectBookmark) => void;
  onRemove?: (bookmarkId: string) => void;
  onEdit?: (bookmark: ProjectBookmark) => void;
  compact?: boolean;
}

const BookmarkItem: Component<BookmarkItemProps> = (props) => {
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
                <span class="text-[10px] px-1.5 py-0.5 bg-bg-secondary text-txt-muted rounded">
                  {tag}
                </span>
              )}
            </For>
            <Show when={props.bookmark.tags!.length > 3}>
              <span class="text-[10px] text-txt-muted">
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

// =============================================================================
// Bookmark Group Component
// =============================================================================

interface BookmarkGroupProps {
  type: BookmarkTargetType;
  bookmarks: ProjectBookmark[];
  onNavigate?: (bookmark: ProjectBookmark) => void;
  onRemove?: (bookmarkId: string) => void;
  onEdit?: (bookmark: ProjectBookmark) => void;
  compact?: boolean;
  defaultExpanded?: boolean;
}

const BookmarkGroup: Component<BookmarkGroupProps> = (props) => {
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

// =============================================================================
// Bookmark Edit Dialog
// =============================================================================

interface BookmarkEditDialogProps {
  bookmark: ProjectBookmark;
  onSave: (bookmarkId: string, updates: Partial<Pick<ProjectBookmark, 'name' | 'color' | 'tags' | 'notes'>>) => void;
  onCancel: () => void;
}

const BOOKMARK_COLORS = [
  { value: "", label: "Default", class: "bg-accent" },
  { value: "red", label: "Red", class: "bg-error" },
  { value: "yellow", label: "Yellow", class: "bg-warning" },
  { value: "green", label: "Green", class: "bg-success" },
  { value: "blue", label: "Blue", class: "bg-info" },
  { value: "purple", label: "Purple", class: "bg-accent" },
  { value: "orange", label: "Orange", class: "bg-type-ad1" },
];

const BookmarkEditDialog: Component<BookmarkEditDialogProps> = (props) => {
  const [name, setName] = createSignal(props.bookmark.name);
  const [color, setColor] = createSignal(props.bookmark.color || "");
  const [notes, setNotes] = createSignal(props.bookmark.notes || "");
  const [tagsInput, setTagsInput] = createSignal((props.bookmark.tags || []).join(", "));

  const handleSave = () => {
    const tags = tagsInput()
      .split(",")
      .map(t => t.trim())
      .filter(t => t.length > 0);

    props.onSave(props.bookmark.id, {
      name: name(),
      color: color() || undefined,
      tags: tags.length > 0 ? tags : undefined,
      notes: notes() || undefined,
    });
  };

  return (
    <div class="fixed inset-0 z-modal-backdrop flex items-center justify-center" onClick={(e) => { if (e.target === e.currentTarget) props.onCancel(); }}>
      <div class="bg-bg-panel border border-border rounded-xl shadow-xl w-[400px] max-w-[90vw] animate-fade-in" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div class="flex items-center justify-between px-5 py-4 border-b border-border">
          <div class="flex items-center gap-2">
            <HiOutlinePencilSquare class="w-5 h-5 text-accent" />
            <h3 class="text-sm font-semibold text-txt">Edit Bookmark</h3>
          </div>
          <button onClick={props.onCancel} class="icon-btn-sm">
            <HiOutlineXMark class="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div class="p-5 space-y-4">
          {/* Name */}
          <div class="form-group">
            <label class="label">Name</label>
            <input
              class="input"
              type="text"
              value={name()}
              onInput={(e) => setName(e.currentTarget.value)}
              placeholder="Bookmark name"
              autofocus
              onKeyDown={(e) => { if (e.key === "Enter") handleSave(); if (e.key === "Escape") props.onCancel(); }}
            />
          </div>

          {/* Color */}
          <div class="form-group">
            <label class="label">Color</label>
            <div class="flex items-center gap-2 flex-wrap">
              <For each={BOOKMARK_COLORS}>
                {(c) => (
                  <button
                    onClick={() => setColor(c.value)}
                    class={`w-6 h-6 rounded-full border-2 transition-all ${c.class} ${
                      color() === c.value ? "border-txt ring-2 ring-accent/50 scale-110" : "border-transparent hover:scale-105"
                    }`}
                    title={c.label}
                  />
                )}
              </For>
            </div>
          </div>

          {/* Tags */}
          <div class="form-group">
            <label class="label">Tags <span class="text-txt-muted font-normal">(comma-separated)</span></label>
            <input
              class="input"
              type="text"
              value={tagsInput()}
              onInput={(e) => setTagsInput(e.currentTarget.value)}
              placeholder="evidence, critical, review"
            />
          </div>

          {/* Notes */}
          <div class="form-group">
            <label class="label">Notes</label>
            <textarea
              class="textarea"
              rows="3"
              value={notes()}
              onInput={(e) => setNotes(e.currentTarget.value)}
              placeholder="Add notes about this bookmark..."
            />
          </div>
        </div>

        {/* Footer */}
        <div class="flex items-center justify-end gap-2 px-5 py-4 border-t border-border">
          <button class="btn btn-secondary" onClick={props.onCancel}>
            Cancel
          </button>
          <button class="btn btn-primary" onClick={handleSave} disabled={!name().trim()}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
};

// =============================================================================
// Main BookmarksPanel Component
// =============================================================================

export const BookmarksPanel: Component<BookmarksPanelProps> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal("");
  const [filterType, setFilterType] = createSignal<BookmarkTargetType | "all">("all");
  const [showFilter, setShowFilter] = createSignal(false);
  const [editingBookmark, setEditingBookmark] = createSignal<ProjectBookmark | null>(null);

  // Handle edit: if onUpdate is provided, open inline editor; otherwise fall back to onEdit
  const handleEditBookmark = (bookmark: ProjectBookmark) => {
    if (props.onUpdate) {
      setEditingBookmark(bookmark);
    } else {
      props.onEdit?.(bookmark);
    }
  };

  const handleSaveEdit = (bookmarkId: string, updates: Partial<Pick<ProjectBookmark, 'name' | 'color' | 'tags' | 'notes'>>) => {
    props.onUpdate?.(bookmarkId, updates);
    setEditingBookmark(null);
  };

  // Filter and group bookmarks
  const filteredBookmarks = createMemo(() => {
    let result = props.bookmarks || [];

    // Apply search filter
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

    // Apply type filter
    if (filterType() !== "all") {
      result = result.filter((b) => b.target_type === filterType());
    }

    return result;
  });

  // Group bookmarks by type
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

  // Check if there are any bookmarks in a group
  const hasBookmarksInGroup = (type: BookmarkTargetType) =>
    groupedBookmarks()[type].length > 0;

  const targetTypes: BookmarkTargetType[] = ["file", "artifact", "search_result", "location"];

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
          {/* Search Input */}
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

          {/* Type Filter */}
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
            <For each={targetTypes}>
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
            <For each={targetTypes}>
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
