import { describe, expect, it, vi } from "vitest";
import type { SelectedEntry } from "../../components/EvidenceTree/types";
import { readBytesFromSource, readTextFromSource } from "../useEntrySource";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

const mockInvoke = vi.mocked(invoke);

function makeEntry(): SelectedEntry {
  return {
    containerPath: "/evidence/container.ad1",
    entryPath: "/files/test.bin",
    name: "test.bin",
    size: 2048,
    isDir: false,
    isArchiveEntry: false,
    isVfsEntry: false,
    isDiskFile: false,
  };
}

describe("useEntrySource browser runtime guards", () => {
  it("does not invoke native byte readers outside Tauri", async () => {
    await expect(readBytesFromSource(null, makeEntry(), 0, 256)).rejects.toThrow(
      "Evidence content viewing is available in the desktop app.",
    );
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not invoke native text readers outside Tauri", async () => {
    await expect(readTextFromSource(null, makeEntry(), 0, 256)).rejects.toThrow(
      "Evidence content viewing is available in the desktop app.",
    );
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
