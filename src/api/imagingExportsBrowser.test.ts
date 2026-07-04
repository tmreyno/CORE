import { describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../__tests__/setup";
import { cancelAff4Export, createAff4Image } from "./aff4Export";
import {
  cancelE01Export,
  createE01Image,
  getEwfVersion,
  readEwfImageInfo,
} from "./ewfExport";
import { cancelL01Export, createL01Image, estimateL01Size } from "./l01Export";
import { cancelRawExport, createRawImage } from "./rawExport";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

const baseOptions = {
  sourcePaths: ["/case/source"],
  outputPath: "/case/output",
};

describe("imaging export API browser runtime guards", () => {
  it("rejects image creation before native progress listeners or commands", async () => {
    await expect(createE01Image(baseOptions, vi.fn())).rejects.toThrow("desktop app");
    await expect(createL01Image(baseOptions, vi.fn())).rejects.toThrow("desktop app");
    await expect(createRawImage(baseOptions, vi.fn())).rejects.toThrow("desktop app");
    await expect(createAff4Image(baseOptions, vi.fn())).rejects.toThrow("desktop app");

    expect(mockListen).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("returns safe cancel results without native commands", async () => {
    await expect(cancelE01Export("/case/output.E01")).resolves.toBe(false);
    await expect(cancelL01Export("/case/output.L01")).resolves.toBe(false);
    await expect(cancelRawExport("/case/output.raw")).resolves.toBe(false);
    await expect(cancelAff4Export("/case/output.aff4")).resolves.toBe(false);

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("guards supporting EWF/L01 metadata helpers", async () => {
    await expect(getEwfVersion()).resolves.toBe("desktop app required");
    await expect(readEwfImageInfo("/case/output.E01")).rejects.toThrow("desktop app");
    await expect(estimateL01Size(["/case/source"], "fast")).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
