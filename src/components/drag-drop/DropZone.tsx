// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { HiOutlineFolder, HiOutlineArrowDownTray } from "../icons";
import { useDragDrop } from "./useDragDrop";
import type { DropZoneProps } from "./types";

/**
 * Drop Zone component for file uploads
 */
export function DropZone(props: DropZoneProps) {
  let containerRef: HTMLDivElement | undefined;

  const { isDragging, isOver } = useDragDrop(() => containerRef, {
    accept: props.accept,
    multiple: props.multiple,
    disabled: props.disabled,
    onDrop: props.onDrop,
  });

  // Also support click to browse
  let inputRef: HTMLInputElement | undefined;

  const handleClick = () => {
    if (!props.disabled) {
      inputRef?.click();
    }
  };

  const handleFileSelect = (e: Event) => {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      const files = Array.from(input.files);
      props.onDrop(files);
      input.value = ""; // Reset for next selection
    }
  };

  return (
    <div
      ref={containerRef}
      class={`
        relative rounded-lg border-2 border-dashed transition-all duration-200 cursor-pointer
        ${
          isOver()
            ? "border-accent bg-accent/10 scale-[1.02]"
            : isDragging()
              ? "border-accent/50 bg-accent/5"
              : "border-border hover:border-border-subtle bg-bg-panel/30 hover:bg-bg-panel/50"
        }
        ${props.disabled ? "opacity-50 cursor-not-allowed" : ""}
        ${props.class ?? ""}
      `}
      onClick={handleClick}
      role="button"
      aria-label="Drop files here or click to browse"
      tabIndex={props.disabled ? -1 : 0}
      onKeyDown={(e) => e.key === "Enter" && handleClick()}
    >
      {/* Hidden file input */}
      <input
        ref={inputRef}
        type="file"
        class="hidden"
        accept={props.accept?.join(",")}
        multiple={props.multiple}
        onChange={handleFileSelect}
        disabled={props.disabled}
      />

      {/* Content */}
      <Show
        when={props.children}
        fallback={
          <div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
            <div
              class={`text-4xl transition-transform duration-200 ${isOver() ? "scale-125" : ""}`}
            >
              {props.icon ?? <HiOutlineFolder class="w-10 h-10 text-accent" />}
            </div>
            <div>
              <p class="text-sm font-medium text-txt">
                {props.title ?? "Drop files here"}
              </p>
              <p class="text-xs text-txt-secondary mt-1">
                {props.description ?? "or click to browse"}
              </p>
            </div>
            <Show when={props.accept && props.accept.length > 0}>
              <p class="text-xs text-txt-muted">
                Accepted: {props.accept!.join(", ")}
              </p>
            </Show>
          </div>
        }
      >
        {props.children}
      </Show>

      {/* Drag overlay */}
      <Show when={isOver()}>
        <div class="absolute inset-0 rounded-lg bg-accent/20 flex items-center justify-center pointer-events-none">
          <div class="text-accent font-medium flex items-center gap-2">
            <HiOutlineArrowDownTray class="w-6 h-6" />
            <span>Drop to add</span>
          </div>
        </div>
      </Show>
    </div>
  );
}
