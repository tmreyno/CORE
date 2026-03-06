// =============================================================================
// hex viewer constants — color map and region color lookup tests
// =============================================================================

import { describe, it, expect, vi } from "vitest";

// Mock the preferences import
vi.mock("../../preferences", () => ({
  getPreference: vi.fn(() => 50), // 50 MB default
}));

import {
  BYTES_PER_LINE,
  INITIAL_LOAD_SIZE,
  LOAD_MORE_SIZE,
  SCROLL_THRESHOLD,
  getMaxLoadedBytes,
  COLOR_MAP,
  NAVIGATED_COLOR,
  getRegionColor,
} from "../constants";

describe("numeric constants", () => {
  it("BYTES_PER_LINE is 16", () => {
    expect(BYTES_PER_LINE).toBe(16);
  });

  it("INITIAL_LOAD_SIZE is 64KB", () => {
    expect(INITIAL_LOAD_SIZE).toBe(65536);
  });

  it("LOAD_MORE_SIZE is 32KB", () => {
    expect(LOAD_MORE_SIZE).toBe(32768);
  });

  it("SCROLL_THRESHOLD is 200px", () => {
    expect(SCROLL_THRESHOLD).toBe(200);
  });
});

describe("getMaxLoadedBytes", () => {
  it("converts MB preference to bytes", () => {
    // Mock returns 50 MB
    expect(getMaxLoadedBytes()).toBe(50 * 1024 * 1024);
  });
});

describe("COLOR_MAP", () => {
  it("has 8 region color entries", () => {
    expect(Object.keys(COLOR_MAP)).toHaveLength(8);
  });

  it("all values are rgba strings", () => {
    for (const [key, value] of Object.entries(COLOR_MAP)) {
      expect(value).toMatch(/^rgba\(/);
    }
  });

  it("includes all expected region types", () => {
    const expected = [
      "region-signature",
      "region-header",
      "region-segment",
      "region-metadata",
      "region-data",
      "region-checksum",
      "region-reserved",
    ];
    // region-header and region-segment share a key but both may appear
    for (const key of expected) {
      expect(COLOR_MAP).toHaveProperty(key);
    }
  });
});

describe("NAVIGATED_COLOR", () => {
  it("is a green rgba with higher opacity", () => {
    expect(NAVIGATED_COLOR).toBe("rgba(34, 197, 94, 0.4)");
  });
});

describe("getRegionColor", () => {
  it("returns mapped color for known region", () => {
    expect(getRegionColor("region-signature")).toBe("rgba(239, 68, 68, 0.15)");
  });

  it("returns mapped color for region-data", () => {
    expect(getRegionColor("region-data")).toBe("rgba(34, 197, 94, 0.15)");
  });

  it("returns mapped color for region-checksum", () => {
    expect(getRegionColor("region-checksum")).toBe("rgba(59, 130, 246, 0.15)");
  });

  it("returns fallback gray for unknown region", () => {
    expect(getRegionColor("region-unknown")).toBe("#6a6a7a");
  });

  it("returns fallback for empty string", () => {
    expect(getRegionColor("")).toBe("#6a6a7a");
  });

  it("returns region-footer color", () => {
    expect(getRegionColor("region-footer")).toBe("rgba(236, 72, 153, 0.15)");
  });
});
