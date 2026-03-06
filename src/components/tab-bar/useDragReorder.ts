// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, onCleanup } from "solid-js";
import type { OpenTab, ContextMenuState } from "./types";

interface UseDragReorderOptions {
  tabs: () => OpenTab[];
  onTabMove: (fromIndex: number, toIndex: number) => void;
  onTabClose: (tabId: string) => void;
}

const DRAG_THRESHOLD = 5;

export function useDragReorder(options: UseDragReorderOptions) {
  const [dragOverTabId, setDragOverTabId] = createSignal<string | null>(null);
  const tabRefs = new Map<string, HTMLDivElement>();

  const setupDragHandlers = (el: HTMLDivElement, tabId: string) => {
    tabRefs.set(tabId, el);

    let isDragging = false;
    let startX = 0;
    let startY = 0;
    let currentTargetId: string | null = null;

    el.addEventListener("dragstart", (e) => e.preventDefault());

    const onMouseDown = (e: MouseEvent) => {
      if (e.button === 1) {
        e.preventDefault();
        options.onTabClose(tabId);
        return;
      }

      if (e.button !== 0) return;
      const target = e.target as HTMLElement;
      if (target.closest(".detail-tab-close") || target.closest(".tab-move-btn")) return;

      e.preventDefault();
      startX = e.clientX;
      startY = e.clientY;

      const onMouseMove = (moveEvent: MouseEvent) => {
        moveEvent.preventDefault();
        const dx = Math.abs(moveEvent.clientX - startX);
        const dy = Math.abs(moveEvent.clientY - startY);

        if (!isDragging && (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD)) {
          isDragging = true;
          el.classList.add("dragging");
          document.body.style.cursor = "grabbing";
        }

        if (isDragging) {
          let foundTarget: string | null = null;
          for (const tab of options.tabs()) {
            if (tab.id === tabId) continue;
            const tabEl = tabRefs.get(tab.id);
            if (tabEl) {
              const rect = tabEl.getBoundingClientRect();
              if (
                moveEvent.clientX >= rect.left &&
                moveEvent.clientX <= rect.right &&
                moveEvent.clientY >= rect.top &&
                moveEvent.clientY <= rect.bottom
              ) {
                foundTarget = tab.id;
                break;
              }
            }
          }
          currentTargetId = foundTarget;
          setDragOverTabId(foundTarget);
        }
      };

      const onMouseUp = (upEvent: MouseEvent) => {
        upEvent.preventDefault();
        document.removeEventListener("mousemove", onMouseMove);
        document.removeEventListener("mouseup", onMouseUp);

        if (isDragging) {
          if (currentTargetId && currentTargetId !== tabId) {
            const sourceIndex = options.tabs().findIndex((t) => t.id === tabId);
            const targetIndex = options.tabs().findIndex((t) => t.id === currentTargetId);
            if (sourceIndex !== -1 && targetIndex !== -1) {
              options.onTabMove(sourceIndex, targetIndex);
            }
          }
          el.classList.remove("dragging");
          document.body.style.cursor = "";
          setDragOverTabId(null);
          currentTargetId = null;
          isDragging = false;
        }
      };

      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
    };

    el.addEventListener("mousedown", onMouseDown);
    onCleanup(() => {
      el.removeEventListener("mousedown", onMouseDown);
      tabRefs.delete(tabId);
    });
  };

  const moveTabLeft = (tabId: string) => {
    const index = options.tabs().findIndex((t) => t.id === tabId);
    if (index > 0) options.onTabMove(index, index - 1);
  };

  const moveTabRight = (tabId: string) => {
    const index = options.tabs().findIndex((t) => t.id === tabId);
    if (index < options.tabs().length - 1) options.onTabMove(index, index + 1);
  };

  return { dragOverTabId, setupDragHandlers, moveTabLeft, moveTabRight };
}

export function useContextMenu() {
  const [contextMenu, setContextMenu] = createSignal<ContextMenuState | null>(null);

  const handleContextMenu = (tabId: string, e: MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, tabId });
  };

  const handleDocumentClick = () => setContextMenu(null);

  createEffect(() => {
    if (contextMenu()) {
      document.addEventListener("click", handleDocumentClick);
      onCleanup(() => document.removeEventListener("click", handleDocumentClick));
    }
  });

  return { contextMenu, setContextMenu, handleContextMenu };
}
