// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";

export interface KeyboardShortcut {
  /** Unique identifier for the shortcut */
  id: string;
  /** Key combination (e.g., "cmd+s", "ctrl+shift+o") */
  keys: string;
  /** Description for help/documentation */
  description: string;
  /** Handler function */
  handler: (e: KeyboardEvent) => void;
  /** Whether shortcut is currently enabled */
  enabled?: boolean;
  /** Prevent default browser behavior */
  preventDefault?: boolean;
  /** Category for grouping in help */
  category?: string;
}

interface UseKeyboardShortcutsOptions {
  /** Whether shortcuts are globally enabled */
  enabled?: boolean;
  /** Don't trigger when focused on input/textarea */
  ignoreInputs?: boolean;
}

/**
 * Parse key combination string into parts
 */
function parseKeyCombination(keys: string): {
  key: string;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  meta: boolean;
} {
  const parts = keys.toLowerCase().split("+").map((p) => p.trim());
  const key = parts.pop() || "";
  
  return {
    key,
    ctrl: parts.includes("ctrl"),
    shift: parts.includes("shift"),
    alt: parts.includes("alt"),
    meta: parts.includes("cmd") || parts.includes("meta") || parts.includes("win"),
  };
}

/**
 * Check if event matches key combination
 */
function matchesShortcut(e: KeyboardEvent, keys: string): boolean {
  const combo = parseKeyCombination(keys);
  
  // Normalize key (handle special cases)
  let eventKey = e.key.toLowerCase();
  if (eventKey === " ") eventKey = "space";
  if (eventKey === "escape") eventKey = "esc";
  if (eventKey === "arrowup") eventKey = "up";
  if (eventKey === "arrowdown") eventKey = "down";
  if (eventKey === "arrowleft") eventKey = "left";
  if (eventKey === "arrowright") eventKey = "right";
  
  return (
    eventKey === combo.key &&
    e.ctrlKey === combo.ctrl &&
    e.shiftKey === combo.shift &&
    e.altKey === combo.alt &&
    e.metaKey === combo.meta
  );
}

/**
 * Hook for managing keyboard shortcuts
 * 
 * Usage:
 * ```tsx
 * const shortcuts = useKeyboardShortcuts({
 *   shortcuts: [
 *     { id: "save", keys: "cmd+s", description: "Save project", handler: handleSave },
 *     { id: "open", keys: "cmd+o", description: "Open file", handler: handleOpen },
 *   ]
 * });
 * 
 * // Dynamically add/remove shortcuts
 * shortcuts.register({ id: "custom", keys: "cmd+k", ... });
 * shortcuts.unregister("custom");
 * ```
 */
export function useKeyboardShortcuts(
  initialShortcuts: KeyboardShortcut[] = [],
  options: UseKeyboardShortcutsOptions = {}
) {
  const [shortcuts, setShortcuts] = createSignal<KeyboardShortcut[]>(initialShortcuts);
  const [enabled, setEnabled] = createSignal(options.enabled ?? true);
  const ignoreInputs = options.ignoreInputs ?? true;

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!enabled()) return;

    // Skip if focused on input elements (unless overridden)
    if (ignoreInputs) {
      const target = e.target as HTMLElement;
      const tagName = target.tagName.toLowerCase();
      const isInput = tagName === "input" || tagName === "textarea" || target.isContentEditable;
      if (isInput) return;
    }

    // Find matching shortcut
    for (const shortcut of shortcuts()) {
      if (shortcut.enabled === false) continue;
      
      if (matchesShortcut(e, shortcut.keys)) {
        if (shortcut.preventDefault !== false) {
          e.preventDefault();
        }
        shortcut.handler(e);
        break;
      }
    }
  };

  onMount(() => {
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(window, "keydown", handleKeyDown);
  });

  // Register a new shortcut
  const register = (shortcut: KeyboardShortcut) => {
    setShortcuts((prev) => {
      // Replace if exists, otherwise add
      const existing = prev.findIndex((s) => s.id === shortcut.id);
      if (existing >= 0) {
        const updated = [...prev];
        updated[existing] = shortcut;
        return updated;
      }
      return [...prev, shortcut];
    });
  };

  // Unregister a shortcut by id
  const unregister = (id: string) => {
    setShortcuts((prev) => prev.filter((s) => s.id !== id));
  };

  // Enable/disable a specific shortcut
  const setShortcutEnabled = (id: string, isEnabled: boolean) => {
    setShortcuts((prev) =>
      prev.map((s) => (s.id === id ? { ...s, enabled: isEnabled } : s))
    );
  };

  // Get all shortcuts (for help display)
  const getAll = () => shortcuts();

  // Get shortcuts by category
  const getByCategory = (category: string) =>
    shortcuts().filter((s) => s.category === category);

  return {
    shortcuts,
    enabled,
    setEnabled,
    register,
    unregister,
    setShortcutEnabled,
    getAll,
    getByCategory,
  };
}

/**
 * Format shortcut keys for display (e.g., "⌘S" on Mac)
 */
export function formatShortcutKeys(keys: string): string {
  const isMac = navigator.platform.toLowerCase().includes("mac");
  const combo = parseKeyCombination(keys);
  
  const parts: string[] = [];
  
  if (combo.ctrl) parts.push(isMac ? "⌃" : "Ctrl");
  if (combo.alt) parts.push(isMac ? "⌥" : "Alt");
  if (combo.shift) parts.push(isMac ? "⇧" : "Shift");
  if (combo.meta) parts.push(isMac ? "⌘" : "Win");
  
  // Capitalize single letter keys
  const key = combo.key.length === 1 ? combo.key.toUpperCase() : combo.key;
  parts.push(key);
  
  return isMac ? parts.join("") : parts.join("+");
}

/**
 * Common application shortcuts preset
 */
export const commonShortcuts = {
  save: "cmd+s",
  saveAs: "cmd+shift+s",
  open: "cmd+o",
  close: "cmd+w",
  undo: "cmd+z",
  redo: "cmd+shift+z",
  find: "cmd+f",
  selectAll: "cmd+a",
  copy: "cmd+c",
  cut: "cmd+x",
  paste: "cmd+v",
  escape: "esc",
  help: "cmd+?",
};
