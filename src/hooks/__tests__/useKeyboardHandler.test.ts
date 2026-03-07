// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot, createSignal, type Setter, type Accessor } from "solid-js";
import { useKeyboardHandler, type KeyboardHandlerDeps } from "../useKeyboardHandler";

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

// Mock useKeyDownEvent to return a controllable signal
let setKeyEvent: Setter<KeyboardEvent | null>;
let keyEventAccessor: Accessor<KeyboardEvent | null>;

vi.mock("@solid-primitives/keyboard", () => ({
  useKeyDownEvent: () => {
    return keyEventAccessor;
  },
}));

vi.mock("../../utils/accessibility", () => ({
  announce: vi.fn(),
}));

vi.mock("../../utils/telemetry", () => ({
  logError: vi.fn(),
}));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeEvent(key: string, opts: Partial<KeyboardEvent> = {}): KeyboardEvent {
  return {
    key,
    metaKey: false,
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    preventDefault: vi.fn(),
    target: { tagName: "DIV" } as unknown as EventTarget,
    ...opts,
  } as unknown as KeyboardEvent;
}

function meta(key: string, extra: Partial<KeyboardEvent> = {}): KeyboardEvent {
  return makeEvent(key, { metaKey: true, ...extra });
}

function createDeps(overrides: Partial<KeyboardHandlerDeps> = {}): KeyboardHandlerDeps {
  return {
    setShowCommandPalette: vi.fn((v: any) => (typeof v === "function" ? v(false) : v)) as unknown as Setter<boolean>,
    setShowSettingsPanel: vi.fn() as unknown as Setter<boolean>,
    setShowSearchPanel: vi.fn() as unknown as Setter<boolean>,
    setShowPerformancePanel: vi.fn() as unknown as Setter<boolean>,
    setShowShortcutsModal: vi.fn() as unknown as Setter<boolean>,
    setShowProjectWizard: vi.fn() as unknown as Setter<boolean>,
    showCommandPalette: () => false,
    showShortcutsModal: () => false,
    onLoadProject: vi.fn(),
    history: {
      state: {
        canUndo: () => true,
        canRedo: () => true,
        undoDescription: () => "Bookmark added",
        redoDescription: () => "Bookmark removed",
      },
      actions: {
        undo: vi.fn(),
        redo: vi.fn(),
      },
    },
    toast: {
      success: vi.fn(),
      error: vi.fn(),
      info: vi.fn(),
    },
    projectManager: {
      projectPath: () => null,
      hasProject: () => false,
      saveProject: vi.fn().mockResolvedValue({ success: true }),
      saveProjectAs: vi.fn().mockResolvedValue({ success: true }),
    },
    buildSaveOptions: vi.fn().mockReturnValue({ name: "test" }),
    ...overrides,
  };
}

function setup(depsOverrides: Partial<KeyboardHandlerDeps> = {}) {
  let dispose!: () => void;
  let deps: KeyboardHandlerDeps;
  createRoot((d) => {
    dispose = d;
    deps = createDeps(depsOverrides);
    useKeyboardHandler(deps);
  });
  return { deps: deps!, dispose };
}

function fireKey(event: KeyboardEvent) {
  setKeyEvent(event);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("useKeyboardHandler", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Create a fresh signal for each test
    createRoot(() => {
      const [acc, set] = createSignal<KeyboardEvent | null>(null);
      keyEventAccessor = acc;
      setKeyEvent = set;
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+K: Command palette toggle
  // -----------------------------------------------------------------------
  describe("Cmd+K — command palette", () => {
    it("toggles command palette", () => {
      const { deps, dispose } = setup();
      fireKey(meta("k"));

      expect(deps.setShowCommandPalette).toHaveBeenCalled();
      dispose();
    });

    it("prevents default", () => {
      const { dispose } = setup();
      const e = meta("k");
      fireKey(e);

      expect(e.preventDefault).toHaveBeenCalled();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+,: Settings
  // -----------------------------------------------------------------------
  describe("Cmd+comma — settings", () => {
    it("opens settings panel", () => {
      const { deps, dispose } = setup();
      fireKey(meta(","));

      expect(deps.setShowSettingsPanel).toHaveBeenCalledWith(true);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+F: Search
  // -----------------------------------------------------------------------
  describe("Cmd+F — search", () => {
    it("opens search panel", () => {
      const { deps, dispose } = setup({
        projectManager: {
          projectPath: () => "/test/project.cffx",
          hasProject: () => true,
          saveProject: vi.fn().mockResolvedValue({ success: true }),
          saveProjectAs: vi.fn().mockResolvedValue({ success: true }),
        },
      });
      fireKey(meta("f"));

      expect(deps.setShowSearchPanel).toHaveBeenCalledWith(true);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+Shift+N: New Project
  // -----------------------------------------------------------------------
  describe("Cmd+Shift+N — new project", () => {
    it("opens project wizard", () => {
      const { deps, dispose } = setup();
      fireKey(meta("n", { shiftKey: true }));

      expect(deps.setShowProjectWizard).toHaveBeenCalledWith(true);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+O: Open Project
  // -----------------------------------------------------------------------
  describe("Cmd+O — open project", () => {
    it("calls onLoadProject", () => {
      const { deps, dispose } = setup();
      fireKey(meta("o"));

      expect(deps.onLoadProject).toHaveBeenCalled();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Ctrl+Shift+P: Performance panel
  // -----------------------------------------------------------------------
  describe("Ctrl+Shift+P — performance panel", () => {
    it("toggles performance panel", () => {
      const { deps, dispose } = setup();
      fireKey(makeEvent("p", { ctrlKey: true, shiftKey: true }));

      expect(deps.setShowPerformancePanel).toHaveBeenCalled();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // ?: Shortcuts help
  // -----------------------------------------------------------------------
  describe("? — shortcuts help", () => {
    it("opens shortcuts modal when not in input", () => {
      const { deps, dispose } = setup();
      fireKey(makeEvent("?"));

      expect(deps.setShowShortcutsModal).toHaveBeenCalledWith(true);
      dispose();
    });

    it("does NOT open when typing in input", () => {
      const { deps, dispose } = setup();
      fireKey(makeEvent("?", { target: { tagName: "INPUT" } as unknown as EventTarget }));

      expect(deps.setShowShortcutsModal).not.toHaveBeenCalled();
      dispose();
    });

    it("does NOT open when typing in textarea", () => {
      const { deps, dispose } = setup();
      fireKey(makeEvent("?", { target: { tagName: "TEXTAREA" } as unknown as EventTarget }));

      expect(deps.setShowShortcutsModal).not.toHaveBeenCalled();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+Z: Undo
  // -----------------------------------------------------------------------
  describe("Cmd+Z — undo", () => {
    it("calls undo when canUndo is true", () => {
      const { deps, dispose } = setup();
      fireKey(meta("z"));

      expect(deps.history.actions.undo).toHaveBeenCalled();
      expect(deps.toast.info).toHaveBeenCalledWith("Undo", "Bookmark added");
      dispose();
    });

    it("does NOT call undo when canUndo is false", () => {
      const { deps, dispose } = setup({
        history: {
          state: {
            canUndo: () => false,
            canRedo: () => false,
            undoDescription: () => null,
            redoDescription: () => null,
          },
          actions: { undo: vi.fn(), redo: vi.fn() },
        },
      });
      fireKey(meta("z"));

      expect(deps.history.actions.undo).not.toHaveBeenCalled();
      dispose();
    });

    it("uses fallback description when undoDescription is null", () => {
      const { deps, dispose } = setup({
        history: {
          state: {
            canUndo: () => true,
            canRedo: () => false,
            undoDescription: () => null,
            redoDescription: () => null,
          },
          actions: { undo: vi.fn(), redo: vi.fn() },
        },
      });
      fireKey(meta("z"));

      expect(deps.toast.info).toHaveBeenCalledWith("Undo", "Action undone");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+Shift+Z / Cmd+Y: Redo
  // -----------------------------------------------------------------------
  describe("Cmd+Shift+Z — redo", () => {
    it("calls redo when canRedo is true", () => {
      const { deps, dispose } = setup();
      fireKey(meta("z", { shiftKey: true }));

      expect(deps.history.actions.redo).toHaveBeenCalled();
      expect(deps.toast.info).toHaveBeenCalledWith("Redo", "Bookmark removed");
      dispose();
    });

    it("does NOT call redo when canRedo is false", () => {
      const { deps, dispose } = setup({
        history: {
          state: {
            canUndo: () => false,
            canRedo: () => false,
            undoDescription: () => null,
            redoDescription: () => null,
          },
          actions: { undo: vi.fn(), redo: vi.fn() },
        },
      });
      fireKey(meta("z", { shiftKey: true }));

      expect(deps.history.actions.redo).not.toHaveBeenCalled();
      dispose();
    });
  });

  describe("Cmd+Y — redo (alternative)", () => {
    it("calls redo via Cmd+Y", () => {
      const { deps, dispose } = setup();
      fireKey(meta("y"));

      expect(deps.history.actions.redo).toHaveBeenCalled();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+S: Save
  // -----------------------------------------------------------------------
  describe("Cmd+S — save", () => {
    it("calls saveProjectAs when no project path exists", async () => {
      const { deps, dispose } = setup();
      fireKey(meta("s"));

      // Wait for async handler
      await vi.waitFor(() => {
        expect(deps.projectManager.saveProjectAs).toHaveBeenCalled();
      });
      dispose();
    });

    it("calls saveProject when project path exists", async () => {
      const { deps, dispose } = setup({
        projectManager: {
          projectPath: () => "/path/to/project.cffx",
          hasProject: () => true,
          saveProject: vi.fn().mockResolvedValue({ success: true }),
          saveProjectAs: vi.fn().mockResolvedValue({ success: true }),
        },
      });
      fireKey(meta("s"));

      await vi.waitFor(() => {
        expect(deps.projectManager.saveProject).toHaveBeenCalledWith(
          expect.anything(),
          "/path/to/project.cffx"
        );
      });
      dispose();
    });

    it("does nothing when buildSaveOptions returns null", () => {
      const { deps, dispose } = setup({
        buildSaveOptions: vi.fn().mockReturnValue(null),
      });
      fireKey(meta("s"));

      expect(deps.projectManager.saveProject).not.toHaveBeenCalled();
      expect(deps.projectManager.saveProjectAs).not.toHaveBeenCalled();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Cmd+Shift+S: Save As
  // -----------------------------------------------------------------------
  describe("Cmd+Shift+S — save as", () => {
    it("calls saveProjectAs", async () => {
      const { deps, dispose } = setup();
      fireKey(meta("s", { shiftKey: true }));

      await vi.waitFor(() => {
        expect(deps.projectManager.saveProjectAs).toHaveBeenCalled();
      });
      dispose();
    });

    it("shows error toast when no save options", () => {
      const { deps, dispose } = setup({
        buildSaveOptions: vi.fn().mockReturnValue(null),
      });
      fireKey(meta("s", { shiftKey: true }));

      expect(deps.toast.error).toHaveBeenCalledWith("No Evidence", "Open an evidence directory first");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Escape: Close modals
  // -----------------------------------------------------------------------
  describe("Escape — close modals", () => {
    it("closes command palette when open", () => {
      const { deps, dispose } = setup({
        showCommandPalette: () => true,
      });
      fireKey(makeEvent("Escape"));

      expect(deps.setShowCommandPalette).toHaveBeenCalledWith(false);
      dispose();
    });

    it("closes shortcuts modal when open (and palette is closed)", () => {
      const { deps, dispose } = setup({
        showCommandPalette: () => false,
        showShortcutsModal: () => true,
      });
      fireKey(makeEvent("Escape"));

      expect(deps.setShowShortcutsModal).toHaveBeenCalledWith(false);
      dispose();
    });

    it("closes command palette first (priority)", () => {
      const { deps, dispose } = setup({
        showCommandPalette: () => true,
        showShortcutsModal: () => true,
      });
      fireKey(makeEvent("Escape"));

      expect(deps.setShowCommandPalette).toHaveBeenCalledWith(false);
      expect(deps.setShowShortcutsModal).not.toHaveBeenCalled();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Edge cases
  // -----------------------------------------------------------------------
  describe("edge cases", () => {
    it("handles null event gracefully", () => {
      const { dispose } = setup();
      // null event is the initial state — should not throw
      expect(() => setKeyEvent(null)).not.toThrow();
      dispose();
    });

    it("ignores unrecognized keys", () => {
      const { deps, dispose } = setup();
      fireKey(meta("x"));

      expect(deps.setShowCommandPalette).not.toHaveBeenCalled();
      expect(deps.setShowSettingsPanel).not.toHaveBeenCalled();
      expect(deps.setShowSearchPanel).not.toHaveBeenCalled();
      dispose();
    });
  });
});
