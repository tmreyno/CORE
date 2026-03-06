// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi } from "vitest";

// Mock formatBytes before importing
vi.mock("../../utils", () => ({
  formatBytes: (n: number) => {
    if (n === 0) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    const i = Math.min(Math.floor(Math.log(n) / Math.log(1024)), units.length - 1);
    return `${(n / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
  },
}));

import { formatDuration, calculateSpeed } from "../fileExport";

// =============================================================================
// formatDuration
// =============================================================================

describe("formatDuration", () => {
  it("formats milliseconds < 1 second", () => {
    expect(formatDuration(500)).toBe("500ms");
    expect(formatDuration(0)).toBe("0ms");
    expect(formatDuration(999)).toBe("999ms");
  });

  it("formats seconds < 1 minute", () => {
    expect(formatDuration(1000)).toBe("1.0s");
    expect(formatDuration(5432)).toBe("5.4s");
    expect(formatDuration(59999)).toBe("60.0s");
  });

  it("formats minutes and seconds", () => {
    expect(formatDuration(60000)).toBe("1m 0s");
    expect(formatDuration(90000)).toBe("1m 30s");
    expect(formatDuration(3661000)).toBe("61m 1s");
  });
});

// =============================================================================
// calculateSpeed
// =============================================================================

describe("calculateSpeed", () => {
  it("returns '0 B/s' for zero duration", () => {
    expect(calculateSpeed(1024, 0)).toBe("0 B/s");
  });

  it("calculates speed in bytes per second", () => {
    // 1024 bytes in 1000ms = 1024 B/s → "1.0 KB/s"
    expect(calculateSpeed(1024, 1000)).toBe("1.0 KB/s");
  });

  it("calculates MB/s for large transfers", () => {
    // 100MB in 1000ms = 100MB/s
    const result = calculateSpeed(100 * 1024 * 1024, 1000);
    expect(result).toBe("100.0 MB/s");
  });

  it("handles fractional speeds", () => {
    // 500 bytes in 1000ms = 500 B/s
    expect(calculateSpeed(500, 1000)).toBe("500 B/s");
  });
});
