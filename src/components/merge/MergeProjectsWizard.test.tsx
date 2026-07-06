import { describe, expect, it, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { open } from "@tauri-apps/plugin-dialog";
import MergeProjectsWizard from "./MergeProjectsWizard";

vi.mock("../../api/projectMerge", () => ({
  analyzeProjects: vi.fn(),
  executeMerge: vi.fn(),
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("MergeProjectsWizard", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("shows a browser-preview message instead of opening a native project picker", () => {
    const { container, dispose } = renderComponent(() => (
      <MergeProjectsWizard onClose={vi.fn()} onMergeComplete={vi.fn()} />
    ));

    const addProjects = Array.from(container.querySelectorAll("button")).find((button) =>
      button.textContent?.includes("Add Project Files"),
    );
    expect(addProjects).toBeDefined();
    addProjects!.click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Project file selection is available in the desktop app.",
    );
    dispose();
  });
});
