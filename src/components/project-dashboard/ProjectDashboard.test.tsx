import { describe, expect, it, vi, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../../__tests__/setup";
import { ProjectDashboard } from "./ProjectDashboardComponent";

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("ProjectDashboard", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("wires empty-state project actions without loading native DB stats in browser preview", async () => {
    const onOpenProject = vi.fn();
    const onNewProject = vi.fn();
    const { container, dispose } = renderComponent(() => (
      <ProjectDashboard
        project={() => null}
        discoveredFiles={() => []}
        fileHashMap={() => new Map()}
        bookmarkCount={() => 0}
        noteCount={() => 0}
        onOpenProject={onOpenProject}
        onNewProject={onNewProject}
      />
    ));

    const buttons = Array.from(container.querySelectorAll("button"));
    const newProjectButton = buttons.find((button) =>
      button.textContent?.includes("New Project"),
    );
    const openProjectButton = buttons.find((button) =>
      button.textContent?.includes("Open Project"),
    );

    expect(newProjectButton).toBeDefined();
    expect(openProjectButton).toBeDefined();

    newProjectButton!.click();
    openProjectButton!.click();

    expect(onNewProject).toHaveBeenCalledTimes(1);
    expect(onOpenProject).toHaveBeenCalledTimes(1);
    await Promise.resolve();
    expect(mockInvoke).not.toHaveBeenCalledWith("project_db_get_stats");
    dispose();
  });
});
