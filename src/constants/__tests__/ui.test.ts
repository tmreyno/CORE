// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  getStatusTextColor,
  getContainerType,
  getContainerTextColor,
  getContainerBadgeClass,
  getTransferPhaseColor,
  getFileCategory,
  BAR_HEIGHT_SMALL,
  BAR_HEIGHT_BASE,
  BAR_HEIGHT_LG,
  ICON_SIZE_MICRO,
  ICON_SIZE_SMALL,
  ICON_SIZE_BASE,
  ICON_SIZE_LG,
  Z_INDEX,
  TREE_DENSITY_PRESETS,
} from "../ui";

// =============================================================================
// Constants sanity checks
// =============================================================================

describe("numeric constants", () => {
  it("bar heights are ordered small < base < large", () => {
    expect(BAR_HEIGHT_SMALL).toBeLessThan(BAR_HEIGHT_BASE);
    expect(BAR_HEIGHT_BASE).toBeLessThan(BAR_HEIGHT_LG);
  });

  it("icon sizes are ordered micro < small < base < lg", () => {
    expect(ICON_SIZE_MICRO).toBeLessThan(ICON_SIZE_SMALL);
    expect(ICON_SIZE_SMALL).toBeLessThan(ICON_SIZE_BASE);
    expect(ICON_SIZE_BASE).toBeLessThan(ICON_SIZE_LG);
  });

  it("z-index values are strictly increasing", () => {
    const values = Object.values(Z_INDEX);
    for (let i = 1; i < values.length; i++) {
      expect(values[i]).toBeGreaterThan(values[i - 1]);
    }
  });
});

describe("tree density presets", () => {
  it("compact has smallest values", () => {
    expect(TREE_DENSITY_PRESETS.compact.iconSize).toBeLessThan(TREE_DENSITY_PRESETS.default.iconSize);
    expect(TREE_DENSITY_PRESETS.compact.gap).toBeLessThan(TREE_DENSITY_PRESETS.default.gap);
  });

  it("comfortable has largest values", () => {
    expect(TREE_DENSITY_PRESETS.comfortable.iconSize).toBeGreaterThan(TREE_DENSITY_PRESETS.default.iconSize);
    expect(TREE_DENSITY_PRESETS.comfortable.gap).toBeGreaterThan(TREE_DENSITY_PRESETS.default.gap);
  });
});

// =============================================================================
// getStatusTextColor
// =============================================================================

describe("getStatusTextColor", () => {
  it("returns correct class for success", () => {
    expect(getStatusTextColor("success")).toBe("text-green-400");
  });

  it("returns correct class for error", () => {
    expect(getStatusTextColor("error")).toBe("text-red-400");
  });

  it("returns correct class for warning", () => {
    expect(getStatusTextColor("warning")).toBe("text-yellow-400");
  });

  it("returns correct class for info", () => {
    expect(getStatusTextColor("info")).toBe("text-blue-400");
  });

  it("returns correct class for accent", () => {
    expect(getStatusTextColor("accent")).toBe("text-accent");
  });

  it("returns correct class for muted", () => {
    expect(getStatusTextColor("muted")).toBe("text-txt-muted");
  });

  it("returns correct class for pending", () => {
    expect(getStatusTextColor("pending")).toBe("text-txt-muted");
  });

  it("returns correct class for active", () => {
    expect(getStatusTextColor("active")).toBe("text-accent");
  });

  it("returns correct class for verifying", () => {
    expect(getStatusTextColor("verifying")).toBe("text-purple-400");
  });

  it("returns correct class for cancelled", () => {
    expect(getStatusTextColor("cancelled")).toBe("text-amber-400");
  });
});

// =============================================================================
// getContainerType
// =============================================================================

describe("getContainerType", () => {
  it("detects AD1 containers", () => {
    expect(getContainerType("ad1")).toBe("ad1");
    expect(getContainerType("AD1")).toBe("ad1");
    expect(getContainerType("file.ad1")).toBe("ad1");
  });

  it("detects E01/EWF containers", () => {
    expect(getContainerType("e01")).toBe("e01");
    expect(getContainerType("E01")).toBe("e01");
    expect(getContainerType("ewf")).toBe("e01");
    expect(getContainerType("encase")).toBe("e01");
  });

  it("detects L01 containers", () => {
    expect(getContainerType("l01")).toBe("l01");
    expect(getContainerType("L01")).toBe("l01");
    expect(getContainerType("lx01")).toBe("l01");
    expect(getContainerType("lvf")).toBe("l01");
  });

  it("detects raw/dd containers", () => {
    expect(getContainerType("raw")).toBe("raw");
    expect(getContainerType("dd")).toBe("raw");
    expect(getContainerType("img")).toBe("raw");
    expect(getContainerType("001")).toBe("raw");
  });

  it("detects UFED containers", () => {
    expect(getContainerType("ufed")).toBe("ufed");
    expect(getContainerType("UFED")).toBe("ufed");
    expect(getContainerType("ufd")).toBe("ufed");
  });

  it("detects archive containers", () => {
    expect(getContainerType("zip")).toBe("archive");
    expect(getContainerType("7z")).toBe("archive");
    expect(getContainerType("rar")).toBe("archive");
    expect(getContainerType("tar")).toBe("archive");
    expect(getContainerType("archive")).toBe("archive");
  });

  it("returns default for unknown types", () => {
    expect(getContainerType("unknown")).toBe("default");
    expect(getContainerType("")).toBe("default");
    expect(getContainerType("txt")).toBe("default");
  });
});

// =============================================================================
// getContainerTextColor
// =============================================================================

describe("getContainerTextColor", () => {
  it("returns correct class for AD1", () => {
    expect(getContainerTextColor("ad1")).toBe("text-type-ad1");
  });

  it("returns correct class for E01", () => {
    expect(getContainerTextColor("e01")).toBe("text-type-e01");
  });

  it("returns correct class for unknown type", () => {
    expect(getContainerTextColor("unknown")).toBe("text-txt-secondary");
  });
});

// =============================================================================
// getContainerBadgeClass
// =============================================================================

describe("getContainerBadgeClass", () => {
  it("returns badge class for AD1", () => {
    expect(getContainerBadgeClass("ad1")).toBe("badge-ad1");
  });

  it("returns badge class for E01", () => {
    expect(getContainerBadgeClass("e01")).toBe("badge-e01");
  });

  it("returns badge class for UFED", () => {
    expect(getContainerBadgeClass("ufed")).toBe("badge-ufed");
  });

  it("returns muted badge for unknown", () => {
    expect(getContainerBadgeClass("unknown")).toBe("badge-muted");
  });
});

// =============================================================================
// getTransferPhaseColor
// =============================================================================

describe("getTransferPhaseColor", () => {
  it("returns blue for scanning", () => {
    expect(getTransferPhaseColor("scanning")).toBe("text-blue-400");
  });

  it("returns accent for copying", () => {
    expect(getTransferPhaseColor("copying")).toBe("text-accent");
  });

  it("returns purple for verifying", () => {
    expect(getTransferPhaseColor("verifying")).toBe("text-purple-400");
  });

  it("returns green for completed", () => {
    expect(getTransferPhaseColor("completed")).toBe("text-green-400");
  });

  it("returns red for failed", () => {
    expect(getTransferPhaseColor("failed")).toBe("text-red-400");
  });

  it("returns amber for cancelled", () => {
    expect(getTransferPhaseColor("cancelled")).toBe("text-amber-400");
  });

  it("returns muted for pending", () => {
    expect(getTransferPhaseColor("pending")).toBe("text-txt-muted");
  });
});

// =============================================================================
// getFileCategory
// =============================================================================

describe("getFileCategory", () => {
  describe("image files", () => {
    it.each([
      "photo.jpg", "image.jpeg", "icon.png", "logo.gif",
      "scan.bmp", "photo.webp", "graphic.svg", "icon.ico",
      "scan.tiff", "photo.TIF",
    ])("classifies %s as image", (filename) => {
      expect(getFileCategory(filename)).toBe("image");
    });
  });

  describe("video files", () => {
    it.each([
      "video.mp4", "clip.avi", "movie.mov", "film.mkv",
      "stream.webm", "media.wmv", "video.flv", "clip.m4v",
    ])("classifies %s as video", (filename) => {
      expect(getFileCategory(filename)).toBe("video");
    });
  });

  describe("audio files", () => {
    it.each([
      "song.mp3", "audio.wav", "music.flac", "track.aac",
      "podcast.ogg", "song.m4a", "audio.wma",
    ])("classifies %s as audio", (filename) => {
      expect(getFileCategory(filename)).toBe("audio");
    });
  });

  describe("document files", () => {
    it.each([
      "report.pdf", "letter.doc", "resume.docx",
      "data.xls", "budget.xlsx", "slides.ppt", "deck.pptx",
      "document.odt", "spreadsheet.ods", "presentation.odp",
    ])("classifies %s as document", (filename) => {
      expect(getFileCategory(filename)).toBe("document");
    });
  });

  describe("archive files", () => {
    it.each([
      "backup.zip", "archive.7z", "files.rar",
      "bundle.tar", "compressed.gz", "data.bz2", "file.xz",
    ])("classifies %s as archive", (filename) => {
      expect(getFileCategory(filename)).toBe("archive");
    });
  });

  describe("code files", () => {
    it.each([
      "app.js", "server.ts", "component.jsx", "page.tsx",
      "script.py", "Main.java", "module.c", "lib.cpp",
      "header.h", "Program.cs", "main.go", "lib.rs",
      "script.rb", "index.php",
    ])("classifies %s as code", (filename) => {
      expect(getFileCategory(filename)).toBe("code");
    });
  });

  describe("text files", () => {
    it.each([
      "readme.txt", "notes.md", "config.json", "data.xml",
      "data.csv", "output.log", "settings.ini", "app.cfg",
      "config.yaml", "docker.yml",
    ])("classifies %s as text", (filename) => {
      expect(getFileCategory(filename)).toBe("text");
    });
  });

  describe("binary fallback", () => {
    it("classifies unknown extensions as binary", () => {
      expect(getFileCategory("data.bin")).toBe("binary");
      expect(getFileCategory("file.dat")).toBe("binary");
      expect(getFileCategory("evidence.e01")).toBe("binary");
    });

    it("classifies extensionless files as binary", () => {
      expect(getFileCategory("Makefile")).toBe("binary");
    });
  });
});
