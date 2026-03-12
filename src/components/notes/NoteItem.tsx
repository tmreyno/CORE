// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, createSignal } from "solid-js";
import { Dynamic } from "solid-js/web";
import {
  HiOutlineTrash,
  HiOutlinePencilSquare,
  HiOutlineArrowTopRightOnSquare,
} from "../icons";
import type { NoteItemProps } from "./types";
import { getNoteTargetTypeIcon, getPriorityColor, getPriorityLabel, formatNoteDate } from "./helpers";
import { getBasename } from "../../utils/pathUtils";

export const NoteItem: Component<NoteItemProps> = (props) => {
  const [showActions, setShowActions] = createSignal(false);

  const targetName = () => {
    if (!props.note.target_path) return null;
    return getBasename(props.note.target_path) || props.note.target_path;
  };

  const Icon = getNoteTargetTypeIcon(props.note.target_type);

  return (
    <div
      class="group px-3 py-2 hover:bg-bg-hover transition-colors cursor-pointer border-b border-border/20"
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
      onClick={() => props.onEdit?.(props.note)}
    >
      {/* Title row */}
      <div class="flex items-center gap-2 min-w-0">
        <Dynamic component={Icon} class="w-3.5 h-3.5 text-txt-muted shrink-0" />
        <span class="text-sm text-txt truncate flex-1 font-medium">
          {props.note.title}
        </span>
        <Show when={props.note.priority && props.note.priority !== "normal"}>
          <span class={`text-2xs font-medium ${getPriorityColor(props.note.priority)}`}>
            {getPriorityLabel(props.note.priority)}
          </span>
        </Show>
      </div>

      {/* Content preview */}
      <Show when={props.note.content}>
        <p class="text-xs text-txt-muted mt-1 line-clamp-2 pl-5.5">
          {props.note.content.slice(0, 120)}
          {props.note.content.length > 120 ? "…" : ""}
        </p>
      </Show>

      {/* Meta row */}
      <div class="flex items-center gap-2 mt-1 pl-5.5">
        <Show when={targetName()}>
          <span class="text-2xs text-txt-muted truncate max-w-[140px]" title={props.note.target_path}>
            {targetName()}
          </span>
          <span class="text-2xs text-txt-muted">·</span>
        </Show>
        <span class="text-2xs text-txt-muted">
          {formatNoteDate(props.note.modified_at || props.note.created_at)}
        </span>

        {/* Action buttons */}
        <Show when={showActions()}>
          <div class="flex items-center gap-1 ml-auto">
            <Show when={props.note.target_path && props.onNavigate}>
              <button
                onClick={(e) => { e.stopPropagation(); props.onNavigate?.(props.note); }}
                class="p-0.5 rounded hover:bg-bg-secondary transition-colors"
                title="Go to target"
              >
                <HiOutlineArrowTopRightOnSquare class="w-3.5 h-3.5 text-txt-muted" />
              </button>
            </Show>
            <button
              onClick={(e) => { e.stopPropagation(); props.onEdit?.(props.note); }}
              class="p-0.5 rounded hover:bg-bg-secondary transition-colors"
              title="Edit note"
            >
              <HiOutlinePencilSquare class="w-3.5 h-3.5 text-txt-muted" />
            </button>
            <button
              onClick={(e) => { e.stopPropagation(); props.onRemove?.(props.note.id); }}
              class="p-0.5 rounded hover:bg-bg-secondary transition-colors"
              title="Delete note"
            >
              <HiOutlineTrash class="w-3.5 h-3.5 text-error/70 hover:text-error" />
            </button>
          </div>
        </Show>
      </div>
    </div>
  );
};
