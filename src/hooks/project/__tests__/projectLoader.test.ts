// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
import { restoreCenterTabs } from "../projectLoader";
import type { DiscoveredFile } from "../../../types";
import type { ProjectTab } from "../../../types/project";

function makeFile(overrides: Partial<DiscoveredFile> = {}): DiscoveredFile {
  return {
    path: "./1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
    filename: "4Dell Latitude CPi.E01",
    container_type: "EnCase (E01)",
    size: 671_094_597,
    segment_count: 1,
    ...overrides,
  };
}

describe("projectLoader", () => {
  it("restores legacy entry tabs from file_path plus entry_path", () => {
    const tabs: ProjectTab[] = [
      {
        id: "entry:/Partition1_NTFS/pagefile.sys",
        type: "entry",
        file_path: "./1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
        entry_path: "/Partition1_NTFS/pagefile.sys",
        name: "pagefile.sys",
        subtitle: "4Dell Latitude CPi.E01",
        order: 0,
      },
    ];

    const restored = restoreCenterTabs(tabs, [makeFile()], [], []);

    expect(restored).toHaveLength(1);
    expect(restored[0]).toMatchObject({
      id: "entry:/Partition1_NTFS/pagefile.sys",
      type: "entry",
      title: "pagefile.sys",
      subtitle: "4Dell Latitude CPi.E01",
    });
    expect(restored[0].entry).toMatchObject({
      containerPath: "./1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
      entryPath: "/Partition1_NTFS/pagefile.sys",
      name: "pagefile.sys",
      isVfsEntry: true,
      isArchiveEntry: false,
      containerType: "EnCase (E01)",
    });
  });
});
