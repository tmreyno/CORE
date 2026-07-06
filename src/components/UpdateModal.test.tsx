// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { afterEach, describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import UpdateModal from "./UpdateModal";

const updaterMocks = vi.hoisted(() => {
  const checkForUpdates = vi.fn(() => Promise.resolve());
  const downloadAndInstall = vi.fn(() => Promise.resolve());
  const handleRelaunch = vi.fn(() => Promise.resolve());

  return {
    checkForUpdates,
    updater: {
      state: () => "available",
      errorMessage: () => "",
      progress: () => 0,
      downloadedBytes: () => 0,
      totalBytes: () => 0,
      updateVersion: () => "0.1.113",
      updateBody: () => "",
      updateDate: () => "2026-07-06T00:00:00.000Z",
      currentVersion: () => "0.1.111",
      checkForUpdates,
      downloadAndInstall,
      handleRelaunch,
    },
  };
});

vi.mock("@core-suite/components/updater", () => ({
  useUpdater: () => updaterMocks.updater,
  UpdateModal: (props: any) => (
    <div data-testid="shared-update-modal">
      {props.updater.updateBody()}
    </div>
  ),
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("UpdateModal", () => {
  afterEach(() => {
    document.body.innerHTML = "";
    updaterMocks.checkForUpdates.mockClear();
  });

  it("passes bundled release notes into the shared updater modal", async () => {
    const { container, dispose } = renderComponent(() => (
      <UpdateModal show={true} onClose={() => undefined} />
    ));

    await Promise.resolve();

    expect(updaterMocks.checkForUpdates).toHaveBeenCalledTimes(1);
    expect(container.textContent).toContain("CORE-FFX 0.1.113");
    expect(container.textContent).toContain("CORE-FFX 0.1.112");
    expect(container.textContent).toContain("Project open repair");

    dispose();
  });
});
