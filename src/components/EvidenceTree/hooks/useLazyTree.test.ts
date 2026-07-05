import { createRoot } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useLazyTree } from "./useLazyTree";
import type { LazyLoadConfig, LazyLoadResult, LazyTreeEntry } from "../../../types/lazy-loading";

const mocks = vi.hoisted(() => ({
  getContainerSummary: vi.fn(),
  getRootChildren: vi.fn(),
  getChildren: vi.fn(),
}));

vi.mock("../../../hooks/useLazyLoading", () => mocks);

function withHook<T>(factory: () => T): { hook: T; dispose: () => void } {
  let hook!: T;
  let dispose!: () => void;

  createRoot((d) => {
    dispose = d;
    hook = factory();
  });

  return { hook, dispose };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

const config: LazyLoadConfig = {
  enabled: true,
  batch_size: 100,
  auto_expand_threshold: 50,
  large_container_threshold: 10_000,
  pagination_threshold: 500,
  show_entry_count: true,
  count_timeout_ms: 5_000,
  load_timeout_ms: 30_000,
};

function entry(name: string, path = name): LazyTreeEntry {
  return {
    id: path,
    name,
    path,
    is_dir: false,
    size: 1,
    entry_type: "file",
    child_count: 0,
    children_loaded: false,
    hash: null,
    modified: null,
    metadata: null,
  };
}

function result(entries: LazyTreeEntry[]): LazyLoadResult {
  return {
    entries,
    total_count: entries.length,
    has_more: false,
    next_offset: entries.length,
    lazy_loaded: true,
    config,
  };
}

describe("useLazyTree", () => {
  beforeEach(() => {
    mocks.getContainerSummary.mockReset();
    mocks.getRootChildren.mockReset();
    mocks.getChildren.mockReset();
  });

  it("shares concurrent root child loads for the same offset", async () => {
    const pending = deferred<LazyLoadResult>();
    mocks.getRootChildren.mockReturnValue(pending.promise);
    const { hook, dispose } = withHook(useLazyTree);

    const first = hook.loadLazyRootChildren("/case/large.ufdr", 0, 100);
    const second = hook.loadLazyRootChildren("/case/large.ufdr", 0, 100);

    pending.resolve(result([entry("Photos", "Photos")]));
    const [firstEntries, secondEntries] = await Promise.all([first, second]);

    expect(mocks.getRootChildren).toHaveBeenCalledTimes(1);
    expect(firstEntries).toEqual(secondEntries);
    expect(hook.lazyChildrenCache().get("/case/large.ufdr::lazy::root")).toHaveLength(1);
    dispose();
  });

  it("shares concurrent child loads for the same path and offset", async () => {
    const pending = deferred<LazyLoadResult>();
    mocks.getChildren.mockReturnValue(pending.promise);
    const { hook, dispose } = withHook(useLazyTree);

    const first = hook.loadLazyChildren("/case/large.ufdr", "/Photos", 0, 100);
    const second = hook.loadLazyChildren("/case/large.ufdr", "/Photos", 0, 100);

    pending.resolve(result([entry("IMG_0001.JPG", "/Photos/IMG_0001.JPG")]));
    const [firstEntries, secondEntries] = await Promise.all([first, second]);

    expect(mocks.getChildren).toHaveBeenCalledTimes(1);
    expect(firstEntries).toEqual(secondEntries);
    expect(hook.lazyChildrenCache().get("/case/large.ufdr::lazy::/Photos")).toHaveLength(1);
    dispose();
  });
});
