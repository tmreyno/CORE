// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { OfficeViewer } from "./OfficeViewer";
import { mockInvoke } from "../__tests__/setup";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

const tick = (ms = 50) => new Promise((resolve) => setTimeout(resolve, ms));

describe("OfficeViewer browser runtime guard", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  it("shows a desktop-only message without invoking native Office extraction", async () => {
    const { container, dispose } = renderComponent(() => (
      <OfficeViewer path="/tmp/report.docx" />
    ));
    await tick();

    expect(container.textContent).toContain("Office document extraction is available in the desktop app.");
    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });

  it("does not invoke native Office extraction for container entry sources in browser preview", async () => {
    const source = {
      containerPath: "/evidence/case.ad1",
      entryPath: "docs/report.docx",
      containerType: "ad1",
      size: 4096,
    };

    const { container, dispose } = renderComponent(() => (
      <OfficeViewer path="/tmp/report.docx" source={source} />
    ));
    await tick();

    expect(container.textContent).toContain("desktop app");
    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });
});
