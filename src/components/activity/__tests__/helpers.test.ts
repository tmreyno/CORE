// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { formatDuration, formatTimestamp } from "../helpers";

// =============================================================================
// formatDuration
// =============================================================================

describe("formatDuration", () => {
  it("returns '—' for null", () => {
    expect(formatDuration(null)).toBe("—");
  });

  it("returns '—' for undefined", () => {
    expect(formatDuration(undefined)).toBe("—");
  });

  it("returns '—' for 0 (falsy)", () => {
    expect(formatDuration(0)).toBe("—");
  });

  it("formats seconds under a minute", () => {
    expect(formatDuration(30)).toBe("30s");
    expect(formatDuration(59)).toBe("59s");
  });

  it("formats minutes and seconds", () => {
    expect(formatDuration(65)).toBe("1m 5s");
    expect(formatDuration(3599)).toBe("59m 59s");
  });

  it("formats hours and minutes", () => {
    expect(formatDuration(3600)).toBe("1h 0m");
    expect(formatDuration(7265)).toBe("2h 1m");
    expect(formatDuration(86400)).toBe("24h 0m");
  });
});

// =============================================================================
// formatTimestamp
// =============================================================================

describe("formatTimestamp", () => {
  it("returns 'just now' for very recent timestamps", () => {
    const now = new Date().toISOString();
    expect(formatTimestamp(now, true)).toBe("just now");
  });

  it("formats minutes ago", () => {
    const fiveMinAgo = new Date(Date.now() - 5 * 60000).toISOString();
    expect(formatTimestamp(fiveMinAgo, true)).toBe("5m ago");
  });

  it("formats hours ago", () => {
    const twoHoursAgo = new Date(Date.now() - 2 * 3600000).toISOString();
    expect(formatTimestamp(twoHoursAgo, true)).toBe("2h ago");
  });

  it("formats days ago", () => {
    const threeDaysAgo = new Date(Date.now() - 3 * 86400000).toISOString();
    expect(formatTimestamp(threeDaysAgo, true)).toBe("3d ago");
  });

  it("returns absolute date for old timestamps", () => {
    const oldDate = "2020-06-15T12:00:00Z";
    const result = formatTimestamp(oldDate, true);
    // Old enough (>7 days) that it falls through to absolute format
    // toLocaleDateString + toLocaleTimeString produce locale-dependent output
    expect(result).toMatch(/2020/);
  });

  it("returns absolute format when relative is false", () => {
    const date = "2025-06-15T10:30:00Z";
    const result = formatTimestamp(date, false);
    expect(result).toContain("2025");
  });
});
