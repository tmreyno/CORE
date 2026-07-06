import { describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

vi.mock("./preferences", () => ({
  getPreference: vi.fn(() => 50),
}));

import { HexViewer } from "./HexViewer";

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("HexViewer browser runtime guard", () => {
  it("does not invoke native hex analysis or byte reads in browser preview", async () => {
    const file = {
      filename: "evidence.bin",
      path: "/evidence/evidence.bin",
      size: 4096,
      container_type: "raw",
    };

    const { container, dispose } = renderComponent(() => (
      <HexViewer file={file} />
    ));

    await tick();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Evidence content viewing is available in the desktop app.",
    );
    dispose();
  });
});
