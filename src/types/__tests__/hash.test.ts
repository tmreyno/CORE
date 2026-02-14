// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  HASH_ALGORITHMS,
  HASH_ALGORITHM_MAP,
  normalizeAlgorithm,
  algorithmsMatch,
  HashError,
  type HashAlgorithmName,
  type HashErrorCode,
} from "../hash";

// =============================================================================
// HASH_ALGORITHMS Constants
// =============================================================================

describe("HASH_ALGORITHMS", () => {
  it("contains all expected algorithms", () => {
    expect(HASH_ALGORITHMS.MD5).toBe("MD5");
    expect(HASH_ALGORITHMS.SHA1).toBe("SHA-1");
    expect(HASH_ALGORITHMS.SHA256).toBe("SHA-256");
    expect(HASH_ALGORITHMS.SHA512).toBe("SHA-512");
    expect(HASH_ALGORITHMS.BLAKE3).toBe("BLAKE3");
    expect(HASH_ALGORITHMS.BLAKE2).toBe("BLAKE2b");
    expect(HASH_ALGORITHMS.XXH3).toBe("XXH3");
    expect(HASH_ALGORITHMS.XXH64).toBe("XXH64");
    expect(HASH_ALGORITHMS.CRC32).toBe("CRC32");
  });

  it("has 9 algorithms", () => {
    expect(Object.keys(HASH_ALGORITHMS)).toHaveLength(9);
  });
});

// =============================================================================
// HASH_ALGORITHM_MAP
// =============================================================================

describe("HASH_ALGORITHM_MAP", () => {
  it("maps common MD5 variants", () => {
    expect(HASH_ALGORITHM_MAP["MD5"]).toBe("MD5");
  });

  it("maps SHA1 variants", () => {
    expect(HASH_ALGORITHM_MAP["SHA1"]).toBe("SHA-1");
    expect(HASH_ALGORITHM_MAP["SHA-1"]).toBe("SHA-1");
  });

  it("maps SHA256 variants", () => {
    expect(HASH_ALGORITHM_MAP["SHA256"]).toBe("SHA-256");
    expect(HASH_ALGORITHM_MAP["SHA-256"]).toBe("SHA-256");
  });

  it("maps SHA512 variants", () => {
    expect(HASH_ALGORITHM_MAP["SHA512"]).toBe("SHA-512");
    expect(HASH_ALGORITHM_MAP["SHA-512"]).toBe("SHA-512");
  });

  it("maps BLAKE3 variants", () => {
    expect(HASH_ALGORITHM_MAP["Blake3"]).toBe("BLAKE3");
    expect(HASH_ALGORITHM_MAP["BLAKE3"]).toBe("BLAKE3");
  });

  it("maps BLAKE2 variants", () => {
    expect(HASH_ALGORITHM_MAP["Blake2"]).toBe("BLAKE2b");
    expect(HASH_ALGORITHM_MAP["BLAKE2"]).toBe("BLAKE2b");
    expect(HASH_ALGORITHM_MAP["BLAKE2b"]).toBe("BLAKE2b");
  });

  it("maps XXH3", () => {
    expect(HASH_ALGORITHM_MAP["XXH3"]).toBe("XXH3");
  });

  it("maps XXH64", () => {
    expect(HASH_ALGORITHM_MAP["XXH64"]).toBe("XXH64");
  });

  it("maps CRC32", () => {
    expect(HASH_ALGORITHM_MAP["CRC32"]).toBe("CRC32");
  });

  it("returns undefined for unknown algorithms", () => {
    expect(HASH_ALGORITHM_MAP["RIPEMD160"]).toBeUndefined();
    expect(HASH_ALGORITHM_MAP[""]).toBeUndefined();
  });
});

// =============================================================================
// normalizeAlgorithm
// =============================================================================

describe("normalizeAlgorithm", () => {
  it("uppercases lowercase input", () => {
    expect(normalizeAlgorithm("md5")).toBe("MD5");
    expect(normalizeAlgorithm("sha256")).toBe("SHA256");
  });

  it("removes hyphens", () => {
    expect(normalizeAlgorithm("SHA-256")).toBe("SHA256");
    expect(normalizeAlgorithm("SHA-1")).toBe("SHA1");
    expect(normalizeAlgorithm("SHA-512")).toBe("SHA512");
  });

  it("removes all non-alphanumeric characters", () => {
    expect(normalizeAlgorithm("BLAKE2b")).toBe("BLAKE2B");
    expect(normalizeAlgorithm("sha_256")).toBe("SHA256");
    expect(normalizeAlgorithm("md-5")).toBe("MD5");
  });

  it("handles mixed case", () => {
    expect(normalizeAlgorithm("Blake3")).toBe("BLAKE3");
    expect(normalizeAlgorithm("Sha256")).toBe("SHA256");
  });

  it("handles empty string", () => {
    expect(normalizeAlgorithm("")).toBe("");
  });

  it("handles already normalized input", () => {
    expect(normalizeAlgorithm("MD5")).toBe("MD5");
    expect(normalizeAlgorithm("SHA256")).toBe("SHA256");
    expect(normalizeAlgorithm("CRC32")).toBe("CRC32");
  });
});

// =============================================================================
// algorithmsMatch
// =============================================================================

describe("algorithmsMatch", () => {
  it("matches identical algorithms", () => {
    expect(algorithmsMatch("MD5", "MD5")).toBe(true);
    expect(algorithmsMatch("SHA-256", "SHA-256")).toBe(true);
  });

  it("matches case-insensitive", () => {
    expect(algorithmsMatch("md5", "MD5")).toBe(true);
    expect(algorithmsMatch("sha256", "SHA256")).toBe(true);
  });

  it("matches with and without hyphens", () => {
    expect(algorithmsMatch("SHA-256", "SHA256")).toBe(true);
    expect(algorithmsMatch("SHA-1", "SHA1")).toBe(true);
    expect(algorithmsMatch("SHA-512", "SHA512")).toBe(true);
  });

  it("matches Blake variants", () => {
    expect(algorithmsMatch("Blake3", "BLAKE3")).toBe(true);
    expect(algorithmsMatch("blake2b", "BLAKE2B")).toBe(true);
  });

  it("does not match different algorithms", () => {
    expect(algorithmsMatch("MD5", "SHA256")).toBe(false);
    expect(algorithmsMatch("SHA-1", "SHA-256")).toBe(false);
    expect(algorithmsMatch("BLAKE3", "BLAKE2b")).toBe(false);
  });

  it("handles empty strings", () => {
    expect(algorithmsMatch("", "")).toBe(true);
    expect(algorithmsMatch("", "MD5")).toBe(false);
  });
});

// =============================================================================
// HashError
// =============================================================================

describe("HashError", () => {
  describe("constructor", () => {
    it("creates error with code and message", () => {
      const err = new HashError("FILE_READ_ERROR", "Cannot read file");
      expect(err.code).toBe("FILE_READ_ERROR");
      expect(err.message).toBe("Cannot read file");
      expect(err.name).toBe("HashError");
      expect(err.context).toBeUndefined();
    });

    it("creates error with context", () => {
      const err = new HashError("VERIFICATION_FAILED", "Hash mismatch", {
        expected: "abc123",
        actual: "def456",
      });
      expect(err.context).toEqual({ expected: "abc123", actual: "def456" });
    });

    it("is instance of Error", () => {
      const err = new HashError("COMPUTATION_ERROR", "fail");
      expect(err).toBeInstanceOf(Error);
    });

    it("is instance of HashError", () => {
      const err = new HashError("COMPUTATION_ERROR", "fail");
      expect(err).toBeInstanceOf(HashError);
    });
  });

  describe("is()", () => {
    it("returns true for matching code", () => {
      const err = new HashError("SEGMENT_MISSING", "Missing segments");
      expect(err.is("SEGMENT_MISSING")).toBe(true);
    });

    it("returns false for non-matching code", () => {
      const err = new HashError("SEGMENT_MISSING", "Missing segments");
      expect(err.is("FILE_READ_ERROR")).toBe(false);
      expect(err.is("VERIFICATION_FAILED")).toBe(false);
    });
  });

  describe("getUserMessage()", () => {
    const testCases: [HashErrorCode, string][] = [
      ["SEGMENT_MISSING", "Cannot hash incomplete container"],
      ["VERIFICATION_FAILED", "Hash verification failed"],
      ["UNSUPPORTED_FORMAT", "does not support hash verification"],
      ["FILE_READ_ERROR", "Failed to read file"],
      ["PROGRESS_TIMEOUT", "timed out"],
      ["COMPUTATION_ERROR", "Hash computation failed"],
      ["INVALID_ALGORITHM", "Unknown hash algorithm"],
    ];

    testCases.forEach(([code, expectedSubstring]) => {
      it(`returns user message for ${code}`, () => {
        const err = new HashError(code, "internal msg");
        expect(err.getUserMessage()).toContain(expectedSubstring);
      });
    });

    it("returns message itself for unknown codes", () => {
      // Force an unknown code for the default branch
      const err = new HashError("FILE_READ_ERROR", "custom message");
      // Manually override code to test default
      (err as any).code = "UNKNOWN_CODE";
      expect(err.getUserMessage()).toBe("custom message");
    });
  });

  describe("toString()", () => {
    it("includes code and message", () => {
      const err = new HashError("FILE_READ_ERROR", "cannot read");
      const str = err.toString();
      expect(str).toContain("[FILE_READ_ERROR]");
      expect(str).toContain("cannot read");
    });

    it("includes context when present", () => {
      const err = new HashError("VERIFICATION_FAILED", "mismatch", {
        file: "test.e01",
      });
      const str = err.toString();
      expect(str).toContain("Context:");
      expect(str).toContain("test.e01");
    });

    it("omits context section when no context", () => {
      const err = new HashError("COMPUTATION_ERROR", "fail");
      const str = err.toString();
      expect(str).not.toContain("Context:");
    });

    it("omits context section when context is empty object", () => {
      const err = new HashError("COMPUTATION_ERROR", "fail", {});
      const str = err.toString();
      expect(str).not.toContain("Context:");
    });
  });
});
