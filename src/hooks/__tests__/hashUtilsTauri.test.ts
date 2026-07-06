import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import { extractAd1StoredHashes } from "../hashUtils";

vi.mock("../../utils/platform", () => ({
  isTauri: true,
}));

describe("extractAd1StoredHashes", () => {
  it("requests AD1 metadata with the logical_info inputPath parameter", async () => {
    mockInvoke.mockResolvedValueOnce({
      stored_hash: {
        algorithm: "SHA-256",
        hash: "abc123",
      },
    });

    await expect(extractAd1StoredHashes("/case/evidence.AD1")).resolves.toEqual([
      {
        algorithm: "SHA256",
        hash: "ABC123",
        source: "container",
      },
    ]);

    expect(mockInvoke).toHaveBeenCalledWith("logical_info", {
      inputPath: "/case/evidence.AD1",
      includeTree: false,
    });
  });
});
