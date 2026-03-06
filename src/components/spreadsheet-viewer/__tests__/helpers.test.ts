// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  formatCell,
  getColumnLetter,
} from "../helpers";
import type { CellValue } from "../types";

// =============================================================================
// formatCell
// =============================================================================

describe("formatCell", () => {
  it("returns empty string for null/undefined", () => {
    expect(formatCell(null as unknown as CellValue)).toBe("");
    expect(formatCell(undefined as unknown as CellValue)).toBe("");
  });

  it("returns empty string for Empty type", () => {
    expect(formatCell({ type: "Empty" })).toBe("");
  });

  it("returns empty string when value is undefined", () => {
    expect(formatCell({ type: "String", value: undefined })).toBe("");
  });

  it("returns empty string when value is null", () => {
    expect(formatCell({ type: "String", value: null as unknown as string })).toBe("");
  });

  it("formats string values", () => {
    expect(formatCell({ type: "String", value: "hello" })).toBe("hello");
  });

  it("formats int values", () => {
    expect(formatCell({ type: "Int", value: 42 })).toBe("42");
  });

  it("formats float values as locale string", () => {
    const result = formatCell({ type: "Float", value: 3.14159 });
    // Locale formatting varies, but should contain the digits
    expect(result).toContain("3");
    expect(result).toContain("14");
  });

  it("formats float string value", () => {
    expect(formatCell({ type: "Float", value: "3.14" as unknown as number })).toBe("3.14");
  });

  it("formats boolean true", () => {
    expect(formatCell({ type: "Bool", value: true })).toBe("TRUE");
  });

  it("formats boolean false", () => {
    expect(formatCell({ type: "Bool", value: false })).toBe("FALSE");
  });

  it("formats DateTime as string", () => {
    expect(formatCell({ type: "DateTime", value: "2025-01-01" })).toBe("2025-01-01");
  });

  it("formats Error as string", () => {
    expect(formatCell({ type: "Error", value: "#REF!" })).toBe("#REF!");
  });

  it("handles unknown type gracefully", () => {
    expect(formatCell({ type: "Unknown" as CellValue["type"], value: "test" })).toBe("test");
  });
});

// =============================================================================
// getColumnLetter
// =============================================================================

describe("getColumnLetter", () => {
  it("returns A for index 0", () => {
    expect(getColumnLetter(0)).toBe("A");
  });

  it("returns B for index 1", () => {
    expect(getColumnLetter(1)).toBe("B");
  });

  it("returns Z for index 25", () => {
    expect(getColumnLetter(25)).toBe("Z");
  });

  it("returns AA for index 26", () => {
    expect(getColumnLetter(26)).toBe("AA");
  });

  it("returns AB for index 27", () => {
    expect(getColumnLetter(27)).toBe("AB");
  });

  it("returns AZ for index 51", () => {
    expect(getColumnLetter(51)).toBe("AZ");
  });

  it("returns BA for index 52", () => {
    expect(getColumnLetter(52)).toBe("BA");
  });

  it("handles triple letters correctly", () => {
    // AAA = 26*26 + 26 + 0 = 702
    expect(getColumnLetter(702)).toBe("AAA");
  });
});
