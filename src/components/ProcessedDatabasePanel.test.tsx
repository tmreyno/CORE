import { describe, expect, it, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { open } from "@tauri-apps/plugin-dialog";
import { ProcessedDatabasePanel } from "./ProcessedDatabasePanel";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function toolbarButton(container: HTMLElement, title: string): HTMLButtonElement {
  const button = container.querySelector(`button[title="${title}"]`);
  expect(button).toBeDefined();
  return button as HTMLButtonElement;
}

describe("ProcessedDatabasePanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("shows a browser-preview message instead of opening a scan folder dialog", () => {
    const { container, dispose } = renderComponent(() => <ProcessedDatabasePanel />);

    toolbarButton(container, "Scan folder for databases").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Processed database file and folder selection is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening an add database dialog", () => {
    const { container, dispose } = renderComponent(() => <ProcessedDatabasePanel />);

    toolbarButton(container, "Add database file").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Processed database file and folder selection is available in the desktop app.",
    );
    dispose();
  });
});
