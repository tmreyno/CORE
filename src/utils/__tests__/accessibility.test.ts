// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  generateAriaId,
  buttonAriaProps,
  checkboxAriaProps,
  listboxAriaProps,
  optionAriaProps,
  treeAriaProps,
  treeitemAriaProps,
  dialogAriaProps,
  announce,
  initAnnouncer,
  getFocusableElements,
  focusFirst,
  focusLast,
  focusNext,
  focusPrevious,
  prefersReducedMotion,
  visuallyHide,
} from "../accessibility";

// Helper: create a DOM container with focusable children
function createFocusableContainer(): HTMLDivElement {
  const container = document.createElement("div");
  container.innerHTML = `
    <button id="btn1">Button 1</button>
    <button id="btn2">Button 2</button>
    <input id="input1" />
    <a href="#" id="link1">Link</a>
    <button id="btn-disabled" disabled>Disabled</button>
    <div id="div1">Not focusable</div>
    <div id="tabbed" tabindex="0">Tabbed div</div>
  `;
  document.body.appendChild(container);
  return container;
}

describe("accessibility", () => {
  // ===========================================================================
  // generateAriaId
  // ===========================================================================

  describe("generateAriaId", () => {
    it("generates unique IDs with default prefix", () => {
      const id1 = generateAriaId();
      const id2 = generateAriaId();
      expect(id1).toMatch(/^aria-\d+$/);
      expect(id2).toMatch(/^aria-\d+$/);
      expect(id1).not.toBe(id2);
    });

    it("uses custom prefix", () => {
      const id = generateAriaId("custom");
      expect(id).toMatch(/^custom-\d+$/);
    });

    it("generates incrementing IDs", () => {
      const id1 = generateAriaId("test");
      const id2 = generateAriaId("test");
      const num1 = parseInt(id1.split("-")[1]);
      const num2 = parseInt(id2.split("-")[1]);
      expect(num2).toBe(num1 + 1);
    });
  });

  // ===========================================================================
  // ARIA Helpers - buttonAriaProps
  // ===========================================================================

  describe("buttonAriaProps", () => {
    it("returns correct role and label", () => {
      const props = buttonAriaProps({ label: "Save" });
      expect(props.role).toBe("button");
      expect(props["aria-label"]).toBe("Save");
      expect(props.tabIndex).toBe(0);
    });

    it("sets pressed state", () => {
      const props = buttonAriaProps({ label: "Toggle", pressed: true });
      expect(props["aria-pressed"]).toBe(true);
    });

    it("sets expanded state", () => {
      const props = buttonAriaProps({ label: "Menu", expanded: true });
      expect(props["aria-expanded"]).toBe(true);
    });

    it("sets controls reference", () => {
      const props = buttonAriaProps({ label: "Open", controls: "panel-1" });
      expect(props["aria-controls"]).toBe("panel-1");
    });

    it("sets disabled with tabIndex -1", () => {
      const props = buttonAriaProps({ label: "Disabled", disabled: true });
      expect(props["aria-disabled"]).toBe(true);
      expect(props.tabIndex).toBe(-1);
    });

    it("sets describedBy reference", () => {
      const props = buttonAriaProps({ label: "Help", describedBy: "help-text" });
      expect(props["aria-describedby"]).toBe("help-text");
    });
  });

  // ===========================================================================
  // checkboxAriaProps
  // ===========================================================================

  describe("checkboxAriaProps", () => {
    it("returns checkbox role with checked state", () => {
      const props = checkboxAriaProps({ label: "Accept", checked: true });
      expect(props.role).toBe("checkbox");
      expect(props["aria-label"]).toBe("Accept");
      expect(props["aria-checked"]).toBe(true);
      expect(props.tabIndex).toBe(0);
    });

    it("handles unchecked state", () => {
      const props = checkboxAriaProps({ label: "Accept", checked: false });
      expect(props["aria-checked"]).toBe(false);
    });

    it("handles mixed state", () => {
      const props = checkboxAriaProps({ label: "Select All", checked: "mixed" });
      expect(props["aria-checked"]).toBe("mixed");
    });

    it("handles disabled state", () => {
      const props = checkboxAriaProps({ label: "Accept", checked: false, disabled: true });
      expect(props["aria-disabled"]).toBe(true);
      expect(props.tabIndex).toBe(-1);
    });
  });

  // ===========================================================================
  // listboxAriaProps
  // ===========================================================================

  describe("listboxAriaProps", () => {
    it("returns listbox role with label", () => {
      const props = listboxAriaProps({ label: "Options" });
      expect(props.role).toBe("listbox");
      expect(props["aria-label"]).toBe("Options");
      expect(props.tabIndex).toBe(0);
    });

    it("sets required flag", () => {
      const props = listboxAriaProps({ label: "Required", required: true });
      expect(props["aria-required"]).toBe(true);
    });
  });

  // ===========================================================================
  // optionAriaProps
  // ===========================================================================

  describe("optionAriaProps", () => {
    it("returns option role with selected state", () => {
      const props = optionAriaProps({ selected: true });
      expect(props.role).toBe("option");
      expect(props["aria-selected"]).toBe(true);
      expect(props.tabIndex).toBe(0);
    });

    it("unselected option has tabIndex -1", () => {
      const props = optionAriaProps({ selected: false });
      expect(props["aria-selected"]).toBe(false);
      expect(props.tabIndex).toBe(-1);
    });

    it("includes setsize and posinset", () => {
      const props = optionAriaProps({ selected: false, setsize: 5, posinset: 3 });
      expect(props["aria-setsize"]).toBe(5);
      expect(props["aria-posinset"]).toBe(3);
    });
  });

  // ===========================================================================
  // treeAriaProps
  // ===========================================================================

  describe("treeAriaProps", () => {
    it("returns tree role with label", () => {
      const props = treeAriaProps({ label: "File Tree" });
      expect(props.role).toBe("tree");
      expect(props["aria-label"]).toBe("File Tree");
      expect(props.tabIndex).toBe(0);
    });
  });

  // ===========================================================================
  // treeitemAriaProps
  // ===========================================================================

  describe("treeitemAriaProps", () => {
    it("returns treeitem role with full attributes", () => {
      const props = treeitemAriaProps({
        expanded: true,
        selected: true,
        level: 2,
        setsize: 5,
        posinset: 3,
      });
      expect(props.role).toBe("treeitem");
      expect(props["aria-expanded"]).toBe(true);
      expect(props["aria-selected"]).toBe(true);
      expect(props["aria-level"]).toBe(2);
      expect(props["aria-setsize"]).toBe(5);
      expect(props["aria-posinset"]).toBe(3);
      expect(props.tabIndex).toBe(0);
    });

    it("unselected treeitem has tabIndex -1", () => {
      const props = treeitemAriaProps({
        selected: false,
        level: 1,
        setsize: 3,
        posinset: 1,
      });
      expect(props.tabIndex).toBe(-1);
    });

    it("handles collapsed state", () => {
      const props = treeitemAriaProps({
        expanded: false,
        selected: false,
        level: 1,
        setsize: 1,
        posinset: 1,
      });
      expect(props["aria-expanded"]).toBe(false);
    });
  });

  // ===========================================================================
  // dialogAriaProps
  // ===========================================================================

  describe("dialogAriaProps", () => {
    it("returns dialog role for non-modal", () => {
      const props = dialogAriaProps({ labelledBy: "title-1" });
      expect(props.role).toBe("dialog");
      expect(props["aria-labelledby"]).toBe("title-1");
    });

    it("returns alertdialog role for modal", () => {
      const props = dialogAriaProps({ labelledBy: "title-1", modal: true });
      expect(props.role).toBe("alertdialog");
    });

    it("sets describedBy reference", () => {
      const props = dialogAriaProps({ labelledBy: "t", describedBy: "desc-1" });
      expect(props["aria-describedby"]).toBe("desc-1");
    });
  });

  // ===========================================================================
  // Focus Management
  // ===========================================================================

  describe("getFocusableElements", () => {
    let container: HTMLDivElement;

    afterEach(() => {
      container?.remove();
    });

    it("finds focusable elements (buttons, inputs, links, tabindexed)", () => {
      container = createFocusableContainer();
      const elements = getFocusableElements(container);
      // btn1, btn2, input1, link1, tabbed = 5 (btn-disabled excluded, div1 excluded)
      expect(elements.length).toBe(5);
    });

    it("excludes disabled elements", () => {
      container = createFocusableContainer();
      const elements = getFocusableElements(container);
      const ids = elements.map(el => el.id);
      expect(ids).not.toContain("btn-disabled");
    });

    it("includes elements with tabindex=0", () => {
      container = createFocusableContainer();
      const elements = getFocusableElements(container);
      const ids = elements.map(el => el.id);
      expect(ids).toContain("tabbed");
    });

    it("returns empty for container with no focusable elements", () => {
      container = document.createElement("div");
      container.innerHTML = "<div>Not focusable</div><span>Also not</span>";
      document.body.appendChild(container);
      expect(getFocusableElements(container)).toEqual([]);
    });
  });

  describe("focusFirst / focusLast", () => {
    let container: HTMLDivElement;

    afterEach(() => {
      container?.remove();
    });

    it("focusFirst focuses the first focusable element", () => {
      container = createFocusableContainer();
      const result = focusFirst(container);
      expect(result).toBe(true);
      expect(document.activeElement?.id).toBe("btn1");
    });

    it("focusLast focuses the last focusable element", () => {
      container = createFocusableContainer();
      const result = focusLast(container);
      expect(result).toBe(true);
      expect(document.activeElement?.id).toBe("tabbed");
    });

    it("focusFirst returns false for empty container", () => {
      container = document.createElement("div");
      container.innerHTML = "<div>No buttons</div>";
      document.body.appendChild(container);
      expect(focusFirst(container)).toBe(false);
    });

    it("focusLast returns false for empty container", () => {
      container = document.createElement("div");
      container.innerHTML = "<div>No buttons</div>";
      document.body.appendChild(container);
      expect(focusLast(container)).toBe(false);
    });
  });

  describe("focusNext / focusPrevious", () => {
    let container: HTMLDivElement;

    afterEach(() => {
      container?.remove();
    });

    it("focusNext moves to next element", () => {
      container = createFocusableContainer();
      // Focus first element
      const btn1 = container.querySelector("#btn1") as HTMLElement;
      btn1.focus();
      const result = focusNext(container);
      expect(result).toBe(true);
      expect(document.activeElement?.id).toBe("btn2");
    });

    it("focusNext wraps around to first", () => {
      container = createFocusableContainer();
      const last = container.querySelector("#tabbed") as HTMLElement;
      last.focus();
      const result = focusNext(container, true);
      expect(result).toBe(true);
      expect(document.activeElement?.id).toBe("btn1");
    });

    it("focusNext does not wrap when wrap=false", () => {
      container = createFocusableContainer();
      const last = container.querySelector("#tabbed") as HTMLElement;
      last.focus();
      const result = focusNext(container, false);
      expect(result).toBe(false);
    });

    it("focusPrevious moves to previous element", () => {
      container = createFocusableContainer();
      const btn2 = container.querySelector("#btn2") as HTMLElement;
      btn2.focus();
      const result = focusPrevious(container);
      expect(result).toBe(true);
      expect(document.activeElement?.id).toBe("btn1");
    });

    it("focusPrevious wraps around to last", () => {
      container = createFocusableContainer();
      const btn1 = container.querySelector("#btn1") as HTMLElement;
      btn1.focus();
      const result = focusPrevious(container, true);
      expect(result).toBe(true);
      expect(document.activeElement?.id).toBe("tabbed");
    });

    it("focusPrevious does not wrap when wrap=false", () => {
      container = createFocusableContainer();
      const btn1 = container.querySelector("#btn1") as HTMLElement;
      btn1.focus();
      const result = focusPrevious(container, false);
      expect(result).toBe(false);
    });

    it("focusNext falls back to focusFirst when no active element in container", () => {
      container = createFocusableContainer();
      // Don't focus anything inside the container
      document.body.focus();
      focusNext(container);
      expect(document.activeElement?.id).toBe("btn1");
    });

    it("focusPrevious falls back to focusLast when no active element in container", () => {
      container = createFocusableContainer();
      document.body.focus();
      focusPrevious(container);
      expect(document.activeElement?.id).toBe("tabbed");
    });
  });

  // ===========================================================================
  // Screen Reader Announcements
  // ===========================================================================

  describe("announce", () => {
    it("creates announcer element with correct ARIA attributes on first call", () => {
      // announce() internally calls initAnnouncer() which creates the DOM element
      announce("hello screen reader");
      const el = document.querySelector('[role="status"]');
      expect(el).not.toBeNull();
      expect(el?.getAttribute("aria-live")).toBe("polite");
      expect(el?.getAttribute("aria-atomic")).toBe("true");
      // Element should be visually hidden (1px x 1px)
      const style = el?.getAttribute("style") ?? "";
      expect(style).toContain("width: 1px");
      expect(style).toContain("height: 1px");
    });

    it("updates aria-live for assertive priority", () => {
      // The module-level announcer persists, so subsequent calls reuse it
      announce("urgent!", "assertive");
      const el = document.querySelector('[role="status"]');
      expect(el?.getAttribute("aria-live")).toBe("assertive");
    });

    it("resets to polite priority", () => {
      announce("calm", "polite");
      const el = document.querySelector('[role="status"]');
      expect(el?.getAttribute("aria-live")).toBe("polite");
    });

    it("idempotent initAnnouncer does not create duplicate elements", () => {
      const before = document.querySelectorAll('[role="status"]').length;
      initAnnouncer();
      initAnnouncer();
      const after = document.querySelectorAll('[role="status"]').length;
      expect(after).toBe(before);
    });
  });

  // ===========================================================================
  // Visibility helpers
  // ===========================================================================

  describe("visuallyHide", () => {
    it("applies visually hidden styles", () => {
      const el = document.createElement("div");
      document.body.appendChild(el);
      visuallyHide(el);
      expect(el.style.position).toBe("absolute");
      expect(el.style.width).toBe("1px");
      expect(el.style.height).toBe("1px");
      expect(el.style.overflow).toBe("hidden");
      el.remove();
    });
  });

  describe("prefersReducedMotion", () => {
    it("returns a boolean", () => {
      // jsdom doesn't implement matchMedia, so mock it
      const original = window.matchMedia;
      window.matchMedia = vi.fn().mockReturnValue({ matches: false }) as unknown as typeof window.matchMedia;
      try {
        const result = prefersReducedMotion();
        expect(typeof result).toBe("boolean");
        expect(result).toBe(false);
      } finally {
        window.matchMedia = original;
      }
    });

    it("returns true when reduced motion is preferred", () => {
      const original = window.matchMedia;
      window.matchMedia = vi.fn().mockReturnValue({ matches: true }) as unknown as typeof window.matchMedia;
      try {
        expect(prefersReducedMotion()).toBe(true);
      } finally {
        window.matchMedia = original;
      }
    });
  });
});
