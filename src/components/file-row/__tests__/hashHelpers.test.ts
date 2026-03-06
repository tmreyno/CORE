// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi } from "vitest";

// Mock formatBytes before importing hashHelpers
vi.mock("../../../utils", () => ({
  formatBytes: (n: number) => `${n} bytes`,
}));

import {
  isContainerIncomplete,
  getTotalContainerSize,
  buildSizeLabel,
  getStoredHashCount,
  getTotalHashCount,
  hasVerifiedMatch,
  getHashState,
  isCurrentlyHashing,
  isCurrentlyCompleting,
  formatChunks,
} from "../hashHelpers";
import type { ContainerInfo, HashHistoryEntry } from "../../../types";

// =============================================================================
// isContainerIncomplete
// =============================================================================

describe("isContainerIncomplete", () => {
  it("returns false for undefined info", () => {
    expect(isContainerIncomplete(undefined)).toBe(false);
  });

  it("returns false when no missing segments", () => {
    expect(isContainerIncomplete({ ad1: { missing_segments: [] } } as any)).toBe(false);
  });

  it("returns true when missing segments exist", () => {
    expect(
      isContainerIncomplete({ ad1: { missing_segments: ["seg2"] } } as any)
    ).toBe(true);
  });
});

// =============================================================================
// getTotalContainerSize
// =============================================================================

describe("getTotalContainerSize", () => {
  it("returns null for undefined info", () => {
    expect(getTotalContainerSize(undefined)).toBeNull();
  });

  it("returns ad1 total_size when present", () => {
    expect(getTotalContainerSize({ ad1: { total_size: 100 } } as any)).toBe(100);
  });

  it("returns e01 total_size when present", () => {
    expect(getTotalContainerSize({ e01: { total_size: 200 } } as any)).toBe(200);
  });

  it("returns l01 total_size when present", () => {
    expect(getTotalContainerSize({ l01: { total_size: 300 } } as any)).toBe(300);
  });

  it("returns raw total_size when present", () => {
    expect(getTotalContainerSize({ raw: { total_size: 400 } } as any)).toBe(400);
  });

  it("returns archive total_size when present", () => {
    expect(getTotalContainerSize({ archive: { total_size: 500 } } as any)).toBe(500);
  });

  it("returns null when no total_size found", () => {
    expect(getTotalContainerSize({} as any)).toBeNull();
  });
});

// =============================================================================
// buildSizeLabel
// =============================================================================

describe("buildSizeLabel", () => {
  it("shows simple size when single segment", () => {
    expect(buildSizeLabel(1024, null, 1)).toBe("1024 bytes");
  });

  it("shows total size with segment info for multi-segment", () => {
    const result = buildSizeLabel(500, 2000, 4);
    expect(result).toContain("Total: 2000 bytes");
    expect(result).toContain("4 segments");
    expect(result).toContain("first segment: 500 bytes");
  });

  it("uses totalSize when available with single segment", () => {
    expect(buildSizeLabel(100, 100, 1)).toBe("100 bytes");
  });

  it("falls back to fileSize when totalSize is null", () => {
    expect(buildSizeLabel(512, null, undefined)).toBe("512 bytes");
  });
});

// =============================================================================
// getStoredHashCount
// =============================================================================

describe("getStoredHashCount", () => {
  it("returns 0 for undefined info", () => {
    expect(getStoredHashCount(undefined)).toBe(0);
  });

  it("counts e01 stored hashes", () => {
    expect(getStoredHashCount({ e01: { stored_hashes: [{}, {}] } } as any)).toBe(2);
  });

  it("counts companion_log stored hashes", () => {
    expect(getStoredHashCount({ companion_log: { stored_hashes: [{}] } } as any)).toBe(1);
  });

  it("sums e01 and companion_log hashes", () => {
    const info = {
      e01: { stored_hashes: [{}, {}] },
      companion_log: { stored_hashes: [{}] },
    } as any;
    expect(getStoredHashCount(info)).toBe(3);
  });
});

// =============================================================================
// getTotalHashCount
// =============================================================================

describe("getTotalHashCount", () => {
  it("returns 0 when no hashes anywhere", () => {
    expect(getTotalHashCount(undefined, undefined, [])).toBe(0);
  });

  it("counts stored + fileHash + history", () => {
    const info = { e01: { stored_hashes: [{}] } } as any;
    const fileHash = { hash: "abc", algorithm: "MD5" } as any;
    const history = [
      { hash: "def", algorithm: "SHA-1" },
      { hash: "ghi", algorithm: "SHA-256" },
    ] as HashHistoryEntry[];
    expect(getTotalHashCount(info, fileHash, history)).toBe(4); // 1+1+2
  });
});

// =============================================================================
// hasVerifiedMatch
// =============================================================================

describe("hasVerifiedMatch", () => {
  it("returns false with no stored hashes and no history", () => {
    expect(hasVerifiedMatch(undefined, [])).toBe(false);
  });

  it("returns true when stored hash matches history entry", () => {
    const info = {
      e01: {
        stored_hashes: [{ algorithm: "MD5", hash: "ABC123" }],
      },
    } as any;
    const history = [
      { algorithm: "md5", hash: "abc123" },
    ] as HashHistoryEntry[];
    expect(hasVerifiedMatch(info, history)).toBe(true);
  });

  it("returns false when algorithms differ", () => {
    const info = {
      e01: {
        stored_hashes: [{ algorithm: "MD5", hash: "ABC123" }],
      },
    } as any;
    const history = [
      { algorithm: "SHA-1", hash: "abc123" },
    ] as HashHistoryEntry[];
    expect(hasVerifiedMatch(info, history)).toBe(false);
  });

  it("returns true when two history entries match each other", () => {
    const history = [
      { algorithm: "MD5", hash: "abc123" },
      { algorithm: "MD5", hash: "ABC123" },
    ] as HashHistoryEntry[];
    expect(hasVerifiedMatch(undefined, history)).toBe(true);
  });

  it("returns false when history entries have different algorithms", () => {
    const history = [
      { algorithm: "MD5", hash: "abc123" },
      { algorithm: "SHA-1", hash: "ABC123" },
    ] as HashHistoryEntry[];
    expect(hasVerifiedMatch(undefined, history)).toBe(false);
  });

  it("checks ufed stored hashes", () => {
    const info = {
      ufed: {
        stored_hashes: [{ algorithm: "SHA-256", hash: "DEADBEEF" }],
      },
    } as any;
    const history = [
      { algorithm: "sha-256", hash: "deadbeef" },
    ] as HashHistoryEntry[];
    expect(hasVerifiedMatch(info, history)).toBe(true);
  });
});

// =============================================================================
// getHashState
// =============================================================================

describe("getHashState", () => {
  it("returns 'incomplete' when container has missing segments", () => {
    const info = { ad1: { missing_segments: ["s2"] } } as any;
    expect(getHashState(info, undefined, [])).toBe("incomplete");
  });

  it("returns 'verified' when fileHash.verified is true", () => {
    const fileHash = { verified: true, hash: "", algorithm: "" } as any;
    expect(getHashState(undefined, fileHash, [])).toBe("verified");
  });

  it("returns 'failed' when fileHash.verified is false", () => {
    const fileHash = { verified: false, hash: "", algorithm: "" } as any;
    expect(getHashState(undefined, fileHash, [])).toBe("failed");
  });

  it("returns 'computed' when fileHash exists without verified flag", () => {
    const fileHash = { hash: "abc", algorithm: "MD5" } as any;
    expect(getHashState(undefined, fileHash, [])).toBe("computed");
  });

  it("returns 'verified' when stored match found in history", () => {
    const info = {
      e01: { stored_hashes: [{ algorithm: "MD5", hash: "abc" }] },
    } as any;
    const history = [{ algorithm: "md5", hash: "abc" }] as HashHistoryEntry[];
    expect(getHashState(info, undefined, history)).toBe("verified");
  });

  it("returns 'stored' when stored hashes exist without match", () => {
    const info = {
      e01: { stored_hashes: [{ algorithm: "MD5", hash: "abc" }] },
    } as any;
    expect(getHashState(info, undefined, [])).toBe("stored");
  });

  it("returns 'computed' when only history exists", () => {
    const history = [{ algorithm: "MD5", hash: "abc" }] as HashHistoryEntry[];
    expect(getHashState(undefined, undefined, history)).toBe("computed");
  });

  it("returns 'none' when nothing exists", () => {
    expect(getHashState(undefined, undefined, [])).toBe("none");
  });
});

// =============================================================================
// isCurrentlyHashing / isCurrentlyCompleting
// =============================================================================

describe("isCurrentlyHashing", () => {
  it("returns true when status is hashing, no result, < 95%", () => {
    const status = { status: "hashing", progress: 50 } as any;
    expect(isCurrentlyHashing(status, undefined)).toBe(true);
  });

  it("returns false when result exists", () => {
    const status = { status: "hashing", progress: 50 } as any;
    const hash = { hash: "abc" } as any;
    expect(isCurrentlyHashing(status, hash)).toBe(false);
  });

  it("returns false when progress >= 95", () => {
    const status = { status: "hashing", progress: 96 } as any;
    expect(isCurrentlyHashing(status, undefined)).toBe(false);
  });

  it("returns false for undefined status", () => {
    expect(isCurrentlyHashing(undefined, undefined)).toBe(false);
  });
});

describe("isCurrentlyCompleting", () => {
  it("returns true when hashing at >= 95% with no result", () => {
    const status = { status: "hashing", progress: 97 } as any;
    expect(isCurrentlyCompleting(status, undefined)).toBe(true);
  });

  it("returns false when progress < 95", () => {
    const status = { status: "hashing", progress: 90 } as any;
    expect(isCurrentlyCompleting(status, undefined)).toBe(false);
  });

  it("returns false when result exists", () => {
    const status = { status: "hashing", progress: 99 } as any;
    const hash = { hash: "abc" } as any;
    expect(isCurrentlyCompleting(status, hash)).toBe(false);
  });
});

// =============================================================================
// formatChunks
// =============================================================================

describe("formatChunks", () => {
  it("formats small numbers as-is", () => {
    expect(formatChunks(42)).toBe("42");
    expect(formatChunks(999)).toBe("999");
  });

  it("formats thousands with k suffix", () => {
    expect(formatChunks(1000)).toBe("1k");
    expect(formatChunks(5500)).toBe("6k");
    expect(formatChunks(999999)).toBe("1000k");
  });

  it("formats millions with M suffix", () => {
    expect(formatChunks(1000000)).toBe("1.0M");
    expect(formatChunks(2500000)).toBe("2.5M");
  });
});
