import { describe, expect, it, vi } from "vitest";
import { buildLinkedDataTree } from "./linkedDataBuilder";

const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

describe("buildLinkedDataTree in browser preview", () => {
  it("returns an empty tree without querying the project database", async () => {
    await expect(buildLinkedDataTree("collection-1", "CASE-1")).resolves.toEqual([]);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
