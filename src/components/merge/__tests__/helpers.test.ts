// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  friendlyTemplateName,
  basename,
  formatDate,
  fmtBytes,
  roleBadgeClass,
  TEMPLATE_DISPLAY_NAMES,
} from "../helpers";

// =============================================================================
// friendlyTemplateName
// =============================================================================

describe("friendlyTemplateName", () => {
  it("maps known template IDs to display names", () => {
    expect(friendlyTemplateName("evidence_collection")).toBe("Evidence Collection");
    expect(friendlyTemplateName("iar")).toBe("Investigative Activity Report");
    expect(friendlyTemplateName("user_activity")).toBe("User Activity Log");
    expect(friendlyTemplateName("chain_of_custody")).toBe("Chain of Custody");
  });

  it("title-cases unknown template IDs replacing underscores", () => {
    expect(friendlyTemplateName("my_custom_template")).toBe("My Custom Template");
    expect(friendlyTemplateName("single")).toBe("Single");
  });

  it("covers all entries in TEMPLATE_DISPLAY_NAMES", () => {
    for (const [key, value] of Object.entries(TEMPLATE_DISPLAY_NAMES)) {
      expect(friendlyTemplateName(key)).toBe(value);
    }
  });
});

// =============================================================================
// basename
// =============================================================================

describe("basename", () => {
  it("extracts filename from a path", () => {
    expect(basename("/Users/test/Documents/project.cffx")).toBe("project.cffx");
  });

  it("handles paths with no slashes", () => {
    expect(basename("file.txt")).toBe("file.txt");
  });

  it("handles trailing slash by falling back to full path", () => {
    // split("/").pop() on "/path/" returns "" (falsy) → || path
    const result = basename("/path/");
    expect(result).toBe("/path/");
  });

  it("returns original path when pop returns empty", () => {
    // basename("") would have pop() return "", then || path returns ""
    expect(basename("")).toBe("");
  });
});

// =============================================================================
// formatDate
// =============================================================================

describe("formatDate", () => {
  it("formats a valid ISO date string", () => {
    const result = formatDate("2025-06-15T10:30:00Z");
    // Result depends on locale, but should contain the year and month
    expect(result).toContain("2025");
    expect(result).toContain("15");
  });

  it("returns 'Invalid Date' for unparseable strings", () => {
    // new Date("not-a-date-string") returns Invalid Date without throwing
    expect(formatDate("not-a-date-string")).toBe("Invalid Date");
  });

  it("returns 'Invalid Date' for short invalid strings", () => {
    expect(formatDate("bad")).toBe("Invalid Date");
  });
});

// =============================================================================
// fmtBytes
// =============================================================================

describe("fmtBytes", () => {
  it("returns '0 B' for zero bytes", () => {
    expect(fmtBytes(0)).toBe("0 B");
  });

  it("formats bytes correctly", () => {
    expect(fmtBytes(512)).toBe("512 B");
  });

  it("formats kilobytes", () => {
    expect(fmtBytes(1024)).toBe("1.0 KB");
    expect(fmtBytes(1536)).toBe("1.5 KB");
  });

  it("formats megabytes", () => {
    expect(fmtBytes(1048576)).toBe("1.0 MB");
  });

  it("formats gigabytes", () => {
    expect(fmtBytes(1073741824)).toBe("1.0 GB");
  });

  it("formats terabytes", () => {
    expect(fmtBytes(1099511627776)).toBe("1.0 TB");
  });
});

// =============================================================================
// roleBadgeClass
// =============================================================================

describe("roleBadgeClass", () => {
  it("returns success badge for project owner", () => {
    expect(roleBadgeClass("project owner")).toBe("badge badge-success");
  });

  it("returns plain badge for session user", () => {
    expect(roleBadgeClass("session user")).toBe("badge");
  });

  it("returns warning badge for COC-related roles", () => {
    expect(roleBadgeClass("submitted by (COC)")).toBe("badge badge-warning");
    expect(roleBadgeClass("received by (COC)")).toBe("badge badge-warning");
  });

  it("returns plain badge for unknown roles", () => {
    expect(roleBadgeClass("AXIOM examiner")).toBe("badge");
    expect(roleBadgeClass("bookmark author")).toBe("badge");
  });
});
