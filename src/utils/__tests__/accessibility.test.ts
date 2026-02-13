// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, afterEach } from "vitest";
import {
  announce,
  initAnnouncer,
  getFocusableElements,
} from "../accessibility";

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
  afterEach(() => {
    document.body.innerHTML = "";
  });

  // ===========================================================================
  // initAnnouncer
  // ===========================================================================

  describe("initAnnouncer", () => {
    it("creates a live region element in the DOM", () => {
      initAnnouncer();
      const el = document.querySelector('[role="status"]');
      expect(el).toBeTruthy();
      expect(el?.getAttribute("aria-live")).toBe("polite");
      expect(el?.getAttribute("aria-atomic")).toBe("true");
    });

    it("does not create duplicate announcers", () => {
      initAnnouncer();
      initAnnouncer();
      const elements = document.querySelectorAll('[role="status"]');
      expect(elements.length).toBe(1);
    });
  });

  // ===========================================================================
  // announce
  // ===========================================================================

  describe("announce", () => {
    it("sets text content on the announcer", async () => {
      initAnnouncer();
      announce("File loaded");
      // announce uses requestAnimationFrame
      await new Promise((r) => requestAnimationFrame(r));
      const el = document.querySelector('[role="status"]');
      expect(el?.textContent).toBe("File loaded");
    });

    it("creates announcer if not initialized", async () => {
      announce("Auto-init test");
      await new Promise((r) => requestAnimationFrame(r));
      const el = document.querySelector('[role="status"]');
      expect(el).toBeTruthy();
      expect(el?.textContent).toBe("Auto-init test");
    });

    it("sets assertive priority", async () => {
      initAnnouncer();
      announce("Error occurred", "assertive");
      const el = document.querySelector('[role="status"]');
      expect(el?.getAttribute("aria-live")).toBe("assertive");
    });
  });

  // ===========================================================================
  // getFocusableElements
  // ===========================================================================

  describe("getFocusableElements", () => {
    it("returns only focusable elements", () => {
      const container = createFocusableContainer();
      const elements = getFocusableElements(container);
      const ids = elements.map((el) => el.id);
      // btn1, btn2, input1, link1, tabbed (not disabled btn, not div1)
      expect(ids).toContain("btn1");
      expect(ids).toContain("btn2");
      expect(ids).toContain("input1");
      expect(ids).toContain("link1");
      expect(ids).toContain("tabbed");
      expect(ids).not.toContain("btn-disabled");
      expect(ids).not.toContain("div1");
    });

    it("returns empty array for container with no focusable elements", () => {
      const container = document.createElement("div");
      container.innerHTML = "<div>Not focusable</div><span>Also not</span>";
      const elements = getFocusableElements(container);
      expect(elements).toEqual([]);
    });
  });
});
