// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot } from "solid-js";
import {
  usePreviewCache,
  createCacheKey,
  type PreviewCacheManager,
} from "../usePreviewCache";
import type { PreviewCache } from "../../types/project";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function createHook(): { hook: PreviewCacheManager; dispose: () => void } {
  let hook!: PreviewCacheManager;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = usePreviewCache();
  });
  return { hook, dispose };
}

// ==========================================================================
// createCacheKey (exported pure function)
// ==========================================================================

describe("createCacheKey", () => {
  it("joins container and entry paths with ::", () => {
    expect(createCacheKey("/path/to/container.ad1", "folder/file.txt"))
      .toBe("/path/to/container.ad1::folder/file.txt");
  });

  it("handles empty entry path", () => {
    expect(createCacheKey("/container.ad1", "")).toBe("/container.ad1::");
  });

  it("handles paths with special characters", () => {
    expect(createCacheKey("/path/my file.ad1", "dir/sub dir/file (1).pdf"))
      .toBe("/path/my file.ad1::dir/sub dir/file (1).pdf");
  });
});

// ==========================================================================
// usePreviewCache
// ==========================================================================

describe("usePreviewCache", () => {
  // -----------------------------------------------------------------------
  // Initial state
  // -----------------------------------------------------------------------
  describe("initial state", () => {
    it("starts with empty cache", () => {
      const { hook, dispose } = createHook();
      expect(hook.getStats().count).toBe(0);
      expect(hook.getStats().totalSize).toBe(0);
      dispose();
    });

    it("cacheDir starts null", () => {
      const { hook, dispose } = createHook();
      expect(hook.cacheDir()).toBeNull();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // addToCache
  // -----------------------------------------------------------------------
  describe("addToCache", () => {
    it("adds an entry to the cache", () => {
      const { hook, dispose } = createHook();

      hook.addToCache("/c.ad1", "file.txt", "/tmp/file.txt", 1024);

      expect(hook.getCachedPath("/c.ad1", "file.txt")).toBe("/tmp/file.txt");
      expect(hook.getStats().count).toBe(1);
      dispose();
    });

    it("overwrites existing entry for same key", () => {
      const { hook, dispose } = createHook();

      hook.addToCache("/c.ad1", "file.txt", "/tmp/old.txt", 100);
      hook.addToCache("/c.ad1", "file.txt", "/tmp/new.txt", 200);

      expect(hook.getCachedPath("/c.ad1", "file.txt")).toBe("/tmp/new.txt");
      expect(hook.getStats().count).toBe(1);
      expect(hook.getStats().totalSize).toBe(200);
      dispose();
    });

    it("supports multiple entries", () => {
      const { hook, dispose } = createHook();

      hook.addToCache("/c.ad1", "a.txt", "/tmp/a.txt", 100);
      hook.addToCache("/c.ad1", "b.txt", "/tmp/b.txt", 200);
      hook.addToCache("/d.e01", "c.txt", "/tmp/c.txt", 300);

      expect(hook.getStats().count).toBe(3);
      expect(hook.getStats().totalSize).toBe(600);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // getCachedPath
  // -----------------------------------------------------------------------
  describe("getCachedPath", () => {
    it("returns null for uncached entry", () => {
      const { hook, dispose } = createHook();
      expect(hook.getCachedPath("/c.ad1", "missing.txt")).toBeNull();
      dispose();
    });

    it("returns temp_path for cached valid entry", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "file.txt", "/tmp/file.txt", 1024);

      expect(hook.getCachedPath("/c.ad1", "file.txt")).toBe("/tmp/file.txt");
      dispose();
    });

    it("imported entries with valid=false become undefined (pending validation)", () => {
      const { hook, dispose } = createHook();
      // importCache overrides valid to undefined for all entries
      const cache: PreviewCache = {
        entries: [
          {
            key: "/c.ad1::file.txt",
            container_path: "/c.ad1",
            entry_path: "file.txt",
            temp_path: "/tmp/file.txt",
            entry_size: 100,
            extracted_at: "2025-01-01T00:00:00Z",
            valid: false,
          },
        ],
        cached_at: "2025-01-01T00:00:00Z",
      };
      hook.importCache(cache);

      // importCache sets valid=undefined (pending), so path IS returned
      expect(hook.getCachedPath("/c.ad1", "file.txt")).toBe("/tmp/file.txt");
      dispose();
    });

    it("returns path when entry has valid=undefined (pending validation)", () => {
      const { hook, dispose } = createHook();
      const cache: PreviewCache = {
        entries: [
          {
            key: "/c.ad1::file.txt",
            container_path: "/c.ad1",
            entry_path: "file.txt",
            temp_path: "/tmp/file.txt",
            entry_size: 100,
            extracted_at: "2025-01-01T00:00:00Z",
            valid: undefined,
          },
        ],
        cached_at: "2025-01-01T00:00:00Z",
      };
      hook.importCache(cache);

      // valid=undefined (not explicitly false) → should return path
      expect(hook.getCachedPath("/c.ad1", "file.txt")).toBe("/tmp/file.txt");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // removeFromCache
  // -----------------------------------------------------------------------
  describe("removeFromCache", () => {
    it("removes a specific entry", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "a.txt", "/tmp/a.txt", 100);
      hook.addToCache("/c.ad1", "b.txt", "/tmp/b.txt", 200);

      hook.removeFromCache("/c.ad1", "a.txt");

      expect(hook.getCachedPath("/c.ad1", "a.txt")).toBeNull();
      expect(hook.getCachedPath("/c.ad1", "b.txt")).toBe("/tmp/b.txt");
      expect(hook.getStats().count).toBe(1);
      dispose();
    });

    it("is a no-op for non-existent entry", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "a.txt", "/tmp/a.txt", 100);

      expect(() => hook.removeFromCache("/c.ad1", "missing.txt")).not.toThrow();
      expect(hook.getStats().count).toBe(1);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // clearCache
  // -----------------------------------------------------------------------
  describe("clearCache", () => {
    it("removes all entries", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "a.txt", "/tmp/a.txt", 100);
      hook.addToCache("/c.ad1", "b.txt", "/tmp/b.txt", 200);

      hook.clearCache();

      expect(hook.getStats().count).toBe(0);
      expect(hook.getCachedPath("/c.ad1", "a.txt")).toBeNull();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // exportCache
  // -----------------------------------------------------------------------
  describe("exportCache", () => {
    it("exports entries array with timestamp", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "file.txt", "/tmp/file.txt", 1024);

      const exported = hook.exportCache();

      expect(exported.entries).toHaveLength(1);
      expect(exported.entries[0].container_path).toBe("/c.ad1");
      expect(exported.entries[0].entry_path).toBe("file.txt");
      expect(exported.entries[0].temp_path).toBe("/tmp/file.txt");
      expect(exported.entries[0].entry_size).toBe(1024);
      expect(exported.entries[0].valid).toBe(true);
      expect(exported.cached_at).toBeTruthy();
      dispose();
    });

    it("exports empty array when cache is empty", () => {
      const { hook, dispose } = createHook();

      const exported = hook.exportCache();

      expect(exported.entries).toHaveLength(0);
      dispose();
    });

    it("includes cache_dir when set", () => {
      const { hook, dispose } = createHook();
      hook.setCacheDir("/tmp/cache");

      const exported = hook.exportCache();

      expect(exported.cache_dir).toBe("/tmp/cache");
      dispose();
    });

    it("cache_dir is undefined when not set", () => {
      const { hook, dispose } = createHook();

      const exported = hook.exportCache();

      expect(exported.cache_dir).toBeUndefined();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // importCache
  // -----------------------------------------------------------------------
  describe("importCache", () => {
    it("imports entries from project cache", () => {
      const { hook, dispose } = createHook();
      const cache: PreviewCache = {
        entries: [
          {
            key: "/c.ad1::a.txt",
            container_path: "/c.ad1",
            entry_path: "a.txt",
            temp_path: "/tmp/a.txt",
            entry_size: 100,
            extracted_at: "2025-01-01T00:00:00Z",
            valid: true,
          },
          {
            key: "/c.ad1::b.txt",
            container_path: "/c.ad1",
            entry_path: "b.txt",
            temp_path: "/tmp/b.txt",
            entry_size: 200,
            extracted_at: "2025-01-01T00:00:00Z",
          },
        ],
        cached_at: "2025-01-01T00:00:00Z",
        cache_dir: "/tmp/cache",
      };

      hook.importCache(cache);

      expect(hook.getStats().count).toBe(2);
      expect(hook.cacheDir()).toBe("/tmp/cache");
      dispose();
    });

    it("marks imported entries as valid=undefined for later validation", () => {
      const { hook, dispose } = createHook();
      const cache: PreviewCache = {
        entries: [
          {
            key: "/c.ad1::a.txt",
            container_path: "/c.ad1",
            entry_path: "a.txt",
            temp_path: "/tmp/a.txt",
            entry_size: 100,
            extracted_at: "2025-01-01T00:00:00Z",
            valid: true, // was true in saved data
          },
        ],
        cached_at: "2025-01-01T00:00:00Z",
      };

      hook.importCache(cache);

      // getCachedPath should still work (valid !== false)
      expect(hook.getCachedPath("/c.ad1", "a.txt")).toBe("/tmp/a.txt");
      dispose();
    });

    it("clears cache when called with undefined", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "a.txt", "/tmp/a.txt", 100);

      hook.importCache(undefined);

      expect(hook.getStats().count).toBe(0);
      dispose();
    });

    it("clears cache when entries are missing", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "a.txt", "/tmp/a.txt", 100);

      hook.importCache({ entries: undefined as any, cached_at: "" });

      expect(hook.getStats().count).toBe(0);
      dispose();
    });

    it("replaces existing cache entirely", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "old.txt", "/tmp/old.txt", 50);

      hook.importCache({
        entries: [
          {
            key: "/c.ad1::new.txt",
            container_path: "/c.ad1",
            entry_path: "new.txt",
            temp_path: "/tmp/new.txt",
            entry_size: 999,
            extracted_at: "2025-01-01T00:00:00Z",
          },
        ],
        cached_at: "2025-01-01T00:00:00Z",
      });

      expect(hook.getCachedPath("/c.ad1", "old.txt")).toBeNull();
      expect(hook.getStats().count).toBe(1);
      expect(hook.getStats().totalSize).toBe(999);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // getStats
  // -----------------------------------------------------------------------
  describe("getStats", () => {
    it("returns count and totalSize", () => {
      const { hook, dispose } = createHook();
      hook.addToCache("/c.ad1", "a.txt", "/tmp/a", 100);
      hook.addToCache("/c.ad1", "b.txt", "/tmp/b", 250);
      hook.addToCache("/c.ad1", "c.txt", "/tmp/c", 650);

      const stats = hook.getStats();
      expect(stats.count).toBe(3);
      expect(stats.totalSize).toBe(1000);
      dispose();
    });

    it("returns zeros for empty cache", () => {
      const { hook, dispose } = createHook();
      expect(hook.getStats()).toEqual({ count: 0, totalSize: 0 });
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // cacheDir
  // -----------------------------------------------------------------------
  describe("cacheDir", () => {
    it("can be set and read", () => {
      const { hook, dispose } = createHook();
      hook.setCacheDir("/tmp/my-cache");
      expect(hook.cacheDir()).toBe("/tmp/my-cache");
      dispose();
    });

    it("can be reset to null", () => {
      const { hook, dispose } = createHook();
      hook.setCacheDir("/tmp/cache");
      hook.setCacheDir(null);
      expect(hook.cacheDir()).toBeNull();
      dispose();
    });
  });
});
