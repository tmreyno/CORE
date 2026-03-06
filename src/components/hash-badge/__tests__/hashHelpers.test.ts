// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi } from "vitest";

// Mock compareHashes before importing
vi.mock("../../../hooks/hashUtils", () => ({
  compareHashes: (
    hash1: string,
    hash2: string,
    alg1: string,
    alg2: string
  ): boolean => alg1.toLowerCase() === alg2.toLowerCase() && hash1.toLowerCase() === hash2.toLowerCase(),
}));

import {
  getHashState,
  hasVerifiedMatch,
  getStoredHashCount,
  getTotalHashCount,
  isHashing,
  isCompleting,
  formatChunks,
} from "../hashHelpers";
import type { HashHistoryEntry } from "../../../types";

// =============================================================================
// getHashState
// =============================================================================

describe("getHashState", () => {
  it("returns 'incomplete' when container has missing segments", () => {
    expect(
      getHashState({
        containerInfo: { ad1: { missing_segments: ["s2"] } } as any,
      })
    ).toBe("incomplete");
  });

  it("returns 'verified' when fileHash.verified is true", () => {
    expect(
      getHashState({ fileHash: { verified: true, hash: "", algorithm: "" } as any })
    ).toBe("verified");
  });

  it("returns 'failed' when fileHash.verified is false", () => {
    expect(
      getHashState({ fileHash: { verified: false, hash: "", algorithm: "" } as any })
    ).toBe("failed");
  });

  it("returns 'computed' when fileHash exists without verified flag", () => {
    expect(
      getHashState({ fileHash: { hash: "abc", algorithm: "MD5" } as any })
    ).toBe("computed");
  });

  it("returns 'verified' when stored match in history", () => {
    expect(
      getHashState({
        containerInfo: {
          e01: { stored_hashes: [{ algorithm: "MD5", hash: "abc" }] },
        } as any,
        hashHistory: [{ algorithm: "md5", hash: "abc" }] as HashHistoryEntry[],
      })
    ).toBe("verified");
  });

  it("returns 'stored' when stored hashes exist without match", () => {
    expect(
      getHashState({
        containerInfo: {
          e01: { stored_hashes: [{ algorithm: "MD5", hash: "abc" }] },
        } as any,
        hashHistory: [],
      })
    ).toBe("stored");
  });

  it("returns 'computed' when only history exists", () => {
    expect(
      getHashState({
        hashHistory: [{ algorithm: "MD5", hash: "abc" }] as HashHistoryEntry[],
      })
    ).toBe("computed");
  });

  it("returns 'none' when nothing provided", () => {
    expect(getHashState({})).toBe("none");
  });
});

// =============================================================================
// hasVerifiedMatch
// =============================================================================

describe("hasVerifiedMatch", () => {
  it("returns false with no stored hashes and no history", () => {
    expect(hasVerifiedMatch(undefined, [])).toBe(false);
  });

  it("returns true when stored matches history (case-insensitive)", () => {
    const info = {
      e01: { stored_hashes: [{ algorithm: "MD5", hash: "ABC" }] },
    } as any;
    const history = [{ algorithm: "md5", hash: "abc" }] as HashHistoryEntry[];
    expect(hasVerifiedMatch(info, history)).toBe(true);
  });

  it("returns true when history entries match each other", () => {
    const history = [
      { algorithm: "MD5", hash: "abc" },
      { algorithm: "MD5", hash: "ABC" },
    ] as HashHistoryEntry[];
    expect(hasVerifiedMatch(undefined, history)).toBe(true);
  });

  it("returns false when algorithms differ", () => {
    const info = {
      e01: { stored_hashes: [{ algorithm: "MD5", hash: "abc" }] },
    } as any;
    const history = [{ algorithm: "SHA-1", hash: "abc" }] as HashHistoryEntry[];
    expect(hasVerifiedMatch(info, history)).toBe(false);
  });
});

// =============================================================================
// getStoredHashCount
// =============================================================================

describe("getStoredHashCount", () => {
  it("returns 0 for undefined", () => {
    expect(getStoredHashCount(undefined)).toBe(0);
  });

  it("sums e01 and companion_log hashes", () => {
    expect(
      getStoredHashCount({
        e01: { stored_hashes: [{}, {}] },
        companion_log: { stored_hashes: [{}] },
      } as any)
    ).toBe(3);
  });
});

// =============================================================================
// getTotalHashCount
// =============================================================================

describe("getTotalHashCount", () => {
  it("sums stored + fileHash + history", () => {
    expect(
      getTotalHashCount({
        containerInfo: { e01: { stored_hashes: [{}] } } as any,
        fileHash: { hash: "x" } as any,
        hashHistory: [{} as any, {} as any],
      })
    ).toBe(4);
  });

  it("returns 0 when empty", () => {
    expect(getTotalHashCount({})).toBe(0);
  });
});

// =============================================================================
// isHashing / isCompleting
// =============================================================================

describe("isHashing", () => {
  it("returns true during active hashing < 95%", () => {
    expect(isHashing({ status: "hashing", progress: 50 } as any, null)).toBe(true);
  });

  it("returns false when result exists", () => {
    expect(isHashing({ status: "hashing", progress: 50 } as any, { hash: "x" } as any)).toBe(false);
  });

  it("returns false at >= 95%", () => {
    expect(isHashing({ status: "hashing", progress: 96 } as any, null)).toBe(false);
  });
});

describe("isCompleting", () => {
  it("returns true at >= 95% with no result", () => {
    expect(isCompleting({ status: "hashing", progress: 97 } as any, null)).toBe(true);
  });

  it("returns false when < 95%", () => {
    expect(isCompleting({ status: "hashing", progress: 90 } as any, null)).toBe(false);
  });
});

// =============================================================================
// formatChunks
// =============================================================================

describe("formatChunks", () => {
  it("formats small numbers as-is", () => {
    expect(formatChunks(42)).toBe("42");
  });

  it("formats thousands with k suffix", () => {
    expect(formatChunks(5500)).toBe("6k");
  });

  it("formats millions with M suffix", () => {
    expect(formatChunks(2500000)).toBe("2.5M");
  });
});
