import { describe, expect, it, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { open } from "@tauri-apps/plugin-dialog";
import DriveSourcePanel from "./DriveSourcePanel";
import { listDrives } from "../../api/drives";

vi.mock("../../api/drives", () => ({
  formatDriveSize: (bytes: number) => `${bytes} B`,
  listDrives: vi.fn(() => Promise.resolve([])),
}));

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

describe("DriveSourcePanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("shows a browser-preview message instead of opening a native file picker", () => {
    const { container, dispose } = renderComponent(() => (
      <DriveSourcePanel onExportSources={vi.fn()} />
    ));

    toolbarButton(container, "Add files").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Drive source file and folder selection is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening a native folder picker", () => {
    const { container, dispose } = renderComponent(() => (
      <DriveSourcePanel onExportSources={vi.fn()} />
    ));

    toolbarButton(container, "Add folder").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Drive source file and folder selection is available in the desktop app.",
    );
    dispose();
  });

  it("does not scan drives through Tauri while running in browser preview", () => {
    const { dispose } = renderComponent(() => (
      <DriveSourcePanel onExportSources={vi.fn()} />
    ));

    expect(listDrives).not.toHaveBeenCalled();
    dispose();
  });
});
