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
} from "./pathUtils";

// =============================================================================
// getExtension
// =============================================================================

describe("getExtension", () => {
  it("extracts lowercase extension", () => {
    expect(getExtension("file.PDF")).toBe("pdf");
    expect(getExtension("document.TXT")).toBe("txt");
    expect(getExtension("photo.JPEG")).toBe("jpeg");
  });

  it("handles lowercase extensions", () => {
    expect(getExtension("file.pdf")).toBe("pdf");
    expect(getExtension("file.rs")).toBe("rs");
  });

  it("handles multiple dots (returns last)", () => {
    expect(getExtension("archive.tar.gz")).toBe("gz");
    expect(getExtension("file.backup.2024.zip")).toBe("zip");
  });

  it("returns empty for no extension", () => {
    expect(getExtension("README")).toBe("");
    expect(getExtension("Makefile")).toBe("");
  });

  it("returns empty for trailing dot", () => {
    expect(getExtension("file.")).toBe("");
  });

  it("handles hidden files", () => {
    // .gitignore: lastDot is 0, so returns ""
    expect(getExtension(".gitignore")).toBe("");
  });

  it("handles paths", () => {
    expect(getExtension("/path/to/file.txt")).toBe("txt");
    expect(getExtension("C:\\Users\\file.doc")).toBe("doc");
  });

  it("handles empty string", () => {
    expect(getExtension("")).toBe("");
  });
});

// =============================================================================
// hasExtension
// =============================================================================

describe("hasExtension", () => {
  it("returns true for matching extension", () => {
    expect(hasExtension("file.pdf", "pdf")).toBe(true);
    expect(hasExtension("file.PDF", "pdf")).toBe(true);
  });

  it("is case-insensitive for both filename and extension arg", () => {
    expect(hasExtension("file.PDF", "PDF")).toBe(true);
    expect(hasExtension("file.pdf", "PDF")).toBe(true);
  });

  it("returns false for non-matching extension", () => {
    expect(hasExtension("file.pdf", "doc")).toBe(false);
    expect(hasExtension("README", "txt")).toBe(false);
  });
});

// =============================================================================
// hasAnyExtension
// =============================================================================

describe("hasAnyExtension", () => {
  it("returns true if extension matches any in list", () => {
    expect(hasAnyExtension("file.pdf", ["pdf", "doc", "docx"])).toBe(true);
    expect(hasAnyExtension("file.doc", ["pdf", "doc", "docx"])).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(hasAnyExtension("file.PDF", ["pdf", "doc"])).toBe(true);
  });

  it("returns false if extension matches none", () => {
    expect(hasAnyExtension("file.jpg", ["pdf", "doc"])).toBe(false);
  });

  it("returns false for files without extension", () => {
    expect(hasAnyExtension("README", ["txt", "md"])).toBe(false);
  });

  it("handles empty extension list", () => {
    expect(hasAnyExtension("file.pdf", [])).toBe(false);
  });
});

// =============================================================================
// getBasename
// =============================================================================

describe("getBasename", () => {
  it("extracts filename from forward slash paths", () => {
    expect(getBasename("/path/to/file.txt")).toBe("file.txt");
    expect(getBasename("/file.txt")).toBe("file.txt");
  });

  it("extracts filename from backslash paths", () => {
    expect(getBasename("C:\\folder\\doc.pdf")).toBe("doc.pdf");
    expect(getBasename("\\\\server\\share\\file.txt")).toBe("file.txt");
  });

  it("returns full string for bare filename", () => {
    expect(getBasename("file.txt")).toBe("file.txt");
    expect(getBasename("README")).toBe("README");
  });

  it("returns empty string for trailing slash", () => {
    expect(getBasename("/path/to/")).toBe("");
  });

  it("returns empty string for empty input", () => {
    expect(getBasename("")).toBe("");
  });
});

// =============================================================================
// getBasenameWithoutExt
// =============================================================================

describe("getBasenameWithoutExt", () => {
  it("strips extension from filename", () => {
    expect(getBasenameWithoutExt("file.txt")).toBe("file");
    expect(getBasenameWithoutExt("/path/to/file.txt")).toBe("file");
  });

  it("handles multiple dots (strips only last)", () => {
    expect(getBasenameWithoutExt("archive.tar.gz")).toBe("archive.tar");
  });

  it("returns full name if no extension", () => {
    expect(getBasenameWithoutExt("README")).toBe("README");
    expect(getBasenameWithoutExt("Makefile")).toBe("Makefile");
  });

  it("returns empty for hidden files (dot at position 0)", () => {
    expect(getBasenameWithoutExt(".gitignore")).toBe("");
  });
});

// =============================================================================
// getDirname
// =============================================================================

describe("getDirname", () => {
  it("returns parent directory for forward slash paths", () => {
    expect(getDirname("/path/to/file.txt")).toBe("/path/to");
    expect(getDirname("/path/to/dir/")).toBe("/path/to");
  });

  it("returns parent for backslash paths", () => {
    expect(getDirname("C:\\Users\\file.txt")).toBe("C:\\Users");
  });

  it("returns empty string for bare filename", () => {
    expect(getDirname("file.txt")).toBe("");
  });

  it("returns empty string for root-level file", () => {
    expect(getDirname("/file.txt")).toBe("");
  });

  it("handles trailing slashes", () => {
    expect(getDirname("/path/to/dir/")).toBe("/path/to");
  });
});

// =============================================================================
// joinPath
// =============================================================================

describe("joinPath", () => {
  it("joins path components with forward slashes", () => {
    expect(joinPath("/path", "to", "file.txt")).toBe("/path/to/file.txt");
  });

  it("preserves leading slash from absolute paths", () => {
    expect(joinPath("/Users/terry/Desktop", "MyProject")).toBe("/Users/terry/Desktop/MyProject");
  });

  it("removes trailing slashes from components", () => {
    expect(joinPath("/path/", "to/", "file.txt")).toBe("/path/to/file.txt");
  });

  it("filters out empty components", () => {
    expect(joinPath("", "file.txt")).toBe("file.txt");
    expect(joinPath("/path", "", "file.txt")).toBe("/path/file.txt");
  });

  it("handles single component", () => {
    expect(joinPath("file.txt")).toBe("file.txt");
  });

  it("handles single absolute component", () => {
    expect(joinPath("/path")).toBe("/path");
  });

  it("handles relative paths without leading slash", () => {
    expect(joinPath("path", "to", "file.txt")).toBe("path/to/file.txt");
  });

  it("handles all empty", () => {
    expect(joinPath("", "", "")).toBe("");
  });
});

// =============================================================================
// normalizePath
// =============================================================================

describe("normalizePath", () => {
  it("converts backslashes to forward slashes", () => {
    expect(normalizePath("C:\\path\\to\\file.txt")).toBe("C:/path/to/file.txt");
  });

  it("removes redundant forward slashes", () => {
    expect(normalizePath("/path//to///file.txt")).toBe("/path/to/file.txt");
  });

  it("removes trailing slash (except root)", () => {
    expect(normalizePath("/path/to/")).toBe("/path/to");
    expect(normalizePath("/")).toBe("/");
  });

  it("preserves root slash", () => {
    expect(normalizePath("/")).toBe("/");
  });

  it("handles already-normalized paths", () => {
    expect(normalizePath("path/to/file.txt")).toBe("path/to/file.txt");
  });

  it("handles mixed slashes", () => {
    expect(normalizePath("path\\to/file.txt")).toBe("path/to/file.txt");
  });
});

// =============================================================================
// isAbsolutePath
// =============================================================================

describe("isAbsolutePath", () => {
  it("returns true for Unix absolute paths", () => {
    expect(isAbsolutePath("/path/to/file")).toBe(true);
    expect(isAbsolutePath("/")).toBe(true);
  });

  it("returns true for Windows drive letter paths", () => {
    expect(isAbsolutePath("C:\\path\\to\\file")).toBe(true);
    expect(isAbsolutePath("D:file")).toBe(true);
    expect(isAbsolutePath("c:/path")).toBe(true);
  });

  it("returns false for relative paths", () => {
    expect(isAbsolutePath("relative/path")).toBe(false);
    expect(isAbsolutePath("file.txt")).toBe(false);
    expect(isAbsolutePath("./file.txt")).toBe(false);
    expect(isAbsolutePath("../file.txt")).toBe(false);
  });

  it("returns false for empty string", () => {
    expect(isAbsolutePath("")).toBe(false);
  });
});

// =============================================================================
// isHiddenFile
// =============================================================================

describe("isHiddenFile", () => {
  it("returns true for dot-prefixed filenames", () => {
    expect(isHiddenFile(".gitignore")).toBe(true);
    expect(isHiddenFile(".config")).toBe(true);
    expect(isHiddenFile(".env")).toBe(true);
  });

  it("returns true for dot-prefixed basename in full path", () => {
    expect(isHiddenFile("/path/.config")).toBe(true);
    expect(isHiddenFile("C:\\Users\\.env")).toBe(true);
  });

  it("returns false for normal files", () => {
    expect(isHiddenFile("file.txt")).toBe(false);
    expect(isHiddenFile("/path/to/file.txt")).toBe(false);
  });

  it("does not flag intermediate dot-dirs as hidden files", () => {
    // Only the basename matters
    expect(isHiddenFile("/path/.hidden/file.txt")).toBe(false);
  });

  it("returns false for empty string", () => {
    expect(isHiddenFile("")).toBe(false);
  });
});
