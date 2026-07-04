import { describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../__tests__/setup";
import {
  closeSearchIndex,
  deleteSearchIndex,
  getSearchIndexStats,
  indexAllContainers,
  indexContainer,
  onIndexProgress,
  openSearchIndex,
  rebuildSearchIndex,
  searchQuery,
} from "./search";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("search API browser runtime guards", () => {
  it("returns empty/default values without invoking native search commands", async () => {
    await expect(openSearchIndex("/case/project.ffxdb")).resolves.toMatchObject({
      numDocs: 0,
      categoryCounts: [],
    });
    await expect(getSearchIndexStats()).resolves.toMatchObject({ numDocs: 0 });
    await expect(searchQuery({ query: "evidence" })).resolves.toEqual({
      hits: [],
      totalHits: 0,
      elapsedMs: 0,
      categoryCounts: [],
      containerTypeCounts: [],
    });

    await closeSearchIndex();
    await deleteSearchIndex("/case/project.ffxdb");
    await indexContainer("/case/disk.E01");
    await indexAllContainers(["/case/disk.E01"]);
    await rebuildSearchIndex(["/case/disk.E01"]);
    const unlisten = await onIndexProgress(vi.fn());
    unlisten();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(mockListen).not.toHaveBeenCalled();
  });
});
