// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  setPerformanceMonitoringEnabled,
  isPerformanceMonitoringEnabled,
  trackEffect,
  trackMemo,
  trackAction,
  getPerformanceEntries,
  getPerformanceEntriesByType,
  getAllRenderMetrics,
  getRenderMetrics,
  clearPerformanceData,
  getPerformanceSummary,
  formatDuration,
  getPerformanceGrade,
  type PerformanceSummary,
} from "../performance";

// =============================================================================
// formatDuration
// =============================================================================

describe("formatDuration", () => {
  it("formats sub-millisecond durations as microseconds", () => {
    const result = formatDuration(0.5);
    expect(result).toContain("μs");
    expect(result).toContain("500");
  });

  it("formats millisecond durations", () => {
    expect(formatDuration(1)).toContain("ms");
    expect(formatDuration(15.5)).toContain("ms");
    expect(formatDuration(999)).toContain("ms");
  });

  it("formats second durations", () => {
    expect(formatDuration(1000)).toContain("s");
    expect(formatDuration(2500)).toContain("s");
  });

  it("handles zero", () => {
    const result = formatDuration(0);
    expect(result).toContain("μs");
  });
});

// =============================================================================
// getPerformanceGrade
// =============================================================================

describe("getPerformanceGrade", () => {
  it("returns 'excellent' for durations under 5ms", () => {
    expect(getPerformanceGrade(0)).toBe("excellent");
    expect(getPerformanceGrade(1)).toBe("excellent");
    expect(getPerformanceGrade(4.99)).toBe("excellent");
  });

  it("returns 'good' for durations 5-16ms", () => {
    expect(getPerformanceGrade(5)).toBe("good");
    expect(getPerformanceGrade(10)).toBe("good");
    expect(getPerformanceGrade(15.99)).toBe("good");
  });

  it("returns 'fair' for durations 16-33ms", () => {
    expect(getPerformanceGrade(16)).toBe("fair");
    expect(getPerformanceGrade(25)).toBe("fair");
    expect(getPerformanceGrade(32.99)).toBe("fair");
  });

  it("returns 'poor' for durations 33ms and above", () => {
    expect(getPerformanceGrade(33)).toBe("poor");
    expect(getPerformanceGrade(100)).toBe("poor");
    expect(getPerformanceGrade(1000)).toBe("poor");
  });
});

// =============================================================================
// Monitoring enable/disable
// =============================================================================

describe("performance monitoring toggle", () => {
  beforeEach(() => {
    clearPerformanceData();
    setPerformanceMonitoringEnabled(false);
  });

  it("defaults to disabled", () => {
    expect(isPerformanceMonitoringEnabled()).toBe(false);
  });

  it("can be enabled and disabled", () => {
    setPerformanceMonitoringEnabled(true);
    expect(isPerformanceMonitoringEnabled()).toBe(true);

    setPerformanceMonitoringEnabled(false);
    expect(isPerformanceMonitoringEnabled()).toBe(false);
  });
});

// =============================================================================
// trackEffect
// =============================================================================

describe("trackEffect", () => {
  beforeEach(() => {
    clearPerformanceData();
    setPerformanceMonitoringEnabled(true);
  });

  it("records an effect entry when monitoring is enabled", () => {
    trackEffect("testEffect", () => 42);
    const entries = getPerformanceEntries();
    expect(entries.length).toBeGreaterThanOrEqual(1);
    const entry = entries.find(e => e.name === "testEffect" && e.type === "effect");
    expect(entry).toBeDefined();
    expect(entry!.duration).toBeGreaterThanOrEqual(0);
  });

  it("returns the result of the wrapped function", () => {
    const result = trackEffect("testEffect", () => "hello");
    expect(result).toBe("hello");
  });

  it("does not record when monitoring is disabled", () => {
    setPerformanceMonitoringEnabled(false);
    const result = trackEffect("testEffect", () => 42);
    expect(result).toBe(42);
    const entries = getPerformanceEntries();
    const entry = entries.find(e => e.name === "testEffect" && e.type === "effect");
    expect(entry).toBeUndefined();
  });
});

// =============================================================================
// trackMemo
// =============================================================================

describe("trackMemo", () => {
  beforeEach(() => {
    clearPerformanceData();
    setPerformanceMonitoringEnabled(true);
  });

  it("records a memo entry when monitoring is enabled", () => {
    const result = trackMemo("testMemo", () => "computed");
    expect(result).toBe("computed");
    const entries = getPerformanceEntries();
    const entry = entries.find(e => e.name === "testMemo" && e.type === "memo");
    expect(entry).toBeDefined();
    expect(entry!.duration).toBeGreaterThanOrEqual(0);
  });
});

// =============================================================================
// trackAction
// =============================================================================

describe("trackAction", () => {
  beforeEach(() => {
    clearPerformanceData();
    setPerformanceMonitoringEnabled(true);
  });

  it("records an action entry and returns the result", async () => {
    const result = await trackAction("testAction", async () => {
      return 42;
    });
    expect(result).toBe(42);
    const entries = getPerformanceEntries();
    const entry = entries.find(e => e.name === "testAction" && e.type === "action");
    expect(entry).toBeDefined();
  });

  it("propagates errors from the action", async () => {
    await expect(
      trackAction("failAction", async () => {
        throw new Error("test error");
      })
    ).rejects.toThrow("test error");
  });
});

// =============================================================================
// getPerformanceEntriesByType
// =============================================================================

describe("getPerformanceEntriesByType", () => {
  beforeEach(() => {
    clearPerformanceData();
    setPerformanceMonitoringEnabled(true);
  });

  it("filters entries by type", () => {
    trackEffect("eff1", () => 1);
    trackEffect("eff2", () => 2);
    trackMemo("memo1", () => 3);

    const effects = getPerformanceEntriesByType("effect");
    expect(effects.length).toBe(2);
    expect(effects.every(e => e.type === "effect")).toBe(true);

    const memos = getPerformanceEntriesByType("memo");
    expect(memos.length).toBe(1);
  });
});

// =============================================================================
// clearPerformanceData
// =============================================================================

describe("clearPerformanceData", () => {
  it("clears all entries and render metrics", () => {
    setPerformanceMonitoringEnabled(true);
    trackEffect("eff", () => 1);
    trackMemo("memo", () => 2);

    expect(getPerformanceEntries().length).toBeGreaterThan(0);

    clearPerformanceData();
    expect(getPerformanceEntries().length).toBe(0);
    expect(getAllRenderMetrics().length).toBe(0);
  });
});

// =============================================================================
// getRenderMetrics / getAllRenderMetrics
// =============================================================================

describe("render metrics", () => {
  beforeEach(() => {
    clearPerformanceData();
  });

  it("getAllRenderMetrics returns empty array when cleared", () => {
    expect(getAllRenderMetrics().length).toBe(0);
  });

  it("getRenderMetrics returns undefined for unknown component", () => {
    expect(getRenderMetrics("NonExistent")).toBeUndefined();
  });
});

// =============================================================================
// getPerformanceSummary
// =============================================================================

describe("getPerformanceSummary", () => {
  beforeEach(() => {
    clearPerformanceData();
    setPerformanceMonitoringEnabled(true);
  });

  it("returns a summary object with counts", () => {
    trackEffect("eff1", () => 1);
    trackEffect("eff2", () => 10);
    trackMemo("memo1", () => 5);

    const summary = getPerformanceSummary();
    expect(summary).toBeDefined();
    expect(summary.totalEffects).toBe(2);
    expect(summary.totalMemos).toBe(1);
  });

  it("returns zeroes when no data", () => {
    const summary = getPerformanceSummary();
    expect(summary.totalRenders).toBe(0);
    expect(summary.totalEffects).toBe(0);
    expect(summary.totalMemos).toBe(0);
    expect(summary.totalActions).toBe(0);
  });
});
