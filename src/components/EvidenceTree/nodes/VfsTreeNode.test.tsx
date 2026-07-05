// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { PartitionNode } from "./VfsTreeNode";
import type { VfsPartitionInfo } from "../../../types";

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
});
