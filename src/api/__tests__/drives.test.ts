// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { formatDriveSize } from "../drives";

// =============================================================================
// formatDriveSize
// =============================================================================

describe("formatDriveSize", () => {
  it("returns '0 B' for zero bytes", () => {
    expect(formatDriveSize(0)).toBe("0 B");
  });

  it("formats bytes", () => {
    expect(formatDriveSize(512)).toBe("512 B");
  });

  it("formats kilobytes", () => {
    expect(formatDriveSize(1024)).toBe("1.0 KB");
  });

  it("formats megabytes", () => {
    expect(formatDriveSize(1048576)).toBe("1.0 MB");
  });

  it("formats gigabytes", () => {
    expect(formatDriveSize(256 * 1024 * 1024 * 1024)).toBe("256.0 GB");
  });

  it("formats terabytes", () => {
    expect(formatDriveSize(2 * 1024 * 1024 * 1024 * 1024)).toBe("2.0 TB");
  });

  it("formats fractional values", () => {
    expect(formatDriveSize(1536)).toBe("1.5 KB");
  });
});
