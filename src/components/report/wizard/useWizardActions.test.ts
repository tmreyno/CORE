import { describe, expect, it, vi } from "vitest";
import { save } from "@tauri-apps/plugin-dialog";
import { mockInvoke } from "../../../__tests__/setup";

vi.mock("solid-js", async (importOriginal) => {
  const actual = await importOriginal<typeof import("solid-js")>();
  return {
    ...actual,
    onMount: vi.fn(),
  };
});

import { useWizardActions } from "./useWizardActions";

const BROWSER_EXPORT_MESSAGE = "Report export is available in the desktop app.";

function createState() {
  return {
    activeSteps: () => [{ id: "export" }],
    currentStep: () => "export",
    setCurrentStep: vi.fn(),
    setSelectedPreset: vi.fn(),
    setMetadata: vi.fn(),
    setEnabledSections: vi.fn(),
    setPreviewLoading: vi.fn(),
    setPreviewHtml: vi.fn(),
    setExporting: vi.fn(),
    setExportError: vi.fn(),
  };
}

const props = {
  files: [],
  fileInfoMap: new Map(),
  fileHashMap: new Map(),
  onClose: vi.fn(),
};

describe("useWizardActions browser export guards", () => {
  it("sets an export error instead of opening a native report save dialog in browser preview", async () => {
    const state = createState();
    const actions = useWizardActions(state as any, props);

    await actions.exportReport();

    expect(save).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
    expect(state.setExportError).toHaveBeenCalledWith(BROWSER_EXPORT_MESSAGE);
    expect(state.setExporting).toHaveBeenLastCalledWith(false);
  });

  it("sets an export error instead of opening a native standard package save dialog in browser preview", async () => {
    const state = createState();
    const actions = useWizardActions(state as any, props);

    await actions.exportStandardPackage();

    expect(save).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
    expect(state.setExportError).toHaveBeenCalledWith(BROWSER_EXPORT_MESSAGE);
    expect(state.setExporting).toHaveBeenLastCalledWith(false);
  });
});
