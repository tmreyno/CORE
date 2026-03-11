// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import type { UseTextViewerReturn } from "./useTextViewer";

interface TextToolbarProps {
  tv: UseTextViewerReturn;
  onPrevResult: () => void;
  onNextResult: () => void;
}

export function TextToolbar(props: TextToolbarProps) {
  const { tv } = props;

  return (
    <div class="panel-header gap-4">
      <div class="row text-xs">
        <span class="text-accent">{tv.detectLanguage()}</span>
        <span class="text-txt-muted">
          {formatBytesInline(tv.loadedChars())}
          <Show when={tv.isTruncated()}>
            {" / " + formatBytesInline(tv.totalSize()) + " (truncated)"}
          </Show>
        </span>
        <span class="text-txt-muted">{tv.lineCount()} lines</span>
      </div>

      <div class="flex-1 flex justify-center">
        {/* Search */}
        <div class="flex items-center gap-1 bg-bg-panel rounded border border-border">
          <input
            type="text"
            class="w-40 px-2 py-1 text-xs bg-transparent text-txt outline-none"
            placeholder="Search..."
            value={tv.searchQuery()}
            onInput={(e) => tv.setSearchQuery(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.shiftKey ? props.onPrevResult() : props.onNextResult();
              }
            }}
          />
          <Show when={tv.searchQuery()}>
            <span class="text-2xs leading-tight text-txt-muted px-1">
              {tv.searchResults().length > 0
                ? `${tv.currentResult() + 1}/${tv.searchResults().length}`
                : "No results"}
            </span>
            <button
              class="px-1 text-xs text-txt-secondary hover:text-txt"
              onClick={props.onPrevResult}
              title="Previous (Shift+Enter)"
            >
              ▲
            </button>
            <button
              class="px-1 text-xs text-txt-secondary hover:text-txt"
              onClick={props.onNextResult}
              title="Next (Enter)"
            >
              ▼
            </button>
          </Show>
        </div>
      </div>

      <div class="row gap-3 text-xs">
        <label class="label-with-icon">
          <input
            type="checkbox"
            class="w-3.5 h-3.5 accent-accent"
            checked={tv.showLineNumbers()}
            onChange={(e) => tv.setShowLineNumbers(e.currentTarget.checked)}
          />
          Lines
        </label>
        <label class="label-with-icon">
          <input
            type="checkbox"
            class="w-3.5 h-3.5 accent-accent"
            checked={tv.wordWrap()}
            onChange={(e) => tv.setWordWrap(e.currentTarget.checked)}
          />
          Wrap
        </label>

        <div class="row gap-1 text-txt-secondary">
          <button
            class="px-1.5 py-0.5 rounded hover:bg-bg-hover hover:text-txt"
            onClick={() => tv.setFontSize((s) => Math.max(10, s - 1))}
          >
            −
          </button>
          <span class="w-8 text-center">{tv.fontSize()}px</span>
          <button
            class="px-1.5 py-0.5 rounded hover:bg-bg-hover hover:text-txt"
            onClick={() => tv.setFontSize((s) => Math.min(24, s + 1))}
          >
            +
          </button>
        </div>
      </div>
    </div>
  );
}

// Inline helper to avoid importing the full utils bundle in the toolbar.
// TextViewer's parent already imports formatBytes, so this is a local alias.
import { formatBytes } from "../../utils";
function formatBytesInline(n: number): string {
  return formatBytes(n);
}
