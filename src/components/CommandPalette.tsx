// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, For, Show, onMount, onCleanup, JSX } from "solid-js";
import { HiOutlineMagnifyingGlass } from "./icons";

export interface CommandAction {
  id: string;
  label: string;
  shortcut?: string;
  icon?: string | JSX.Element;
  category?: string;
  onSelect: () => void;
  disabled?: boolean;
}

interface CommandPaletteProps {
  actions: CommandAction[];
  isOpen: boolean;
  onClose: () => void;
  placeholder?: string;
}

export function CommandPalette(props: CommandPaletteProps) {
  const [query, setQuery] = createSignal("");
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  let inputRef: HTMLInputElement | undefined;
  let containerRef: HTMLDivElement | undefined;

  // Filter actions based on query
  const filteredActions = () => {
    const q = query().toLowerCase().trim();
    if (!q) return props.actions.filter(a => !a.disabled);
    
    return props.actions
      .filter(a => !a.disabled)
      .filter(action => {
        const label = action.label.toLowerCase();
        const category = (action.category || "").toLowerCase();
        // Fuzzy match - check if all query chars appear in order
        let queryIdx = 0;
        for (const char of label + " " + category) {
          if (char === q[queryIdx]) queryIdx++;
          if (queryIdx === q.length) return true;
        }
        return false;
      })
      .sort((a, b) => {
        // Prioritize exact prefix matches
        const aStarts = a.label.toLowerCase().startsWith(q);
        const bStarts = b.label.toLowerCase().startsWith(q);
        if (aStarts && !bStarts) return -1;
        if (bStarts && !aStarts) return 1;
        return 0;
      });
  };

  // Group actions by category
  const groupedActions = () => {
    const actions = filteredActions();
    const groups: Record<string, CommandAction[]> = {};
    
    for (const action of actions) {
      const category = action.category || "Actions";
      if (!groups[category]) groups[category] = [];
      groups[category].push(action);
    }
    
    return Object.entries(groups);
  };

  // Reset state when opening
  createEffect(() => {
    if (props.isOpen) {
      setQuery("");
      setSelectedIndex(0);
      // Focus input after mount
      requestAnimationFrame(() => inputRef?.focus());
    }
  });

  // Handle keyboard navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    const actions = filteredActions();
    
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex(i => Math.min(i + 1, actions.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex(i => Math.max(i - 1, 0));
        break;
      case "Enter":
        e.preventDefault();
        const selected = actions[selectedIndex()];
        if (selected) {
          selected.onSelect();
          props.onClose();
        }
        break;
      case "Escape":
        e.preventDefault();
        props.onClose();
        break;
    }
  };

  // Reset selection when query changes
  createEffect(() => {
    query(); // Track
    setSelectedIndex(0);
  });

  // Global keyboard listener for Cmd+K
  onMount(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        if (props.isOpen) {
          props.onClose();
        }
      }
    };
    window.addEventListener("keydown", handleGlobalKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleGlobalKeyDown));
  });

  // Focus trap
  createEffect(() => {
    if (!props.isOpen) return;
    
    const handleFocusOut = (e: FocusEvent) => {
      if (containerRef && !containerRef.contains(e.relatedTarget as Node)) {
        inputRef?.focus();
      }
    };
    
    containerRef?.addEventListener("focusout", handleFocusOut);
    onCleanup(() => containerRef?.removeEventListener("focusout", handleFocusOut));
  });

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

  // Highlight matching characters
  const highlightMatch = (text: string) => {
    const q = query().toLowerCase();
    if (!q) return text;
    
    let result = "";
    let queryIdx = 0;
    
    for (const char of text) {
      if (queryIdx < q.length && char.toLowerCase() === q[queryIdx]) {
        result += `<mark class="bg-cyan-500/30 text-cyan-300">${char}</mark>`;
        queryIdx++;
      } else {
        result += char;
      }
    }
    
    return result;
  };

  let flatIndex = 0;

  return (
    <Show when={props.isOpen}>
      {/* Backdrop */}
      <div 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-start justify-center pt-[15vh]"
        onClick={(e) => {
          if (e.target === e.currentTarget) props.onClose();
        }}
      >
        {/* Palette container */}
        <div 
          ref={containerRef}
          class="w-full max-w-xl bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl overflow-hidden"
          role="dialog"
          aria-modal="true"
          aria-label="Command palette"
        >
          {/* Search input */}
          <div class="flex items-center gap-3 px-4 py-3 border-b border-zinc-700">
            <HiOutlineMagnifyingGlass class="w-5 h-5 text-zinc-400" />
            <input
              ref={inputRef}
              type="text"
              value={query()}
              onInput={(e) => setQuery(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              placeholder={props.placeholder || "Type a command or search..."}
              class="flex-1 bg-transparent border-none outline-none text-zinc-100 text-base placeholder:text-zinc-500"
              autocomplete="off"
              spellcheck={false}
            />
            <kbd class="kbd">
              ESC
            </kbd>
          </div>

          {/* Results */}
          <div class="max-h-80 overflow-y-auto">
            <Show 
              when={filteredActions().length > 0}
              fallback={
                <div class="px-4 py-8 text-center text-zinc-500">
                  <HiOutlineMagnifyingGlass class="w-8 h-8 mb-2 mx-auto opacity-60" />
                  No commands found for "{query()}"
                </div>
              }
            >
              {(() => {
                flatIndex = 0;
                return null;
              })()}
              <For each={groupedActions()}>
                {([category, actions]) => (
                  <div class="py-1">
                    <div class="px-4 py-1.5 text-xs font-semibold uppercase tracking-wider text-zinc-500">
                      {category}
                    </div>
                    <For each={actions}>
                      {(action) => {
                        const currentIndex = flatIndex++;
                        return (
                          <button
                            class={`w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors ${
                              selectedIndex() === currentIndex
                                ? "bg-cyan-600/20 text-cyan-100"
                                : "text-zinc-300 hover:bg-zinc-800"
                            }`}
                            onClick={() => {
                              action.onSelect();
                              props.onClose();
                            }}
                            onMouseEnter={() => setSelectedIndex(currentIndex)}
                          >
                            <Show when={action.icon}>
                              <span class="w-5 text-center flex items-center justify-center">{action.icon}</span>
                            </Show>
                            <span 
                              class="flex-1 text-sm"
                              innerHTML={highlightMatch(action.label)}
                            />
                            <Show when={action.shortcut}>
                              <kbd class="kbd">
                                {formatShortcut(action.shortcut!)}
                              </kbd>
                            </Show>
                          </button>
                        );
                      }}
                    </For>
                  </div>
                )}
              </For>
            </Show>
          </div>

          {/* Footer hint */}
          <div class="px-4 py-2 border-t border-zinc-700 text-xs text-zinc-500 flex items-center gap-4">
            <span class="flex items-center gap-1">
              <kbd class="px-1 bg-zinc-800 rounded">↑</kbd>
              <kbd class="px-1 bg-zinc-800 rounded">↓</kbd>
              navigate
            </span>
            <span class="flex items-center gap-1">
              <kbd class="px-1 bg-zinc-800 rounded">↵</kbd>
              select
            </span>
            <span class="flex items-center gap-1">
              <kbd class="px-1 bg-zinc-800 rounded">esc</kbd>
              close
            </span>
          </div>
        </div>
      </div>
    </Show>
  );
}

// Hook to manage command palette state
export function createCommandPalette() {
  const [isOpen, setIsOpen] = createSignal(false);

  const open = () => setIsOpen(true);
  const close = () => setIsOpen(false);
  const toggle = () => setIsOpen(v => !v);

  // Register global Cmd+K shortcut
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        toggle();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  return { isOpen, open, close, toggle };
}
