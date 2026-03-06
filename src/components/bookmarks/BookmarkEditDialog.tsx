// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For, createSignal } from "solid-js";
import {
  HiOutlinePencilSquare,
  HiOutlineXMark,
} from "../icons";
import type { BookmarkEditDialogProps } from "./types";
import { BOOKMARK_COLORS } from "./helpers";

export const BookmarkEditDialog: Component<BookmarkEditDialogProps> = (props) => {
  const [name, setName] = createSignal(props.bookmark.name);
  const [color, setColor] = createSignal(props.bookmark.color || "");
  const [notes, setNotes] = createSignal(props.bookmark.notes || "");
  const [tagsInput, setTagsInput] = createSignal((props.bookmark.tags || []).join(", "));

  const handleSave = () => {
    const tags = tagsInput()
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0);

    props.onSave(props.bookmark.id, {
      name: name(),
      color: color() || undefined,
      tags: tags.length > 0 ? tags : undefined,
      notes: notes() || undefined,
    });
  };

  return (
    <div
      class="fixed inset-0 z-modal-backdrop flex items-center justify-center"
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onCancel();
      }}
    >
      <div
        class="bg-bg-panel border border-border rounded-xl shadow-xl w-[400px] max-w-[90vw] animate-fade-in"
        onClick={(e) => e.stopPropagation()}
      >
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
              onKeyDown={(e) => {
                if (e.key === "Enter") handleSave();
                if (e.key === "Escape") props.onCancel();
              }}
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
                      color() === c.value
                        ? "border-txt ring-2 ring-accent/50 scale-110"
                        : "border-transparent hover:scale-105"
                    }`}
                    title={c.label}
                  />
                )}
              </For>
            </div>
          </div>

          {/* Tags */}
          <div class="form-group">
            <label class="label">
              Tags <span class="text-txt-muted font-normal">(comma-separated)</span>
            </label>
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
