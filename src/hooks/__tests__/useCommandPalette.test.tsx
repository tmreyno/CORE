import { describe, expect, it, vi } from "vitest";
import { createCommandPaletteActions } from "../useCommandPalette";

function makeConfig(overrides: Record<string, unknown> = {}) {
  return {
    fileManager: {
      browseScanDir: vi.fn(),
      scanForFiles: vi.fn(),
      activeFile: () => null,
      discoveredFiles: () => [],
    },
    hashManager: {
      hashSingleFile: vi.fn(),
      hashAllFiles: vi.fn(),
    },
    setCurrentViewMode: vi.fn(),
    setLeftCollapsed: vi.fn(),
    setRightCollapsed: vi.fn(),
    setShowReportWizard: vi.fn(),
    setShowSettingsPanel: vi.fn(),
    setShowShortcutsModal: vi.fn(),
    setShowProjectWizard: vi.fn(),
    setShowSearchPanel: vi.fn(),
    hasProject: () => false,
    ...overrides,
  } as any;
}

describe("createCommandPaletteActions", () => {
  it("exposes New Project and Open Project file actions", () => {
    const onNewProject = vi.fn();
    const onOpenProject = vi.fn();
    const actions = createCommandPaletteActions(makeConfig({
      onNewProject,
      onOpenProject,
    }))();

    const newProject = actions.find((action) => action.id === "new-project");
    const openProject = actions.find((action) => action.id === "open-project");

    expect(newProject).toMatchObject({
      label: "New Project",
      category: "File",
      shortcut: "cmd+shift+n",
    });
    expect(openProject).toMatchObject({
      label: "Open Project",
      category: "File",
      shortcut: "cmd+o",
    });

    newProject!.onSelect();
    openProject!.onSelect();

    expect(onNewProject).toHaveBeenCalledTimes(1);
    expect(onOpenProject).toHaveBeenCalledTimes(1);
  });

  it("falls back to opening the project wizard when no unified New Project handler is provided", () => {
    const setShowProjectWizard = vi.fn();
    const actions = createCommandPaletteActions(makeConfig({
      setShowProjectWizard,
    }))();

    actions.find((action) => action.id === "new-project")!.onSelect();

    expect(setShowProjectWizard).toHaveBeenCalledWith(true);
  });
});
