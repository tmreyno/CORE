import { describe, expect, it, vi } from "vitest";
import { open, save } from "@tauri-apps/plugin-dialog";
import { mockInvoke } from "../../../__tests__/setup";
import {
  exportEvidenceCollection,
  exportEvidenceCollectionPdf,
  importEvidenceCollectionPackage,
} from "./cocExport";
import { loadEvidenceCollectionById } from "./cocPersistence";

vi.mock("./cocPersistence", () => ({
  loadEvidenceCollectionById: vi.fn(),
}));

describe("cocExport browser guards", () => {
  it("does not open native save dialogs or load DB data for PDF export in browser preview", async () => {
    const result = await exportEvidenceCollectionPdf("collection-1", "1827-1001");

    expect(result).toBeNull();
    expect(loadEvidenceCollectionById).not.toHaveBeenCalled();
    expect(save).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not open native save dialogs or load DB data for collection package export in browser preview", async () => {
    const result = await exportEvidenceCollection("collection-1", "json", "1827-1001");

    expect(result).toBeNull();
    expect(loadEvidenceCollectionById).not.toHaveBeenCalled();
    expect(save).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not open native file dialogs or import packages in browser preview", async () => {
    const result = await importEvidenceCollectionPackage();

    expect(result).toBeNull();
    expect(open).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
