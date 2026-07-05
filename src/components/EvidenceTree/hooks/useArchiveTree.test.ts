import { createRoot } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { useArchiveTree, type ArchiveQuickMetadata } from "./useArchiveTree";
import type { ArchiveTreeEntry } from "../../../types";

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

const metadata: ArchiveQuickMetadata = {
  entry_count: 1,
  archive_size: 2048,
  format: "zip",
  encrypted: false,
};

const archiveEntry: ArchiveTreeEntry = {
  path: "nested/source.AD1",
  name: "source.AD1",
  isDir: false,
  size: 1024,
  compressedSize: 512,
  crc32: 123,
  modified: "2026-01-01T00:00:00Z",
};

describe("useArchiveTree", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("shares concurrent archive metadata loads", async () => {
    const pending = deferred<ArchiveQuickMetadata>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "archive_get_metadata") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useArchiveTree);

    const first = hook.loadArchiveMetadata("/case/export.zip");
    const second = hook.loadArchiveMetadata("/case/export.zip");

    pending.resolve(metadata);
    const [firstMetadata, secondMetadata] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstMetadata).toEqual(secondMetadata);
    expect(hook.archiveMetaCache().get("/case/export.zip")).toEqual(metadata);
    dispose();
  });

  it("shares concurrent archive tree loads", async () => {
    const pending = deferred<ArchiveTreeEntry[]>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "archive_get_tree") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useArchiveTree);

    const first = hook.loadArchiveTree("/case/export.zip");
    const second = hook.loadArchiveTree("/case/export.zip");

    pending.resolve([archiveEntry]);
    const [firstEntries, secondEntries] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstEntries).toEqual(secondEntries);
    expect(hook.archiveTreeCache().get("/case/export.zip")).toHaveLength(1);
    dispose();
  });

  it("shares concurrent nested archive extraction requests", async () => {
    const pending = deferred<string>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "archive_extract_entry") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useArchiveTree);
    const onOpenNestedContainer = vi.fn();
    let loading = new Set<string>();
    const setLoading = (fn: (prev: Set<string>) => Set<string>) => {
      loading = fn(loading);
    };

    const first = hook.openNestedContainer(
      "/case/export.zip",
      "nested/source.AD1",
      "source.AD1",
      onOpenNestedContainer,
      loading,
      setLoading,
    );
    const second = hook.openNestedContainer(
      "/case/export.zip",
      "nested/source.AD1",
      "source.AD1",
      onOpenNestedContainer,
      loading,
      setLoading,
    );

    pending.resolve("/tmp/source.AD1");
    await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(onOpenNestedContainer).toHaveBeenCalledTimes(1);
    expect(onOpenNestedContainer).toHaveBeenCalledWith(
      "/tmp/source.AD1",
      "source.AD1",
      "ad1",
      "/case/export.zip",
    );
    expect(loading.size).toBe(0);
    dispose();
  });
});
