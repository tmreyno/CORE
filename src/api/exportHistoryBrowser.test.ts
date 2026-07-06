import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../__tests__/setup";
import { deleteExportRecord, getExportHistory } from "./exportHistory";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("export history API browser runtime guards", () => {
  it("returns empty export history without native database commands", async () => {
    await expect(getExportHistory(20)).resolves.toEqual([]);

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("ignores delete requests without native database commands", async () => {
    await expect(deleteExportRecord("export-1")).resolves.toBeUndefined();

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
