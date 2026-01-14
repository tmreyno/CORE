// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, onMount, onCleanup } from "solid-js";
import { HiOutlineCommandLine, HiOutlineXMark } from "./icons";

export interface ShortcutGroup {
  title: string;
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

  // Close on Escape
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && props.isOpen) {
        props.onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
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
      {/* Backdrop */}
      <div 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center"
        onClick={(e) => {
          if (e.target === e.currentTarget) props.onClose();
        }}
      >
        {/* Modal */}
        <div 
          ref={modalRef}
          class="w-full max-w-2xl max-h-[80vh] bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl overflow-hidden flex flex-col"
          role="dialog"
          aria-modal="true"
          aria-labelledby="shortcuts-title"
        >
          {/* Header */}
          <div class="flex items-center justify-between px-6 py-4 border-b border-zinc-700">
            <h2 id="shortcuts-title" class="text-lg font-semibold text-zinc-100 flex items-center gap-2">
              <HiOutlineCommandLine class="w-5 h-5" />
              Keyboard Shortcuts
            </h2>
            <button
              class="p-1.5 rounded hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors"
              onClick={props.onClose}
              aria-label="Close"
            >
              <HiOutlineXMark class="w-5 h-5" />
            </button>
          </div>

          {/* Content */}
          <div class="flex-1 overflow-y-auto p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <For each={props.groups}>
                {(group) => (
                  <div>
                    <h3 class="text-sm font-semibold text-cyan-400 uppercase tracking-wider mb-3">
                      {group.title}
                    </h3>
                    <div class="space-y-2">
                      <For each={group.shortcuts}>
                        {(shortcut) => (
                          <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-zinc-300">{shortcut.description}</span>
                            <div class="flex items-center gap-1">
                              <For each={formatKeys(shortcut.keys)}>
                                {(key) => (
                                  <kbd class="kbd-sm">
                                    {key}
                                  </kbd>
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
          </div>

          {/* Footer */}
          <div class="px-6 py-3 border-t border-zinc-700 bg-zinc-800/50 text-center">
            <p class="text-xs text-zinc-500">
              Press <kbd class="px-1 bg-zinc-700 rounded mx-1">?</kbd> anytime to show this dialog
            </p>
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
      { keys: "cmd+o", description: "Open file" },
      { keys: "cmd+shift+o", description: "Open folder" },
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
