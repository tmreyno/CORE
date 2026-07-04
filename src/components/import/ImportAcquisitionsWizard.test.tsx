import { describe, expect, it, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { open } from "@tauri-apps/plugin-dialog";
import { ImportAcquisitionsWizard } from "./ImportAcquisitionsWizard";

vi.mock("../../hooks/useImportAcquisitions", () => ({
  useImportAcquisitions: () => ({
    scanning: () => false,
    results: () => [],
    selected: () => new Set<string>(),
    importing: () => false,
    importResult: () => null,
    error: () => null,
    scan: vi.fn(),
    toggleSelect: vi.fn(),
    selectAll: vi.fn(),
    deselectAll: vi.fn(),
    importSelected: vi.fn(),
    reset: vi.fn(),
  }),
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("ImportAcquisitionsWizard", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("shows a browser-preview message instead of opening a native directory picker", () => {
    const { container, dispose } = renderComponent(() => (
      <ImportAcquisitionsWizard onClose={vi.fn()} />
    ));

    const browse = Array.from(container.querySelectorAll("button")).find((button) =>
      button.textContent?.includes("Browse"),
    );
    expect(browse).toBeDefined();
    browse!.click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Acquisition directory browsing is available in the desktop app.",
    );
    dispose();
  });
});
