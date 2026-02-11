// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  compareHashes,
  findMatchingStoredHash,
  hasStoredHashForAlgorithm,
  deduplicateStoredHashes,
  groupHashesByAlgorithm,
} from "../hashUtils";
import type { StoredHashEntry, HashAlgorithmName } from "../../types/hash";

// =============================================================================
// Test Data
// =============================================================================

const MD5_HASH = "D41D8CD98F00B204E9800998ECF8427E";
const SHA1_HASH = "DA39A3EE5E6B4B0D3255BFEF95601890AFD80709";
const SHA256_HASH =
  "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855";

const storedHashes: StoredHashEntry[] = [
  { algorithm: "MD5", hash: MD5_HASH, source: "container" },
  { algorithm: "SHA-1", hash: SHA1_HASH, source: "container" },
  { algorithm: "SHA-256", hash: SHA256_HASH, source: "container" },
];

// =============================================================================
// compareHashes
// =============================================================================

describe("compareHashes", () => {
  it("returns true for matching hashes with same algorithm", () => {
    expect(compareHashes(MD5_HASH, MD5_HASH, "MD5", "MD5")).toBe(true);
  });

  it("returns true for case-insensitive hash comparison", () => {
    expect(
      compareHashes(MD5_HASH.toLowerCase(), MD5_HASH.toUpperCase(), "MD5", "MD5")
    ).toBe(true);
  });

  it("returns false for different hash values same algorithm", () => {
    expect(compareHashes(MD5_HASH, "AABBCCDD", "MD5", "MD5")).toBe(false);
  });

  it("returns false for different algorithms", () => {
    expect(compareHashes(MD5_HASH, MD5_HASH, "MD5", "SHA-1")).toBe(false);
  });

  it("returns true for algorithm name variations (SHA1 vs SHA-1)", () => {
    expect(compareHashes(SHA1_HASH, SHA1_HASH, "SHA1", "SHA-1")).toBe(true);
  });

  it("returns true for algorithm name variations (sha-256 vs SHA256)", () => {
    expect(compareHashes(SHA256_HASH, SHA256_HASH, "sha-256", "SHA256")).toBe(
      true
    );
  });

  it("returns false for empty hash strings", () => {
    expect(compareHashes("", "", "MD5", "MD5")).toBe(true); // empty == empty
    expect(compareHashes("", MD5_HASH, "MD5", "MD5")).toBe(false);
  });
});

// =============================================================================
// findMatchingStoredHash
// =============================================================================

describe("findMatchingStoredHash", () => {
  it("finds matching MD5 hash", () => {
    const result = findMatchingStoredHash(MD5_HASH, "MD5", storedHashes);
    expect(result).not.toBeNull();
    expect(result!.algorithm).toBe("MD5");
    expect(result!.hash).toBe(MD5_HASH);
  });

  it("finds matching SHA-256 hash", () => {
    const result = findMatchingStoredHash(SHA256_HASH, "SHA-256", storedHashes);
    expect(result).not.toBeNull();
    expect(result!.algorithm).toBe("SHA-256");
  });

  it("returns null when hash value doesn't match", () => {
    const result = findMatchingStoredHash("DEADBEEF", "MD5", storedHashes);
    expect(result).toBeNull();
  });

  it("returns null when algorithm doesn't match any stored", () => {
    const result = findMatchingStoredHash(MD5_HASH, "SHA-512" as HashAlgorithmName, storedHashes);
    expect(result).toBeNull();
  });

  it("returns null for empty stored hashes", () => {
    const result = findMatchingStoredHash(MD5_HASH, "MD5", []);
    expect(result).toBeNull();
  });

  it("matches case-insensitively", () => {
    const result = findMatchingStoredHash(
      MD5_HASH.toLowerCase(),
      "MD5",
      storedHashes
    );
    expect(result).not.toBeNull();
  });

  it("returns the first match when multiple exist", () => {
    const duplicates: StoredHashEntry[] = [
      { algorithm: "MD5", hash: MD5_HASH, source: "container" },
      { algorithm: "MD5", hash: MD5_HASH, source: "computed" },
    ];
    const result = findMatchingStoredHash(MD5_HASH, "MD5", duplicates);
    expect(result).not.toBeNull();
    expect(result!.source).toBe("container");
  });
});

// =============================================================================
// hasStoredHashForAlgorithm
// =============================================================================

describe("hasStoredHashForAlgorithm", () => {
  it("returns true when algorithm is present", () => {
    expect(hasStoredHashForAlgorithm(storedHashes, "MD5")).toBe(true);
    expect(hasStoredHashForAlgorithm(storedHashes, "SHA-1")).toBe(true);
    expect(hasStoredHashForAlgorithm(storedHashes, "SHA-256")).toBe(true);
  });

  it("returns false when algorithm is absent", () => {
    expect(hasStoredHashForAlgorithm(storedHashes, "SHA-512" as HashAlgorithmName)).toBe(
      false
    );
  });

  it("returns false for empty array", () => {
    expect(hasStoredHashForAlgorithm([], "MD5")).toBe(false);
  });

  it("matches algorithm name variations", () => {
    // "SHA1" should match "SHA-1" stored hash
    expect(hasStoredHashForAlgorithm(storedHashes, "SHA1" as HashAlgorithmName)).toBe(
      true
    );
  });

  it("is case-insensitive for algorithm names", () => {
    expect(hasStoredHashForAlgorithm(storedHashes, "md5" as HashAlgorithmName)).toBe(
      true
    );
  });
});

// =============================================================================
// deduplicateStoredHashes
// =============================================================================

describe("deduplicateStoredHashes", () => {
  it("returns same array when no duplicates", () => {
    const result = deduplicateStoredHashes(storedHashes);
    expect(result).toHaveLength(3);
  });

  it("removes duplicate algorithms keeping first", () => {
    const withDupes: StoredHashEntry[] = [
      { algorithm: "MD5", hash: "AAAA", source: "container" },
      { algorithm: "MD5", hash: "BBBB", source: "computed" },
      { algorithm: "SHA-1", hash: SHA1_HASH, source: "container" },
    ];
    const result = deduplicateStoredHashes(withDupes);
    expect(result).toHaveLength(2);
    expect(result[0].hash).toBe("AAAA"); // first MD5 kept
  });

  it("normalizes algorithm names during dedup", () => {
    const withVariants: StoredHashEntry[] = [
      { algorithm: "SHA-1", hash: SHA1_HASH, source: "container" },
      { algorithm: "SHA1", hash: "DIFFERENT", source: "computed" },
    ];
    const result = deduplicateStoredHashes(withVariants);
    expect(result).toHaveLength(1);
    expect(result[0].hash).toBe(SHA1_HASH);
  });

  it("returns empty array for empty input", () => {
    expect(deduplicateStoredHashes([])).toHaveLength(0);
  });

  it("normalizes algorithm names in output", () => {
    const input: StoredHashEntry[] = [
      { algorithm: "sha-256", hash: SHA256_HASH, source: "container" },
    ];
    const result = deduplicateStoredHashes(input);
    expect(result[0].algorithm).toBe("SHA256"); // normalized form
  });

  it("handles single entry", () => {
    const single: StoredHashEntry[] = [
      { algorithm: "MD5", hash: MD5_HASH, source: "container" },
    ];
    const result = deduplicateStoredHashes(single);
    expect(result).toHaveLength(1);
  });
});

// =============================================================================
// groupHashesByAlgorithm
// =============================================================================

describe("groupHashesByAlgorithm", () => {
  it("groups hashes by normalized algorithm name", () => {
    const groups = groupHashesByAlgorithm(storedHashes);
    expect(Object.keys(groups)).toHaveLength(3);
    expect(groups["MD5"]).toHaveLength(1);
    expect(groups["SHA1"]).toHaveLength(1);
    expect(groups["SHA256"]).toHaveLength(1);
  });

  it("groups duplicates together", () => {
    const withDupes: StoredHashEntry[] = [
      { algorithm: "MD5", hash: "AAAA", source: "container" },
      { algorithm: "MD5", hash: "BBBB", source: "computed" },
      { algorithm: "SHA-1", hash: SHA1_HASH, source: "container" },
    ];
    const groups = groupHashesByAlgorithm(withDupes);
    expect(groups["MD5"]).toHaveLength(2);
    expect(groups["SHA1"]).toHaveLength(1);
  });

  it("normalizes algorithm names in keys", () => {
    const input: StoredHashEntry[] = [
      { algorithm: "sha-256", hash: SHA256_HASH, source: "container" },
      { algorithm: "SHA256", hash: "OTHER", source: "computed" },
    ];
    const groups = groupHashesByAlgorithm(input);
    expect(Object.keys(groups)).toHaveLength(1);
    expect(groups["SHA256"]).toHaveLength(2);
  });

  it("returns empty object for empty input", () => {
    const groups = groupHashesByAlgorithm([]);
    expect(Object.keys(groups)).toHaveLength(0);
  });

  it("normalizes algorithm names in entries", () => {
    const input: StoredHashEntry[] = [
      { algorithm: "sha-1", hash: SHA1_HASH, source: "container" },
    ];
    const groups = groupHashesByAlgorithm(input);
    expect(groups["SHA1"][0].algorithm).toBe("SHA1");
  });

  it("preserves hash and source in grouped entries", () => {
    const groups = groupHashesByAlgorithm(storedHashes);
    expect(groups["MD5"][0].hash).toBe(MD5_HASH);
    expect(groups["MD5"][0].source).toBe("container");
  });
});
