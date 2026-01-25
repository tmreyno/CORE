// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, For, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { makeEventListener } from "@solid-primitives/event-listener";
import { Kbd } from "./ui/Kbd";

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  separator?: boolean;
  checked?: boolean;  // For toggle/checkbox items
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
    
    // Delay to prevent immediate close, then use reactive event listeners
    requestAnimationFrame(() => {
      // makeEventListener auto-cleans up when effect re-runs or component unmounts
      makeEventListener(window, "click", handleClickOutside);
      makeEventListener(window, "keydown", handleEscape);
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
          class="fixed z-[100] min-w-[200px] bg-bg-panel border border-border rounded-xl shadow-2xl py-1.5 animate-fade-in overflow-hidden"
          style={{
            top: `${adjustedPosition().top}px`,
            left: `${adjustedPosition().left}px`,
            "box-shadow": "0 20px 40px -12px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(var(--color-accent-rgb), 0.05)",
          }}
          role="menu"
          tabIndex={-1}
          onKeyDown={handleKeyDown}
        >
          <For each={props.items}>
            {(item, index) => (
              <Show when={!item.separator} fallback={
                <div class="h-px bg-border/50 my-1.5 mx-3" role="separator" />
              }>
                <button
                  class={`w-full flex items-center gap-2.5 px-3 py-2 text-left text-sm transition-all duration-100 ${
                    item.disabled
                      ? "text-txt-muted cursor-not-allowed opacity-50"
                      : item.danger
                        ? selectedIndex() === index()
                          ? "bg-error/15 text-error"
                          : "text-error/80 hover:bg-error/10 hover:text-error"
                        : selectedIndex() === index()
                          ? "bg-accent/15 text-txt"
                          : "text-txt-secondary hover:bg-bg-hover hover:text-txt"
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
                  {/* Checkbox indicator for toggle items */}
                  <Show when={item.checked !== undefined}>
                    <span class={`w-4 h-4 flex items-center justify-center rounded border transition-colors ${
                      item.checked 
                        ? "bg-accent border-accent text-white" 
                        : "border-border bg-bg-secondary"
                    }`}>
                      <Show when={item.checked}>
                        <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                        </svg>
                      </Show>
                    </span>
                  </Show>
                  <Show when={item.icon && item.checked === undefined}>
                    <span class={`w-5 h-5 flex items-center justify-center text-base ${
                      selectedIndex() === index() ? "opacity-100" : "opacity-70"
                    }`}>{item.icon}</span>
                  </Show>
                  <span class="flex-1 font-medium">{item.label}</span>
                  <Show when={item.shortcut}>
                    <Kbd 
                      keys={formatShortcut(item.shortcut!)} 
                      class={selectedIndex() === index() ? "text-accent" : ""}
                      muted={selectedIndex() !== index()}
                      size="sm"
                    />
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
