// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { HiOutlineExclamationTriangle } from "../icons";
import { formatBytes } from "../../utils";
import type { TextViewerProps } from "./types";
import { useTextViewer } from "./useTextViewer";
import { TextToolbar } from "./TextToolbar";
import { TextContent } from "./TextContent";

export function TextViewer(props: TextViewerProps) {
  let contentRef: HTMLDivElement | undefined;
  const tv = useTextViewer(props);

  const handleNextResult = () => {
    tv.nextResult();
    tv.scrollToResult(contentRef);
  };

  const handlePrevResult = () => {
    tv.prevResult();
    tv.scrollToResult(contentRef);
  };

  return (
    <div class="flex flex-col h-full bg-bg text-sm">
      <TextToolbar tv={tv} onPrevResult={handlePrevResult} onNextResult={handleNextResult} />

      {/* Error display */}
      <Show when={tv.error()}>
        <div class="p-4 text-red-400 bg-red-900/20">{tv.error()}</div>
      </Show>

      {/* Loading indicator */}
      <Show when={tv.loading()}>
        <div class="flex-1 flex items-center justify-center text-txt-muted">Loading...</div>
      </Show>

      {/* Content */}
      <Show when={!tv.loading() && tv.content()}>
        <TextContent
          tv={tv}
          contentRef={(el) => (contentRef = el)}
          onScroll={() => contentRef && tv.handleScroll(contentRef)}
        />
      </Show>

      {/* Progress info bar */}
      <Show when={!tv.loading() && tv.isTruncated()}>
        <div class="flex items-center gap-2 px-3 py-2 text-xs text-amber-400 bg-amber-900/20 border-t border-border">
          <HiOutlineExclamationTriangle class="w-4 h-4" />
          <span>
            Loaded {formatBytes(tv.loadedChars())} of {formatBytes(tv.totalSize())} (
            {Math.round((tv.loadedChars() / tv.totalSize()) * 100)}%)
          </span>
          <Show when={tv.loadedChars() < tv.maxLoadedChars()}>
            <button
              class="px-2 py-0.5 bg-bg-hover hover:bg-bg-active rounded text-txt-tertiary"
              onClick={tv.loadMoreContent}
              disabled={tv.loadingMore()}
            >
              {tv.loadingMore() ? "Loading..." : "Load More"}
            </button>
          </Show>
          <Show when={tv.loadedChars() >= tv.maxLoadedChars()}>
            <span class="text-txt-muted">(max preview reached)</span>
          </Show>
        </div>
      </Show>

      {/* Empty state */}
      <Show when={!tv.loading() && !tv.content() && !tv.error()}>
        <div class="flex-1 flex items-center justify-center text-txt-muted">
          Select a text file to view its contents
        </div>
      </Show>
    </div>
  );
}
