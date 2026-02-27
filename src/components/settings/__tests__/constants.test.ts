// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  SETTINGS_TABS,
  SHORTCUT_LABELS,
  formatShortcut,
} from "../constants";

// =============================================================================
// SETTINGS_TABS
// =============================================================================
describe("SETTINGS_TABS", () => {
  it("has 8 tabs", () => {
    expect(SETTINGS_TABS).toHaveLength(8);
  });

  it("all tabs have required fields", () => {
    for (const tab of SETTINGS_TABS) {
      expect(tab.id).toBeTruthy();
      expect(tab.label).toBeTruthy();
      expect(tab.icon).toBeTruthy();
    }
  });

  it("has unique tab IDs", () => {
    const ids = SETTINGS_TABS.map(t => t.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("has expected tab IDs", () => {
    const ids = SETTINGS_TABS.map(t => t.id);
    expect(ids).toContain("appearance");
    expect(ids).toContain("defaults");
    expect(ids).toContain("behavior");
    expect(ids).toContain("performance");
    expect(ids).toContain("security");
    expect(ids).toContain("shortcuts");
  });
});

// =============================================================================
// SHORTCUT_LABELS
// =============================================================================
describe("SHORTCUT_LABELS", () => {
  it("has entries for common shortcuts", () => {
    expect(SHORTCUT_LABELS.openCommandPalette).toBeTruthy();
    expect(SHORTCUT_LABELS.save).toBeTruthy();
    expect(SHORTCUT_LABELS.search).toBeTruthy();
    expect(SHORTCUT_LABELS.settings).toBeTruthy();
  });

  it("all values are non-empty strings", () => {
    for (const [_key, value] of Object.entries(SHORTCUT_LABELS)) {
      expect(typeof value).toBe("string");
      expect(value.length).toBeGreaterThan(0);
    }
  });
});

// =============================================================================
// formatShortcut
// =============================================================================
describe("formatShortcut", () => {
  it("replaces Meta with ⌘", () => {
    expect(formatShortcut("Meta+S")).toBe("⌘ S");
  });

  it("replaces Shift with ⇧", () => {
    expect(formatShortcut("Shift+A")).toBe("⇧ A");
  });

  it("replaces Alt with ⌥", () => {
    expect(formatShortcut("Alt+F4")).toBe("⌥ F4");
  });

  it("replaces Control with ⌃", () => {
    expect(formatShortcut("Control+C")).toBe("⌃ C");
  });

  it("replaces + with space", () => {
    expect(formatShortcut("A+B+C")).toBe("A B C");
  });

  it("handles combined modifiers", () => {
    expect(formatShortcut("Meta+Shift+P")).toBe("⌘ ⇧ P");
  });

  it("handles simple key without modifiers", () => {
    expect(formatShortcut("F1")).toBe("F1");
  });

  it("handles empty string", () => {
    expect(formatShortcut("")).toBe("");
  });
});
