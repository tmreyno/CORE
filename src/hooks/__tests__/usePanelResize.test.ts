// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot } from "solid-js";
import {
  usePanelResize,
  useDualPanelResize,
  type PanelResizeOptions,
  type DualPanelResizeOptions,
} from "../usePanelResize";

// ---------------------------------------------------------------------------
// Mock makeEventListener — capture handlers instead of attaching to real DOM
// ---------------------------------------------------------------------------

let capturedListeners: Record<string, Function[]> = {};

vi.mock("@solid-primitives/event-listener", () => ({
  makeEventListener: (_target: unknown, event: string, handler: Function) => {
    if (!capturedListeners[event]) capturedListeners[event] = [];
    capturedListeners[event].push(handler);
  },
}));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function createHook<T>(factory: () => T): { hook: T; dispose: () => void } {
  let hook!: T;
  let dispose!: () => void;
  // createRoot triggers onMount synchronously in the test environment
  createRoot((d) => {
    dispose = d;
    hook = factory();
  });
  return { hook, dispose };
}

function fireMouseMove(clientX: number) {
  const handlers = capturedListeners["mousemove"] || [];
  const event = { clientX } as MouseEvent;
  handlers.forEach((h) => h(event));
}

function fireMouseUp() {
  const handlers = capturedListeners["mouseup"] || [];
  handlers.forEach((h) => h());
}

// Mock window.innerWidth for right-side calculations
const WINDOW_WIDTH = 1200;

beforeEach(() => {
  capturedListeners = {};
  vi.stubGlobal("innerWidth", WINDOW_WIDTH);
});

// ==========================================================================
// usePanelResize
// ==========================================================================

describe("usePanelResize", () => {
  const defaultOpts: PanelResizeOptions = {
    initialWidth: 300,
    minWidth: 150,
    maxWidth: 600,
    side: "left",
  };

  describe("initial state", () => {
    it("uses initial width", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      expect(hook.width()).toBe(300);
      dispose();
    });

    it("starts expanded by default", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      expect(hook.collapsed()).toBe(false);
      dispose();
    });

    it("can start collapsed", () => {
      const { hook, dispose } = createHook(() =>
        usePanelResize({ ...defaultOpts, startCollapsed: true })
      );
      expect(hook.collapsed()).toBe(true);
      dispose();
    });

    it("is not dragging initially", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      expect(hook.isDragging()).toBe(false);
      dispose();
    });
  });

  describe("setWidth", () => {
    it("sets width within constraints", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.setWidth(400);
      expect(hook.width()).toBe(400);
      dispose();
    });

    it("clamps to minWidth", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.setWidth(50);
      expect(hook.width()).toBe(150);
      dispose();
    });

    it("clamps to maxWidth", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.setWidth(1000);
      expect(hook.width()).toBe(600);
      dispose();
    });
  });

  describe("collapse / expand", () => {
    it("toggleCollapsed toggles state", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      expect(hook.collapsed()).toBe(false);

      hook.toggleCollapsed();
      expect(hook.collapsed()).toBe(true);

      hook.toggleCollapsed();
      expect(hook.collapsed()).toBe(false);
      dispose();
    });

    it("setCollapsed with boolean value", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.setCollapsed(true);
      expect(hook.collapsed()).toBe(true);
      hook.setCollapsed(false);
      expect(hook.collapsed()).toBe(false);
      dispose();
    });

    it("setCollapsed with updater function", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.setCollapsed((prev) => !prev);
      expect(hook.collapsed()).toBe(true);
      dispose();
    });
  });

  describe("drag — left panel", () => {
    it("startDrag enables dragging", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.startDrag();
      expect(hook.isDragging()).toBe(true);
      dispose();
    });

    it("mousemove updates width when dragging", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.startDrag();

      fireMouseMove(400);

      expect(hook.width()).toBe(400);
      expect(hook.collapsed()).toBe(false);
      dispose();
    });

    it("mousemove does nothing when not dragging", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));

      fireMouseMove(400);

      expect(hook.width()).toBe(300); // unchanged
      dispose();
    });

    it("auto-collapses when dragged below minWidth", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.startDrag();

      fireMouseMove(100); // below minWidth=150

      expect(hook.collapsed()).toBe(true);
      dispose();
    });

    it("expands from collapsed when dragged above minWidth", () => {
      const { hook, dispose } = createHook(() =>
        usePanelResize({ ...defaultOpts, startCollapsed: true })
      );
      hook.startDrag();

      fireMouseMove(250);

      expect(hook.collapsed()).toBe(false);
      expect(hook.width()).toBe(250);
      dispose();
    });

    it("clamps to maxWidth during drag", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.startDrag();

      fireMouseMove(800); // above maxWidth=600

      expect(hook.width()).toBe(600);
      dispose();
    });

    it("mouseup stops dragging", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.startDrag();
      expect(hook.isDragging()).toBe(true);

      fireMouseUp();

      expect(hook.isDragging()).toBe(false);
      dispose();
    });

    it("mousemove after mouseup has no effect", () => {
      const { hook, dispose } = createHook(() => usePanelResize(defaultOpts));
      hook.startDrag();
      fireMouseMove(400);
      fireMouseUp();

      fireMouseMove(500);

      expect(hook.width()).toBe(400); // unchanged after mouseup
      dispose();
    });
  });

  describe("drag — right panel", () => {
    const rightOpts: PanelResizeOptions = {
      initialWidth: 300,
      minWidth: 150,
      maxWidth: 500,
      side: "right",
    };

    it("calculates right panel width as innerWidth - clientX", () => {
      const { hook, dispose } = createHook(() => usePanelResize(rightOpts));
      hook.startDrag();

      // clientX=900, window=1200 → rawWidth=300
      fireMouseMove(900);

      expect(hook.width()).toBe(300);
      dispose();
    });

    it("auto-collapses right panel below minWidth", () => {
      const { hook, dispose } = createHook(() => usePanelResize(rightOpts));
      hook.startDrag();

      // clientX=1100, window=1200 → rawWidth=100 < 150
      fireMouseMove(1100);

      expect(hook.collapsed()).toBe(true);
      dispose();
    });

    it("clamps right panel to maxWidth", () => {
      const { hook, dispose } = createHook(() => usePanelResize(rightOpts));
      hook.startDrag();

      // clientX=200, window=1200 → rawWidth=1000 > 500
      fireMouseMove(200);

      expect(hook.width()).toBe(500);
      dispose();
    });
  });

  describe("default constraints", () => {
    it("uses minWidth=150 by default", () => {
      const { hook, dispose } = createHook(() =>
        usePanelResize({ initialWidth: 300, side: "left" })
      );
      hook.setWidth(50);
      expect(hook.width()).toBe(150);
      dispose();
    });

    it("uses unlimited maxWidth by default", () => {
      const { hook, dispose } = createHook(() =>
        usePanelResize({ initialWidth: 300, side: "left" })
      );
      hook.setWidth(5000);
      expect(hook.width()).toBe(5000);
      dispose();
    });
  });
});

// ==========================================================================
// useDualPanelResize
// ==========================================================================

describe("useDualPanelResize", () => {
  const defaultOpts: DualPanelResizeOptions = {
    left: { initialWidth: 280, minWidth: 150, maxWidth: 600 },
    right: { initialWidth: 320, minWidth: 150, maxWidth: 500 },
  };

  describe("initial state", () => {
    it("sets left panel width", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      expect(hook.left.width()).toBe(280);
      dispose();
    });

    it("sets right panel width", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      expect(hook.right.width()).toBe(320);
      dispose();
    });

    it("both panels start expanded by default", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      expect(hook.left.collapsed()).toBe(false);
      expect(hook.right.collapsed()).toBe(false);
      dispose();
    });

    it("panels can start collapsed", () => {
      const { hook, dispose } = createHook(() =>
        useDualPanelResize({
          left: { ...defaultOpts.left, startCollapsed: true },
          right: { ...defaultOpts.right, startCollapsed: true },
        })
      );
      expect(hook.left.collapsed()).toBe(true);
      expect(hook.right.collapsed()).toBe(true);
      dispose();
    });

    it("not dragging initially", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      expect(hook.isDragging()).toBe(false);
      expect(hook.draggingPanel()).toBeNull();
      dispose();
    });
  });

  describe("setWidth constraints", () => {
    it("left panel clamps to min/max", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.setWidth(50);
      expect(hook.left.width()).toBe(150);

      hook.left.setWidth(1000);
      expect(hook.left.width()).toBe(600);
      dispose();
    });

    it("right panel clamps to min/max", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.right.setWidth(50);
      expect(hook.right.width()).toBe(150);

      hook.right.setWidth(1000);
      expect(hook.right.width()).toBe(500);
      dispose();
    });
  });

  describe("toggle", () => {
    it("toggles left panel", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.toggleCollapsed();
      expect(hook.left.collapsed()).toBe(true);

      hook.left.toggleCollapsed();
      expect(hook.left.collapsed()).toBe(false);
      dispose();
    });

    it("toggles right panel", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.right.toggleCollapsed();
      expect(hook.right.collapsed()).toBe(true);
      dispose();
    });
  });

  describe("drag — left panel", () => {
    it("startDrag sets dragging state to left", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.startDrag();

      expect(hook.isDragging()).toBe(true);
      expect(hook.draggingPanel()).toBe("left");
      dispose();
    });

    it("collapsed left panel cannot start drag", () => {
      const { hook, dispose } = createHook(() =>
        useDualPanelResize({
          ...defaultOpts,
          left: { ...defaultOpts.left, startCollapsed: true },
        })
      );
      hook.left.startDrag();

      expect(hook.isDragging()).toBe(false);
      expect(hook.draggingPanel()).toBeNull();
      dispose();
    });

    it("dragging left panel updates width via clientX", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.startDrag();

      fireMouseMove(400);

      expect(hook.left.width()).toBe(400);
      expect(hook.left.collapsed()).toBe(false);
      dispose();
    });

    it("dragging left below min auto-collapses", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.startDrag();

      fireMouseMove(100);

      expect(hook.left.collapsed()).toBe(true);
      dispose();
    });

    it("dragging left clamps to maxWidth", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.startDrag();

      fireMouseMove(800);

      expect(hook.left.width()).toBe(600);
      dispose();
    });

    it("dragging left does not affect right panel", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.startDrag();
      fireMouseMove(400);

      expect(hook.right.width()).toBe(320); // unchanged
      dispose();
    });
  });

  describe("drag — right panel", () => {
    it("startDrag sets dragging state to right", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.right.startDrag();

      expect(hook.isDragging()).toBe(true);
      expect(hook.draggingPanel()).toBe("right");
      dispose();
    });

    it("collapsed right panel cannot start drag", () => {
      const { hook, dispose } = createHook(() =>
        useDualPanelResize({
          ...defaultOpts,
          right: { ...defaultOpts.right, startCollapsed: true },
        })
      );
      hook.right.startDrag();

      expect(hook.isDragging()).toBe(false);
      dispose();
    });

    it("dragging right panel uses innerWidth - clientX", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.right.startDrag();

      // clientX=850, window=1200 → rawWidth=350
      fireMouseMove(850);

      expect(hook.right.width()).toBe(350);
      expect(hook.right.collapsed()).toBe(false);
      dispose();
    });

    it("dragging right below min auto-collapses", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.right.startDrag();

      // clientX=1100, window=1200 → rawWidth=100 < 150
      fireMouseMove(1100);

      expect(hook.right.collapsed()).toBe(true);
      dispose();
    });

    it("dragging right clamps to maxWidth", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.right.startDrag();

      // clientX=200, window=1200 → rawWidth=1000 > 500
      fireMouseMove(200);

      expect(hook.right.width()).toBe(500);
      dispose();
    });

    it("dragging right does not affect left panel", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.right.startDrag();
      fireMouseMove(850);

      expect(hook.left.width()).toBe(280); // unchanged
      dispose();
    });
  });

  describe("mouseup", () => {
    it("stops left panel drag", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.startDrag();
      fireMouseUp();

      expect(hook.isDragging()).toBe(false);
      expect(hook.draggingPanel()).toBeNull();
      dispose();
    });

    it("stops right panel drag", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.right.startDrag();
      fireMouseUp();

      expect(hook.isDragging()).toBe(false);
      dispose();
    });

    it("mousemove after mouseup has no effect", () => {
      const { hook, dispose } = createHook(() => useDualPanelResize(defaultOpts));
      hook.left.startDrag();
      fireMouseMove(400);
      fireMouseUp();

      fireMouseMove(500);

      expect(hook.left.width()).toBe(400);
      dispose();
    });
  });

  describe("default constraints", () => {
    it("uses default min=150 and max values", () => {
      const { hook, dispose } = createHook(() =>
        useDualPanelResize({
          left: { initialWidth: 200 },
          right: { initialWidth: 200 },
        })
      );

      hook.left.setWidth(50);
      expect(hook.left.width()).toBe(150);

      hook.left.setWidth(1000);
      expect(hook.left.width()).toBe(600); // default left maxWidth=600

      hook.right.setWidth(50);
      expect(hook.right.width()).toBe(150);

      hook.right.setWidth(1000);
      expect(hook.right.width()).toBe(500); // default right maxWidth=500

      dispose();
    });
  });
});
