// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { formatCpuUsage } from "../helpers";

// =============================================================================
// formatCpuUsage
// =============================================================================

describe("formatCpuUsage", () => {
  it("returns '—' for undefined cpuPercent", () => {
    expect(formatCpuUsage(undefined, 8)).toBe("—");
  });

  it("returns '—' for undefined cores", () => {
    expect(formatCpuUsage(50, undefined)).toBe("—");
  });

  it("returns '—' when both undefined", () => {
    expect(formatCpuUsage(undefined, undefined)).toBe("—");
  });

  it("formats as 'cores used / total cores' when >= 100% (1+ core)", () => {
    expect(formatCpuUsage(400, 8)).toBe("4.0/8");
  });

  it("formats as percentage when < 100%", () => {
    expect(formatCpuUsage(50, 8)).toBe("50%");
  });

  it("formats 100% as cores used", () => {
    expect(formatCpuUsage(100, 4)).toBe("1.0/4");
  });

  it("handles single-core high usage", () => {
    expect(formatCpuUsage(800, 8)).toBe("8.0/8");
  });

  it("handles low percentage", () => {
    expect(formatCpuUsage(5, 12)).toBe("5%");
  });

  it("formats zero percent gracefully", () => {
    // 0/100 = 0 which is < 1, so shows as percentage
    expect(formatCpuUsage(0, 8)).toBe("0%");
  });
});
