import { describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";
import { ExifPanel } from "./ExifPanel";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("ExifPanel browser runtime guard", () => {
  it("does not invoke native EXIF extraction in browser preview", async () => {
    const onMetadata = vi.fn();
    const { container, dispose } = renderComponent(() => (
      <ExifPanel path="/evidence/photo.jpg" onMetadata={onMetadata} />
    ));

    await tick();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(onMetadata).not.toHaveBeenCalled();
    expect(container.textContent).toContain("No EXIF data available");
    dispose();
  });
});
