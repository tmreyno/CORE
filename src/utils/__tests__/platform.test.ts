// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { afterEach, describe, it, expect, vi } from "vitest";
import { formatShortcut, platform } from "../platform";

type TauriWindow = Window & {
  __TAURI_INTERNALS__?: unknown;
  __TAURI__?: unknown;
};

const originalTauriInternals = Object.getOwnPropertyDescriptor(window, "__TAURI_INTERNALS__");
const originalTauri = Object.getOwnPropertyDescriptor(window, "__TAURI__");

function restoreTauriGlobals() {
  if (originalTauriInternals) {
    Object.defineProperty(window, "__TAURI_INTERNALS__", originalTauriInternals);
  } else {
    delete (window as TauriWindow).__TAURI_INTERNALS__;
  }

  if (originalTauri) {
    Object.defineProperty(window, "__TAURI__", originalTauri);
  } else {
    delete (window as TauriWindow).__TAURI__;
  }
}

afterEach(() => {
  vi.resetModules();
  restoreTauriGlobals();
});

// =============================================================================
// formatShortcut
// =============================================================================

describe("formatShortcut", () => {
  // Tests are platform-aware: results depend on whether tests run on Mac or not
  const isMac = platform.isMac;

  it("formats a single modifier + key", () => {
    const result = formatShortcut(["mod"], "s");
    if (isMac) {
      expect(result).toBe("⌘S");
    } else {
      expect(result).toBe("Ctrl+S");
    }
  });

  it("formats multiple modifiers", () => {
    const result = formatShortcut(["mod", "shift"], "z");
    if (isMac) {
      expect(result).toBe("⌘⇧Z");
    } else {
      expect(result).toBe("Ctrl+Shift+Z");
    }
  });

  it("formats alt/option modifier", () => {
    const result = formatShortcut(["alt"], "p");
    if (isMac) {
      expect(result).toBe("⌥P");
    } else {
      expect(result).toBe("Alt+P");
    }
  });

  it("formats ctrl modifier", () => {
    const result = formatShortcut(["ctrl"], "c");
    if (isMac) {
      expect(result).toBe("⌃C");
    } else {
      expect(result).toBe("Ctrl+C");
    }
  });

  it("accepts 'cmd' as alias for 'mod'", () => {
    const result = formatShortcut(["cmd"], "n");
    if (isMac) {
      expect(result).toBe("⌘N");
    } else {
      expect(result).toBe("Ctrl+N");
    }
  });

  it("accepts 'command' as alias for 'mod'", () => {
    const result = formatShortcut(["command"], "o");
    if (isMac) {
      expect(result).toBe("⌘O");
    } else {
      expect(result).toBe("Ctrl+O");
    }
  });

  it("accepts 'option' as alias for 'alt'", () => {
    const result = formatShortcut(["option"], "f");
    if (isMac) {
      expect(result).toBe("⌥F");
    } else {
      expect(result).toBe("Alt+F");
    }
  });

  it("accepts 'control' as alias for 'ctrl'", () => {
    const result = formatShortcut(["control"], "a");
    if (isMac) {
      expect(result).toBe("⌃A");
    } else {
      expect(result).toBe("Ctrl+A");
    }
  });

  it("uppercases the key", () => {
    const result = formatShortcut([], "a");
    expect(result).toBe("A");
  });

  it("handles empty modifiers", () => {
    const result = formatShortcut([], "f5");
    expect(result).toBe("F5");
  });

  it("passes through unknown modifier names as-is", () => {
    const result = formatShortcut(["super"], "x");
    if (isMac) {
      expect(result).toBe("superX");
    } else {
      expect(result).toBe("super+X");
    }
  });

  it("formats three modifiers", () => {
    const result = formatShortcut(["mod", "shift", "alt"], "k");
    if (isMac) {
      expect(result).toBe("⌘⇧⌥K");
    } else {
      expect(result).toBe("Ctrl+Shift+Alt+K");
    }
  });

  it("is case-insensitive for modifier names", () => {
    const result1 = formatShortcut(["MOD"], "s");
    const result2 = formatShortcut(["Mod"], "s");
    const result3 = formatShortcut(["mod"], "s");
    expect(result1).toBe(result2);
    expect(result2).toBe(result3);
  });
});

// =============================================================================
// platform object
// =============================================================================

describe("platform", () => {
  it("has a name property", () => {
    expect(["mac", "windows", "linux", "ios", "android", "unknown"]).toContain(
      platform.name
    );
  });

  it("has modifierKey", () => {
    expect(typeof platform.modifierKey).toBe("string");
    expect(platform.modifierKey.length).toBeGreaterThan(0);
  });

  it("has altKey", () => {
    expect(typeof platform.altKey).toBe("string");
  });

  it("has shiftKey", () => {
    expect(typeof platform.shiftKey).toBe("string");
  });

  it("has ctrlKey", () => {
    expect(typeof platform.ctrlKey).toBe("string");
  });

  it("isBrowser is defined", () => {
    expect(typeof platform.isBrowser).toBe("boolean");
  });

  it("isDesktop is the inverse of isMobile", () => {
    expect(platform.isDesktop).toBe(!platform.isMobile);
  });
});

describe("isTauri detection", () => {
  async function importWithTauriInternals(internals?: unknown) {
    vi.resetModules();
    delete (window as TauriWindow).__TAURI_INTERNALS__;
    delete (window as TauriWindow).__TAURI__;

    if (internals !== undefined) {
      Object.defineProperty(window, "__TAURI_INTERNALS__", {
        configurable: true,
        value: internals,
      });
    }

    return import("../platform");
  }

  it("ignores partial Tauri markers in plain browser dev", async () => {
    const { isTauri } = await importWithTauriInternals({});

    expect(isTauri).toBe(false);
  });

  it("requires the native invoke and callback bridge", async () => {
    const { isTauri } = await importWithTauriInternals({
      invoke: vi.fn(),
      transformCallback: vi.fn(),
    });

    expect(isTauri).toBe(true);
  });
});
