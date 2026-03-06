// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  getDataTypeColor,
  formatSize,
} from "../helpers";

// =============================================================================
// getDataTypeColor
// =============================================================================

describe("getDataTypeColor", () => {
  it("returns green for string registry types", () => {
    expect(getDataTypeColor("REG_SZ")).toBe("text-green-400");
    expect(getDataTypeColor("REG_EXPAND_SZ")).toBe("text-green-400");
  });

  it("returns blue for numeric registry types", () => {
    expect(getDataTypeColor("REG_DWORD")).toBe("text-blue-400");
    expect(getDataTypeColor("REG_QWORD")).toBe("text-blue-400");
    expect(getDataTypeColor("REG_DWORD_BIG_ENDIAN")).toBe("text-blue-400");
  });

  it("returns orange for REG_BINARY", () => {
    expect(getDataTypeColor("REG_BINARY")).toBe("text-orange-400");
  });

  it("returns purple for REG_MULTI_SZ", () => {
    expect(getDataTypeColor("REG_MULTI_SZ")).toBe("text-purple-400");
  });

  it("returns muted for REG_NONE", () => {
    expect(getDataTypeColor("REG_NONE")).toBe("text-txt-muted");
  });

  it("returns secondary for unknown types", () => {
    expect(getDataTypeColor("REG_CUSTOM")).toBe("text-txt-secondary");
  });
});

// =============================================================================
// formatSize
// =============================================================================

describe("formatSize", () => {
  it("returns '0 B' for zero", () => {
    expect(formatSize(0)).toBe("0 B");
  });

  it("formats bytes under 1 KB", () => {
    expect(formatSize(512)).toBe("512 B");
    expect(formatSize(1)).toBe("1 B");
    expect(formatSize(1023)).toBe("1023 B");
  });

  it("formats kilobytes", () => {
    expect(formatSize(1024)).toBe("1.0 KB");
    expect(formatSize(2048)).toBe("2.0 KB");
    expect(formatSize(1536)).toBe("1.5 KB");
  });

  it("formats megabytes", () => {
    expect(formatSize(1048576)).toBe("1.0 MB");
    expect(formatSize(5242880)).toBe("5.0 MB");
  });
});
