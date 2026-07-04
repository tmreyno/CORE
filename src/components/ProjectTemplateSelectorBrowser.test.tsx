import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";
import { ProjectTemplateSelector } from "./ProjectTemplateSelector";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("ProjectTemplateSelector in browser preview", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("does not call the native template API", async () => {
    const onSelect = vi.fn();
    const { container, dispose } = renderComponent(() => (
      <ProjectTemplateSelector selectedId={null} onSelect={onSelect} />
    ));

    await Promise.resolve();

    expect(mockInvoke).not.toHaveBeenCalledWith("template_list");

    const skipButton = Array.from(container.querySelectorAll("button")).find((button) =>
      button.textContent?.includes("Start from Scratch"),
    );
    expect(skipButton).toBeDefined();

    skipButton!.click();
    expect(onSelect).toHaveBeenCalledWith(null);
    dispose();
  });
});
