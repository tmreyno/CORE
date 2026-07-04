import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import { useWizardState } from "./useWizardState";

const BROWSER_DISCOVERY_MESSAGE =
  "Project folder scanning is available in the desktop app. In browser preview, use Open Project to load a .cffx file.";

function createState(overrides: Partial<Parameters<typeof useWizardState>[0]> = {}) {
  let state!: ReturnType<typeof useWizardState>;
  let dispose!: () => void;
  const onComplete = vi.fn();

  createRoot((d) => {
    dispose = d;
    state = useWizardState({
      isOpen: false,
      projectRoot: "",
      onClose: vi.fn(),
      onComplete,
      ...overrides,
    });
  });

  return { state, dispose, onComplete };
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("useWizardState browser runtime guards", () => {
  it("does not invoke backend discovery commands for direct evidence/database discovery", async () => {
    const { state, dispose } = createState();

    await expect(state.discoverEvidence("/case/root")).resolves.toEqual([]);
    await expect(state.discoverDatabases("/case/root")).resolves.toEqual([]);

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(state.discoveredEvidence()).toEqual([]);
    expect(state.discoveredDatabases()).toEqual([]);
    expect(state.error()).toBe(BROWSER_DISCOVERY_MESSAGE);
    dispose();
  });

  it("does not invoke backend auto-discovery commands when opened with a project root", async () => {
    const { state, dispose } = createState({
      isOpen: true,
      projectRoot: "/case/root",
    });

    await tick();
    await tick();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(state.step()).toBe(1);
    expect(state.scanning()).toBe(false);
    expect(state.evidencePath()).toBe("/case/root");
    expect(state.processedDbPath()).toBe("/case/root");
    expect(state.caseDocumentsPath()).toBe("/case/root");
    expect(state.error()).toBe(BROWSER_DISCOVERY_MESSAGE);
    dispose();
  });

  it("routes browser folder selection to Open Project when a fallback is provided", async () => {
    const onClose = vi.fn();
    const onOpenProject = vi.fn();
    const { state, dispose } = createState({
      isOpen: true,
      onClose,
      onOpenProject,
    });

    await state.browseProjectRoot();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(onOpenProject).toHaveBeenCalledTimes(1);
    expect(state.error()).toBeNull();
    dispose();
  });
});
