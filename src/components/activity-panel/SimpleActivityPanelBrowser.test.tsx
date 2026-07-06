import { describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../../__tests__/setup";
import { SimpleActivityPanelComponent } from "./SimpleActivityPanelComponent";
import type { Activity } from "../../types/activity";

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

describe("SimpleActivityPanel browser runtime guard", () => {
  it("does not invoke the native opener for finished activity destinations", () => {
    const activity: Activity = {
      id: "a1",
      type: "export",
      status: "completed",
      destination: "/case/Exports/report.zip",
      sourceCount: 1,
      startTime: new Date("2026-01-01T00:00:00Z"),
      endTime: new Date("2026-01-01T00:01:00Z"),
    };

    const { container, dispose } = renderComponent(() => (
      <SimpleActivityPanelComponent activities={[activity]} />
    ));

    const openButton = Array.from(container.querySelectorAll("button")).find(
      (button) => button.getAttribute("title") === "Open folder",
    );
    expect(openButton).toBeDefined();

    openButton!.click();

    expect(mockInvoke).not.toHaveBeenCalledWith(
      "plugin:opener|open_path",
      expect.anything(),
    );
    dispose();
  });
});
