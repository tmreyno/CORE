import { describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

vi.mock("./document/documentHelpers", () => ({
  getFormatIcon: () => "DOC",
  performSearch: vi.fn(() => 0),
  printDocument: vi.fn(),
  downloadHtml: vi.fn(),
}));

import { DocumentViewer } from "./DocumentViewer";

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("DocumentViewer browser runtime guard", () => {
  it("does not invoke native document reads in browser preview", async () => {
    const { container, dispose } = renderComponent(() => (
      <DocumentViewer path="/evidence/report.pdf" />
    ));

    await tick();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(container.textContent).toContain("Failed to load document");
    expect(container.textContent).toContain(
      "Document evidence viewing is available in the desktop app.",
    );
    dispose();
  });
});
