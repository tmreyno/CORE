import { describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { CenterPaneComponent } from "./CenterPaneComponent";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("CenterPaneComponent", () => {
  it("wires no-project New Project and Open Project actions", () => {
    const onNewProject = vi.fn();
    const onOpenProject = vi.fn();
    const { container, dispose } = renderComponent(() => (
      <CenterPaneComponent
        tabs={() => []}
        activeTabId={() => null}
        onTabSelect={vi.fn()}
        onTabClose={vi.fn()}
        onTabsChange={vi.fn()}
        viewMode={() => "info"}
        onViewModeChange={vi.fn()}
        onNewProject={onNewProject}
        onOpenProject={onOpenProject}
      >
        <div />
      </CenterPaneComponent>
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
    dispose();
  });
});
