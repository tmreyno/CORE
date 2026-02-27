// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, afterEach } from "vitest";
import {
  initAnnouncer,
  announce,
  getFocusableElements,
} from "./accessibility";

// =============================================================================
// initAnnouncer
// =============================================================================

describe("initAnnouncer", () => {
  afterEach(() => {
    // Clean up any announcer elements
    const announcers = document.querySelectorAll('[role="status"][aria-live]');
    announcers.forEach((el) => el.remove());
  });

  it("creates an announcer element in the DOM", () => {
    initAnnouncer();
    const el = document.querySelector('[role="status"][aria-live="polite"]');
    expect(el).not.toBeNull();
  });

  it("sets aria-atomic to true", () => {
    initAnnouncer();
    const el = document.querySelector('[role="status"]');
    expect(el?.getAttribute("aria-atomic")).toBe("true");
  });

  it("hides the announcer visually", () => {
    initAnnouncer();
    const el = document.querySelector('[role="status"]') as HTMLElement;
    expect(el.style.position).toBe("absolute");
    expect(el.style.overflow).toBe("hidden");
  });

  it("does not create duplicate announcers on repeated calls", () => {
    initAnnouncer();
    initAnnouncer();
    const all = document.querySelectorAll('[role="status"][aria-live]');
    expect(all.length).toBe(1);
  });
});

// =============================================================================
// announce
// =============================================================================

describe("announce", () => {
  afterEach(() => {
    const announcers = document.querySelectorAll('[role="status"][aria-live]');
    announcers.forEach((el) => el.remove());
  });

  it("creates announcer if not initialized", () => {
    announce("Hello");
    const el = document.querySelector('[role="status"]');
    expect(el).not.toBeNull();
  });

  it("clears text content before setting (for re-read)", () => {
    initAnnouncer();
    const el = document.querySelector('[role="status"]') as HTMLElement;
    announce("First message");
    // After announce(), textContent is cleared synchronously
    // then set via requestAnimationFrame
    expect(el.textContent).toBe("");
  });

  it("sets aria-live priority", () => {
    initAnnouncer();
    announce("Urgent!", "assertive");
    const el = document.querySelector('[role="status"]') as HTMLElement;
    expect(el.getAttribute("aria-live")).toBe("assertive");
  });

  it("defaults to polite priority", () => {
    initAnnouncer();
    announce("Info message");
    const el = document.querySelector('[role="status"]') as HTMLElement;
    expect(el.getAttribute("aria-live")).toBe("polite");
  });
});

// =============================================================================
// getFocusableElements
// =============================================================================

describe("getFocusableElements", () => {
  it("finds buttons", () => {
    const container = document.createElement("div");
    container.innerHTML = '<button>Click</button>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(1);
    expect(result[0].tagName).toBe("BUTTON");
  });

  it("excludes disabled buttons", () => {
    const container = document.createElement("div");
    container.innerHTML = '<button disabled>Click</button>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(0);
  });

  it("finds links with href", () => {
    const container = document.createElement("div");
    container.innerHTML = '<a href="#">Link</a>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(1);
  });

  it("excludes links without href", () => {
    const container = document.createElement("div");
    container.innerHTML = '<a>Not focusable</a>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(0);
  });

  it("finds inputs", () => {
    const container = document.createElement("div");
    container.innerHTML = '<input type="text" />';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(1);
  });

  it("excludes disabled inputs", () => {
    const container = document.createElement("div");
    container.innerHTML = '<input type="text" disabled />';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(0);
  });

  it("finds elements with positive tabindex", () => {
    const container = document.createElement("div");
    container.innerHTML = '<div tabindex="0">Focusable</div>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(1);
  });

  it("excludes elements with tabindex -1", () => {
    const container = document.createElement("div");
    container.innerHTML = '<div tabindex="-1">Not focusable</div>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(0);
  });

  it("finds selects and textareas", () => {
    const container = document.createElement("div");
    container.innerHTML = '<select><option>A</option></select><textarea></textarea>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(2);
  });

  it("finds contenteditable elements", () => {
    const container = document.createElement("div");
    container.innerHTML = '<div contenteditable="true">Editable</div>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(1);
  });

  it("returns empty array for container with no focusable elements", () => {
    const container = document.createElement("div");
    container.innerHTML = '<div>Not focusable</div><span>Also not</span>';
    const result = getFocusableElements(container);
    expect(result).toHaveLength(0);
  });

  it("finds multiple mixed focusable elements", () => {
    const container = document.createElement("div");
    container.innerHTML = `
      <button>Btn</button>
      <input type="text" />
      <a href="#">Link</a>
      <div>Not focusable</div>
    `;
    const result = getFocusableElements(container);
    expect(result).toHaveLength(3);
  });
});
