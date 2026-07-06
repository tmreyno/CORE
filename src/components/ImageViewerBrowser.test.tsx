import { describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

import { ImageViewer } from "./ImageViewer";

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("ImageViewer browser runtime guard", () => {
  it("does not invoke native image reads in browser preview", async () => {
    const { container, dispose } = renderComponent(() => (
      <ImageViewer path="/evidence/photo.jpg" />
    ));

    await tick();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(container.textContent).toContain("Failed to load image");
    expect(container.textContent).toContain(
      "Image evidence viewing is available in the desktop app.",
    );
    dispose();
  });
});
