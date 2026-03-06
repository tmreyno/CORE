// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * HexViewerComponent — slim render shell for the hex viewer.
 * All state and logic live in useHexData hook.
 */

import { For, Show } from "solid-js";
import { byteToHex, formatBytes } from "../../utils";
import { HexToolbar } from "./HexToolbar";
import { HexLine } from "./HexLine";
import { useHexData } from "./useHexData";
import { BYTES_PER_LINE, NAVIGATED_COLOR } from "./constants";
import type { HexViewerProps } from "./types";

// Re-export viewer types for backward compatibility
export type { FileChunk, HeaderRegion, MetadataField, ParsedMetadata, FileTypeInfo } from "../../types";

export function HexViewer(props: HexViewerProps) {
  const hex = useHexData({
    file: () => props.file ?? null,
    entry: () => props.entry,
    onMetadataLoaded: props.onMetadataLoaded,
    onNavigatorReady: props.onNavigatorReady,
  });

  return (
    <div
      class="flex flex-col h-full bg-bg text-txt font-mono text-sm"
      tabIndex={0}
      onKeyDown={hex.handleKeyDown}
    >
      <HexToolbar
        fileType={hex.fileType()}
        fileSize={hex.totalFileSize()}
        loadedBytes={hex.loadedUpTo()}
        loadProgress={hex.loadProgress()}
        loading={hex.loading()}
        loadingMore={hex.loadingMore()}
        canLoadMore={hex.canLoadMore()}
        maxLoadedBytes={hex.maxLoadedBytes()}
        gotoOffset={hex.gotoOffset()}
        showAscii={hex.showAscii()}
        highlightRegions={hex.highlightRegions()}
        hasRegions={hex.hasRegions()}
        regions={hex.metadataRegions()}
        onLoadMore={hex.loadMoreData}
        onGotoOffset={hex.handleGotoOffset}
        onSetGotoOffset={hex.setGotoOffset}
        onToggleAscii={hex.setShowAscii}
        onToggleHighlight={hex.setHighlightRegions}
        onSelectRegion={hex.handleSelectRegion}
      />

      <Show when={hex.error()}>
        <div class="px-3 py-2 text-sm text-red-400 bg-red-900/20 border-b border-red-500/30">
          {hex.error()}
        </div>
      </Show>

      <Show when={hex.loading()}>
        <div class="flex items-center justify-center py-8 text-txt-secondary">Loading...</div>
      </Show>

      <Show when={!hex.loading() && hex.loadedBytes().length > 0}>
        <div
          ref={hex.setScrollContainerRef}
          class="flex-1 overflow-auto p-2"
          onScroll={hex.handleScroll}
        >
          <div class="flex items-center gap-0 text-[10px] leading-tight text-txt-muted pb-1 border-b border-border/50 mb-1 sticky top-0 bg-bg z-10">
            <Show when={hex.showAddress()}>
              <span class="w-20 shrink-0">Offset</span>
            </Show>
            <span class="flex gap-0">
              <For each={[...Array(BYTES_PER_LINE).keys()]}>
                {(i) => (
                  <span class="w-[22px] text-center">{byteToHex(i)}</span>
                )}
              </For>
            </span>
            <Show when={hex.showAscii()}>
              <span class="ml-2 w-32">ASCII</span>
            </Show>
          </div>

          <div class="flex flex-col">
            <For each={hex.hexLines()}>
              {(line) => (
                <HexLine
                  line={line}
                  showAddress={hex.showAddress()}
                  showAscii={hex.showAscii()}
                  hoveredOffset={hex.hoveredOffset()}
                  selectedRegion={hex.selectedRegion()}
                  navigatedRange={hex.navigatedRange()}
                  navigatedColor={NAVIGATED_COLOR}
                  onHoverByte={hex.setHoveredOffset}
                  onClearNavigation={() => hex.setNavigatedRange(null)}
                />
              )}
            </For>
          </div>

          <Show when={hex.loadingMore()}>
            <div class="flex items-center justify-center py-4 text-txt-muted text-xs">
              Loading more...
            </div>
          </Show>
          <Show when={!hex.loadingMore() && hex.loadedUpTo() >= hex.totalFileSize()}>
            <div class="flex items-center justify-center py-4 text-txt-muted text-xs">
              — End of file —
            </div>
          </Show>
          <Show
            when={
              !hex.loadingMore() &&
              hex.loadedUpTo() >= hex.maxLoadedBytes() &&
              hex.totalFileSize() > hex.maxLoadedBytes()
            }
          >
            <div class="flex items-center justify-center py-4 text-amber-500/70 text-xs">
              — Maximum preview size reached ({formatBytes(hex.maxLoadedBytes())}) —
            </div>
          </Show>
        </div>
      </Show>

      <Show when={!hex.loading() && hex.loadedBytes().length === 0 && !hex.error()}>
        <div class="flex items-center justify-center py-8 text-txt-muted">
          Select a file to view its contents
        </div>
      </Show>
    </div>
  );
}
