// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  getTargetTypeLabel,
  getBookmarkColorClass,
  BOOKMARK_COLORS,
} from "../helpers";

// =============================================================================
// getTargetTypeLabel
// =============================================================================

describe("getTargetTypeLabel", () => {
  it("returns 'Files' for file type", () => {
    expect(getTargetTypeLabel("file")).toBe("Files");
  });

  it("returns 'Artifacts' for artifact type", () => {
    expect(getTargetTypeLabel("artifact")).toBe("Artifacts");
  });

  it("returns 'Search Results' for search_result type", () => {
    expect(getTargetTypeLabel("search_result")).toBe("Search Results");
  });

  it("returns 'Locations' for location type", () => {
    expect(getTargetTypeLabel("location")).toBe("Locations");
  });

  it("returns 'Other' for unknown types", () => {
    expect(getTargetTypeLabel("unknown" as any)).toBe("Other");
  });
});

// =============================================================================
// getBookmarkColorClass
// =============================================================================

describe("getBookmarkColorClass", () => {
  it("returns 'text-accent' for undefined color", () => {
    expect(getBookmarkColorClass(undefined)).toBe("text-accent");
  });

  it("returns 'text-accent' for empty string", () => {
    expect(getBookmarkColorClass("")).toBe("text-accent");
  });

  it("maps red to text-error", () => {
    expect(getBookmarkColorClass("red")).toBe("text-error");
  });

  it("maps yellow to text-warning", () => {
    expect(getBookmarkColorClass("yellow")).toBe("text-warning");
  });

  it("maps green to text-success", () => {
    expect(getBookmarkColorClass("green")).toBe("text-success");
  });

  it("maps blue to text-info", () => {
    expect(getBookmarkColorClass("blue")).toBe("text-info");
  });

  it("maps purple to text-accent", () => {
    expect(getBookmarkColorClass("purple")).toBe("text-accent");
  });

  it("maps orange to text-type-ad1", () => {
    expect(getBookmarkColorClass("orange")).toBe("text-type-ad1");
  });

  it("is case-insensitive", () => {
    expect(getBookmarkColorClass("RED")).toBe("text-error");
    expect(getBookmarkColorClass("Blue")).toBe("text-info");
  });

  it("returns fallback for unknown colours", () => {
    expect(getBookmarkColorClass("magenta")).toBe("text-accent");
  });
});

// =============================================================================
// BOOKMARK_COLORS
// =============================================================================

describe("BOOKMARK_COLORS", () => {
  it("contains 7 color options", () => {
    expect(BOOKMARK_COLORS).toHaveLength(7);
  });

  it("first entry is the default (empty value)", () => {
    expect(BOOKMARK_COLORS[0].value).toBe("");
    expect(BOOKMARK_COLORS[0].label).toBe("Default");
  });

  it("every entry has value, label, and class", () => {
    for (const c of BOOKMARK_COLORS) {
      expect(typeof c.value).toBe("string");
      expect(typeof c.label).toBe("string");
      expect(typeof c.class).toBe("string");
    }
  });
});
