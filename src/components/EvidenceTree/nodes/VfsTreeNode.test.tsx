// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { PartitionNode, VfsTreeNode } from "./VfsTreeNode";
import type { VfsEntry, VfsPartitionInfo } from "../../../types";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function makePartition(overrides: Partial<VfsPartitionInfo> = {}): VfsPartitionInfo {
  return {
    number: 1,
    mountName: "Partition1_NTFS",
    fsType: "NTFS",
    size: 4096,
    startOffset: 0,
    ...overrides,
  };
}

function makeVfsEntry(overrides: Partial<VfsEntry> = {}): VfsEntry {
  return {
    name: "file.txt",
    path: "/Partition1_NTFS/file.txt",
    isDir: false,
    size: 12,
    fileType: "text",
    ...overrides,
  };
}

function makeManyEntries(count: number, parent = "/Partition1_NTFS"): VfsEntry[] {
  return Array.from({ length: count }, (_, index) =>
    makeVfsEntry({
      name: `file-${index.toString().padStart(3, "0")}.txt`,
      path: `${parent}/file-${index.toString().padStart(3, "0")}.txt`,
    }),
  );
}

function clickLoadMore(container: HTMLElement): void {
  const loadMore = Array.from(container.querySelectorAll<HTMLElement>(".cursor-pointer")).find((element) =>
    element.textContent?.includes("Load more"),
  );

  expect(loadMore).toBeTruthy();
  loadMore!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
}

describe("PartitionNode", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("uses backend-provided rootPath when toggling physical-mode images", () => {
    const onToggle = vi.fn().mockResolvedValue(undefined);
    const partition = makePartition({
      mountName: "Physical Image",
      fsType: "Raw sectors",
      rootPath: "/",
    });

    const { container, dispose } = renderComponent(() => (
      <PartitionNode
        partition={partition}
        containerPath="/case/disk.E01"
        index={0}
        isExpanded={() => false}
        isLoading={() => false}
        isSelected={() => false}
        getChildren={() => []}
        onToggle={onToggle}
        onEntryClick={vi.fn()}
      />
    ));

    container
      .querySelector('[role="treeitem"]')!
      .dispatchEvent(new MouseEvent("click", { bubbles: true }));

    expect(onToggle).toHaveBeenCalledWith("/case/disk.E01", "/");
    dispose();
  });

  it("caps partition child rendering and loads more on demand", () => {
    const entries = makeManyEntries(250);

    const { container, dispose } = renderComponent(() => (
      <PartitionNode
        partition={makePartition()}
        containerPath="/case/disk.E01"
        index={0}
        isExpanded={() => true}
        isLoading={() => false}
        isSelected={() => false}
        getChildren={() => entries}
        onToggle={vi.fn()}
        onEntryClick={vi.fn()}
      />
    ));

    expect(container.querySelectorAll("[data-entry-path]").length).toBe(200);
    expect(container.textContent).toContain("Load more (200 of 250)");

    clickLoadMore(container);

    expect(container.querySelectorAll("[data-entry-path]").length).toBe(250);
    dispose();
  });
});

describe("VfsTreeNode", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("caps expanded directory child rendering and loads more on demand", () => {
    const entry = makeVfsEntry({
      name: "System32",
      path: "/Partition1_NTFS/Windows/System32",
      isDir: true,
    });
    const entries = makeManyEntries(250, entry.path);

    const { container, dispose } = renderComponent(() => (
      <VfsTreeNode
        entry={entry}
        containerPath="/case/disk.E01"
        depth={1}
        partitionIndex={0}
        isExpanded={() => true}
        isLoading={() => false}
        isSelected={() => false}
        getChildren={() => entries}
        onToggle={vi.fn()}
        onClick={vi.fn()}
      />
    ));

    expect(container.querySelectorAll("[data-entry-path]").length).toBe(201);
    expect(container.textContent).toContain("Load more (200 of 250)");

    clickLoadMore(container);

    expect(container.querySelectorAll("[data-entry-path]").length).toBe(251);
    dispose();
  });
});
