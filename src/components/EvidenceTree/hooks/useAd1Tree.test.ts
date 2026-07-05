import { createRoot } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { useAd1Tree, type ItemMetadata } from "./useAd1Tree";
import type { TreeEntry } from "../../../types";

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

async function flushPromises() {
  await Promise.resolve();
  await Promise.resolve();
}

function entry(name: string, overrides: Partial<TreeEntry> = {}): TreeEntry {
  return {
    path: name,
    name,
    is_dir: false,
    size: 1,
    item_type: 0,
    ...overrides,
  };
}

describe("useAd1Tree", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("shares concurrent AD1 root child loads", async () => {
    const pending = deferred<TreeEntry[]>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "container_get_root_children_v2") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useAd1Tree);

    const first = hook.loadRootChildren("/case/source.AD1");
    const second = hook.loadRootChildren("/case/source.AD1");

    pending.resolve([entry("Documents", { is_dir: true, item_addr: 10 })]);
    const [firstEntries, secondEntries] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstEntries).toEqual(secondEntries);
    expect(hook.childrenCache().get("/case/source.AD1::root")).toHaveLength(1);
    dispose();
  });

  it("shares concurrent AD1 child loads by item address", async () => {
    const pending = deferred<TreeEntry[]>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "container_get_children_at_addr_v2") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useAd1Tree);

    const first = hook.loadChildrenByAddr("/case/source.AD1", 42, "/Documents");
    const second = hook.loadChildrenByAddr("/case/source.AD1", 42, "/Documents");

    pending.resolve([entry("report.pdf", { path: "/Documents/report.pdf", item_addr: 50 })]);
    const [firstEntries, secondEntries] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstEntries).toEqual(secondEntries);
    expect(hook.childrenCache().get("/case/source.AD1::addr:42")).toHaveLength(1);
    dispose();
  });

  it("collects system identity entries from AD1 child loads", async () => {
    const systemHive = entry("SYSTEM", {
      path: "/Windows/System32/config/SYSTEM",
      size: 8192,
      data_addr: 123,
      item_addr: 456,
    });
    mockInvoke.mockImplementation(async (command: string) => {
      if (command === "container_get_children_at_addr_v2") return [systemHive];
      if (command === "project_db_is_open") return true;
      if (command === "project_db_collect_system_identity_sources") {
        return { scanned: 1, matched: 1, inserted: 1, skipped: 0, errors: [] };
      }
      throw new Error(`Unexpected command: ${command}`);
    });
    const { hook, dispose } = withHook(useAd1Tree);

    await hook.loadChildrenByAddr("/case/source.AD1", 42, "/Windows/System32/config");
    await flushPromises();

    expect(mockInvoke).toHaveBeenCalledWith("project_db_collect_system_identity_sources", {
      request: {
        sources: [
          {
            containerPath: "/case/source.AD1",
            entryPath: "/Windows/System32/config/SYSTEM",
            containerType: "ad1",
            size: 8192,
            dataAddr: 123,
            itemAddr: 456,
          },
        ],
        extractor: "evidence-tree-system-identity",
      },
    });
    dispose();
  });

  it("shares concurrent AD1 item metadata loads", async () => {
    const pending = deferred<ItemMetadata>();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "container_get_item_metadata_v2") return pending.promise;
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const { hook, dispose } = withHook(useAd1Tree);

    const first = hook.loadItemMetadata("/case/source.AD1", 99);
    const second = hook.loadItemMetadata("/case/source.AD1", 99);

    pending.resolve({ itemAddr: 99, md5Hash: "abc123" });
    const [firstMetadata, secondMetadata] = await Promise.all([first, second]);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(firstMetadata).toEqual(secondMetadata);
    expect(hook.getItemMetadata("/case/source.AD1", 99)?.md5Hash).toBe("abc123");
    dispose();
  });
});
