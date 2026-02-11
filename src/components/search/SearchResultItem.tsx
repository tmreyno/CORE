// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { HiOutlineFolder, HiOutlineDocument } from "../icons";
import { formatBytes } from "../../utils";
import type { SearchResult } from "../SearchPanel";

interface SearchResultItemProps {
  result: SearchResult;
  isSelected: boolean;
  onSelect: () => void;
  onMouseEnter: () => void;
}

export const SearchResultItem: Component<SearchResultItemProps> = (props) => {
  return (
    <button
      class={`w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors ${
        props.isSelected ? "bg-accent/20" : "hover:bg-bg-panel"
      }`}
      onClick={props.onSelect}
      onMouseEnter={props.onMouseEnter}
    >
      <span class="shrink-0">
        {props.result.isDir ? (
          <HiOutlineFolder class="w-5 h-5 text-amber-400" />
        ) : (
          <HiOutlineDocument class="w-5 h-5 text-txt-secondary" />
        )}
      </span>
      <div class="flex-1 min-w-0">
        <div class="text-sm text-txt truncate">{props.result.name}</div>
        <Show when={props.result.containerPath}>
          <div class="text-xs text-accent truncate flex items-center gap-1">
            <span class="px-1 py-0.5 bg-accent/20 rounded text-[10px] uppercase">
              {props.result.containerType || "container"}
            </span>
            <span class="truncate">
              {props.result.containerPath?.split("/").pop()}
            </span>
            <span class="text-txt-muted">→</span>
            <span class="truncate text-txt-secondary">{props.result.path}</span>
          </div>
        </Show>
        <Show when={!props.result.containerPath}>
          <div class="text-xs text-txt-muted truncate">{props.result.path}</div>
        </Show>
        <Show when={props.result.matchContext}>
          <div class="text-xs text-txt-secondary mt-0.5 truncate">
            ...{props.result.matchContext}...
          </div>
        </Show>
      </div>
      <span class="file-size">{formatBytes(props.result.size)}</span>
    </button>
  );
};
