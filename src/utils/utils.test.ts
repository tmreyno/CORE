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
  formatHashDate,
  formatDuration, 
  debounce,
  formatOffset,
  formatOffsetLabel,
  byteToHex,
  byteToAscii,
  getExtension,
  getBasename,
  parseTimestamp,
  formatDateByPreference,
} from "../utils";

describe("Utility Functions", () => {
  describe("getExtension", () => {
    it("extracts lowercase extension from filename", () => {
      expect(getExtension("file.PDF")).toBe("pdf");
      expect(getExtension("document.TXT")).toBe("txt");
      expect(getExtension("photo.JPEG")).toBe("jpeg");
    });

    it("handles filenames with multiple dots", () => {
      expect(getExtension("archive.tar.gz")).toBe("gz");
      expect(getExtension("my.file.name.txt")).toBe("txt");
    });

    it("returns empty string for files without extension", () => {
      expect(getExtension("README")).toBe("");
      expect(getExtension("Makefile")).toBe("");
    });

    it("handles empty string", () => {
      expect(getExtension("")).toBe("");
    });
  });

  describe("getBasename", () => {
    it("extracts filename from path", () => {
      expect(getBasename("/path/to/file.txt")).toBe("file.txt");
      expect(getBasename("/Users/test/document.pdf")).toBe("document.pdf");
    });

    it("handles just filename", () => {
      expect(getBasename("file.txt")).toBe("file.txt");
    });

    it("handles trailing slash", () => {
      expect(getBasename("/path/to/")).toBe("");
    });

    it("handles empty string", () => {
      expect(getBasename("")).toBe("");
    });
  });

  describe("formatBytes", () => {
    it("formats 0 bytes", () => {
      expect(formatBytes(0)).toBe("0 B");
    });

    it("formats bytes correctly", () => {
      // Function uses toFixed(2) for values < 10, toFixed(1) otherwise
      expect(formatBytes(500)).toBe("500.0 B");
    });

    it("formats kilobytes correctly", () => {
      expect(formatBytes(1024)).toBe("1.00 KB");
    });

    it("formats megabytes correctly", () => {
      expect(formatBytes(1048576)).toBe("1.00 MB");
    });

    it("formats gigabytes correctly", () => {
      expect(formatBytes(1073741824)).toBe("1.00 GB");
    });

    it("formats terabytes correctly", () => {
      expect(formatBytes(1099511627776)).toBe("1.00 TB");
    });
  });

  describe("normalizeError", () => {
    it("returns 'Unknown error' for null/undefined", () => {
      expect(normalizeError(null)).toBe("Unknown error");
      expect(normalizeError(undefined)).toBe("Unknown error");
    });

    it("returns string as-is", () => {
      expect(normalizeError("Test error")).toBe("Test error");
    });

    it("extracts message from Error objects", () => {
      expect(normalizeError(new Error("Error message"))).toBe("Error message");
    });

    it("stringifies other objects", () => {
      const result = normalizeError({ code: 404 });
      expect(result).toContain("404");
    });
  });

  describe("typeClass", () => {
    it("returns correct class for AD1", () => {
      expect(typeClass("ad1")).toBe("type-ad1");
    });

    it("returns correct class for E01", () => {
      expect(typeClass("e01")).toBe("type-e01");
    });

    it("returns correct class for unknown", () => {
      expect(typeClass("unknown")).toBe("type-other");
    });
  });

  describe("formatHashDate", () => {
    it("formats ISO date string", () => {
      const date = "2024-01-15T10:30:00Z";
      const formatted = formatHashDate(date);
      // Function uses short format: 'Jan 15, 24'
      expect(formatted).toContain("Jan");
      expect(formatted).toContain("15");
    });

    it("returns original timestamp for unparseable input", () => {
      // parseTimestamp returns null for invalid strings, so formatHashDate returns original
      const result = formatHashDate("invalid");
      expect(result).toBe("invalid");
    });

    it("parses UFED date format", () => {
      // UFED format: DD/MM/YYYY HH:MM:SS (timezone)
      const result = formatHashDate("26/08/2024 17:48:01 (-4)");
      expect(result).toContain("Aug");
      expect(result).toContain("26");
    });

    it("parses DD/MM/YYYY format", () => {
      const result = formatHashDate("15/01/2024");
      expect(result).toContain("Jan");
      expect(result).toContain("15");
    });
  });

  describe("formatDuration", () => {
    it("formats seconds only", () => {
      expect(formatDuration(30)).toBe("30.0s");
    });

    it("formats minutes and seconds", () => {
      expect(formatDuration(90)).toBe("1m 30s");
    });
  });

  describe("debounce", () => {
    it("delays function execution", async () => {
      let count = 0;
      const increment = debounce(() => count++, 50);

      increment();
      increment();
      increment();

      expect(count).toBe(0);
      
      await new Promise(resolve => setTimeout(resolve, 100));
      expect(count).toBe(1);
    });

    it("only executes once after multiple rapid calls", async () => {
      let count = 0;
      const increment = debounce(() => count++, 50);

      for (let i = 0; i < 10; i++) {
        increment();
      }

      await new Promise(resolve => setTimeout(resolve, 100));
      expect(count).toBe(1);
    });
  });

  describe("formatOffset", () => {
    it("formats offset with default padding", () => {
      expect(formatOffset(255)).toBe("000000FF");
    });

    it("formats with custom width", () => {
      expect(formatOffset(255, { width: 4 })).toBe("00FF");
    });

    it("formats with 0x prefix", () => {
      expect(formatOffset(255, { prefix: true })).toBe("0x000000FF");
    });

    it("returns empty string for null/undefined", () => {
      expect(formatOffset(null)).toBe("");
      expect(formatOffset(undefined)).toBe("");
    });
  });

  describe("formatOffsetLabel", () => {
    it("formats offset with @ prefix", () => {
      expect(formatOffsetLabel(255)).toBe("@ 0xFF");
    });

    it("returns empty string for null/undefined", () => {
      expect(formatOffsetLabel(null)).toBe("");
      expect(formatOffsetLabel(undefined)).toBe("");
    });
  });

  describe("byteToHex", () => {
    it("converts byte to uppercase hex", () => {
      expect(byteToHex(0)).toBe("00");
      expect(byteToHex(255)).toBe("FF");
      expect(byteToHex(10)).toBe("0A");
    });
  });

  describe("byteToAscii", () => {
    it("returns character for printable ASCII", () => {
      expect(byteToAscii(65)).toBe("A");
      expect(byteToAscii(97)).toBe("a");
      expect(byteToAscii(48)).toBe("0");
    });

    it("returns dot for non-printable characters", () => {
      expect(byteToAscii(0)).toBe(".");
      expect(byteToAscii(31)).toBe(".");
      expect(byteToAscii(127)).toBe(".");
    });
  });

  // ===========================================================================
  // parseTimestamp
  // ===========================================================================

  describe("parseTimestamp", () => {
    it("parses ISO 8601 date string", () => {
      const result = parseTimestamp("2024-08-26T17:48:01Z");
      expect(result).toBeInstanceOf(Date);
      expect(result!.getUTCFullYear()).toBe(2024);
      expect(result!.getUTCMonth()).toBe(7); // August = 7 (0-indexed)
      expect(result!.getUTCDate()).toBe(26);
    });

    it("parses standard date string", () => {
      const result = parseTimestamp("2024-01-15");
      expect(result).toBeInstanceOf(Date);
      expect(result!.getFullYear()).toBe(2024);
    });

    it("parses UFED format with negative timezone", () => {
      const result = parseTimestamp("26/08/2024 17:48:01 (-4)");
      expect(result).toBeInstanceOf(Date);
      expect(result!.getFullYear()).toBe(2024);
    });

    it("parses UFED format with positive timezone", () => {
      const result = parseTimestamp("15/01/2024 09:30:00 (+5)");
      expect(result).toBeInstanceOf(Date);
      expect(result!.getFullYear()).toBe(2024);
    });

    it("parses DD/MM/YYYY format without time", () => {
      const result = parseTimestamp("26/08/2024");
      expect(result).toBeInstanceOf(Date);
      expect(result!.getFullYear()).toBe(2024);
      expect(result!.getMonth()).toBe(7); // August
      expect(result!.getDate()).toBe(26);
    });

    it("parses DD/MM/YYYY with different dates", () => {
      // Note: "01/12/2023" is parseable by JS Date as MM/DD/YYYY (US format),
      // so parseTimestamp uses the standard parser first. Use a date that's
      // unambiguously DD/MM/YYYY (day > 12) to test the DD/MM/YYYY regex path.
      const result = parseTimestamp("25/12/2023");
      expect(result).toBeInstanceOf(Date);
      expect(result!.getFullYear()).toBe(2023);
      expect(result!.getMonth()).toBe(11); // December
      expect(result!.getDate()).toBe(25);
    });

    it("returns null for completely invalid strings", () => {
      expect(parseTimestamp("not a date")).toBeNull();
      expect(parseTimestamp("")).toBeNull();
      expect(parseTimestamp("abc/def/ghij")).toBeNull();
    });

    it("returns null for partial UFED format", () => {
      // Missing timezone
      expect(parseTimestamp("26/08/2024 17:48:01")).toBeNull();
    });
  });

  // ===========================================================================
  // formatDateByPreference
  // ===========================================================================

  describe("formatDateByPreference", () => {
    it("returns empty string for null", () => {
      expect(formatDateByPreference(null)).toBe("");
    });

    it("returns empty string for undefined", () => {
      expect(formatDateByPreference(undefined)).toBe("");
    });

    it("formats a Date object", () => {
      const date = new Date("2024-06-15T10:30:00Z");
      const result = formatDateByPreference(date);
      expect(result).toBeTruthy();
      expect(typeof result).toBe("string");
      expect(result.length).toBeGreaterThan(0);
    });

    it("formats an ISO string", () => {
      const result = formatDateByPreference("2024-06-15T10:30:00Z");
      expect(result).toBeTruthy();
      expect(result.length).toBeGreaterThan(0);
    });

    it("formats with includeTime=false", () => {
      const date = new Date("2024-06-15T10:30:00Z");
      const withTime = formatDateByPreference(date, true);
      const withoutTime = formatDateByPreference(date, false);
      // Without time should be shorter or equal
      expect(withoutTime.length).toBeLessThanOrEqual(withTime.length);
    });

    it("returns original string for invalid date string", () => {
      expect(formatDateByPreference("not a date")).toBe("not a date");
    });

    it("returns empty string for invalid Date object", () => {
      expect(formatDateByPreference(new Date("invalid"))).toBe("");
    });

    it("handles UFED formatted dates via parseTimestamp", () => {
      const result = formatDateByPreference("26/08/2024");
      expect(result).toBeTruthy();
      expect(result.length).toBeGreaterThan(0);
    });
  });
});
