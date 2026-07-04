import { describe, expect, it, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { open } from "@tauri-apps/plugin-dialog";
import { ImportAcquisitionsWizard } from "./ImportAcquisitionsWizard";

const importHookMocks = vi.hoisted(() => ({
  scan: vi.fn(),
  importSelected: vi.fn(),
  selected: vi.fn(),
  results: vi.fn(),
}));

vi.mock("../../hooks/useImportAcquisitions", () => ({
  useImportAcquisitions: () => ({
    scanning: () => false,
    results: importHookMocks.results,
    selected: importHookMocks.selected,
    importing: () => false,
    importResult: () => null,
    error: () => null,
    scan: importHookMocks.scan,
    toggleSelect: vi.fn(),
    selectAll: vi.fn(),
    deselectAll: vi.fn(),
    importSelected: importHookMocks.importSelected,
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
    vi.clearAllMocks();
    importHookMocks.results.mockReturnValue([]);
    importHookMocks.selected.mockReturnValue(new Set<string>());
    importHookMocks.scan.mockResolvedValue(undefined);
    importHookMocks.importSelected.mockResolvedValue({ imported: 0, skipped: 0, errors: [] });
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
      "Acquisition directory scanning is available in the desktop app.",
    );
    dispose();
  });

  it("does not scan typed acquisition paths in browser preview", async () => {
    const { container, dispose } = renderComponent(() => (
      <ImportAcquisitionsWizard onClose={vi.fn()} />
    ));

    const input = container.querySelector("input") as HTMLInputElement;
    input.value = "/cases/acquisitions";
    input.dispatchEvent(new InputEvent("input", { bubbles: true }));

    const scanButton = Array.from(container.querySelectorAll("button")).find((button) =>
      button.title === "Scan directory",
    );
    expect(scanButton).toBeDefined();
    scanButton!.click();

    await Promise.resolve();

    expect(importHookMocks.scan).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Acquisition directory scanning is available in the desktop app.",
    );
    dispose();
  });

  it("notifies callers after a successful import so project autosave can run", async () => {
    importHookMocks.results.mockReturnValue([
      {
        companionPath: "/cases/acq.ffx-companion.json",
        outputExists: true,
        companion: {
          version: "1",
          tool: "CORE Acquire",
          toolVersion: "1.0.0",
          createdAt: "2026-07-04T12:00:00.000Z",
          acquisitionType: "e01",
          source: {
            paths: ["/source/disk"],
            totalBytes: 1024,
          },
          output: {
            primaryPath: "/cases/disk.E01",
            format: "E01",
            totalBytes: 1024,
          },
          hashes: {},
          timing: {
            startedAt: "2026-07-04T12:00:00.000Z",
            completedAt: "2026-07-04T12:05:00.000Z",
          },
        },
      },
    ]);
    importHookMocks.selected.mockReturnValue(new Set(["/cases/acq.ffx-companion.json"]));
    importHookMocks.importSelected.mockResolvedValue({ imported: 1, skipped: 0, errors: [] });
    const onImportComplete = vi.fn();

    const { container, dispose } = renderComponent(() => (
      <ImportAcquisitionsWizard onClose={vi.fn()} onImportComplete={onImportComplete} />
    ));

    const importButton = Array.from(container.querySelectorAll("button")).find((button) =>
      button.textContent?.includes("Import 1 Acquisition"),
    );
    expect(importButton).toBeDefined();
    importButton!.click();

    await Promise.resolve();

    expect(importHookMocks.importSelected).toHaveBeenCalledOnce();
    expect(onImportComplete).toHaveBeenCalledWith({ imported: 1, skipped: 0, errors: [] });
    dispose();
  });
});
