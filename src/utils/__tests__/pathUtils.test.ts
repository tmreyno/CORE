// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  getExtension,
  hasExtension,
  hasAnyExtension,
  getBasename,
  getBasenameWithoutExt,
  getDirname,
  joinPath,
  normalizePath,
  isAbsolutePath,
  isHiddenFile,
} from "../pathUtils";

describe("pathUtils", () => {
  // ===========================================================================
  // getExtension
  // ===========================================================================
  describe("getExtension", () => {
    it("extracts lowercase extension", () => {
      expect(getExtension("file.PDF")).toBe("pdf");
      expect(getExtension("document.TXT")).toBe("txt");
      expect(getExtension("photo.JPEG")).toBe("jpeg");
    });

    it("handles multiple dots", () => {
      expect(getExtension("archive.tar.gz")).toBe("gz");
      expect(getExtension("my.file.name.txt")).toBe("txt");
    });

    it("returns empty for no extension", () => {
      expect(getExtension("README")).toBe("");
      expect(getExtension("Makefile")).toBe("");
    });

    it("returns empty for empty string", () => {
      expect(getExtension("")).toBe("");
    });

    it("returns empty for hidden files (dot-only prefix, no real extension)", () => {
      // getExtension treats ".gitignore" as having dot at index 0 → no extension
      expect(getExtension(".gitignore")).toBe("");
      expect(getExtension(".bashrc")).toBe("");
    });

    it("handles trailing dot", () => {
      expect(getExtension("file.")).toBe("");
    });

    it("handles paths with directories", () => {
      expect(getExtension("/path/to/file.txt")).toBe("txt");
      expect(getExtension("C:\\Users\\doc.pdf")).toBe("pdf");
    });
  });

  // ===========================================================================
  // hasExtension
  // ===========================================================================
  describe("hasExtension", () => {
    it("matches case-insensitively", () => {
      expect(hasExtension("file.PDF", "pdf")).toBe(true);
      expect(hasExtension("file.pdf", "PDF")).toBe(true);
    });

    it("returns false for non-matching", () => {
      expect(hasExtension("file.txt", "pdf")).toBe(false);
    });

    it("returns false for no extension", () => {
      expect(hasExtension("README", "txt")).toBe(false);
    });
  });

  // ===========================================================================
  // hasAnyExtension
  // ===========================================================================
  describe("hasAnyExtension", () => {
    it("matches any in the list", () => {
      expect(hasAnyExtension("photo.png", ["jpg", "png", "gif"])).toBe(true);
      expect(hasAnyExtension("photo.JPG", ["jpg", "png", "gif"])).toBe(true);
    });

    it("returns false if none match", () => {
      expect(hasAnyExtension("doc.pdf", ["jpg", "png", "gif"])).toBe(false);
    });

    it("handles empty extensions list", () => {
      expect(hasAnyExtension("file.txt", [])).toBe(false);
    });
  });

  // ===========================================================================
  // getBasename
  // ===========================================================================
  describe("getBasename", () => {
    it("extracts filename from Unix paths", () => {
      expect(getBasename("/path/to/file.txt")).toBe("file.txt");
      expect(getBasename("/Users/test/doc.pdf")).toBe("doc.pdf");
    });

    it("extracts filename from Windows paths", () => {
      expect(getBasename("C:\\Users\\doc.pdf")).toBe("doc.pdf");
      expect(getBasename("D:\\folder\\file.txt")).toBe("file.txt");
    });

    it("handles just a filename", () => {
      expect(getBasename("file.txt")).toBe("file.txt");
    });

    it("returns empty for trailing slash", () => {
      expect(getBasename("/path/to/")).toBe("");
    });

    it("returns empty for empty string", () => {
      expect(getBasename("")).toBe("");
    });
  });

  // ===========================================================================
  // getBasenameWithoutExt
  // ===========================================================================
  describe("getBasenameWithoutExt", () => {
    it("removes extension", () => {
      expect(getBasenameWithoutExt("/path/to/file.txt")).toBe("file");
      expect(getBasenameWithoutExt("archive.tar.gz")).toBe("archive.tar");
    });

    it("returns full name if no extension", () => {
      expect(getBasenameWithoutExt("README")).toBe("README");
    });

    it("handles hidden files", () => {
      expect(getBasenameWithoutExt(".gitignore")).toBe("");
    });
  });

  // ===========================================================================
  // getDirname
  // ===========================================================================
  describe("getDirname", () => {
    it("extracts parent directory", () => {
      expect(getDirname("/path/to/file.txt")).toBe("/path/to");
    });

    it("handles trailing slashes (strips trailing then gets parent)", () => {
      // "/path/to/" → trimmed to "/path/to" → dirname is "/path"
      expect(getDirname("/path/to/")).toBe("/path");
    });

    it("returns empty for just a filename", () => {
      expect(getDirname("file.txt")).toBe("");
    });

    it("returns empty for root-level file", () => {
      expect(getDirname("/file.txt")).toBe("");
    });
  });

  // ===========================================================================
  // joinPath
  // ===========================================================================
  describe("joinPath", () => {
    it("joins multiple components", () => {
      expect(joinPath("/path", "to", "file.txt")).toBe("path/to/file.txt");
    });

    it("strips trailing slashes from components", () => {
      expect(joinPath("/path/", "to/", "file.txt")).toBe("path/to/file.txt");
    });

    it("skips empty components", () => {
      expect(joinPath("", "file.txt")).toBe("file.txt");
      expect(joinPath("/path", "", "file.txt")).toBe("path/file.txt");
    });

    it("handles single component", () => {
      expect(joinPath("file.txt")).toBe("file.txt");
    });
  });

  // ===========================================================================
  // normalizePath
  // ===========================================================================
  describe("normalizePath", () => {
    it("converts backslashes to forward slashes", () => {
      expect(normalizePath("C:\\path\\to\\file.txt")).toBe("C:/path/to/file.txt");
    });

    it("removes redundant slashes", () => {
      expect(normalizePath("/path//to///file.txt")).toBe("/path/to/file.txt");
    });

    it("removes trailing slash (except root)", () => {
      expect(normalizePath("/path/to/")).toBe("/path/to");
      expect(normalizePath("/")).toBe("/");
    });

    it("handles already-normalized paths", () => {
      expect(normalizePath("path/to/file.txt")).toBe("path/to/file.txt");
    });
  });

  // ===========================================================================
  // isAbsolutePath
  // ===========================================================================
  describe("isAbsolutePath", () => {
    it("detects Unix absolute paths", () => {
      expect(isAbsolutePath("/path/to/file")).toBe(true);
      expect(isAbsolutePath("/")).toBe(true);
    });

    it("detects Windows absolute paths", () => {
      expect(isAbsolutePath("C:\\path")).toBe(true);
      expect(isAbsolutePath("D:/path")).toBe(true);
    });

    it("returns false for relative paths", () => {
      expect(isAbsolutePath("relative/path")).toBe(false);
      expect(isAbsolutePath("file.txt")).toBe(false);
      expect(isAbsolutePath("")).toBe(false);
    });
  });

  // ===========================================================================
  // isHiddenFile
  // ===========================================================================
  describe("isHiddenFile", () => {
    it("detects dot-prefixed filenames", () => {
      expect(isHiddenFile(".gitignore")).toBe(true);
      expect(isHiddenFile("/path/.config")).toBe(true);
    });

    it("returns false for normal files", () => {
      expect(isHiddenFile("/path/to/file.txt")).toBe(false);
      expect(isHiddenFile("file.txt")).toBe(false);
    });

    it("only checks basename, not directories", () => {
      expect(isHiddenFile("/.hidden/visible.txt")).toBe(false);
    });
  });
});
