// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, For, Show, onMount, onCleanup, JSX } from "solid-js";
import { Portal } from "solid-js/web";

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  separator?: boolean;
  onSelect?: () => void;
}

interface ContextMenuProps {
  items: ContextMenuItem[];
  position: { x: number; y: number } | null;
  onClose: () => void;
}

export function ContextMenu(props: ContextMenuProps) {
  const [selectedIndex, setSelectedIndex] = createSignal(-1);
  let menuRef: HTMLDivElement | undefined;

  // Close on click outside
  createEffect(() => {
    if (!props.position) return;
    
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef && !menuRef.contains(e.target as Node)) {
        props.onClose();
      }
    };
    
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        props.onClose();
      }
    };
    
    // Delay to prevent immediate close
    requestAnimationFrame(() => {
      window.addEventListener("click", handleClickOutside);
      window.addEventListener("keydown", handleEscape);
    });
    
    onCleanup(() => {
      window.removeEventListener("click", handleClickOutside);
      window.removeEventListener("keydown", handleEscape);
    });
  });

  // Keyboard navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    const selectableItems = props.items.filter(i => !i.separator && !i.disabled);
    
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex(i => {
          const next = i + 1;
          return next >= selectableItems.length ? 0 : next;
        });
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex(i => {
          const prev = i - 1;
          return prev < 0 ? selectableItems.length - 1 : prev;
        });
        break;
      case "Enter":
        e.preventDefault();
        const selected = selectableItems[selectedIndex()];
        if (selected?.onSelect) {
          selected.onSelect();
          props.onClose();
        }
        break;
    }
  };

  // Position adjustment to stay in viewport
  const adjustedPosition = () => {
    if (!props.position) return { top: 0, left: 0 };
    
    const menuWidth = 200;
    const menuHeight = props.items.length * 36;
    const padding = 8;
    
    let { x, y } = props.position;
    
    // Adjust horizontal
    if (x + menuWidth + padding > window.innerWidth) {
      x = window.innerWidth - menuWidth - padding;
    }
    
    // Adjust vertical
    if (y + menuHeight + padding > window.innerHeight) {
      y = window.innerHeight - menuHeight - padding;
    }
    
    return { top: y, left: x };
  };

  // Format shortcut for display
  const formatShortcut = (shortcut: string) => {
    return shortcut
      .replace("cmd", "⌘")
      .replace("ctrl", "⌃")
      .replace("alt", "⌥")
      .replace("shift", "⇧")
      .replace(/\+/g, "")
      .toUpperCase();
  };

  return (
    <Show when={props.position}>
      <Portal>
        <div
          ref={menuRef}
          class="fixed z-[100] min-w-[180px] bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl py-1 animate-[fadeIn_0.1s_ease-out]"
          style={{
            top: `${adjustedPosition().top}px`,
            left: `${adjustedPosition().left}px`,
          }}
          role="menu"
          tabIndex={-1}
          onKeyDown={handleKeyDown}
        >
          <For each={props.items}>
            {(item, index) => (
              <Show when={!item.separator} fallback={
                <div class="h-px bg-zinc-700 my-1 mx-2" role="separator" />
              }>
                <button
                  class={`w-full flex items-center gap-2 px-3 py-2 text-left text-sm transition-colors ${
                    item.disabled
                      ? "text-zinc-600 cursor-not-allowed"
                      : item.danger
                        ? selectedIndex() === index()
                          ? "bg-red-500/20 text-red-400"
                          : "text-red-400 hover:bg-red-500/20"
                        : selectedIndex() === index()
                          ? "bg-cyan-500/20 text-cyan-100"
                          : "text-zinc-300 hover:bg-zinc-800"
                  }`}
                  role="menuitem"
                  disabled={item.disabled}
                  onClick={() => {
                    if (!item.disabled && item.onSelect) {
                      item.onSelect();
                      props.onClose();
                    }
                  }}
                  onMouseEnter={() => !item.disabled && setSelectedIndex(index())}
                >
                  <Show when={item.icon}>
                    <span class="w-4 text-center">{item.icon}</span>
                  </Show>
                  <span class="flex-1">{item.label}</span>
                  <Show when={item.shortcut}>
                    <kbd class="text-xs text-zinc-500 font-mono">
                      {formatShortcut(item.shortcut!)}
                    </kbd>
                  </Show>
                </button>
              </Show>
            )}
          </For>
        </div>
      </Portal>
    </Show>
  );
}

// Hook for managing context menu state
export function createContextMenu() {
  const [position, setPosition] = createSignal<{ x: number; y: number } | null>(null);
  const [items, setItems] = createSignal<ContextMenuItem[]>([]);

  const open = (e: MouseEvent, menuItems: ContextMenuItem[]) => {
    e.preventDefault();
    e.stopPropagation();
    setItems(menuItems);
    setPosition({ x: e.clientX, y: e.clientY });
  };

  const close = () => {
    setPosition(null);
  };

  return { position, items, open, close };
}
