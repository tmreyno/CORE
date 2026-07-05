import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { render } from "solid-js/web";
import { WelcomeModal } from "./WelcomeModal";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function buttonByText(text: string): HTMLButtonElement {
  const button = Array.from(document.body.querySelectorAll("button")).find(
    (candidate) => candidate.textContent?.includes(text),
  );
  expect(button).toBeDefined();
  return button as HTMLButtonElement;
}

describe("WelcomeModal project actions", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  afterEach(() => {
    document.body.innerHTML = "";
    vi.restoreAllMocks();
  });

  it("runs New Project before closing so the click gesture is preserved", () => {
    const calls: string[] = [];
    const onNewProject = vi.fn(() => calls.push("new"));
    const onClose = vi.fn(() => calls.push("close"));
    const { dispose } = renderComponent(() => (
      <WelcomeModal
        isOpen
        onClose={onClose}
        onStartTour={vi.fn()}
        onNewProject={onNewProject}
      />
    ));

    buttonByText("New Project").click();

    expect(onNewProject).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(calls).toEqual(["new", "close"]);
    dispose();
  });

  it("runs Open Project before closing so file pickers can open from the user click", () => {
    const calls: string[] = [];
    const onOpenProject = vi.fn(() => calls.push("open"));
    const onClose = vi.fn(() => calls.push("close"));
    const { dispose } = renderComponent(() => (
      <WelcomeModal
        isOpen
        onClose={onClose}
        onStartTour={vi.fn()}
        onOpenProject={onOpenProject}
      />
    ));

    buttonByText("Open Project").click();

    expect(onOpenProject).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(calls).toEqual(["open", "close"]);
    dispose();
  });

  it("selects a recent project before closing the welcome modal", () => {
    const calls: string[] = [];
    const onSelectRecentProject = vi.fn((path: string) =>
      calls.push(`recent:${path}`),
    );
    const onClose = vi.fn(() => calls.push("close"));
    const { dispose } = renderComponent(() => (
      <WelcomeModal
        isOpen
        onClose={onClose}
        onStartTour={vi.fn()}
        recentProjects={() => [
          {
            path: "/cases/seed.cffx",
            name: "Seed Case",
            lastOpened: new Date().toISOString(),
          },
        ]}
        onSelectRecentProject={onSelectRecentProject}
      />
    ));

    buttonByText("Seed Case").click();

    expect(onSelectRecentProject).toHaveBeenCalledWith("/cases/seed.cffx");
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(calls).toEqual(["recent:/cases/seed.cffx", "close"]);
    dispose();
  });
});
