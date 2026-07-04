import { describe, expect, it, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { createSignal } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import AcquireVerifyView from "./AcquireVerifyView";
import AcquireExportView from "./AcquireExportView";
import StartSessionDialog from "./StartSessionDialog";
import AcquireDashboard from "./AcquireDashboard";
import AcquireLayout, { type AcquireView } from "./AcquireLayout";
import type { ExportMode } from "../../hooks/export/types";

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

function buttonByTitle(container: HTMLElement, title: string): HTMLButtonElement {
  const button = container.querySelector(`button[title="${title}"]`);
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

  it("allows manual output folder entry instead of opening a native session folder picker", () => {
    const { container, dispose } = renderComponent(() => (
      <StartSessionDialog isOpen onClose={vi.fn()} onCreate={vi.fn()} defaultExaminer="examiner" />
    ));

    buttonByText(container, "Browse").click();

    const outputInput = container.querySelector('input[placeholder="Select folder..."]') as HTMLInputElement;
    expect(open).not.toHaveBeenCalled();
    expect(outputInput.readOnly).toBe(false);
    expect(container.textContent).toContain(
      "Output folder browsing is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening a native dashboard destination picker", () => {
    const { container, dispose } = renderComponent(() => (
      <AcquireDashboard
        onAction={vi.fn()}
        onSettings={vi.fn()}
        onHelp={vi.fn()}
        onCommandPalette={vi.fn()}
        onOpenProject={vi.fn()}
        onNewProject={vi.fn()}
        projectName={() => "Case 1827"}
        hasProject={() => true}
        evidenceCount={() => 0}
        isPortable={() => false}
        portableConfig={() => null}
      />
    ));

    buttonByTitle(container, "Choose output folder").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Output folder browsing is available in the desktop app.",
    );
    dispose();
  });

  it("shows a browser-preview message instead of opening the native quick verify picker", () => {
    const [view, setView] = createSignal<AcquireView>("dashboard");
    const [mode, setMode] = createSignal<ExportMode>("native");
    const { container, dispose } = renderComponent(() => (
      <AcquireLayout
        onSettings={vi.fn()}
        onHelp={vi.fn()}
        onCommandPalette={vi.fn()}
        onOpenProject={vi.fn()}
        onNewProject={vi.fn()}
        projectName={() => "Case 1827"}
        hasProject={() => true}
        evidenceCount={() => 0}
        initialSources={() => []}
        initialExaminerName={() => "examiner"}
        onExportComplete={vi.fn()}
        onActivityCreate={vi.fn()}
        onActivityUpdate={vi.fn()}
        onVerifyHashes={vi.fn()}
        acquireView={view}
        setAcquireView={setView}
        initialExportMode={mode}
        setInitialExportMode={setMode}
        isPortable={() => false}
        portableConfig={() => null}
        initialSystemStats={{
          osName: "macOS",
          cpuCores: 8,
          memoryTotal: 16 * 1024 * 1024 * 1024,
        } as any}
        initialDrives={[]}
      />
    ));

    buttonByTitle(container, "Quick Hash File").click();

    expect(open).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Quick hash file selection is available in the desktop app.",
    );
    dispose();
  });
});
