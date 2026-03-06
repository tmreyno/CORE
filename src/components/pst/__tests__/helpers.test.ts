// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { formatEmailDate, importanceLabel } from "../helpers";

// =============================================================================
// formatEmailDate
// =============================================================================

describe("formatEmailDate", () => {
  it("returns empty string for null", () => {
    expect(formatEmailDate(null)).toBe("");
  });

  it("formats a valid date string", () => {
    const result = formatEmailDate("2025-06-15T10:30:00Z");
    expect(result).toContain("2025");
    expect(result).toContain("15");
  });

  it("returns 'Invalid Date' for unparseable string", () => {
    // new Date("not-a-date") returns Invalid Date without throwing
    expect(formatEmailDate("not-a-date")).toBe("Invalid Date");
  });

  it("returns empty string for empty string input", () => {
    // empty string is falsy, so should return ""
    expect(formatEmailDate("")).toBe("");
  });
});

// =============================================================================
// importanceLabel
// =============================================================================

describe("importanceLabel", () => {
  it("returns 'Low' for importance 0", () => {
    expect(importanceLabel(0)).toBe("Low");
  });

  it("returns 'High' for importance 2", () => {
    expect(importanceLabel(2)).toBe("High");
  });

  it("returns null for Normal importance (1)", () => {
    expect(importanceLabel(1)).toBeNull();
  });

  it("returns null for null input", () => {
    expect(importanceLabel(null)).toBeNull();
  });

  it("returns null for other values", () => {
    expect(importanceLabel(3)).toBeNull();
    expect(importanceLabel(99)).toBeNull();
  });
});
