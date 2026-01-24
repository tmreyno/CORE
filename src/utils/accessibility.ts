// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Accessibility utilities for CORE-FFX.
 * Provides ARIA helpers, keyboard navigation, screen reader announcements,
 * and focus management utilities.
 */

import { createSignal, onMount, Accessor } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";

// ============================================================================
// Types
// ============================================================================

export type AriaRole = 
  | "alert"
  | "alertdialog"
  | "application"
  | "article"
  | "banner"
  | "button"
  | "cell"
  | "checkbox"
  | "columnheader"
  | "combobox"
  | "complementary"
  | "contentinfo"
  | "definition"
  | "dialog"
  | "directory"
  | "document"
  | "feed"
  | "figure"
  | "form"
  | "grid"
  | "gridcell"
  | "group"
  | "heading"
  | "img"
  | "link"
  | "list"
  | "listbox"
  | "listitem"
  | "log"
  | "main"
  | "marquee"
  | "math"
  | "menu"
  | "menubar"
  | "menuitem"
  | "menuitemcheckbox"
  | "menuitemradio"
  | "navigation"
  | "none"
  | "note"
  | "option"
  | "presentation"
  | "progressbar"
  | "radio"
  | "radiogroup"
  | "region"
  | "row"
  | "rowgroup"
  | "rowheader"
  | "scrollbar"
  | "search"
  | "searchbox"
  | "separator"
  | "slider"
  | "spinbutton"
  | "status"
  | "switch"
  | "tab"
  | "table"
  | "tablist"
  | "tabpanel"
  | "term"
  | "textbox"
  | "timer"
  | "toolbar"
  | "tooltip"
  | "tree"
  | "treegrid"
  | "treeitem";

export type AriaLive = "off" | "polite" | "assertive";

export interface AriaAttributes {
  role?: AriaRole;
  "aria-label"?: string;
  "aria-labelledby"?: string;
  "aria-describedby"?: string;
  "aria-expanded"?: boolean;
  "aria-selected"?: boolean;
  "aria-checked"?: boolean | "mixed";
  "aria-disabled"?: boolean;
  "aria-hidden"?: boolean;
  "aria-live"?: AriaLive;
  "aria-atomic"?: boolean;
  "aria-busy"?: boolean;
  "aria-controls"?: string;
  "aria-current"?: boolean | "page" | "step" | "location" | "date" | "time";
  "aria-haspopup"?: boolean | "menu" | "listbox" | "tree" | "grid" | "dialog";
  "aria-invalid"?: boolean | "grammar" | "spelling";
  "aria-pressed"?: boolean | "mixed";
  "aria-readonly"?: boolean;
  "aria-required"?: boolean;
  "aria-sort"?: "none" | "ascending" | "descending" | "other";
  "aria-valuemin"?: number;
  "aria-valuemax"?: number;
  "aria-valuenow"?: number;
  "aria-valuetext"?: string;
  tabIndex?: number;
}

// ============================================================================
// Screen Reader Announcements
// ============================================================================

let announcer: HTMLDivElement | null = null;

/**
 * Initialize the screen reader announcer element.
 * This should be called once when the app mounts.
 */
export function initAnnouncer(): void {
  if (announcer) return;

  announcer = document.createElement("div");
  announcer.setAttribute("aria-live", "polite");
  announcer.setAttribute("aria-atomic", "true");
  announcer.setAttribute("role", "status");
  announcer.style.cssText = `
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  `;
  document.body.appendChild(announcer);
}

/**
 * Announce a message to screen readers.
 * 
 * @param message - The message to announce
 * @param priority - "polite" (default) or "assertive" for urgent messages
 * 
 * @example
 * ```tsx
 * // Polite announcement (waits for user to finish current task)
 * announce("File loaded successfully");
 * 
 * // Assertive announcement (interrupts current task)
 * announce("Error: Connection lost", "assertive");
 * ```
 */
export function announce(message: string, priority: AriaLive = "polite"): void {
  if (!announcer) initAnnouncer();
  if (!announcer) return;

  announcer.setAttribute("aria-live", priority);
  
  // Clear and re-set to ensure announcement
  announcer.textContent = "";
  requestAnimationFrame(() => {
    if (announcer) {
      announcer.textContent = message;
    }
  });
}

/**
 * Hook for managing announcements
 */
export function useAnnouncer() {
  onMount(() => initAnnouncer());
  
  return {
    announce,
    announcePolite: (msg: string) => announce(msg, "polite"),
    announceAssertive: (msg: string) => announce(msg, "assertive"),
  };
}

// ============================================================================
// Focus Management
// ============================================================================

/**
 * Get all focusable elements within a container
 */
export function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selectors = [
    'button:not([disabled]):not([tabindex="-1"])',
    'input:not([disabled]):not([tabindex="-1"])',
    'select:not([disabled]):not([tabindex="-1"])',
    'textarea:not([disabled]):not([tabindex="-1"])',
    'a[href]:not([tabindex="-1"])',
    '[tabindex]:not([tabindex="-1"]):not([disabled])',
    '[contenteditable]:not([tabindex="-1"])',
  ].join(", ");

  return Array.from(container.querySelectorAll<HTMLElement>(selectors))
    .filter(el => {
      // Additional check: element must be visible
      const style = window.getComputedStyle(el);
      return style.display !== "none" && style.visibility !== "hidden";
    });
}

/**
 * Focus the first focusable element in a container
 */
export function focusFirst(container: HTMLElement): boolean {
  const focusable = getFocusableElements(container);
  if (focusable.length > 0) {
    focusable[0].focus();
    return true;
  }
  return false;
}

/**
 * Focus the last focusable element in a container
 */
export function focusLast(container: HTMLElement): boolean {
  const focusable = getFocusableElements(container);
  if (focusable.length > 0) {
    focusable[focusable.length - 1].focus();
    return true;
  }
  return false;
}

/**
 * Move focus to the next focusable element
 */
export function focusNext(container: HTMLElement, wrap: boolean = true): boolean {
  const focusable = getFocusableElements(container);
  const currentIndex = focusable.indexOf(document.activeElement as HTMLElement);
  
  if (currentIndex === -1) {
    return focusFirst(container);
  }
  
  const nextIndex = currentIndex + 1;
  if (nextIndex < focusable.length) {
    focusable[nextIndex].focus();
    return true;
  } else if (wrap) {
    return focusFirst(container);
  }
  
  return false;
}

/**
 * Move focus to the previous focusable element
 */
export function focusPrevious(container: HTMLElement, wrap: boolean = true): boolean {
  const focusable = getFocusableElements(container);
  const currentIndex = focusable.indexOf(document.activeElement as HTMLElement);
  
  if (currentIndex === -1) {
    return focusLast(container);
  }
  
  const prevIndex = currentIndex - 1;
  if (prevIndex >= 0) {
    focusable[prevIndex].focus();
    return true;
  } else if (wrap) {
    return focusLast(container);
  }
  
  return false;
}

// ============================================================================
// Keyboard Navigation
// ============================================================================

export interface KeyboardNavigationOptions {
  /** Whether to wrap around when reaching the end */
  wrap?: boolean;
  /** Orientation of the navigation */
  orientation?: "horizontal" | "vertical" | "both";
  /** Callback when an item is selected */
  onSelect?: (index: number) => void;
  /** Callback when navigation occurs */
  onNavigate?: (index: number) => void;
}

/**
 * Hook for list/grid keyboard navigation
 */
export function useKeyboardNavigation(
  containerRef: Accessor<HTMLElement | undefined>,
  itemCount: Accessor<number>,
  options: KeyboardNavigationOptions = {}
) {
  const { wrap = true, orientation = "vertical", onSelect, onNavigate } = options;
  const [currentIndex, setCurrentIndex] = createSignal(0);

  const handleKeyDown = (e: KeyboardEvent) => {
    const container = containerRef();
    if (!container) return;

    const count = itemCount();
    if (count === 0) return;

    let newIndex = currentIndex();
    let handled = false;

    switch (e.key) {
      case "ArrowDown":
        if (orientation === "vertical" || orientation === "both") {
          newIndex = wrap 
            ? (currentIndex() + 1) % count 
            : Math.min(currentIndex() + 1, count - 1);
          handled = true;
        }
        break;

      case "ArrowUp":
        if (orientation === "vertical" || orientation === "both") {
          newIndex = wrap
            ? (currentIndex() - 1 + count) % count
            : Math.max(currentIndex() - 1, 0);
          handled = true;
        }
        break;

      case "ArrowRight":
        if (orientation === "horizontal" || orientation === "both") {
          newIndex = wrap
            ? (currentIndex() + 1) % count
            : Math.min(currentIndex() + 1, count - 1);
          handled = true;
        }
        break;

      case "ArrowLeft":
        if (orientation === "horizontal" || orientation === "both") {
          newIndex = wrap
            ? (currentIndex() - 1 + count) % count
            : Math.max(currentIndex() - 1, 0);
          handled = true;
        }
        break;

      case "Home":
        newIndex = 0;
        handled = true;
        break;

      case "End":
        newIndex = count - 1;
        handled = true;
        break;

      case "Enter":
      case " ":
        onSelect?.(currentIndex());
        handled = true;
        break;
    }

    if (handled) {
      e.preventDefault();
      if (newIndex !== currentIndex()) {
        setCurrentIndex(newIndex);
        onNavigate?.(newIndex);
      }
    }
  };

  onMount(() => {
    const container = containerRef();
    if (container) {
      // makeEventListener auto-cleans up on component unmount
      makeEventListener(container, "keydown", handleKeyDown);
    }
  });

  return {
    currentIndex,
    setCurrentIndex,
    goToFirst: () => setCurrentIndex(0),
    goToLast: () => setCurrentIndex(itemCount() - 1),
  };
}

// ============================================================================
// ARIA Helpers
// ============================================================================

/**
 * Generate a unique ID for ARIA references
 */
let idCounter = 0;
export function generateAriaId(prefix: string = "aria"): string {
  return `${prefix}-${++idCounter}`;
}

/**
 * Create ARIA attributes for a button
 */
export function buttonAriaProps(options: {
  label: string;
  pressed?: boolean;
  expanded?: boolean;
  controls?: string;
  disabled?: boolean;
  describedBy?: string;
}): AriaAttributes {
  return {
    role: "button",
    "aria-label": options.label,
    "aria-pressed": options.pressed,
    "aria-expanded": options.expanded,
    "aria-controls": options.controls,
    "aria-disabled": options.disabled,
    "aria-describedby": options.describedBy,
    tabIndex: options.disabled ? -1 : 0,
  };
}

/**
 * Create ARIA attributes for a checkbox
 */
export function checkboxAriaProps(options: {
  label: string;
  checked: boolean | "mixed";
  disabled?: boolean;
}): AriaAttributes {
  return {
    role: "checkbox",
    "aria-label": options.label,
    "aria-checked": options.checked,
    "aria-disabled": options.disabled,
    tabIndex: options.disabled ? -1 : 0,
  };
}

/**
 * Create ARIA attributes for a listbox
 */
export function listboxAriaProps(options: {
  label: string;
  multiselectable?: boolean;
  required?: boolean;
}): AriaAttributes {
  return {
    role: "listbox",
    "aria-label": options.label,
    "aria-required": options.required,
    tabIndex: 0,
  };
}

/**
 * Create ARIA attributes for a listbox option
 */
export function optionAriaProps(options: {
  selected: boolean;
  disabled?: boolean;
  setsize?: number;
  posinset?: number;
}): AriaAttributes & { "aria-setsize"?: number; "aria-posinset"?: number } {
  return {
    role: "option",
    "aria-selected": options.selected,
    "aria-disabled": options.disabled,
    "aria-setsize": options.setsize,
    "aria-posinset": options.posinset,
    tabIndex: options.selected ? 0 : -1,
  };
}

/**
 * Create ARIA attributes for a tree
 */
export function treeAriaProps(options: {
  label: string;
  multiselectable?: boolean;
}): AriaAttributes {
  return {
    role: "tree",
    "aria-label": options.label,
    tabIndex: 0,
  };
}

/**
 * Create ARIA attributes for a tree item
 */
export function treeitemAriaProps(options: {
  expanded?: boolean;
  selected?: boolean;
  level: number;
  setsize: number;
  posinset: number;
}): AriaAttributes & { "aria-level"?: number; "aria-setsize"?: number; "aria-posinset"?: number } {
  return {
    role: "treeitem",
    "aria-expanded": options.expanded,
    "aria-selected": options.selected,
    "aria-level": options.level,
    "aria-setsize": options.setsize,
    "aria-posinset": options.posinset,
    tabIndex: options.selected ? 0 : -1,
  };
}

/**
 * Create ARIA attributes for a dialog
 */
export function dialogAriaProps(options: {
  labelledBy: string;
  describedBy?: string;
  modal?: boolean;
}): AriaAttributes {
  return {
    role: options.modal ? "alertdialog" : "dialog",
    "aria-labelledby": options.labelledBy,
    "aria-describedby": options.describedBy,
  };
}

/**
 * Create ARIA attributes for a progress bar
 */
export function progressAriaProps(options: {
  label: string;
  value: number;
  min?: number;
  max?: number;
  valueText?: string;
}): AriaAttributes {
  return {
    role: "progressbar",
    "aria-label": options.label,
    "aria-valuenow": options.value,
    "aria-valuemin": options.min ?? 0,
    "aria-valuemax": options.max ?? 100,
    "aria-valuetext": options.valueText,
  };
}

/**
 * Create ARIA attributes for a tab list
 */
export function tablistAriaProps(options: {
  label: string;
  orientation?: "horizontal" | "vertical";
}): AriaAttributes & { "aria-orientation"?: "horizontal" | "vertical" } {
  return {
    role: "tablist",
    "aria-label": options.label,
    "aria-orientation": options.orientation ?? "horizontal",
  };
}

/**
 * Create ARIA attributes for a tab
 */
export function tabAriaProps(options: {
  selected: boolean;
  controls: string;
  disabled?: boolean;
}): AriaAttributes {
  return {
    role: "tab",
    "aria-selected": options.selected,
    "aria-controls": options.controls,
    "aria-disabled": options.disabled,
    tabIndex: options.selected ? 0 : -1,
  };
}

/**
 * Create ARIA attributes for a tab panel
 */
export function tabpanelAriaProps(options: {
  labelledBy: string;
  hidden?: boolean;
}): AriaAttributes {
  return {
    role: "tabpanel",
    "aria-labelledby": options.labelledBy,
    "aria-hidden": options.hidden,
    tabIndex: 0,
  };
}

// ============================================================================
// Visibility Helpers
// ============================================================================

/**
 * Hide element from screen readers
 */
export function visuallyHide(element: HTMLElement): void {
  element.style.cssText = `
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  `;
}

/**
 * Check if user prefers reduced motion
 */
export function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/**
 * Hook for checking reduced motion preference
 */
export function useReducedMotion() {
  const [reducedMotion, setReducedMotion] = createSignal(prefersReducedMotion());

  onMount(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const handleChange = (e: MediaQueryListEvent) => setReducedMotion(e.matches);
    
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(mediaQuery, "change", handleChange);
  });

  return reducedMotion;
}
