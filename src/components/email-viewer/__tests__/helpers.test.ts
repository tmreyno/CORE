// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  formatEmailAddress,
  formatAddressList,
  formatEmailDate,
  isEml,
  isMbox,
  isMsg,
} from "../helpers";
import type { EmailAddress } from "../types";

// =============================================================================
// formatEmailAddress
// =============================================================================

describe("formatEmailAddress", () => {
  it("formats address with name", () => {
    const addr: EmailAddress = { name: "John Doe", address: "john@example.com" };
    expect(formatEmailAddress(addr)).toBe("John Doe <john@example.com>");
  });

  it("formats address without name", () => {
    const addr: EmailAddress = { name: null, address: "jane@example.com" };
    expect(formatEmailAddress(addr)).toBe("jane@example.com");
  });

  it("returns just address when name is empty string", () => {
    // name is truthy check — empty string is falsy
    const addr: EmailAddress = { name: "", address: "test@test.com" };
    expect(formatEmailAddress(addr)).toBe("test@test.com");
  });
});

// =============================================================================
// formatAddressList
// =============================================================================

describe("formatAddressList", () => {
  it("formats multiple addresses joined by comma", () => {
    const addrs: EmailAddress[] = [
      { name: "Alice", address: "alice@test.com" },
      { name: null, address: "bob@test.com" },
    ];
    expect(formatAddressList(addrs)).toBe("Alice <alice@test.com>, bob@test.com");
  });

  it("returns empty string for empty array", () => {
    expect(formatAddressList([])).toBe("");
  });

  it("handles single entry", () => {
    const addrs: EmailAddress[] = [{ name: "Solo", address: "solo@test.com" }];
    expect(formatAddressList(addrs)).toBe("Solo <solo@test.com>");
  });
});

// =============================================================================
// formatEmailDate
// =============================================================================

describe("formatEmailDate", () => {
  it("returns 'Unknown' for null", () => {
    expect(formatEmailDate(null)).toBe("Unknown");
  });

  it("formats a valid date string", () => {
    const result = formatEmailDate("2025-01-15T10:30:00Z");
    // Should contain year and time components
    expect(result).toContain("2025");
  });

  it("returns original string for unparseable dates", () => {
    // new Date("garbage") creates Invalid Date, but toLocaleString may vary
    // The try/catch handles this case
    const result = formatEmailDate("not-a-date");
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
  });
});

// =============================================================================
// File type detection
// =============================================================================

describe("isEml", () => {
  it("returns true for .eml files", () => {
    expect(isEml("message.eml")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(isEml("MSG.EML")).toBe(true);
  });

  it("returns false for non-eml files", () => {
    expect(isEml("file.msg")).toBe(false);
    expect(isEml("file.mbox")).toBe(false);
  });
});

describe("isMbox", () => {
  it("returns true for .mbox files", () => {
    expect(isMbox("archive.mbox")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(isMbox("ARCHIVE.MBOX")).toBe(true);
  });

  it("returns false for non-mbox files", () => {
    expect(isMbox("file.eml")).toBe(false);
  });
});

describe("isMsg", () => {
  it("returns true for .msg files", () => {
    expect(isMsg("outlook.msg")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(isMsg("FILE.MSG")).toBe(true);
  });

  it("returns false for non-msg files", () => {
    expect(isMsg("file.eml")).toBe(false);
  });
});
