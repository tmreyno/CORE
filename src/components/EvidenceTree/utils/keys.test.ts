// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  getAd1EntryKey,
  getAd1NodeKey,
  getVfsEntryKey,
  getArchiveEntryKey,
  getLazyEntryKey,
  getUfedEntryKey,
  getEntryKey,
  type EntryKeyType,
} from "./keys";
import type { TreeEntry, VfsEntry, ArchiveTreeEntry, UfedTreeEntry } from "../../../types";
import type { LazyTreeEntry } from "../../../types/lazy-loading";

// =============================================================================
// getAd1EntryKey
// =============================================================================

describe("getAd1EntryKey", () => {
  it("uses item_addr when available", () => {
    const entry = { path: "dir/file.txt", name: "file.txt", is_dir: false, size: 100, item_type: 0, item_addr: 12345 } as TreeEntry;
    expect(getAd1EntryKey("/evidence/test.ad1", entry)).toBe("/evidence/test.ad1::12345");
  });

  it("falls back to path when item_addr is null", () => {
    const entry = { path: "dir/file.txt", name: "file.txt", is_dir: false, size: 100, item_type: 0, item_addr: null } as TreeEntry;
    expect(getAd1EntryKey("/evidence/test.ad1", entry)).toBe("/evidence/test.ad1::dir/file.txt");
  });

  it("falls back to path when item_addr is undefined", () => {
    const entry = { path: "root/doc", name: "doc", is_dir: true, size: 0, item_type: 0 } as TreeEntry;
    expect(getAd1EntryKey("/test.ad1", entry)).toBe("/test.ad1::root/doc");
  });
});

// =============================================================================
// getAd1NodeKey
// =============================================================================

describe("getAd1NodeKey", () => {
  it("uses first_child_addr with addr prefix when available", () => {
    const entry = { path: "Documents", name: "Documents", is_dir: true, size: 0, item_type: 0, first_child_addr: 9999 } as TreeEntry;
    expect(getAd1NodeKey("/test.ad1", entry)).toBe("/test.ad1::addr:9999");
  });

  it("uses path prefix when first_child_addr is null", () => {
    const entry = { path: "Documents", name: "Documents", is_dir: true, size: 0, item_type: 0, first_child_addr: null } as TreeEntry;
    expect(getAd1NodeKey("/test.ad1", entry)).toBe("/test.ad1::path:Documents");
  });

  it("uses path prefix when first_child_addr is undefined", () => {
    const entry = { path: "dir", name: "dir", is_dir: true, size: 0, item_type: 0 } as TreeEntry;
    expect(getAd1NodeKey("/test.ad1", entry)).toBe("/test.ad1::path:dir");
  });

  it("uses path prefix when first_child_addr is 0 (falsy)", () => {
    const entry = { path: "root", name: "root", is_dir: true, size: 0, item_type: 0, first_child_addr: 0 } as TreeEntry;
    // 0 is falsy, so this falls back to path
    expect(getAd1NodeKey("/test.ad1", entry)).toBe("/test.ad1::path:root");
  });
});

// =============================================================================
// getVfsEntryKey
// =============================================================================

describe("getVfsEntryKey", () => {
  it("generates vfs-prefixed key", () => {
    const entry = { path: "Partition1/NTFS/file.txt", name: "file.txt", isDir: false, size: 100 } as VfsEntry;
    expect(getVfsEntryKey("/disk.e01", entry)).toBe("/disk.e01::vfs::Partition1/NTFS/file.txt");
  });

  it("handles root path", () => {
    const entry = { path: "/", name: "/", isDir: true, size: 0 } as VfsEntry;
    expect(getVfsEntryKey("/disk.e01", entry)).toBe("/disk.e01::vfs::/");
  });
});

// =============================================================================
// getArchiveEntryKey
// =============================================================================

describe("getArchiveEntryKey", () => {
  it("generates archive-prefixed key", () => {
    const entry = { path: "docs/readme.txt", name: "readme.txt", isDir: false, size: 50, compressedSize: 20, crc32: 0, modified: "" } as ArchiveTreeEntry;
    expect(getArchiveEntryKey("/backup.zip", entry)).toBe("/backup.zip::archive::docs/readme.txt");
  });
});

// =============================================================================
// getLazyEntryKey
// =============================================================================

describe("getLazyEntryKey", () => {
  it("generates lazy-prefixed key", () => {
    const entry = { id: "1", path: "folder/item", name: "item", is_dir: false, size: 0, entry_type: "file", child_count: 0, children_loaded: false } as LazyTreeEntry;
    expect(getLazyEntryKey("/container.ad1", entry)).toBe("/container.ad1::lazy::folder/item");
  });
});

// =============================================================================
// getUfedEntryKey
// =============================================================================

describe("getUfedEntryKey", () => {
  it("generates ufed-prefixed key", () => {
    const entry = { path: "Contacts/contact1", name: "contact1", isDir: false, size: 0, entryType: "file" } as UfedTreeEntry;
    expect(getUfedEntryKey("/phone.ufdr", entry)).toBe("/phone.ufdr::ufed::Contacts/contact1");
  });
});

// =============================================================================
// getEntryKey (type-based dispatcher)
// =============================================================================

describe("getEntryKey", () => {
  const container = "/evidence/test.ad1";
  const entryPath = "dir/file.txt";

  it("generates ad1 key", () => {
    expect(getEntryKey(container, entryPath, "ad1")).toBe(`${container}::${entryPath}`);
  });

  it("generates vfs key", () => {
    expect(getEntryKey(container, entryPath, "vfs")).toBe(`${container}::vfs::${entryPath}`);
  });

  it("generates archive key", () => {
    expect(getEntryKey(container, entryPath, "archive")).toBe(`${container}::archive::${entryPath}`);
  });

  it("generates lazy key", () => {
    expect(getEntryKey(container, entryPath, "lazy")).toBe(`${container}::lazy::${entryPath}`);
  });

  it("generates ufed key", () => {
    expect(getEntryKey(container, entryPath, "ufed")).toBe(`${container}::ufed::${entryPath}`);
  });

  it("each type produces a unique key for the same path", () => {
    const types: EntryKeyType[] = ["ad1", "vfs", "archive", "lazy", "ufed"];
    const keys = types.map((t) => getEntryKey(container, entryPath, t));
    const unique = new Set(keys);
    expect(unique.size).toBe(types.length);
  });
});
