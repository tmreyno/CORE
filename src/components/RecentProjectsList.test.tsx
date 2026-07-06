import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { RecentProjectsList } from "./RecentProjectsList";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

describe("RecentProjectsList", () => {
  let storage: Record<string, string>;

  beforeEach(() => {
    document.body.innerHTML = "";
    storage = {};
    vi.stubGlobal("localStorage", {
      getItem: vi.fn((key: string) => storage[key] ?? null),
      setItem: vi.fn((key: string, value: string) => {
        storage[key] = value;
      }),
    });
    localStorage.setItem("ffx-recent-projects", "[]");
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("filters recent items before rendering and opening", () => {
    localStorage.setItem(
      "ffx-recent-projects",
      JSON.stringify([
        {
          path: "/cases/full.cffx",
          name: "Full Project",
          lastOpened: "2026-07-04T10:00:00.000Z",
        },
        {
          path: "/cases/acquire.acquisition.json",
          name: "Acquire Session",
          lastOpened: "2026-07-04T09:00:00.000Z",
        },
      ]),
    );
    const onOpenProject = vi.fn();

    const { container, dispose } = renderComponent(() => (
      <RecentProjectsList
        onOpenProject={onOpenProject}
        pathFilter={(path) => path.toLowerCase().endsWith(".cffx")}
      />
    ));

    expect(container.textContent).toContain("Full Project");
    expect(container.textContent).not.toContain("Acquire Session");

    const projectRow = Array.from(container.querySelectorAll("[title]")).find(
      (element) => element.getAttribute("title") === "/cases/full.cffx",
    ) as HTMLElement | undefined;
    expect(projectRow).toBeDefined();

    projectRow!.click();
    expect(onOpenProject).toHaveBeenCalledWith("/cases/full.cffx");
    dispose();
  });

  it("opens recent projects with keyboard activation", () => {
    localStorage.setItem(
      "ffx-recent-projects",
      JSON.stringify([
        {
          path: "/cases/keyboard.cffx",
          name: "Keyboard Project",
          lastOpened: "2026-07-04T10:00:00.000Z",
        },
      ]),
    );
    const onOpenProject = vi.fn();

    const { container, dispose } = renderComponent(() => (
      <RecentProjectsList onOpenProject={onOpenProject} />
    ));

    const projectRow = container.querySelector(
      '[role="button"][title="/cases/keyboard.cffx"]',
    ) as HTMLElement | null;
    expect(projectRow).not.toBeNull();
    expect(projectRow!.tabIndex).toBe(0);

    projectRow!.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Enter", bubbles: true }),
    );
    projectRow!.dispatchEvent(
      new KeyboardEvent("keydown", { key: " ", bubbles: true }),
    );
    projectRow!.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
    );

    expect(onOpenProject).toHaveBeenCalledTimes(2);
    expect(onOpenProject).toHaveBeenNthCalledWith(1, "/cases/keyboard.cffx");
    expect(onOpenProject).toHaveBeenNthCalledWith(2, "/cases/keyboard.cffx");
    dispose();
  });
});
