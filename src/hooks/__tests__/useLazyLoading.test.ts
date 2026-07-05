import { createRoot, createSignal } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import { useLazyLoading } from "../useLazyLoading";
import type { LazyLoadConfig, LazyLoadResult, LazyTreeEntry } from "../../types/lazy-loading";

vi.mock("../../utils/platform", () => ({
  isTauri: true,
}));

function withHook() {
  let lazy!: ReturnType<typeof useLazyLoading>;
  let dispose!: () => void;
  const [path, setPath] = createSignal<string | null>("/case/first.ufdr");

  createRoot((d) => {
    dispose = d;
    lazy = useLazyLoading(path);
  });

  return { lazy, setPath, dispose };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

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

describe("useLazyLoading", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "lazy_get_settings") return Promise.resolve(config);
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
  });

  it("ignores stale root children after the container path changes", async () => {
    const pendingRoot = deferred<LazyLoadResult>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "lazy_get_settings") return Promise.resolve(config);
      if (command === "lazy_get_root_children") return pendingRoot.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { lazy, setPath, dispose } = withHook();
    await tick();

    const load = lazy.loadRootChildren();
    expect(lazy.isLoadingRoot()).toBe(true);

    setPath("/case/second.ufdr");
    await tick();
    expect(lazy.isLoadingRoot()).toBe(false);

    pendingRoot.resolve(result([entry("stale-root")]));
    await load;
    await tick();

    expect(lazy.rootChildren()).toEqual([]);
    expect(lazy.rootTotalCount()).toBe(0);
    expect(lazy.hasMoreRoot()).toBe(false);
    dispose();
  });

  it("ignores stale child entries after the container path changes", async () => {
    const pendingChildren = deferred<LazyLoadResult>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "lazy_get_settings") return Promise.resolve(config);
      if (command === "lazy_get_children") return pendingChildren.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { lazy, setPath, dispose } = withHook();
    await tick();

    const load = lazy.loadChildren("/Photos");
    expect(lazy.isLoading()).toBe(true);

    setPath("/case/second.ufdr");
    await tick();
    expect(lazy.isLoading()).toBe(false);

    pendingChildren.resolve(result([entry("stale-child", "/Photos/stale-child")]));
    await load;
    await tick();

    expect(lazy.childrenCache().size).toBe(0);
    expect(lazy.getCachedChildren("/Photos")).toBeUndefined();
    dispose();
  });
});
