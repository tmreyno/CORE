// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show } from "solid-js";
import { HiOutlineChevronLeft, HiOutlineChevronRight, HiOutlineXMark } from "../icons";
import { typeClass } from "../../utils";
import { getContainerTypeIcon } from "../tree";
import { useDragReorder, useContextMenu } from "./useDragReorder";
import { ViewModeSelector } from "./ViewModeSelector";
import { TabContextMenu } from "./TabContextMenu";
import type { TabBarProps } from "./types";

export function TabBarComponent(props: TabBarProps) {
  const { dragOverTabId, setupDragHandlers, moveTabLeft, moveTabRight } = useDragReorder({
    tabs: () => props.tabs,
    onTabMove: props.onTabMove,
    onTabClose: props.onTabClose,
  });

  const { contextMenu, setContextMenu, handleContextMenu } = useContextMenu();

  return (
    <>
      <div class="flex items-center justify-between border-b border-border bg-bg-toolbar h-6 min-h-[24px]">
        <div class="flex items-center overflow-x-auto flex-1 min-w-0 scrollbar-none">
          <For each={props.tabs}>
            {(tab, index) => (
              <div
                ref={(el) => setupDragHandlers(el, tab.id)}
                class={`group relative flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] leading-tight cursor-pointer transition-colors select-none border-r border-border/50
                  ${props.activeTabId === tab.id
                    ? "bg-bg-panel text-txt"
                    : "bg-transparent text-txt-secondary hover:bg-bg-panel/50 hover:text-txt"}
                  ${dragOverTabId() === tab.id ? "bg-accent-soft border-accent" : ""}`}
                onClick={() => props.onTabSelect(tab)}
                onContextMenu={(e) => handleContextMenu(tab.id, e)}
                title={tab.file.path}
              >
                <Show when={index() > 0}>
                  <button
                    class="opacity-0 group-hover:opacity-100 w-2 h-2 flex items-center justify-center text-txt-muted hover:text-txt transition-opacity"
                    onClick={(e) => { e.stopPropagation(); moveTabLeft(tab.id); }}
                    title="Move tab left"
                  >
                    <HiOutlineChevronLeft class="w-2.5 h-2.5" />
                  </button>
                </Show>
                <span class={`shrink-0 ${typeClass(tab.file.container_type)}`}>
                  {(() => {
                    const IconComponent = getContainerTypeIcon(tab.file.container_type);
                    return <IconComponent class="w-[10px] h-[10px]" />;
                  })()}
                </span>
                <span class="truncate max-w-[120px]">{tab.file.filename}</span>
                <Show when={index() < props.tabs.length - 1}>
                  <button
                    class="opacity-0 group-hover:opacity-100 w-2 h-2 flex items-center justify-center text-txt-muted hover:text-txt transition-opacity"
                    onClick={(e) => { e.stopPropagation(); moveTabRight(tab.id); }}
                    title="Move tab right"
                  >
                    <HiOutlineChevronRight class="w-2.5 h-2.5" />
                  </button>
                </Show>
                <button
                  class="ml-0.5 w-2 h-2 flex items-center justify-center text-txt-muted hover:text-txt hover:bg-bg-active rounded transition-colors opacity-60 group-hover:opacity-100"
                  onClick={(e) => { e.stopPropagation(); props.onTabClose(tab.id); }}
                  title="Close tab"
                >
                  <HiOutlineXMark class="w-2.5 h-2.5" />
                </button>
              </div>
            )}
          </For>
        </div>
        <ViewModeSelector
          viewMode={props.viewMode}
          onViewModeChange={props.onViewModeChange}
          tabCount={props.tabs.length}
          onCloseAll={props.onCloseAll}
        />
      </div>

      <Show when={contextMenu()}>
        {(menu) => (
          <TabContextMenu
            menu={menu()}
            tabs={props.tabs}
            onMoveLeft={moveTabLeft}
            onMoveRight={moveTabRight}
            onTabClose={props.onTabClose}
            onCloseOthers={props.onCloseOthers}
            onCloseToRight={props.onCloseToRight}
            onCloseAll={props.onCloseAll}
            onCopyPath={props.onCopyPath}
            onRevealInTree={props.onRevealInTree}
            onDismiss={() => setContextMenu(null)}
          />
        )}
      </Show>
    </>
  );
}
