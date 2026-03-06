// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  CompressionLevel,
  formatProgress,
  getCompressionRatio,
  ArchiveCreationError,
} from "../archiveCreate";
import type { ArchiveCreateProgress } from "../archiveCreate";

// =============================================================================
// CompressionLevel
// =============================================================================

describe("CompressionLevel", () => {
  it("has Store = 0", () => {
    expect(CompressionLevel.Store).toBe(0);
  });

  it("has Fastest = 1", () => {
    expect(CompressionLevel.Fastest).toBe(1);
  });

  it("has Fast = 3", () => {
    expect(CompressionLevel.Fast).toBe(3);
  });

  it("has Normal = 5", () => {
    expect(CompressionLevel.Normal).toBe(5);
  });

  it("has Maximum = 7", () => {
    expect(CompressionLevel.Maximum).toBe(7);
  });

  it("has Ultra = 9", () => {
    expect(CompressionLevel.Ultra).toBe(9);
  });
});

// =============================================================================
// formatProgress
// =============================================================================

describe("formatProgress", () => {
  it("formats percentage to one decimal", () => {
    const progress = { percent: 42.567 } as ArchiveCreateProgress;
    expect(formatProgress(progress)).toBe("42.6%");
  });

  it("formats zero percent", () => {
    expect(formatProgress({ percent: 0 } as ArchiveCreateProgress)).toBe("0.0%");
  });

  it("formats 100 percent", () => {
    expect(formatProgress({ percent: 100 } as ArchiveCreateProgress)).toBe("100.0%");
  });
});

// =============================================================================
// getCompressionRatio
// =============================================================================

describe("getCompressionRatio", () => {
  it("returns 0 for zero uncompressed size", () => {
    expect(getCompressionRatio(0, 0)).toBe(0);
  });

  it("calculates 50% ratio", () => {
    expect(getCompressionRatio(1000, 500)).toBeCloseTo(50);
  });

  it("calculates 0% ratio (no compression)", () => {
    expect(getCompressionRatio(1000, 1000)).toBeCloseTo(0);
  });

  it("calculates high ratio", () => {
    expect(getCompressionRatio(1000, 100)).toBeCloseTo(90);
  });

  it("handles compressed bigger than original (negative ratio)", () => {
    const ratio = getCompressionRatio(100, 200);
    expect(ratio).toBeLessThan(0);
  });
});

// =============================================================================
// ArchiveCreationError
// =============================================================================

describe("ArchiveCreationError", () => {
  it("extends Error", () => {
    const err = new ArchiveCreationError("test");
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(ArchiveCreationError);
  });

  it("has correct name", () => {
    const err = new ArchiveCreationError("test");
    expect(err.name).toBe("ArchiveCreationError");
  });

  it("stores message", () => {
    const err = new ArchiveCreationError("compression failed");
    expect(err.message).toBe("compression failed");
  });
});
