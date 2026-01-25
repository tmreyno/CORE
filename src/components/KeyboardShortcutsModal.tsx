// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createMemo, For, Show, onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { HiOutlineCommandLine, HiOutlineXMark, HiOutlineMagnifyingGlass } from "./icons";
import { Kbd, Shortcut, CommonShortcuts, ModifierKeys } from "./ui/Kbd";

export interface ShortcutGroup {
  title: string;
  icon?: string;
  shortcuts: {
    keys: string;
    description: string;
  }[];
}

interface KeyboardShortcutsModalProps {
  isOpen: boolean;
  onClose: () => void;
  groups: ShortcutGroup[];
}

export function KeyboardShortcutsModal(props: KeyboardShortcutsModalProps) {
  let modalRef: HTMLDivElement | undefined;
  let inputRef: HTMLInputElement | undefined;
  
  const [searchQuery, setSearchQuery] = createSignal("");
  
  // Filter shortcuts based on search query
  const filteredGroups = createMemo(() => {
    const query = searchQuery().toLowerCase().trim();
    if (!query) return props.groups;
    
    return props.groups
      .map(group => ({
        ...group,
        shortcuts: group.shortcuts.filter(
          s => s.description.toLowerCase().includes(query) ||
               s.keys.toLowerCase().includes(query)
        )
      }))
      .filter(group => group.shortcuts.length > 0);
  });
  
  const totalShortcuts = createMemo(() => 
    props.groups.reduce((sum, g) => sum + g.shortcuts.length, 0)
  );
  
  const matchedCount = createMemo(() =>
    filteredGroups().reduce((sum, g) => sum + g.shortcuts.length, 0)
  );

  // Close on Escape, focus input when opening
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && props.isOpen) {
        if (searchQuery()) {
          setSearchQuery("");
        } else {
          props.onClose();
        }
      }
    };
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(window, "keydown", handleKeyDown);
  });
  
  // Focus input when modal opens
  createMemo(() => {
    if (props.isOpen) {
      setTimeout(() => inputRef?.focus(), 100);
    } else {
      setSearchQuery("");
    }
  });

  // Format shortcut keys for display
  const formatKeys = (keys: string) => {
    return keys
      .split("+")
      .map(key => {
        switch (key.toLowerCase()) {
          case "cmd": return "⌘";
          case "ctrl": return "⌃";
          case "alt": return "⌥";
          case "shift": return "⇧";
          case "enter": return "↵";
          case "esc": return "ESC";
          case "space": return "␣";
          case "up": return "↑";
          case "down": return "↓";
          case "left": return "←";
          case "right": return "→";
          case "tab": return "⇥";
          case "backspace": return "⌫";
          case "delete": return "⌦";
          default: return key.toUpperCase();
        }
      });
  };

  return (
    <Show when={props.isOpen}>
      {/* Modal Overlay */}
      <div 
        class="modal-overlay"
        onClick={(e) => {
          if (e.target === e.currentTarget) props.onClose();
        }}
      >
        {/* Modal */}
        <div 
          ref={modalRef}
          class="modal-content max-w-2xl w-full max-h-[80vh] flex flex-col"
          role="dialog"
          aria-modal="true"
          aria-labelledby="shortcuts-title"
        >
          {/* Header */}
          <div class="modal-header bg-bg-secondary/50">
            <div class="flex items-center gap-3">
              <div class="p-2 rounded-lg bg-accent/10 text-accent">
                <HiOutlineCommandLine class="w-5 h-5" />
              </div>
              <div>
                <h2 id="shortcuts-title" class="text-lg font-semibold text-txt">
                  Keyboard Shortcuts
                </h2>
                <p class="text-xs text-txt-muted mt-0.5">
                  {totalShortcuts()} shortcuts across {props.groups.length} categories
                </p>
              </div>
            </div>
            <button
              class="icon-btn"
              onClick={props.onClose}
              aria-label="Close"
            >
              <HiOutlineXMark class="w-5 h-5" />
            </button>
          </div>
          
          {/* Search Bar */}
          <div class="px-5 py-3 border-b border-border/50 bg-bg-secondary/30">
            <div class="relative">
              <HiOutlineMagnifyingGlass class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-txt-muted" />
              <input
                ref={inputRef}
                type="text"
                placeholder="Search shortcuts..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                class="input pl-10 pr-4 py-2"
              />
              <Show when={searchQuery()}>
                <span class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-txt-muted">
                  {matchedCount()} match{matchedCount() !== 1 ? "es" : ""}
                </span>
              </Show>
            </div>
          </div>

          {/* Content */}
          <div class="modal-body">
            <Show when={filteredGroups().length > 0} fallback={
              <div class="text-center py-12 text-txt-muted">
                <p class="text-sm">No shortcuts match "{searchQuery()}"</p>
                <button 
                  class="mt-2 text-xs text-accent hover:underline"
                  onClick={() => setSearchQuery("")}
                >
                  Clear search
                </button>
              </div>
            }>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <For each={filteredGroups()}>
                  {(group) => (
                    <div class="space-y-2">
                      <h3 class="flex items-center gap-2 text-xs font-semibold text-accent uppercase tracking-wider pb-1 border-b border-border/30">
                        {group.title}
                        <span class="text-txt-muted font-normal">({group.shortcuts.length})</span>
                      </h3>
                      <div class="space-y-1">
                        <For each={group.shortcuts}>
                          {(shortcut) => (
                            <div class="flex items-center justify-between py-1.5 px-2 rounded-md hover:bg-bg-hover/50 transition-colors group">
                              <span class="text-sm text-txt-secondary group-hover:text-txt transition-colors">
                                {shortcut.description}
                              </span>
                              <div class="flex items-center gap-1">
                                <For each={formatKeys(shortcut.keys)}>
                                  {(key, index) => (
                                    <>
                                      <Kbd keys={key} class="group-hover:text-accent transition-colors" />
                                      <Show when={index() < formatKeys(shortcut.keys).length - 1}>
                                        <span class="text-txt-muted text-xs">+</span>
                                      </Show>
                                    </>
                                  )}
                                </For>
                              </div>
                            </div>
                          )}
                        </For>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>

          {/* Footer */}
          <div class="px-5 py-3 border-t border-border bg-bg-secondary/30 flex items-center justify-between">
            <div class="flex items-center gap-4 text-xs text-txt-muted">
              <span class="flex items-center gap-1.5">
                <Kbd keys="?" muted />
                <span>Show this dialog</span>
              </span>
              <span class="flex items-center gap-1.5">
                <Kbd keys={ModifierKeys.esc} muted />
                <span>Close</span>
              </span>
            </div>
            <span class="text-xs text-txt-muted flex items-center gap-1.5">
              Pro tip: Use <Shortcut {...CommonShortcuts.commandPalette} /> for quick actions
            </span>
          </div>
        </div>
      </div>
    </Show>
  );
}

// Default shortcut groups for CORE-FFX
export const DEFAULT_SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    title: "General",
    shortcuts: [
      { keys: "cmd+k", description: "Open command palette" },
      { keys: "cmd+,", description: "Open preferences" },
      { keys: "?", description: "Show keyboard shortcuts" },
      { keys: "esc", description: "Close dialog / Clear filter" },
    ],
  },
  {
    title: "Project",
    shortcuts: [
      { keys: "cmd+shift+n", description: "New project" },
      { keys: "cmd+o", description: "Open project" },
      { keys: "cmd+s", description: "Save project" },
      { keys: "cmd+shift+s", description: "Save project as..." },
    ],
  },
  {
    title: "Navigation",
    shortcuts: [
      { keys: "up", description: "Move selection up" },
      { keys: "down", description: "Move selection down" },
      { keys: "enter", description: "Open selected item" },
      { keys: "space", description: "Toggle selection" },
      { keys: "home", description: "Jump to first item" },
      { keys: "end", description: "Jump to last item" },
    ],
  },
  {
    title: "File Operations",
    shortcuts: [
      { keys: "cmd+f", description: "Search files" },
      { keys: "cmd+r", description: "Refresh" },
    ],
  },
  {
    title: "View",
    shortcuts: [
      { keys: "cmd+1", description: "Show info view" },
      { keys: "cmd+2", description: "Show hex view" },
      { keys: "cmd+3", description: "Show text view" },
      { keys: "cmd+b", description: "Toggle left panel" },
      { keys: "cmd+shift+b", description: "Toggle right panel" },
    ],
  },
  {
    title: "Tree Navigation",
    shortcuts: [
      { keys: "left", description: "Collapse node" },
      { keys: "right", description: "Expand node" },
      { keys: "cmd+left", description: "Collapse all" },
      { keys: "cmd+right", description: "Expand all" },
    ],
  },
  {
    title: "Actions",
    shortcuts: [
      { keys: "cmd+h", description: "Compute hash" },
      { keys: "cmd+v", description: "Verify hash" },
      { keys: "cmd+e", description: "Export" },
      { keys: "cmd+p", description: "Generate report" },
    ],
  },
];
