// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  sortByDirFirst,
  sortTreeEntries,
  sortVfsEntries,
  sortArchiveEntries,
  sortUfedEntries,
  sortLazyEntries,
} from "../sorting";
import type { TreeEntry, VfsEntry, ArchiveTreeEntry, UfedTreeEntry } from "../../../../types";
import type { LazyTreeEntry } from "../../../../types/lazy-loading";

// =============================================================================
// Test Helpers
// =============================================================================

function makeTreeEntry(overrides: Partial<TreeEntry> & { name: string; is_dir: boolean }): TreeEntry {
  return {
    path: `/${overrides.name}`,
    size: 0,
    item_type: overrides.is_dir ? 0x05 : 0x00,
    ...overrides,
  };
}

function makeVfsEntry(overrides: Partial<VfsEntry> & { name: string; isDir: boolean }): VfsEntry {
  return {
    path: `/${overrides.name}`,
    size: 0,
    ...overrides,
  };
}

function makeArchiveEntry(overrides: Partial<ArchiveTreeEntry> & { name: string; isDir: boolean }): ArchiveTreeEntry {
  return {
    path: overrides.name,
    size: 0,
    compressedSize: 0,
    crc32: 0,
    modified: "",
    ...overrides,
  };
}

function makeUfedEntry(overrides: Partial<UfedTreeEntry> & { name: string; isDir: boolean }): UfedTreeEntry {
  return {
    path: `/${overrides.name}`,
    size: 0,
    entryType: overrides.isDir ? "folder" : "file",
    ...overrides,
  };
}

function makeLazyEntry(overrides: Partial<LazyTreeEntry> & { name: string; is_dir: boolean }): LazyTreeEntry {
  return {
    id: `lazy-${overrides.name}`,
    path: `/${overrides.name}`,
    size: 0,
    entry_type: overrides.is_dir ? "folder" : "file",
    child_count: 0,
    children_loaded: false,
    hash: null,
    modified: null,
    metadata: null,
    ...overrides,
  };
}

// =============================================================================
// sortByDirFirst (generic)
// =============================================================================
describe("sortByDirFirst", () => {
  it("returns empty array for empty input", () => {
    expect(sortByDirFirst([])).toEqual([]);
  });

  it("places directories before files", () => {
    const entries = [
      { name: "file.txt", isDir: false },
      { name: "docs", isDir: true },
      { name: "readme.md", isDir: false },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted[0].name).toBe("docs");
    expect(sorted[1].isDir).toBe(false);
    expect(sorted[2].isDir).toBe(false);
  });

  it("sorts directories alphabetically among themselves", () => {
    const entries = [
      { name: "zebra", isDir: true },
      { name: "alpha", isDir: true },
      { name: "middle", isDir: true },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted.map(e => e.name)).toEqual(["alpha", "middle", "zebra"]);
  });

  it("sorts files alphabetically among themselves", () => {
    const entries = [
      { name: "c.txt", isDir: false },
      { name: "a.txt", isDir: false },
      { name: "b.txt", isDir: false },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted.map(e => e.name)).toEqual(["a.txt", "b.txt", "c.txt"]);
  });

  it("supports is_dir (snake_case) field", () => {
    const entries = [
      { name: "file.txt", is_dir: false },
      { name: "folder", is_dir: true },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted[0].name).toBe("folder");
    expect(sorted[1].name).toBe("file.txt");
  });

  it("falls back to path when name is not available", () => {
    const entries = [
      { path: "z/file", isDir: false },
      { path: "a/file", isDir: false },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted[0].path).toBe("a/file");
    expect(sorted[1].path).toBe("z/file");
  });

  it("does not mutate original array", () => {
    const entries = [
      { name: "b", isDir: false },
      { name: "a", isDir: true },
    ];
    const original = [...entries];
    sortByDirFirst(entries);
    expect(entries).toEqual(original);
  });

  it("handles single element", () => {
    const entries = [{ name: "only", isDir: false }];
    const sorted = sortByDirFirst(entries);
    expect(sorted).toEqual([{ name: "only", isDir: false }]);
  });
});

// =============================================================================
// sortTreeEntries (AD1)
// =============================================================================
describe("sortTreeEntries", () => {
  it("sorts AD1 entries: directories first, then alphabetical", () => {
    const entries = [
      makeTreeEntry({ name: "readme.txt", is_dir: false }),
      makeTreeEntry({ name: "Documents", is_dir: true }),
      makeTreeEntry({ name: "archive.zip", is_dir: false }),
      makeTreeEntry({ name: "AppData", is_dir: true }),
    ];
    const sorted = sortTreeEntries(entries);
    expect(sorted.map(e => e.name)).toEqual([
      "AppData", "Documents", "archive.zip", "readme.txt",
    ]);
  });

  it("handles all directories", () => {
    const entries = [
      makeTreeEntry({ name: "B_Dir", is_dir: true }),
      makeTreeEntry({ name: "A_Dir", is_dir: true }),
    ];
    const sorted = sortTreeEntries(entries);
    expect(sorted.map(e => e.name)).toEqual(["A_Dir", "B_Dir"]);
  });

  it("handles all files", () => {
    const entries = [
      makeTreeEntry({ name: "z.doc", is_dir: false }),
      makeTreeEntry({ name: "a.doc", is_dir: false }),
    ];
    const sorted = sortTreeEntries(entries);
    expect(sorted.map(e => e.name)).toEqual(["a.doc", "z.doc"]);
  });
});

// =============================================================================
// sortVfsEntries
// =============================================================================
describe("sortVfsEntries", () => {
  it("sorts VFS entries: directories first, then alphabetical", () => {
    const entries = [
      makeVfsEntry({ name: "file.log", isDir: false }),
      makeVfsEntry({ name: "var", isDir: true }),
      makeVfsEntry({ name: "etc", isDir: true }),
      makeVfsEntry({ name: "app.conf", isDir: false }),
    ];
    const sorted = sortVfsEntries(entries);
    expect(sorted.map(e => e.name)).toEqual([
      "etc", "var", "app.conf", "file.log",
    ]);
  });

  it("handles empty array", () => {
    expect(sortVfsEntries([])).toEqual([]);
  });
});

// =============================================================================
// sortArchiveEntries
// =============================================================================
describe("sortArchiveEntries", () => {
  it("sorts archive entries by path, directories first", () => {
    const entries = [
      makeArchiveEntry({ name: "readme.txt", path: "readme.txt", isDir: false }),
      makeArchiveEntry({ name: "src", path: "src", isDir: true }),
      makeArchiveEntry({ name: "docs", path: "docs", isDir: true }),
      makeArchiveEntry({ name: "license", path: "license", isDir: false }),
    ];
    const sorted = sortArchiveEntries(entries);
    expect(sorted.map(e => e.path)).toEqual([
      "docs", "src", "license", "readme.txt",
    ]);
  });

  it("does not mutate original array", () => {
    const entries = [
      makeArchiveEntry({ name: "b", path: "b", isDir: false }),
      makeArchiveEntry({ name: "a", path: "a", isDir: true }),
    ];
    const original = [...entries];
    sortArchiveEntries(entries);
    expect(entries).toEqual(original);
  });
});

// =============================================================================
// sortUfedEntries
// =============================================================================
describe("sortUfedEntries", () => {
  it("sorts UFED entries: directories first, then alphabetical by name", () => {
    const entries = [
      makeUfedEntry({ name: "sms.db", isDir: false }),
      makeUfedEntry({ name: "Media", isDir: true }),
      makeUfedEntry({ name: "Contacts", isDir: true }),
      makeUfedEntry({ name: "call_log.db", isDir: false }),
    ];
    const sorted = sortUfedEntries(entries);
    expect(sorted.map(e => e.name)).toEqual([
      "Contacts", "Media", "call_log.db", "sms.db",
    ]);
  });
});

// =============================================================================
// sortLazyEntries
// =============================================================================
describe("sortLazyEntries", () => {
  it("sorts lazy entries: directories first, then alphabetical", () => {
    const entries = [
      makeLazyEntry({ name: "data.bin", is_dir: false }),
      makeLazyEntry({ name: "System", is_dir: true }),
      makeLazyEntry({ name: "Users", is_dir: true }),
      makeLazyEntry({ name: "boot.ini", is_dir: false }),
    ];
    const sorted = sortLazyEntries(entries);
    expect(sorted.map(e => e.name)).toEqual([
      "System", "Users", "boot.ini", "data.bin",
    ]);
  });

  it("handles empty array", () => {
    expect(sortLazyEntries([])).toEqual([]);
  });
});
