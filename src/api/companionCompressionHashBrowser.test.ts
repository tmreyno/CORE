import { describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../__tests__/setup";
import { findCompanionFile, readCompanionFile, writeCompanionFile } from "./companion";
import {
  compressToLzma,
  compressToLzma2,
  decompressLzma,
  decompressLzma2,
} from "./lzmaApi";
import { hashContainerSegments, listenSegmentHashProgress } from "./segmentHash";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("companion/compression/segment hash browser runtime guards", () => {
  it("guards companion sidecar file operations", async () => {
    await expect(findCompanionFile("/case/output.E01")).resolves.toBeNull();
    await expect(readCompanionFile("/case/output.ffx-companion.json")).rejects.toThrow("desktop app");
    await expect(
      writeCompanionFile("/case/output.E01", {
        acquisitionType: "e01",
        source: { paths: ["/case/source"], totalFiles: 1, totalBytes: 10 },
        output: { primaryPath: "/case/output.E01", format: "e01", totalBytes: 10 },
        timing: {
          startedAt: "2026-07-04T00:00:00.000Z",
          completedAt: "2026-07-04T00:00:01.000Z",
          durationMs: 1000,
        },
      }),
    ).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("guards LZMA compression and decompression operations", async () => {
    await expect(compressToLzma("/case/in", "/case/out.lzma")).rejects.toThrow("desktop app");
    await expect(decompressLzma("/case/out.lzma", "/case/out")).rejects.toThrow("desktop app");
    await expect(compressToLzma2("/case/in", "/case/out.xz")).rejects.toThrow("desktop app");
    await expect(decompressLzma2("/case/out.xz", "/case/out")).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("guards segment hashing and progress listeners", async () => {
    const onProgress = vi.fn();
    const unlisten = await listenSegmentHashProgress(onProgress);

    await expect(hashContainerSegments("/case/evidence.E01", "SHA-256", true, true)).rejects.toThrow("desktop app");
    unlisten();

    expect(onProgress).not.toHaveBeenCalled();
    expect(mockListen).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
