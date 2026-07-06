import { createRoot } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { useAd1Tree } from "./useAd1Tree";
import { useArchiveTree } from "./useArchiveTree";
import { useNestedContainers } from "./useNestedContainers";
import { useVfsTree } from "./useVfsTree";

vi.mock("../../../utils/platform", () => ({
  isTauri: false,
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

describe("EvidenceTree browser runtime guards", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  it("skips AD1 backend tree and metadata commands", async () => {
    const { hook, dispose } = withHook(useAd1Tree);

    await expect(hook.loadContainerStatus("/case/source.AD1")).resolves.toBeNull();
    await expect(hook.loadAd1Info("/case/source.AD1")).resolves.toBeNull();
    await expect(hook.loadRootChildren("/case/source.AD1")).resolves.toEqual([]);
    await expect(hook.loadChildrenByAddr("/case/source.AD1", 1, "/")).resolves.toEqual([]);
    await expect(hook.loadItemMetadata("/case/source.AD1", 1)).resolves.toBeNull();
    await expect(hook.loadItemsMetadata("/case/source.AD1", [1, 2])).resolves.toEqual([]);

    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });

  it("skips archive backend tree, metadata, and extraction commands", async () => {
    const { hook, dispose } = withHook(useArchiveTree);
    const onOpenNestedContainer = vi.fn();

    await expect(hook.loadArchiveMetadata("/case/export.zip")).resolves.toBeNull();
    await expect(hook.loadArchiveTree("/case/export.zip")).resolves.toEqual([]);
    await hook.openNestedContainer(
      "/case/export.zip",
      "nested.AD1",
      "nested.AD1",
      onOpenNestedContainer,
      new Set(),
      (fn) => fn(new Set()),
    );

    expect(onOpenNestedContainer).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });

  it("skips VFS mount and directory commands", async () => {
    const { hook, dispose } = withHook(useVfsTree);

    await expect(hook.mountVfsContainer("/case/disk.E01")).resolves.toBeNull();
    await expect(hook.loadVfsChildren("/case/disk.E01", "/Partition1")).resolves.toEqual([]);

    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });

  it("skips nested container backend commands and clears local cache only", async () => {
    const { hook, dispose } = withHook(useNestedContainers);

    await expect(hook.loadNestedContainerTree("/case/export.zip", "nested.AD1")).resolves.toEqual([]);
    await expect(hook.loadNestedContainerInfo("/case/export.zip", "nested.AD1")).resolves.toBeNull();
    await hook.clearNestedCache();

    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });
});
