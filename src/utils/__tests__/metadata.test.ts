// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  formatTimestamp,
  truncateHash,
  formatAlgorithm,
  getVerificationStatus,
  getVerificationIcon,
  getVerificationClass,
  formatNumber,
  formatCount,
  formatOffset,
  formatDecimalOffset,
  filterEmptyFields,
  groupFieldsByCategory,
  sortFieldsByLabel,
  getSourceIndicator,
  getSourceDescription,
  SOURCE_INDICATORS,
  type DisplayField,
} from "../metadata";

describe("metadata", () => {
  // ===========================================================================
  // formatTimestamp
  // ===========================================================================
  describe("formatTimestamp", () => {
    it("formats a valid Unix timestamp", () => {
      // 1704067200 = Jan 1, 2024 00:00:00 UTC (may show Dec 31 in some timezones)
      const result = formatTimestamp(1704067200);
      // Should produce a non-empty formatted string regardless of timezone
      expect(result.length).toBeGreaterThan(0);
      // Should contain a year (either 2023 or 2024 depending on timezone)
      expect(result).toMatch(/202[34]/);
    });

    it("returns empty for null/undefined", () => {
      expect(formatTimestamp(null)).toBe("");
      expect(formatTimestamp(undefined)).toBe("");
    });

    it("returns empty for NaN-producing timestamps", () => {
      expect(formatTimestamp(NaN)).toBe("");
    });
  });

  // ===========================================================================
  // truncateHash
  // ===========================================================================
  describe("truncateHash", () => {
    it("truncates long hashes", () => {
      const hash = "a".repeat(64);
      const result = truncateHash(hash, 16);
      expect(result).toBe("a".repeat(16) + "...");
    });

    it("returns short hashes unchanged", () => {
      expect(truncateHash("abcdef", 16)).toBe("abcdef");
    });

    it("uses default length of 16", () => {
      const hash = "a".repeat(64);
      const result = truncateHash(hash);
      expect(result).toBe("a".repeat(16) + "...");
    });

    it("handles empty/falsy input", () => {
      expect(truncateHash("")).toBe("");
    });
  });

  // ===========================================================================
  // formatAlgorithm
  // ===========================================================================
  describe("formatAlgorithm", () => {
    it("normalizes SHA-256 variants", () => {
      expect(formatAlgorithm("sha256")).toBe("SHA-256");
      expect(formatAlgorithm("SHA256")).toBe("SHA-256");
      expect(formatAlgorithm("SHA-256")).toBe("SHA-256");
    });

    it("normalizes SHA-1 variants", () => {
      expect(formatAlgorithm("sha1")).toBe("SHA-1");
      expect(formatAlgorithm("SHA1")).toBe("SHA-1");
      expect(formatAlgorithm("SHA-1")).toBe("SHA-1");
    });

    it("normalizes MD5", () => {
      expect(formatAlgorithm("md5")).toBe("MD5");
      expect(formatAlgorithm("MD5")).toBe("MD5");
    });

    it("uppercases unknown algorithms", () => {
      expect(formatAlgorithm("blake2b")).toBe("BLAKE2B");
    });

    it("returns empty for empty input", () => {
      expect(formatAlgorithm("")).toBe("");
    });
  });

  // ===========================================================================
  // getVerificationStatus
  // ===========================================================================
  describe("getVerificationStatus", () => {
    it("returns verified for true", () => {
      expect(getVerificationStatus(true)).toBe("verified");
    });

    it("returns mismatch for false", () => {
      expect(getVerificationStatus(false)).toBe("mismatch");
    });

    it("returns pending for null", () => {
      expect(getVerificationStatus(null)).toBe("pending");
    });

    it("returns unknown for undefined", () => {
      expect(getVerificationStatus(undefined)).toBe("unknown");
    });
  });

  // ===========================================================================
  // getVerificationIcon
  // ===========================================================================
  describe("getVerificationIcon", () => {
    it("returns double-check for verified", () => {
      expect(getVerificationIcon("verified")).toBe("✓✓");
    });

    it("returns X for mismatch", () => {
      expect(getVerificationIcon("mismatch")).toBe("✗");
    });

    it("returns ? for pending", () => {
      expect(getVerificationIcon("pending")).toBe("?");
    });

    it("returns empty for unknown", () => {
      expect(getVerificationIcon("unknown")).toBe("");
    });
  });

  // ===========================================================================
  // getVerificationClass
  // ===========================================================================
  describe("getVerificationClass", () => {
    it("returns green for verified", () => {
      expect(getVerificationClass("verified")).toContain("green");
    });

    it("returns red for mismatch", () => {
      expect(getVerificationClass("mismatch")).toContain("red");
    });

    it("returns yellow for pending", () => {
      expect(getVerificationClass("pending")).toContain("yellow");
    });

    it("returns muted for unknown", () => {
      expect(getVerificationClass("unknown")).toContain("muted");
    });
  });

  // ===========================================================================
  // formatNumber
  // ===========================================================================
  describe("formatNumber", () => {
    it("formats numbers with locale separators", () => {
      const result = formatNumber(1234567);
      // Locale-dependent, but should contain digits
      expect(result).toContain("1");
      expect(result.length).toBeGreaterThan(3);
    });

    it("returns empty for null/undefined", () => {
      expect(formatNumber(null)).toBe("");
      expect(formatNumber(undefined)).toBe("");
    });

    it("formats zero", () => {
      expect(formatNumber(0)).toBe("0");
    });
  });

  // ===========================================================================
  // formatCount
  // ===========================================================================
  describe("formatCount", () => {
    it("uses singular for count of 1", () => {
      expect(formatCount(1, "item")).toContain("item");
      expect(formatCount(1, "item")).not.toContain("items");
    });

    it("uses plural for count > 1", () => {
      expect(formatCount(5, "item")).toContain("items");
    });

    it("uses custom plural when provided", () => {
      expect(formatCount(5, "child", "children")).toContain("children");
    });

    it("uses plural for count of 0", () => {
      expect(formatCount(0, "item")).toContain("items");
    });

    it("returns empty for null/undefined", () => {
      expect(formatCount(null, "item")).toBe("");
      expect(formatCount(undefined, "item")).toBe("");
    });
  });

  // ===========================================================================
  // formatOffset
  // ===========================================================================
  describe("formatOffset", () => {
    it("formats hex offset with default prefix", () => {
      expect(formatOffset(255)).toBe("@ 0xFF");
    });

    it("formats with custom prefix", () => {
      expect(formatOffset(255, "offset")).toBe("offset 0xFF");
    });

    it("returns empty for null/undefined", () => {
      expect(formatOffset(null)).toBe("");
      expect(formatOffset(undefined)).toBe("");
    });

    it("formats zero", () => {
      expect(formatOffset(0)).toBe("@ 0x0");
    });
  });

  // ===========================================================================
  // formatDecimalOffset
  // ===========================================================================
  describe("formatDecimalOffset", () => {
    it("formats decimal offset with separators", () => {
      const result = formatDecimalOffset(1234);
      expect(result).toContain("@");
      expect(result).toContain("1");
    });

    it("returns empty for null/undefined", () => {
      expect(formatDecimalOffset(null)).toBe("");
      expect(formatDecimalOffset(undefined)).toBe("");
    });
  });

  // ===========================================================================
  // filterEmptyFields
  // ===========================================================================
  describe("filterEmptyFields", () => {
    it("removes null/undefined/empty values", () => {
      const fields: DisplayField[] = [
        { label: "Name", value: "test" },
        { label: "Empty", value: "" },
        { label: "Null", value: null },
        { label: "Undef", value: undefined },
        { label: "Size", value: 42 },
      ];
      const filtered = filterEmptyFields(fields);
      expect(filtered).toHaveLength(2);
      expect(filtered[0].label).toBe("Name");
      expect(filtered[1].label).toBe("Size");
    });

    it("returns empty array for all-empty input", () => {
      const fields: DisplayField[] = [
        { label: "A", value: null },
        { label: "B", value: "" },
      ];
      expect(filterEmptyFields(fields)).toHaveLength(0);
    });

    it("returns all fields when none are empty", () => {
      const fields: DisplayField[] = [
        { label: "A", value: "a" },
        { label: "B", value: 0 },
      ];
      expect(filterEmptyFields(fields)).toHaveLength(2);
    });
  });

  // ===========================================================================
  // groupFieldsByCategory
  // ===========================================================================
  describe("groupFieldsByCategory", () => {
    it("groups fields by category", () => {
      const fields: DisplayField[] = [
        { label: "Name", value: "test", category: "General" },
        { label: "Size", value: "42", category: "General" },
        { label: "Hash", value: "abc", category: "Security" },
      ];
      const groups = groupFieldsByCategory(fields);
      expect(groups.get("General")).toHaveLength(2);
      expect(groups.get("Security")).toHaveLength(1);
    });

    it("uses 'General' as default category", () => {
      const fields: DisplayField[] = [
        { label: "Name", value: "test" },
      ];
      const groups = groupFieldsByCategory(fields);
      expect(groups.has("General")).toBe(true);
    });

    it("handles empty input", () => {
      const groups = groupFieldsByCategory([]);
      expect(groups.size).toBe(0);
    });
  });

  // ===========================================================================
  // sortFieldsByLabel
  // ===========================================================================
  describe("sortFieldsByLabel", () => {
    it("sorts alphabetically by label", () => {
      const fields: DisplayField[] = [
        { label: "Zebra", value: "z" },
        { label: "Apple", value: "a" },
        { label: "Mango", value: "m" },
      ];
      const sorted = sortFieldsByLabel(fields);
      expect(sorted[0].label).toBe("Apple");
      expect(sorted[1].label).toBe("Mango");
      expect(sorted[2].label).toBe("Zebra");
    });

    it("does not mutate original array", () => {
      const fields: DisplayField[] = [
        { label: "B", value: "b" },
        { label: "A", value: "a" },
      ];
      sortFieldsByLabel(fields);
      expect(fields[0].label).toBe("B");
    });
  });

  // ===========================================================================
  // Source helpers
  // ===========================================================================
  describe("getSourceIndicator", () => {
    it("returns correct symbols", () => {
      expect(getSourceIndicator("container")).toBe(SOURCE_INDICATORS.container);
      expect(getSourceIndicator("companion")).toBe(SOURCE_INDICATORS.companion);
      expect(getSourceIndicator("computed")).toBe(SOURCE_INDICATORS.computed);
    });

    it("returns empty for undefined source", () => {
      expect(getSourceIndicator(undefined)).toBe("");
    });
  });

  describe("getSourceDescription", () => {
    it("returns descriptions for known sources", () => {
      expect(getSourceDescription("container")).toContain("container");
      expect(getSourceDescription("companion")).toContain("companion");
      expect(getSourceDescription("computed")).toContain("verification");
    });

    it("returns empty for undefined", () => {
      expect(getSourceDescription(undefined)).toBe("");
    });
  });
});
