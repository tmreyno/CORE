// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  getTypeColor,
  getDepth,
  getKeyName,
  isContainerType,
  NOTABLE_KEY_PREFIXES,
} from "../helpers";

// =============================================================================
// getTypeColor
// =============================================================================

describe("getTypeColor", () => {
  it("returns green for String type", () => {
    expect(getTypeColor("String")).toBe("text-green-400");
  });

  it("returns blue for Integer type", () => {
    expect(getTypeColor("Integer")).toBe("text-blue-400");
  });

  it("returns blue for Real type", () => {
    expect(getTypeColor("Real")).toBe("text-blue-400");
  });

  it("returns yellow for Boolean type", () => {
    expect(getTypeColor("Boolean")).toBe("text-yellow-400");
  });

  it("returns purple for Date type", () => {
    expect(getTypeColor("Date")).toBe("text-purple-400");
  });

  it("returns orange for Data type", () => {
    expect(getTypeColor("Data")).toBe("text-orange-400");
  });

  it("returns cyan for Array types", () => {
    expect(getTypeColor("Array (5 items)")).toBe("text-cyan-400");
  });

  it("returns pink for Dictionary types", () => {
    expect(getTypeColor("Dictionary (10 items)")).toBe("text-pink-400");
  });

  it("returns muted for unknown types", () => {
    expect(getTypeColor("SomeUnknown")).toBe("text-txt-muted");
  });
});

// =============================================================================
// getDepth
// =============================================================================

describe("getDepth", () => {
  it("returns 0 for root-level key", () => {
    expect(getDepth("CFBundleIdentifier")).toBe(0);
  });

  it("returns 1 for one level deep", () => {
    expect(getDepth("Root/CFBundleIdentifier")).toBe(1);
  });

  it("returns correct depth for nested paths", () => {
    expect(getDepth("Root/Nested/DeepKey")).toBe(2);
  });

  it("handles leading and trailing slashes", () => {
    // filter(Boolean) removes empty strings from leading/trailing /
    expect(getDepth("/Root/Key/")).toBe(1);
  });
});

// =============================================================================
// getKeyName
// =============================================================================

describe("getKeyName", () => {
  it("returns leaf key from path", () => {
    expect(getKeyName("Root/Nested/MyKey")).toBe("MyKey");
  });

  it("returns the key itself when no path separator", () => {
    expect(getKeyName("SimpleKey")).toBe("SimpleKey");
  });

  it("handles empty string", () => {
    expect(getKeyName("")).toBe("");
  });

  it("handles trailing slash by returning last non-empty segment", () => {
    expect(getKeyName("Root/Key/")).toBe("Key");
  });
});

// =============================================================================
// isContainerType
// =============================================================================

describe("isContainerType", () => {
  it("returns true for Array types", () => {
    expect(isContainerType("Array")).toBe(true);
    expect(isContainerType("Array (5 items)")).toBe(true);
  });

  it("returns true for Dictionary types", () => {
    expect(isContainerType("Dictionary")).toBe(true);
    expect(isContainerType("Dictionary (10 items)")).toBe(true);
  });

  it("returns false for scalar types", () => {
    expect(isContainerType("String")).toBe(false);
    expect(isContainerType("Integer")).toBe(false);
    expect(isContainerType("Boolean")).toBe(false);
    expect(isContainerType("Data")).toBe(false);
  });
});

// =============================================================================
// NOTABLE_KEY_PREFIXES
// =============================================================================

describe("NOTABLE_KEY_PREFIXES", () => {
  it("contains expected forensic-relevant key prefixes", () => {
    expect(NOTABLE_KEY_PREFIXES).toContain("CFBundleIdentifier");
    expect(NOTABLE_KEY_PREFIXES).toContain("CFBundleName");
    expect(NOTABLE_KEY_PREFIXES).toContain("MinimumOSVersion");
  });

  it("is a non-empty array", () => {
    expect(NOTABLE_KEY_PREFIXES.length).toBeGreaterThan(0);
  });
});
