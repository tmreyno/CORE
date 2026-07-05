// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { beforeEach, describe, expect, it, vi } from "vitest";
import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { createSearchHandlers } from "../useAppActions";
import type { DiscoveredFile } from "../../types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/logger", () => ({
  logger: {
    scope: () => ({
      debug: vi.fn(),
      warn: vi.fn(),
      info: vi.fn(),
      error: vi.fn(),
    }),
  },
}));

vi.mock("../../utils/accessibility", () => ({
  announce: vi.fn(),
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

vi.mock("../../api/search", () => ({
  searchQuery: vi.fn().mockRejectedValue(new Error("Search index unavailable")),
}));

const mockInvoke = vi.mocked(invoke);

const makeFile = (path: string): DiscoveredFile => ({
  path,
  filename: path.split("/").pop() || path,
  size: 1024,
  container_type: "ad1",
});

function mockFileManager(files: DiscoveredFile[]) {
  const [activeFile, setActiveFile] = createSignal<DiscoveredFile | null>(null);
  return {
    discoveredFiles: () => files,
    activeFile,
    setActiveFile,
  };
}

const mockProjectManager = () => ({
  project: () => ({ name: "Browser Project" }),
});

describe("createSearchHandlers browser runtime guards", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("uses in-memory search and skips project DB FTS commands outside Tauri", async () => {
    const fileManager = mockFileManager([
      makeFile("/evidence/case/photo.jpg"),
      makeFile("/evidence/case/report.docx"),
    ]);
    const { handleSearch } = createSearchHandlers({
      fileManager: fileManager as any,
      projectManager: mockProjectManager() as any,
    });

    const results = await handleSearch("report", {} as any);

    expect(results).toEqual([
      expect.objectContaining({
        path: "/evidence/case/report.docx",
        name: "report.docx",
        matchType: "name",
      }),
    ]);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
