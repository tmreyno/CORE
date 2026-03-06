// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import type { OpenTab, ContextMenuState } from "./types";

interface TabContextMenuProps {
  menu: ContextMenuState;
  tabs: OpenTab[];
  onMoveLeft: (tabId: string) => void;
  onMoveRight: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
  onCloseOthers: (tabId: string) => void;
  onCloseToRight?: (tabId: string) => void;
  onCloseAll: () => void;
  onCopyPath?: (path: string) => void;
  onRevealInTree?: (tabId: string) => void;
  onDismiss: () => void;
}

export function TabContextMenu(props: TabContextMenuProps) {
  const tab = () => props.tabs.find((t) => t.id === props.menu.tabId);
  const tabIndex = () => props.tabs.findIndex((t) => t.id === props.menu.tabId);
  const hasTabsToRight = () => tabIndex() < props.tabs.length - 1;

  return (
    <div
      class="context-menu"
      style={{ left: `${props.menu.x}px`, top: `${props.menu.y}px` }}
    >
      <button
        class="menu-item"
        onClick={() => { props.onMoveLeft(props.menu.tabId); props.onDismiss(); }}
      >
        ← Move Left
      </button>
      <button
        class="menu-item"
        onClick={() => { props.onMoveRight(props.menu.tabId); props.onDismiss(); }}
      >
        Move Right →
      </button>
      <hr class="my-1 border-border" />
      <Show when={tab()}>
        <button
          class="menu-item"
          onClick={() => { props.onCopyPath?.(tab()!.file.path); props.onDismiss(); }}
        >
          📋 Copy Path
        </button>
        <button
          class="menu-item"
          onClick={() => { props.onRevealInTree?.(props.menu.tabId); props.onDismiss(); }}
        >
          🔍 Reveal in Tree
        </button>
        <hr class="my-1 border-border" />
      </Show>
      <button
        class="menu-item"
        onClick={() => { props.onTabClose(props.menu.tabId); props.onDismiss(); }}
      >
        Close
      </button>
      <button
        class="menu-item"
        onClick={() => { props.onCloseOthers(props.menu.tabId); props.onDismiss(); }}
      >
        Close Others
      </button>
      <Show when={hasTabsToRight() && props.onCloseToRight}>
        <button
          class="menu-item"
          onClick={() => { props.onCloseToRight?.(props.menu.tabId); props.onDismiss(); }}
        >
          Close Tabs to Right
        </button>
      </Show>
      <button
        class="w-full px-3 py-1.5 text-left text-red-400 hover:bg-red-500/20 transition-colors"
        onClick={() => { props.onCloseAll(); props.onDismiss(); }}
      >
        Close All
      </button>
    </div>
  );
}
