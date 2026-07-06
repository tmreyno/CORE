import { describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../__tests__/setup";
import {
  cancelCreation,
  clearLastArchiveError,
  createArchive,
  decryptDataNative,
  encryptDataNative,
  estimateSize,
  extractSplitArchive,
  getLastArchiveError,
  listenToProgress,
  listenToRepairProgress,
  listenToSplitExtractProgress,
  repairArchive,
  testArchive,
  validateArchive,
} from "./archiveCreate";
import { cancelExport, exportFiles } from "./fileExport";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("native export/archive API browser runtime guards", () => {
  it("rejects native export and archive operations before listener or command calls", async () => {
    await expect(exportFiles(["/case/source"], "/case/out", {}, vi.fn())).rejects.toThrow("desktop app");
    await expect(createArchive("/case/out.7z", ["/case/source"], {}, vi.fn())).rejects.toThrow("desktop app");
    await expect(testArchive("/case/out.7z")).rejects.toThrow("desktop app");
    await expect(estimateSize(["/case/source"])).rejects.toThrow("desktop app");
    await expect(repairArchive("/case/bad.7z", "/case/good.7z", vi.fn())).rejects.toThrow("desktop app");
    await expect(extractSplitArchive("/case/out.7z.001", "/case/out", undefined, vi.fn())).rejects.toThrow("desktop app");
    await expect(encryptDataNative(new Uint8Array([1, 2, 3]), "pw")).rejects.toThrow("desktop app");
    await expect(decryptDataNative(new Uint8Array([1, 2, 3]), "pw")).rejects.toThrow("desktop app");

    expect(mockListen).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("returns safe cancellation, validation, and error-state defaults", async () => {
    await expect(cancelExport("copy-1")).resolves.toBe(false);
    await expect(cancelCreation("/case/out.7z")).resolves.toBeUndefined();
    await expect(validateArchive("/case/out.7z")).resolves.toMatchObject({
      isValid: false,
      errorMessage: expect.stringContaining("desktop app"),
    });
    await expect(getLastArchiveError()).resolves.toBeNull();
    await expect(clearLastArchiveError()).resolves.toBeUndefined();

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not register native archive progress listeners in browser preview", async () => {
    const onProgress = vi.fn();
    const unlistenCreate = await listenToProgress(onProgress, "/case/out.7z");
    const unlistenRepair = await listenToRepairProgress(onProgress);
    const unlistenExtract = await listenToSplitExtractProgress(onProgress);

    unlistenCreate();
    unlistenRepair();
    unlistenExtract();

    expect(onProgress).not.toHaveBeenCalled();
    expect(mockListen).not.toHaveBeenCalled();
  });
});
