import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import {
  deleteEvidenceCollection,
  loadAllEvidenceCollections,
  loadCocItemsFromDb,
  loadEvidenceCollectionById,
  loadEvidenceCollectionFromDb,
  persistCocItemsToDb,
  persistEvidenceCollectionToDb,
  updateEvidenceCollectionStatus,
} from "./cocPersistence";

vi.mock("../../../utils/platform", () => ({
  isTauri: false,
}));

describe("COC persistence browser runtime guards", () => {
  it("returns empty read models without native project DB commands", async () => {
    await expect(loadCocItemsFromDb("1827-1001")).resolves.toEqual([]);
    await expect(loadEvidenceCollectionFromDb("1827-1001")).resolves.toBeNull();
    await expect(loadEvidenceCollectionById("collection-1")).resolves.toBeNull();
    await expect(loadAllEvidenceCollections("1827-1001")).resolves.toEqual([]);

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("rejects required persistence writes before native project DB commands", async () => {
    await expect(persistCocItemsToDb([])).rejects.toThrow("desktop app");
    await expect(
      persistEvidenceCollectionToDb(
        {
          collection_date: "",
          collecting_officer: "",
          authorization: "",
          witnesses: [],
          collected_items: [],
          documentation_notes: "",
        },
        "collection-1",
      ),
    ).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("returns false for status and delete mutations without native commands", async () => {
    await expect(updateEvidenceCollectionStatus("collection-1", "complete")).resolves.toBe(false);
    await expect(deleteEvidenceCollection("collection-1")).resolves.toBe(false);

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
