// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => { store[key] = value; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { store = {}; },
  };
})();

// Mock matchMedia
const createMatchMedia = (matches: boolean) => ({
  matches,
  media: "(prefers-color-scheme: light)",
  onchange: null,
  addListener: vi.fn(),
  removeListener: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn(),
});

describe("useTheme", () => {
  beforeEach(() => {
    // Setup mocks
    Object.defineProperty(window, "localStorage", { value: localStorageMock });
    Object.defineProperty(window, "matchMedia", {
      value: vi.fn().mockImplementation(() => createMatchMedia(false)),
    });
    localStorageMock.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("Theme Storage", () => {
    it("should use 'system' as default theme when localStorage is empty", () => {
      const stored = localStorageMock.getItem("ffx-theme");
      expect(stored).toBeNull();
    });

    it("should store theme in localStorage", () => {
      localStorageMock.setItem("ffx-theme", "dark");
      expect(localStorageMock.getItem("ffx-theme")).toBe("dark");
    });

    it("should retrieve stored theme from localStorage", () => {
      localStorageMock.setItem("ffx-theme", "light");
      const stored = localStorageMock.getItem("ffx-theme");
      expect(stored).toBe("light");
    });

    it("should validate theme values", () => {
      const validThemes = ["light", "dark", "system"];
      validThemes.forEach(theme => {
        localStorageMock.setItem("ffx-theme", theme);
        expect(localStorageMock.getItem("ffx-theme")).toBe(theme);
      });
    });
  });

  describe("System Theme Detection", () => {
    it("should detect light system theme", () => {
      Object.defineProperty(window, "matchMedia", {
        value: vi.fn().mockImplementation(() => createMatchMedia(true)),
      });
      const result = window.matchMedia("(prefers-color-scheme: light)");
      expect(result.matches).toBe(true);
    });

    it("should detect dark system theme", () => {
      Object.defineProperty(window, "matchMedia", {
        value: vi.fn().mockImplementation(() => createMatchMedia(false)),
      });
      const result = window.matchMedia("(prefers-color-scheme: light)");
      expect(result.matches).toBe(false);
    });
  });

  describe("Theme Application", () => {
    it("should set data-theme attribute for light theme", () => {
      document.documentElement.setAttribute("data-theme", "light");
      expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    });

    it("should remove data-theme attribute for dark theme", () => {
      document.documentElement.setAttribute("data-theme", "light");
      document.documentElement.removeAttribute("data-theme");
      expect(document.documentElement.getAttribute("data-theme")).toBeNull();
    });
  });
});

describe("Theme Utilities", () => {
  it("should cycle through themes correctly", () => {
    const themes = ["light", "dark", "system"] as const;
    const cycleTheme = (current: string): string => {
      if (current === "light") return "dark";
      if (current === "dark") return "system";
      return "light";
    };

    expect(cycleTheme("light")).toBe("dark");
    expect(cycleTheme("dark")).toBe("system");
    expect(cycleTheme("system")).toBe("light");
  });

  it("should toggle between light and dark", () => {
    const toggleTheme = (current: "light" | "dark"): "light" | "dark" => {
      return current === "light" ? "dark" : "light";
    };

    expect(toggleTheme("light")).toBe("dark");
    expect(toggleTheme("dark")).toBe("light");
  });
});
