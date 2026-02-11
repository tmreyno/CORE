// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For } from "solid-js";
import type { FileTypeInfo, HeaderRegion } from "../../types";
import { formatOffset, formatBytes } from "../../utils";

interface HexToolbarProps {
  fileType: FileTypeInfo | null;
  fileSize: number;
  loadedBytes: number;
  loadProgress: number;
  loading: boolean;
  loadingMore: boolean;
  canLoadMore: boolean;
  maxLoadedBytes: number;
  gotoOffset: string;
  showAscii: boolean;
  highlightRegions: boolean;
  hasRegions: boolean;
  regions: HeaderRegion[];
  onLoadMore: () => void;
  onGotoOffset: () => void;
  onSetGotoOffset: (value: string) => void;
  onToggleAscii: (checked: boolean) => void;
  onToggleHighlight: (checked: boolean) => void;
  onSelectRegion: (index: number) => void;
}

export const HexToolbar: Component<HexToolbarProps> = (props) => {
  const fileSizeText = () => (props.fileSize > 0 ? formatBytes(props.fileSize) : "");
  
  const loadedBytesText = () => {
    const loaded = props.loadedBytes;
    const total = props.fileSize;
    const progress = props.loadProgress;
    return loaded < total ? `${formatBytes(loaded)} (${progress}%)` : formatBytes(loaded);
  };

  return (
    <div class="flex items-center gap-2 px-3 py-2 bg-bg-panel border-b border-border flex-wrap">
      <div class="flex items-center gap-2">
        <Show when={props.fileType}>
          {type => (
            <span class="text-xs text-accent px-2 py-0.5 bg-bg rounded" title={type().magic_hex}>
              {type().description}
            </span>
          )}
        </Show>
        <Show when={fileSizeText()}>
          <span class="text-xs text-txt-secondary">{fileSizeText()}</span>
        </Show>
      </div>
      
      <div class="flex items-center gap-2 ml-auto mr-auto">
        <span class="text-xs text-txt-secondary">
          Loaded: {loadedBytesText()}
        </span>
        <Show when={props.canLoadMore}>
          <button
            class="px-2 py-0.5 text-xs bg-bg-hover hover:bg-bg-active rounded text-txt-tertiary"
            onClick={props.onLoadMore}
            disabled={props.loadingMore}
          >
            {props.loadingMore ? "Loading..." : "Load More"}
          </button>
        </Show>
        <Show when={props.loadedBytes >= props.maxLoadedBytes && props.fileSize > props.maxLoadedBytes}>
          <span class="text-xs text-amber-400">Max preview reached</span>
        </Show>
      </div>
      
      <div class="flex items-center gap-2">
        <input
          type="text"
          class="w-32 px-2 py-1 text-xs bg-bg border border-border rounded text-txt placeholder-txt-muted focus:border-accent focus:outline-none"
          placeholder="Go to offset (hex: 0x...)"
          value={props.gotoOffset}
          onInput={e => props.onSetGotoOffset(e.currentTarget.value)}
          onKeyDown={e => e.key === "Enter" && props.onGotoOffset()}
        />
        <button
          class="px-2 py-1 text-xs bg-accent hover:bg-accent-hover rounded text-white"
          onClick={props.onGotoOffset}
        >
          Go
        </button>
        
        <label class="label-with-icon">
          <input
            type="checkbox"
            class="w-3 h-3 accent-accent"
            checked={props.showAscii}
            onChange={e => props.onToggleAscii(e.currentTarget.checked)}
          />
          ASCII
        </label>
        <label class="label-with-icon">
          <input
            type="checkbox"
            class="w-3 h-3 accent-accent"
            checked={props.highlightRegions}
            onChange={e => props.onToggleHighlight(e.currentTarget.checked)}
          />
          Highlight
        </label>
        
        <Show when={props.highlightRegions && props.hasRegions}>
          <select
            class="px-2 py-1 text-xs bg-bg border border-border rounded text-txt focus:border-accent focus:outline-none"
            onChange={e => {
              const idx = parseInt(e.currentTarget.value);
              if (!isNaN(idx)) {
                props.onSelectRegion(idx);
              }
              e.currentTarget.value = "";
            }}
          >
            <option value="">Jump to region...</option>
            <For each={props.regions}>
              {(region, idx) => (
                <option value={idx()}>
                  {region.name} (0x{formatOffset(region.start, { width: 4 })})
                </option>
              )}
            </For>
          </select>
        </Show>
      </div>
    </div>
  );
};
