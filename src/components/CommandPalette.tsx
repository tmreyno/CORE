// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, For, Show, onMount, JSX } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { HiOutlineMagnifyingGlass } from "./icons";
import { Kbd, Shortcut, CommonShortcuts, ModifierKeys } from "./ui/Kbd";

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
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(window, "keydown", handleGlobalKeyDown);
  });

  // Focus trap
  createEffect(() => {
    if (!props.isOpen) return;
    
    const handleFocusOut = (e: FocusEvent) => {
      if (containerRef && !containerRef.contains(e.relatedTarget as Node)) {
        inputRef?.focus();
      }
    };
    
    // makeEventListener auto-cleans up when effect re-runs or component unmounts
    if (containerRef) {
      makeEventListener(containerRef, "focusout", handleFocusOut);
    }
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
        result += `<mark class="bg-accent/30 text-accent-hover">${char}</mark>`;
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
        class="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-start justify-center pt-[12vh] animate-fade-in"
        onClick={(e) => {
          if (e.target === e.currentTarget) props.onClose();
        }}
      >
        {/* Palette container */}
        <div 
          ref={containerRef}
          class="w-full max-w-xl bg-bg-panel border border-border rounded-2xl shadow-2xl overflow-hidden animate-slide-up"
          style={{ "box-shadow": "0 25px 50px -12px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(var(--color-accent-rgb), 0.1)" }}
          role="dialog"
          aria-modal="true"
          aria-label="Command palette"
        >
          {/* Search input */}
          <div class="flex items-center gap-3 px-5 py-4 border-b border-border/50 bg-bg-secondary/30">
            <div class="p-2 rounded-lg bg-accent/10 text-accent">
              <HiOutlineMagnifyingGlass class="w-5 h-5" />
            </div>
            <input
              ref={inputRef}
              type="text"
              value={query()}
              onInput={(e) => setQuery(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              placeholder={props.placeholder || "Type a command or search..."}
              class="flex-1 bg-transparent border-none outline-none text-txt text-base placeholder:text-txt-muted"
              autocomplete="off"
              spellcheck={false}
            />
            <Show when={query()}>
              <span class="text-xs text-txt-muted px-2 py-1 bg-bg-secondary rounded-md">
                {filteredActions().length} result{filteredActions().length !== 1 ? "s" : ""}
              </span>
            </Show>
            <Kbd keys={ModifierKeys.esc} muted />
          </div>

          {/* Results */}
          <div class="max-h-[50vh] overflow-y-auto">
            <Show 
              when={filteredActions().length > 0}
              fallback={
                <div class="px-6 py-12 text-center">
                  <div class="inline-flex items-center justify-center w-16 h-16 bg-bg-secondary rounded-2xl mb-4">
                    <HiOutlineMagnifyingGlass class="w-8 h-8 text-txt-muted" />
                  </div>
                  <p class="text-txt-secondary font-medium mb-1">No commands found</p>
                  <p class="text-sm text-txt-muted">
                    Try a different search term for "{query()}"
                  </p>
                </div>
              }
            >
              {(() => {
                flatIndex = 0;
                return null;
              })()}
              <For each={groupedActions()}>
                {([category, actions]) => (
                  <div class="py-2">
                    <div class="px-5 py-2 text-[10px] font-semibold uppercase tracking-wider text-txt-muted flex items-center gap-2">
                      <span class="w-5 h-px bg-border/50" />
                      {category}
                      <span class="text-txt-faint">({actions.length})</span>
                      <span class="flex-1 h-px bg-border/50" />
                    </div>
                    <For each={actions}>
                      {(action) => {
                        const currentIndex = flatIndex++;
                        return (
                          <button
                            class={`w-full flex items-center gap-3 px-5 py-3 text-left transition-all duration-150 ${
                              selectedIndex() === currentIndex
                                ? "bg-accent/15 border-l-2 border-accent"
                                : "border-l-2 border-transparent hover:bg-bg-hover/50"
                            }`}
                            onClick={() => {
                              action.onSelect();
                              props.onClose();
                            }}
                            onMouseEnter={() => setSelectedIndex(currentIndex)}
                          >
                            <Show when={action.icon}>
                              <span class={`w-8 h-8 flex items-center justify-center rounded-lg transition-colors ${
                                selectedIndex() === currentIndex 
                                  ? "bg-accent/20 text-accent" 
                                  : "bg-bg-secondary text-txt-secondary"
                              }`}>
                                {action.icon}
                              </span>
                            </Show>
                            <span class="flex-1">
                              <span 
                                class={`text-sm font-medium ${selectedIndex() === currentIndex ? "text-txt" : "text-txt-secondary"}`}
                                innerHTML={highlightMatch(action.label)}
                              />
                            </span>
                            <Show when={action.shortcut}>
                              <Kbd 
                                keys={formatShortcut(action.shortcut!)} 
                                class={selectedIndex() === currentIndex ? "text-accent" : ""}
                                muted={selectedIndex() !== currentIndex}
                              />
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
          <div class="px-5 py-3 border-t border-border/50 bg-bg-secondary/30 flex items-center justify-between">
            <div class="flex items-center gap-4 text-xs text-txt-muted">
              <span class="flex items-center gap-1.5">
                <Kbd keys={[ModifierKeys.up, ModifierKeys.down]} muted />
                <span>Navigate</span>
              </span>
              <span class="flex items-center gap-1.5">
                <Kbd keys={ModifierKeys.enter} muted />
                <span>Select</span>
              </span>
            </div>
            <span class="text-xs text-txt-muted flex items-center gap-1">
              Press <Shortcut {...CommonShortcuts.commandPalette} /> anywhere
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
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(window, "keydown", handleKeyDown);
  });

  return { isOpen, open, close, toggle };
}
