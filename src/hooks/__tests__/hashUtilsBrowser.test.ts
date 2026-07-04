import { describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../../__tests__/setup";
import {
  extractStoredHashes,
  hashContainer,
  setupProgressListener,
} from "../hashUtils";

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

describe("hashUtils browser runtime guards", () => {
  it("returns no stored hashes without native metadata commands", async () => {
    await expect(extractStoredHashes("/case/evidence.E01", "e01")).resolves.toEqual([]);
    await expect(extractStoredHashes("/case/evidence.AD1", "ad1")).resolves.toEqual([]);
    await expect(extractStoredHashes("/case/evidence.ufed", "ufed")).resolves.toEqual([]);

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not register native hash progress listeners in browser preview", async () => {
    const onProgress = vi.fn();
    const unlisten = await setupProgressListener("hash-progress", onProgress);

    unlisten();

    expect(onProgress).not.toHaveBeenCalled();
    expect(mockListen).not.toHaveBeenCalled();
  });

  it("rejects container hashing before invoking native code", async () => {
    await expect(hashContainer("/case/evidence.E01", "e01", "MD5")).rejects.toThrow("desktop app");
    await expect(hashContainer("/case/evidence.AD1", "ad1", "SHA-256")).rejects.toThrow("desktop app");
    await expect(hashContainer("/case/evidence.raw", "raw", "SHA-1")).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
