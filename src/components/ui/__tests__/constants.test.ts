// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  CONTAINER_ICON_COLORS,
  getContainerIconColor,
  getContainerIconType,
  type ContainerIconType,
} from "../constants";

// =============================================================================
// CONTAINER_ICON_COLORS
// =============================================================================
describe("CONTAINER_ICON_COLORS", () => {
  it("has all expected container type keys", () => {
    expect(CONTAINER_ICON_COLORS.ad1).toBeTruthy();
    expect(CONTAINER_ICON_COLORS.e01).toBeTruthy();
    expect(CONTAINER_ICON_COLORS.l01).toBeTruthy();
    expect(CONTAINER_ICON_COLORS.raw).toBeTruthy();
    expect(CONTAINER_ICON_COLORS.ufed).toBeTruthy();
    expect(CONTAINER_ICON_COLORS.archive).toBeTruthy();
    expect(CONTAINER_ICON_COLORS.default).toBeTruthy();
  });
});

// =============================================================================
// getContainerIconColor
// =============================================================================
describe("getContainerIconColor", () => {
  it("returns AD1 color for ad1 types", () => {
    expect(getContainerIconColor("ad1")).toBe(CONTAINER_ICON_COLORS.ad1);
    expect(getContainerIconColor("AD1")).toBe(CONTAINER_ICON_COLORS.ad1);
    expect(getContainerIconColor("file.ad1")).toBe(CONTAINER_ICON_COLORS.ad1);
  });

  it("returns E01 color for e01/ewf/encase types", () => {
    expect(getContainerIconColor("e01")).toBe(CONTAINER_ICON_COLORS.e01);
    expect(getContainerIconColor("EWF")).toBe(CONTAINER_ICON_COLORS.e01);
    expect(getContainerIconColor("EnCase")).toBe(CONTAINER_ICON_COLORS.e01);
  });

  it("returns L01 color for l01/lx01/lvf types", () => {
    expect(getContainerIconColor("l01")).toBe(CONTAINER_ICON_COLORS.l01);
    expect(getContainerIconColor("LX01")).toBe(CONTAINER_ICON_COLORS.l01);
    expect(getContainerIconColor("lvf")).toBe(CONTAINER_ICON_COLORS.l01);
  });

  it("returns RAW color for raw/dd/img/001 types", () => {
    expect(getContainerIconColor("raw")).toBe(CONTAINER_ICON_COLORS.raw);
    expect(getContainerIconColor("dd")).toBe(CONTAINER_ICON_COLORS.raw);
    expect(getContainerIconColor("disk.img")).toBe(CONTAINER_ICON_COLORS.raw);
    expect(getContainerIconColor("image.001")).toBe(CONTAINER_ICON_COLORS.raw);
  });

  it("returns UFED color for ufed/ufd types", () => {
    expect(getContainerIconColor("ufed")).toBe(CONTAINER_ICON_COLORS.ufed);
    expect(getContainerIconColor("ufd")).toBe(CONTAINER_ICON_COLORS.ufed);
  });

  it("returns ARCHIVE color for archive types", () => {
    expect(getContainerIconColor("zip")).toBe(CONTAINER_ICON_COLORS.archive);
    expect(getContainerIconColor("7z")).toBe(CONTAINER_ICON_COLORS.archive);
    expect(getContainerIconColor("rar")).toBe(CONTAINER_ICON_COLORS.archive);
    expect(getContainerIconColor("tar")).toBe(CONTAINER_ICON_COLORS.archive);
    expect(getContainerIconColor("archive")).toBe(CONTAINER_ICON_COLORS.archive);
  });

  it("returns default color for unknown types", () => {
    expect(getContainerIconColor("unknown")).toBe(CONTAINER_ICON_COLORS.default);
    expect(getContainerIconColor("")).toBe(CONTAINER_ICON_COLORS.default);
    expect(getContainerIconColor("pdf")).toBe(CONTAINER_ICON_COLORS.default);
  });
});

// =============================================================================
// getContainerIconType
// =============================================================================
describe("getContainerIconType", () => {
  it("returns 'ad1' for ad1 types", () => {
    expect(getContainerIconType("ad1")).toBe("ad1");
    expect(getContainerIconType("AD1")).toBe("ad1");
  });

  it("returns 'e01' for e01/ewf/encase types", () => {
    expect(getContainerIconType("e01")).toBe("e01");
    expect(getContainerIconType("ewf")).toBe("e01");
    expect(getContainerIconType("encase")).toBe("e01");
  });

  it("returns 'l01' for l01/lx01/lvf types", () => {
    expect(getContainerIconType("l01")).toBe("l01");
    expect(getContainerIconType("lx01")).toBe("l01");
    expect(getContainerIconType("lvf")).toBe("l01");
  });

  it("returns 'raw' for raw/dd/img/001 types", () => {
    expect(getContainerIconType("raw")).toBe("raw");
    expect(getContainerIconType("dd")).toBe("raw");
    expect(getContainerIconType("img")).toBe("raw");
    expect(getContainerIconType("001")).toBe("raw");
  });

  it("returns 'ufed' for ufed/ufd types", () => {
    expect(getContainerIconType("ufed")).toBe("ufed");
    expect(getContainerIconType("ufd")).toBe("ufed");
  });

  it("returns 'archive' for archive types", () => {
    expect(getContainerIconType("zip")).toBe("archive");
    expect(getContainerIconType("7z")).toBe("archive");
    expect(getContainerIconType("rar")).toBe("archive");
    expect(getContainerIconType("tar")).toBe("archive");
    expect(getContainerIconType("archive")).toBe("archive");
  });

  it("returns 'default' for unknown types", () => {
    expect(getContainerIconType("unknown")).toBe("default");
    expect(getContainerIconType("")).toBe("default");
  });

  it("returns valid ContainerIconType values", () => {
    const validTypes: ContainerIconType[] = ["ad1", "e01", "l01", "raw", "ufed", "archive", "default"];
    const testInputs = ["ad1", "e01", "l01", "raw", "ufed", "zip", "unknown"];
    for (const input of testInputs) {
      expect(validTypes).toContain(getContainerIconType(input));
    }
  });
});
