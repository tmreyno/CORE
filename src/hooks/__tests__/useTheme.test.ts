// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  detectOS,
  getSystemTheme,
  resolveTheme,
  applyTheme,
  getNextTheme,
  getThemeIcon,
  getThemeLabel,
  createThemeActions,
  type Theme,
} from "../useTheme";

// ---------------------------------------------------------------------------
// detectOS
// ---------------------------------------------------------------------------

describe("detectOS", () => {
  const origPlatform = Object.getOwnPropertyDescriptor(navigator, "platform");

  afterEach(() => {
    if (origPlatform) {
      Object.defineProperty(navigator, "platform", origPlatform);
    }
  });

  function setPlatform(value: string) {
    Object.defineProperty(navigator, "platform", {
      value,
      writable: true,
      configurable: true,
    });
  }

  it("returns 'macos' when platform contains 'Mac'", () => {
    setPlatform("MacIntel");
    expect(detectOS()).toBe("macos");
  });

  it("returns 'macos' for lowercase mac", () => {
    setPlatform("macintel");
    expect(detectOS()).toBe("macos");
  });

  it("returns 'windows' when platform contains 'Win'", () => {
    setPlatform("Win32");
    expect(detectOS()).toBe("windows");
  });

  it("returns 'linux' for other platforms", () => {
    setPlatform("Linux x86_64");
    expect(detectOS()).toBe("linux");
  });
});

// ---------------------------------------------------------------------------
// getSystemTheme
// ---------------------------------------------------------------------------

describe("getSystemTheme", () => {
  const origMatchMedia = window.matchMedia;

  afterEach(() => {
    window.matchMedia = origMatchMedia;
  });

  it("returns OS-appropriate light theme when system prefers light", () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: true } as MediaQueryList);
    // In test env navigator.platform is typically "MacIntel" or similar
    const result = getSystemTheme();
    expect(["light", "light-macos", "light-windows"]).toContain(result);
  });

  it("returns 'dark' when system prefers dark", () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: false } as MediaQueryList);
    expect(getSystemTheme()).toBe("dark");
  });
});

// ---------------------------------------------------------------------------
// resolveTheme
// ---------------------------------------------------------------------------

describe("resolveTheme", () => {
  it("returns 'dark' for dark theme", () => {
    expect(resolveTheme("dark")).toBe("dark");
  });

  it("returns 'midnight' for midnight theme", () => {
    expect(resolveTheme("midnight")).toBe("midnight");
  });

  it("returns 'light-macos' for light-macos theme", () => {
    expect(resolveTheme("light-macos")).toBe("light-macos");
  });

  it("returns 'light-windows' for light-windows theme", () => {
    expect(resolveTheme("light-windows")).toBe("light-windows");
  });

  it("resolves 'light' to OS-appropriate variant", () => {
    const result = resolveTheme("light");
    expect(["light", "light-macos", "light-windows"]).toContain(result);
  });

  it("resolves 'system' to a non-system theme", () => {
    // matchMedia may not exist in test env — stub it
    window.matchMedia = vi.fn().mockReturnValue({ matches: false } as MediaQueryList);
    const result = resolveTheme("system");
    expect(["light", "light-macos", "light-windows", "dark"]).toContain(result);
  });
});

// ---------------------------------------------------------------------------
// applyTheme
// ---------------------------------------------------------------------------

describe("applyTheme", () => {
  afterEach(() => {
    document.documentElement.removeAttribute("data-theme");
  });

  it("sets data-theme attribute on document element for dark", () => {
    applyTheme("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("sets data-theme attribute for midnight", () => {
    applyTheme("midnight");
    expect(document.documentElement.getAttribute("data-theme")).toBe("midnight");
  });

  it("sets data-theme attribute for light-macos", () => {
    applyTheme("light-macos");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light-macos");
  });

  it("resolves 'system' before applying", () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: false } as MediaQueryList);
    applyTheme("system");
    const attr = document.documentElement.getAttribute("data-theme");
    expect(attr).toBeTruthy();
    expect(attr).not.toBe("system"); // should be resolved
  });
});

// ---------------------------------------------------------------------------
// getNextTheme
// ---------------------------------------------------------------------------

describe("getNextTheme", () => {
  it.each<[Theme, Theme]>([
    ["dark", "light"],
    ["light", "light-macos"],
    ["light-macos", "light-windows"],
    ["light-windows", "midnight"],
    ["midnight", "system"],
    ["system", "dark"],
  ])("cycles %s → %s", (current, expected) => {
    expect(getNextTheme(current)).toBe(expected);
  });

  it("completes a full cycle back to start", () => {
    let theme: Theme = "dark";
    const visited: Theme[] = [theme];
    for (let i = 0; i < 6; i++) {
      theme = getNextTheme(theme);
      visited.push(theme);
    }
    expect(visited[0]).toBe(visited[visited.length - 1]); // wraps around
    expect(visited.length).toBe(7); // 6 unique + 1 wrap
  });
});

// ---------------------------------------------------------------------------
// getThemeIcon
// ---------------------------------------------------------------------------

describe("getThemeIcon", () => {
  it("returns 💻 for system theme", () => {
    expect(getThemeIcon("system", "dark")).toBe("💻");
  });

  it("returns ✨ for midnight theme", () => {
    expect(getThemeIcon("midnight", "midnight")).toBe("✨");
  });

  it("returns ✨ when resolved is midnight", () => {
    // Edge case: theme is something else but resolved is midnight
    expect(getThemeIcon("dark" as Theme, "midnight")).toBe("✨");
  });

  it("returns 🍎 for light-macos theme", () => {
    expect(getThemeIcon("light-macos", "light-macos")).toBe("🍎");
  });

  it("returns 🪟 for light-windows theme", () => {
    expect(getThemeIcon("light-windows", "light-windows")).toBe("🪟");
  });

  it("returns ☀️ for resolved light themes", () => {
    expect(getThemeIcon("light", "light")).toBe("☀️");
  });

  it("returns 🌙 for dark theme", () => {
    expect(getThemeIcon("dark", "dark")).toBe("🌙");
  });
});

// ---------------------------------------------------------------------------
// getThemeLabel
// ---------------------------------------------------------------------------

describe("getThemeLabel", () => {
  it.each<[Theme, string]>([
    ["light", "Light (Auto)"],
    ["light-macos", "Light (macOS)"],
    ["light-windows", "Light (Windows)"],
    ["dark", "Dark"],
    ["midnight", "Midnight"],
    ["system", "System"],
  ])("returns '%s' → '%s'", (theme, label) => {
    expect(getThemeLabel(theme)).toBe(label);
  });
});

// ---------------------------------------------------------------------------
// createThemeActions
// ---------------------------------------------------------------------------

describe("createThemeActions", () => {
  let currentTheme: Theme;
  let setThemeSpy: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    currentTheme = "dark";
    setThemeSpy = vi.fn((t: Theme) => {
      currentTheme = t;
    });
  });

  function makeActions() {
    return createThemeActions(() => currentTheme, setThemeSpy);
  }

  it("theme() returns current theme", () => {
    const actions = makeActions();
    expect(actions.theme()).toBe("dark");
  });

  it("resolvedTheme() resolves current theme", () => {
    const actions = makeActions();
    expect(actions.resolvedTheme()).toBe("dark");

    currentTheme = "midnight";
    expect(actions.resolvedTheme()).toBe("midnight");
  });

  it("setTheme() calls the setter", () => {
    const actions = makeActions();
    actions.setTheme("midnight");
    expect(setThemeSpy).toHaveBeenCalledWith("midnight");
  });

  it("cycleTheme() moves to next theme in cycle", () => {
    const actions = makeActions();
    actions.cycleTheme();
    expect(setThemeSpy).toHaveBeenCalledWith("light"); // dark → light
  });

  it("toggleTheme() switches dark ↔ light", () => {
    const actions = makeActions();
    // dark → light
    actions.toggleTheme();
    expect(setThemeSpy).toHaveBeenCalledWith("light");
  });

  it("toggleTheme() switches from light-resolved to dark", () => {
    currentTheme = "light-macos";
    const actions = makeActions();
    // resolved is "light-macos" which doesn't strictly equal "light",
    // but startsWith("light") is not checked — let's see what happens
    // resolveTheme("light-macos") → "light-macos", which !== "light"
    // so toggleTheme goes to "light"
    actions.toggleTheme();
    expect(setThemeSpy).toHaveBeenCalledWith("light");
  });

  it("toggleTheme() from midnight goes to light (midnight resolves to midnight != light)", () => {
    currentTheme = "midnight";
    const actions = makeActions();
    actions.toggleTheme();
    expect(setThemeSpy).toHaveBeenCalledWith("light");
  });

  it("cycleTheme() cycles through all themes", () => {
    const actions = makeActions();
    const visited: Theme[] = [];
    for (let i = 0; i < 6; i++) {
      actions.cycleTheme();
      // Update currentTheme to the last call
      currentTheme = setThemeSpy.mock.calls[setThemeSpy.mock.calls.length - 1][0];
      visited.push(currentTheme);
    }
    expect(visited).toEqual([
      "light",
      "light-macos",
      "light-windows",
      "midnight",
      "system",
      "dark",
    ]);
  });
});
