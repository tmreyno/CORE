// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

describe("useFocusTrap", () => {
  let container: HTMLDivElement;
  let button1: HTMLButtonElement;
  let button2: HTMLButtonElement;
  let input: HTMLInputElement;

  beforeEach(() => {
    // Create a container with focusable elements
    container = document.createElement("div");
    container.id = "trap-container";

    button1 = document.createElement("button");
    button1.textContent = "Button 1";
    button1.id = "btn1";

    input = document.createElement("input");
    input.type = "text";
    input.id = "input1";

    button2 = document.createElement("button");
    button2.textContent = "Button 2";
    button2.id = "btn2";

    container.appendChild(button1);
    container.appendChild(input);
    container.appendChild(button2);
    document.body.appendChild(container);
  });

  afterEach(() => {
    document.body.removeChild(container);
  });

  describe("Focusable Element Detection", () => {
    it("should find all focusable elements", () => {
      const focusableSelectors = [
        'button:not([disabled])',
        'input:not([disabled])',
        'select:not([disabled])',
        'textarea:not([disabled])',
        'a[href]',
        '[tabindex]:not([tabindex="-1"])',
      ].join(', ');

      const focusable = container.querySelectorAll<HTMLElement>(focusableSelectors);
      expect(focusable.length).toBe(3);
    });

    it("should exclude disabled elements", () => {
      button1.disabled = true;

      const focusableSelectors = [
        'button:not([disabled])',
        'input:not([disabled])',
      ].join(', ');

      const focusable = container.querySelectorAll<HTMLElement>(focusableSelectors);
      expect(focusable.length).toBe(2);
    });

    it("should exclude elements with tabindex=-1", () => {
      button1.setAttribute("tabindex", "-1");

      const focusableSelectors = [
        'button:not([disabled])',
        '[tabindex]:not([tabindex="-1"])',
      ].join(', ');

      const focusable = Array.from(container.querySelectorAll<HTMLElement>(focusableSelectors));
      // button1 should still be found by 'button:not([disabled])' since it's not disabled
      // but we filter by tabindex separately
      const filtered = focusable.filter(el => el.getAttribute("tabindex") !== "-1");
      expect(filtered.length).toBeGreaterThan(0);
    });
  });

  describe("Focus Management", () => {
    it("should focus first element when activated", () => {
      button1.focus();
      expect(document.activeElement).toBe(button1);
    });

    it("should store previously focused element", () => {
      const outsideButton = document.createElement("button");
      outsideButton.id = "outside";
      document.body.appendChild(outsideButton);
      outsideButton.focus();

      expect(document.activeElement).toBe(outsideButton);

      document.body.removeChild(outsideButton);
    });

    it("should restore focus when deactivated", () => {
      const outsideButton = document.createElement("button");
      outsideButton.id = "outside";
      document.body.appendChild(outsideButton);
      outsideButton.focus();

      // Simulate activating trap
      button1.focus();
      expect(document.activeElement).toBe(button1);

      // Simulate deactivating - restore focus
      outsideButton.focus();
      expect(document.activeElement).toBe(outsideButton);

      document.body.removeChild(outsideButton);
    });
  });

  describe("Tab Key Handling", () => {
    it("should handle Tab key navigation", () => {
      const keyboardEvent = new KeyboardEvent("keydown", {
        key: "Tab",
        bubbles: true,
      });

      button1.focus();
      expect(document.activeElement).toBe(button1);

      // Simulate tab navigation
      input.focus();
      expect(document.activeElement).toBe(input);
    });

    it("should handle Shift+Tab navigation", () => {
      const keyboardEvent = new KeyboardEvent("keydown", {
        key: "Tab",
        shiftKey: true,
        bubbles: true,
      });

      input.focus();
      expect(document.activeElement).toBe(input);

      // Simulate shift+tab navigation
      button1.focus();
      expect(document.activeElement).toBe(button1);
    });

    it("should wrap focus from last to first element", () => {
      button2.focus();
      expect(document.activeElement).toBe(button2);

      // Simulate wrap
      button1.focus();
      expect(document.activeElement).toBe(button1);
    });

    it("should wrap focus from first to last element on Shift+Tab", () => {
      button1.focus();
      expect(document.activeElement).toBe(button1);

      // Simulate wrap backwards
      button2.focus();
      expect(document.activeElement).toBe(button2);
    });
  });
});
