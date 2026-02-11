// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  detectContainerType,
  isForensicContainer,
  detectContainerTypeString,
  getContainerDisplayName,
  CONTAINER_DISPLAY_NAMES,
} from "../containerUtils";

describe("containerUtils", () => {
  // ===========================================================================
  // detectContainerType
  // ===========================================================================
  describe("detectContainerType", () => {
    it("detects AD1 containers", () => {
      expect(detectContainerType("evidence.ad1")).toBe("Ad1");
      expect(detectContainerType("EVIDENCE.AD1")).toBe("Ad1");
    });

    it("detects EWF containers (E01/L01)", () => {
      expect(detectContainerType("evidence.e01")).toBe("Ewf");
      expect(detectContainerType("evidence.l01")).toBe("Ewf");
      expect(detectContainerType("evidence.ex01")).toBe("Ewf");
      expect(detectContainerType("evidence.lx01")).toBe("Ewf");
    });

    it("detects UFED containers", () => {
      expect(detectContainerType("phone.ufd")).toBe("Ufed");
      expect(detectContainerType("phone.ufdr")).toBe("Ufed");
      expect(detectContainerType("phone.ufdx")).toBe("Ufed");
    });

    it("detects archive containers", () => {
      expect(detectContainerType("backup.zip")).toBe("Zip");
      expect(detectContainerType("backup.7z")).toBe("SevenZip");
      expect(detectContainerType("backup.tar")).toBe("Tar");
      expect(detectContainerType("backup.gz")).toBe("Tar");
      expect(detectContainerType("backup.tgz")).toBe("Tar");
      expect(detectContainerType("backup.rar")).toBe("Rar");
    });

    it("detects raw image containers", () => {
      expect(detectContainerType("disk.dd")).toBe("Raw");
      expect(detectContainerType("disk.raw")).toBe("Raw");
      expect(detectContainerType("disk.img")).toBe("Raw");
      expect(detectContainerType("disk.001")).toBe("Raw");
    });

    it("returns null for unknown types", () => {
      expect(detectContainerType("document.pdf")).toBeNull();
      expect(detectContainerType("photo.jpg")).toBeNull();
      expect(detectContainerType("README")).toBeNull();
    });
  });

  // ===========================================================================
  // isForensicContainer
  // ===========================================================================
  describe("isForensicContainer", () => {
    it("returns true for known containers", () => {
      expect(isForensicContainer("evidence.ad1")).toBe(true);
      expect(isForensicContainer("evidence.e01")).toBe(true);
      expect(isForensicContainer("backup.zip")).toBe(true);
    });

    it("returns false for non-containers", () => {
      expect(isForensicContainer("document.pdf")).toBe(false);
      expect(isForensicContainer("photo.jpg")).toBe(false);
    });
  });

  // ===========================================================================
  // detectContainerTypeString
  // ===========================================================================
  describe("detectContainerTypeString", () => {
    it("detects E01 type", () => {
      expect(detectContainerTypeString("evidence.e01")).toBe("e01");
      expect(detectContainerTypeString("evidence.ex01")).toBe("e01");
    });

    it("detects L01 type", () => {
      expect(detectContainerTypeString("evidence.l01")).toBe("l01");
      expect(detectContainerTypeString("evidence.lx01")).toBe("l01");
    });

    it("detects AD1 type", () => {
      expect(detectContainerTypeString("evidence.ad1")).toBe("ad1");
    });

    it("detects UFED type", () => {
      expect(detectContainerTypeString("phone.ufd")).toBe("ufed");
      expect(detectContainerTypeString("phone.ufdr")).toBe("ufed");
      expect(detectContainerTypeString("phone.ufdx")).toBe("ufed");
    });

    it("detects raw type", () => {
      expect(detectContainerTypeString("disk.dd")).toBe("raw");
      expect(detectContainerTypeString("disk.raw")).toBe("raw");
      expect(detectContainerTypeString("disk.img")).toBe("raw");
    });

    it("returns 'unknown' for unrecognized files", () => {
      expect(detectContainerTypeString("document.pdf")).toBe("unknown");
      expect(detectContainerTypeString("photo.jpg")).toBe("unknown");
    });
  });

  // ===========================================================================
  // getContainerDisplayName
  // ===========================================================================
  describe("getContainerDisplayName", () => {
    it("returns display name for known types", () => {
      expect(getContainerDisplayName("ad1")).toBe("AD1");
      expect(getContainerDisplayName("e01")).toBe("E01");
      expect(getContainerDisplayName("ewf")).toBe("EWF");
      expect(getContainerDisplayName("zip")).toBe("ZIP");
      expect(getContainerDisplayName("7z")).toBe("7-Zip");
      expect(getContainerDisplayName("raw")).toBe("Raw Image");
    });

    it("is case-insensitive", () => {
      expect(getContainerDisplayName("AD1")).toBe("AD1");
      expect(getContainerDisplayName("E01")).toBe("E01");
    });

    it("returns uppercase for unknown types", () => {
      expect(getContainerDisplayName("custom")).toBe("CUSTOM");
      expect(getContainerDisplayName("xyz")).toBe("XYZ");
    });
  });

  // ===========================================================================
  // CONTAINER_DISPLAY_NAMES
  // ===========================================================================
  describe("CONTAINER_DISPLAY_NAMES", () => {
    it("has entries for common forensic formats", () => {
      expect(CONTAINER_DISPLAY_NAMES).toHaveProperty("ad1");
      expect(CONTAINER_DISPLAY_NAMES).toHaveProperty("e01");
      expect(CONTAINER_DISPLAY_NAMES).toHaveProperty("ewf");
      expect(CONTAINER_DISPLAY_NAMES).toHaveProperty("ufd");
    });

    it("has entries for archive formats", () => {
      expect(CONTAINER_DISPLAY_NAMES).toHaveProperty("zip");
      expect(CONTAINER_DISPLAY_NAMES).toHaveProperty("7z");
      expect(CONTAINER_DISPLAY_NAMES).toHaveProperty("tar");
      expect(CONTAINER_DISPLAY_NAMES).toHaveProperty("rar");
    });
  });
});
