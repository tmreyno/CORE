// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  formatBytes,
  normalizeError,
  typeClass,
  parseTimestamp,
  formatHashDate,
  formatDuration,
  formatOffset,
  formatOffsetLabel,
  byteToHex,
  byteToAscii,
} from "../utils";

// =============================================================================
// formatBytes
// =============================================================================
describe("formatBytes", () => {
  it("returns '0 B' for zero", () => {
    expect(formatBytes(0)).toBe("0 B");
  });

  it("returns '0 B' for falsy values", () => {
    expect(formatBytes(NaN)).toBe("0 B");
  });

  it("formats bytes", () => {
    expect(formatBytes(100)).toBe("100.0 B");
    expect(formatBytes(1)).toBe("1.00 B");
  });

  it("formats kilobytes", () => {
    expect(formatBytes(1024)).toBe("1.00 KB");
    expect(formatBytes(1536)).toBe("1.50 KB");
    expect(formatBytes(10240)).toBe("10.0 KB");
  });

  it("formats megabytes", () => {
    expect(formatBytes(1048576)).toBe("1.00 MB");
    expect(formatBytes(52428800)).toBe("50.0 MB");
  });

  it("formats gigabytes", () => {
    expect(formatBytes(1073741824)).toBe("1.00 GB");
  });

  it("formats terabytes", () => {
    expect(formatBytes(1099511627776)).toBe("1.00 TB");
  });
});

// =============================================================================
// normalizeError
// =============================================================================
describe("normalizeError", () => {
  it("returns 'Unknown error' for falsy values", () => {
    expect(normalizeError(null)).toBe("Unknown error");
    expect(normalizeError(undefined)).toBe("Unknown error");
    expect(normalizeError("")).toBe("Unknown error");
  });

  it("returns string errors as-is", () => {
    expect(normalizeError("Something broke")).toBe("Something broke");
  });

  it("extracts message from Error objects", () => {
    expect(normalizeError(new Error("Disk full"))).toBe("Disk full");
  });

  it("extracts message from objects with message property", () => {
    expect(normalizeError({ message: "Custom error" })).toBe("Custom error");
  });

  it("JSON-stringifies other objects", () => {
    expect(normalizeError({ code: 42 })).toBe('{"code":42}');
  });
});

// =============================================================================
// typeClass
// =============================================================================
describe("typeClass", () => {
  it("returns type-ad1 for AD1 containers", () => {
    expect(typeClass("AD1")).toBe("type-ad1");
    expect(typeClass("ad1")).toBe("type-ad1");
  });

  it("returns type-e01 for E01/EnCase containers", () => {
    expect(typeClass("E01")).toBe("type-e01");
    expect(typeClass("EnCase")).toBe("type-e01");
    expect(typeClass("encase")).toBe("type-e01");
  });

  it("returns type-l01 for L01 containers", () => {
    expect(typeClass("L01")).toBe("type-l01");
  });

  it("returns type-raw for raw/dd images", () => {
    expect(typeClass("RAW")).toBe("type-raw");
    expect(typeClass("dd")).toBe("type-raw");
  });

  it("returns type-ufed for UFED containers", () => {
    expect(typeClass("UFED")).toBe("type-ufed");
    expect(typeClass("UFD")).toBe("type-ufed");
  });

  it("returns type-archive for archive formats", () => {
    expect(typeClass("tar")).toBe("type-archive");
    expect(typeClass("7z")).toBe("type-archive");
    expect(typeClass("zip")).toBe("type-archive");
    expect(typeClass("rar")).toBe("type-archive");
    expect(typeClass("gz")).toBe("type-archive");
  });

  it("returns type-other for unknown types", () => {
    expect(typeClass("unknown")).toBe("type-other");
    expect(typeClass("")).toBe("type-other");
  });
});

// =============================================================================
// parseTimestamp
// =============================================================================
describe("parseTimestamp", () => {
  it("parses ISO 8601 dates", () => {
    const d = parseTimestamp("2024-01-15T10:30:00Z");
    expect(d).toBeInstanceOf(Date);
    expect(d!.getUTCFullYear()).toBe(2024);
    expect(d!.getUTCMonth()).toBe(0); // January
    expect(d!.getUTCDate()).toBe(15);
  });

  it("parses UFED format: DD/MM/YYYY HH:MM:SS (timezone)", () => {
    const d = parseTimestamp("26/08/2024 17:48:01 (-4)");
    expect(d).toBeInstanceOf(Date);
    expect(d!.getFullYear()).toBe(2024);
  });

  it("parses DD/MM/YYYY format without time", () => {
    const d = parseTimestamp("15/01/2024");
    expect(d).toBeInstanceOf(Date);
    expect(d!.getFullYear()).toBe(2024);
    expect(d!.getMonth()).toBe(0); // January
    expect(d!.getDate()).toBe(15);
  });

  it("returns null for unparseable timestamps", () => {
    expect(parseTimestamp("not a date")).toBeNull();
    expect(parseTimestamp("")).toBeNull();
  });

  it("parses standard Date-parseable strings", () => {
    const d = parseTimestamp("January 15, 2024");
    expect(d).toBeInstanceOf(Date);
    expect(d!.getFullYear()).toBe(2024);
  });
});

// =============================================================================
// formatHashDate
// =============================================================================
describe("formatHashDate", () => {
  it("formats ISO dates to short format", () => {
    const result = formatHashDate("2024-01-15T10:30:00Z");
    // Contains month abbreviation and year
    expect(result).toBeTruthy();
    expect(result.length).toBeGreaterThan(0);
  });

  it("returns input for unparseable dates", () => {
    expect(formatHashDate("not a date")).toBe("not a date");
  });
});

// =============================================================================
// formatDuration
// =============================================================================
describe("formatDuration", () => {
  it("formats seconds", () => {
    expect(formatDuration(5.5)).toBe("5.5s");
    expect(formatDuration(30)).toBe("30.0s");
    expect(formatDuration(0.1)).toBe("0.1s");
  });

  it("formats minutes and seconds", () => {
    expect(formatDuration(90)).toBe("1m 30s");
    expect(formatDuration(125)).toBe("2m 5s");
  });

  it("handles exactly 60 seconds", () => {
    expect(formatDuration(60)).toBe("1m 0s");
  });
});

// =============================================================================
// formatOffset
// =============================================================================
describe("formatOffset", () => {
  it("returns empty string for null/undefined", () => {
    expect(formatOffset(null)).toBe("");
    expect(formatOffset(undefined)).toBe("");
  });

  it("formats zero with default width", () => {
    expect(formatOffset(0)).toBe("00000000");
  });

  it("formats hex values uppercase with padding", () => {
    expect(formatOffset(0xA1B2)).toBe("0000A1B2");
  });

  it("supports custom width", () => {
    expect(formatOffset(255, { width: 4 })).toBe("00FF");
  });

  it("supports prefix", () => {
    expect(formatOffset(255, { prefix: true })).toBe("0x000000FF");
  });

  it("supports both width and prefix", () => {
    expect(formatOffset(16, { width: 4, prefix: true })).toBe("0x0010");
  });
});

// =============================================================================
// formatOffsetLabel
// =============================================================================
describe("formatOffsetLabel", () => {
  it("returns empty string for null/undefined", () => {
    expect(formatOffsetLabel(null)).toBe("");
    expect(formatOffsetLabel(undefined)).toBe("");
  });

  it("formats with @ 0x prefix", () => {
    expect(formatOffsetLabel(0)).toBe("@ 0x0");
    expect(formatOffsetLabel(255)).toBe("@ 0xFF");
    expect(formatOffsetLabel(4096)).toBe("@ 0x1000");
  });
});

// =============================================================================
// byteToHex
// =============================================================================
describe("byteToHex", () => {
  it("formats single digit bytes with leading zero", () => {
    expect(byteToHex(0)).toBe("00");
    expect(byteToHex(10)).toBe("0A");
    expect(byteToHex(15)).toBe("0F");
  });

  it("formats two digit bytes", () => {
    expect(byteToHex(16)).toBe("10");
    expect(byteToHex(255)).toBe("FF");
    expect(byteToHex(128)).toBe("80");
  });
});

// =============================================================================
// byteToAscii
// =============================================================================
describe("byteToAscii", () => {
  it("returns printable ASCII characters", () => {
    expect(byteToAscii(65)).toBe("A");
    expect(byteToAscii(97)).toBe("a");
    expect(byteToAscii(48)).toBe("0");
    expect(byteToAscii(32)).toBe(" ");
    expect(byteToAscii(126)).toBe("~");
  });

  it("returns dot for non-printable characters", () => {
    expect(byteToAscii(0)).toBe(".");
    expect(byteToAscii(31)).toBe(".");
    expect(byteToAscii(127)).toBe(".");
    expect(byteToAscii(255)).toBe(".");
  });
});
