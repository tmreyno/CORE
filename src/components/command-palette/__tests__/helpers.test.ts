// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { formatShortcut, highlightMatch } from "../helpers";

// =============================================================================
// formatShortcut
// =============================================================================

describe("formatShortcut", () => {
  it("replaces cmd with ⌘ symbol", () => {
    expect(formatShortcut("cmd+k")).toBe("⌘K");
  });

  it("replaces ctrl with ⌃ symbol", () => {
    expect(formatShortcut("ctrl+s")).toBe("⌃S");
  });

  it("replaces alt with ⌥ symbol", () => {
    expect(formatShortcut("alt+f")).toBe("⌥F");
  });

  it("replaces shift with ⇧ symbol", () => {
    expect(formatShortcut("shift+a")).toBe("⇧A");
  });

  it("removes plus separators and uppercases", () => {
    expect(formatShortcut("cmd+shift+p")).toBe("⌘⇧P");
  });

  it("handles single key", () => {
    expect(formatShortcut("f1")).toBe("F1");
  });
});

// =============================================================================
// highlightMatch
// =============================================================================

describe("highlightMatch", () => {
  it("returns original text for empty query", () => {
    expect(highlightMatch("Hello", "")).toBe("Hello");
  });

  it("wraps matched characters in mark tags", () => {
    const result = highlightMatch("Hello", "hel");
    expect(result).toContain("<mark");
    expect(result).toContain("H</mark>");
    expect(result).toContain("e</mark>");
    expect(result).toContain("l</mark>");
  });

  it("matches case-insensitively", () => {
    const result = highlightMatch("FileExport", "fe");
    // F and E should be highlighted
    expect(result).toContain("F</mark>");
  });

  it("preserves non-matching characters", () => {
    const result = highlightMatch("abc", "a");
    expect(result).toContain("a</mark>");
    expect(result).toContain("bc");
  });

  it("handles fuzzy matching (non-contiguous characters)", () => {
    const result = highlightMatch("Generate Report", "gr");
    // G from "Generate" and e from "Generate" match
    expect(result).toContain("G</mark>");
  });
});
