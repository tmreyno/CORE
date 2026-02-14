// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  // Constants
  TREE_ROW_HEIGHT,
  TREE_ROW_HEIGHT_COMPACT,
  TREE_ROW_HEIGHT_COMFORTABLE,
  TREE_INDENT_SIZE,
  VIRTUAL_LIST_OVERSCAN,
  TREE_ROW_BASE_CLASSES,
  TREE_ROW_SELECTED_CLASSES,
  TREE_ROW_NORMAL_CLASSES,
  TREE_ROW_ACTIVE_CLASSES,
  TREE_ROW_DISABLED_CLASSES,
  // Container colors
  CONTAINER_COLOR_AD1,
  CONTAINER_COLOR_E01,
  CONTAINER_COLOR_L01,
  CONTAINER_COLOR_RAW,
  CONTAINER_COLOR_UFED,
  CONTAINER_COLOR_ARCHIVE,
  CONTAINER_COLOR_DEFAULT,
  // Icon colors
  ICON_COLOR_FOLDER,
  ICON_COLOR_FILE,
  ICON_COLOR_IMAGE,
  ICON_COLOR_DOCUMENT,
  ICON_COLOR_DATABASE,
  ICON_COLOR_ARCHIVE,
  ICON_COLOR_CODE,
  ICON_COLOR_EXECUTABLE,
  // Functions
  getContainerColors,
  getTreeIndent,
  getTreeRowClasses,
  getFileIconColor,
} from "../constants";

// =============================================================================
// Layout Constants
// =============================================================================
describe("layout constants", () => {
  it("has ascending row heights for density modes", () => {
    expect(TREE_ROW_HEIGHT_COMPACT).toBeLessThan(TREE_ROW_HEIGHT);
    expect(TREE_ROW_HEIGHT).toBeLessThan(TREE_ROW_HEIGHT_COMFORTABLE);
  });

  it("has positive indent size", () => {
    expect(TREE_INDENT_SIZE).toBeGreaterThan(0);
  });

  it("has positive virtual list overscan", () => {
    expect(VIRTUAL_LIST_OVERSCAN).toBeGreaterThan(0);
  });
});

// =============================================================================
// Container Color Objects
// =============================================================================
describe("container color objects", () => {
  it("all have icon, badge, and border properties", () => {
    const colors = [
      CONTAINER_COLOR_AD1,
      CONTAINER_COLOR_E01,
      CONTAINER_COLOR_L01,
      CONTAINER_COLOR_RAW,
      CONTAINER_COLOR_UFED,
      CONTAINER_COLOR_ARCHIVE,
      CONTAINER_COLOR_DEFAULT,
    ];
    for (const color of colors) {
      expect(color.icon).toBeTruthy();
      expect(color.badge).toBeTruthy();
      expect(color.border).toBeTruthy();
    }
  });
});

// =============================================================================
// getContainerColors
// =============================================================================
describe("getContainerColors", () => {
  it("returns AD1 colors for ad1 type", () => {
    expect(getContainerColors("ad1")).toBe(CONTAINER_COLOR_AD1);
    expect(getContainerColors("AD1")).toBe(CONTAINER_COLOR_AD1);
    expect(getContainerColors("file.ad1")).toBe(CONTAINER_COLOR_AD1);
  });

  it("returns E01 colors for e01/ewf/encase types", () => {
    expect(getContainerColors("e01")).toBe(CONTAINER_COLOR_E01);
    expect(getContainerColors("EWF")).toBe(CONTAINER_COLOR_E01);
    expect(getContainerColors("EnCase")).toBe(CONTAINER_COLOR_E01);
  });

  it("returns L01 colors for l01/lx01/lvf types", () => {
    expect(getContainerColors("l01")).toBe(CONTAINER_COLOR_L01);
    expect(getContainerColors("lx01")).toBe(CONTAINER_COLOR_L01);
    expect(getContainerColors("LVF")).toBe(CONTAINER_COLOR_L01);
  });

  it("returns RAW colors for raw/dd/img/001 types", () => {
    expect(getContainerColors("raw")).toBe(CONTAINER_COLOR_RAW);
    expect(getContainerColors("dd")).toBe(CONTAINER_COLOR_RAW);
    expect(getContainerColors("disk.img")).toBe(CONTAINER_COLOR_RAW);
    expect(getContainerColors("image.001")).toBe(CONTAINER_COLOR_RAW);
  });

  it("returns UFED colors for ufed/ufd types", () => {
    expect(getContainerColors("ufed")).toBe(CONTAINER_COLOR_UFED);
    expect(getContainerColors("UFED")).toBe(CONTAINER_COLOR_UFED);
    expect(getContainerColors("ufd")).toBe(CONTAINER_COLOR_UFED);
  });

  it("returns ARCHIVE colors for archive types", () => {
    expect(getContainerColors("zip")).toBe(CONTAINER_COLOR_ARCHIVE);
    expect(getContainerColors("7z")).toBe(CONTAINER_COLOR_ARCHIVE);
    expect(getContainerColors("rar")).toBe(CONTAINER_COLOR_ARCHIVE);
    expect(getContainerColors("tar")).toBe(CONTAINER_COLOR_ARCHIVE);
    expect(getContainerColors("archive")).toBe(CONTAINER_COLOR_ARCHIVE);
  });

  it("returns DEFAULT colors for unknown types", () => {
    expect(getContainerColors("unknown")).toBe(CONTAINER_COLOR_DEFAULT);
    expect(getContainerColors("")).toBe(CONTAINER_COLOR_DEFAULT);
    expect(getContainerColors("pdf")).toBe(CONTAINER_COLOR_DEFAULT);
  });
});

// =============================================================================
// getTreeIndent
// =============================================================================
describe("getTreeIndent", () => {
  it("returns pixel string for depth 0", () => {
    expect(getTreeIndent(0)).toBe(`${TREE_INDENT_SIZE}px`);
  });

  it("returns pixel string for depth 1", () => {
    expect(getTreeIndent(1)).toBe(`${2 * TREE_INDENT_SIZE}px`);
  });

  it("increases indent with depth", () => {
    const d0 = parseInt(getTreeIndent(0));
    const d1 = parseInt(getTreeIndent(1));
    const d5 = parseInt(getTreeIndent(5));
    expect(d1).toBeGreaterThan(d0);
    expect(d5).toBeGreaterThan(d1);
  });
});

// =============================================================================
// getTreeRowClasses
// =============================================================================
describe("getTreeRowClasses", () => {
  it("includes base classes for default state", () => {
    const classes = getTreeRowClasses({});
    expect(classes).toContain(TREE_ROW_BASE_CLASSES);
    expect(classes).toContain(TREE_ROW_NORMAL_CLASSES);
  });

  it("uses selected classes when isSelected", () => {
    const classes = getTreeRowClasses({ isSelected: true });
    expect(classes).toContain(TREE_ROW_SELECTED_CLASSES);
    expect(classes).not.toContain(TREE_ROW_NORMAL_CLASSES);
  });

  it("uses active classes when isActive", () => {
    const classes = getTreeRowClasses({ isActive: true });
    expect(classes).toContain(TREE_ROW_ACTIVE_CLASSES);
    expect(classes).not.toContain(TREE_ROW_NORMAL_CLASSES);
  });

  it("uses disabled classes when isDisabled", () => {
    const classes = getTreeRowClasses({ isDisabled: true });
    expect(classes).toContain(TREE_ROW_DISABLED_CLASSES);
    expect(classes).not.toContain(TREE_ROW_NORMAL_CLASSES);
  });

  it("disabled takes priority over selected", () => {
    const classes = getTreeRowClasses({ isDisabled: true, isSelected: true });
    expect(classes).toContain(TREE_ROW_DISABLED_CLASSES);
    expect(classes).not.toContain(TREE_ROW_SELECTED_CLASSES);
  });

  it("selected takes priority over active", () => {
    const classes = getTreeRowClasses({ isSelected: true, isActive: true });
    expect(classes).toContain(TREE_ROW_SELECTED_CLASSES);
    expect(classes).not.toContain(TREE_ROW_ACTIVE_CLASSES);
  });

  it("appends custom class", () => {
    const classes = getTreeRowClasses({ customClass: "my-extra" });
    expect(classes).toContain("my-extra");
  });
});

// =============================================================================
// getFileIconColor
// =============================================================================
describe("getFileIconColor", () => {
  it("returns image color for image extensions", () => {
    expect(getFileIconColor("photo.jpg")).toBe(ICON_COLOR_IMAGE);
    expect(getFileIconColor("photo.jpeg")).toBe(ICON_COLOR_IMAGE);
    expect(getFileIconColor("image.png")).toBe(ICON_COLOR_IMAGE);
    expect(getFileIconColor("photo.gif")).toBe(ICON_COLOR_IMAGE);
    expect(getFileIconColor("icon.webp")).toBe(ICON_COLOR_IMAGE);
    expect(getFileIconColor("vector.svg")).toBe(ICON_COLOR_IMAGE);
  });

  it("returns document color for document extensions", () => {
    expect(getFileIconColor("file.pdf")).toBe(ICON_COLOR_DOCUMENT);
    expect(getFileIconColor("file.docx")).toBe(ICON_COLOR_DOCUMENT);
    expect(getFileIconColor("readme.md")).toBe(ICON_COLOR_DOCUMENT);
    expect(getFileIconColor("notes.txt")).toBe(ICON_COLOR_DOCUMENT);
  });

  it("returns database color for database extensions", () => {
    expect(getFileIconColor("data.db")).toBe(ICON_COLOR_DATABASE);
    expect(getFileIconColor("data.sqlite")).toBe(ICON_COLOR_DATABASE);
    expect(getFileIconColor("data.sqlite3")).toBe(ICON_COLOR_DATABASE);
  });

  it("returns archive color for archive extensions", () => {
    expect(getFileIconColor("file.zip")).toBe(ICON_COLOR_ARCHIVE);
    expect(getFileIconColor("file.7z")).toBe(ICON_COLOR_ARCHIVE);
    expect(getFileIconColor("file.tar")).toBe(ICON_COLOR_ARCHIVE);
  });

  it("returns code color for code extensions", () => {
    expect(getFileIconColor("main.ts")).toBe(ICON_COLOR_CODE);
    expect(getFileIconColor("app.py")).toBe(ICON_COLOR_CODE);
    expect(getFileIconColor("lib.rs")).toBe(ICON_COLOR_CODE);
    expect(getFileIconColor("style.css")).toBe(ICON_COLOR_CODE);
    expect(getFileIconColor("page.html")).toBe(ICON_COLOR_CODE);
  });

  it("returns executable color for executable extensions", () => {
    expect(getFileIconColor("setup.exe")).toBe(ICON_COLOR_EXECUTABLE);
    expect(getFileIconColor("app.dmg")).toBe(ICON_COLOR_EXECUTABLE);
  });

  it("returns default file color for unknown extensions", () => {
    expect(getFileIconColor("file.xyz")).toBe(ICON_COLOR_FILE);
    expect(getFileIconColor("noext")).toBe(ICON_COLOR_FILE);
  });

  it("is case insensitive via getExtension", () => {
    // getExtension normalizes to lowercase
    expect(getFileIconColor("photo.JPG")).toBe(ICON_COLOR_IMAGE);
    expect(getFileIconColor("file.PDF")).toBe(ICON_COLOR_DOCUMENT);
  });
});
