import { describe, expect, it, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { createSignal } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import AcquireVerifyView from "./AcquireVerifyView";
import AcquireExportView from "./AcquireExportView";

vi.mock("../export-panel/DriveTreeBrowser", () => ({
  DriveTreeBrowser: () => <div data-testid="drive-tree-browser">Drive tree</div>,
}));

vi.mock("../export-panel/ExportPanelComponent", () => ({
  ExportPanelComponent: () => <div data-testid="export-panel">Export panel</div>,
}));

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

describe("acquisition browser dialog guards", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("shows a browser-preview message instead of opening native verify file pickers", () => {
    const { container, dispose } = renderComponent(() => (
      <AcquireVerifyView
        onBack={vi.fn()}
        onHashAll={vi.fn()}
        evidenceCount={() => 0}
        hasProject={() => false}
      />
    ));

    buttonByText(container, "Add Files").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Hash verification file and folder selection is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening native verify folder pickers", () => {
    const { container, dispose } = renderComponent(() => (
      <AcquireVerifyView
        onBack={vi.fn()}
        onHashAll={vi.fn()}
        evidenceCount={() => 0}
        hasProject={() => false}
      />
    ));

    buttonByText(container, "Add Folder").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Hash verification file and folder selection is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening native acquire source pickers", () => {
    const [sources] = createSignal<string[]>([]);
    const [examiner] = createSignal<string | undefined>("examiner");
    const [mode] = createSignal("native" as const);
    const { container, dispose } = renderComponent(() => (
      <AcquireExportView
        onBack={vi.fn()}
        initialSources={sources}
        initialExaminerName={examiner}
        initialMode={mode}
        onComplete={vi.fn()}
        onActivityCreate={vi.fn()}
        onActivityUpdate={vi.fn()}
      />
    ));

    buttonByText(container, "Files").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Acquire source file and folder selection is available in the desktop app.",
    );

    buttonByText(container, "Folder").click();
    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Acquire source file and folder selection is available in the desktop app.",
    );
    dispose();
  });
});
