// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";
import { ToastProvider } from "./Toast";
import { SystemIdentitySummaryPanel } from "./SystemIdentitySummaryPanel";
import type { DiscoveredFile } from "../types";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

const activeFile: DiscoveredFile = {
  path: "/evidence/disk.E01",
  filename: "disk.E01",
  container_type: "ewf",
  size: 1024,
};

async function renderLoadedPanel() {
  const result = renderComponent(() => (
    <ToastProvider>
      <SystemIdentitySummaryPanel
        activeFile={() => activeFile}
        hasProject={() => true}
      />
    </ToastProvider>
  ));

  await tick();
  await tick();
  return result;
}

describe("SystemIdentitySummaryPanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  it("does not query artifact tables in browser preview", async () => {
    const { container, dispose } = await renderLoadedPanel();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(container.textContent).toContain("System Identity");
    expect(container.textContent).toContain("No system identity artifacts found");
    expect(container.querySelector(`button[title="Copy system identity"]`)).toBeNull();
    dispose();
  });
});
