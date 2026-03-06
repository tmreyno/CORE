// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, type Accessor } from "solid-js";
import { TabItem } from "../TabItem";
import type { CenterTab } from "./types";

interface PrimaryTabBarProps {
  containerTabs: Accessor<CenterTab[]>;
  activeTabId: Accessor<string | null>;
  activeContainerTabId: Accessor<string | null | undefined>;
  hasMultipleTabs: Accessor<boolean>;
  tabCount: Accessor<number>;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
}

/** Container-level tab bar (top row) */
export function PrimaryTabBar(props: PrimaryTabBarProps) {
  return (
    <div class="flex items-center bg-bg-secondary border-b border-border px-2 gap-1 shrink-0 h-9 min-h-[36px]">
      {/* Tab count indicator */}
      <Show when={props.hasMultipleTabs()}>
        <span
          class="flex items-center justify-center min-w-[18px] h-4 px-1 text-[10px] font-medium text-txt-muted bg-bg-hover rounded mr-1"
          title={`${props.tabCount()} open tabs`}
        >
          {props.tabCount()}
        </span>
      </Show>
      
      {/* Container Tabs - scrollable container */}
      <div class="flex items-center gap-0.5 overflow-x-auto scrollbar-thin flex-1 py-0.5">
        <For each={props.containerTabs()}>
          {(tab) => (
            <TabItem
              tab={tab}
              isActive={
                props.activeTabId() === tab.id || 
                (tab.type === "evidence" && props.activeContainerTabId() === tab.id)
              }
              onSelect={() => props.onSelect(tab.id)}
              onClose={() => props.onClose(tab.id)}
            />
          )}
        </For>
      </div>
    </div>
  );
}
