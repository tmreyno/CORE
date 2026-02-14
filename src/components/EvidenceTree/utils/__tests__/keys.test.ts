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
} from "../keys";
import type { TreeEntry, VfsEntry, ArchiveTreeEntry, UfedTreeEntry } from "../../../../types";
import type { LazyTreeEntry } from "../../../../types/lazy-loading";

// Minimal mock entries
const ad1Entry: TreeEntry = {
  name: "evidence.doc",
  path: "/root/evidence.doc",
  is_dir: false,
  item_addr: 12345,
  first_child_addr: null,
  size: 1024,
  item_type: 0x00,
};

const ad1Dir: TreeEntry = {
  name: "Documents",
  path: "/root/Documents",
  is_dir: true,
  item_addr: 100,
  first_child_addr: 200,
  size: 0,
  item_type: 0x05,
};

const vfsEntry: VfsEntry = {
  name: "system.log",
  path: "/var/log/system.log",
  isDir: false,
  size: 2048,
};

const archiveEntry: ArchiveTreeEntry = {
  path: "docs/readme.txt",
  name: "readme.txt",
  isDir: false,
  size: 512,
  compressedSize: 256,
  crc32: 0xDEADBEEF,
  modified: "2024-01-15",
};

const ufedEntry: UfedTreeEntry = {
  name: "sms.db",
  path: "/data/sms.db",
  isDir: false,
  size: 4096,
  entryType: "file",
};

// =============================================================================
// getAd1EntryKey
// =============================================================================
describe("getAd1EntryKey", () => {
  it("uses item_addr when available", () => {
    const key = getAd1EntryKey("/evidence/case.ad1", ad1Entry);
    expect(key).toBe("/evidence/case.ad1::12345");
  });

  it("falls back to path when no item_addr", () => {
    const entry = { ...ad1Entry, item_addr: undefined } as unknown as TreeEntry;
    const key = getAd1EntryKey("/evidence/case.ad1", entry);
    expect(key).toBe("/evidence/case.ad1::/root/evidence.doc");
  });
});

// =============================================================================
// getAd1NodeKey
// =============================================================================
describe("getAd1NodeKey", () => {
  it("uses first_child_addr for directories", () => {
    const key = getAd1NodeKey("/evidence/case.ad1", ad1Dir);
    expect(key).toBe("/evidence/case.ad1::addr:200");
  });

  it("falls back to path-based key when no first_child_addr", () => {
    const entry = { ...ad1Dir, first_child_addr: 0 };
    const key = getAd1NodeKey("/evidence/case.ad1", entry);
    expect(key).toBe("/evidence/case.ad1::path:/root/Documents");
  });
});

// =============================================================================
// getVfsEntryKey
// =============================================================================
describe("getVfsEntryKey", () => {
  it("generates vfs-namespaced key", () => {
    const key = getVfsEntryKey("/disk.e01", vfsEntry);
    expect(key).toBe("/disk.e01::vfs::/var/log/system.log");
  });
});

// =============================================================================
// getArchiveEntryKey
// =============================================================================
describe("getArchiveEntryKey", () => {
  it("generates archive-namespaced key", () => {
    const key = getArchiveEntryKey("/files/backup.7z", archiveEntry);
    expect(key).toBe("/files/backup.7z::archive::docs/readme.txt");
  });
});

// =============================================================================
// getLazyEntryKey
// =============================================================================
describe("getLazyEntryKey", () => {
  it("generates lazy-namespaced key", () => {
    const entry: LazyTreeEntry = {
      id: "lazy-1",
      name: "file.txt",
      path: "/deep/file.txt",
      is_dir: false,
      size: 100,
      entry_type: "file",
      child_count: 0,
      children_loaded: false,
      hash: null,
      modified: null,
      metadata: null,
    };
    const key = getLazyEntryKey("/large.ad1", entry);
    expect(key).toBe("/large.ad1::lazy::/deep/file.txt");
  });
});

// =============================================================================
// getUfedEntryKey
// =============================================================================
describe("getUfedEntryKey", () => {
  it("generates ufed-namespaced key", () => {
    const key = getUfedEntryKey("/phone.ufed", ufedEntry);
    expect(key).toBe("/phone.ufed::ufed::/data/sms.db");
  });
});

// =============================================================================
// getEntryKey (generic)
// =============================================================================
describe("getEntryKey", () => {
  it("generates correct key for each type", () => {
    const container = "/evidence/container";
    const path = "/files/test.bin";
    
    expect(getEntryKey(container, path, "ad1")).toBe(`${container}::${path}`);
    expect(getEntryKey(container, path, "vfs")).toBe(`${container}::vfs::${path}`);
    expect(getEntryKey(container, path, "archive")).toBe(`${container}::archive::${path}`);
    expect(getEntryKey(container, path, "lazy")).toBe(`${container}::lazy::${path}`);
    expect(getEntryKey(container, path, "ufed")).toBe(`${container}::ufed::${path}`);
  });

  it("keys are unique across types", () => {
    const keys = new Set([
      getEntryKey("/c", "/f", "ad1"),
      getEntryKey("/c", "/f", "vfs"),
      getEntryKey("/c", "/f", "archive"),
      getEntryKey("/c", "/f", "lazy"),
      getEntryKey("/c", "/f", "ufed"),
    ]);
    expect(keys.size).toBe(5);
  });
});
