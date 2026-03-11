// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, type Accessor } from "solid-js";
import {
  HiOutlineDocument,
  HiOutlineFolderOpen,
  HiOutlineXMark,
  HiOutlineInformationCircle,
} from "../../icons";
import type { CenterTab } from "./types";

interface SecondaryTabBarProps {
  entries: Accessor<CenterTab[]>;
  activeTabId: Accessor<string | null>;
  activeContainerTab: Accessor<CenterTab | null>;
  isViewingEntry: Accessor<boolean>;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
}

/** Entry-level tab bar for files inside the active container (second row) */
export function SecondaryTabBar(props: SecondaryTabBarProps) {
  return (
    <Show when={props.entries().length > 0}>
      <div class="flex items-center bg-bg-panel/50 border-b border-border/50 px-2 gap-1 shrink-0 h-7 min-h-[28px]">
        {/* Breadcrumb indicator */}
        <div class="flex items-center gap-1 text-2xs text-txt-muted mr-2 shrink-0">
          <HiOutlineFolderOpen class="w-3 h-3" />
          <span class="truncate max-w-[100px]">{props.activeContainerTab()?.title}</span>
          <span class="text-txt-muted/50">/</span>
        </div>
        
        {/* Entry tabs */}
        <div class="flex items-center gap-0.5 overflow-x-auto scrollbar-thin flex-1">
          <For each={props.entries()}>
            {(tab) => (
              <div
                class={`flex items-center gap-1 px-2 py-1 text-compact rounded transition-all duration-150 group cursor-pointer select-none ${
                  props.activeTabId() === tab.id
                    ? "bg-bg text-txt border border-border/50 shadow-sm"
                    : "text-txt-muted hover:text-txt hover:bg-bg-hover/70"
                }`}
                onClick={() => props.onSelect(tab.id)}
                title={tab.entry?.entryPath}
              >
                <HiOutlineDocument class="w-3 h-3 shrink-0" />
                <span class="truncate max-w-[120px]">{tab.title}</span>
                <button
                  class={`ml-0.5 p-0.5 rounded transition-all ${
                    props.activeTabId() === tab.id
                      ? "hover:bg-bg-hover opacity-60 hover:opacity-100"
                      : "hover:bg-bg-active opacity-0 group-hover:opacity-60 hover:!opacity-100"
                  }`}
                  onClick={(e) => {
                    e.stopPropagation();
                    props.onClose(tab.id);
                  }}
                  title="Close"
                >
                  <HiOutlineXMark class="w-2.5 h-2.5" />
                </button>
              </div>
            )}
          </For>
        </div>
        
        {/* Back to container info button */}
        <Show when={props.isViewingEntry() && props.activeContainerTab()}>
          <button
            class="flex items-center gap-1 px-2 py-1 text-2xs text-accent hover:text-accent-hover hover:bg-accent/10 rounded transition-colors ml-1"
            onClick={() => props.onSelect(props.activeContainerTab()!.id)}
            title="Back to container info"
          >
            <HiOutlineInformationCircle class="w-3 h-3" />
            <span>Info</span>
          </button>
        </Show>
      </div>
    </Show>
  );
}
