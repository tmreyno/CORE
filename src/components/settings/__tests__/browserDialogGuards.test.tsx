import { describe, expect, it, vi, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { open } from "@tauri-apps/plugin-dialog";
import { PathsSettings } from "../PathsTab";
import { ReportsSettings } from "../ReportsTab";
import { UserProfilesSettings } from "../UserProfilesTab";
import { DEFAULT_PREFERENCES, type AppPreferences } from "../../preferences";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function preferences(overrides: Partial<AppPreferences> = {}): AppPreferences {
  return {
    ...DEFAULT_PREFERENCES,
    ...overrides,
  };
}

function clickButton(container: HTMLElement, text: string) {
  const button = Array.from(container.querySelectorAll("button")).find((item) =>
    item.textContent?.includes(text),
  );
  expect(button).toBeDefined();
  button!.click();
}

describe("settings browser dialog guards", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("shows a browser-preview message instead of opening native path dialogs", () => {
    const { container, dispose } = renderComponent(() => (
      <PathsSettings preferences={preferences()} onUpdate={vi.fn()} />
    ));

    clickButton(container, "Browse");

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Folder browsing is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening native report logo dialogs", () => {
    const { container, dispose } = renderComponent(() => (
      <ReportsSettings preferences={preferences()} onUpdate={vi.fn()} />
    ));

    clickButton(container, "Browse");

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Logo file browsing is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening native profile logo dialogs", () => {
    const { container, dispose } = renderComponent(() => (
      <UserProfilesSettings preferences={preferences()} onUpdate={vi.fn()} />
    ));

    clickButton(container, "Add Profile");
    clickButton(container, "Browse");

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Logo file browsing is available in the desktop app.",
    );
    dispose();
  });
});
