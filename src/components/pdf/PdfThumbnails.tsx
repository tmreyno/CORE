// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For } from "solid-js";

interface PdfThumbnailsProps {
  thumbnails: string[];
  currentPage: number;
  show: boolean;
  onSelectPage: (page: number) => void;
}

export const PdfThumbnails: Component<PdfThumbnailsProps> = (props) => {
  return (
    <Show when={props.show && props.thumbnails.length > 0}>
      <div class="w-32 shrink-0 border-r border-border overflow-y-auto bg-bg">
        <div class="p-2 space-y-2">
          <For each={props.thumbnails}>
            {(thumb, index) => (
              <button
                class={`w-full p-1 rounded border transition-colors ${
                  props.currentPage === index() + 1
                    ? "border-accent bg-accent/10"
                    : "border-border hover:border-border hover:bg-bg-panel/50"
                }`}
                onClick={() => props.onSelectPage(index() + 1)}
              >
                <Show
                  when={thumb}
                  fallback={
                    <div class="aspect-[3/4] bg-bg-panel flex items-center justify-center">
                      <span class="text-xs text-txt-muted">{index() + 1}</span>
                    </div>
                  }
                >
                  <img
                    src={thumb}
                    alt={`Page ${index() + 1}`}
                    class="w-full"
                  />
                </Show>
                <span class="text-2xs leading-tight text-txt-secondary mt-1 block">
                  {index() + 1}
                </span>
              </button>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
};
