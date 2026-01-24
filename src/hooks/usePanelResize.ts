// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * usePanelResize - Hook for managing resizable panels
 * 
 * Handles:
 * - Panel width state with min/max constraints
 * - Collapse/expand functionality with auto-collapse on small drag
 * - Mouse drag handling for resize
 */

import { createSignal, onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";

export interface PanelResizeOptions {
  /** Initial width of the panel */
  initialWidth: number;
  /** Minimum width before auto-collapse (default: 150) */
  minWidth?: number;
  /** Maximum width constraint (default: unlimited) */
  maxWidth?: number;
  /** Start collapsed */
  startCollapsed?: boolean;
  /** Which side of the screen ('left' | 'right') */
  side: "left" | "right";
}

export interface UsePanelResizeReturn {
  /** Current width of the panel */
  width: () => number;
  /** Set the width */
  setWidth: (width: number) => void;
  /** Whether the panel is collapsed */
  collapsed: () => boolean;
  /** Set collapsed state */
  setCollapsed: (collapsed: boolean | ((prev: boolean) => boolean)) => void;
  /** Toggle collapsed state */
  toggleCollapsed: () => void;
  /** Start dragging (call from onMouseDown) */
  startDrag: () => void;
  /** Whether currently dragging */
  isDragging: () => boolean;
}

/**
 * Hook for managing a single resizable panel
 */
export function usePanelResize(options: PanelResizeOptions): UsePanelResizeReturn {
  const {
    initialWidth,
    minWidth = 150,
    maxWidth = Infinity,
    startCollapsed = false,
    side,
  } = options;

  const [width, setWidthInternal] = createSignal(initialWidth);
  const [collapsed, setCollapsed] = createSignal(startCollapsed);
  const [dragging, setDragging] = createSignal(false);

  // Set width with constraints
  const setWidth = (newWidth: number) => {
    setWidthInternal(Math.max(minWidth, Math.min(maxWidth, newWidth)));
  };

  const toggleCollapsed = () => setCollapsed((v) => !v);

  const startDrag = () => setDragging(true);

  const isDragging = () => dragging();

  // Mouse move handler
  const handleMouseMove = (e: MouseEvent) => {
    if (!dragging()) return;

    let rawWidth: number;
    if (side === "left") {
      rawWidth = e.clientX;
    } else {
      rawWidth = window.innerWidth - e.clientX;
    }

    if (rawWidth < minWidth) {
      setCollapsed(true);
    } else {
      setCollapsed(false);
      setWidthInternal(Math.min(maxWidth, rawWidth));
    }
  };

  // Mouse up handler
  const handleMouseUp = () => {
    setDragging(false);
  };

  // Setup event listeners - makeEventListener auto-cleans up on component unmount
  onMount(() => {
    makeEventListener(window, "mousemove", handleMouseMove);
    makeEventListener(window, "mouseup", handleMouseUp);
  });

  return {
    width,
    setWidth,
    collapsed,
    setCollapsed,
    toggleCollapsed,
    startDrag,
    isDragging,
  };
}

export interface DualPanelResizeOptions {
  /** Left panel options */
  left: {
    initialWidth: number;
    minWidth?: number;
    maxWidth?: number;
    startCollapsed?: boolean;
  };
  /** Right panel options */
  right: {
    initialWidth: number;
    minWidth?: number;
    maxWidth?: number;
    startCollapsed?: boolean;
  };
}

export interface DualPanelState {
  width: () => number;
  setWidth: (width: number) => void;
  collapsed: () => boolean;
  setCollapsed: (collapsed: boolean | ((prev: boolean) => boolean)) => void;
  toggleCollapsed: () => void;
  startDrag: () => void;
}

export interface UseDualPanelResizeReturn {
  /** Left panel controls */
  left: DualPanelState;
  /** Right panel controls */
  right: DualPanelState;
  /** Whether either panel is being dragged */
  isDragging: () => boolean;
  /** Which panel is dragging (for CSS classes) */
  draggingPanel: () => 'left' | 'right' | null;
}

/**
 * Hook for managing dual left/right resizable panels
 * Uses a single set of event listeners for efficiency
 */
export function useDualPanelResize(options: DualPanelResizeOptions): UseDualPanelResizeReturn {
  const leftMinWidth = options.left.minWidth ?? 150;
  const leftMaxWidth = options.left.maxWidth ?? 600;
  const rightMinWidth = options.right.minWidth ?? 150;
  const rightMaxWidth = options.right.maxWidth ?? 500;

  // State
  const [leftWidth, setLeftWidthInternal] = createSignal(options.left.initialWidth);
  const [rightWidth, setRightWidthInternal] = createSignal(options.right.initialWidth);
  const [leftCollapsed, setLeftCollapsed] = createSignal(options.left.startCollapsed ?? false);
  const [rightCollapsed, setRightCollapsed] = createSignal(options.right.startCollapsed ?? false);
  const [draggingPanel, setDraggingPanel] = createSignal<'left' | 'right' | null>(null);

  // Width setters with constraints
  const setLeftWidth = (w: number) => setLeftWidthInternal(Math.max(leftMinWidth, Math.min(leftMaxWidth, w)));
  const setRightWidth = (w: number) => setRightWidthInternal(Math.max(rightMinWidth, Math.min(rightMaxWidth, w)));

  // Mouse handlers
  const handleMouseMove = (e: MouseEvent) => {
    const drag = draggingPanel();
    if (!drag) return;

    if (drag === 'left') {
      const rawWidth = e.clientX;
      if (rawWidth < leftMinWidth) {
        setLeftCollapsed(true);
      } else {
        setLeftCollapsed(false);
        setLeftWidthInternal(Math.min(leftMaxWidth, rawWidth));
      }
    } else {
      const rawWidth = window.innerWidth - e.clientX;
      if (rawWidth < rightMinWidth) {
        setRightCollapsed(true);
      } else {
        setRightCollapsed(false);
        setRightWidthInternal(Math.min(rightMaxWidth, rawWidth));
      }
    }
  };

  const handleMouseUp = () => setDraggingPanel(null);

  // Setup single set of event listeners - makeEventListener auto-cleans up on component unmount
  onMount(() => {
    makeEventListener(window, 'mousemove', handleMouseMove);
    makeEventListener(window, 'mouseup', handleMouseUp);
  });

  return {
    left: {
      width: leftWidth,
      setWidth: setLeftWidth,
      collapsed: leftCollapsed,
      setCollapsed: setLeftCollapsed,
      toggleCollapsed: () => setLeftCollapsed(v => !v),
      startDrag: () => !leftCollapsed() && setDraggingPanel('left'),
    },
    right: {
      width: rightWidth,
      setWidth: setRightWidth,
      collapsed: rightCollapsed,
      setCollapsed: setRightCollapsed,
      toggleCollapsed: () => setRightCollapsed(v => !v),
      startDrag: () => !rightCollapsed() && setDraggingPanel('right'),
    },
    isDragging: () => draggingPanel() !== null,
    draggingPanel,
  };
}
