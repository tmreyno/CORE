import { createRoot } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { useVfsTree } from "./useVfsTree";
import type { VfsEntry, VfsMountInfo } from "../../../types";

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

const mountInfo: VfsMountInfo = {
  containerPath: "/case/disk.E01",
  containerType: "e01",
  diskSize: 4096,
  mode: "filesystem",
  partitions: [
    {
      number: 1,
      mountName: "Partition1_NTFS",
      fsType: "NTFS",
      size: 4096,
      startOffset: 0,
    },
  ],
};

const windowsEntry: VfsEntry = {
  name: "Windows",
  path: "/Partition1_NTFS/Windows",
  isDir: true,
  size: 0,
  fileType: "directory",
};

describe("useVfsTree", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("shares concurrent VFS image mounts", async () => {
    const pending = deferred<VfsMountInfo>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "vfs_mount_image") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useVfsTree);

    const first = hook.mountVfsContainer("/case/disk.E01");
    const second = hook.mountVfsContainer("/case/disk.E01");

    pending.resolve(mountInfo);
    const [firstInfo, secondInfo] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstInfo).toEqual(secondInfo);
    expect(hook.vfsMountCache().get("/case/disk.E01")).toEqual(mountInfo);
    dispose();
  });

  it("shares concurrent VFS directory listings", async () => {
    const pending = deferred<VfsEntry[]>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "vfs_list_dir") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useVfsTree);

    const first = hook.loadVfsChildren("/case/disk.E01", "/Partition1_NTFS");
    const second = hook.loadVfsChildren("/case/disk.E01", "/Partition1_NTFS");

    pending.resolve([windowsEntry]);
    const [firstEntries, secondEntries] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstEntries).toEqual(secondEntries);
    expect(hook.vfsChildrenCache().get("/case/disk.E01::vfs::/Partition1_NTFS")).toHaveLength(1);
    dispose();
  });
});
