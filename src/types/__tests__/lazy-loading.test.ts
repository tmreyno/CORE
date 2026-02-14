// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  supportsLazyLoading,
  alwaysUseLazyLoading,
  shouldUseLazyLoading,
  calculateBatchSize,
  needsPagination,
  shouldAutoExpand,
  DEFAULT_LAZY_LOAD_CONFIG,
} from "../lazy-loading";
import type { ContainerSummary, LazyLoadConfig } from "../lazy-loading";

// =============================================================================
// Test Helpers
// =============================================================================

function makeSummary(overrides: Partial<ContainerSummary> = {}): ContainerSummary {
  return {
    path: "/evidence/test.ad1",
    container_type: "ad1",
    total_size: 1024 * 1024,
    entry_count: 100,
    root_entry_count: 10,
    lazy_loading_recommended: false,
    estimated_load_time_ms: null,
    ...overrides,
  };
}

// =============================================================================
// supportsLazyLoading
// =============================================================================
describe("supportsLazyLoading", () => {
  it("returns true for supported container types", () => {
    expect(supportsLazyLoading("ad1")).toBe(true);
    expect(supportsLazyLoading("ufed")).toBe(true);
    expect(supportsLazyLoading("ufd")).toBe(true);
    expect(supportsLazyLoading("ufdr")).toBe(true);
    expect(supportsLazyLoading("ufdx")).toBe(true);
    expect(supportsLazyLoading("zip")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(supportsLazyLoading("AD1")).toBe(true);
    expect(supportsLazyLoading("UFED")).toBe(true);
    expect(supportsLazyLoading("ZIP")).toBe(true);
  });

  it("returns false for unsupported types", () => {
    expect(supportsLazyLoading("e01")).toBe(false);
    expect(supportsLazyLoading("raw")).toBe(false);
    expect(supportsLazyLoading("7z")).toBe(false);
    expect(supportsLazyLoading("unknown")).toBe(false);
    expect(supportsLazyLoading("")).toBe(false);
  });
});

// =============================================================================
// alwaysUseLazyLoading
// =============================================================================
describe("alwaysUseLazyLoading", () => {
  it("returns true for always-lazy types", () => {
    expect(alwaysUseLazyLoading("e01")).toBe(true);
    expect(alwaysUseLazyLoading("l01")).toBe(true);
    expect(alwaysUseLazyLoading("ex01")).toBe(true);
    expect(alwaysUseLazyLoading("lx01")).toBe(true);
    expect(alwaysUseLazyLoading("ewf")).toBe(true);
    expect(alwaysUseLazyLoading("7z")).toBe(true);
    expect(alwaysUseLazyLoading("rar")).toBe(true);
    expect(alwaysUseLazyLoading("tar")).toBe(true);
    expect(alwaysUseLazyLoading("raw")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(alwaysUseLazyLoading("E01")).toBe(true);
    expect(alwaysUseLazyLoading("RAW")).toBe(true);
  });

  it("returns false for types that don't always use lazy loading", () => {
    expect(alwaysUseLazyLoading("ad1")).toBe(false);
    expect(alwaysUseLazyLoading("zip")).toBe(false);
    expect(alwaysUseLazyLoading("ufed")).toBe(false);
    expect(alwaysUseLazyLoading("unknown")).toBe(false);
  });
});

// =============================================================================
// shouldUseLazyLoading
// =============================================================================
describe("shouldUseLazyLoading", () => {
  it("returns false when config disabled", () => {
    const config: LazyLoadConfig = { ...DEFAULT_LAZY_LOAD_CONFIG, enabled: false };
    const summary = makeSummary({ entry_count: 999999 });
    expect(shouldUseLazyLoading(summary, config)).toBe(false);
  });

  it("returns true when lazy_loading_recommended is true", () => {
    const summary = makeSummary({ lazy_loading_recommended: true, entry_count: 1 });
    expect(shouldUseLazyLoading(summary)).toBe(true);
  });

  it("returns true when entry_count exceeds threshold", () => {
    const summary = makeSummary({ entry_count: 10_001 });
    expect(shouldUseLazyLoading(summary)).toBe(true);
  });

  it("returns false when entry_count is below threshold", () => {
    const summary = makeSummary({ entry_count: 100 });
    expect(shouldUseLazyLoading(summary)).toBe(false);
  });

  it("returns false when entry_count equals threshold", () => {
    const summary = makeSummary({ entry_count: 10_000 });
    expect(shouldUseLazyLoading(summary)).toBe(false);
  });

  it("uses custom config threshold", () => {
    const config: LazyLoadConfig = { ...DEFAULT_LAZY_LOAD_CONFIG, large_container_threshold: 50 };
    const summary = makeSummary({ entry_count: 51 });
    expect(shouldUseLazyLoading(summary, config)).toBe(true);
  });
});

// =============================================================================
// calculateBatchSize
// =============================================================================
describe("calculateBatchSize", () => {
  it("returns batch_size when totalEntries is larger", () => {
    expect(calculateBatchSize(1000)).toBe(100); // default batch_size
  });

  it("returns totalEntries when smaller than batch_size", () => {
    expect(calculateBatchSize(50)).toBe(50);
  });

  it("returns totalEntries when equal to batch_size", () => {
    expect(calculateBatchSize(100)).toBe(100);
  });

  it("uses custom config", () => {
    const config: LazyLoadConfig = { ...DEFAULT_LAZY_LOAD_CONFIG, batch_size: 200 };
    expect(calculateBatchSize(150, config)).toBe(150);
    expect(calculateBatchSize(300, config)).toBe(200);
  });

  it("handles zero entries", () => {
    expect(calculateBatchSize(0)).toBe(0);
  });
});

// =============================================================================
// needsPagination
// =============================================================================
describe("needsPagination", () => {
  it("returns true when childCount exceeds threshold", () => {
    expect(needsPagination(501)).toBe(true); // default threshold 500
  });

  it("returns false when childCount is at threshold", () => {
    expect(needsPagination(500)).toBe(false);
  });

  it("returns false when childCount is below threshold", () => {
    expect(needsPagination(100)).toBe(false);
  });

  it("uses custom config", () => {
    const config: LazyLoadConfig = { ...DEFAULT_LAZY_LOAD_CONFIG, pagination_threshold: 10 };
    expect(needsPagination(11, config)).toBe(true);
    expect(needsPagination(10, config)).toBe(false);
  });
});

// =============================================================================
// shouldAutoExpand
// =============================================================================
describe("shouldAutoExpand", () => {
  it("returns true when childCount is at or below threshold", () => {
    expect(shouldAutoExpand(50)).toBe(true);  // default threshold 50
    expect(shouldAutoExpand(1)).toBe(true);
    expect(shouldAutoExpand(0)).toBe(true);
  });

  it("returns false when childCount exceeds threshold", () => {
    expect(shouldAutoExpand(51)).toBe(false);
  });

  it("returns true for any count when lazy loading is disabled", () => {
    const config: LazyLoadConfig = { ...DEFAULT_LAZY_LOAD_CONFIG, enabled: false };
    expect(shouldAutoExpand(99999, config)).toBe(true);
  });

  it("uses custom config", () => {
    const config: LazyLoadConfig = { ...DEFAULT_LAZY_LOAD_CONFIG, auto_expand_threshold: 10 };
    expect(shouldAutoExpand(10, config)).toBe(true);
    expect(shouldAutoExpand(11, config)).toBe(false);
  });
});

// =============================================================================
// DEFAULT_LAZY_LOAD_CONFIG
// =============================================================================
describe("DEFAULT_LAZY_LOAD_CONFIG", () => {
  it("has expected defaults", () => {
    expect(DEFAULT_LAZY_LOAD_CONFIG.enabled).toBe(true);
    expect(DEFAULT_LAZY_LOAD_CONFIG.batch_size).toBe(100);
    expect(DEFAULT_LAZY_LOAD_CONFIG.auto_expand_threshold).toBe(50);
    expect(DEFAULT_LAZY_LOAD_CONFIG.large_container_threshold).toBe(10_000);
    expect(DEFAULT_LAZY_LOAD_CONFIG.pagination_threshold).toBe(500);
    expect(DEFAULT_LAZY_LOAD_CONFIG.show_entry_count).toBe(true);
    expect(DEFAULT_LAZY_LOAD_CONFIG.count_timeout_ms).toBe(5_000);
    expect(DEFAULT_LAZY_LOAD_CONFIG.load_timeout_ms).toBe(30_000);
  });
});
