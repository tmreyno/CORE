// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For, createSignal } from "solid-js";
import type { ProjectNote } from "../../types/project";
import {
  HiOutlinePencilSquare,
  HiOutlineXMark,
} from "../icons";
import type { NoteEditDialogProps } from "./types";
import { NOTE_PRIORITIES } from "./types";

export const NoteEditDialog: Component<NoteEditDialogProps> = (props) => {
  const isNew = () => props.note === null;
  const [title, setTitle] = createSignal(props.note?.title ?? "");
  const [content, setContent] = createSignal(props.note?.content ?? "");
  const [priority, setPriority] = createSignal<ProjectNote["priority"]>(props.note?.priority ?? "normal");
  const [tagsInput, setTagsInput] = createSignal((props.note?.tags || []).join(", "));

  const handleSave = () => {
    if (!title().trim()) return;
    const tags = tagsInput()
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0);

    props.onSave(props.note?.id ?? null, {
      title: title().trim(),
      content: content(),
      tags: tags.length > 0 ? tags : undefined,
      priority: priority(),
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
        class="bg-bg-panel border border-border rounded-xl shadow-xl w-[480px] max-w-[90vw] animate-fade-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div class="flex items-center justify-between px-5 py-4 border-b border-border">
          <div class="flex items-center gap-2">
            <HiOutlinePencilSquare class="w-5 h-5 text-accent" />
            <h3 class="text-sm font-semibold text-txt">
              {isNew() ? "New Note" : "Edit Note"}
            </h3>
          </div>
          <button onClick={props.onCancel} class="icon-btn-sm">
            <HiOutlineXMark class="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div class="p-5 space-y-4">
          {/* Title */}
          <div class="form-group">
            <label class="label">Title</label>
            <input
              class="input"
              type="text"
              value={title()}
              onInput={(e) => setTitle(e.currentTarget.value)}
              placeholder="Note title"
              autofocus
              onKeyDown={(e) => {
                if (e.key === "Escape") props.onCancel();
              }}
            />
          </div>

          {/* Content */}
          <div class="form-group">
            <label class="label">Content</label>
            <textarea
              class="textarea"
              rows={6}
              value={content()}
              onInput={(e) => setContent(e.currentTarget.value)}
              placeholder="Write your note here... (supports markdown)"
              onKeyDown={(e) => {
                if (e.key === "Escape") props.onCancel();
              }}
            />
          </div>

          {/* Priority */}
          <div class="form-group">
            <label class="label">Priority</label>
            <div class="flex items-center gap-2">
              <For each={NOTE_PRIORITIES}>
                {(p) => (
                  <button
                    onClick={() => setPriority(p.value)}
                    class={`px-3 py-1.5 text-xs rounded-lg border transition-all ${
                      priority() === p.value
                        ? "border-accent bg-accent/10 text-accent font-medium"
                        : "border-border bg-bg text-txt-muted hover:text-txt hover:border-border"
                    }`}
                  >
                    {p.label}
                  </button>
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
              placeholder="e.g. evidence, suspicious, followup"
              onKeyDown={(e) => {
                if (e.key === "Escape") props.onCancel();
              }}
            />
          </div>
        </div>

        {/* Footer */}
        <div class="flex items-center justify-end gap-2 px-5 py-4 border-t border-border">
          <button onClick={props.onCancel} class="btn btn-secondary">
            Cancel
          </button>
          <button
            onClick={handleSave}
            class="btn btn-primary"
            disabled={!title().trim()}
          >
            {isNew() ? "Create Note" : "Save Changes"}
          </button>
        </div>
      </div>
    </div>
  );
};
