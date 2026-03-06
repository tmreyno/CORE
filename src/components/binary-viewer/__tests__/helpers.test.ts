// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  formatHex,
  formatTimestamp,
  formatBadge,
} from "../helpers";

// =============================================================================
// formatHex
// =============================================================================

describe("formatHex", () => {
  it("returns 'N/A' for null", () => {
    expect(formatHex(null)).toBe("N/A");
  });

  it("formats 0 correctly", () => {
    expect(formatHex(0)).toBe("0x0");
  });

  it("formats positive values in uppercase hex", () => {
    expect(formatHex(255)).toBe("0xFF");
    expect(formatHex(4096)).toBe("0x1000");
    expect(formatHex(0x1a2b)).toBe("0x1A2B");
  });

  it("formats large values", () => {
    expect(formatHex(0xdeadbeef)).toBe("0xDEADBEEF");
  });
});

// =============================================================================
// formatTimestamp
// =============================================================================

describe("formatTimestamp", () => {
  it("returns 'N/A' for null", () => {
    expect(formatTimestamp(null)).toBe("N/A");
  });

  it("formats a Unix timestamp to UTC string", () => {
    // 1704067200 = 2024-01-01T00:00:00Z
    const result = formatTimestamp(1704067200);
    expect(result).toContain("2024");
    expect(result).toContain("UTC");
  });

  it("formats epoch zero", () => {
    const result = formatTimestamp(0);
    expect(result).toContain("1970");
    expect(result).toContain("UTC");
  });

  it("separates date and time with a space (no ISO T)", () => {
    const result = formatTimestamp(1704067200);
    // The date-time boundary uses a space, not ISO "T"
    // ("UTC" in the suffix naturally contains T, so just check the date-time part)
    const dateTimePart = result.replace(/ UTC$/, "");
    expect(dateTimePart).not.toContain("T");
  });
});

// =============================================================================
// formatBadge
// =============================================================================

describe("formatBadge", () => {
  it("returns blue styling for PE format", () => {
    const badge = formatBadge("PE32");
    expect(badge.label).toBe("PE32");
    expect(badge.color).toContain("blue");
  });

  it("returns blue styling for PE32+ format", () => {
    const badge = formatBadge("PE32+");
    expect(badge.label).toBe("PE32+");
    expect(badge.color).toContain("blue");
  });

  it("returns green styling for ELF format", () => {
    const badge = formatBadge("ELF64");
    expect(badge.label).toBe("ELF64");
    expect(badge.color).toContain("green");
  });

  it("returns purple styling for MachO format", () => {
    const badge = formatBadge("MachO (ARM64)");
    expect(badge.label).toBe("MachO (ARM64)");
    expect(badge.color).toContain("purple");
  });

  it("returns default styling for unknown formats", () => {
    const badge = formatBadge("UnknownFormat");
    expect(badge.label).toBe("UnknownFormat");
    expect(badge.color).toContain("bg-bg-secondary");
  });
});
