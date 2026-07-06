import { createRoot } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { useNestedContainers } from "./useNestedContainers";
import type { NestedContainerEntry, NestedContainerInfo } from "../../../types";

vi.mock("../../../utils/platform", () => ({
  isTauri: true,
}));

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

const nestedEntry: NestedContainerEntry = {
  path: "Documents/report.pdf",
  name: "report.pdf",
  isDir: false,
  size: 1024,
  hash: null,
  modified: "2026-01-01T00:00:00Z",
  sourceType: "ad1",
  isNestedContainer: false,
  nestedType: null,
};

const nestedInfo: NestedContainerInfo = {
  containerType: "ad1",
  entryCount: 1,
  totalSize: 1024,
  encrypted: false,
  tempPath: "/tmp/source.AD1",
  originalPath: "nested/source.AD1",
};

describe("useNestedContainers", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("shares concurrent nested tree loads", async () => {
    const pending = deferred<NestedContainerEntry[]>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "nested_container_get_tree") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useNestedContainers);

    const first = hook.loadNestedContainerTree("/case/export.zip", "nested/source.AD1");
    const second = hook.loadNestedContainerTree("/case/export.zip", "nested/source.AD1");

    pending.resolve([nestedEntry]);
    const [firstEntries, secondEntries] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstEntries).toEqual(secondEntries);
    expect(hook.nestedEntriesCache().get("/case/export.zip::nested::nested/source.AD1")).toHaveLength(1);
    expect(hook.isNestedLoading("/case/export.zip", "nested/source.AD1")).toBe(false);
    dispose();
  });

  it("shares concurrent nested info loads", async () => {
    const pending = deferred<NestedContainerInfo>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "nested_container_get_info") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useNestedContainers);

    const first = hook.loadNestedContainerInfo("/case/export.zip", "nested/source.AD1");
    const second = hook.loadNestedContainerInfo("/case/export.zip", "nested/source.AD1");

    pending.resolve(nestedInfo);
    const [firstInfo, secondInfo] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstInfo).toEqual(secondInfo);
    expect(hook.nestedInfoCache().get("/case/export.zip::nested::nested/source.AD1")).toEqual(nestedInfo);
    dispose();
  });

  it("shares concurrent nested cache clears", async () => {
    const pending = deferred<void>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "nested_container_clear_cache") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useNestedContainers);

    const first = hook.clearNestedCache();
    const second = hook.clearNestedCache();

    pending.resolve();
    await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(hook.nestedEntriesCache().size).toBe(0);
    expect(hook.nestedInfoCache().size).toBe(0);
    dispose();
  });
});
