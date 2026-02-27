// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi } from "vitest";

// Mock ../utils to avoid SolidJS/Tauri dependencies
vi.mock("../utils", () => ({
  formatDateByPreference: vi.fn((date: string | null | undefined, _includeTime?: boolean) => {
    if (!date) return "";
    return `formatted:${date}`;
  }),
}));

import {
  formatDate,
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
} from "./metadata";

// =============================================================================
// formatDate
// =============================================================================

describe("formatDate", () => {
  it("delegates to formatDateByPreference", () => {
    expect(formatDate("2024-01-15")).toBe("formatted:2024-01-15");
  });

  it("returns empty string for null", () => {
    expect(formatDate(null)).toBe("");
  });

  it("returns empty string for undefined", () => {
    expect(formatDate(undefined)).toBe("");
  });
});

// =============================================================================
// formatTimestamp
// =============================================================================

describe("formatTimestamp", () => {
  it("formats a valid Unix timestamp", () => {
    const result = formatTimestamp(1700000000); // Nov 14, 2023
    expect(result).toBeTruthy();
    expect(result.length).toBeGreaterThan(0);
  });

  it("returns empty string for null", () => {
    expect(formatTimestamp(null)).toBe("");
  });

  it("returns empty string for undefined", () => {
    expect(formatTimestamp(undefined)).toBe("");
  });

  it("returns empty string for NaN-producing timestamps", () => {
    expect(formatTimestamp(NaN)).toBe("");
  });

  it("handles zero timestamp (Unix epoch)", () => {
    const result = formatTimestamp(0);
    expect(result).toBeTruthy(); // Jan 1, 1970
  });
});

// =============================================================================
// truncateHash
// =============================================================================

describe("truncateHash", () => {
  it("truncates a long hash to default length", () => {
    const hash = "d41d8cd98f00b204e9800998ecf8427e";
    const result = truncateHash(hash);
    expect(result).toBe("d41d8cd98f00b204...");
    expect(result.length).toBe(19); // 16 chars + "..."
  });

  it("truncates to custom length", () => {
    const hash = "d41d8cd98f00b204e9800998ecf8427e";
    expect(truncateHash(hash, 8)).toBe("d41d8cd9...");
  });

  it("returns full hash if shorter than length", () => {
    expect(truncateHash("abc", 16)).toBe("abc");
  });

  it("returns full hash if exactly the length", () => {
    expect(truncateHash("abcdefghijklmnop", 16)).toBe("abcdefghijklmnop");
  });

  it("handles empty hash", () => {
    expect(truncateHash("")).toBe("");
  });

  it("handles falsy hash", () => {
    expect(truncateHash(undefined as unknown as string)).toBe(undefined);
  });
});

// =============================================================================
// formatAlgorithm
// =============================================================================

describe("formatAlgorithm", () => {
  it("normalizes SHA-256 variants", () => {
    expect(formatAlgorithm("SHA256")).toBe("SHA-256");
    expect(formatAlgorithm("sha256")).toBe("SHA-256");
    expect(formatAlgorithm("SHA-256")).toBe("SHA-256");
    expect(formatAlgorithm("sha-256")).toBe("SHA-256");
  });

  it("normalizes SHA-1 variants", () => {
    expect(formatAlgorithm("SHA1")).toBe("SHA-1");
    expect(formatAlgorithm("sha1")).toBe("SHA-1");
    expect(formatAlgorithm("SHA-1")).toBe("SHA-1");
    expect(formatAlgorithm("sha-1")).toBe("SHA-1");
  });

  it("normalizes MD5", () => {
    expect(formatAlgorithm("MD5")).toBe("MD5");
    expect(formatAlgorithm("md5")).toBe("MD5");
  });

  it("uppercases unknown algorithms", () => {
    expect(formatAlgorithm("crc32")).toBe("CRC32");
    expect(formatAlgorithm("blake2b")).toBe("BLAKE2B");
  });

  it("returns empty string for empty input", () => {
    expect(formatAlgorithm("")).toBe("");
  });
});

// =============================================================================
// Verification Status
// =============================================================================

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

describe("getVerificationIcon", () => {
  it("returns double check for verified", () => {
    expect(getVerificationIcon("verified")).toBe("✓✓");
  });

  it("returns X for mismatch", () => {
    expect(getVerificationIcon("mismatch")).toBe("✗");
  });

  it("returns question mark for pending", () => {
    expect(getVerificationIcon("pending")).toBe("?");
  });

  it("returns empty string for unknown", () => {
    expect(getVerificationIcon("unknown")).toBe("");
  });
});

describe("getVerificationClass", () => {
  it("returns green class for verified", () => {
    expect(getVerificationClass("verified")).toBe("text-green-400");
  });

  it("returns red class for mismatch", () => {
    expect(getVerificationClass("mismatch")).toBe("text-red-400");
  });

  it("returns yellow class for pending", () => {
    expect(getVerificationClass("pending")).toBe("text-yellow-400");
  });

  it("returns muted class for unknown", () => {
    expect(getVerificationClass("unknown")).toBe("text-txt-muted");
  });
});

// =============================================================================
// Number Formatting
// =============================================================================

describe("formatNumber", () => {
  it("formats a number with locale separators", () => {
    const result = formatNumber(1234567);
    expect(result).toBeTruthy();
    // Locale-dependent but should contain digits
    expect(result).toContain("1");
  });

  it("returns empty string for null", () => {
    expect(formatNumber(null)).toBe("");
  });

  it("returns empty string for undefined", () => {
    expect(formatNumber(undefined)).toBe("");
  });

  it("accepts Intl.NumberFormat options", () => {
    const result = formatNumber(0.5, { style: "percent" });
    expect(result).toContain("50");
  });

  it("formats zero", () => {
    expect(formatNumber(0)).toBe("0");
  });
});

describe("formatCount", () => {
  it("uses singular for count of 1", () => {
    expect(formatCount(1, "item")).toBe("1 item");
  });

  it("uses plural for count > 1", () => {
    expect(formatCount(5, "item")).toBe("5 items");
  });

  it("uses plural for count of 0", () => {
    expect(formatCount(0, "item")).toBe("0 items");
  });

  it("uses custom plural form", () => {
    expect(formatCount(3, "entry", "entries")).toBe("3 entries");
  });

  it("uses custom plural for count of 1 still uses singular", () => {
    expect(formatCount(1, "entry", "entries")).toBe("1 entry");
  });

  it("returns empty string for null", () => {
    expect(formatCount(null, "item")).toBe("");
  });

  it("returns empty string for undefined", () => {
    expect(formatCount(undefined, "item")).toBe("");
  });

  it("formats large counts with locale separators", () => {
    const result = formatCount(1000000, "file");
    expect(result).toContain("file");
    // Large numbers should be locale-formatted
    expect(result.length).toBeGreaterThan(5);
  });
});

// =============================================================================
// Offset Formatting
// =============================================================================

describe("formatOffset", () => {
  it("formats offset as hex with default prefix", () => {
    expect(formatOffset(0)).toBe("@ 0x0");
    expect(formatOffset(255)).toBe("@ 0xFF");
    expect(formatOffset(4096)).toBe("@ 0x1000");
  });

  it("uses custom prefix", () => {
    expect(formatOffset(16, "offset:")).toBe("offset: 0x10");
  });

  it("returns empty string for null", () => {
    expect(formatOffset(null)).toBe("");
  });

  it("returns empty string for undefined", () => {
    expect(formatOffset(undefined)).toBe("");
  });
});

describe("formatDecimalOffset", () => {
  it("formats offset as decimal with default prefix", () => {
    expect(formatDecimalOffset(0)).toContain("@ ");
    expect(formatDecimalOffset(0)).toContain("0");
  });

  it("uses custom prefix", () => {
    expect(formatDecimalOffset(100, "pos:")).toContain("pos:");
  });

  it("returns empty string for null", () => {
    expect(formatDecimalOffset(null)).toBe("");
  });

  it("returns empty string for undefined", () => {
    expect(formatDecimalOffset(undefined)).toBe("");
  });
});

// =============================================================================
// Field Operations
// =============================================================================

describe("filterEmptyFields", () => {
  it("removes fields with null values", () => {
    const fields: DisplayField[] = [
      { label: "Name", value: "test" },
      { label: "Empty", value: null },
    ];
    const result = filterEmptyFields(fields);
    expect(result).toHaveLength(1);
    expect(result[0].label).toBe("Name");
  });

  it("removes fields with undefined values", () => {
    const fields: DisplayField[] = [
      { label: "Name", value: "test" },
      { label: "Empty", value: undefined },
    ];
    expect(filterEmptyFields(fields)).toHaveLength(1);
  });

  it("removes fields with empty string values", () => {
    const fields: DisplayField[] = [
      { label: "Name", value: "test" },
      { label: "Empty", value: "" },
    ];
    expect(filterEmptyFields(fields)).toHaveLength(1);
  });

  it("keeps fields with 0 values", () => {
    const fields: DisplayField[] = [
      { label: "Count", value: 0 },
    ];
    expect(filterEmptyFields(fields)).toHaveLength(1);
  });

  it("returns empty array for empty input", () => {
    expect(filterEmptyFields([])).toHaveLength(0);
  });
});

describe("groupFieldsByCategory", () => {
  it("groups fields by category", () => {
    const fields: DisplayField[] = [
      { label: "Name", value: "test", category: "General" },
      { label: "Size", value: 100, category: "General" },
      { label: "Hash", value: "abc", category: "Security" },
    ];
    const groups = groupFieldsByCategory(fields);
    expect(groups.size).toBe(2);
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

describe("sortFieldsByLabel", () => {
  it("sorts fields alphabetically by label", () => {
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

// =============================================================================
// Source Indicators
// =============================================================================

describe("SOURCE_INDICATORS", () => {
  it("has expected symbols", () => {
    expect(SOURCE_INDICATORS.container).toBe("◆");
    expect(SOURCE_INDICATORS.companion).toBe("◇");
    expect(SOURCE_INDICATORS.computed).toBe("▣");
  });
});

describe("getSourceIndicator", () => {
  it("returns symbol for container source", () => {
    expect(getSourceIndicator("container")).toBe("◆");
  });

  it("returns symbol for companion source", () => {
    expect(getSourceIndicator("companion")).toBe("◇");
  });

  it("returns symbol for computed source", () => {
    expect(getSourceIndicator("computed")).toBe("▣");
  });

  it("returns empty string for undefined", () => {
    expect(getSourceIndicator(undefined)).toBe("");
  });
});

describe("getSourceDescription", () => {
  it("returns description for container", () => {
    expect(getSourceDescription("container")).toBe("Stored in container header");
  });

  it("returns description for companion", () => {
    expect(getSourceDescription("companion")).toBe("From companion log file");
  });

  it("returns description for computed", () => {
    expect(getSourceDescription("computed")).toBe("Computed during verification");
  });

  it("returns empty string for undefined", () => {
    expect(getSourceDescription(undefined)).toBe("");
  });
});
