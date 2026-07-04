import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  getChildren,
  getContainerSummary,
  getLazyLoadSettings,
  getRootChildren,
  updateLazyLoadSettings,
  useLazyLoading,
} from "../useLazyLoading";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

const mockInvoke = vi.mocked(invoke);
const BROWSER_LAZY_LOADING_MESSAGE =
  "Evidence container lazy loading is available in the desktop app.";

function tick() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

describe("useLazyLoading browser runtime guards", () => {
  it("does not auto-load container data or settings outside Tauri", async () => {
    let dispose!: () => void;
    let lazy!: ReturnType<typeof useLazyLoading>;
    const onError = vi.fn();

    createRoot((d) => {
      dispose = d;
      lazy = useLazyLoading(() => "/case/evidence.ad1", {
        autoLoad: true,
        onError,
      });
    });

    await tick();
    await tick();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(lazy.summary()).toBeNull();
    expect(lazy.rootChildren()).toEqual([]);
    expect(lazy.error()?.message).toBe(BROWSER_LAZY_LOADING_MESSAGE);
    expect(onError).toHaveBeenCalledWith(expect.objectContaining({
      message: BROWSER_LAZY_LOADING_MESSAGE,
    }));
    dispose();
  });

  it("guards explicit hook lazy-load actions outside Tauri", async () => {
    let dispose!: () => void;
    let lazy!: ReturnType<typeof useLazyLoading>;

    createRoot((d) => {
      dispose = d;
      lazy = useLazyLoading(() => "/case/evidence.e01");
    });

    await expect(lazy.loadSummary()).resolves.toBeNull();
    await expect(lazy.loadRootChildren()).resolves.toBeNull();
    await expect(lazy.loadChildren("/")).resolves.toBeNull();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(lazy.error()?.message).toBe(BROWSER_LAZY_LOADING_MESSAGE);
    expect(lazy.isLoading()).toBe(false);
    dispose();
  });

  it("keeps hook settings local outside Tauri", async () => {
    let dispose!: () => void;
    let lazy!: ReturnType<typeof useLazyLoading>;

    createRoot((d) => {
      dispose = d;
      lazy = useLazyLoading(() => null, { batchSize: 25 });
    });

    await expect(lazy.refreshSettings()).resolves.toMatchObject({ batch_size: 25 });
    await expect(lazy.updateSettings({ batch_size: 50 })).resolves.toMatchObject({
      batch_size: 50,
    });

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(lazy.config().batch_size).toBe(50);
    dispose();
  });

  it("guards standalone lazy-loading helpers outside Tauri", async () => {
    await expect(getContainerSummary("/case/evidence.ad1")).rejects.toThrow(
      BROWSER_LAZY_LOADING_MESSAGE,
    );
    await expect(getRootChildren("/case/evidence.ad1")).rejects.toThrow(
      BROWSER_LAZY_LOADING_MESSAGE,
    );
    await expect(getChildren("/case/evidence.ad1", "/")).rejects.toThrow(
      BROWSER_LAZY_LOADING_MESSAGE,
    );
    await expect(getLazyLoadSettings()).rejects.toThrow(BROWSER_LAZY_LOADING_MESSAGE);
    await expect(updateLazyLoadSettings({ batchSize: 25 })).rejects.toThrow(
      BROWSER_LAZY_LOADING_MESSAGE,
    );

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
