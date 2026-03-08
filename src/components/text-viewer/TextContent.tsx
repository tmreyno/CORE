// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { formatBytes } from "../../utils";
import type { UseTextViewerReturn } from "./useTextViewer";

interface TextContentProps {
  tv: UseTextViewerReturn;
  contentRef: (el: HTMLDivElement) => void;
  onScroll: () => void;
}

export function TextContent(props: TextContentProps) {
  const { tv } = props;

  return (
    <div
      ref={props.contentRef}
      class={`flex-1 overflow-auto font-mono ${tv.wordWrap() ? "whitespace-pre-wrap" : "whitespace-pre"}`}
      style={{ "font-size": `${tv.fontSize()}px` }}
      onScroll={props.onScroll}
    >
      <div class="flex">
        <div
          class="flex flex-col text-right pr-3 mr-3 border-r border-border bg-bg sticky left-0 select-none line-numbers-column"
          classList={{ hidden: !tv.showLineNumbers() }}
        >
          {tv.lines().map((_: string, i: number) => (
            <div class="text-txt-muted leading-relaxed">{i + 1}</div>
          ))}
        </div>
        <pre class="flex-1 text-txt-secondary">
          <code class={`language-${tv.detectLanguage()}`}>{tv.content()}</code>
        </pre>
      </div>

      {/* Loading more indicator */}
      <Show when={tv.loadingMore()}>
        <div class="flex items-center justify-center py-4 text-txt-muted text-xs">Loading more...</div>
      </Show>

      {/* End of file indicator */}
      <Show when={!tv.loadingMore() && tv.loadedChars() >= tv.totalSize()}>
        <div class="flex items-center justify-center py-4 text-txt-muted text-xs">— End of file —</div>
      </Show>

      {/* Max loaded indicator */}
      <Show when={!tv.loadingMore() && tv.loadedChars() >= tv.maxLoadedChars() && tv.totalSize() > tv.maxLoadedChars()}>
        <div class="flex items-center justify-center py-4 text-amber-500/70 text-xs">
          — Maximum preview size reached ({formatBytes(tv.maxLoadedChars())}) —
        </div>
      </Show>
    </div>
  );
}
