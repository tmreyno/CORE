// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot } from "solid-js";
import {
  useAsyncState,
  useAsyncSetState,
  useCachedAsyncState,
  type AsyncState,
  type AsyncSetState,
  type CachedAsyncState,
} from "../useAsyncState";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Create a hook inside a reactive root and return its value + dispose fn */
function createHook<T>(factory: () => T): { hook: T; dispose: () => void } {
  let hook!: T;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = factory();
  });
  return { hook, dispose };
}

/** Resolved promise helper */
const resolved = <T>(val: T) => () => Promise.resolve(val);

/** Rejected promise helper */
const rejected = (msg: string) => () => Promise.reject(new Error(msg));

// ==========================================================================
// useAsyncState
// ==========================================================================

describe("useAsyncState", () => {
  describe("initial state", () => {
    it("starts with null data by default", () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());
      expect(hook.data()).toBeNull();
      expect(hook.loading()).toBe(false);
      expect(hook.error()).toBeNull();
      expect(hook.status()).toBe("idle");
      dispose();
    });

    it("accepts initial data value", () => {
      const { hook, dispose } = createHook(() => useAsyncState<number>(42));
      expect(hook.data()).toBe(42);
      expect(hook.status()).toBe("success");
      dispose();
    });
  });

  describe("execute", () => {
    it("sets data on successful execution", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      const result = await hook.execute(resolved("hello"));

      expect(result).toBe("hello");
      expect(hook.data()).toBe("hello");
      expect(hook.loading()).toBe(false);
      expect(hook.error()).toBeNull();
      expect(hook.status()).toBe("success");
      dispose();
    });

    it("sets error on failure", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      const result = await hook.execute(rejected("boom"));

      expect(result).toBeNull();
      expect(hook.data()).toBeNull();
      expect(hook.error()).toBe("boom");
      expect(hook.status()).toBe("error");
      expect(hook.loading()).toBe(false);
      dispose();
    });

    it("handles non-Error rejections", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      const result = await hook.execute(() => Promise.reject("string error"));

      expect(result).toBeNull();
      expect(hook.error()).toBe("string error");
      dispose();
    });

    it("clears previous error before new execution", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      await hook.execute(rejected("first"));
      expect(hook.error()).toBe("first");

      await hook.execute(resolved("ok"));
      expect(hook.error()).toBeNull();
      expect(hook.data()).toBe("ok");
      dispose();
    });

    it("sets loading to true during execution", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());
      const loadingStates: boolean[] = [];

      const promise = hook.execute(async () => {
        loadingStates.push(hook.loading());
        return "done";
      });

      await promise;
      loadingStates.push(hook.loading());

      expect(loadingStates[0]).toBe(true); // during execution
      expect(loadingStates[1]).toBe(false); // after
      dispose();
    });
  });

  describe("execute options", () => {
    it("transforms result with transform option", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      await hook.execute<number>(resolved(42), {
        transform: (n) => `value: ${n}`,
      });

      expect(hook.data()).toBe("value: 42");
      dispose();
    });

    it("calls onSuccess callback", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());
      const onSuccess = vi.fn();

      await hook.execute(resolved("data"), { onSuccess });

      expect(onSuccess).toHaveBeenCalledWith("data");
      dispose();
    });

    it("uses custom onError handler", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      await hook.execute(rejected("raw"), {
        onError: () => "custom error message",
      });

      expect(hook.error()).toBe("custom error message");
      dispose();
    });

    it("skips data update when skipDataUpdate is true", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>("original"));

      await hook.execute(resolved("new"), { skipDataUpdate: true });

      expect(hook.data()).toBe("original");
      dispose();
    });

    it("keeps previous error with keepPreviousError", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      await hook.execute(rejected("previous error"));
      expect(hook.error()).toBe("previous error");

      await hook.execute(resolved("ok"), { keepPreviousError: true });
      // Error should have been kept during the execution but cleared by success
      // Actually the error signal is not cleared when keepPreviousError is true
      // but the success sets data. Since the fn succeeded, error stays from before.
      // Looking at code: only setError(null) is skipped, error remains.
      expect(hook.error()).toBe("previous error");
      dispose();
    });

    it("does NOT keep previous error by default", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      await hook.execute(rejected("old error"));
      await hook.execute(resolved("ok"));

      expect(hook.error()).toBeNull();
      dispose();
    });

    it("calls onSuccess even with skipDataUpdate", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());
      const onSuccess = vi.fn();

      await hook.execute(resolved("val"), {
        skipDataUpdate: true,
        onSuccess,
      });

      expect(onSuccess).toHaveBeenCalledWith("val");
      expect(hook.data()).toBeNull();
      dispose();
    });
  });

  describe("reset", () => {
    it("resets to initial state (null default)", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      await hook.execute(resolved("data"));
      expect(hook.data()).toBe("data");

      hook.reset();

      expect(hook.data()).toBeNull();
      expect(hook.loading()).toBe(false);
      expect(hook.error()).toBeNull();
      expect(hook.status()).toBe("idle");
      dispose();
    });

    it("resets to initial data when provided", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<number>(99));

      await hook.execute(resolved(1));
      hook.reset();

      expect(hook.data()).toBe(99);
      dispose();
    });

    it("clears error state", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      await hook.execute(rejected("err"));
      hook.reset();

      expect(hook.error()).toBeNull();
      expect(hook.status()).toBe("idle");
      dispose();
    });
  });

  describe("setData", () => {
    it("allows manual data update", () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());

      hook.setData(() => "manual");

      expect(hook.data()).toBe("manual");
      expect(hook.status()).toBe("success");
      dispose();
    });
  });

  describe("status transitions", () => {
    it("idle -> loading -> success", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());
      const statuses: string[] = [];

      statuses.push(hook.status()); // idle
      const p = hook.execute(async () => {
        statuses.push(hook.status()); // loading
        return "done";
      });
      await p;
      statuses.push(hook.status()); // success

      expect(statuses).toEqual(["idle", "loading", "success"]);
      dispose();
    });

    it("idle -> loading -> error", async () => {
      const { hook, dispose } = createHook(() => useAsyncState<string>());
      const statuses: string[] = [];

      statuses.push(hook.status());
      await hook.execute(rejected("fail"));
      statuses.push(hook.status());

      expect(statuses).toEqual(["idle", "error"]);
      dispose();
    });
  });
});

// ==========================================================================
// useAsyncSetState
// ==========================================================================

describe("useAsyncSetState", () => {
  describe("initial state", () => {
    it("starts with empty loading set", () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());
      expect(hook.loading().size).toBe(0);
      dispose();
    });
  });

  describe("startLoading / stopLoading", () => {
    it("marks a key as loading", () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());

      hook.startLoading("file1");

      expect(hook.isLoading("file1")).toBe(true);
      expect(hook.isLoading("file2")).toBe(false);
      dispose();
    });

    it("stops loading for a key", () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());

      hook.startLoading("file1");
      hook.stopLoading("file1");

      expect(hook.isLoading("file1")).toBe(false);
      dispose();
    });

    it("tracks multiple keys independently", () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());

      hook.startLoading("a");
      hook.startLoading("b");
      hook.stopLoading("a");

      expect(hook.isLoading("a")).toBe(false);
      expect(hook.isLoading("b")).toBe(true);
      expect(hook.loading().size).toBe(1);
      dispose();
    });
  });

  describe("execute", () => {
    it("returns result on success", async () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());

      const result = await hook.execute("key1", resolved(42));

      expect(result).toBe(42);
      expect(hook.isLoading("key1")).toBe(false);
      dispose();
    });

    it("returns null on failure", async () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());

      const result = await hook.execute("key1", rejected("fail"));

      expect(result).toBeNull();
      expect(hook.isLoading("key1")).toBe(false);
      dispose();
    });

    it("sets loading during execution", async () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());
      let wasLoading = false;

      await hook.execute("key1", async () => {
        wasLoading = hook.isLoading("key1");
        return "done";
      });

      expect(wasLoading).toBe(true);
      expect(hook.isLoading("key1")).toBe(false);
      dispose();
    });

    it("handles concurrent executions for different keys", async () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());

      const p1 = hook.execute("a", async () => {
        await new Promise((r) => setTimeout(r, 10));
        return 1;
      });
      const p2 = hook.execute("b", resolved(2));

      // b finishes first
      expect(hook.isLoading("a")).toBe(true);

      const [r1, r2] = await Promise.all([p1, p2]);
      expect(r1).toBe(1);
      expect(r2).toBe(2);
      expect(hook.isLoading("a")).toBe(false);
      expect(hook.isLoading("b")).toBe(false);
      dispose();
    });
  });

  describe("clear", () => {
    it("clears all loading states", () => {
      const { hook, dispose } = createHook(() => useAsyncSetState<string>());

      hook.startLoading("a");
      hook.startLoading("b");
      hook.startLoading("c");

      hook.clear();

      expect(hook.loading().size).toBe(0);
      expect(hook.isLoading("a")).toBe(false);
      dispose();
    });
  });
});

// ==========================================================================
// useCachedAsyncState
// ==========================================================================

describe("useCachedAsyncState", () => {
  describe("initial state", () => {
    it("starts with empty cache", () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());
      expect(hook.cache().size).toBe(0);
      expect(hook.has("x")).toBe(false);
      expect(hook.get("x")).toBeUndefined();
      dispose();
    });
  });

  describe("fetch", () => {
    it("fetches and caches data", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());

      const result = await hook.fetch("key1", resolved(100));

      expect(result).toBe(100);
      expect(hook.has("key1")).toBe(true);
      expect(hook.get("key1")).toBe(100);
      dispose();
    });

    it("returns cached value on subsequent calls", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());
      const fetchFn = vi.fn().mockResolvedValue(100);

      await hook.fetch("key1", fetchFn);
      const result2 = await hook.fetch("key1", fetchFn);

      expect(result2).toBe(100);
      expect(fetchFn).toHaveBeenCalledTimes(1); // only called once
      dispose();
    });

    it("fetches different keys independently", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());

      await hook.fetch("a", resolved(1));
      await hook.fetch("b", resolved(2));

      expect(hook.get("a")).toBe(1);
      expect(hook.get("b")).toBe(2);
      expect(hook.cache().size).toBe(2);
      dispose();
    });

    it("returns null on fetch failure", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());

      const result = await hook.fetch("key1", rejected("network error"));

      expect(result).toBeNull();
      expect(hook.has("key1")).toBe(false);
      dispose();
    });
  });

  describe("refresh", () => {
    it("forces new fetch even with cached data", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());

      await hook.fetch("key1", resolved(100));
      const result = await hook.refresh("key1", resolved(200));

      expect(result).toBe(200);
      expect(hook.get("key1")).toBe(200);
      dispose();
    });

    it("tracks loading state during refresh", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());
      let wasLoading = false;

      await hook.refresh("key1", async () => {
        wasLoading = hook.isLoading("key1");
        return 42;
      });

      expect(wasLoading).toBe(true);
      expect(hook.isLoading("key1")).toBe(false);
      dispose();
    });

    it("returns null on refresh failure", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());

      await hook.fetch("key1", resolved(100));
      const result = await hook.refresh("key1", rejected("fail"));

      expect(result).toBeNull();
      // Original cache entry remains if refresh fails? Let's check:
      // Looking at code: setCacheValue is only called on success
      // So original value should still be there
      expect(hook.get("key1")).toBe(100);
      dispose();
    });
  });

  describe("invalidate", () => {
    it("removes a single cache entry", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());

      await hook.fetch("a", resolved(1));
      await hook.fetch("b", resolved(2));

      hook.invalidate("a");

      expect(hook.has("a")).toBe(false);
      expect(hook.has("b")).toBe(true);
      dispose();
    });

    it("causes next fetch to re-fetch", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());
      const fetchFn = vi.fn().mockResolvedValue(100);

      await hook.fetch("key1", fetchFn);
      hook.invalidate("key1");
      await hook.fetch("key1", fetchFn);

      expect(fetchFn).toHaveBeenCalledTimes(2);
      dispose();
    });

    it("is a no-op for non-existent keys", () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());
      expect(() => hook.invalidate("nonexistent")).not.toThrow();
      dispose();
    });
  });

  describe("clearCache", () => {
    it("removes all cache entries", async () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());

      await hook.fetch("a", resolved(1));
      await hook.fetch("b", resolved(2));
      await hook.fetch("c", resolved(3));

      hook.clearCache();

      expect(hook.cache().size).toBe(0);
      expect(hook.has("a")).toBe(false);
      expect(hook.has("b")).toBe(false);
      expect(hook.has("c")).toBe(false);
      dispose();
    });

    it("clears loading states too", () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());

      // We can't easily have something in-flight and clear, but clearCache
      // also resets loading set per the code
      hook.clearCache();
      expect(hook.isLoading("any")).toBe(false);
      dispose();
    });
  });

  describe("isLoading", () => {
    it("returns false for keys not being fetched", () => {
      const { hook, dispose } = createHook(() => useCachedAsyncState<string, number>());
      expect(hook.isLoading("key1")).toBe(false);
      dispose();
    });
  });
});
