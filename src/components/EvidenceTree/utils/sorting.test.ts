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
} from "./sorting";
import type { TreeEntry, VfsEntry, ArchiveTreeEntry, UfedTreeEntry } from "../../../types";
import type { LazyTreeEntry } from "../../../types/lazy-loading";

// =============================================================================
// Helpers
// =============================================================================

function makeTreeEntry(overrides: Partial<TreeEntry> & { name: string; is_dir: boolean }): TreeEntry {
  const { name, is_dir, ...rest } = overrides;
  return {
    path: rest.path ?? name,
    size: 0,
    item_type: 0,
    ...rest,
    name,
    is_dir,
  };
}

function makeVfsEntry(overrides: Partial<VfsEntry> & { name: string; isDir: boolean }): VfsEntry {
  const { name, isDir, ...rest } = overrides;
  return {
    path: rest.path ?? name,
    size: 0,
    ...rest,
    name,
    isDir,
  };
}

function makeArchiveEntry(overrides: Partial<ArchiveTreeEntry> & { path: string; isDir: boolean }): ArchiveTreeEntry {
  const { path, isDir, ...rest } = overrides;
  return {
    name: rest.name ?? path.split("/").pop() ?? "",
    size: 0,
    compressedSize: 0,
    crc32: 0,
    modified: "",
    ...rest,
    path,
    isDir,
  };
}

function makeUfedEntry(overrides: Partial<UfedTreeEntry> & { name: string; isDir: boolean }): UfedTreeEntry {
  const { name, isDir, ...rest } = overrides;
  return {
    path: rest.path ?? name,
    size: 0,
    entryType: "file",
    ...rest,
    name,
    isDir,
  };
}

// =============================================================================
// sortByDirFirst (generic)
// =============================================================================

describe("sortByDirFirst", () => {
  it("puts directories before files", () => {
    const entries = [
      { name: "file.txt", isDir: false },
      { name: "folder", isDir: true },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted[0].name).toBe("folder");
    expect(sorted[1].name).toBe("file.txt");
  });

  it("sorts directories alphabetically among themselves", () => {
    const entries = [
      { name: "zebra", isDir: true },
      { name: "alpha", isDir: true },
      { name: "mango", isDir: true },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted.map((e) => e.name)).toEqual(["alpha", "mango", "zebra"]);
  });

  it("sorts files alphabetically among themselves", () => {
    const entries = [
      { name: "c.txt", isDir: false },
      { name: "a.txt", isDir: false },
      { name: "b.txt", isDir: false },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted.map((e) => e.name)).toEqual(["a.txt", "b.txt", "c.txt"]);
  });

  it("supports is_dir (snake_case) field", () => {
    const entries = [
      { name: "file.txt", is_dir: false },
      { name: "folder", is_dir: true },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted[0].name).toBe("folder");
  });

  it("falls back to path when name is absent", () => {
    const entries = [
      { path: "z/path", isDir: false },
      { path: "a/path", isDir: false },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted[0].path).toBe("a/path");
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

  it("handles empty array", () => {
    expect(sortByDirFirst([])).toEqual([]);
  });

  it("handles single entry", () => {
    const entries = [{ name: "only", isDir: false }];
    expect(sortByDirFirst(entries)).toEqual(entries);
  });

  it("handles mixed isDir/is_dir entries", () => {
    const entries = [
      { name: "file", is_dir: false },
      { name: "dir", isDir: true },
    ];
    const sorted = sortByDirFirst(entries);
    expect(sorted[0].name).toBe("dir");
  });
});

// =============================================================================
// sortTreeEntries (AD1)
// =============================================================================

describe("sortTreeEntries", () => {
  it("sorts AD1 entries dirs-first then alphabetically", () => {
    const entries: TreeEntry[] = [
      makeTreeEntry({ name: "photo.jpg", is_dir: false }),
      makeTreeEntry({ name: "Documents", is_dir: true }),
      makeTreeEntry({ name: "archive.zip", is_dir: false }),
      makeTreeEntry({ name: "AppData", is_dir: true }),
    ];
    const sorted = sortTreeEntries(entries);
    expect(sorted.map((e) => e.name)).toEqual([
      "AppData", "Documents", "archive.zip", "photo.jpg",
    ]);
  });

  it("handles empty array", () => {
    expect(sortTreeEntries([])).toEqual([]);
  });

  it("does not mutate original", () => {
    const entries = [
      makeTreeEntry({ name: "b", is_dir: false }),
      makeTreeEntry({ name: "a", is_dir: true }),
    ];
    const copy = [...entries];
    sortTreeEntries(entries);
    expect(entries[0].name).toBe(copy[0].name);
  });
});

// =============================================================================
// sortVfsEntries
// =============================================================================

describe("sortVfsEntries", () => {
  it("sorts VFS entries dirs-first then alphabetically by name", () => {
    const entries: VfsEntry[] = [
      makeVfsEntry({ name: "file.txt", isDir: false }),
      makeVfsEntry({ name: "subdir", isDir: true }),
      makeVfsEntry({ name: "another.doc", isDir: false }),
    ];
    const sorted = sortVfsEntries(entries);
    expect(sorted[0].name).toBe("subdir");
    expect(sorted[1].name).toBe("another.doc");
    expect(sorted[2].name).toBe("file.txt");
  });

  it("falls back to path for name comparison", () => {
    const entries: VfsEntry[] = [
      makeVfsEntry({ name: "", path: "z-path", isDir: false }),
      makeVfsEntry({ name: "", path: "a-path", isDir: false }),
    ];
    const sorted = sortVfsEntries(entries);
    expect(sorted[0].path).toBe("a-path");
  });

  it("does not mutate original", () => {
    const entries = [
      makeVfsEntry({ name: "b", isDir: false }),
      makeVfsEntry({ name: "a", isDir: false }),
    ];
    sortVfsEntries(entries);
    expect(entries[0].name).toBe("b");
  });
});

// =============================================================================
// sortArchiveEntries
// =============================================================================

describe("sortArchiveEntries", () => {
  it("sorts archive entries dirs-first then alphabetically by path", () => {
    const entries: ArchiveTreeEntry[] = [
      makeArchiveEntry({ path: "docs/readme.txt", isDir: false }),
      makeArchiveEntry({ path: "docs/images/", isDir: true }),
      makeArchiveEntry({ path: "docs/appendix.txt", isDir: false }),
    ];
    const sorted = sortArchiveEntries(entries);
    expect(sorted[0].path).toBe("docs/images/");
    expect(sorted[1].path).toBe("docs/appendix.txt");
    expect(sorted[2].path).toBe("docs/readme.txt");
  });

  it("does not mutate original", () => {
    const entries = [
      makeArchiveEntry({ path: "b.txt", isDir: false }),
      makeArchiveEntry({ path: "a.txt", isDir: false }),
    ];
    sortArchiveEntries(entries);
    expect(entries[0].path).toBe("b.txt");
  });
});

// =============================================================================
// sortUfedEntries
// =============================================================================

describe("sortUfedEntries", () => {
  it("sorts UFED entries dirs-first then alphabetically", () => {
    const entries: UfedTreeEntry[] = [
      makeUfedEntry({ name: "sms_log.db", isDir: false }),
      makeUfedEntry({ name: "Contacts", isDir: true }),
      makeUfedEntry({ name: "call_log.db", isDir: false }),
    ];
    const sorted = sortUfedEntries(entries);
    expect(sorted[0].name).toBe("Contacts");
    expect(sorted[1].name).toBe("call_log.db");
    expect(sorted[2].name).toBe("sms_log.db");
  });

  it("does not mutate original", () => {
    const entries = [
      makeUfedEntry({ name: "b", isDir: false }),
      makeUfedEntry({ name: "a", isDir: false }),
    ];
    sortUfedEntries(entries);
    expect(entries[0].name).toBe("b");
  });
});

// =============================================================================
// sortLazyEntries
// =============================================================================

describe("sortLazyEntries", () => {
  it("sorts lazy entries dirs-first then alphabetically", () => {
    const entries = [
      { id: "1", name: "file.txt", path: "file.txt", is_dir: false, size: 0, entry_type: "file", child_count: 0, children_loaded: false },
      { id: "2", name: "folder", path: "folder", is_dir: true, size: 0, entry_type: "folder", child_count: 0, children_loaded: false },
    ] as LazyTreeEntry[];
    const sorted = sortLazyEntries(entries);
    expect(sorted[0].name).toBe("folder");
    expect(sorted[1].name).toBe("file.txt");
  });
});
