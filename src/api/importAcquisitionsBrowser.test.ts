import { describe, expect, it } from "vitest";
import { mockInvoke } from "../__tests__/setup";

import { scanForAcquisitions } from "./importAcquisitions";

describe("scanForAcquisitions browser runtime guard", () => {
  it("does not invoke the desktop acquisition scanner outside Tauri", async () => {
    await expect(scanForAcquisitions("/cases/acquisitions")).rejects.toThrow(
      "Acquisition directory scanning is available in the desktop app.",
    );

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
