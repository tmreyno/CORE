// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi } from "vitest";
import { handleLoadProject, restoreCenterTabs } from "../projectLoader";
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

  it("shows browser file picker and cancel feedback when Open Project has no selection", async () => {
    const setScanDir = vi.fn();
    const toast = {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    };

    await handleLoadProject({
      fileManager: {
        scanDir: () => "/previous/evidence",
        setScanDir,
      },
      hashManager: {},
      projectManager: {
        loadProject: vi.fn().mockResolvedValue({ project: null, error: "Open cancelled" }),
      },
      processedDbManager: {},
      setLeftWidth: vi.fn(),
      setRightWidth: vi.fn(),
      setLeftCollapsed: vi.fn(),
      setRightCollapsed: vi.fn(),
      setLeftPanelTab: vi.fn(),
      setCurrentViewMode: vi.fn(),
      setEntryContentViewMode: vi.fn(),
      setCaseDocumentsPath: vi.fn(),
      setTreeExpansionState: vi.fn(),
      setSelectedContainerEntry: vi.fn(),
      setOpenTabs: vi.fn(),
      setCaseDocuments: vi.fn(),
      setCenterTabs: vi.fn(),
      setActiveTabId: vi.fn(),
      setCenterViewMode: vi.fn(),
      toast,
    } as any);

    expect(setScanDir).toHaveBeenCalledWith("");
    expect(setScanDir).toHaveBeenCalledWith("/previous/evidence");
    expect(toast.info).toHaveBeenCalledWith(
      "Open Project",
      "Choose a .cffx project file in the browser file picker.",
    );
    expect(toast.info).toHaveBeenCalledWith(
      "Open Cancelled",
      "No project file was selected. Use Open Project and choose a .cffx file.",
    );
  });
});
