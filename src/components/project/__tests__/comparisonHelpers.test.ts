// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getSimilarityColor, mergeStrategies } from "../comparisonHelpers";

// =============================================================================
// getSimilarityColor
// =============================================================================
describe("getSimilarityColor", () => {
  it('returns "text-success" for similarity >= 80', () => {
    expect(getSimilarityColor(80)).toBe("text-success");
    expect(getSimilarityColor(100)).toBe("text-success");
    expect(getSimilarityColor(95)).toBe("text-success");
  });

  it('returns "text-warning" for similarity >= 50 and < 80', () => {
    expect(getSimilarityColor(50)).toBe("text-warning");
    expect(getSimilarityColor(79)).toBe("text-warning");
    expect(getSimilarityColor(65)).toBe("text-warning");
  });

  it('returns "text-error" for similarity < 50', () => {
    expect(getSimilarityColor(0)).toBe("text-error");
    expect(getSimilarityColor(49)).toBe("text-error");
    expect(getSimilarityColor(25)).toBe("text-error");
  });

  it("handles boundary values correctly", () => {
    expect(getSimilarityColor(80)).toBe("text-success");
    expect(getSimilarityColor(79.9)).toBe("text-warning");
    expect(getSimilarityColor(50)).toBe("text-warning");
    expect(getSimilarityColor(49.9)).toBe("text-error");
  });
});

// =============================================================================
// mergeStrategies
// =============================================================================
describe("mergeStrategies", () => {
  it("contains 5 strategies", () => {
    expect(mergeStrategies).toHaveLength(5);
  });

  it("has expected strategy values", () => {
    const values = mergeStrategies.map(s => s.value);
    expect(values).toContain("PreferA");
    expect(values).toContain("PreferB");
    expect(values).toContain("KeepBoth");
    expect(values).toContain("Skip");
    expect(values).toContain("Manual");
  });

  it("all strategies have required fields", () => {
    for (const strategy of mergeStrategies) {
      expect(strategy.value).toBeTruthy();
      expect(strategy.label).toBeTruthy();
      expect(strategy.desc).toBeTruthy();
    }
  });

  it("has unique strategy values", () => {
    const values = mergeStrategies.map(s => s.value);
    expect(new Set(values).size).toBe(values.length);
  });
});
