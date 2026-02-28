// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, createSignal, createEffect, onCleanup } from "solid-js";
import {
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineXMark,
  HiOutlineInformationCircle,
  HiOutlineCodeBracket,
  HiOutlineDocumentText,
  HiOutlineDocument,
  HiOutlineArrowUpTray,
} from "./icons";
import type { DiscoveredFile } from "../types";
import { typeClass } from "../utils";
import { getContainerTypeIcon } from "./tree";

export type TabViewMode = "info" | "hex" | "text" | "pdf" | "export";

export interface OpenTab {
  file: DiscoveredFile;
  id: string;
  viewMode?: TabViewMode;
}

interface TabBarProps {
  tabs: OpenTab[];
  activeTabId: string | null;
  viewMode: TabViewMode;
  onTabSelect: (tab: OpenTab) => void;
  onTabClose: (tabId: string, e?: MouseEvent) => void;
  onCloseOthers: (tabId: string) => void;
  onCloseAll: () => void;
  onCloseToRight?: (tabId: string) => void;
  onTabMove: (fromIndex: number, toIndex: number) => void;
  onViewModeChange: (mode: TabViewMode) => void;
  onCopyPath?: (path: string) => void;
  onRevealInTree?: (tabId: string) => void;
}

export function TabBar(props: TabBarProps) {
  // Drag and drop state
  const [dragOverTabId, setDragOverTabId] = createSignal<string | null>(null);
  
  // Context menu state
  const [contextMenu, setContextMenu] = createSignal<{ x: number; y: number; tabId: string } | null>(null);
  
  // Store tab element refs for mouse-based drag detection
  const tabRefs = new Map<string, HTMLDivElement>();
  
  // Mouse-based drag and drop (more reliable in Tauri than native HTML5 drag)
  const setupDragHandlers = (el: HTMLDivElement, tabId: string) => {
    tabRefs.set(tabId, el);
    
    let isDragging = false;
    let startX = 0;
    let startY = 0;
    let currentTargetId: string | null = null;
    const DRAG_THRESHOLD = 5;
    
    // Prevent native drag behavior
    el.addEventListener('dragstart', (e) => e.preventDefault());
    
    const onMouseDown = (e: MouseEvent) => {
      // Middle click to close tab
      if (e.button === 1) {
        e.preventDefault();
        props.onTabClose(tabId);
        return;
      }
      
      // Only left mouse button for drag, and not on buttons
      if (e.button !== 0) return;
      const target = e.target as HTMLElement;
      if (target.closest('.detail-tab-close') || target.closest('.tab-move-btn')) return;
      
      e.preventDefault();
      startX = e.clientX;
      startY = e.clientY;
      
      const onMouseMove = (moveEvent: MouseEvent) => {
        moveEvent.preventDefault();
        const dx = Math.abs(moveEvent.clientX - startX);
        const dy = Math.abs(moveEvent.clientY - startY);
        
        if (!isDragging && (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD)) {
          isDragging = true;
          el.classList.add('dragging');
          document.body.style.cursor = 'grabbing';
        }
        
        if (isDragging) {
          // Find which tab we're over
          let foundTarget: string | null = null;
          
          for (const tab of props.tabs) {
            if (tab.id === tabId) continue;
            const tabEl = tabRefs.get(tab.id);
            if (tabEl) {
              const rect = tabEl.getBoundingClientRect();
              if (moveEvent.clientX >= rect.left && moveEvent.clientX <= rect.right &&
                  moveEvent.clientY >= rect.top && moveEvent.clientY <= rect.bottom) {
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
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);
        
        if (isDragging) {
          if (currentTargetId && currentTargetId !== tabId) {
            const sourceIndex = props.tabs.findIndex(t => t.id === tabId);
            const targetIndex = props.tabs.findIndex(t => t.id === currentTargetId);
            
            if (sourceIndex !== -1 && targetIndex !== -1) {
              props.onTabMove(sourceIndex, targetIndex);
            }
          }
          
          el.classList.remove('dragging');
          document.body.style.cursor = '';
          setDragOverTabId(null);
          currentTargetId = null;
          isDragging = false;
        }
      };
      
      document.addEventListener('mousemove', onMouseMove);
      document.addEventListener('mouseup', onMouseUp);
    };
    
    el.addEventListener('mousedown', onMouseDown);
    
    onCleanup(() => {
      el.removeEventListener('mousedown', onMouseDown);
      tabRefs.delete(tabId);
    });
  };
  
  // Button-based tab reordering
  const moveTabLeft = (tabId: string) => {
    const index = props.tabs.findIndex(t => t.id === tabId);
    if (index > 0) {
      props.onTabMove(index, index - 1);
    }
  };
  
  const moveTabRight = (tabId: string) => {
    const index = props.tabs.findIndex(t => t.id === tabId);
    if (index < props.tabs.length - 1) {
      props.onTabMove(index, index + 1);
    }
  };
  
  // Context menu handlers
  const handleContextMenu = (tabId: string, e: MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, tabId });
  };
  
  const handleDocumentClick = () => {
    setContextMenu(null);
  };
  
  createEffect(() => {
    if (contextMenu()) {
      document.addEventListener('click', handleDocumentClick);
      onCleanup(() => document.removeEventListener('click', handleDocumentClick));
    }
  });
  
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
                    ? 'bg-bg-panel text-txt' 
                    : 'bg-transparent text-txt-secondary hover:bg-bg-panel/50 hover:text-txt'}
                  ${dragOverTabId() === tab.id ? 'bg-accent-soft border-accent' : ''}`}
                onClick={() => props.onTabSelect(tab)}
                onContextMenu={(e) => handleContextMenu(tab.id, e)}
                title={tab.file.path}
              >
                <Show when={index() > 0}>
                  <button
                    class={`opacity-0 group-hover:opacity-100 w-2 h-2 flex items-center justify-center text-txt-muted hover:text-txt transition-opacity`}
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
                    class={`opacity-0 group-hover:opacity-100 w-2 h-2 flex items-center justify-center text-txt-muted hover:text-txt transition-opacity`}
                    onClick={(e) => { e.stopPropagation(); moveTabRight(tab.id); }}
                    title="Move tab right"
                  >
                    <HiOutlineChevronRight class="w-2.5 h-2.5" />
                  </button>
                </Show>
                <button
                  class={`ml-0.5 w-2 h-2 flex items-center justify-center text-txt-muted hover:text-txt hover:bg-bg-active rounded transition-colors opacity-60 group-hover:opacity-100`}
                  onClick={(e) => { e.stopPropagation(); props.onTabClose(tab.id); }}
                  title="Close tab"
                >
                  <HiOutlineXMark class="w-2.5 h-2.5" />
                </button>
              </div>
            )}
          </For>
        </div>
        <div class="flex items-center gap-1 px-1.5 shrink-0 border-l border-border/50">
          {/* View mode selector */}
          <div class="flex items-center rounded-md bg-bg-panel/50 p-0.5">
            <button
              class={`flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] leading-tight rounded transition-colors ${
                props.viewMode === "info" 
                  ? "bg-accent text-white" 
                  : "text-txt-secondary hover:text-txt"
              }`}
              onClick={() => props.onViewModeChange("info")}
              title="Container Info"
            >
              <HiOutlineInformationCircle class="w-[10px] h-[10px]" /> Info
            </button>
            <button
              class={`flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] leading-tight rounded transition-colors ${
                props.viewMode === "hex" 
                  ? "bg-accent text-white" 
                  : "text-txt-secondary hover:text-txt"
              }`}
              onClick={() => props.onViewModeChange("hex")}
              title="Hex Viewer"
            >
              <HiOutlineCodeBracket class="w-[10px] h-[10px]" /> Hex
            </button>
            <button
              class={`flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] leading-tight rounded transition-colors ${
                props.viewMode === "text" 
                  ? "bg-accent text-white" 
                  : "text-txt-secondary hover:text-txt"
              }`}
              onClick={() => props.onViewModeChange("text")}
              title="Text Viewer"
            >
              <HiOutlineDocumentText class="w-[10px] h-[10px]" /> Text
            </button>
            <button
              class={`flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] leading-tight rounded transition-colors ${
                props.viewMode === "pdf" 
                  ? "bg-accent text-white" 
                  : "text-txt-secondary hover:text-txt"
              }`}
              onClick={() => props.onViewModeChange("pdf")}
              title="PDF Viewer"
            >
              <HiOutlineDocument class="w-[10px] h-[10px]" /> PDF
            </button>
            <button
              class={`flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] leading-tight rounded transition-colors ${
                props.viewMode === "export" 
                  ? "bg-accent text-white" 
                  : "text-txt-secondary hover:text-txt"
              }`}
              onClick={() => props.onViewModeChange("export")}
              title="Export Files"
            >
              <HiOutlineArrowUpTray class="w-[10px] h-[10px]" /> Export
            </button>
          </div>
          <button
            class="btn-text-danger"
            onClick={props.onCloseAll}
            title="Close all tabs"
            disabled={props.tabs.length === 0}
          >
            <HiOutlineXMark class="w-[10px] h-[10px]" /> All
          </button>
        </div>
      </div>
      
      {/* Context menu */}
      <Show when={contextMenu()}>
        {(menu) => {
          const tab = () => props.tabs.find(t => t.id === menu().tabId);
          const tabIndex = () => props.tabs.findIndex(t => t.id === menu().tabId);
          const hasTabsToRight = () => tabIndex() < props.tabs.length - 1;
          return (
            <div 
              class="context-menu"
              style={{ left: `${menu().x}px`, top: `${menu().y}px` }}
            >
              <button 
                class="menu-item"
                onClick={() => { moveTabLeft(menu().tabId); setContextMenu(null); }}
              >
                ← Move Left
              </button>
              <button 
                class="menu-item"
                onClick={() => { moveTabRight(menu().tabId); setContextMenu(null); }}
              >
                Move Right →
              </button>
              <hr class="my-1 border-border" />
              <Show when={tab()}>
                <button 
                  class="menu-item"
                  onClick={() => { props.onCopyPath?.(tab()!.file.path); setContextMenu(null); }}
                >
                  📋 Copy Path
                </button>
                <button 
                  class="menu-item"
                  onClick={() => { props.onRevealInTree?.(menu().tabId); setContextMenu(null); }}
                >
                  🔍 Reveal in Tree
                </button>
                <hr class="my-1 border-border" />
              </Show>
              <button 
                class="menu-item"
                onClick={() => { props.onTabClose(menu().tabId); setContextMenu(null); }}
              >
                Close
              </button>
              <button 
                class="menu-item"
                onClick={() => { props.onCloseOthers(menu().tabId); setContextMenu(null); }}
              >
                Close Others
              </button>
              <Show when={hasTabsToRight() && props.onCloseToRight}>
                <button 
                  class="menu-item"
                  onClick={() => { props.onCloseToRight?.(menu().tabId); setContextMenu(null); }}
                >
                  Close Tabs to Right
                </button>
              </Show>
              <button 
                class="w-full px-3 py-1.5 text-left text-red-400 hover:bg-red-500/20 transition-colors"
                onClick={() => { props.onCloseAll(); setContextMenu(null); }}
              >
                Close All
              </button>
            </div>
          );
        }}
      </Show>
    </>
  );
}
