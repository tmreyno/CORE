import { describe, expect, it, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { save } from "@tauri-apps/plugin-dialog";
import { ActivityPanel } from "./ActivityPanel";
import { createEmptyProject } from "../../types/project";
import { ToastProvider } from "../Toast";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function buttonByText(container: HTMLElement, text: string): HTMLButtonElement {
  const button = Array.from(container.querySelectorAll("button")).find((candidate) =>
    candidate.textContent?.includes(text),
  );
  expect(button).toBeDefined();
  return button as HTMLButtonElement;
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("ActivityPanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("shows a browser-preview message instead of opening a native activity export dialog", async () => {
    const project = createEmptyProject("/cases/1827-1001", "examiner", "0.1.112", "1827-1001");
    const { container, dispose } = renderComponent(() => (
      <ToastProvider>
        <ActivityPanel project={project} />
      </ToastProvider>
    ));

    buttonByText(container, "Export CSV").click();
    await tick();

    expect(save).not.toHaveBeenCalled();
    expect(document.body.textContent).toContain(
      "Activity export file saving is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening a native timeline export dialog", async () => {
    const project = createEmptyProject("/cases/1827-1001", "examiner", "0.1.112", "1827-1001");
    const { container, dispose } = renderComponent(() => (
      <ToastProvider>
        <ActivityPanel project={project} />
      </ToastProvider>
    ));

    buttonByText(container, "Timeline JSON").click();
    await tick();

    expect(save).not.toHaveBeenCalled();
    expect(document.body.textContent).toContain(
      "Activity export file saving is available in the desktop app.",
    );
    dispose();
  });
});
