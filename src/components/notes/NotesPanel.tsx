// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * NotesPanel — Displays and manages project notes.
 *
 * Sub-components:
 *   - NoteItem.tsx (individual note row)
 *   - NoteEditDialog.tsx (create/edit modal)
 *   - helpers.ts (icon/label/formatting utilities)
 *   - types.ts (shared interfaces)
 */

import { Component, For, Show, createSignal, createMemo } from "solid-js";
import type { ProjectNote } from "../../types/project";
import {
  HiOutlineDocumentText,
  HiOutlineMagnifyingGlass,
  HiOutlineXMark,
  HiOutlineFunnel,
  HiOutlinePlusCircle,
} from "../icons";
import type { NotesPanelProps, NoteTargetType } from "./types";
import { getNoteTargetTypeLabel } from "./helpers";
import { NoteItem } from "./NoteItem";
import { NoteEditDialog } from "./NoteEditDialog";

const TARGET_TYPES: NoteTargetType[] = ["file", "artifact", "database", "case", "general"];

export const NotesPanel: Component<NotesPanelProps> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal("");
  const [filterType, setFilterType] = createSignal<NoteTargetType | "all">("all");
  const [showFilter, setShowFilter] = createSignal(false);
  const [editingNote, setEditingNote] = createSignal<ProjectNote | null | "new">(null);

  const filteredNotes = createMemo(() => {
    let result = props.notes || [];

    const query = searchQuery().toLowerCase();
    if (query) {
      result = result.filter(
        (n) =>
          n.title.toLowerCase().includes(query) ||
          n.content.toLowerCase().includes(query) ||
          n.target_path?.toLowerCase().includes(query) ||
          n.tags?.some((t) => t.toLowerCase().includes(query))
      );
    }

    if (filterType() !== "all") {
      result = result.filter((n) => n.target_type === filterType());
    }

    // Sort by modified_at descending (most recent first)
    return [...result].sort((a, b) => {
      const aDate = a.modified_at || a.created_at;
      const bDate = b.modified_at || b.created_at;
      return bDate.localeCompare(aDate);
    });
  });

  const handleEditNote = (note: ProjectNote) => {
    setEditingNote(note);
  };

  const handleCreateNote = () => {
    setEditingNote("new");
  };

  const handleSaveNote = (
    noteId: string | null,
    data: { title: string; content: string; tags?: string[]; priority?: ProjectNote["priority"] }
  ) => {
    if (noteId) {
      // Editing existing note
      props.onUpdate?.(noteId, data);
    } else {
      // Creating new note
      props.onCreate?.({
        target_type: "general",
        title: data.title,
        content: data.content,
        priority: data.priority,
      });
    }
    setEditingNote(null);
  };

  return (
    <div class="flex flex-col h-full bg-bg-panel">
      {/* Header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-border">
        <div class="flex items-center gap-2">
          <HiOutlineDocumentText class="w-4 h-4 text-accent" />
          <span class="text-sm font-medium text-txt">Notes</span>
          <Show when={props.notes.length > 0}>
            <span class="text-xs text-txt-muted">({props.notes.length})</span>
          </Show>
        </div>
        <div class="flex items-center gap-1">
          <button
            onClick={handleCreateNote}
            class="p-1 rounded transition-colors hover:bg-bg-hover text-txt-muted hover:text-accent"
            title="New note"
          >
            <HiOutlinePlusCircle class="w-4 h-4" />
          </button>
          <button
            onClick={() => setShowFilter(!showFilter())}
            class={`p-1 rounded transition-colors ${
              showFilter() ? "bg-accent/20 text-accent" : "hover:bg-bg-hover text-txt-muted"
            }`}
            title="Filter notes"
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
              placeholder="Search notes..."
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
                  {getNoteTargetTypeLabel(type)}
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

        <Show when={!props.loading && props.notes.length === 0}>
          <div class="flex flex-col items-center justify-center py-12 px-4">
            <HiOutlineDocumentText class="w-12 h-12 text-txt-muted/30 mb-3" />
            <p class="text-sm text-txt-muted text-center">No notes yet</p>
            <p class="text-xs text-txt-muted/70 text-center mt-1">
              Click + to create a note, or right-click files to add notes
            </p>
            <button
              onClick={handleCreateNote}
              class="mt-3 btn-sm btn-text"
            >
              Create your first note
            </button>
          </div>
        </Show>

        <Show when={!props.loading && props.notes.length > 0 && filteredNotes().length === 0}>
          <div class="flex flex-col items-center justify-center py-12 px-4">
            <HiOutlineMagnifyingGlass class="w-10 h-10 text-txt-muted/30 mb-3" />
            <p class="text-sm text-txt-muted text-center">No matching notes</p>
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

        <Show when={!props.loading && filteredNotes().length > 0}>
          <div>
            <For each={filteredNotes()}>
              {(note) => (
                <NoteItem
                  note={note}
                  onNavigate={props.onNavigate}
                  onRemove={props.onRemove}
                  onEdit={handleEditNote}
                />
              )}
            </For>
          </div>
        </Show>
      </div>

      {/* Edit/Create Dialog */}
      <Show when={editingNote() !== null}>
        <NoteEditDialog
          note={editingNote() === "new" ? null : (editingNote() as ProjectNote)}
          onSave={handleSaveNote}
          onCancel={() => setEditingNote(null)}
        />
      </Show>
    </div>
  );
};

export default NotesPanel;
