// =============================================================================
// recovery/types — getHealthColor, formatFileSize, formatAutosaveAge tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { getHealthColor, formatFileSize, formatAutosaveAge } from "../types";

describe("getHealthColor", () => {
  it('returns "text-success" for Healthy', () => {
    expect(getHealthColor("Healthy")).toBe("text-success");
  });

  it('returns "text-warning" for Warning', () => {
    expect(getHealthColor("Warning")).toBe("text-warning");
  });

  it('returns "text-error" for Critical', () => {
    expect(getHealthColor("Critical")).toBe("text-error");
  });

  it('returns "text-txt-muted" for unknown status', () => {
    expect(getHealthColor("Unknown" as any)).toBe("text-txt-muted");
  });
});

describe("formatFileSize", () => {
  it("formats bytes", () => {
    expect(formatFileSize(0)).toBe("0 B");
    expect(formatFileSize(512)).toBe("512 B");
    expect(formatFileSize(1023)).toBe("1023 B");
  });

  it("formats kilobytes", () => {
    expect(formatFileSize(1024)).toBe("1.0 KB");
    expect(formatFileSize(1536)).toBe("1.5 KB");
    expect(formatFileSize(1024 * 1023)).toMatch(/KB/);
  });

  it("formats megabytes", () => {
    expect(formatFileSize(1024 * 1024)).toBe("1.0 MB");
    expect(formatFileSize(1024 * 1024 * 5)).toBe("5.0 MB");
    expect(formatFileSize(1024 * 1024 * 512)).toBe("512.0 MB");
  });

  it("formats gigabytes", () => {
    expect(formatFileSize(1024 * 1024 * 1024)).toBe("1.0 GB");
    expect(formatFileSize(1024 * 1024 * 1024 * 2.5)).toBe("2.5 GB");
  });
});

describe("formatAutosaveAge", () => {
  it('returns "Unknown" for null', () => {
    expect(formatAutosaveAge(null)).toBe("Unknown");
  });

  it("formats seconds", () => {
    expect(formatAutosaveAge(0)).toBe("0 seconds ago");
    expect(formatAutosaveAge(30)).toBe("30 seconds ago");
    expect(formatAutosaveAge(59)).toBe("59 seconds ago");
  });

  it("formats minutes", () => {
    expect(formatAutosaveAge(60)).toBe("1 minutes ago");
    expect(formatAutosaveAge(120)).toBe("2 minutes ago");
    expect(formatAutosaveAge(3599)).toBe("59 minutes ago");
  });

  it("formats hours", () => {
    expect(formatAutosaveAge(3600)).toBe("1 hours ago");
    expect(formatAutosaveAge(7200)).toBe("2 hours ago");
    expect(formatAutosaveAge(86399)).toBe("23 hours ago");
  });

  it("formats days", () => {
    expect(formatAutosaveAge(86400)).toBe("1 days ago");
    expect(formatAutosaveAge(172800)).toBe("2 days ago");
  });
});
