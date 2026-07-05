// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import { useFileManager } from "../useFileManager";
import type { ContainerInfo, DiscoveredFile } from "../../types";

vi.mock("../../utils/platform", () => ({
  isTauri: true,
}));

const file: DiscoveredFile = {
  path: "/evidence/source.ad1",
  filename: "source.ad1",
  container_type: "ad1",
  size: 1024,
};

const basicAd1Info: ContainerInfo = {
  container: "ad1",
  ad1: {
    segment: {} as never,
    logical: {} as never,
    item_count: 1,
  },
};

const fullAd1Info: ContainerInfo = {
  container: "ad1",
  ad1: {
    segment: {} as never,
    logical: {} as never,
    item_count: 1,
    tree: [],
  },
};

describe("useFileManager", () => {
  it("refreshes cached AD1 metadata when full tree details are requested", async () => {
    const fileManager = useFileManager();
    fileManager.setFileInfoMap(new Map([[file.path, basicAd1Info]]));
    mockInvoke.mockResolvedValueOnce(fullAd1Info);

    const result = await fileManager.loadFileInfo(file, true);

    expect(result).toBe(fullAd1Info);
    expect(mockInvoke).toHaveBeenCalledWith("logical_info", {
      inputPath: file.path,
      includeTree: true,
    });
    expect(fileManager.fileInfoMap().get(file.path)).toBe(fullAd1Info);
  });

  it("uses cached AD1 metadata for basic info requests", async () => {
    const fileManager = useFileManager();
    fileManager.setFileInfoMap(new Map([[file.path, basicAd1Info]]));

    const result = await fileManager.loadFileInfo(file, false);

    expect(result).toBe(basicAd1Info);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
