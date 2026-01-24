// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SourceList Component
 * 
 * Displays the list of source files/folders to be transferred with
 * container detection badges and remove buttons.
 */

import { Show, For } from "solid-js";
import { getBasename } from "../../utils";
import {
  HiOutlineFolderOpen,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineArchiveBox,
  HiOutlineXMark,
} from "../icons";
import { detectContainerType } from "./utils";
import type { SourceListProps } from "./types";

export function SourceList(props: SourceListProps) {
  return (
    <div class="space-y-1.5">
      <div class="flex items-center justify-between">
        <label class={`text-[11px] leading-tight text-txt-secondary uppercase tracking-wider`}>Source</label>
        <div class="flex gap-1">
          <button
            onClick={props.onBrowseFolder}
            class={`px-1.5 py-0.5 text-[11px] leading-tight bg-bg-secondary rounded text-txt-tertiary hover:text-txt hover:bg-bg-hover transition-colors`}
            title="Add folder"
          >
            <HiOutlineFolder class="w-3 h-3" />
          </button>
          <button
            onClick={props.onBrowseFile}
            class={`px-1.5 py-0.5 text-[11px] leading-tight bg-bg-secondary rounded text-txt-tertiary hover:text-txt hover:bg-bg-hover transition-colors`}
            title="Add files"
          >
            <HiOutlineDocument class="w-3 h-3" />
          </button>
        </div>
      </div>
      
      <div class={`bg-bg-secondary rounded border border-border/50 min-h-[60px] max-h-[120px] overflow-y-auto`}>
        <Show when={props.sources.length > 0} fallback={
          <div class={`flex items-center justify-center h-[60px] text-[11px] leading-tight text-txt-muted`}>
            No sources selected. Project directory will be used by default.
          </div>
        }>
          <For each={props.sources}>
            {(path) => {
              const containerType = detectContainerType(path);
              const isContainer = containerType !== "unknown";
              const containerLabel = containerType.toUpperCase();
              return (
                <div class="flex items-center gap-2 px-2 py-1 hover:bg-bg-hover/50 group">
                  <Show when={isContainer} fallback={
                    <HiOutlineFolderOpen class={`w-3 h-3 text-txt-secondary flex-shrink-0`} />
                  }>
                    <HiOutlineArchiveBox class={`w-3 h-3 text-accent flex-shrink-0`} title={`${containerLabel} forensic container`} />
                  </Show>
                  <span class={`flex-1 text-[11px] leading-tight text-txt-tertiary truncate`} title={path}>
                    {getBasename(path) || path}
                  </span>
                  <Show when={isContainer}>
                    <span class={`text-[11px] leading-tight text-accent px-1 py-0.5 rounded bg-accent/10`}>
                      {containerLabel}
                    </span>
                  </Show>
                  <button
                    onClick={() => props.onRemoveSource(path)}
                    class="opacity-0 group-hover:opacity-100 text-txt-muted hover:text-red-400 transition-opacity"
                  >
                    <HiOutlineXMark class="w-3 h-3" />
                  </button>
                </div>
              );
            }}
          </For>
        </Show>
      </div>
    </div>
  );
}
