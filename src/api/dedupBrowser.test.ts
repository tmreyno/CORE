import { describe, expect, it, vi } from "vitest";
import {
  analyzeDuplicates,
  enrichWithHashes,
  exportDedupCsv,
  type DedupResults,
} from "./dedup";

const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

const emptyResults: DedupResults = {
  groups: [],
  stats: {
    totalFilesScanned: 0,
    totalDuplicateGroups: 0,
    totalDuplicateFiles: 0,
    totalWastedBytes: 0,
    uniqueFiles: 0,
    elapsedMs: 0,
  },
};

describe("dedup API in browser preview", () => {
  it("rejects native deduplication commands without invoking Tauri", async () => {
    await expect(analyzeDuplicates()).rejects.toThrow(
      "File deduplication analysis is available in the desktop app.",
    );
    await expect(enrichWithHashes(emptyResults, {})).rejects.toThrow(
      "File deduplication analysis is available in the desktop app.",
    );
    await expect(exportDedupCsv(emptyResults)).rejects.toThrow(
      "File deduplication analysis is available in the desktop app.",
    );

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
