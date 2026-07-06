// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { render } from "solid-js/web";
import { describe, expect, it, beforeEach, vi } from "vitest";
import { RightPanel } from "./RightPanel";
import { ToastProvider } from "../Toast";
import type { ViewerMetadata } from "../../types/viewerMetadata";

vi.mock("../../utils/platform", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../utils/platform")>()),
  isTauri: false,
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

const activeFile = {
  path: "/evidence/disk.E01",
  filename: "disk.E01",
  container_type: "ewf",
  size: 1024,
};

const viewerMetadata: ViewerMetadata = {
  fileInfo: {
    name: "SOFTWARE",
    path: "/Windows/System32/config/SOFTWARE",
    size: 65536,
    containerPath: activeFile.path,
    containerType: "e01",
  },
  viewerType: "Registry",
  sections: [],
};

describe("RightPanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("keeps system identity visible while showing viewer metadata for open entries", () => {
    const { container, dispose } = renderComponent(() => (
      <ToastProvider>
        <RightPanel
          collapsed={() => false}
          width={() => 360}
          currentViewMode={() => "info" as any}
          setRequestViewMode={() => undefined}
          hexMetadata={() => null}
          hexNavigator={() => null}
          activeFile={() => activeFile as any}
          activeFileInfo={() => undefined}
          selectedEntry={() => null}
          viewerMetadata={() => viewerMetadata}
          activeTabType={() => "entry" as any}
          linkedDataNodes={() => []}
          hasProject={() => true}
          activities={() => []}
        />
      </ToastProvider>
    ));

    expect(container.textContent).toContain("System Identity");
    expect(container.textContent).toContain("No system identity artifacts found");
    expect(container.textContent).toContain("File Info");
    expect(container.textContent).toContain("SOFTWARE");
    dispose();
  });
});
