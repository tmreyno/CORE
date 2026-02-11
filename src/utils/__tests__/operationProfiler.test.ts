// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// We need to access the class internals via the singleton
// The module exports a singleton `operationProfiler` and types
import {
  operationProfiler,
  OperationCategories,
  type OperationStats,
  type ProfilerReport,
} from "../operationProfiler";

describe("operationProfiler", () => {
  beforeEach(() => {
    // Suppress logger output
    vi.spyOn(console, "log").mockImplementation(() => {});
    vi.spyOn(console, "warn").mockImplementation(() => {});
    operationProfiler.disable();
    operationProfiler.clear();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    operationProfiler.disable();
  });

  // ===========================================================================
  // Enable / Disable / isEnabled
  // ===========================================================================

  describe("enable / disable / isEnabled", () => {
    it("starts disabled", () => {
      expect(operationProfiler.isEnabled()).toBe(false);
    });

    it("becomes enabled after enable()", () => {
      operationProfiler.enable();
      expect(operationProfiler.isEnabled()).toBe(true);
    });

    it("becomes disabled after disable()", () => {
      operationProfiler.enable();
      operationProfiler.disable();
      expect(operationProfiler.isEnabled()).toBe(false);
    });
  });

  // ===========================================================================
  // measureSync
  // ===========================================================================

  describe("measureSync", () => {
    it("returns the function result when disabled", () => {
      const result = operationProfiler.measureSync("test-op", () => 42);
      expect(result).toBe(42);
    });

    it("returns the function result when enabled", () => {
      operationProfiler.enable();
      const result = operationProfiler.measureSync("test-op", () => "hello");
      expect(result).toBe("hello");
    });

    it("records timing when enabled", () => {
      operationProfiler.enable();
      operationProfiler.measureSync("my-op", () => {
        // simulate some work
        let sum = 0;
        for (let i = 0; i < 100; i++) sum += i;
        return sum;
      });
      const stats = operationProfiler.getStats("my-op");
      expect(stats).not.toBeNull();
      expect(stats!.count).toBe(1);
      expect(stats!.avgMs).toBeGreaterThanOrEqual(0);
    });

    it("does NOT record timing when disabled", () => {
      operationProfiler.measureSync("skip-op", () => 1);
      expect(operationProfiler.getStats("skip-op")).toBeNull();
    });

    it("records timing even when function throws", () => {
      operationProfiler.enable();
      expect(() =>
        operationProfiler.measureSync("throw-op", () => {
          throw new Error("boom");
        })
      ).toThrow("boom");
      const stats = operationProfiler.getStats("throw-op");
      expect(stats).not.toBeNull();
      expect(stats!.count).toBe(1);
    });

    it("includes error metadata when function throws", () => {
      operationProfiler.enable();
      try {
        operationProfiler.measureSync("err-meta", () => {
          throw new Error("fail");
        });
      } catch {
        // expected
      }
      const report = operationProfiler.getReport();
      const timing = report.recentTimings.find(t => t.name === "err-meta");
      expect(timing?.metadata?.error).toBe(true);
    });
  });

  // ===========================================================================
  // measure (async)
  // ===========================================================================

  describe("measure (async)", () => {
    it("returns the async function result when disabled", async () => {
      const result = await operationProfiler.measure("async-op", async () => 99);
      expect(result).toBe(99);
    });

    it("returns the async function result when enabled", async () => {
      operationProfiler.enable();
      const result = await operationProfiler.measure("async-op", async () => "done");
      expect(result).toBe("done");
    });

    it("records timing for async operations", async () => {
      operationProfiler.enable();
      await operationProfiler.measure("async-timed", async () => {
        return new Promise(resolve => setTimeout(resolve, 5));
      });
      const stats = operationProfiler.getStats("async-timed");
      expect(stats).not.toBeNull();
      expect(stats!.count).toBe(1);
      expect(stats!.minMs).toBeGreaterThanOrEqual(0);
    });

    it("records timing when async function rejects", async () => {
      operationProfiler.enable();
      await expect(
        operationProfiler.measure("async-err", async () => {
          throw new Error("async boom");
        })
      ).rejects.toThrow("async boom");
      const stats = operationProfiler.getStats("async-err");
      expect(stats).not.toBeNull();
      expect(stats!.count).toBe(1);
    });

    it("includes metadata on success", async () => {
      operationProfiler.enable();
      await operationProfiler.measure(
        "meta-op",
        async () => "ok",
        { fileCount: 10 }
      );
      const report = operationProfiler.getReport();
      const timing = report.recentTimings.find(t => t.name === "meta-op");
      expect(timing?.metadata).toEqual({ fileCount: 10 });
    });
  });

  // ===========================================================================
  // startTiming (manual start/stop)
  // ===========================================================================

  describe("startTiming", () => {
    it("returns a noop when disabled", () => {
      const end = operationProfiler.startTiming("noop-op");
      end();
      expect(operationProfiler.getStats("noop-op")).toBeNull();
    });

    it("records timing when enabled", () => {
      operationProfiler.enable();
      const end = operationProfiler.startTiming("manual-op");
      // simulate work
      let sum = 0;
      for (let i = 0; i < 100; i++) sum += i;
      end();
      const stats = operationProfiler.getStats("manual-op");
      expect(stats).not.toBeNull();
      expect(stats!.count).toBe(1);
    });
  });

  // ===========================================================================
  // getStats - statistical calculations
  // ===========================================================================

  describe("getStats", () => {
    it("returns null for unknown operation", () => {
      expect(operationProfiler.getStats("nonexistent")).toBeNull();
    });

    it("computes correct statistics for multiple measurements", () => {
      operationProfiler.enable();
      // Record several sync operations
      for (let i = 0; i < 10; i++) {
        operationProfiler.measureSync("stats-op", () => {
          let sum = 0;
          for (let j = 0; j < (i + 1) * 50; j++) sum += j;
          return sum;
        });
      }
      const stats = operationProfiler.getStats("stats-op") as OperationStats;
      expect(stats.count).toBe(10);
      expect(stats.totalMs).toBeGreaterThanOrEqual(0);
      expect(stats.avgMs).toBeCloseTo(stats.totalMs / 10, 1);
      expect(stats.minMs).toBeLessThanOrEqual(stats.maxMs);
      expect(stats.p50Ms).toBeGreaterThanOrEqual(stats.minMs);
      expect(stats.p95Ms).toBeGreaterThanOrEqual(stats.p50Ms);
      expect(stats.p99Ms).toBeGreaterThanOrEqual(stats.p95Ms);
      expect(stats.durations).toHaveLength(10);
    });
  });

  // ===========================================================================
  // getOperationNames
  // ===========================================================================

  describe("getOperationNames", () => {
    it("returns empty array when no operations recorded", () => {
      expect(operationProfiler.getOperationNames()).toEqual([]);
    });

    it("returns unique operation names", () => {
      operationProfiler.enable();
      operationProfiler.measureSync("op-a", () => 1);
      operationProfiler.measureSync("op-b", () => 2);
      operationProfiler.measureSync("op-a", () => 3);
      const names = operationProfiler.getOperationNames();
      expect(names).toContain("op-a");
      expect(names).toContain("op-b");
      expect(names.length).toBe(2);
    });
  });

  // ===========================================================================
  // getReport
  // ===========================================================================

  describe("getReport", () => {
    it("returns report with enabled status", () => {
      const report = operationProfiler.getReport();
      expect(report.enabled).toBe(false);
    });

    it("returns report with operations after measurements", () => {
      operationProfiler.enable();
      operationProfiler.measureSync("report-op", () => 1);
      const report = operationProfiler.getReport();
      expect(report.enabled).toBe(true);
      expect(report.operations["report-op"]).toBeDefined();
      expect(report.operations["report-op"].count).toBe(1);
      expect(report.recentTimings.length).toBe(1);
    });

    it("limits recentTimings to last 50", () => {
      operationProfiler.enable();
      for (let i = 0; i < 60; i++) {
        operationProfiler.measureSync(`bulk-${i % 3}`, () => i);
      }
      const report = operationProfiler.getReport();
      expect(report.recentTimings.length).toBeLessThanOrEqual(50);
    });
  });

  // ===========================================================================
  // clear
  // ===========================================================================

  describe("clear", () => {
    it("removes all recorded timings", () => {
      operationProfiler.enable();
      operationProfiler.measureSync("clear-op", () => 1);
      expect(operationProfiler.getStats("clear-op")).not.toBeNull();
      operationProfiler.clear();
      expect(operationProfiler.getStats("clear-op")).toBeNull();
      expect(operationProfiler.getOperationNames()).toEqual([]);
    });
  });

  // ===========================================================================
  // printReport
  // ===========================================================================

  describe("printReport", () => {
    it("does not throw when no data", () => {
      expect(() => operationProfiler.printReport()).not.toThrow();
    });

    it("does not throw with recorded data", () => {
      operationProfiler.enable();
      operationProfiler.measureSync("print-op", () => 1);
      expect(() => operationProfiler.printReport()).not.toThrow();
    });
  });

  // ===========================================================================
  // Export
  // ===========================================================================

  describe("exportJSON", () => {
    it("returns valid JSON", () => {
      operationProfiler.enable();
      operationProfiler.measureSync("json-op", () => 1);
      const json = operationProfiler.exportJSON();
      const parsed = JSON.parse(json) as ProfilerReport;
      expect(parsed.enabled).toBe(true);
      expect(parsed.operations["json-op"]).toBeDefined();
    });
  });

  describe("exportCSV", () => {
    it("returns CSV with header and data rows", () => {
      operationProfiler.enable();
      operationProfiler.measureSync("csv-op", () => 1);
      const csv = operationProfiler.exportCSV();
      const lines = csv.split("\n");
      expect(lines[0]).toBe("name,startTime,endTime,duration,metadata");
      expect(lines.length).toBeGreaterThanOrEqual(2);
      expect(lines[1]).toContain("csv-op");
    });

    it("returns only header when no data", () => {
      const csv = operationProfiler.exportCSV();
      const lines = csv.split("\n");
      expect(lines.length).toBe(1);
    });
  });

  // ===========================================================================
  // OperationCategories constants
  // ===========================================================================

  describe("OperationCategories", () => {
    it("has expected category values", () => {
      expect(OperationCategories.TREE_EXPAND).toBe("tree-expand");
      expect(OperationCategories.VIEWER_HEX_CHUNK).toBe("viewer-hex-chunk");
      expect(OperationCategories.HASH_COMPLETE).toBe("hash-complete");
      expect(OperationCategories.CONTAINER_INFO).toBe("container-info");
      expect(OperationCategories.FILE_DISCOVER).toBe("file-discover");
    });

    it("all values are strings", () => {
      for (const key of Object.keys(OperationCategories)) {
        expect(typeof OperationCategories[key as keyof typeof OperationCategories]).toBe("string");
      }
    });
  });
});
